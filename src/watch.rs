//! `OpenPaths`: the one registry of the folders and files IVE shows, and the only code that talks to `notify`.
//! Everything that shows a path registers it here; registrations are counted per OS path, so a folder shown twice
//! has one OS watch, removed with its last registration.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};

use notify::event::{EventKind, ModifyKind, RenameMode};
use notify::{Event, RecursiveMode, Watcher};

/// One registrant: a project's tree, the open documents, a project's git dir.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct WatchOwner(u64);

/// What a registrant shows; paths are canonical.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Watched {
    /// a folder whose entries are shown
    Folder(PathBuf),
    /// a file shown in a tab; its parent folder is watched, events filtered to its name
    File(PathBuf),
    /// a repository's common git dir
    GitDir(PathBuf),
}

/// What changed on disk, as IVE acts on it.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Change {
    /// an entry was created, removed, renamed or written in this watched folder
    Folder(PathBuf),
    /// this watched file was written, and exists
    File(PathBuf),
    /// this watched file was removed or renamed away, and does not exist
    FileRemoved(PathBuf),
    /// something changed in this git dir
    Git(PathBuf),
    /// something changed inside this checkout's folder
    WorkTree(PathBuf),
    /// a `.favicon.*` file changed inside this checkout's folder: a project's or module's icon
    Favicon(PathBuf),
    /// events were lost: re-read everything
    Rescan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Flat,
    Recursive,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WatchOp {
    Watch(PathBuf, Mode),
    Unwatch(PathBuf),
}

/// OS watches; a fake in tests.
pub trait Backend {
    /// Applies one batch in order; returns each path it could not watch, with the reason.
    fn apply(&mut self, batch: &[WatchOp]) -> Vec<(PathBuf, String)>;

    /// Whether changing watches can lose events meanwhile (macOS: FSEvents restarts its one stream).
    fn loses_events_on_change(&self) -> bool;
}

/// Registrations needing an OS watch on one path.
#[derive(Debug, Clone, Copy, Default)]
struct Counts {
    flat: usize,
    recursive: usize,
}

impl Counts {
    fn mode(self) -> Option<Mode> {
        match self {
            Self { recursive: 1.., .. } => Some(Mode::Recursive),
            Self { flat: 1.., .. } => Some(Mode::Flat),
            _ => None,
        }
    }

    fn count(&mut self, mode: Mode, delta: isize) {
        let slot = match mode {
            Mode::Flat => &mut self.flat,
            Mode::Recursive => &mut self.recursive,
        };
        *slot = slot.saturating_add_signed(delta);
    }
}

pub struct OpenPaths {
    backend: Box<dyn Backend>,
    next_owner: u64,
    /// each owner's registrations with the OS paths each one needs
    owners: HashMap<WatchOwner, HashMap<Watched, Vec<(PathBuf, Mode)>>>,
    os: HashMap<PathBuf, Counts>,
    /// OS paths that could not be watched, with the reason; polled instead
    failed: BTreeMap<PathBuf, String>,
    /// folders to re-read because a batch may have lost their events
    lost: BTreeSet<PathBuf>,
    /// OS paths that were removed while registered, each with the existing ancestor watched until they are back
    waiting: BTreeMap<PathBuf, PathBuf>,
}

impl OpenPaths {
    pub fn new(backend: Box<dyn Backend>) -> Self {
        Self {
            backend,
            next_owner: 0,
            owners: HashMap::new(),
            os: HashMap::new(),
            failed: BTreeMap::new(),
            lost: BTreeSet::new(),
            waiting: BTreeMap::new(),
        }
    }

    pub fn new_owner(&mut self) -> WatchOwner {
        self.next_owner += 1;
        WatchOwner(self.next_owner)
    }

    pub fn remove_owner(&mut self, owner: WatchOwner) {
        self.set(owner, Vec::new());
        self.owners.remove(&owner);
    }

    /// Makes `wanted` all the owner registers, applying the difference as one batch: owners state what they
    /// show, rather than adding and removing paths one by one.
    pub fn set(&mut self, owner: WatchOwner, wanted: impl IntoIterator<Item = Watched>) {
        let wanted: HashSet<Watched> = wanted.into_iter().collect();
        let current = self.owners.remove(&owner).unwrap_or_default();
        let mut deltas = Vec::new();
        let mut kept = HashMap::new();
        for (watched, paths) in current {
            if wanted.contains(&watched) {
                kept.insert(watched, paths);
            } else {
                deltas.extend(paths.into_iter().map(|(path, mode)| (path, mode, -1)));
            }
        }
        for watched in wanted {
            if kept.contains_key(&watched) {
                continue;
            }
            let paths = os_paths(&watched);
            deltas.extend(paths.iter().cloned().map(|(path, mode)| (path, mode, 1)));
            kept.insert(watched, paths);
        }
        self.owners.insert(owner, kept);
        self.apply(deltas);
    }

    /// Re-reads which parts of git dir `dir` exist (`worktrees/` appears with the first linked worktree).
    pub fn renew_git_dir(&mut self, dir: &Path) {
        let watched = Watched::GitDir(dir.to_path_buf());
        let fresh = os_paths(&watched);
        let mut deltas = Vec::new();
        for registrations in self.owners.values_mut() {
            if let Some(paths) = registrations.get_mut(&watched) {
                deltas.extend(paths.drain(..).map(|(path, mode)| (path, mode, -1)));
                deltas.extend(fresh.iter().cloned().map(|(path, mode)| (path, mode, 1)));
                paths.clone_from(&fresh);
            }
        }
        self.apply(deltas);
    }

    /// Turns raw events into changes, without duplicates; `checkouts` are the folders of all open checkouts. A
    /// watched file is `File` when it exists afterwards and `FileRemoved` when not, whatever order the events of
    /// the batch came in. Watched folders that were removed are watched again once they are back.
    pub fn changes(&mut self, events: &[Event], checkouts: &[PathBuf]) -> Vec<Change> {
        if events.iter().any(Event::need_rescan) {
            return vec![Change::Rescan];
        }
        let back = self.rearm(events);
        let registered = self.all_registered();
        let mut changes = Vec::new();
        for event in events {
            for (path, _) in touched(event) {
                classify(path, &registered, checkouts, &mut changes);
            }
        }
        changes.extend(self.changes_needing(&back));
        settle(changes)
    }

    /// Changes standing for the registrations the OS could not watch, to apply on each poll.
    pub fn poll_changes(&self) -> Vec<Change> {
        let failed: Vec<PathBuf> = self.failed.keys().cloned().collect();
        settle(self.changes_needing(&failed))
    }

    /// Folders whose events a batch may have lost; empty after the call.
    pub fn take_lost(&mut self) -> Vec<Change> {
        std::mem::take(&mut self.lost).into_iter().map(Change::Folder).collect()
    }

    /// One line per path that is polled instead of watched.
    pub fn failures(&self) -> Vec<String> {
        self.failed.iter().map(|(path, reason)| failure_message(path, reason)).collect()
    }

    /// An error notify reports while watching (e.g. no inotify watch left for a folder created under a
    /// recursive watch): the watched paths it names are polled from now on; one naming none is only reported.
    pub fn note_error(&mut self, error: &notify::Error) {
        let reason = reason_of(error);
        let covering: BTreeSet<PathBuf> = error.paths.iter().filter_map(|path| self.os_path_holding(path)).collect();
        if covering.is_empty() {
            eprintln!("ive: file notifications: {error}");
        }
        for path in covering {
            self.note_failure(path, reason.clone());
        }
    }

    fn all_registered(&self) -> BTreeSet<&Watched> {
        self.owners.values().flat_map(HashMap::keys).collect()
    }

    /// The registrations that need one of the OS paths `paths`, as the changes that re-read them.
    fn changes_needing(&self, paths: &[PathBuf]) -> Vec<Change> {
        let needs = |watched: &&Watched| os_paths(watched).iter().any(|(path, _)| paths.contains(path));
        let changes = self.all_registered().into_iter().filter(needs).map(|watched| match watched {
            Watched::Folder(path) => Change::Folder(path.clone()),
            Watched::File(path) => Change::File(path.clone()),
            Watched::GitDir(path) => Change::Git(path.clone()),
        });
        changes.collect()
    }

    /// The deepest watched OS path at or above `path`.
    fn os_path_holding(&self, path: &Path) -> Option<PathBuf> {
        path.ancestors().find(|ancestor| self.os.contains_key(*ancestor)).map(Path::to_path_buf)
    }

    /// How `path` is watched now: recursively, flat, or not at all, which includes a flat registration a
    /// recursive watch above it already covers (watching it too would replace that watch's own on inotify).
    fn effective(&self, path: &Path) -> Option<Mode> {
        let mode = self.os.get(path)?.mode()?;
        let covered = || path.ancestors().skip(1).any(|ancestor| self.os.get(ancestor).is_some_and(|counts| counts.recursive > 0));
        match mode {
            Mode::Flat if covered() => None,
            mode => Some(mode),
        }
    }

    /// Counts the deltas in and sends the OS watches whose mode changed as one batch.
    fn apply(&mut self, deltas: Vec<(PathBuf, Mode, isize)>) {
        // a recursive watch coming or going changes how the paths below it are watched
        let below: Vec<PathBuf> =
            self.os.keys().filter(|path| deltas.iter().any(|(changed, _, _)| path.starts_with(changed))).cloned().collect();
        let mut before: BTreeMap<PathBuf, Option<Mode>> = below.into_iter().map(|path| (path.clone(), self.effective(&path))).collect();
        for (path, _, _) in &deltas {
            before.entry(path.clone()).or_insert_with(|| self.effective(path));
        }
        for (path, mode, delta) in deltas {
            self.os.entry(path).or_default().count(mode, delta);
        }
        let mut batch = Vec::new();
        let mut unregistered = Vec::new();
        for (path, was) in &before {
            batch.extend(ops(path, *was, self.effective(path)));
            if self.os.get(path).and_then(|counts| counts.mode()).is_none() {
                self.os.remove(path);
                self.failed.remove(path);
                unregistered.push(path.clone());
            }
        }
        self.send(&batch);
        if self.backend.loses_events_on_change() && !batch.is_empty() {
            self.note_lost(before.keys());
        }
        for path in unregistered {
            self.stop_waiting(&path);
        }
    }

    /// Sends `batch` to the backend; paths it refused are polled, paths it watched are no longer.
    fn send(&mut self, batch: &[WatchOp]) {
        if batch.is_empty() {
            return;
        }
        let refused = self.backend.apply(batch);
        for op in batch {
            if let WatchOp::Watch(path, _) = op
                && refused.iter().all(|(failed, _)| failed != path)
            {
                self.failed.remove(path);
            }
        }
        for (path, reason) in refused {
            self.note_failure(path, reason);
        }
    }

    fn note_failure(&mut self, path: PathBuf, reason: String) {
        let message = failure_message(&path, &reason);
        if self.failed.insert(path, reason).is_none() {
            eprintln!("ive: {message}");
        }
    }

    /// notify drops its own watch of a folder that is removed or renamed away, and of everything below it
    /// (inotify). Each such OS path that exists again is watched anew and returned, so what it shows is re-read;
    /// one still missing waits, with its nearest existing ancestor watched until it is back.
    fn rearm(&mut self, events: &[Event]) -> Vec<PathBuf> {
        let gone: Vec<PathBuf> = events.iter().flat_map(touched).filter(|(_, gone)| *gone).map(|(path, _)| path.to_path_buf()).collect();
        let mut dropped: BTreeSet<PathBuf> = self.os.keys().filter(|path| gone.iter().any(|removed| path.starts_with(removed))).cloned().collect();
        dropped.extend(self.waiting.keys().cloned());
        let mut batch = Vec::new();
        let mut back = Vec::new();
        for path in dropped {
            match (self.effective(&path), path.exists()) {
                (Some(mode), true) => {
                    batch.extend([WatchOp::Unwatch(path.clone()), WatchOp::Watch(path.clone(), mode)]);
                    self.stop_waiting(&path);
                    back.push(path);
                }
                (Some(_), false) => self.wait_for(&path),
                (None, _) => self.stop_waiting(&path),
            }
        }
        self.send(&batch);
        back
    }

    /// Watches the nearest existing ancestor of missing `path` (flat), to see it come back.
    fn wait_for(&mut self, path: &Path) {
        let Some(ancestor) = path.ancestors().skip(1).find(|ancestor| ancestor.exists()).map(Path::to_path_buf) else {
            return;
        };
        if self.waiting.get(path) == Some(&ancestor) {
            return;
        }
        self.stop_waiting(path);
        self.waiting.insert(path.to_path_buf(), ancestor.clone());
        self.apply(vec![(ancestor, Mode::Flat, 1)]);
    }

    fn stop_waiting(&mut self, path: &Path) {
        if let Some(ancestor) = self.waiting.remove(path) {
            self.apply(vec![(ancestor, Mode::Flat, -1)]);
        }
    }

    /// Registered folders that are, contain or lie in a path of the batch.
    fn note_lost<'a>(&mut self, touched: impl Iterator<Item = &'a PathBuf>) {
        let touched: Vec<&PathBuf> = touched.collect();
        let folders: Vec<PathBuf> = self
            .all_registered()
            .into_iter()
            .filter_map(|watched| match watched {
                Watched::Folder(path) => Some(path.clone()),
                Watched::File(_) | Watched::GitDir(_) => None,
            })
            .filter(|folder| touched.iter().any(|path| path.starts_with(folder) || folder.starts_with(path)))
            .collect();
        self.lost.extend(folders);
    }
}

/// The OS watches a registration needs: a folder flat; a file's parent flat; a git dir flat, with `refs/` and
/// `worktrees/` (when present) recursive, since both stay small.
fn os_paths(watched: &Watched) -> Vec<(PathBuf, Mode)> {
    match watched {
        Watched::Folder(path) => vec![(path.clone(), Mode::Flat)],
        Watched::File(path) => path.parent().map(|parent| (parent.to_path_buf(), Mode::Flat)).into_iter().collect(),
        Watched::GitDir(dir) => {
            let parts = ["refs", "worktrees"].map(|name| dir.join(name)).into_iter().filter(|part| part.is_dir());
            std::iter::once((dir.clone(), Mode::Flat)).chain(parts.map(|part| (part, Mode::Recursive))).collect()
        }
    }
}

/// What the backend must do for `path` whose mode went from `was` to `now`.
fn ops(path: &Path, was: Option<Mode>, now: Option<Mode>) -> Vec<WatchOp> {
    match (was, now) {
        (None, Some(mode)) => vec![WatchOp::Watch(path.to_path_buf(), mode)],
        (Some(_), None) => vec![WatchOp::Unwatch(path.to_path_buf())],
        (Some(was), Some(now)) if was != now => vec![WatchOp::Unwatch(path.to_path_buf()), WatchOp::Watch(path.to_path_buf(), now)],
        _ => Vec::new(),
    }
}

/// The paths of an event, each with whether it is gone afterwards.
fn touched(event: &Event) -> Vec<(&Path, bool)> {
    let paths = event.paths.iter().map(PathBuf::as_path);
    match event.kind {
        EventKind::Access(_) => Vec::new(),
        EventKind::Remove(_) | EventKind::Modify(ModifyKind::Name(RenameMode::From)) => paths.map(|path| (path, true)).collect(),
        // [from, to]
        EventKind::Modify(ModifyKind::Name(RenameMode::Both)) => paths.enumerate().map(|(index, path)| (path, index == 0)).collect(),
        // FSEvents does not say which side of a rename a path is
        EventKind::Modify(ModifyKind::Name(RenameMode::Any | RenameMode::Other)) => paths.map(|path| (path, !path.exists())).collect(),
        _ => paths.map(|path| (path, false)).collect(),
    }
}

fn classify(path: &Path, registered: &BTreeSet<&Watched>, checkouts: &[PathBuf], changes: &mut Vec<Change>) {
    let git_dir = registered.iter().find_map(|watched| match watched {
        Watched::GitDir(dir) if path.starts_with(dir) => Some(dir),
        _ => None,
    });
    if let Some(dir) = git_dir {
        // git takes `*.lock` files while it writes; the rename that ends the write is what counts
        if path.extension().is_some_and(|extension| extension == "lock") {
            return;
        }
        changes.push(Change::Git(dir.clone()));
        return;
    }
    if registered.contains(&Watched::File(path.to_path_buf())) {
        changes.push(Change::File(path.to_path_buf()));
    }
    if let Some(parent) = path.parent().filter(|parent| registered.contains(&Watched::Folder(parent.to_path_buf()))) {
        changes.push(Change::Folder(parent.to_path_buf()));
    }
    if let Some(checkout) = checkouts.iter().filter(|checkout| path.starts_with(checkout)).max_by_key(|checkout| checkout.components().count()) {
        changes.push(Change::WorkTree(checkout.clone()));
        if crate::favicon::is_favicon_file(path) {
            changes.push(Change::Favicon(checkout.clone()));
        }
    }
}

/// One change per file and kind, a watched file decided by whether it exists now: editors save by writing a
/// temp file and renaming it over the old one, and macOS coalesces a removal and a re-creation into one event.
fn settle(changes: Vec<Change>) -> Vec<Change> {
    let settled = changes.into_iter().map(|change| match change {
        Change::File(path) | Change::FileRemoved(path) if path.exists() => Change::File(path),
        Change::File(path) | Change::FileRemoved(path) => Change::FileRemoved(path),
        other => other,
    });
    let mut seen = HashSet::new();
    settled.filter(|change| seen.insert(change.clone())).collect()
}

fn failure_message(path: &Path, reason: &str) -> String {
    format!("cannot watch {} for changes ({reason}); checking it every 5 s instead", path.display())
}

/// Why notify failed, without its paths, which the failure message names itself.
fn reason_of(error: &notify::Error) -> String {
    match &error.kind {
        notify::ErrorKind::MaxFilesWatch => "OS file watch limit reached".to_owned(),
        notify::ErrorKind::Generic(text) => text.clone(),
        notify::ErrorKind::Io(source) => source.to_string(),
        notify::ErrorKind::PathNotFound => "no such path".to_owned(),
        notify::ErrorKind::WatchNotFound => "no such watch".to_owned(),
        notify::ErrorKind::InvalidConfig(config) => format!("invalid configuration {config:?}"),
    }
}

/// OS file notifications through `notify`: inotify, FSEvents, `ReadDirectoryChangesW`.
pub struct NotifyBackend {
    watcher: notify::RecommendedWatcher,
}

impl NotifyBackend {
    /// `handler` gets every raw event on `notify`'s thread.
    pub fn new(handler: impl notify::EventHandler) -> Result<Self, String> {
        let watcher = notify::recommended_watcher(handler).map_err(|error| format!("cannot start watching files: {error}"))?;
        Ok(Self { watcher })
    }
}

impl Backend for NotifyBackend {
    fn apply(&mut self, batch: &[WatchOp]) -> Vec<(PathBuf, String)> {
        let mut failed = Vec::new();
        // FSEvents restarts its stream once per commit, not once per path
        let mut paths = self.watcher.paths_mut();
        for op in batch {
            match op {
                WatchOp::Watch(path, mode) => {
                    if let Err(error) = paths.add(path, recursive_mode(*mode)) {
                        failed.push((path.clone(), error.to_string()));
                    }
                }
                // the OS drops the watch of a deleted folder by itself, so "not watched" is fine here
                WatchOp::Unwatch(path) => drop(paths.remove(path)),
            }
        }
        if let Err(error) = paths.commit() {
            let watched = batch.iter().filter_map(|op| match op {
                WatchOp::Watch(path, _) => Some((path.clone(), error.to_string())),
                WatchOp::Unwatch(_) => None,
            });
            failed.extend(watched);
        }
        failed
    }

    fn loses_events_on_change(&self) -> bool {
        cfg!(target_os = "macos")
    }
}

/// Where the OS offers no file notifications: every path is polled.
pub struct NoBackend {
    pub reason: String,
}

impl Backend for NoBackend {
    fn apply(&mut self, batch: &[WatchOp]) -> Vec<(PathBuf, String)> {
        let watched = batch.iter().filter_map(|op| match op {
            WatchOp::Watch(path, _) => Some((path.clone(), self.reason.clone())),
            WatchOp::Unwatch(_) => None,
        });
        watched.collect()
    }

    fn loses_events_on_change(&self) -> bool {
        false
    }
}

fn recursive_mode(mode: Mode) -> RecursiveMode {
    match mode {
        Mode::Flat => RecursiveMode::NonRecursive,
        Mode::Recursive => RecursiveMode::Recursive,
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::sync::mpsc;
    use std::time::{Duration, Instant};

    use notify::event::{CreateKind, DataChange, Flag, RemoveKind};

    use super::*;

    /// Records each batch; fails the paths in `refused`.
    #[derive(Default)]
    struct FakeBackend {
        batches: Rc<RefCell<Vec<Vec<WatchOp>>>>,
        refused: Rc<RefCell<Vec<PathBuf>>>,
        loses_events: bool,
    }

    impl Backend for FakeBackend {
        fn apply(&mut self, batch: &[WatchOp]) -> Vec<(PathBuf, String)> {
            self.batches.borrow_mut().push(batch.to_vec());
            let refused = batch.iter().filter_map(|op| match op {
                WatchOp::Watch(path, _) if self.refused.borrow().contains(path) => Some((path.clone(), "inotify watch limit reached".to_owned())),
                _ => None,
            });
            refused.collect()
        }

        fn loses_events_on_change(&self) -> bool {
            self.loses_events
        }
    }

    fn registry(backend: FakeBackend) -> (OpenPaths, Rc<RefCell<Vec<Vec<WatchOp>>>>) {
        let batches = backend.batches.clone();
        (OpenPaths::new(Box::new(backend)), batches)
    }

    fn folder(path: &str) -> Watched {
        Watched::Folder(PathBuf::from(path))
    }

    fn event(kind: EventKind, path: &str) -> Event {
        Event::new(kind).add_path(PathBuf::from(path))
    }

    fn at(kind: EventKind, path: &Path) -> Event {
        Event::new(kind).add_path(path.to_path_buf())
    }

    #[test]
    fn should_watch_a_shared_folder_once_and_unwatch_it_with_its_last_registration() {
        // given
        let (mut paths, batches) = registry(FakeBackend::default());
        let (tree, documents) = (paths.new_owner(), paths.new_owner());

        // when
        paths.set(tree, [folder("/r/src")]);
        paths.set(documents, [Watched::File(PathBuf::from("/r/src/main.rs"))]);
        paths.remove_owner(tree);
        let after_tree = batches.borrow().len();
        paths.remove_owner(documents);

        // then
        assert_eq!(after_tree, 1, "the document still needs /r/src");
        assert_eq!(*batches.borrow(), [vec![WatchOp::Watch(PathBuf::from("/r/src"), Mode::Flat)], vec![WatchOp::Unwatch(PathBuf::from("/r/src"))]]);
    }

    #[test]
    fn should_unwatch_a_collapsed_subtree_in_one_batch() {
        // given
        let (mut paths, batches) = registry(FakeBackend::default());
        let tree = paths.new_owner();
        paths.set(tree, [folder("/r"), folder("/r/a"), folder("/r/a/b")]);

        // when
        paths.set(tree, [folder("/r")]);

        // then
        let last = batches.borrow().last().cloned();
        assert_eq!(last, Some(vec![WatchOp::Unwatch(PathBuf::from("/r/a")), WatchOp::Unwatch(PathBuf::from("/r/a/b"))]));
    }

    #[test]
    fn should_watch_a_git_dir_flat_and_its_refs_recursively() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let git = dir.path().join(".git");
        std::fs::create_dir_all(git.join("refs/heads")).unwrap();
        let (mut paths, batches) = registry(FakeBackend::default());
        let project = paths.new_owner();

        // when
        paths.set(project, [Watched::GitDir(git.clone())]);
        std::fs::create_dir(git.join("worktrees")).unwrap();
        paths.renew_git_dir(&git);

        // then
        assert_eq!(batches.borrow()[0], [WatchOp::Watch(git.clone(), Mode::Flat), WatchOp::Watch(git.join("refs"), Mode::Recursive)]);
        assert_eq!(batches.borrow().last().cloned(), Some(vec![WatchOp::Watch(git.join("worktrees"), Mode::Recursive)]));
    }

    #[test]
    fn should_leave_folders_under_a_recursive_watch_to_it() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let git = dir.path().join(".git");
        std::fs::create_dir_all(git.join("refs/heads")).unwrap();
        let (mut paths, batches) = registry(FakeBackend::default());
        let (project, tree) = (paths.new_owner(), paths.new_owner());
        paths.set(project, [Watched::GitDir(git.clone())]);
        let before = batches.borrow().len();

        // when
        paths.set(tree, [Watched::Folder(git.join("refs/heads"))]);
        paths.set(tree, []);

        // then
        assert_eq!(batches.borrow().len(), before, "the tree's own watch on refs/heads would replace the recursive one");
    }

    #[test]
    fn should_classify_events_by_what_shows_the_path() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let doc = dir.path().join("doc.md");
        std::fs::write(&doc, "").unwrap();
        let gone = dir.path().join("gone.md");
        let (mut paths, _) = registry(FakeBackend::default());
        let owner = paths.new_owner();
        paths.set(owner, [folder("/r/src"), Watched::File(doc.clone()), Watched::File(gone.clone()), Watched::GitDir(PathBuf::from("/r/.git"))]);
        let checkouts = [PathBuf::from("/r"), PathBuf::from("/r/.claude/worktrees/x")];
        let events = [
            event(EventKind::Create(CreateKind::File), "/r/src/new.rs"),
            at(EventKind::Modify(ModifyKind::Data(DataChange::Content)), &doc),
            event(EventKind::Modify(ModifyKind::Data(DataChange::Content)), "/r/other.md"),
            at(EventKind::Remove(RemoveKind::File), &gone),
            event(EventKind::Create(CreateKind::File), "/r/.git/index.lock"),
            event(EventKind::Modify(ModifyKind::Name(RenameMode::To)), "/r/.git/index"),
            event(EventKind::Create(CreateKind::File), "/r/.claude/worktrees/x/a.txt"),
        ];

        // when
        let changes = paths.changes(&events, &checkouts);

        // then
        assert_eq!(changes, [
            Change::Folder(PathBuf::from("/r/src")),
            Change::WorkTree(PathBuf::from("/r")),
            Change::File(doc),
            Change::FileRemoved(gone),
            Change::Git(PathBuf::from("/r/.git")),
            Change::WorkTree(PathBuf::from("/r/.claude/worktrees/x")),
        ]);
    }

    #[test]
    fn should_report_a_favicon_written_in_a_checkout() {
        // given
        let (mut paths, _) = registry(FakeBackend::default());
        let checkouts = [PathBuf::from("/r"), PathBuf::from("/r/.claude/worktrees/x")];
        let events = [
            event(EventKind::Create(CreateKind::File), "/r/.claude/worktrees/x/svc/.favicon.svg"),
            event(EventKind::Create(CreateKind::File), "/r/svc/favicon.svg"),
        ];

        // when
        let changes = paths.changes(&events, &checkouts);

        // then
        assert_eq!(changes, [
            Change::WorkTree(PathBuf::from("/r/.claude/worktrees/x")),
            Change::Favicon(PathBuf::from("/r/.claude/worktrees/x")),
            Change::WorkTree(PathBuf::from("/r")),
        ]);
    }

    #[test]
    fn should_count_a_rename_over_a_document_as_written() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let (doc, staged) = (dir.path().join("doc.md"), dir.path().join("doc.md.tmp"));
        std::fs::write(&doc, "saved").unwrap();
        let (mut paths, _) = registry(FakeBackend::default());
        let documents = paths.new_owner();
        paths.set(documents, [Watched::File(doc.clone())]);
        let rename = |mode: RenameMode| Event::new(EventKind::Modify(ModifyKind::Name(mode)));
        let renames = [
            rename(RenameMode::From).add_path(staged.clone()),
            rename(RenameMode::To).add_path(doc.clone()),
            rename(RenameMode::Both).add_path(staged.clone()).add_path(doc.clone()),
            rename(RenameMode::Any).add_path(doc.clone()),
        ];

        // when
        let changes: Vec<Vec<Change>> = renames.iter().map(|event| paths.changes(std::slice::from_ref(event), &[])).collect();

        // then
        assert_eq!(changes, [Vec::new(), vec![Change::File(doc.clone())], vec![Change::File(doc.clone())], vec![Change::File(doc)]]);
    }

    #[test]
    fn should_decide_by_existence_when_a_batch_both_writes_and_removes_a_file() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let (kept, removed) = (dir.path().join("kept.md"), dir.path().join("removed.md"));
        std::fs::write(&kept, "").unwrap();
        let (mut paths, _) = registry(FakeBackend::default());
        let documents = paths.new_owner();
        paths.set(documents, [Watched::File(kept.clone()), Watched::File(removed.clone())]);
        // macOS coalesces flags, so a batch may say "created, removed" for a file that was removed and written again
        let both = |path: &Path| [at(EventKind::Create(CreateKind::File), path), at(EventKind::Remove(RemoveKind::File), path)];

        // when
        let changes = paths.changes(&[both(&kept), both(&removed)].concat(), &[]);

        // then
        assert_eq!(changes, [Change::File(kept), Change::FileRemoved(removed)]);
    }

    #[test]
    fn should_ask_for_a_rescan_when_events_were_lost() {
        // given
        let (mut paths, _) = registry(FakeBackend::default());
        let overflow = Event::new(EventKind::Other).set_flag(Flag::Rescan);

        // when
        let changes = paths.changes(&[event(EventKind::Any, "/r/a"), overflow], &[]);

        // then
        assert_eq!(changes, [Change::Rescan]);
    }

    #[test]
    fn should_poll_paths_the_os_refused_to_watch_until_they_are_unregistered() {
        // given
        let backend = FakeBackend { refused: Rc::new(RefCell::new(vec![PathBuf::from("/r/big")])), ..FakeBackend::default() };
        let (mut paths, _) = registry(backend);
        let tree = paths.new_owner();
        paths.set(tree, [folder("/r"), folder("/r/big")]);

        // when
        let polled = (paths.poll_changes(), paths.failures());
        paths.set(tree, [folder("/r")]);

        // then
        assert_eq!(polled.0, [Change::Folder(PathBuf::from("/r/big"))]);
        assert_eq!(polled.1, ["cannot watch /r/big for changes (inotify watch limit reached); checking it every 5 s instead"]);
        assert_eq!((paths.poll_changes(), paths.failures()), (Vec::new(), Vec::<String>::new()));
    }

    #[test]
    fn should_poll_a_removed_document_as_removed() {
        // given
        let backend = FakeBackend { refused: Rc::new(RefCell::new(vec![PathBuf::from("/nonexistent")])), ..FakeBackend::default() };
        let (mut paths, _) = registry(backend);
        let documents = paths.new_owner();
        paths.set(documents, [Watched::File(PathBuf::from("/nonexistent/doc.md"))]);

        // when
        let polled = paths.poll_changes();

        // then
        assert_eq!(polled, [Change::FileRemoved(PathBuf::from("/nonexistent/doc.md"))]);
    }

    #[test]
    fn should_poll_the_watched_folder_a_notify_error_names() {
        // given
        let (mut paths, _) = registry(FakeBackend::default());
        let project = paths.new_owner();
        paths.set(project, [folder("/r/.git/refs")]);
        let error = notify::Error::new(notify::ErrorKind::MaxFilesWatch).add_path(PathBuf::from("/r/.git/refs/heads/feature"));

        // when
        paths.note_error(&error);

        // then
        assert_eq!(paths.poll_changes(), [Change::Folder(PathBuf::from("/r/.git/refs"))]);
        assert_eq!(paths.failures().len(), 1);
    }

    #[test]
    fn should_watch_a_removed_folder_again_once_it_is_back() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let docs = dir.path().join("docs");
        std::fs::create_dir(&docs).unwrap();
        let (mut paths, batches) = registry(FakeBackend::default());
        let documents = paths.new_owner();
        paths.set(documents, [Watched::File(docs.join("todo.md"))]);
        std::fs::remove_dir(&docs).unwrap();

        // when
        paths.changes(&[at(EventKind::Remove(RemoveKind::Folder), &docs)], &[]);
        let waiting = batches.borrow().last().cloned();
        std::fs::create_dir(&docs).unwrap();
        let back = paths.changes(&[at(EventKind::Create(CreateKind::Folder), &docs)], &[]);

        // then
        assert_eq!(waiting, Some(vec![WatchOp::Watch(dir.path().to_path_buf(), Mode::Flat)]), "the parent, until docs/ is back");
        let rearmed = &batches.borrow()[batches.borrow().len() - 2..];
        assert_eq!(rearmed, [
            vec![WatchOp::Unwatch(dir.path().to_path_buf())],
            vec![WatchOp::Unwatch(docs.clone()), WatchOp::Watch(docs.clone(), Mode::Flat)],
        ]);
        assert_eq!(back, [Change::FileRemoved(docs.join("todo.md"))]);
    }

    #[test]
    fn should_watch_again_a_refused_folder_that_was_removed_and_recreated() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let big = dir.path().join("big");
        std::fs::create_dir(&big).unwrap();
        let refused = Rc::new(RefCell::new(vec![big.clone()]));
        let (mut paths, _) = registry(FakeBackend { refused: refused.clone(), ..FakeBackend::default() });
        let tree = paths.new_owner();
        paths.set(tree, [Watched::Folder(big.clone())]);
        refused.borrow_mut().clear();

        // when
        let back = paths.changes(&[at(EventKind::Remove(RemoveKind::Folder), &big)], &[]);

        // then
        assert_eq!((back, paths.failures()), (vec![Change::Folder(big)], Vec::<String>::new()));
    }

    #[test]
    fn should_reread_folders_near_a_batch_when_watching_loses_events() {
        // given
        let (mut paths, _) = registry(FakeBackend { loses_events: true, ..FakeBackend::default() });
        let tree = paths.new_owner();
        paths.set(tree, [folder("/r"), folder("/r/a"), folder("/r/a/b"), folder("/s")]);
        paths.take_lost();

        // when
        paths.set(tree, [folder("/r"), folder("/r/a"), folder("/s")]);

        // then
        assert_eq!(paths.take_lost(), [Change::Folder(PathBuf::from("/r")), Change::Folder(PathBuf::from("/r/a"))]);
    }

    /// Collects raw events until `until` holds for them or `limit` passes.
    fn wait_for(events: &mpsc::Receiver<notify::Result<Event>>, limit: Duration, until: impl Fn(&[Event]) -> bool) -> Vec<Event> {
        let deadline = Instant::now() + limit;
        let mut seen = Vec::new();
        while let Some(left) = deadline.checked_duration_since(Instant::now()) {
            match events.recv_timeout(left) {
                Ok(Ok(event)) => seen.push(event),
                Ok(Err(_)) | Err(_) => break,
            }
            if until(&seen) {
                break;
            }
        }
        seen
    }

    #[test]
    fn should_report_a_new_file_in_a_watched_folder_but_not_in_its_unwatched_subfolder() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let watched = dunce::canonicalize(dir.path()).unwrap();
        let unwatched = watched.join("sub");
        std::fs::create_dir(&unwatched).unwrap();
        let (sender, events) = mpsc::channel();
        let mut paths = OpenPaths::new(Box::new(NotifyBackend::new(sender).unwrap()));
        let tree = paths.new_owner();
        paths.set(tree, [Watched::Folder(watched.clone())]);
        let checkouts = [watched.clone()];

        // when
        std::fs::write(watched.join("a.txt"), "a").unwrap();
        let first = wait_for(&events, Duration::from_secs(2), |seen| !seen.is_empty());
        // the rest of a.txt's events (written, closed)
        wait_for(&events, Duration::from_millis(300), |_| false);
        std::fs::write(unwatched.join("b.txt"), "b").unwrap();
        let second = wait_for(&events, Duration::from_secs(1), |seen| !seen.is_empty());

        // then
        assert!(paths.changes(&first, &checkouts).contains(&Change::Folder(watched.clone())), "{first:?}");
        // a recursive watch would see b.txt, and the change would be the checkout's WorkTree
        assert!(second.is_empty(), "{second:?}");
    }
}
