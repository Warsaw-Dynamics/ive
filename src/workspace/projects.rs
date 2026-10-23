//! Opening, refreshing and closing projects, and what the tree's project rows do.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use gpui::{App, Context, ExternalPaths, PathPromptOptions, PromptLevel, Task, Window};

use super::Workspace;
use crate::document::Document;
use crate::editor::{FileTab, Tab};
use crate::file_tree::{Entry, EntryKind, FileTreeError, Listing, read_folder};
use crate::open_project::{OpenProject, PagedList};
use crate::overview::ProjectOverview;
use crate::project::{self, CheckoutRef, ProjectError, ProjectId, ProjectScan, Statuses};
use crate::project_list::{self, NewProject};
use crate::sidebar::rows::TreeKey;
use crate::sidebar::selection::{self, Listed, Selection};
use crate::state::{self, SavedState};
use crate::tabs::TabId;

/// What to show once a folder's project is open.
pub enum AfterOpen {
    ProjectOverview,
    /// the checkout holding the folder, expanded and selected in the tree
    RevealCheckout,
    /// the checkout holding the folder, listed collapsed under its expanded project; nothing selected
    ListCheckout,
    OpenFile(PathBuf),
    /// the checkout holding the folder, expanded and selected, with a new terminal in it
    NewTerminal,
}

/// What a scan read, with the number it was started as: of two scans of a project, the later-started one wins.
struct Scanned {
    number: u64,
    result: Result<ProjectScan, ProjectError>,
}

/// A refresh about to start: the number of its scan and whose status it reads.
struct Started {
    number: u64,
    statuses: Statuses,
}

/// What the background part of opening a folder read.
struct Opened {
    /// canonical
    folder: PathBuf,
    scan: Result<ProjectScan, ProjectError>,
    after: AfterOpen,
    ids_on_disk: Vec<String>,
}

impl Opened {
    /// Reads git, the folder and the config folder; run it on the background executor.
    #[tracing::instrument(level = "debug", name = "open_project", skip_all, fields(folder = %folder.display()))]
    fn read(folder: PathBuf, after: AfterOpen) -> Self {
        let scan = project::scan_opened(&folder);
        let folder = dunce::canonicalize(&folder).unwrap_or(folder);
        // tree rows and tabs hold canonical paths
        let after = match after {
            AfterOpen::OpenFile(file) => AfterOpen::OpenFile(dunce::canonicalize(&file).unwrap_or(file)),
            overview => overview,
        };
        Self { folder, scan, after, ids_on_disk: state::ids_with_data() }
    }
}

impl Workspace {
    /// Shows the projects of `state.toml` at once, as they were, and reads each from disk in the background.
    pub(super) fn restore(&mut self, saved: &SavedState, cx: &mut Context<Self>) {
        for project in &saved.projects {
            // ids were validated when the file was read
            if let Ok(id) = ProjectId::parse(&project.id) {
                let mut open = OpenProject::restored(project, id);
                open.set_every_folder(&self.shown_folders);
                self.projects.push(open);
            }
        }
        self.forget_stale_hidden();
        self.refresh_all(cx);
        self.tree_changed(cx);
    }

    pub fn prompt_for_folder(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.close_menu(window, cx);
        let picked = cx.prompt_for_paths(PathPromptOptions { files: false, directories: true, multiple: false, prompt: Some("Open".into()) });
        cx.spawn_in(window, async move |this, cx| {
            let folder = match picked.await {
                Ok(Ok(Some(paths))) => paths.into_iter().next(),
                Ok(Err(error)) => {
                    let error = ProjectError::PickerFailed(error.to_string());
                    this.update_in(cx, |workspace, window, cx| workspace.show_open_error(&error, window, cx)).ok();
                    None
                }
                // cancelled by the user, or the window closed while the picker was open
                Ok(Ok(None)) | Err(_) => None,
            };
            if let Some(folder) = folder {
                this.update_in(cx, |workspace, window, cx| workspace.open_folder(folder, AfterOpen::ProjectOverview, window, cx)).ok();
            }
        })
        .detach();
    }

    pub(crate) fn open_recent(&mut self, path: PathBuf, window: &mut Window, cx: &mut Context<Self>) {
        self.close_menu(window, cx);
        self.open_folder(path, AfterOpen::ProjectOverview, window, cx);
    }

    /// Opens the project of `folder` (adding it when new) in the background, expands it to the checkout holding
    /// `folder`, then does `after`.
    pub fn open_folder(&mut self, folder: PathBuf, after: AfterOpen, window: &mut Window, cx: &mut Context<Self>) {
        self.open_folder_task(folder, after, window, cx).detach();
    }

    /// As [`Self::open_folder`]; the task ends with the project's id, or with why the folder could not open.
    pub fn open_folder_task(&mut self, folder: PathBuf, after: AfterOpen, window: &mut Window, cx: &mut Context<Self>) -> Task<Result<ProjectId, String>> {
        let number = self.next_scan();
        let opened = cx.background_executor().spawn(async move { Opened::read(folder, after) });
        cx.spawn_in(window, async move |this, cx| {
            let opened = opened.await;
            this.update_in(cx, |workspace, window, cx| workspace.finish_open(number, opened, window, cx))
                .unwrap_or_else(|_| Err("the IVE window is closed".to_owned()))
        })
    }

    fn finish_open(&mut self, number: u64, opened: Opened, window: &mut Window, cx: &mut Context<Self>) -> Result<ProjectId, String> {
        let new = opened.scan.map(|scan| NewProject { scan, number, ids_on_disk: opened.ids_on_disk });
        let placed = new.and_then(|new| self.place_opened(new, &opened.folder, cx));
        let id = match placed {
            Ok(id) => id,
            Err(error) => {
                self.show_open_error(&error, window, cx);
                return Err(error.to_string());
            }
        };
        // the project list or Recent may have changed
        self.schedule_save(cx);
        if let Some(checkout) = self.checkout_holding(&id, &opened.folder) {
            // opening a hidden worktree's own folder asks for it: it is shown again
            if checkout.path == opened.folder {
                self.show_worktree(&checkout, cx);
            }
            if matches!(opened.after, AfterOpen::ListCheckout) {
                self.list_checkout(&checkout, cx);
                return Ok(id);
            }
            self.expand_checkout(&checkout, cx);
            match opened.after {
                AfterOpen::ProjectOverview => {
                    self.show_project_overview(&id, window, cx);
                }
                AfterOpen::RevealCheckout => self.reveal_checkout(&checkout, cx),
                AfterOpen::ListCheckout => {}
                AfterOpen::OpenFile(file) => self.open_document(&file, window, cx),
                AfterOpen::NewTerminal => {
                    self.reveal_checkout(&checkout, cx);
                    self.open_terminal_in(Some(checkout), window, cx);
                }
            }
        }
        Ok(id)
    }

    /// Puts a scanned folder in the tree: the open project it belongs to takes the scan as it takes a refresh's
    /// (the later-started scan wins), a new one is added at the end.
    fn place_opened(&mut self, new: NewProject, folder: &Path, cx: &mut Context<Self>) -> Result<ProjectId, ProjectError> {
        let placed = self.place_scanned(new, folder, cx);
        // a project opened shows, though the picker left it out
        if let Ok(id) = &placed {
            self.hidden_projects.remove(id);
        }
        placed
    }

    fn place_scanned(&mut self, new: NewProject, folder: &Path, cx: &mut Context<Self>) -> Result<ProjectId, ProjectError> {
        match project_list::holding(&self.projects, folder, &new.scan).map(|open| open.project.id.clone()) {
            Some(id) => {
                self.apply_scan(&id, Scanned { number: new.number, result: Ok(new.scan) }, cx);
                Ok(id)
            }
            None => {
                let id = project_list::add(&mut self.projects, &mut self.recent, new)?;
                let every = self.shown_folders.clone();
                if let Some(open) = self.open_project_mut(&id) {
                    open.set_every_folder(&every);
                }
                Ok(id)
            }
        }
    }

    /// The checkout of project `id` holding `folder`, else its main checkout.
    fn checkout_holding(&self, id: &ProjectId, folder: &Path) -> Option<CheckoutRef> {
        let project = self.project(id)?;
        let checkout = project.checkout_containing(folder).unwrap_or(project.main_checkout());
        Some(project.checkout_ref(checkout))
    }

    /// Expands the project and, with a worktree level, the checkout row, and reads the files it now shows; the
    /// expansion is saved.
    /// The checkout's row in sight under its expanded project, collapsed.
    fn list_checkout(&mut self, checkout: &CheckoutRef, cx: &mut Context<Self>) {
        if let Some(open) = self.open_project_mut(&checkout.project) {
            open.list_checkout(&checkout.path);
            self.tree_changed(cx);
            self.schedule_save(cx);
        }
    }

    pub fn expand_checkout(&mut self, checkout: &CheckoutRef, cx: &mut Context<Self>) {
        if let Some(open) = self.open_project_mut(&checkout.project) {
            open.reveal(&checkout.path);
            self.read_shown_files(cx);
            self.tree_changed(cx);
            self.schedule_save(cx);
        }
    }

    /// Where an open failed: on the welcome screen, or in a tab once there is a sidebar.
    fn show_open_error(&mut self, error: &ProjectError, window: &mut Window, cx: &mut Context<Self>) {
        if self.is_welcome() {
            self.welcome_error = Some(error.to_string().into());
            cx.notify();
            return;
        }
        self.open_tab(Tab::Failed { title: "Open project".to_owned(), error: error.to_string().into() }, window, cx);
    }

    /// Re-reads a project's checkouts and the status of every checkout in the background; while one read runs,
    /// one more is queued.
    pub fn refresh_project(&mut self, id: &ProjectId, cx: &mut Context<Self>) {
        self.refresh_checkouts(id, Statuses::All, cx);
    }

    /// Re-reads a project's checkouts and branches, and the status of the checkouts `statuses` names; the others
    /// keep their last status.
    pub fn refresh_checkouts(&mut self, id: &ProjectId, statuses: Statuses, cx: &mut Context<Self>) {
        let number = self.next_scan();
        let Some(open) = self.open_project_mut(id) else {
            return;
        };
        if let Some(statuses) = open.request_refresh(statuses, number) {
            self.spawn_refresh(id.clone(), Started { number, statuses }, cx);
        }
    }

    pub(super) fn refresh_all(&mut self, cx: &mut Context<Self>) {
        let ids: Vec<ProjectId> = self.projects.iter().map(|open| open.project.id.clone()).collect();
        for id in ids {
            self.refresh_project(&id, cx);
        }
    }

    /// Back in the window: another tool may have added worktrees or switched branches, so every project's
    /// checkouts are listed again; status, the slow part, only where it is on screen, and the git folders the tree
    /// shows, which are not polled meanwhile.
    pub(crate) fn refresh_on_activation(&mut self, cx: &mut Context<Self>) {
        let shown = self.statuses_on_screen();
        let ids: Vec<ProjectId> = self.projects.iter().map(|open| open.project.id.clone()).collect();
        for id in ids {
            let paths = shown.iter().filter(|checkout| checkout.project == id).map(|checkout| checkout.path.clone());
            self.refresh_checkouts(&id, Statuses::Only(paths.collect()), cx);
        }
        self.reread_tree_git(cx);
    }

    /// Checkouts whose status the shown tab and the detached ones display.
    pub fn statuses_on_screen(&self) -> Vec<CheckoutRef> {
        self.tabs.on_screen().flat_map(|(_, tab)| tab.shown_statuses(self)).collect()
    }

    fn spawn_refresh(&mut self, id: ProjectId, started: Started, cx: &mut Context<Self>) {
        let Some(root) = self.project(&id).map(|project| project.root.clone()) else {
            return;
        };
        let Started { number, statuses } = started;
        let since = self.status_since();
        let scan = cx.background_executor().spawn(async move { project::scan(&root, &statuses, since) });
        cx.spawn(async move |this, cx| {
            let result = scan.await;
            this.update(cx, |workspace, cx| workspace.finish_refresh(&id, Scanned { number, result }, cx)).ok();
        })
        .detach();
    }

    fn finish_refresh(&mut self, id: &ProjectId, scanned: Scanned, cx: &mut Context<Self>) {
        let (number, read) = (scanned.number, scanned.result.is_ok());
        let applied = self.apply_scan(id, scanned, cx);
        let Some(open) = self.open_project_mut(id) else {
            return;
        };
        if let Some(queued) = open.finish_refresh(number) {
            self.refresh_checkouts(id, queued, cx);
        }
        if applied && read {
            self.project_refreshed(id, cx);
        }
        if applied {
            self.tree_changed(cx);
        }
    }

    /// Hands a scan to project `id`; says whether it was applied, i.e. not started before the last one applied.
    /// A scan that changed what is saved (a root, an expanded checkout that is gone) is saved; terminals of a
    /// checkout it no longer lists move to the main checkout.
    fn apply_scan(&mut self, id: &ProjectId, scanned: Scanned, cx: &mut Context<Self>) -> bool {
        let Some(open) = self.open_project_mut(id) else {
            return false;
        };
        let before = open.saved();
        let applied = open.apply(scanned.number, scanned.result);
        let saved_changed = open.saved() != before;
        let project = &open.project;
        let main = project.checkout_ref(project.main_checkout());
        let listed: HashSet<PathBuf> = project.checkouts.iter().map(|checkout| checkout.path.clone()).collect();
        if applied {
            let orphans = self.processes.move_unlisted_to(&main, |path| listed.contains(path));
            self.end_orphaned_runs(orphans, cx);
            self.follow_all_directories(cx);
            self.read_shown_files(cx);
        }
        if saved_changed {
            self.schedule_save(cx);
        }
        applied
    }

    /// Reads what the tree shows but has not read: the top folders of checkouts shown without files, and the
    /// folders of trees shown again.
    fn read_shown_files(&mut self, cx: &mut Context<Self>) {
        let folders: Vec<PathBuf> = self.projects.iter_mut().flat_map(OpenProject::folders_to_read).collect();
        self.read_folders(folders, cx);
    }

    /// Reads `folders` on the background executor, then hands the listings to every file tree in one update.
    pub(super) fn read_folders(&mut self, folders: Vec<PathBuf>, cx: &mut Context<Self>) {
        if folders.is_empty() {
            return;
        }
        let listings = read_in_background(folders, cx);
        cx.spawn(async move |this, cx| {
            let listings = listings.await;
            this.update(cx, |workspace, cx| workspace.apply_listings(&listings, cx)).ok();
        })
        .detach();
    }

    /// Takes what reading folders found, in every project at once, and rebuilds the rows once.
    pub(super) fn apply_listings(&mut self, listings: &[Listing], cx: &mut Context<Self>) {
        let mut errors = Vec::new();
        for open in &mut self.projects {
            errors.extend(open.apply_listings(listings));
        }
        // a folder that cannot be read collapses again, so locating would ask for it forever
        if !errors.is_empty() {
            self.locating = None;
        }
        report_unreadable(errors);
        // folders expanded again on restore show once their parents are read
        self.read_shown_files(cx);
        self.tree_changed(cx);
        self.continue_locating(cx);
    }

    /// Closes project `id` after a confirmation when processes run in it: it moves to Recent, its processes end
    /// and its tabs close.
    pub fn close_project(&mut self, id: &ProjectId, window: &mut Window, cx: &mut Context<Self>) {
        let running = self.processes.live_in_project(id);
        if running == 0 {
            return self.finish_close_project(id, window, cx);
        }
        let name = self.project(id).map_or_else(|| id.to_string(), |project| project.name.clone());
        let detail = match running {
            1 => "1 process is running in it; closing the project ends it.".to_owned(),
            count => format!("{count} processes are running in it; closing the project ends them."),
        };
        let answer = window.prompt(PromptLevel::Warning, &format!("Close {name}?"), Some(&detail), &["Close project", "Cancel"], cx);
        let id = id.clone();
        cx.spawn_in(window, async move |this, cx| {
            if answer.await == Ok(0) {
                this.update_in(cx, |workspace, window, cx| workspace.finish_close_project(&id, window, cx)).ok();
            }
        })
        .detach();
    }

    fn finish_close_project(&mut self, id: &ProjectId, window: &mut Window, cx: &mut Context<Self>) {
        let tabs: Vec<_> = self
            .tabs
            .iter()
            .filter(|(_, tab)| tab.project(self).as_ref() == Some(id))
            .map(|(tab_id, _)| tab_id)
            .collect();
        // file tabs with edits close once those are saved, and the project with the last of its tabs
        let close = self.close_tabs(tabs, window, cx);
        self.waits.close_project(close, id.clone());
        self.finish_closing_projects(window, cx);
    }

    /// Closes the projects the live close waits for whose tabs are all closed now.
    pub(super) fn finish_closing_projects(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let ready: Vec<_> = self.waits.projects().into_iter().filter(|id| !self.has_tabs_of(id)).collect();
        for id in ready {
            self.waits.project_closed(&id);
            self.close_project_now(&id, window, cx);
        }
    }

    fn has_tabs_of(&self, id: &ProjectId) -> bool {
        self.tabs.iter().any(|(_, tab)| tab.project(self).as_ref() == Some(id))
    }

    /// Closes project `id`, which has no tabs left: its processes end and it moves to Recent.
    fn close_project_now(&mut self, id: &ProjectId, window: &mut Window, cx: &mut Context<Self>) {
        let processes: Vec<_> = self.processes.in_project(id).iter().map(|process| process.id).collect();
        for process in processes {
            if let Some(found) = self.processes.get(process) {
                found.view.hang_up();
            }
            self.processes.close(process);
        }
        if project_list::close(&mut self.projects, &mut self.recent, id).is_none() {
            return;
        }
        self.forget_stale_hidden();
        self.project_closed(id, cx);
        self.tree_changed(cx);
        self.schedule_save(cx);
        self.tab_shown(window, cx);
    }

    pub(crate) fn toggle_project(&mut self, id: &ProjectId, cx: &mut Context<Self>) {
        if let Some(open) = self.open_project_mut(id) {
            open.toggle();
            self.read_shown_files(cx);
            self.tree_changed(cx);
            self.schedule_save(cx);
        }
    }

    pub(crate) fn toggle_checkout(&mut self, checkout: &CheckoutRef, cx: &mut Context<Self>) {
        if let Some(open) = self.open_project_mut(&checkout.project) {
            open.toggle_checkout(&checkout.path);
            self.read_shown_files(cx);
            self.tree_changed(cx);
            self.schedule_save(cx);
        }
    }

    /// Leaves the worktree out of the tree and the hub until shown again; its tabs and processes stay.
    pub(crate) fn hide_worktree(&mut self, checkout: &CheckoutRef, cx: &mut Context<Self>) {
        if self.open_project_mut(&checkout.project).is_some_and(|open| open.hide(&checkout.path)) {
            self.worktrees_listed(&checkout.project, cx);
        }
    }

    pub(crate) fn show_worktree(&mut self, checkout: &CheckoutRef, cx: &mut Context<Self>) {
        if self.open_project_mut(&checkout.project).is_some_and(|open| open.show(&checkout.path)) {
            self.worktrees_listed(&checkout.project, cx);
        }
    }

    /// A worktree was hidden or shown: the tree, the hub's index and the saved state follow.
    fn worktrees_listed(&mut self, project: &ProjectId, cx: &mut Context<Self>) {
        self.read_shown_files(cx);
        self.tree_changed(cx);
        self.index_project(project, cx);
        self.forget_closed_checkouts(cx);
        self.schedule_save(cx);
        cx.notify();
    }

    pub(crate) fn show_more(&mut self, project: &ProjectId, list: &PagedList, cx: &mut Context<Self>) {
        if let Some(open) = self.open_project_mut(project) {
            open.show_more(list);
            self.tree_changed(cx);
        }
    }

    pub(crate) fn show_all(&mut self, project: &ProjectId, list: &PagedList, cx: &mut Context<Self>) {
        if let Some(open) = self.open_project_mut(project) {
            open.show_all(list);
            self.tree_changed(cx);
        }
    }

    /// A folder or hidden group row toggles; a file row shows the file, in its open tab or a new one.
    /// Changes the tree's selection by `change`, which also gets the rows as the selection sees them.
    pub(crate) fn change_selection<R>(&mut self, change: impl FnOnce(&mut Selection, &[Listed]) -> R) -> R {
        let listed = selection::listed(&self.tree.rows);
        change(&mut self.tree.selection, &listed)
    }

    /// What dragging the tree row of `pressed` drags, see [`Selection::dragged_paths`].
    pub(crate) fn dragged_paths(&self, pressed: &Path) -> Vec<PathBuf> {
        self.tree.selection.dragged_paths(pressed, &selection::listed(&self.tree.rows))
    }

    /// Copies the absolute paths of the tree's selected files and folders, one per line in the tree's order.
    pub(crate) fn copy_selected_paths(&self, cx: &mut App) {
        let paths = self.tree.selection.listed_paths(&selection::listed(&self.tree.rows));
        if paths.is_empty() {
            return;
        }
        let lines: Vec<String> = paths.iter().map(|path| path.display().to_string()).collect();
        cx.write_to_clipboard(gpui::ClipboardItem::new_string(lines.join("\n")));
    }

    pub(crate) fn activate_entry(&mut self, checkout: &CheckoutRef, entry: &Entry, window: &mut Window, cx: &mut Context<Self>) {
        self.tree.selection.select_only(TreeKey::Entry(entry.path.clone()));
        let Some(open) = self.open_project_mut(&checkout.project) else {
            return;
        };
        let folders = match entry.kind {
            EntryKind::File => return self.open_document(&entry.path, window, cx),
            EntryKind::Folder => open.toggle_folder(&checkout.path, &entry.path),
            EntryKind::HiddenGroup => open.toggle_hidden(&checkout.path, &entry.path),
        };
        self.read_expanded(folders, entry.name.clone(), window, cx);
        self.tree_changed(cx);
        self.schedule_save(cx);
    }

    /// Reads folders the user just expanded; one that cannot be read collapses again, and a tab says why.
    fn read_expanded(&mut self, folders: Vec<PathBuf>, title: String, window: &mut Window, cx: &mut Context<Self>) {
        if folders.is_empty() {
            return;
        }
        let listings = read_in_background(folders, cx);
        cx.spawn_in(window, async move |this, cx| {
            let listings = listings.await;
            let failed = listings.iter().find_map(|listing| listing.entries.as_ref().err().cloned());
            this.update_in(cx, |workspace, window, cx| {
                workspace.apply_listings(&listings, cx);
                if let Some(error) = failed {
                    workspace.open_tab(Tab::Failed { title, error: error.to_string().into() }, window, cx);
                }
            })
            .ok();
        })
        .detach();
    }

    /// Opens what was dropped from a file manager: files as tabs, folders as projects.
    pub(crate) fn open_dropped(&mut self, dropped: &ExternalPaths, window: &mut Window, cx: &mut Context<Self>) {
        for path in dropped.paths() {
            if path.is_dir() {
                self.open_folder(path.clone(), AfterOpen::ProjectOverview, window, cx);
            } else {
                self.open_document(path, window, cx);
            }
        }
    }

    /// Shows `path` in its tab, opening one when needed; a file that cannot be read gets a failed tab.
    pub(crate) fn open_document(&mut self, path: &Path, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(open) = self.tabs.find(|tab| tab.shows(path)) {
            return self.activate_tab(open, window, cx);
        }
        let name = project::folder_name(path);
        let tab = match Document::open_stamped(path) {
            Ok(opened) => Tab::Document(FileTab::new(opened, cx)),
            Err(error) => Tab::Failed { title: name, error: error.to_string().into() },
        };
        self.open_tab(tab, window, cx);
    }

    /// Shows the project's overview tab, opening it when needed; `None` when the project is not open.
    pub fn show_project_overview(&mut self, id: &ProjectId, window: &mut Window, cx: &mut Context<Self>) -> Option<TabId> {
        self.project(id)?;
        let tab = match self.tabs.find(|tab| tab.is_project_overview(id)) {
            Some(tab) => {
                self.activate_tab(tab, window, cx);
                tab
            }
            None => {
                let git = self.project_views(id, window, cx);
                self.open_tab(Tab::ProjectOverview(ProjectOverview::new(id.clone(), git)), window, cx)
            }
        };
        Some(tab)
    }

    /// Expands the checkout in the tree, selects its row (the project's, without a worktree level) and scrolls to it.
    pub fn reveal_checkout(&mut self, checkout: &CheckoutRef, cx: &mut Context<Self>) {
        let Some(open) = self.open_project(&checkout.project) else {
            return;
        };
        let key = match open.has_worktree_level() {
            true => TreeKey::Checkout(checkout.path.clone()),
            false => TreeKey::Project(checkout.project.clone()),
        };
        self.expand_checkout(checkout, cx);
        self.locating = Some(key);
        self.continue_locating(cx);
    }
}

/// Lists `folders` on the background executor.
pub(super) fn read_in_background(folders: Vec<PathBuf>, cx: &mut Context<Workspace>) -> Task<Vec<Listing>> {
    cx.background_executor().spawn(async move { folders.iter().map(|dir| read_folder(dir)).collect() })
}

/// A folder IVE could not list shows no files; the reason goes to stderr.
pub(super) fn report_unreadable(errors: Vec<FileTreeError>) {
    for error in errors {
        eprintln!("ive: {error}");
    }
}
