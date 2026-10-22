//! The git data of the tree's worktree folders (Uncommitted, Since base, Commits), per checkout: read on the
//! background executor the first time one of its git folders is shown, read again after its files or its git dir
//! changed (and every 2 s) while a git folder of it is shown, and forgotten otherwise.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use codeview::panel::{Cast, Event};
use gpui::{App, Context, Window, prelude::*};

use super::Workspace;
use crate::editor::Tab;
use crate::git::base;
use crate::git::diff::{self, FilePaths, Uncommitted};
use crate::git::status::Change;
use crate::git::types::{GitPath, Sha};
use crate::git::worktree_data::{SinceData, WorktreeData, load_worktree};
use crate::overview::diff_tab::{DiffRequest, DiffSource, DiffTabSpec};
use crate::overview::file_list::FileOrder;
use crate::project::{CheckoutRef, ProjectKind};
use crate::script::open_folders::{OpenFolders, ALL_OPEN};
use crate::settings::SettingsStore;
use crate::sidebar::folders::{ChangeSource, CheckoutGit, FolderKind};
use crate::sidebar::rows::TreeItem;
use crate::tabs::TabId;

/// What is known, asked for and changed since, by checkout path.
#[derive(Debug, Default)]
pub(super) struct TreeGit {
    known: HashMap<PathBuf, CheckoutGit>,
    /// asked of the checkout's Git section, not answered yet
    asked: HashSet<PathBuf>,
    dirty: HashSet<PathBuf>,
    /// the last read failed: asked again only after a change, so a broken checkout is not read every tick
    failed: HashSet<PathBuf>,
    /// commits whose files are being read
    reading: HashSet<(PathBuf, Sha)>,
    /// the last since data of each known checkout, which the next read reuses while HEAD and the fork point stay
    since: HashMap<PathBuf, SinceData>,
}

impl TreeGit {
    pub(super) fn known(&self) -> &HashMap<PathBuf, CheckoutGit> {
        &self.known
    }

    /// Whether `checkout` needs asking: neither known, asked nor failed.
    fn is_unread(&self, checkout: &Path) -> bool {
        !self.known.contains_key(checkout) && !self.asked.contains(checkout) && !self.failed.contains(checkout)
    }

    /// A checkout's files or git dir changed: a known one is read again, a failed one may be.
    pub(super) fn mark_dirty(&mut self, checkout: &Path) {
        self.failed.remove(checkout);
        if self.known.contains_key(checkout) {
            self.dirty.insert(checkout.to_path_buf());
        }
    }

    /// What file events miss (changes in folders the tree has collapsed): `showing` is read again on the next tick.
    fn poll(&mut self, showing: &[PathBuf]) {
        for checkout in showing {
            self.mark_dirty(checkout);
        }
    }

    /// The changed checkouts to read again: those `has_git_folder` shows; the others' data is forgotten, so showing
    /// a git folder of theirs reads it.
    fn take_due(&mut self, has_git_folder: impl Fn(&Path) -> bool) -> Vec<PathBuf> {
        let wanted = |checkout: &PathBuf| has_git_folder(checkout);
        // one read per checkout at a time: a change during a read is read on a later tick
        let reading: HashSet<PathBuf> = self.dirty.intersection(&self.asked).cloned().collect();
        let (due, stale): (Vec<PathBuf>, Vec<PathBuf>) = self.dirty.drain().filter(|checkout| !reading.contains(checkout)).partition(wanted);
        self.dirty = reading;
        for checkout in stale {
            self.known.remove(&checkout);
            self.since.remove(&checkout);
        }
        due
    }

    /// A read of `checkout` ended (`None`: it failed): what the tree lists, and the since data the next read reuses.
    /// Whether what the tree shows changed.
    fn finish(&mut self, checkout: PathBuf, read: Option<WorktreeData>) -> bool {
        let git = read.map(|data| {
            match &data.since {
                Ok(since) => self.since.insert(checkout.clone(), since.clone()),
                Err(_) => self.since.remove(&checkout),
            };
            CheckoutGit::of(&data)
        });
        self.store(checkout, git)
    }

    /// Whether what the tree shows changed.
    fn store(&mut self, checkout: PathBuf, git: Option<CheckoutGit>) -> bool {
        self.asked.remove(&checkout);
        let Some(git) = git else {
            self.failed.insert(checkout.clone());
            return self.known.remove(&checkout).is_some();
        };
        let git = git.keeping_commit_files(self.known.get(&checkout));
        if self.known.get(&checkout) == Some(&git) {
            return false;
        }
        self.known.insert(checkout, git);
        true
    }
}

const GIT_FOLDERS: [FolderKind; 3] = [FolderKind::Uncommitted, FolderKind::SinceBase, FolderKind::Commits];
/// As often as a shown Git section reads git again (one tick a second).
const GIT_POLL_TICKS: u64 = 2;

impl Workspace {
    pub(super) fn tree_git(&self) -> &HashMap<PathBuf, CheckoutGit> {
        self.tree_git.known()
    }

    /// An IVE window became active: what the git folders show may have changed unseen meanwhile.
    pub(super) fn reread_tree_git(&mut self, cx: &mut Context<Self>) {
        self.poll_tree_git();
        self.read_tree_git(cx);
    }

    fn poll_tree_git(&mut self) {
        let shown = self.projects.iter().flat_map(|open| open.shown_checkouts().into_iter().map(|found| found.path.clone()));
        let showing: Vec<PathBuf> = shown.filter(|checkout| self.has_git_folder(checkout)).collect();
        self.tree_git.poll(&showing);
    }

    /// Reads `checkout`'s git data unless it is known or asked for already.
    pub(crate) fn want_tree_git(&mut self, checkout: &CheckoutRef, cx: &mut Context<Self>) {
        if self.tree_git.is_unread(&checkout.path) {
            self.read_checkout_git(&checkout.path, cx);
        }
    }

    /// Reads a git checkout's lists on the background executor, reusing the last since data while HEAD and the fork
    /// point stay; a plain folder has none.
    fn read_checkout_git(&mut self, path: &Path, cx: &mut Context<Self>) {
        let Some(checkout) = self.checkout_of_path(path).filter(|checkout| checkout.path == path) else {
            return;
        };
        let Some(project) = self.project(&checkout.project).filter(|project| project.kind == ProjectKind::Git).cloned() else {
            return;
        };
        self.tree_git.asked.insert(path.to_path_buf());
        let (dir, previous) = (path.to_path_buf(), self.tree_git.since.get(path).cloned());
        let status_since = self.status_since();
        let reading = cx.background_executor().spawn(async move {
            let base = match project.checkout(&dir) {
                Some(found) => base::base(&project, found),
                None => Err("the checkout is no longer listed".to_owned()),
            };
            load_worktree(&dir, &base, previous.as_ref(), status_since)
        });
        let checkout = path.to_path_buf();
        cx.spawn(async move |this, cx| {
            let read = reading.await.map_err(|error| eprintln!("ive: {error}")).ok();
            this.update(cx, |workspace, cx| {
                if workspace.tree_git.finish(checkout.clone(), read) {
                    workspace.follow_tree_git(&checkout, cx);
                    workspace.tree_changed(cx);
                }
            })
            .ok();
        })
        .detach();
    }

    /// Whether a git folder of `checkout` shows: its checkout is expanded (or the project's only one) and a badge
    /// switched on one the checkout can have.
    fn has_git_folder(&self, checkout: &Path) -> bool {
        let shown = self.projects.iter().flat_map(|open| open.shown_checkouts().into_iter().map(move |found| (open, found)));
        shown.filter(|(_, found)| found.path == checkout).any(|(open, _)| open.folders(checkout).shows_git())
    }

    /// On the tick: reads the changed checkouts that show a git folder, and those that show one never
    /// read (a restart with saved folders, a checkout just expanded). While an IVE window is active, every
    /// `GIT_POLL_TICKS` they count as changed: edits in folders the tree has collapsed send no file events. In the
    /// background that would cost a status read per checkout every 2 s for nobody; activation reads them instead.
    pub(super) fn read_tree_git(&mut self, cx: &mut Context<Self>) {
        if self.watches.ticks.is_multiple_of(GIT_POLL_TICKS) && cx.active_window().is_some() {
            self.poll_tree_git();
        }
        let dirty: Vec<PathBuf> = self.tree_git.dirty.iter().cloned().collect();
        let showing: HashSet<PathBuf> = dirty.into_iter().filter(|checkout| self.has_git_folder(checkout)).collect();
        let due = self.tree_git.take_due(|checkout| showing.contains(checkout));
        let unread = self.projects.iter().flat_map(|open| open.shown_checkouts().into_iter().map(|found| open.project.checkout_ref(found)));
        let unread: Vec<CheckoutRef> = unread.filter(|checkout| self.tree_git.is_unread(&checkout.path) && self.has_git_folder(&checkout.path)).collect();
        for checkout in unread {
            self.want_tree_git(&checkout, cx);
        }
        for path in due {
            self.read_checkout_git(&path, cx);
        }
    }

    pub(super) fn tree_git_changed_at(&mut self, checkout: &Path) {
        self.tree_git.mark_dirty(checkout);
    }

    /// The folders menu's tick, or its `act::ToggleFolderEverywhere` key: shows `kind` in every worktree, or hides it.
    pub(crate) fn toggle_folder_everywhere(&mut self, kind: FolderKind, cx: &mut Context<Self>) {
        let on = !self.shown_folders.contains(&kind);
        self.show_folder_everywhere(kind, on, cx);
    }

    /// Shows `kind` in every worktree of every open project, or hides it, and in worktrees opened later; a git
    /// folder switched on reads the git data of the checkouts the tree shows.
    pub(crate) fn show_folder_everywhere(&mut self, kind: FolderKind, on: bool, cx: &mut Context<Self>) {
        match on {
            true => self.shown_folders.insert(kind),
            false => self.shown_folders.remove(&kind),
        };
        for open in &mut self.projects {
            open.show_folder_everywhere(kind, on);
        }
        if on && GIT_FOLDERS.contains(&kind) {
            let shown: Vec<CheckoutRef> = self.projects.iter().flat_map(|open| open.shown_checkouts().into_iter().map(|checkout| open.project.checkout_ref(checkout))).collect();
            for checkout in &shown {
                self.want_tree_git(checkout, cx);
            }
        }
        self.tree_changed(cx);
        self.schedule_save(cx);
    }

    /// What the tree header's folders menu shows.
    pub(crate) fn folders_menu(&self, cx: &App) -> crate::sidebar::view::FoldersMenu {
        let keymap = crate::script::state::keymap(cx);
        let (keys, layout_key) = keymap.map_or_else(|| (FolderKind::ALL.map(|kind| (kind, None)).to_vec(), None), crate::sidebar::view::menu_keys);
        let layout = cx.global::<SettingsStore>().current().git_files.layout;
        crate::sidebar::view::FoldersMenu { every: self.shown_folders.clone(), layout, keys, layout_key, set_apart: self.folders_set_apart() }
    }

    /// The folders menu's Flat | Tree switch.
    pub(crate) fn set_changes_layout(&mut self, layout: crate::overview::file_list::FileLayout, cx: &mut Context<Self>) {
        if cx.global::<SettingsStore>().current().git_files.layout != layout {
            self.toggle_changes_layout(cx);
        }
    }

    /// Every worktree shows every worktree's folders again.
    pub(crate) fn reset_folders(&mut self, cx: &mut Context<Self>) {
        for open in &mut self.projects {
            open.reset_folders();
        }
        self.tree_changed(cx);
        self.schedule_save(cx);
    }

    /// The worktrees whose folders differ from every worktree's, as `<project> · <worktree>`.
    pub(crate) fn folders_set_apart(&self) -> Vec<String> {
        self.projects.iter().flat_map(|open| open.set_apart().into_iter().map(|checkout| format!("{} · {}", open.project.name, checkout.label))).collect()
    }

    /// A folder's chevron.
    pub(crate) fn toggle_folder_folded(&mut self, checkout: &CheckoutRef, kind: FolderKind, cx: &mut Context<Self>) {
        let Some(open) = self.open_project_mut(&checkout.project) else {
            return;
        };
        let open_folders = cx.try_global::<OpenFolders>().unwrap_or(&ALL_OPEN);
        open.folders_mut(&checkout.path).toggle_folded(kind, open_folders);
        self.tree_changed(cx);
        self.schedule_save(cx);
    }

    /// Opens or closes a change folder. The other folders the list shows keep how they look, so the room this one
    /// frees or takes does not open or close them.
    pub(crate) fn toggle_change_folder(&mut self, checkout: &CheckoutRef, source: ChangeSource, path: String, cx: &mut Context<Self>) {
        let shown: Vec<(String, bool)> = self
            .tree
            .rows
            .iter()
            .filter_map(|row| match &row.item {
                TreeItem::ChangeFolder(folder) if folder.checkout.path == checkout.path && folder.source == source => Some((folder.path.clone(), folder.expanded)),
                _ => None,
            })
            .collect();
        let Some(open) = self.open_project_mut(&checkout.project) else {
            return;
        };
        let folders = open.folders_mut(&checkout.path);
        for (shown_path, expanded) in shown {
            let toggled = shown_path == path;
            folders.set_change_folder(source.clone(), shown_path, expanded != toggled);
        }
        self.tree_changed(cx);
        self.schedule_save(cx);
    }

    /// Opens or closes a commit; its files are read the first time it opens.
    pub(crate) fn toggle_commit(&mut self, checkout: &CheckoutRef, sha: &Sha, cx: &mut Context<Self>) {
        let Some(open) = self.open_project_mut(&checkout.project) else {
            return;
        };
        let opened = open.folders_mut(&checkout.path).toggle_commit(sha.clone());
        let cached = self.tree_git.known.get(&checkout.path).is_some_and(|git| git.commit_files.contains_key(sha));
        let job = (checkout.path.clone(), sha.clone());
        if opened && !cached && self.tree_git.reading.insert(job.clone()) {
            self.read_commit_files(job, cx);
        }
        self.tree_changed(cx);
        self.schedule_save(cx);
    }

    /// New git data of `checkout` came: the open commits and change folders it no longer lists are forgotten, and
    /// the files of open commits not read yet (after a restart) are read.
    fn follow_tree_git(&mut self, checkout: &Path, cx: &mut Context<Self>) {
        let (Some(git), Some(found)) = (self.tree_git.known.get(checkout).cloned(), self.checkout_of_path(checkout)) else {
            return;
        };
        let Some(open) = self.open_project_mut(&found.project) else {
            return;
        };
        let forgot = open.forget_unlisted(checkout, &git);
        let unread: Vec<Sha> = open.folders(checkout).open_commits.iter().filter(|sha| !git.commit_files.contains_key(*sha)).cloned().collect();
        for sha in unread {
            let job = (checkout.to_path_buf(), sha);
            if self.tree_git.reading.insert(job.clone()) {
                self.read_commit_files(job, cx);
            }
        }
        if forgot {
            self.schedule_save(cx);
        }
    }

    fn read_commit_files(&mut self, job: (PathBuf, Sha), cx: &mut Context<Self>) {
        let (dir, sha) = job.clone();
        let reading = cx.background_executor().spawn(async move { diff::commit_changes(&dir, &sha) });
        cx.spawn(async move |this, cx| {
            let files = reading.await.map_err(|error| error.to_string());
            this.update(cx, |workspace, cx| {
                workspace.tree_git.reading.remove(&job);
                let (checkout, sha) = job;
                if let Some(git) = workspace.tree_git.known.get_mut(&checkout) {
                    git.commit_files.insert(sha, files);
                    workspace.tree_changed(cx);
                }
            })
            .ok();
        })
        .detach();
    }

    /// The change of `relative` in checkout `checkout`'s Uncommitted or Since base folder, as last read; `None` when it
    /// is not listed there or the checkout's git was not read.
    pub(super) fn known_change(&self, checkout: &Path, relative: &str, source: &ChangeSource) -> Option<Change> {
        let known = self.tree_git.known.get(checkout)?;
        let changes = match source {
            ChangeSource::Uncommitted => &known.changes,
            ChangeSource::SinceBase => &known.since.as_ref()?.files,
            ChangeSource::Commit(_) => return None,
        };
        changes.iter().find(|change| change.path.as_bytes() == relative.as_bytes()).cloned()
    }

    /// A changed file's row: its diff, read in the background, in a tab.
    pub(crate) fn open_change(&mut self, checkout: &CheckoutRef, source: &ChangeSource, change: &Change, window: &mut Window, cx: &mut Context<Self>) {
        let Some(source) = self.diff_source(&checkout.path, source) else {
            return;
        };
        let request = DiffRequest { dir: checkout.path.clone(), file: FilePaths::of(change), source };
        self.load_diff_tab(DiffAsk { checkout: checkout.clone(), request, scroll: DiffScroll::Kept }, window, cx);
    }

    /// Opens the uncommitted diff of the file in tab `id` when its change popup asks; the subscription ends with the
    /// panel. Other tabs ask for none.
    pub(super) fn open_diff_on_request(&self, id: TabId, window: &mut Window, cx: &mut Context<Self>) {
        let Some(Tab::Document(file)) = self.tabs.get(id) else {
            return;
        };
        cx.subscribe_in(&file.panel.clone(), window, move |workspace, _panel, event: &Event, window, cx| {
            let (Event::OpenDiff { line }, Some(Tab::Document(file))) = (event, workspace.tabs.get(id)) else {
                return;
            };
            let path = file.path().to_path_buf();
            workspace.open_uncommitted_diff(&path, *line, window, cx);
        })
        .detach();
    }

    /// The uncommitted diff of the file at `path` in a tab, its hunk showing the file's line `line` (from 1) at the
    /// top. A file outside the open projects' checkouts has none; nor has one without changes.
    pub(crate) fn open_uncommitted_diff(&mut self, path: &Path, line: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some((checkout, relative)) = self.checkout_of_path(path).and_then(|checkout| {
            let relative = path.strip_prefix(&checkout.path).ok()?.to_path_buf();
            Some((checkout, relative))
        }) else {
            return eprintln!("ive: no uncommitted diff for {}: it is not in an open project's checkout", path.display());
        };
        let file = FilePaths { path: GitPath::from_bytes(relative.as_os_str().as_encoded_bytes()), old_path: None };
        let request = DiffRequest { dir: checkout.path.clone(), file, source: DiffSource::Uncommitted };
        self.load_diff_tab(DiffAsk { checkout, request, scroll: DiffScroll::ToLine(line) }, window, cx);
    }

    /// Reads the diff `ask` names in the background and shows it in its tab.
    fn load_diff_tab(&mut self, ask: DiffAsk, window: &mut Window, cx: &mut Context<Self>) {
        let DiffAsk { checkout, request, scroll } = ask;
        let label = self.checkout_label(&checkout);
        let loading = cx.background_executor().spawn({
            let request = request.clone();
            async move { request.load() }
        });
        cx.spawn_in(window, async move |this, cx| {
            let loaded = loading.await;
            this.update_in(cx, |workspace, window, cx| match loaded {
                Ok(Some(file)) => {
                    let id = workspace.open_diff_tab(&DiffTabSpec { checkout, label, request, file }, window, cx);
                    workspace.scroll_diff_tab(id, scroll, cx);
                }
                // the file has no changes any more: the tree catches up on the next read
                Ok(None) => {}
                Err(error) => eprintln!("ive: {error}"),
            })
            .ok();
        })
        .detach();
    }

    fn scroll_diff_tab(&self, id: TabId, scroll: DiffScroll, cx: &mut Context<Self>) {
        let (DiffScroll::ToLine(line), Some(Tab::Diff(tab))) = (scroll, self.tabs.get(id)) else {
            return;
        };
        tab.panel(cx).update(cx, |panel, cx| panel.handle(Cast::RevealLine(line), cx));
    }

    fn diff_source(&self, checkout: &Path, source: &ChangeSource) -> Option<DiffSource> {
        let since = self.tree_git.known.get(checkout)?.since.as_ref();
        match source {
            ChangeSource::Uncommitted => Some(DiffSource::Uncommitted),
            ChangeSource::SinceBase => since.map(|since| DiffSource::Since { base: since.base.clone(), fork: since.fork.clone(), uncommitted: Uncommitted::Included }),
            ChangeSource::Commit(sha) => {
                let subject = since.and_then(|since| since.commits.iter().find(|commit| commit.sha == *sha)).map(|commit| commit.subject.clone());
                Some(DiffSource::Commit { sha: sha.clone(), subject: subject.unwrap_or_default() })
            }
        }
    }

    /// Load more under the listed commits.
    pub(crate) fn show_more_commits(&mut self, checkout: &CheckoutRef, cx: &mut Context<Self>) {
        if let Some(open) = self.open_project_mut(&checkout.project) {
            open.folders_mut(&checkout.path).show_more_commits();
            self.tree_changed(cx);
            self.schedule_save(cx);
        }
    }

    /// Alt+Shift+G: changed files flat or grouped by folder, in the tree and the project's Git sections.
    pub(crate) fn toggle_changes_layout(&mut self, cx: &mut Context<Self>) {
        let stored = cx.update_global::<SettingsStore, _>(|store, _| {
            let order = store.current().git_files;
            let settings = store.current().with_git_files(FileOrder { layout: order.layout.toggled(), ..order });
            store.replace(settings)
        });
        if let Err(error) = stored {
            eprintln!("ive: cannot save the git file order: {error}");
        }
    }
}

/// A diff to show in its tab, and where to scroll it.
struct DiffAsk {
    checkout: CheckoutRef,
    request: DiffRequest,
    scroll: DiffScroll,
}

/// Where a diff tab scrolls once shown.
#[derive(Clone, Copy)]
enum DiffScroll {
    /// where it is: at the top when new
    Kept,
    /// the hunk showing new-side line `.0` (from 1) at the top
    ToLine(usize),
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use super::*;

    fn path(name: &str) -> PathBuf {
        PathBuf::from(format!("/r/{name}"))
    }

    #[test]
    fn should_read_again_only_changed_checkouts_that_show_a_git_folder() {
        // given
        let mut git = TreeGit::default();
        git.store(path("a"), Some(CheckoutGit::default()));
        git.store(path("b"), Some(CheckoutGit::default()));
        git.mark_dirty(&path("a"));
        git.mark_dirty(&path("b"));
        git.mark_dirty(&path("never-read"));

        // when
        let due = git.take_due(|checkout| checkout == path("a"));

        // then
        assert_eq!(due, [path("a")]);
        assert!(git.known().contains_key(&path("a")));
        assert!(git.is_unread(&path("b")));
    }

    #[test]
    fn should_read_shown_checkouts_again_on_a_poll() {
        // given
        let mut git = TreeGit::default();
        for name in ["shown", "other"] {
            git.store(path(name), Some(CheckoutGit::default()));
        }

        // when
        git.poll(&[path("shown")]);
        let mut due = git.take_due(|checkout| checkout == path("shown"));
        due.sort();

        // then
        assert_eq!(due, [path("shown")]);
        assert!(git.known().contains_key(&path("other")));
    }

    #[test]
    fn should_keep_the_since_data_the_next_read_reuses_until_the_base_is_gone() {
        // given
        let repo = crate::git::testing::TestRepo::init();
        repo.write("f.txt", "0\n");
        repo.commit("base");
        let main = Ok(crate::git::worktrees::BaseBranch { name: "main".to_owned(), reference: "refs/heads/main".to_owned() });
        let with_base = load_worktree(repo.path(), &main, None, Instant::now()).unwrap();
        let without = load_worktree(repo.path(), &Err("no upstream".to_owned()), None, Instant::now()).unwrap();
        let mut git = TreeGit::default();

        // when
        git.finish(path("a"), Some(with_base));
        let kept = git.since.contains_key(&path("a"));
        git.finish(path("a"), Some(without));

        // then
        assert_eq!((kept, git.since.contains_key(&path("a")), git.known().contains_key(&path("a"))), (true, false, true));
    }

    #[test]
    fn should_not_ask_twice_before_an_answer() {
        // given
        let mut git = TreeGit::default();
        git.asked.insert(path("a"));

        // when
        let before = git.is_unread(&path("a"));
        git.store(path("a"), Some(CheckoutGit::default()));

        // then
        assert_eq!((before, git.is_unread(&path("a"))), (false, false));
    }

    #[test]
    fn should_ask_again_after_a_failed_read_only_once_the_checkout_changed() {
        // given
        let mut git = TreeGit::default();
        git.asked.insert(path("a"));
        git.store(path("a"), None);

        // when
        let failed = git.is_unread(&path("a"));
        git.mark_dirty(&path("a"));

        // then
        assert_eq!((failed, git.is_unread(&path("a"))), (false, true));
    }
}
