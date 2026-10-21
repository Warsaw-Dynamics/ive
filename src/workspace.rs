use std::cell::Cell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use futures::channel::mpsc::UnboundedReceiver;
use gpui::{
    AnyWindowHandle, App, Context, CursorStyle, Div, ExternalPaths, FocusHandle, Img, IntoElement, KeyDownEvent, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Render, SharedString,
    ScrollHandle, Stateful, Subscription, Svg, Task, Window, canvas, div, img, prelude::*, rgb, svg,
};

use crate::click_targets::{ClickTargets, target};
use crate::age::Clock;
use crate::editor::{self, EditorView, Tab};
use crate::hub::corpus::Corpus;
use crate::hub::history::{History, Shown, WorktreeUse};
use crate::hub::index::HubIndex;
use crate::hub::sources::HubScope;
use crate::syntax::{ParseError, Syntax};
use crate::frame;
use crate::icons;
use crate::api::Call;
use crate::script::{self, Scripts};
use crate::script::temporary_tabs::TemporaryTabs;
use crate::open_project::OpenProject;
use crate::process::{ProcessId, Processes, RunId};
use crate::project::{Checkout, CheckoutRef, Project, ProjectId};
use crate::project_list;
use crate::settings::SettingsStore;
use crate::sidebar::folders::FolderKind;
use crate::sidebar::rows::{self, TreeKey};
use crate::sidebar::view::{self as sidebar_view, Sidebar, Tree};
use crate::sidebar::{self, view::badge};
use crate::state::{RecentProject, SavedState};
#[cfg(unix)]
use crate::terminal::hosted::HostedTerminals;
use crate::tabs::{TabId, Tabs};
use crate::theme::{self, Theme};
use crate::window_controls::window_controls;
use crate::window_drag::WindowDrag;
use crate::window_state::WindowPlace;

mod changes;
mod follow;
mod git_views;
mod hooks;
mod hub;
mod hub_index;
mod locate;
mod api_requests;
mod persist;
mod processes;
mod projects;
mod quit;
mod restart;
mod runs;
mod scratches;
mod scripts;
mod settings_actions;
mod tab_menu;
mod tab_state;
mod tab_windows;
mod tree_git;
mod tree_status;
mod drag_preview;
mod worktree_list;

pub use drag_preview::TabDragArea;
#[cfg(target_os = "macos")]
pub use quit::{add_app_menu, show_main_window};
pub use processes::ProgramEnding;
pub use projects::AfterOpen;
pub use tab_menu::{DEFAULT_MENU, MenuLine, label as tab_command_label, target as tab_command_target};

/// Main window: the projects sidebar and the editor tabs, or a welcome screen while no project and no tab is open.
pub struct Workspace {
    /// in tree order
    projects: Vec<OpenProject>,
    /// closed projects, newest first
    recent: Vec<RecentProject>,
    processes: Processes,
    #[cfg(unix)]
    hosted: Option<HostedTerminals>,
    tabs: Tabs<Tab>,
    /// the gap between tabs a dragged tab would land in, while it is over one that moves it
    tab_drop_gap: Option<usize>,
    /// a tab being dragged, which a release away from the tab row detaches
    tab_drag: Option<tab_windows::TabDrag>,
    drag_preview: Option<drag_preview::DragPreview>,
    /// the tab row's sideways scroll, so the shown tab can be scrolled into view
    tab_scroll: ScrollHandle,
    /// shared with the chip drawn under the pointer, which hides while the preview stands for it
    drag_chip_hidden: Rc<Cell<bool>>,
    tab_windows: tab_windows::TabWindows,
    /// this workspace's own window, which the tab row is in; detached tabs have windows of their own
    main_window: AnyWindowHandle,
    tree: Tree,
    tree_statuses: tree_status::TreeStatuses,
    tree_git: tree_git::TreeGit,
    /// the Open project menu of the sidebar header
    menu_open: bool,
    /// the menu was open when the + button was pressed: that press closed it, so the click must not reopen it
    menu_open_at_press: bool,
    /// the menu a right click on a tab opened
    tab_menu: Option<tab_menu::TabMenu>,
    /// the row Select Opened File is expanding the tree towards, while folders above it are read
    locating: Option<TreeKey>,
    /// `--restart`: the start script the Restart button runs
    restart_script: Option<PathBuf>,
    /// the terminal the Restart button runs the start script in, reused by later presses
    restart_terminal: Option<ProcessId>,
    tab_ownership: tab_state::TabOwnership,
    welcome_error: Option<SharedString>,
    window_drag: WindowDrag,
    window_place: WindowPlace,
    focus_handle: FocusHandle,
    state: persist::StateFile,
    /// what the user dragged the sidebar to; drawn fitted to the window
    sidebar_width: f32,
    sidebar_resize: Option<sidebar::Resize>,
    /// numbers each scan of a project as it starts, so a scan started earlier never replaces a later one
    scans: u64,
    _subscriptions: Vec<Subscription>,
    _foreground_poll: Option<Task<()>>,
    /// redraws while a process reports indeterminate progress, so its bar moves
    progress_ticker: Option<Task<()>>,
    watches: changes::Watches,
    /// part 2's views, kept while IVE runs so they remember what was selected
    git: git_views::GitViews,
    _watch_events: Task<()>,
    _watch_tick: Task<()>,
    /// answers API calls (MCP, scripts)
    _api_calls: Task<()>,
    scripts: scripts::ScriptWatch,
    quit_prompt: Option<quit::QuitPrompt>,
    /// runs waiting for their `before` steps
    pending: runs::PendingRuns,
    /// the tabs shown, for the hub's Recent rows
    history: History,
    /// the last input or run start per checkout, for the hub's worktree order
    worktree_use: WorktreeUse,
    hub_index: HubIndex,
    /// every open project's files, as the hub searches them
    corpus: std::sync::Arc<Corpus>,
    /// the projects `corpus` was built from, in its order
    corpus_projects: Vec<ProjectId>,
    corpus_build: hub_index::CorpusBuild,
    /// the hub's scope, kept from one Ctrl+Shift+E to the next
    hub_scope: HubScope,
    /// worktrees the new terminal picker is creating
    worktree_creations: crate::hub::creations::Creations,
    /// the tab opened from the hub a moment ago, while it is the one shown
    hub_return: Option<hub::HubReturn>,
    _signals: Option<Task<()>>,
}

/// The hub's keys, seen before its query field's own bindings (Enter, Backspace) take them.
fn hub_keys(cx: &mut Context<Workspace>) -> Subscription {
    let workspace = cx.entity().downgrade();
    cx.intercept_keystrokes(move |event, window, cx| {
        if !event.context_stack.iter().any(|context| context.contains("Hub")) {
            return;
        }
        let taken = workspace.update(cx, |workspace, cx| workspace.hub_keystroke(&event.keystroke, window, cx)).unwrap_or(false);
        if taken {
            cx.stop_propagation();
            window.refresh();
        }
    })
}

/// The Worktrees list's keys, seen before its filter field takes them (Space, Enter).
fn worktree_list_keys(cx: &mut Context<Workspace>) -> Subscription {
    let workspace = cx.entity().downgrade();
    cx.intercept_keystrokes(move |event, window, cx| {
        if !event.context_stack.iter().any(|context| context.contains("WorktreeList")) {
            return;
        }
        let taken = workspace.update(cx, |workspace, cx| workspace.worktree_list_keystroke(&event.keystroke, window, cx)).unwrap_or(false);
        if taken {
            cx.stop_propagation();
            window.refresh();
        }
    })
}

/// What the window starts with.
pub struct Startup {
    pub launch: Launch,
    pub saved: SavedState,
    /// where saved state goes; `None` when the OS has no config folder
    pub state_path: Option<PathBuf>,
    /// calls from MCP and scripts
    pub api_calls: UnboundedReceiver<Call>,
    pub scripts: Scripts,
    /// set when this IVE owns the hosted terminals: shells then outlive IVE
    #[cfg(unix)]
    pub hosted: Option<HostedTerminals>,
    /// SIGTERM and SIGINT, which go through the quit prompt, and SIGUSR1, the restart script's quit
    pub signals: Option<UnboundedReceiver<i32>>,
    /// `--restart`: shows the Restart button, which runs this start script
    pub restart_script: Option<PathBuf>,
}

impl Workspace {
    /// Restores the saved projects, then opens what `startup.launch` names.
    pub fn new(startup: Startup, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        window.focus(&focus_handle, cx);
        cx.set_global(theme::SidebarFocus(cx.focus_handle()));
        let subscriptions = vec![
            cx.observe_window_appearance(window, |_workspace, window, cx| {
                theme::apply(window.appearance(), cx);
            }),
            cx.observe_window_activation(window, |workspace, window, cx| {
                // the sidebar and the tabs draw by whether the window is active
                cx.notify();
                if window.is_window_active() {
                    workspace.refresh_on_activation(cx);
                }
            }),
            cx.observe_window_visibility(window, |workspace, visibility, window, _cx| {
                if visibility.is_visible() {
                    workspace.window_place.shown(window);
                }
            }),
            // the git folders follow the Git sections' Flat/Tree
            cx.observe_global::<SettingsStore>(|workspace, cx| workspace.tree_changed(cx)),
            cx.on_release(|workspace, cx| workspace.flush_state(cx)),
            hub_keys(cx),
            worktree_list_keys(cx),
            cx.on_app_quit(|workspace, cx| {
                workspace.flush_state(cx);
                async {}
            }),
        ];
        let (mut watches, events) = tracing::info_span!("start_watches").in_scope(changes::Watches::start);
        let scripts = scripts::ScriptWatch::start(startup.scripts, script::default_dir(), &mut watches.paths, window, cx);
        let mut workspace = Self {
            projects: Vec::new(),
            recent: startup.saved.recent.clone(),
            processes: Processes::default(),
            #[cfg(unix)]
            hosted: startup.hosted,
            tabs: Tabs::default(),
            tab_drop_gap: None,
            tab_drag: None,
            drag_preview: None,
            tab_scroll: ScrollHandle::new(),
            drag_chip_hidden: Rc::default(),
            tab_windows: tab_windows::TabWindows::default(),
            main_window: window.window_handle(),
            tree: Tree::new(),
            tree_statuses: tree_status::TreeStatuses::default(),
            tree_git: tree_git::TreeGit::default(),
            menu_open: false,
            menu_open_at_press: false,
            tab_menu: None,
            locating: None,
            restart_script: startup.restart_script.clone(),
            restart_terminal: None,
            tab_ownership: tab_state::TabOwnership::Owner,
            welcome_error: None,
            window_drag: WindowDrag::default(),
            window_place: WindowPlace::new(startup.saved.window),
            focus_handle,
            state: persist::StateFile::new(startup.state_path, startup.saved.clone()),
            sidebar_width: cx.global::<SettingsStore>().current().sidebar_width.map_or(cx.global::<Theme>().sizes.tree.default_width.0, |width| width as f32),
            sidebar_resize: None,
            scans: 0,
            _subscriptions: subscriptions,
            _foreground_poll: processes::poll_foreground(cx),
            progress_ticker: None,
            watches,
            git: git_views::GitViews::default(),
            _watch_events: changes::forward_events(events, window, cx),
            _watch_tick: changes::tick(window, cx),
            _api_calls: api_requests::answer_calls(startup.api_calls, window, cx),
            scripts,
            quit_prompt: None,
            pending: runs::PendingRuns::default(),
            history: History::restore(startup.saved.history.clone()),
            worktree_use: WorktreeUse::restore(startup.saved.worktree_use.clone()),
            hub_index: HubIndex::default(),
            corpus: std::sync::Arc::default(),
            corpus_projects: Vec::new(),
            corpus_build: hub_index::CorpusBuild::default(),
            hub_scope: HubScope::All,
            worktree_creations: crate::hub::creations::Creations::default(),
            hub_return: None,
            _signals: startup.signals.map(|signals| quit::forward_signals(signals, window, cx)),
        };
        // a closing window goes through the quit prompt; IVE quits from there. Unix only: elsewhere terminals end
        // with IVE. On macOS the × hides the window instead and IVE keeps running in the Dock until quit, as Mac apps
        // do; a click on its Dock icon shows the window again.
        #[cfg(target_os = "macos")]
        window.on_window_should_close(cx, |window, _| {
            window.hide_window();
            false
        });
        #[cfg(all(unix, not(target_os = "macos")))]
        {
            let this = cx.entity().downgrade();
            window.on_window_should_close(cx, move |window, cx| this.update(cx, |workspace, cx| workspace.request_quit(window, cx)).is_err());
        }
        #[cfg(target_os = "macos")]
        {
            let this = cx.entity().downgrade();
            let handle = window.window_handle();
            cx.on_quit_request(move |cx| {
                let asked = handle.update(cx, |_, window, cx| this.update(cx, |workspace, cx| workspace.quit_requested_by_system(window, cx)));
                asked.ok().and_then(Result::ok).unwrap_or(true)
            });
        }
        tracing::info_span!("restore_projects").in_scope(|| workspace.restore(&startup.saved, cx));
        let tabs = startup.saved.tabs.clone();
        #[cfg(unix)]
        let sessions = {
            let adopted_tabs = if tabs.is_some() { processes::AdoptedTabs::AsSaved } else { processes::AdoptedTabs::Each };
            tracing::info_span!("adopt_sessions").in_scope(|| workspace.adopt_sessions(adopted_tabs, window, cx))
        };
        #[cfg(not(unix))]
        let sessions = std::collections::HashMap::new();
        // another IVE in this profile keeps the terminals: its tabs are its own
        #[cfg(unix)]
        if workspace.hosted.is_none() {
            workspace.tab_ownership = tab_state::TabOwnership::Guest(tabs.clone());
        }
        if let (Some(tabs), tab_state::TabOwnership::Owner) = (tabs, &workspace.tab_ownership) {
            tracing::info_span!("restore_tabs").in_scope(|| workspace.restore_tabs(tabs, &sessions, window, cx));
        }
        tracing::info_span!("launch").in_scope(|| workspace.start(startup.launch, window, cx));
        workspace.update_title(window);
        workspace
    }

    fn start(&mut self, launch: Launch, window: &mut Window, cx: &mut Context<Self>) {
        match launch {
            Launch::Restore => {}
            Launch::Folder(folder) => self.open_folder(folder, AfterOpen::RevealCheckout, window, cx),
            Launch::File(file) => {
                let folder = file.parent().map_or_else(|| PathBuf::from("."), PathBuf::from);
                self.open_folder(folder, AfterOpen::OpenFile(file), window, cx);
            }
        }
    }

    pub fn project(&self, id: &ProjectId) -> Option<&Project> {
        self.open_project(id).map(|open| &open.project)
    }

    pub fn processes(&self) -> &Processes {
        &self.processes
    }

    pub fn tabs(&self) -> &Tabs<Tab> {
        &self.tabs
    }

    pub(crate) fn tree(&self) -> &Tree {
        &self.tree
    }

    pub(crate) fn window_drag(&self) -> &WindowDrag {
        &self.window_drag
    }

    fn open_project(&self, id: &ProjectId) -> Option<&OpenProject> {
        self.projects.iter().find(|open| &open.project.id == id)
    }

    fn open_project_mut(&mut self, id: &ProjectId) -> Option<&mut OpenProject> {
        self.projects.iter_mut().find(|open| &open.project.id == id)
    }

    /// The number of a scan about to start.
    fn next_scan(&mut self) -> u64 {
        self.scans += 1;
        self.scans
    }

    /// The checkout whose folder holds `path`, across all projects; the deepest wins.
    pub fn checkout_of_path(&self, path: &Path) -> Option<CheckoutRef> {
        project_list::checkout_of_path(&self.projects, path)
    }

    /// The checkout `checkout` names while its project lists it; `None` once a refresh dropped it (a worktree
    /// deleted outside IVE) or the project closed.
    pub fn checkout(&self, checkout: &CheckoutRef) -> Option<&Checkout> {
        self.project(&checkout.project)?.checkout(&checkout.path)
    }

    /// The checkout's label, or its folder name once the checkout is gone.
    pub fn checkout_label(&self, checkout: &CheckoutRef) -> String {
        self.checkout(checkout).map_or_else(|| crate::project::folder_name(&checkout.path), |found| found.label.clone())
    }

    /// Where Alt+F12 opens a terminal: the shown tab's checkout, else the selected tree row's, else the first
    /// project's main checkout; `None` means the home folder.
    pub fn current_checkout(&self) -> Option<CheckoutRef> {
        let shown = self.tabs.active().and_then(|tab| tab.checkout(self));
        project_list::current_checkout(shown, self.selected_checkout(), &self.projects)
    }

    /// The checkout of the last shown tab that had one in `project`, while that checkout still exists.
    pub fn last_focused_checkout(&self, project: &ProjectId) -> Option<CheckoutRef> {
        let open = self.open_project(project)?;
        let path = open.last_checkout.as_ref()?;
        open.project.checkout(path).map(|checkout| open.project.checkout_ref(checkout))
    }

    /// The glow colour of the worktree being worked in: the selected tree row's, which stays while other tabs are
    /// shown, else the shown tab's; every open worktree has its own (up to nine).
    fn glow_index(&self) -> Option<usize> {
        let lit = self.selected_checkout().or_else(|| self.tabs.active().and_then(|tab| tab.checkout(self)))?;
        let checkouts: Vec<&Path> = self.projects.iter().flat_map(|open| open.project.checkouts.iter().map(|checkout| checkout.path.as_path())).collect();
        let at = checkouts.iter().position(|path| *path == lit.path)?;
        crate::glow::glow_indices(&checkouts).get(at).copied()
    }

    fn selected_checkout(&self) -> Option<CheckoutRef> {
        match self.tree.selection.anchor()? {
            TreeKey::Project(id) => self.project(id).map(|project| project.checkout_ref(project.main_checkout())),
            TreeKey::Checkout(path) | TreeKey::Folder(path, _) | TreeKey::Entry(path) => self.checkout_of_path(path),
            TreeKey::Change(path, ..) | TreeKey::ChangeFolder(path, ..) | TreeKey::Commit(path, _) | TreeKey::CommitsMore(path) | TreeKey::Note(path, _) => {
                self.checkout_of_path(path)
            }
            TreeKey::Process(id) | TreeKey::LiveRun(RunId::Process(id)) => self.processes.get(*id).and_then(|process| process.checkout.clone()),
            TreeKey::LiveRun(RunId::Pending(_)) | TreeKey::More(_) => None,
        }
    }

    /// Opens `tab` right of the shown one and shows it.
    /// Opens `tab` and shows it; temporary when `config.temporary_tabs` says so for its kind.
    pub fn open_tab(&mut self, tab: Tab, window: &mut Window, cx: &mut Context<Self>) -> TabId {
        let temporary = tab.temporary_kind().is_some_and(|kind| cx.try_global::<TemporaryTabs>().is_some_and(|tabs| tabs.opens_temporary(kind)));
        let id = if temporary { self.tabs.open_temporary(tab) } else { self.tabs.open(tab) };
        self.start_highlighting(id, cx);
        self.tab_shown(window, cx);
        id
    }

    fn start_highlighting(&mut self, id: TabId, cx: &mut Context<Self>) {
        let Some(Tab::Document(editor)) = self.tabs.get(id) else {
            return;
        };
        let Some(job) = editor.parse_job() else {
            return;
        };
        let path = editor.path().to_path_buf();
        let stale = job.cancel();
        let parsing = cx.background_executor().spawn(async move { job.run() });
        cx.spawn(async move |workspace, cx| {
            let result = parsing.await;
            // the text was re-read (or the tab closed) after the parse finished, before this update ran
            let result = if stale.is_cancelled() { Err(ParseError::Cancelled) } else { result };
            workspace.update(cx, |workspace, cx| workspace.finish_highlighting(id, &path, result, cx)).ok();
        })
        .detach();
    }

    /// The tab may be gone by now; then the result is dropped.
    fn finish_highlighting(&mut self, id: TabId, path: &Path, result: Result<Syntax, ParseError>, cx: &mut Context<Self>) {
        match (result, self.tabs.get_mut(id)) {
            (Ok(syntax), Some(Tab::Document(editor))) => {
                editor.set_syntax(syntax);
                cx.notify();
            }
            (Err(ParseError::Cancelled), _) | (Ok(_), _) => {}
            (Err(error), _) => eprintln!("ive: {} shown without highlighting: {error}", path.display()),
        }
    }

    /// Shows tab `id`; a detached one by bringing its window to the front.
    pub fn activate_tab(&mut self, id: TabId, window: &mut Window, cx: &mut Context<Self>) {
        if self.activate_tab_window(id, cx) {
            return;
        }
        self.tabs.activate(id);
        self.tab_shown(window, cx);
    }

    pub fn close_tab(&mut self, id: TabId, window: &mut Window, cx: &mut Context<Self>) {
        self.tabs.close(id);
        self.tab_shown(window, cx);
    }

    /// Makes temporary tab `id` stay when another is shown.
    pub fn keep_tab(&mut self, id: TabId, cx: &mut Context<Self>) {
        self.tabs.keep(id);
        cx.notify();
    }

    /// Makes the shown tab stay when it is tree row `row`'s; a row whose tab is in a window of its own keeps none.
    pub fn keep_shown_tab_of(&mut self, row: &TreeKey, cx: &mut Context<Self>) {
        if let Some(shown) = self.tabs.active_if(|tab| self.tree_key_of(tab).as_ref() == Some(row)) {
            self.keep_tab(shown, cx);
        }
    }

    pub fn keep_shown_tab(&mut self, cx: &mut Context<Self>) {
        if let Some(shown) = self.tabs.active_id() {
            self.keep_tab(shown, cx);
        }
    }

    /// Marks gap `gap` as where dragged tab `id` would land; `None`, or a gap that would not move it, clears the mark.
    pub fn hover_tab_drop(&mut self, id: TabId, gap: Option<usize>, cx: &mut Context<Self>) {
        let gap = gap.filter(|gap| self.tabs.moves(id, *gap));
        if self.tab_drop_gap != gap {
            self.tab_drop_gap = gap;
            cx.notify();
        }
    }

    pub fn tab_scroll(&self) -> &ScrollHandle {
        &self.tab_scroll
    }

    /// Scrolls the tab row just enough to show the shown tab; nothing for a detached one.
    fn scroll_to_shown_tab(&self) {
        let shown = self.tabs.active_id();
        if let Some(place) = self.tabs.row().position(|(id, _)| Some(id) == shown) {
            self.tab_scroll.scroll_to_item(place);
        }
    }

    /// The gap a dragged tab would land in, for the tab row to mark.
    pub fn tab_drop_gap(&self) -> Option<usize> {
        self.tab_drop_gap
    }

    /// Moves dragged tab `id` into the marked gap; the new order is saved with the next redraw.
    pub fn drop_dragged_tab(&mut self, id: TabId, cx: &mut Context<Self>) {
        if let Some(gap) = self.tab_drop_gap.take() {
            self.tabs.move_tab(id, gap);
            cx.notify();
        }
    }

    /// A left press in document tab `id`: the workspace takes the keys (a sidebar field may have had them), so the
    /// editor scope's Copy and Select All reach the document.
    pub fn press_document(&mut self, id: TabId, event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.focus_handle, cx);
        self.update_document_tab(
            id,
            |editor| {
                editor.press(event);
                true
            },
            cx,
        );
    }

    /// Changes document tab `id`; redraws when `update` says it changed what shows.
    pub fn update_document_tab(&mut self, id: TabId, update: impl FnOnce(&mut EditorView) -> bool, cx: &mut Context<Self>) {
        if let Some(Tab::Document(editor)) = self.tabs.get_mut(id)
            && update(editor)
        {
            cx.notify();
        }
    }

    /// Changes tab `id` in place, e.g. an overview's selected section; a closed tab is ignored.
    pub fn update_tab(&mut self, id: TabId, update: impl FnOnce(&mut Tab, &mut Context<Self>), cx: &mut Context<Self>) {
        if let Some(tab) = self.tabs.get_mut(id) {
            update(tab, cx);
            cx.notify();
        }
    }

    /// After the shown tab changed: focus, the tree's mark of it, the tab row's scroll, last focused checkout, window title, the shown section
    /// of an overview, and the windows of detached tabs. `window` may be a detached tab's, whose content opened a
    /// tab: the main window then comes to the front.
    fn tab_shown(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.close_stale_tab_windows(cx);
        if window.window_handle() == self.main_window {
            self.focus_active_tab(window, cx);
            self.update_title(window);
        } else {
            let main = self.main_window;
            // the main window is free: only the one handling the event is busy
            let _ = main.update(cx, |_, main, cx| {
                self.focus_active_tab(main, cx);
                self.update_title(main);
                main.activate_window();
            });
        }
        let checkout = self.tabs.active().and_then(|tab| tab.checkout(self));
        let shown = self.tabs.active().and_then(|tab| self.tree_key_of(tab));
        self.tree.show_active(shown);
        self.scroll_to_shown_tab();
        if let Some(checkout) = checkout {
            self.remember_checkout(&checkout, cx);
        }
        if let Some(tab) = self.tabs.active_id() {
            self.section_shown(tab, window, cx);
        }
        self.record_shown(cx);
        if self.hub_return.as_ref().is_some_and(|opened| Some(opened.tab) != self.tabs.active_id()) {
            self.hub_return = None;
        }
        self.sync_watches();
        cx.notify();
    }

    fn record_shown(&mut self, cx: &mut Context<Self>) {
        let Some(tab) = self.tabs.active() else {
            return;
        };
        let Some(identity) = tab.identity(self) else {
            return;
        };
        let checkout = tab.checkout(self).map(|checkout| checkout.path);
        let shown = Shown { identity, saved: tab.saved(self, cx), checkout, title: tab.title(self), shown_at: Clock::system().now() };
        self.history.shown(shown);
        self.schedule_save(cx);
    }

    /// Input was submitted to process `id` (Enter in its terminal): its checkout counts as used now.
    pub fn process_input(&mut self, id: ProcessId, cx: &mut Context<Self>) {
        if let Some(checkout) = self.processes.get(id).and_then(|process| process.checkout.clone()) {
            self.worktree_use.used(&checkout.path, Clock::system().now());
            self.schedule_save(cx);
        }
    }

    fn remember_checkout(&mut self, checkout: &CheckoutRef, cx: &mut Context<Self>) {
        let Some(open) = self.open_project_mut(&checkout.project) else {
            return;
        };
        if open.last_checkout.as_ref() == Some(&checkout.path) {
            return;
        }
        open.last_checkout = Some(checkout.path.clone());
        self.schedule_save(cx);
    }

    /// The tree row standing for `tab`, selected while it is shown.
    fn tree_key_of(&self, tab: &Tab) -> Option<TreeKey> {
        match tab {
            Tab::Document(editor) => Some(TreeKey::Entry(editor.path().to_path_buf())),
            Tab::Process(id) => match self.processes.get(*id).and_then(|process| process.run.as_ref()) {
                Some(_) => Some(TreeKey::LiveRun(RunId::Process(*id))),
                None => Some(TreeKey::Process(*id)),
            },
            Tab::RunValues(tab) => Some(TreeKey::Folder(tab.request.checkout.path.clone(), FolderKind::Processes)),
            Tab::RunConfigs(tab) => Some(TreeKey::Folder(tab.checkout.path.clone(), FolderKind::Processes)),
            Tab::ProjectOverview(overview) => Some(TreeKey::Project(overview.project.clone())),
            Tab::Commit(tab) => Some(TreeKey::Project(tab.project.clone())),
            Tab::GitLog(tab) => Some(TreeKey::Project(tab.project.clone())),
            Tab::Diff(_) | Tab::Settings { .. } | Tab::Failed { .. } | Tab::Scratch(_) | Tab::Unloaded(_) | Tab::Hub(_) => None,
        }
    }

    /// `<project of the shown tab> — IVE`, for the OS task bar.
    fn update_title(&self, window: &mut Window) {
        let project = self.tabs.active().and_then(|tab| tab.checkout(self)).and_then(|checkout| self.project(&checkout.project));
        match project {
            Some(project) => window.set_window_title(&format!("{} — IVE", project.name)),
            None => window.set_window_title("IVE"),
        }
    }

    /// The shown tab takes the keyboard when it has a focus handle (a process); otherwise the workspace does, so
    /// key bindings keep working once a focused terminal is gone, unless the sidebar has the keyboard: a file opened
    /// from the tree leaves it there, as in IntelliJ.
    fn focus_active_tab(&self, window: &mut Window, cx: &mut App) {
        match self.tabs.active().and_then(|tab| tab.focus_handle(self, cx)) {
            Some(handle) => window.focus(&handle, cx),
            None if sidebar_has_keys(window, cx) => {}
            None => window.focus(&self.focus_handle, cx),
        }
    }

    /// A press in the tab row or the shown tab while the sidebar has the keyboard: the tabs take it.
    pub(crate) fn press_tabs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !sidebar_has_keys(window, cx) {
            return;
        }
        match self.tabs.active().and_then(|tab| tab.focus_handle(self, cx)) {
            Some(handle) => window.focus(&handle, cx),
            None => window.focus(&self.focus_handle, cx),
        }
    }

    /// Rebuilds the tree rows after anything they show changed, registers what they show for changes on disk,
    /// and redraws.
    pub(super) fn tree_changed(&mut self, cx: &mut Context<Self>) {
        // no names before the scripts first load; that load rebuilds the rows
        let excluded = cx.try_global::<crate::script::excluded::Excluded>().map_or(&[][..], |excluded| excluded.names());
        let layout = cx.global::<SettingsStore>().current().git_files.layout;
        let look = rows::TreeLook { statuses: self.tree_statuses.known(), excluded, git: self.tree_git(), layout, clock: crate::age::Clock::system() };
        self.tree.rows = rows::build(&self.projects, &self.processes, &*self, &look);
        self.read_tree_statuses(cx);
        self.sync_watches();
        cx.notify();
    }

    /// The + button: opens the Open project menu, or closes it when it was open.
    pub(crate) fn toggle_menu(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let was_open = self.menu_open || self.menu_open_at_press;
        self.menu_open_at_press = false;
        if was_open {
            return self.close_menu(window, cx);
        }
        self.menu_open = true;
        // so Escape reaches the workspace, not a focused terminal; the sidebar keeps the keyboard
        window.focus(&cx.global::<theme::SidebarFocus>().0.clone(), cx);
        cx.notify();
    }

    /// A press on the + button, seen before the menu's press-outside closes the menu.
    pub(crate) fn note_menu_press(&mut self) {
        self.menu_open_at_press = self.menu_open;
    }

    /// Closes the menu and gives the keyboard back to the shown tab.
    pub(crate) fn close_menu(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.menu_open {
            self.menu_open = false;
            self.focus_active_tab(window, cx);
            cx.notify();
        }
    }

    fn close_menu_on_escape(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if event.keystroke.key == "escape" {
            self.close_menu(window, cx);
            self.close_tab_menu(window, cx);
        }
    }

    pub(crate) fn start_sidebar_resize(&mut self, x: f32) {
        self.sidebar_resize = Some(sidebar::Resize::start(x, self.sidebar_width));
    }

    pub(crate) fn reset_sidebar_width(&mut self, cx: &mut Context<Self>) {
        let default_width = cx.global::<Theme>().sizes.tree.default_width.0;
        self.set_sidebar_width(default_width, None, cx);
    }

    fn drag_sidebar_edge(&mut self, event: &MouseMoveEvent, window: &Window, cx: &mut Context<Self>) {
        let Some(resize) = self.sidebar_resize else {
            return;
        };
        let width = resize.width_at(f32::from(event.position.x), f32::from(window.viewport_size().width));
        if width != self.sidebar_width {
            self.sidebar_width = width;
            cx.notify();
        }
    }

    /// Ends a drag of the sidebar's edge and remembers the width for the next start.
    fn finish_sidebar_resize(&mut self, cx: &mut Context<Self>) {
        if self.sidebar_resize.take().is_none() {
            return;
        }
        let width = self.sidebar_width;
        self.set_sidebar_width(width, Some(width.round() as u32), cx);
    }

    /// `saved` is what goes to the settings file; `None` forgets a custom width.
    fn set_sidebar_width(&mut self, width: f32, saved: Option<u32>, cx: &mut Context<Self>) {
        self.sidebar_width = width;
        let stored = cx.update_global::<SettingsStore, _>(|store, _| {
            let settings = store.current().with_sidebar_width(saved);
            store.replace(settings)
        });
        if let Err(error) = stored {
            // the width still applies for this session; a later save retries
            eprintln!("ive: cannot save the project tree width: {error}");
        }
        cx.notify();
    }

    fn render_welcome(&self, theme: Theme, window: &mut Window, cx: &mut Context<Self>) -> Div {
        let sizes = &theme.sizes.window;
        let recent = self.recent.iter().enumerate().map(|(index, project)| {
            let path = project.path.clone();
            let name = crate::project::folder_name(&project.path);
            div()
                .id(("welcome-recent", index))
                .flex()
                .items_center()
                .gap(sizes.welcome_recent_gap.px())
                .px(sizes.welcome_recent_padding_x.px())
                .py(sizes.welcome_recent_padding_y.px())
                .rounded(sizes.welcome_recent_radius.px())
                .cursor_pointer()
                .hover(|style| style.bg(rgb(theme.hover)))
                .child(badge(&rows::initials(&name), theme.hover, theme.text, theme))
                .child(name)
                .on_click(target(format!("recent:{}", project.id), cx.listener(move |workspace, _event, window, cx| workspace.open_recent(path.clone(), window, cx)), cx))
        });
        let body = div()
            .flex()
            .flex_col()
            .gap(sizes.welcome_gap.px())
            .flex_1()
            .justify_center()
            .items_center()
            .child(div().text_size(sizes.welcome_title_size.px()).child("IVE - Integrated Vibe Editor"))
            .child(div().text_color(rgb(theme.muted)).child("No project open"))
            .child(
                button("open-folder", "Open Folder…", theme)
                    .on_click(target("open-folder".to_owned(), cx.listener(|this, _event, window, cx| this.prompt_for_folder(window, cx)), cx)),
            )
            .children(self.welcome_error.as_ref().map(|error| div().text_color(rgb(theme.error)).child(error.clone())))
            .child(div().flex().flex_col().items_center().children(recent));
        div().flex().flex_col().size_full().child(self.render_empty_tab_row(theme, window, cx)).child(body)
    }

    /// `panes` as rounded panels on the frame colour, `gap` apart and from the window's edges, with the glow of the
    /// worktree being worked in behind them; a maximised window's close button still takes a click in the very corner.
    fn render_frame(&self, panes: Div, theme: Theme, window: &Window, cx: &App) -> Div {
        let gap = theme.panels.gap.px();
        div()
            .relative()
            .flex()
            .size_full()
            .bg(rgb(theme.frame))
            .children(self.glow_index().map(|index| crate::glow::frame_glow(theme, index)))
            .children(window.is_maximized().then(|| crate::window_controls::corner_close(theme, window, cx)).flatten())
            .child(panes.relative().p(gap).gap(gap))
    }

    /// The tab row without tabs, so the welcome screen can be dragged and closed too.
    fn render_empty_tab_row(&self, theme: Theme, window: &Window, cx: &App) -> Stateful<Div> {
        self.window_drag
            .apply(div().id("welcome-tab-row"))
            .flex()
            .flex_none()
            .justify_end()
            .h(theme.sizes.tabs.row_height.px())
            .bg(rgb(theme.sidebar))
            .border_b(theme.sizes.tabs.row_border.px())
            .border_color(rgb(theme.border))
            .window_control_area(gpui::WindowControlArea::Drag)
            .children(window_controls(theme, window, cx))
    }

    fn is_welcome(&self) -> bool {
        self.projects.is_empty() && self.tabs.is_empty()
    }
}

/// While the sidebar's edge is dragged, the resize cursor stays on wherever the mouse goes.
fn resize_cursor() -> impl IntoElement {
    canvas(|_, _, _| {}, |_, (), window, _| window.set_window_cursor_style(CursorStyle::ResizeLeftRight)).absolute().size_0()
}

impl Render for Workspace {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        cx.global::<ClickTargets>().clear();
        // tabs, their order, scroll and carets change without telling the workspace each time; so does
        // the window's maximized or fullscreen state
        self.window_place.observe(window);
        self.window_drag.follow(&self.window_place, window);
        self.schedule_save(cx);
        self.load_shown_tab_later(window, cx);
        theme::set_tabs(window, cx);
        let theme = theme::for_panel(theme::Panel::Window, window, cx);
        // text without a size of its own, and text fields, follow the theme's zoom
        window.set_rem_size(theme.sizes.rem.px());
        let content = if self.is_welcome() {
            self.render_welcome(theme, window, cx)
        } else {
            let width = sidebar::fitted_width(self.sidebar_width, f32::from(window.viewport_size().width));
            let sidebar = Sidebar {
                tree: &self.tree,
                recent: &self.recent,
                menu_open: self.menu_open,
                drag: &self.window_drag,
                width,
                resizing: self.sidebar_resize.is_some(),
                watch_failures: self.watches.paths.failures(),
            };
            let sidebar_theme = theme::for_panel(theme::Panel::Sidebar, window, cx);
            let panes = div().flex().size_full().child(sidebar_view::render(sidebar, sidebar_theme, cx)).child(editor::render_pane(self, window, cx));
            match theme.panels.shown() {
                true => self.render_frame(panes, theme, window, cx),
                false => panes,
            }
        };
        let quit_prompt = self.render_quit_prompt(theme, cx);
        let window_content = div()
            .relative()
            .flex()
            .flex_col()
            .size_full()
            .child(content.flex_1().min_h_0())
            .children(quit_prompt)
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(|workspace, event: &KeyDownEvent, window, cx| workspace.close_menu_on_escape(event, window, cx)))
            .on_key_down(cx.listener(|workspace, event: &KeyDownEvent, window, cx| workspace.run_window_binding(event, window, cx)))
            .on_drop(cx.listener(|workspace, dropped: &ExternalPaths, window, cx| workspace.open_dropped(dropped, window, cx)))
            .on_mouse_move(cx.listener(|workspace, event: &MouseMoveEvent, window, cx| workspace.drag_sidebar_edge(event, window, cx)))
            .on_mouse_up(MouseButton::Left, cx.listener(|workspace, _event, _window, cx| workspace.finish_sidebar_resize(cx)))
            .on_mouse_up_out(MouseButton::Left, cx.listener(|workspace, _event, _window, cx| workspace.finish_sidebar_resize(cx)))
            // before the drop and the end of the drag: a tab released away from the row, outside the window too
            .capture_any_mouse_up(cx.listener(|workspace, event: &MouseUpEvent, window, cx| {
                if event.button == MouseButton::Left {
                    workspace.finish_tab_drag(window, cx);
                }
            }))
            .on_mouse_up_out(MouseButton::Left, cx.listener(|workspace, _event, window, cx| workspace.finish_tab_drag(window, cx)))
            .when(self.sidebar_resize.is_some(), |content| content.child(resize_cursor()))
            .bg(rgb(theme.background))
            .font_family(theme::ui_font(cx))
            .text_color(rgb(theme.text));
        let mut window_content = window_content;
        window_content.text_style().font_fallbacks = Some(theme::emoji_fallbacks());
        frame::render(window_content, theme, window)
    }
}

fn button(id: &'static str, label: &'static str, theme: Theme) -> Stateful<Div> {
    div()
        .id(id)
        .px(theme.sizes.window.button_padding_x.px())
        .py(theme.sizes.window.button_padding_y.px())
        .rounded(theme.sizes.window.button_radius.px())
        .bg(rgb(theme.accent))
        .text_color(rgb(theme.on_accent))
        .hover(|style| style.bg(rgb(theme.accent_hover)))
        .cursor_pointer()
        .child(label)
}

pub fn icon(theme: Theme, name: &str) -> Img {
    img(icons::embedded(theme.appearance, name)).flex_none().size(theme.sizes.icon.px())
}

/// A one-colour icon in the theme's icon colour, drawn as a mask at its exact size: `img` draws an SVG at twice the
/// size and scales it down, which thins 1 px strokes such as the crosshair's ticks.
pub fn mask_icon(theme: Theme, name: &str) -> Svg {
    svg().path(icons::path(theme.appearance, name)).flex_none().size(theme.sizes.icon.px()).text_color(rgb(theme.icon))
}

/// What to show at start, from the path given on the command line.
pub enum Launch {
    /// the saved projects only
    Restore,
    /// also the folder's project, expanded to the folder's checkout, with that checkout's overview
    Folder(PathBuf),
    /// also the file's folder's project, with the file open
    File(PathBuf),
}

impl Launch {
    pub fn from_path(path: Option<PathBuf>) -> Self {
        match path {
            None => Self::Restore,
            Some(path) if path.is_file() => Self::File(path),
            Some(path) => Self::Folder(path),
        }
    }
}

/// Whether the sidebar, or a field in it, has the keyboard in `window`.
fn sidebar_has_keys(window: &Window, cx: &App) -> bool {
    cx.try_global::<theme::SidebarFocus>().is_some_and(|sidebar| sidebar.0.contains_focused(window, cx))
}
