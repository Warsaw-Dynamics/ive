//! Changes made outside IVE: what the workspace registers in `OpenPaths`, how it acts on the changes notify
//! reports, and the polling notifications cannot replace. Folders and documents are re-read on the background
//! executor and applied in one update per batch.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use codeview::code_view::{self, CodeView, Reread, Stamp};
use futures::StreamExt;
use futures::channel::mpsc::{UnboundedReceiver, unbounded};
use gpui::{App, Context, Task, Window};

use super::Workspace;
use crate::editor::Tab;
use crate::open_project::OpenProject;
use crate::project::{CheckoutRef, ProjectId, Statuses};
use crate::script::state::ScriptState;
use crate::watch::{Backend, Change, NoBackend, NotifyBackend, OpenPaths, WatchOwner, Watched};

type RawEvent = notify::Result<notify::Event>;

/// Raw events are collected this long, so a burst (a build, a checkout) is applied as one batch.
const EVENT_BATCH: Duration = Duration::from_millis(300);
const TICK: Duration = Duration::from_secs(1);
/// Paths the OS refused to watch are re-read every this many ticks while the window is active.
const FALLBACK_POLL_TICKS: u64 = 5;

/// The registry and what the workspace registered in it.
pub(super) struct Watches {
    pub(super) paths: OpenPaths,
    projects: HashMap<ProjectId, WatchOwner>,
    documents: WatchOwner,
    pending: PendingStatuses,
    pub(super) ticks: u64,
    /// when the last batch of file events came: status reads started before it are stale
    events_at: Instant,
}

impl Watches {
    /// The registry over OS notifications (polling only where they cannot start), and the raw events they send.
    pub(super) fn start() -> (Self, UnboundedReceiver<RawEvent>) {
        let (sender, events) = unbounded();
        let backend: Box<dyn Backend> = match NotifyBackend::new(move |event| {
            let _ = sender.unbounded_send(event);
        }) {
            Ok(backend) => Box::new(backend),
            Err(reason) => {
                eprintln!("ive: {reason}");
                Box::new(NoBackend { reason })
            }
        };
        let mut paths = OpenPaths::new(backend);
        let documents = paths.new_owner();
        (Self { paths, projects: HashMap::new(), documents, pending: PendingStatuses::default(), ticks: 0, events_at: Instant::now() }, events)
    }
}

/// Checkouts changed on disk whose status the next tick re-reads, so each is read at most once a second.
#[derive(Debug, Default)]
struct PendingStatuses(HashSet<CheckoutRef>);

impl PendingStatuses {
    fn add(&mut self, checkout: CheckoutRef) {
        self.0.insert(checkout);
    }

    /// What the tick re-reads, by project; the queue is empty afterwards.
    fn take(&mut self) -> HashMap<ProjectId, BTreeSet<PathBuf>> {
        by_project(self.0.drain())
    }
}

fn by_project(checkouts: impl IntoIterator<Item = CheckoutRef>) -> HashMap<ProjectId, BTreeSet<PathBuf>> {
    let mut grouped: HashMap<ProjectId, BTreeSet<PathBuf>> = HashMap::new();
    for checkout in checkouts {
        grouped.entry(checkout.project).or_default().insert(checkout.path);
    }
    grouped
}

/// Hands raw events to the workspace in batches.
pub(super) fn forward_events(mut events: UnboundedReceiver<RawEvent>, window: &mut Window, cx: &mut Context<Workspace>) -> Task<()> {
    cx.spawn_in(window, async move |this, cx| {
        while let Some(first) = events.next().await {
            cx.background_executor().timer(EVENT_BATCH).await;
            let mut batch = vec![first];
            while let Ok(more) = events.try_recv() {
                batch.push(more);
            }
            if this.update_in(cx, |workspace, window, cx| workspace.take_events(batch, window, cx)).is_err() {
                return;
            }
        }
    })
}

/// Once a second: pending status reads, folders a watch change may have missed, and polling.
pub(super) fn tick(window: &mut Window, cx: &mut Context<Workspace>) -> Task<()> {
    cx.spawn_in(window, async move |this, cx| {
        loop {
            cx.background_executor().timer(TICK).await;
            if this.update_in(cx, |workspace, window, cx| workspace.on_tick(window, cx)).is_err() {
                return;
            }
        }
    })
}

impl Workspace {
    /// The oldest status read a view may still show (see [`crate::git::status::shared`]): one started after the last file
    /// events, and at most a tick ago, as polling would read it anyway.
    pub(super) fn status_since(&self) -> Instant {
        let tick_ago = Instant::now().checked_sub(TICK).unwrap_or(self.watches.events_at);
        self.watches.events_at.max(tick_ago)
    }

    /// Registers what the tree and the tabs show now; each owner's difference is one batch.
    pub(super) fn sync_watches(&mut self) {
        let open: Vec<(ProjectId, Vec<Watched>)> = self.projects.iter().map(|open| (open.project.id.clone(), open.watched())).collect();
        let watches = &mut self.watches;
        let closed: Vec<ProjectId> = watches.projects.keys().filter(|id| open.iter().all(|(open, _)| open != *id)).cloned().collect();
        for id in closed {
            if let Some(owner) = watches.projects.remove(&id) {
                watches.paths.remove_owner(owner);
            }
        }
        for (id, wanted) in open {
            let owner = *watches.projects.entry(id).or_insert_with(|| watches.paths.new_owner());
            watches.paths.set(owner, wanted);
        }
        let documents: Vec<Watched> = self.tabs.iter().filter_map(|(_, tab)| tab.watched()).collect();
        self.watches.paths.set(self.watches.documents, documents);
    }

    fn take_events(&mut self, events: Vec<RawEvent>, window: &mut Window, cx: &mut Context<Self>) {
        self.watches.events_at = Instant::now();
        let mut raw = Vec::new();
        for event in events {
            match event {
                Ok(event) => raw.push(event),
                Err(error) => self.watches.paths.note_error(&error),
            }
        }
        let checkouts: Vec<PathBuf> =
            self.projects.iter().flat_map(|open| open.project.checkouts.iter().map(|checkout| checkout.path.clone())).collect();
        let changes = self.watches.paths.changes(&raw, &checkouts);
        self.apply_changes(changes, window, cx);
    }

    /// Acts on changes made outside IVE, a batch at a time: folders and documents are re-read in one background
    /// job each. Parts 2 and 3 add their arms.
    pub fn apply_changes(&mut self, changes: Vec<Change>, _window: &mut Window, cx: &mut Context<Self>) {
        if changes.is_empty() {
            return;
        }
        let mut folders = BTreeSet::new();
        let mut documents = BTreeSet::new();
        for change in changes {
            match change {
                Change::Folder(dir) => {
                    self.scratch_folder_changed(&dir, cx);
                    self.scripts_folder_changed(&dir, cx);
                    self.tree_files_changed(&dir);
                    folders.insert(dir);
                }
                Change::File(path) => {
                    self.scratch_changed_on_disk(&path, cx);
                    documents.insert(path);
                }
                Change::FileRemoved(path) => {
                    self.update_document(&path, CodeView::mark_deleted, cx);
                    self.scratch_changed_on_disk(&path, cx);
                }
                Change::Git(dir) => {
                    self.tree_git_changed(&dir);
                    self.git_dir_changed(&dir, cx);
                    self.hub_git_changed(&dir, cx);
                }
                Change::WorkTree(path) => {
                    self.tree_files_changed(&path);
                    self.queue_status(&path);
                    if let Some(checkout) = self.checkout_of_path(&path) {
                        self.hub_checkout_changed(&checkout.path, cx);
                    }
                }
                Change::Rescan => {
                    folders.extend(self.projects.iter().flat_map(OpenProject::shown_folders));
                    documents.extend(self.document_paths());
                    self.refresh_all(cx);
                    self.rescan_scratches(cx);
                    cx.global::<ScriptState>().reload();
                }
            }
        }
        self.read_folders(folders.into_iter().collect(), cx);
        self.reread_documents(documents.into_iter().collect(), cx);
        cx.notify();
    }

    fn queue_status(&mut self, path: &Path) {
        if let Some(checkout) = self.checkout_of_path(path) {
            self.watches.pending.add(checkout);
        }
    }

    fn document_paths(&self) -> Vec<PathBuf> {
        let watched = self.tabs.iter().filter_map(|(_, tab)| tab.watched());
        let files = watched.filter_map(|watched| match watched {
            Watched::File(path) => Some(path),
            Watched::Folder(_) | Watched::GitDir(_) => None,
        });
        files.collect()
    }

    fn update_document(&self, path: &Path, update: impl FnOnce(&mut CodeView, &mut Context<CodeView>), cx: &mut App) {
        if let Some(id) = self.tabs.find(|tab| tab.shows(path)) {
            self.update_document_tab(id, update, cx);
        }
    }

    /// A re-read document is parsed again for its new text.
    fn rehighlight(&mut self, path: &Path, cx: &mut Context<Self>) {
        if let Some(id) = self.tabs.find(|tab| tab.shows(path)) {
            self.start_highlighting(id, cx);
        }
    }

    /// Re-reads the open documents at `paths` on the background executor; one whose size and modification time
    /// are unchanged is not read.
    fn reread_documents(&mut self, paths: Vec<PathBuf>, cx: &mut Context<Self>) {
        let known: Vec<_> = paths.into_iter().filter_map(|path| self.document_stamp(&path, cx).map(|stamp| (path, stamp))).collect();
        if known.is_empty() {
            return;
        }
        let rereads = cx.background_executor().spawn(async move {
            known.into_iter().map(|(path, stamp)| {
                let reread = code_view::reread(&path, stamp);
                (path, reread)
            }).collect::<Vec<(PathBuf, Reread)>>()
        });
        cx.spawn(async move |this, cx| {
            let rereads = rereads.await;
            this.update(cx, |workspace, cx| {
                for (path, reread) in rereads {
                    let changed = matches!(reread, Reread::Changed(..));
                    workspace.update_document(&path, |view, cx| view.take_reread(reread, cx), cx);
                    if changed {
                        workspace.rehighlight(&path, cx);
                    }
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    /// The stamp of the document `path` shows, while a tab shows it.
    fn document_stamp(&self, path: &Path, cx: &App) -> Option<Option<Stamp>> {
        match self.tabs.get(self.tabs.find(|tab| tab.shows(path))?) {
            Some(Tab::Document(file)) => Some(file.view.read(cx).stamp()),
            _ => None,
        }
    }

    fn git_dir_changed(&mut self, dir: &Path, cx: &mut Context<Self>) {
        self.watches.paths.renew_git_dir(dir);
        let project = self.projects.iter().find(|open| open.project.git_dir.as_deref() == Some(dir)).map(|open| open.project.id.clone());
        if let Some(id) = project {
            self.refresh_project(&id, cx);
            self.git_changed(&id, cx);
        }
    }

    fn on_tick(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.watches.ticks += 1;
        for (id, paths) in self.watches.pending.take() {
            self.refresh_checkouts(&id, Statuses::Only(paths), cx);
        }
        self.read_tree_statuses(cx);
        self.read_tree_git(cx);
        if self.hub_tab_id().is_some() {
            self.index_due_checkouts(cx);
        }
        let mut changes = self.watches.paths.take_lost();
        // the main window, or a detached tab's
        if cx.active_window().is_some() {
            changes.extend(self.poll(cx));
        }
        self.apply_changes(changes, window, cx);
    }

    /// What notifications cannot see: status of the checkouts on screen at the shortest interval of their tabs, and
    /// the paths the OS refused to watch every 5 s.
    fn poll(&mut self, cx: &mut Context<Self>) -> Vec<Change> {
        let ticks = self.watches.ticks;
        let interval = self.tabs.on_screen().filter_map(|(_, tab)| tab.poll_interval()).min();
        if interval.is_some_and(|interval| ticks.is_multiple_of(interval.as_secs().max(1))) {
            for (id, paths) in by_project(self.statuses_on_screen()) {
                self.refresh_checkouts(&id, Statuses::Only(paths), cx);
            }
        }
        if ticks.is_multiple_of(FALLBACK_POLL_TICKS) {
            return self.watches.paths.poll_changes();
        }
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn checkout(project: &str, path: &str) -> CheckoutRef {
        CheckoutRef { project: ProjectId::new(project), path: PathBuf::from(path) }
    }

    #[test]
    fn should_queue_each_changed_checkout_once_until_the_tick_takes_them() {
        // given
        let mut pending = PendingStatuses::default();
        for _ in 0..1000 {
            pending.add(checkout("a", "/a"));
        }
        pending.add(checkout("a", "/a/wt"));
        pending.add(checkout("b", "/b"));

        // when
        let taken = pending.take();

        // then
        let expected = HashMap::from([
            (ProjectId::new("a"), BTreeSet::from([PathBuf::from("/a"), PathBuf::from("/a/wt")])),
            (ProjectId::new("b"), BTreeSet::from([PathBuf::from("/b")])),
        ]);
        assert_eq!(taken, expected);
        assert!(pending.take().is_empty(), "the next tick reads nothing new");
    }
}
