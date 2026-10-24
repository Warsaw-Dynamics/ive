//! A project in the sidebar: its model plus what the tree shows of it (expansion, file trees, paging) and
//! whether a refresh is running.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::file_tree::{Expansion, FileTree, FileTreeError, Listing};
use crate::git::types::Sha;
use crate::paging::Paging;
use crate::sidebar::folders::{COMMITS_SHOWN, ChangeSource, CheckoutFolders, CheckoutGit, FolderKind};
use crate::project::{Checkout, Project, ProjectError, ProjectId, ProjectScan, Statuses};
use crate::state::{MAX_COMMITS_SHOWN, MAX_SAVED_OPEN, SavedChangeFolder, SavedFolders, SavedProject};
use crate::watch::Watched;

/// A list of the tree that shows 10 rows at a time.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PagedList {
    Checkouts(ProjectId),
    /// the processes of the checkout at this path
    Processes(PathBuf),
}

/// At most one refresh per project runs; a request meanwhile queues a single re-run, reading what all queued
/// requests asked for. A running refresh is known by the number of its scan.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
enum Refresh {
    #[default]
    Idle,
    Running(u64),
    Queued(u64, Statuses),
}

/// Whether a project's group of tabs shows all its tabs or only its head, count and current tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabsFolded {
    Folded,
    Open,
}

pub struct OpenProject {
    pub project: Project,
    /// the folder was gone at the last look; shown greyed, with Close only
    pub missing: bool,
    /// a scan came back, so kind and checkouts are read, not the placeholder made at restore
    pub loaded: bool,
    pub expanded: bool,
    expanded_checkouts: HashSet<PathBuf>,
    /// worktrees the user hid: out of the tree and the hub; never the main checkout
    hidden_worktrees: HashSet<PathBuf>,
    /// what each checkout shows of its folders, when not `default_folders`
    folders: HashMap<PathBuf, CheckoutFolders>,
    /// the folders of a checkout nobody changed: every worktree's set, nothing folded or open
    default_folders: CheckoutFolders,
    files: HashMap<PathBuf, FileTree>,
    /// what `state.toml` kept expanded of file trees not built yet, by checkout
    restored_files: HashMap<PathBuf, Expansion>,
    /// file trees kept while their checkout was hidden, so not watched: re-read when shown again
    unwatched: HashSet<PathBuf>,
    checkout_paging: Paging,
    process_paging: HashMap<PathBuf, Paging>,
    /// checkout of the last shown tab of this project that had one
    pub last_checkout: Option<PathBuf>,
    /// whether its group of tabs is folded
    tabs_folded: TabsFolded,
    /// its worktrees' order in its group of tabs, as dragged in the group's menu
    worktree_order: Vec<PathBuf>,
    /// number of the last scan applied; a scan started before it that comes back later is dropped
    applied_scan: u64,
    refresh: Refresh,
}

impl OpenProject {
    /// A project whose first scan is still to come: its folder as a plain project.
    pub fn placeholder(id: ProjectId, root: PathBuf) -> Self {
        Self {
            project: Project::new(id, ProjectScan::plain(root)),
            missing: false,
            loaded: false,
            expanded: false,
            expanded_checkouts: HashSet::new(),
            hidden_worktrees: HashSet::new(),
            folders: HashMap::new(),
            default_folders: CheckoutFolders::default(),
            files: HashMap::new(),
            restored_files: HashMap::new(),
            unwatched: HashSet::new(),
            checkout_paging: Paging::default(),
            process_paging: HashMap::new(),
            last_checkout: None,
            tabs_folded: TabsFolded::Open,
            worktree_order: Vec::new(),
            applied_scan: 0,
            refresh: Refresh::Idle,
        }
    }

    /// A project from `state.toml`, before its first scan; the scan tells whether its folder still exists.
    pub fn restored(saved: &SavedProject, id: ProjectId) -> Self {
        Self {
            expanded: saved.expanded,
            expanded_checkouts: saved.expanded_checkouts.iter().cloned().collect(),
            hidden_worktrees: saved.hidden_worktrees.iter().cloned().collect(),
            last_checkout: saved.last_checkout.clone(),
            tabs_folded: if saved.tabs_folded { TabsFolded::Folded } else { TabsFolded::Open },
            worktree_order: saved.worktree_order.clone(),
            folders: saved.folders.iter().map(restored_folders).collect(),
            restored_files: saved.folders.iter().map(restored_files).filter(|(_, files)| !files.is_empty()).collect(),
            ..Self::placeholder(id, saved.path.clone())
        }
    }

    /// What `state.toml` keeps of the project; checkout paths that are not UTF-8 are left out, since TOML cannot
    /// hold them (the project's own path is UTF-8: IVE opens no other).
    pub fn saved(&self) -> SavedProject {
        let utf8 = |path: &&PathBuf| path.to_str().is_some();
        let mut expanded_checkouts: Vec<PathBuf> = self.expanded_checkouts.iter().filter(utf8).cloned().collect();
        expanded_checkouts.sort();
        let mut hidden_worktrees: Vec<PathBuf> = self.hidden_worktrees.iter().filter(utf8).cloned().collect();
        hidden_worktrees.sort();
        SavedProject {
            path: self.project.root.clone(),
            id: self.project.id.as_str().to_owned(),
            expanded: self.expanded,
            expanded_checkouts,
            last_checkout: self.last_checkout.iter().find(utf8).cloned(),
            folders: self.saved_folders(),
            hidden_worktrees,
            tabs_folded: self.tabs_folded == TabsFolded::Folded,
            worktree_order: self.worktree_order.iter().filter(utf8).cloned().collect(),
        }
    }

    pub fn tabs_folded(&self) -> TabsFolded {
        self.tabs_folded
    }

    pub fn set_tabs_folded(&mut self, folded: TabsFolded) {
        self.tabs_folded = folded;
    }

    /// Its worktrees' order in its group of tabs; the ones never dragged are not in it.
    pub fn worktree_order(&self) -> &[PathBuf] {
        &self.worktree_order
    }

    pub fn set_worktree_order(&mut self, order: Vec<PathBuf>) {
        self.worktree_order = order;
    }

    /// The checkouts whose folders are not the default, by path.
    fn saved_folders(&self) -> Vec<SavedFolders> {
        let expansions = self.expansions();
        let paths: BTreeSet<&PathBuf> = self.folders.keys().chain(expansions.keys()).filter(|path| path.to_str().is_some()).collect();
        paths.into_iter().filter_map(|path| saved_folders(path, self.folders(path), self.every_folder(), expansions.get(path))).collect()
    }

    /// What each checkout's file tree has expanded; a tree not built yet keeps what was restored.
    fn expansions(&self) -> HashMap<PathBuf, Expansion> {
        let built = self.files.iter().map(|(path, files)| (path.clone(), files.expansion()));
        self.restored_files.clone().into_iter().chain(built).filter(|(_, files)| !files.is_empty()).collect()
    }

    /// Takes what scan number `scan` read, unless a later-started scan was applied already; says whether it was
    /// applied. A git failure keeps the last checkouts and shows the error; an unreadable folder marks the project
    /// missing.
    pub fn apply(&mut self, scan: u64, result: Result<ProjectScan, ProjectError>) -> bool {
        if scan <= self.applied_scan {
            return false;
        }
        self.applied_scan = scan;
        match result {
            Ok(read) => {
                self.project.apply(read);
                self.missing = false;
                self.loaded = true;
                let paths: HashSet<PathBuf> = self.project.checkouts.iter().map(|checkout| checkout.path.clone()).collect();
                self.expanded_checkouts.retain(|path| paths.contains(path));
                self.hidden_worktrees.retain(|path| paths.contains(path));
                self.worktree_order.retain(|path| paths.contains(path));
                self.files.retain(|path, _| paths.contains(path));
                self.restored_files.retain(|path, _| paths.contains(path));
                self.folders.retain(|path, _| paths.contains(path));
            }
            Err(ProjectError::Git(error)) => {
                self.loaded = true;
                self.project.refresh_error = Some(error.to_string());
            }
            Err(error) => {
                self.missing = true;
                self.project.refresh_error = Some(error.to_string());
            }
        }
        true
    }

    /// Whether a scan came back: it read the project, or found its folder gone.
    pub fn is_read(&self) -> bool {
        self.loaded || self.missing
    }

    /// Whether the tree shows rows below the project's own: it is expanded, and its folder is there.
    pub fn shows_children(&self) -> bool {
        self.expanded && !self.missing
    }

    /// Whether the project has a worktree level: two or more shown checkouts.
    pub fn has_worktree_level(&self) -> bool {
        self.listed_checkouts().nth(1).is_some()
    }

    /// The checkouts the tree and the hub list, in project order: all but the hidden worktrees.
    pub fn listed_checkouts(&self) -> impl Iterator<Item = &Checkout> {
        self.project.checkouts.iter().filter(|checkout| !self.is_hidden(&checkout.path))
    }

    /// The hidden worktrees, in project order.
    pub fn hidden_checkouts(&self) -> impl Iterator<Item = &Checkout> {
        self.project.checkouts.iter().filter(|checkout| self.is_hidden(&checkout.path))
    }

    pub fn is_hidden(&self, path: &Path) -> bool {
        self.hidden_worktrees.contains(path)
    }

    /// Hides the worktree at `path`, collapsed; false for the main checkout, a path that is no checkout, or one
    /// hidden already.
    pub fn hide(&mut self, path: &Path) -> bool {
        let worktree = self.project.checkouts.iter().any(|checkout| checkout.path == path && !checkout.is_main);
        if !worktree || !self.hidden_worktrees.insert(path.to_path_buf()) {
            return false;
        }
        self.expanded_checkouts.remove(path);
        true
    }

    /// Shows the hidden worktree at `path` again; false when it was not hidden.
    pub fn show(&mut self, path: &Path) -> bool {
        self.hidden_worktrees.remove(path)
    }

    pub fn is_checkout_expanded(&self, path: &Path) -> bool {
        self.expanded_checkouts.contains(path)
    }

    pub fn folders(&self, checkout: &Path) -> &CheckoutFolders {
        self.folders.get(checkout).unwrap_or(&self.default_folders)
    }

    pub fn folders_mut(&mut self, checkout: &Path) -> &mut CheckoutFolders {
        self.folders.entry(checkout.to_path_buf()).or_insert_with(|| self.default_folders.clone())
    }

    /// The folders every worktree shows: what a checkout nobody changed shows.
    pub fn every_folder(&self) -> &BTreeSet<FolderKind> {
        &self.default_folders.shown
    }

    /// Makes `every` what a checkout nobody changed shows; the checkouts that were changed keep their folders.
    pub fn set_every_folder(&mut self, every: &BTreeSet<FolderKind>) {
        self.default_folders.shown = every.clone();
    }

    /// Shows `kind` in every checkout, or hides it, those set apart on it too.
    pub fn show_folder_everywhere(&mut self, kind: FolderKind, on: bool) {
        for folders in std::iter::once(&mut self.default_folders).chain(self.folders.values_mut()) {
            match on {
                true => folders.shown.insert(kind),
                false => folders.shown.remove(&kind),
            };
        }
    }

    /// Every checkout shows every worktree's folders again; what is folded or open in them stays.
    pub fn reset_folders(&mut self) {
        for folders in self.folders.values_mut() {
            folders.shown = self.default_folders.shown.clone();
        }
    }

    /// The listed checkouts whose folders differ from every worktree's, in project order.
    pub fn set_apart(&self) -> Vec<&Checkout> {
        self.listed_checkouts().filter(|checkout| self.folders(&checkout.path).is_set_apart(self.every_folder())).collect()
    }

    /// See [`CheckoutFolders::forget_unlisted`].
    pub fn forget_unlisted(&mut self, checkout: &Path, git: &CheckoutGit) -> bool {
        self.folders.get_mut(checkout).is_some_and(|folders| folders.forget_unlisted(git))
    }

    pub fn toggle(&mut self) {
        self.expanded = !self.expanded;
    }

    pub fn toggle_checkout(&mut self, path: &Path) {
        if self.expanded_checkouts.remove(path) {
            return;
        }
        self.expanded_checkouts.insert(path.to_path_buf());
    }

    /// Expands the project and, with a worktree level, the checkout at `path`, paging the checkouts far enough
    /// to show it; a hidden worktree stays out of the tree.
    pub fn reveal(&mut self, path: &Path) {
        if self.is_hidden(path) {
            return;
        }
        self.list_checkout(path);
        self.expanded_checkouts.insert(path.to_path_buf());
    }

    /// Expands the project and pages the checkouts far enough to show the one at `path`, leaving it collapsed.
    pub fn list_checkout(&mut self, path: &Path) {
        if self.is_hidden(path) {
            return;
        }
        self.expanded = true;
        let index = self.listed_checkouts().position(|checkout| checkout.path == path);
        if let Some(index) = index {
            self.checkout_paging.reveal(index);
        }
    }

    /// Expands the processes of `checkout`, paging them far enough to show the one at `index`.
    pub fn reveal_process(&mut self, checkout: &Path, index: usize) {
        let folders = self.folders_mut(checkout);
        folders.shown.insert(FolderKind::Processes);
        folders.unfold(FolderKind::Processes);
        self.process_paging.entry(checkout.to_path_buf()).or_default().reveal(index);
    }

    /// Unfolds Files and expands the folders of `checkout`'s tree down to `path`; returns the folders to read for it.
    pub fn reveal_entry(&mut self, checkout: &Path, path: &Path) -> Vec<PathBuf> {
        let folders = self.folders_mut(checkout);
        folders.shown.insert(FolderKind::Files);
        folders.unfold(FolderKind::Files);
        self.files.get_mut(checkout).map(|files| files.expand_to(path)).unwrap_or_default()
    }

    pub fn files(&self, checkout: &Path) -> Option<&FileTree> {
        self.files.get(checkout)
    }

    /// Expands or collapses folder `dir` in `checkout`'s tree; returns the folders to read for it.
    pub fn toggle_folder(&mut self, checkout: &Path, dir: &Path) -> Vec<PathBuf> {
        self.files.get_mut(checkout).map(|files| files.toggle(dir)).unwrap_or_default()
    }

    /// Expands or collapses hidden group `group` in `checkout`'s tree; returns the folders to read for it.
    pub fn toggle_hidden(&mut self, checkout: &Path, group: &Path) -> Vec<PathBuf> {
        self.files.get_mut(checkout).map(|files| files.toggle_hidden(group)).unwrap_or_default()
    }

    /// Checkouts whose content the tree shows: the only one when the project is expanded, else the expanded ones;
    /// none before the first scan, which may still find worktrees, or once the folder is gone.
    pub fn shown_checkouts(&self) -> Vec<&Checkout> {
        let shown = self.shows_children() && self.loaded;
        match (shown, self.has_worktree_level()) {
            (true, true) => self.listed_checkouts().filter(|checkout| self.is_checkout_expanded(&checkout.path)).collect(),
            (true, false) => vec![self.project.main_checkout()],
            (false, _) => Vec::new(),
        }
    }

    /// Folders to read for what the tree shows now: the top folder of each shown checkout without a file tree, and
    /// every shown folder of a tree shown again, which nothing watched while it was hidden.
    pub fn folders_to_read(&mut self) -> Vec<PathBuf> {
        let shown: HashSet<PathBuf> = self.shown_checkouts().iter().map(|checkout| checkout.path.clone()).collect();
        self.unwatched.extend(self.files.keys().filter(|path| !shown.contains(*path)).cloned());
        let mut folders = Vec::new();
        for path in shown {
            match self.files.get(&path) {
                Some(files) if self.unwatched.remove(&path) => folders.extend(files.shown_folders()),
                Some(files) => folders.extend(files.unread_folders()),
                None => folders.push(path),
            }
        }
        folders
    }

    /// Shown checkouts that have no file tree yet.
    fn shown_without_files(&self) -> Vec<PathBuf> {
        let shown = self.shown_checkouts().into_iter().map(|checkout| checkout.path.clone());
        shown.filter(|path| !self.files.contains_key(path)).collect()
    }

    /// Takes what reading folders found: new file trees for shown checkouts, new entries for the folders the
    /// trees show. Returns the folders that could not be read.
    pub fn apply_listings(&mut self, listings: &[Listing]) -> Vec<FileTreeError> {
        let mut errors = Vec::new();
        for root in self.shown_without_files() {
            match listings.iter().find(|listing| listing.dir == root).map(|listing| &listing.entries) {
                Some(Ok(entries)) => {
                    let expansion = self.restored_files.remove(&root).unwrap_or_default();
                    self.files.insert(root.clone(), FileTree::restored(root, entries.clone(), &expansion));
                }
                Some(Err(error)) => errors.push(error.clone()),
                None => {}
            }
        }
        for files in self.files.values_mut() {
            errors.extend(files.apply(listings));
        }
        errors
    }

    /// What the tree shows of this project, for the watch registry: the shown folders of its file trees and the
    /// git dir.
    pub fn watched(&self) -> Vec<Watched> {
        if self.missing {
            return Vec::new();
        }
        let folders = self.shown_folders().into_iter().map(Watched::Folder);
        folders.chain(self.project.git_dir.clone().map(Watched::GitDir)).collect()
    }

    /// Every folder the tree shows of this project.
    pub fn shown_folders(&self) -> Vec<PathBuf> {
        let trees = self.shown_checkouts().into_iter().filter_map(|checkout| self.files.get(&checkout.path));
        trees.flat_map(FileTree::shown_folders).collect()
    }

    /// For tests: reads what the tree asks for at once, on this thread.
    #[cfg(test)]
    pub fn read_files_now(&mut self) -> Vec<FileTreeError> {
        let listings: Vec<Listing> = self.folders_to_read().iter().map(|dir| crate::file_tree::read_folder(dir)).collect();
        self.apply_listings(&listings)
    }

    pub fn paging(&self, list: &PagedList) -> Paging {
        match list {
            PagedList::Checkouts(_) => self.checkout_paging,
            PagedList::Processes(path) => self.process_paging.get(path).copied().unwrap_or_default(),
        }
    }

    pub fn show_more(&mut self, list: &PagedList) {
        self.paging_mut(list).more();
    }

    pub fn show_all(&mut self, list: &PagedList) {
        self.paging_mut(list).all();
    }

    fn paging_mut(&mut self, list: &PagedList) -> &mut Paging {
        match list {
            PagedList::Checkouts(_) => &mut self.checkout_paging,
            PagedList::Processes(path) => self.process_paging.entry(path.clone()).or_default(),
        }
    }

    /// What a refresh starting now as scan number `scan` reads; `None` while one runs, which queues this one
    /// (merged with any queued before).
    pub fn request_refresh(&mut self, statuses: Statuses, scan: u64) -> Option<Statuses> {
        match std::mem::take(&mut self.refresh) {
            Refresh::Idle => {
                self.refresh = Refresh::Running(scan);
                Some(statuses)
            }
            Refresh::Running(running) => {
                self.refresh = Refresh::Queued(running, statuses);
                None
            }
            Refresh::Queued(running, queued) => {
                self.refresh = Refresh::Queued(running, queued.union(statuses));
                None
            }
        }
    }

    /// Ends the refresh of scan `scan`; returns what was queued meanwhile, which the caller requests again. A scan
    /// this project did not start (one of a closed project with the same id) changes nothing.
    pub fn finish_refresh(&mut self, scan: u64) -> Option<Statuses> {
        match std::mem::take(&mut self.refresh) {
            Refresh::Running(running) if running == scan => None,
            Refresh::Queued(running, statuses) if running == scan => Some(statuses),
            other => {
                self.refresh = other;
                None
            }
        }
    }
}

/// What `state.toml` keeps of a checkout's folders and its file tree's `expansion`; `None` when all are as by
/// default, every worktree showing `every`. Keeps at most [`MAX_SAVED_OPEN`] of each kind of open item.
fn saved_folders(checkout: &Path, folders: &CheckoutFolders, every: &BTreeSet<FolderKind>, expansion: Option<&Expansion>) -> Option<SavedFolders> {
    if folders.is_default_saved(every) && expansion.is_none() {
        return None;
    }
    let mut open_commits: Vec<String> = folders.open_commits.iter().map(|sha| sha.as_str().to_owned()).collect();
    open_commits.sort();
    let change_folders = folders.change_folders.iter().map(|((source, path), open)| SavedChangeFolder { list: source.key().to_owned(), path: path.clone(), open: *open });
    let mut change_folders: Vec<SavedChangeFolder> = change_folders.collect();
    change_folders.sort_by(|a, b| (&a.list, &a.path).cmp(&(&b.list, &b.path)));
    let utf8 = |paths: &[PathBuf]| capped(paths.iter().filter(|path| path.to_str().is_some()).cloned().collect());
    let expansion = expansion.cloned().unwrap_or_default();
    Some(SavedFolders {
        checkout: checkout.to_path_buf(),
        shown: folders.shown.iter().copied().filter(|kind| *kind != FolderKind::Files).collect(),
        files_hidden: !folders.shown.contains(&FolderKind::Files),
        folded: folded(folders, true),
        unfolded: folded(folders, false),
        open_commits: capped(open_commits),
        commits_shown: Some(folders.commits_shown.min(MAX_COMMITS_SHOWN)).filter(|shown| *shown != COMMITS_SHOWN),
        files: utf8(&expansion.folders),
        hidden_groups: utf8(&expansion.hidden),
        change_folders: capped(change_folders),
    })
}

/// The folders the user folded (`fold`) or unfolded.
fn folded(folders: &CheckoutFolders, fold: bool) -> Vec<FolderKind> {
    folders.folds.iter().filter(|(_, folded)| **folded == fold).map(|(kind, _)| *kind).collect()
}

fn capped<T>(mut items: Vec<T>) -> Vec<T> {
    items.truncate(MAX_SAVED_OPEN);
    items
}

/// A checkout's folders as `state.toml` kept them; Files shows unless hidden. Commits and lists that are not one are
/// dropped.
fn restored_folders(saved: &SavedFolders) -> (PathBuf, CheckoutFolders) {
    let files = (!saved.files_hidden).then_some(FolderKind::Files);
    let shown = saved.shown.iter().copied().filter(|kind| *kind != FolderKind::Files).chain(files).collect();
    let change_folders = saved.change_folders.iter().filter_map(|folder| Some(((ChangeSource::parse(&folder.list)?, folder.path.clone()), folder.open)));
    let folders = CheckoutFolders {
        shown,
        folds: saved.folded.iter().map(|kind| (*kind, true)).chain(saved.unfolded.iter().map(|kind| (*kind, false))).collect(),
        change_folders: change_folders.collect(),
        open_commits: saved.open_commits.iter().filter_map(|sha| Sha::parse(sha)).collect(),
        commits_shown: saved.commits_shown.unwrap_or(COMMITS_SHOWN),
    };
    (saved.checkout.clone(), folders)
}

fn restored_files(saved: &SavedFolders) -> (PathBuf, Expansion) {
    (saved.checkout.clone(), Expansion { folders: saved.files.clone(), hidden: saved.hidden_groups.clone() })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::types::Sha;
    use crate::project::{CheckoutStatus, ProjectKind};

    fn scan(root: &Path, worktrees: &[&str]) -> ProjectScan {
        let checkout = |path: PathBuf, is_main: bool| Checkout {
            label: path.file_name().unwrap().to_string_lossy().into_owned(),
            path,
            branch: Some("main".to_owned()),
            head: None,
            is_main,
            status: Some(CheckoutStatus::default()),
        };
        let mut checkouts = vec![checkout(root.to_path_buf(), true)];
        checkouts.extend(worktrees.iter().map(|name| checkout(root.join(name), false)));
        ProjectScan { root: root.to_path_buf(), kind: ProjectKind::Git, checkouts, error: None, git_dir: Some(root.join(".git")) }
    }

    #[test]
    fn should_queue_one_rerun_while_a_refresh_runs() {
        // given
        let mut project = OpenProject::placeholder(ProjectId::new("p"), PathBuf::from("/p"));

        // when
        let starts = [1, 2, 3].map(|scan| project.request_refresh(Statuses::All, scan));
        let rerun = project.finish_refresh(1);
        let again = project.request_refresh(Statuses::All, 4);

        // then
        assert_eq!((starts, rerun.clone(), again), ([Some(Statuses::All), None, None], Some(Statuses::All), Some(Statuses::All)));
        assert_eq!(project.finish_refresh(4), None, "nothing queued");
        assert_eq!(project.request_refresh(Statuses::All, 5), Some(Statuses::All), "idle again");
    }

    #[test]
    fn should_queue_one_rerun_reading_what_every_queued_request_asked_for() {
        // given
        let mut project = OpenProject::placeholder(ProjectId::new("p"), PathBuf::from("/p"));
        let only = |path: &str| Statuses::Only([PathBuf::from(path)].into());
        project.request_refresh(only("/p"), 1);

        // when
        project.request_refresh(only("/p/a"), 2);
        project.request_refresh(only("/p/b"), 3);
        let rerun = project.finish_refresh(1);

        // then
        assert_eq!(rerun, Some(Statuses::Only([PathBuf::from("/p/a"), PathBuf::from("/p/b")].into())));
    }

    #[test]
    fn should_ignore_the_end_of_a_refresh_it_did_not_start() {
        // given
        let mut project = OpenProject::placeholder(ProjectId::new("p"), PathBuf::from("/p"));
        project.request_refresh(Statuses::All, 7);
        project.request_refresh(Statuses::All, 8);

        // when
        let ended = project.finish_refresh(3);

        // then
        assert_eq!(ended, None);
        assert_eq!(project.finish_refresh(7), Some(Statuses::All), "scan 7 still ran, with one queued");
    }

    #[test]
    fn should_drop_a_scan_started_before_the_one_applied() {
        // given
        let mut project = OpenProject::placeholder(ProjectId::new("p"), PathBuf::from("/p"));
        project.apply(5, Ok(scan(Path::new("/p"), &["new"])));

        // when
        let applied = project.apply(4, Ok(scan(Path::new("/p"), &[])));

        // then
        assert!(!applied);
        assert_eq!(project.project.checkouts.len(), 2);
    }

    #[test]
    fn should_keep_last_checkouts_and_show_error_when_git_fails() {
        // given
        let mut project = OpenProject::placeholder(ProjectId::new("p"), PathBuf::from("/p"));
        project.apply(1, Ok(scan(Path::new("/p"), &["wt"])));
        let error = crate::git::GitError { operation: "read the status".to_owned(), dir: PathBuf::from("/p"), message: "locked".to_owned() };

        // when
        project.apply(2, Err(ProjectError::Git(error)));

        // then
        assert_eq!(project.project.checkouts.len(), 2);
        assert_eq!(project.project.refresh_error.as_deref(), Some("cannot read the status of /p: locked"));
    }

    #[test]
    fn should_forget_expansion_of_checkouts_that_disappeared() {
        // given
        let mut project = OpenProject::placeholder(ProjectId::new("p"), PathBuf::from("/p"));
        project.apply(1, Ok(scan(Path::new("/p"), &["a", "b"])));
        project.reveal(Path::new("/p/a"));
        project.reveal(Path::new("/p/b"));

        // when
        project.apply(2, Ok(scan(Path::new("/p"), &["a"])));

        // then
        assert_eq!(project.saved().expanded_checkouts, [PathBuf::from("/p/a")]);
    }

    fn paths<'a>(checkouts: impl Iterator<Item = &'a Checkout>) -> Vec<PathBuf> {
        checkouts.map(|checkout| checkout.path.clone()).collect()
    }

    #[test]
    fn should_hide_a_worktree_but_never_the_main_checkout() {
        // given
        let mut project = OpenProject::placeholder(ProjectId::new("p"), PathBuf::from("/p"));
        project.apply(1, Ok(scan(Path::new("/p"), &["wt"])));

        // when
        let hid = [project.hide(Path::new("/p")), project.hide(Path::new("/p/wt")), project.hide(Path::new("/p/gone"))];

        // then
        assert_eq!(hid, [false, true, false]);
        assert_eq!((paths(project.listed_checkouts()), paths(project.hidden_checkouts())), (vec![PathBuf::from("/p")], vec![PathBuf::from("/p/wt")]));
        assert!(!project.has_worktree_level(), "only the main checkout is shown");
    }

    #[test]
    fn should_show_a_hidden_worktree_again() {
        // given
        let mut project = OpenProject::placeholder(ProjectId::new("p"), PathBuf::from("/p"));
        project.apply(1, Ok(scan(Path::new("/p"), &["wt"])));
        project.hide(Path::new("/p/wt"));

        // when
        let shown = [project.show(Path::new("/p/wt")), project.show(Path::new("/p/wt"))];

        // then
        assert_eq!(shown, [true, false]);
        assert!(!project.is_hidden(Path::new("/p/wt")));
        assert!(project.has_worktree_level());
    }

    #[test]
    fn should_save_and_restore_hidden_worktrees() {
        // given
        let mut project = OpenProject::placeholder(ProjectId::new("p"), PathBuf::from("/p"));
        project.apply(1, Ok(scan(Path::new("/p"), &["b", "a"])));
        project.hide(Path::new("/p/b"));
        project.hide(Path::new("/p/a"));

        // when
        let saved = project.saved();
        let restored = OpenProject::restored(&saved, ProjectId::new("p"));

        // then
        assert_eq!(saved.hidden_worktrees, [PathBuf::from("/p/a"), PathBuf::from("/p/b")]);
        assert!(restored.is_hidden(Path::new("/p/a")));
    }

    #[test]
    fn should_save_and_restore_folded_tabs_and_worktree_order() {
        // given
        let mut project = OpenProject::placeholder(ProjectId::new("p"), PathBuf::from("/p"));
        project.apply(1, Ok(scan(Path::new("/p"), &["a", "b"])));
        project.set_tabs_folded(TabsFolded::Folded);
        project.set_worktree_order(vec![PathBuf::from("/p/b"), PathBuf::from("/p")]);

        // when
        let restored = OpenProject::restored(&project.saved(), ProjectId::new("p"));

        // then
        assert_eq!(restored.tabs_folded(), TabsFolded::Folded);
        assert_eq!(restored.worktree_order(), [PathBuf::from("/p/b"), PathBuf::from("/p")]);
    }

    #[test]
    fn should_drop_gone_worktrees_from_the_order() {
        // given
        let mut project = OpenProject::placeholder(ProjectId::new("p"), PathBuf::from("/p"));
        project.apply(1, Ok(scan(Path::new("/p"), &["a", "b"])));
        project.set_worktree_order(vec![PathBuf::from("/p/b"), PathBuf::from("/p/a")]);

        // when
        project.apply(2, Ok(scan(Path::new("/p"), &["a"])));

        // then
        assert_eq!(project.worktree_order(), [PathBuf::from("/p/a")]);
    }

    #[test]
    fn should_forget_a_hidden_worktree_a_scan_no_longer_lists() {
        // given
        let mut project = OpenProject::placeholder(ProjectId::new("p"), PathBuf::from("/p"));
        project.apply(1, Ok(scan(Path::new("/p"), &["wt"])));
        project.hide(Path::new("/p/wt"));

        // when
        project.apply(2, Ok(scan(Path::new("/p"), &[])));
        project.apply(3, Ok(scan(Path::new("/p"), &["wt"])));

        // then
        assert!(!project.is_hidden(Path::new("/p/wt")));
        assert!(project.saved().hidden_worktrees.is_empty());
    }

    #[test]
    fn should_list_a_new_checkout_collapsed_under_its_expanded_project() {
        // given
        let names: Vec<String> = (0..12).map(|n| format!("wt{n:02}")).collect();
        let worktrees: Vec<&str> = names.iter().map(String::as_str).collect();
        let mut project = OpenProject::placeholder(ProjectId::new("p"), PathBuf::from("/p"));
        project.apply(1, Ok(scan(Path::new("/p"), &worktrees)));

        // when
        project.list_checkout(Path::new("/p/wt11"));

        // then
        assert!(project.expanded);
        assert!(!project.is_checkout_expanded(Path::new("/p/wt11")));
        assert_eq!(project.paging(&PagedList::Checkouts(ProjectId::new("p"))).visible(13), 13, "paged far enough to show it");
    }

    #[test]
    fn should_not_expand_or_show_the_content_of_a_hidden_worktree() {
        // given
        let mut project = OpenProject::placeholder(ProjectId::new("p"), PathBuf::from("/p"));
        project.apply(1, Ok(scan(Path::new("/p"), &["a", "b"])));
        project.reveal(Path::new("/p/b"));
        project.hide(Path::new("/p/a"));

        // when
        project.reveal(Path::new("/p/a"));
        project.list_checkout(Path::new("/p/a"));

        // then
        assert!(!project.is_checkout_expanded(Path::new("/p/a")));
        assert_eq!(paths(project.shown_checkouts().into_iter()), [PathBuf::from("/p/b")]);
    }

    #[test]
    fn should_collapse_a_worktree_it_hides() {
        // given
        let mut project = OpenProject::placeholder(ProjectId::new("p"), PathBuf::from("/p"));
        project.apply(1, Ok(scan(Path::new("/p"), &["wt"])));
        project.reveal(Path::new("/p/wt"));

        // when
        project.hide(Path::new("/p/wt"));

        // then
        assert!(!project.is_checkout_expanded(Path::new("/p/wt")));
    }

    #[test]
    fn should_keep_shown_and_folded_folders_across_a_restart() {
        // given
        let mut project = OpenProject::placeholder(ProjectId::new("p"), PathBuf::from("/p"));
        project.apply(1, Ok(scan(Path::new("/p"), &["a"])));
        let folders = project.folders_mut(Path::new("/p/a"));
        let open = crate::script::open_folders::parse(&serde_json::from_value(serde_json::json!(["processes"])).unwrap()).unwrap();
        folders.toggle_shown(FolderKind::SinceBase);
        folders.toggle_folded(FolderKind::Processes, &open);
        folders.toggle_folded(FolderKind::Files, &open);

        // when
        let restored = OpenProject::restored(&project.saved(), ProjectId::new("p"));

        // then
        assert_eq!(project.saved().folders, [SavedFolders {
            checkout: PathBuf::from("/p/a"),
            shown: vec![FolderKind::Processes, FolderKind::SinceBase],
            files_hidden: false,
            folded: vec![FolderKind::Processes],
            unfolded: vec![FolderKind::Files],
            open_commits: Vec::new(),
            commits_shown: None,
            files: Vec::new(),
            hidden_groups: Vec::new(),
            change_folders: Vec::new(),
        }]);
        assert_eq!(restored.folders(Path::new("/p/a")).shown, project.folders(Path::new("/p/a")).shown);
        assert_eq!(restored.folders(Path::new("/p/a")).folds, project.folders(Path::new("/p/a")).folds);
    }

    #[test]
    fn should_keep_files_switched_off_across_a_restart_and_show_it_for_state_saved_before_its_badge() {
        // given
        let mut project = OpenProject::placeholder(ProjectId::new("p"), PathBuf::from("/p"));
        project.apply(1, Ok(scan(Path::new("/p"), &["a", "b"])));
        project.folders_mut(Path::new("/p/a")).toggle_shown(FolderKind::Files);
        project.folders_mut(Path::new("/p/b")).toggle_shown(FolderKind::SinceBase);
        let saved = project.saved();

        // when
        let restored = OpenProject::restored(&saved, ProjectId::new("p"));

        // then
        assert_eq!(saved.folders.iter().map(|folders| folders.files_hidden).collect::<Vec<_>>(), [true, false]);
        assert!(!restored.folders(Path::new("/p/a")).is_shown(FolderKind::Files));
        assert!(restored.folders(Path::new("/p/b")).is_shown(FolderKind::Files));
    }

    #[test]
    fn should_keep_open_commits_and_change_folders_across_a_restart() {
        // given
        let mut project = OpenProject::placeholder(ProjectId::new("p"), PathBuf::from("/p"));
        project.apply(1, Ok(scan(Path::new("/p"), &["a"])));
        let sha = Sha::parse(&"ab".repeat(20)).unwrap();
        let folders = project.folders_mut(Path::new("/p/a"));
        folders.toggle_commit(sha.clone());
        folders.show_more_commits();
        folders.set_change_folder(ChangeSource::SinceBase, "src/ui".to_owned(), false);
        folders.set_change_folder(ChangeSource::Commit(sha), "docs".to_owned(), true);

        // when
        let restored = OpenProject::restored(&project.saved(), ProjectId::new("p"));

        // then
        assert_eq!(restored.folders(Path::new("/p/a")), project.folders(Path::new("/p/a")));
    }

    #[test]
    fn should_expand_the_files_again_after_a_restart() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        std::fs::create_dir_all(root.join("src/ui")).unwrap();
        std::fs::write(root.join("src/ui/view.rs"), "").unwrap();
        let mut project = OpenProject::placeholder(ProjectId::new("p"), root.clone());
        project.toggle();
        project.apply(1, Ok(ProjectScan::plain(root.clone())));
        project.read_files_now();
        project.toggle_folder(&root, &root.join("src"));
        project.read_files_now();
        project.toggle_folder(&root, &root.join("src/ui"));
        let saved = project.saved();
        let mut restored = OpenProject::restored(&saved, ProjectId::new("p"));

        // when
        restored.apply(1, Ok(ProjectScan::plain(root.clone())));
        for _ in 0..3 {
            restored.read_files_now();
        }

        // then
        assert_eq!(saved.folders[0].files, [root.join("src"), root.join("src/ui")]);
        let names: Vec<&str> = restored.files(&root).unwrap().rows().iter().map(|row| row.entry.name.as_str()).collect();
        assert_eq!(names, ["src", "ui", "view.rs"]);
    }

    #[test]
    fn should_not_save_default_folders() {
        // given
        let mut project = OpenProject::placeholder(ProjectId::new("p"), PathBuf::from("/p"));
        project.apply(1, Ok(scan(Path::new("/p"), &["a"])));
        let folders = project.folders_mut(Path::new("/p/a"));
        folders.toggle_shown(FolderKind::Commits);
        folders.toggle_shown(FolderKind::Commits);
        folders.toggle_commit(Sha::parse(&"ab".repeat(20)).unwrap());
        folders.toggle_commit(Sha::parse(&"ab".repeat(20)).unwrap());

        // when
        let saved = project.saved();

        // then
        assert!(saved.folders.is_empty(), "{:?}", saved.folders);
    }

    fn every(kinds: &[FolderKind]) -> BTreeSet<FolderKind> {
        kinds.iter().copied().collect()
    }

    #[test]
    fn should_follow_every_worktree_for_untouched_checkouts() {
        // given
        let mut project = OpenProject::placeholder(ProjectId::new("p"), PathBuf::from("/p"));
        project.apply(1, Ok(scan(Path::new("/p"), &["a"])));

        // when
        project.set_every_folder(&every(&[FolderKind::Processes, FolderKind::Commits, FolderKind::Files]));

        // then
        assert!(project.folders(Path::new("/p/a")).is_shown(FolderKind::Commits));
        assert!(project.folders_mut(Path::new("/p/a")).is_shown(FolderKind::Commits), "a new entry starts as every worktree's");
        assert!(project.saved().folders.is_empty(), "{:?}", project.saved().folders);
    }

    #[test]
    fn should_switch_a_folder_in_every_checkout() {
        // given
        let mut project = OpenProject::placeholder(ProjectId::new("p"), PathBuf::from("/p"));
        project.apply(1, Ok(scan(Path::new("/p"), &["a", "b"])));
        project.folders_mut(Path::new("/p/a")).toggle_shown(FolderKind::Uncommitted);

        // when
        project.show_folder_everywhere(FolderKind::Commits, true);
        let commits = ["/p/a", "/p/b"].map(|path| project.folders(Path::new(path)).is_shown(FolderKind::Commits));
        let kept = project.folders(Path::new("/p/a")).is_shown(FolderKind::Uncommitted);
        project.show_folder_everywhere(FolderKind::Uncommitted, false);

        // then
        assert_eq!(commits, [true, true]);
        assert!(kept, "a folder set apart stays until it is switched everywhere");
        assert_eq!(["/p/a", "/p/b"].map(|path| project.folders(Path::new(path)).is_shown(FolderKind::Uncommitted)), [false, false]);
        assert!(project.every_folder().contains(&FolderKind::Commits));
    }

    #[test]
    fn should_reset_checkouts_set_apart() {
        // given
        let mut project = OpenProject::placeholder(ProjectId::new("p"), PathBuf::from("/p"));
        project.apply(1, Ok(scan(Path::new("/p"), &["a", "b"])));
        let folders = project.folders_mut(Path::new("/p/a"));
        folders.toggle_shown(FolderKind::Commits);
        folders.toggle_folded(FolderKind::Files, &crate::script::open_folders::ALL_OPEN);
        let apart: Vec<PathBuf> = project.set_apart().iter().map(|checkout| checkout.path.clone()).collect();

        // when
        project.reset_folders();

        // then
        assert_eq!(apart, [PathBuf::from("/p/a")]);
        assert_eq!(project.folders(Path::new("/p/a")).shown, *project.every_folder());
        assert!(!project.folders(Path::new("/p/a")).is_expanded(FolderKind::Files, &crate::script::open_folders::ALL_OPEN), "folds stay");
        assert!(project.set_apart().is_empty());
    }

    #[test]
    fn should_keep_old_saved_folders_as_they_were() {
        // given
        let mut project = OpenProject::placeholder(ProjectId::new("p"), PathBuf::from("/p"));
        project.apply(1, Ok(scan(Path::new("/p"), &["a"])));
        project.folders_mut(Path::new("/p/a")).toggle_shown(FolderKind::Commits);
        let saved = project.saved();

        // when
        let mut restored = OpenProject::restored(&saved, ProjectId::new("p"));
        restored.set_every_folder(&crate::sidebar::folders::default_shown());

        // then
        assert_eq!(restored.folders(Path::new("/p/a")).shown, every(&[FolderKind::Processes, FolderKind::Commits, FolderKind::Files]));
    }

    #[test]
    fn should_not_save_a_checkout_that_matches_every_worktree() {
        // given
        let mut project = OpenProject::placeholder(ProjectId::new("p"), PathBuf::from("/p"));
        project.apply(1, Ok(scan(Path::new("/p"), &["a"])));
        project.set_every_folder(&every(&[FolderKind::Processes, FolderKind::Commits, FolderKind::Files]));
        let folders = project.folders_mut(Path::new("/p/a"));
        folders.toggle_shown(FolderKind::Commits);
        folders.toggle_shown(FolderKind::Commits);

        // when
        let saved = project.saved();

        // then
        assert!(saved.folders.is_empty(), "{:?}", saved.folders);
    }

    #[test]
    fn should_forget_folders_of_checkouts_that_disappeared() {
        // given
        let mut project = OpenProject::placeholder(ProjectId::new("p"), PathBuf::from("/p"));
        project.apply(1, Ok(scan(Path::new("/p"), &["a", "b"])));
        project.folders_mut(Path::new("/p/a")).unfold(FolderKind::Processes);
        project.folders_mut(Path::new("/p/b")).unfold(FolderKind::Processes);

        // when
        project.apply(2, Ok(scan(Path::new("/p"), &["a"])));

        // then
        let checkouts: Vec<PathBuf> = project.saved().folders.into_iter().map(|folders| folders.checkout).collect();
        assert_eq!(checkouts, [PathBuf::from("/p/a")]);
    }

    #[test]
    fn should_read_the_files_of_a_shown_checkout_only_after_the_first_scan() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        std::fs::write(root.join("a.txt"), "").unwrap();
        let mut project = OpenProject::placeholder(ProjectId::new("p"), root.clone());
        project.toggle();
        let before = project.folders_to_read();

        // when
        project.apply(1, Ok(ProjectScan::plain(root.clone())));
        let after = project.folders_to_read();
        project.read_files_now();

        // then
        assert_eq!((before, after), (Vec::new(), vec![root.clone()]));
        assert_eq!(project.files(&root).map(|files| files.rows().len()), Some(1));
        assert!(project.folders_to_read().is_empty());
    }

    #[test]
    fn should_watch_the_shown_checkouts_folders_and_the_git_dir() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        std::fs::create_dir_all(root.join("wt/src")).unwrap();
        let mut project = OpenProject::placeholder(ProjectId::new("p"), root.clone());
        project.apply(1, Ok(scan(&root, &["wt"])));
        project.reveal(&root.join("wt"));
        project.read_files_now();

        // when
        let watched = project.watched();

        // then
        assert_eq!(watched, [Watched::Folder(root.join("wt")), Watched::GitDir(root.join(".git"))]);
    }

    #[test]
    fn should_read_a_tree_shown_again_whole_since_it_was_not_watched() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        std::fs::create_dir_all(root.join("wt/src")).unwrap();
        let mut project = OpenProject::placeholder(ProjectId::new("p"), root.clone());
        project.apply(1, Ok(scan(&root, &["wt"])));
        project.reveal(&root.join("wt"));
        project.read_files_now();
        project.toggle_folder(&root.join("wt"), &root.join("wt/src"));
        project.toggle_checkout(&root.join("wt"));
        project.folders_to_read();

        // when
        project.toggle_checkout(&root.join("wt"));
        let reads = project.folders_to_read();

        // then
        assert_eq!(reads, [root.join("wt"), root.join("wt/src")]);
    }

    #[cfg(unix)]
    #[test]
    fn should_leave_checkout_paths_out_of_the_saved_state_when_they_are_not_utf8() {
        // given
        use std::os::unix::ffi::OsStrExt;
        let odd = PathBuf::from(std::ffi::OsStr::from_bytes(b"/p/\xffwt"));
        let mut project = OpenProject::placeholder(ProjectId::new("p"), PathBuf::from("/p"));
        project.reveal(&odd);
        project.reveal(Path::new("/p/wt"));
        project.last_checkout = Some(odd);

        // when
        let saved = project.saved();

        // then
        assert_eq!((saved.expanded_checkouts, saved.last_checkout), (vec![PathBuf::from("/p/wt")], None));
    }

    #[test]
    fn should_mark_project_missing_when_its_folder_is_gone() {
        // given
        let saved = SavedProject {
            path: PathBuf::from("/nonexistent/ive-project"),
            id: "gone".to_owned(),
            expanded: true,
            expanded_checkouts: Vec::new(),
            last_checkout: None,
            folders: Vec::new(),
            hidden_worktrees: Vec::new(),
            tabs_folded: false,
            worktree_order: Vec::new(),
        };
        let mut project = OpenProject::restored(&saved, ProjectId::new("gone"));
        let gone = std::io::Error::from(std::io::ErrorKind::NotFound);

        // when
        project.apply(1, Err(ProjectError::Unreadable { path: saved.path.clone(), source: gone }));

        // then
        assert!(project.missing);
        assert!(project.shown_checkouts().is_empty());
        assert_eq!(project.saved(), saved);
    }

    #[test]
    fn should_count_a_project_as_read_once_its_scan_found_the_folder_gone() {
        // given
        let mut project = OpenProject::placeholder(ProjectId::new("gone"), PathBuf::from("/nonexistent/ive-project"));
        let before = project.is_read();
        let gone = std::io::Error::from(std::io::ErrorKind::NotFound);

        // when
        project.apply(1, Err(ProjectError::Unreadable { path: PathBuf::from("/nonexistent/ive-project"), source: gone }));

        // then
        assert_eq!((before, project.is_read()), (false, true));
    }
}
