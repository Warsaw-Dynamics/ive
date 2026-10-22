use std::cell::Cell;
use std::path::Path;
use std::rc::Rc;
use std::time::Duration;

use codeview::document::{Reread, Stamp};

use gpui::{
    AnyElement, App, AppContext, Bounds, Context, Div, DragMoveEvent, FocusHandle, MouseButton, MouseDownEvent, SharedString, Focusable,
    Stateful, Window, WindowControlArea, div, img, prelude::*, px, rgb, size, transparent_black,
};

use crate::click_targets::target;
use crate::colors::checkout_colour;
pub use crate::file_tab::FileTab;
use crate::hub::history::TabIdentity;
use crate::hub::view::HubTab;
use crate::icons;
use crate::mouse_binding;
use crate::overview::commit_tab::CommitTab;
use crate::overview::diff_tab::DiffTab;
use crate::overview::git_log_tab::GitLogTab;
use crate::overview::{self, ProjectOverview, ProjectSection};
use crate::process::{ProcessId, ProcessKind};
use crate::run::configs_tab::{RunConfigsTab, render_run_configs_tab};
use crate::run::values_tab::{self as run_values, RunValuesTab};
use crate::progress_bar::progress_bar;
use crate::terminal::Progress;
use crate::saved_tabs::SavedTab;
use crate::project::{CheckoutRef, ProjectId};
use crate::scratch::{self, ScratchRef, ScratchTab};
use crate::scratch::icon::scratch_icon;
use crate::script::temporary_tabs::{TabKind, TabMarker, TemporaryTabs};
use crate::settings_page::{self, SettingsView};
use crate::tabs::TabId;
use crate::theme::Theme;
use crate::tooltip;
use crate::watch::Watched;
use crate::window_controls::{TRAFFIC_LIGHTS_WIDTH, window_controls};
use crate::window_drag::WindowDrag;
use crate::workspace::{TabDragArea, Workspace};

/// Notifications cannot see edits in collapsed folders, which the Worktrees cards count.
const WORKTREES_POLL: Duration = Duration::from_secs(5);

/// What a tab in the editor pane shows. Parts 2 and 3 add variants with their arms in each method below.
pub enum Tab {
    Document(FileTab),
    /// a process owned by the workspace's process list; closing the tab keeps the process
    Process(ProcessId),
    ProjectOverview(ProjectOverview),
    /// one file's diff, frozen when it opened
    Diff(DiffTab),
    Commit(CommitTab),
    /// one per project
    GitLog(GitLogTab),
    Settings(SettingsView),
    Failed { title: String, error: SharedString },
    /// a scratch of a project, in its editor
    Scratch(ScratchTab),
    /// the values a run needs and has none of; never saved
    RunValues(RunValuesTab),
    /// every run configuration of a checkout; at most one per checkout, never saved
    RunConfigs(RunConfigsTab),
    /// a tab restored at start, loaded when first shown
    Unloaded(SavedTab),
    /// Ctrl+Shift+E: recent tabs, changes, processes and every file; at most one
    Hub(Box<HubTab>),
}

impl Tab {
    // The finders below match a restored tab not loaded yet too, so opening what it shows reuses it.

    pub fn is_hub(&self) -> bool {
        matches!(self, Self::Hub(_) | Self::Unloaded(SavedTab::Hub))
    }

    pub fn is_settings(&self) -> bool {
        matches!(self, Self::Settings(_) | Self::Unloaded(SavedTab::Settings { .. }))
    }

    /// Whether this tab shows the document at `path`.
    pub fn shows(&self, path: &Path) -> bool {
        match self {
            Self::Document(file) => file.path() == path,
            Self::Unloaded(SavedTab::File { path: saved, .. }) => saved == path,
            _ => false,
        }
    }

    pub fn is_project_overview(&self, id: &ProjectId) -> bool {
        match self {
            Self::ProjectOverview(overview) => &overview.project == id,
            Self::Unloaded(SavedTab::ProjectOverview { project, .. }) => project == id.as_str(),
            _ => false,
        }
    }

    pub fn shows_scratch(&self, scratch: &ScratchRef) -> bool {
        match self {
            Self::Scratch(open) => open.shows(scratch),
            Self::Unloaded(SavedTab::Scratch { project, name, .. }) => project == scratch.project.as_str() && name == &scratch.name,
            _ => false,
        }
    }

    pub fn is_git_log(&self, id: &ProjectId) -> bool {
        match self {
            Self::GitLog(tab) => &tab.project == id,
            Self::Unloaded(SavedTab::GitLog { project, .. }) => project == id.as_str(),
            _ => false,
        }
    }

    pub fn is_commit(&self, id: &ProjectId, sha: &str) -> bool {
        match self {
            Self::Commit(tab) => &tab.project == id && tab.sha.as_str() == sha,
            Self::Unloaded(SavedTab::Commit { project, sha: saved, .. }) => project == id.as_str() && saved == sha,
            _ => false,
        }
    }

    pub fn shows_process(&self, id: ProcessId) -> bool {
        matches!(self, Self::Process(shown) if *shown == id)
    }

    /// As MCP's `list_tabs` reports it.
    /// What `config.temporary_tabs` calls this tab, when it is one that may open temporary; a restored tab is not.
    pub fn temporary_kind(&self) -> Option<TabKind> {
        match self {
            Self::ProjectOverview(_) => Some(TabKind::Project),
            Self::Settings(_) => Some(TabKind::Settings),
            Self::Hub(_) => Some(TabKind::Hub),
            Self::RunConfigs(_) => Some(TabKind::RunConfigs),
            _ => None,
        }
    }

    pub fn kind(&self) -> &'static str {
        match self {
            Self::Document(_) => "document",
            Self::Process(_) => "process",
            Self::ProjectOverview(_) => "project-overview",
            Self::Diff(_) => "diff",
            Self::Commit(_) => "commit",
            Self::GitLog(_) => "git-log",
            Self::Settings(_) => "settings",
            Self::Failed { .. } => "failed",
            Self::Scratch(_) => "scratch",
            Self::RunValues(_) => "run-values",
            Self::RunConfigs(_) => "run-configs",
            Self::Unloaded(saved) => saved.kind(),
            Self::Hub(_) => "hub",
        }
    }

    pub fn title(&self, workspace: &Workspace) -> String {
        match self {
            Self::Document(file) => file.file_name(),
            Self::Process(id) => workspace.processes().get(*id).map_or_else(|| "Terminal".to_owned(), |process| process.title()),
            Self::ProjectOverview(overview) => {
                workspace.project(&overview.project).map_or_else(|| overview.project.to_string(), |project| project.name.clone())
            }
            Self::Diff(tab) => tab.title.clone(),
            Self::Commit(tab) => tab.title.clone(),
            Self::GitLog(_) => "Git log".to_owned(),
            Self::Settings(_) => "Settings".to_owned(),
            Self::Failed { title, .. } => title.clone(),
            Self::Scratch(scratch) => scratch.name().to_owned(),
            Self::RunValues(tab) => tab.title(),
            Self::RunConfigs(_) => "Run configurations".to_owned(),
            Self::Unloaded(saved) => saved.title(),
            Self::Hub(hub) if hub.state.new_terminal => "New terminal".to_owned(),
            Self::Hub(_) => "Hub".to_owned(),
        }
    }

    /// What the process a tab shows last reported with `OSC 9;4`.
    fn progress(&self, workspace: &Workspace) -> Option<Progress> {
        match self {
            Self::Process(id) => workspace.processes().get(*id).map(|process| process.progress()),
            _ => None,
        }
    }

    /// The chip's icon. A kind with a composed icon (part 3's badged scratch icon) matches on itself here.
    pub fn icon(&self, workspace: &Workspace, theme: Theme) -> AnyElement {
        if let Self::Scratch(scratch) = self {
            return scratch_icon(theme, scratch.name()).into_any_element();
        }
        img(icons::embedded(theme.appearance, self.icon_name(workspace))).flex_none().size(theme.sizes.tabs.tab_icon_size.px()).into_any_element()
    }

    fn icon_name(&self, workspace: &Workspace) -> &'static str {
        match self {
            Self::Document(file) => icons::file_icon(&file.file_name()),
            Self::Process(id) => workspace.processes().get(*id).map_or(ProcessKind::Terminal.icon(), |process| process.icon()),
            Self::ProjectOverview(_) => icons::PROJECT_FOLDER,
            Self::Diff(_) => icons::CHANGES,
            Self::Commit(_) => icons::COMMIT,
            Self::GitLog(_) => icons::HISTORY,
            Self::Settings(_) => icons::SETTINGS,
            Self::Failed { title, .. } => icons::file_icon(title),
            Self::Scratch(scratch) => icons::file_icon(scratch.name()),
            Self::RunValues(_) | Self::RunConfigs(_) => icons::RUN_CONFIG,
            Self::Unloaded(saved) => saved_icon(saved),
            Self::Hub(_) => icons::LOCATE,
        }
    }

    /// What `state.toml` keeps to open this tab again; `None` for a tab that is not saved: a failed one, or a
    /// terminal that ends with IVE.
    pub fn saved(&self, workspace: &Workspace, cx: &App) -> Option<SavedTab> {
        let tab = match self {
            Self::Document(file) => SavedTab::File { path: file.path().to_path_buf(), panel: file.panel.read(cx).state() },
            Self::Scratch(scratch) => {
                SavedTab::Scratch { project: scratch.project().as_str().to_owned(), name: scratch.name().to_owned(), panel: scratch.panel_state(cx) }
            }
            Self::Process(id) => SavedTab::Terminal { session: workspace.processes().get(*id)?.view.hosted_id()?.to_owned() },
            Self::ProjectOverview(overview) => {
                SavedTab::ProjectOverview { project: overview.project.as_str().to_owned(), section: overview.section.label().to_owned() }
            }
            Self::GitLog(tab) => SavedTab::GitLog { project: tab.project.as_str().to_owned(), view: tab.body.read(cx).section_name().to_owned() },
            Self::Commit(tab) => {
                let panel = tab.panel_state(cx);
                SavedTab::Commit { project: tab.project.as_str().to_owned(), sha: tab.sha.as_str().to_owned(), title: tab.title.clone(), panel }
            }
            Self::Diff(tab) => SavedTab::Diff { checkout: tab.checkout.path.clone(), request: tab.request.saved(), panel: tab.panel_state(cx) },
            Self::Settings(view) => SavedTab::Settings { page: view.page_name().to_owned() },
            Self::Hub(_) => SavedTab::Hub,
            Self::Failed { .. } | Self::RunValues(_) | Self::RunConfigs(_) => return None,
            Self::Unloaded(saved) => saved.clone(),
        };
        Some(tab)
    }

    /// What the hub's history knows this tab by; `None` for a tab it does not list (a failed one).
    pub fn identity(&self, workspace: &Workspace) -> Option<TabIdentity> {
        let identity = match self {
            Self::Document(file) => TabIdentity::File(file.path().to_path_buf()),
            Self::Process(id) => match workspace.processes().get(*id).and_then(|process| process.view.hosted_id()) {
                Some(session) => TabIdentity::Terminal { session: session.to_owned() },
                None => TabIdentity::Process(id.value()),
            },
            Self::ProjectOverview(overview) => TabIdentity::ProjectOverview(overview.project.as_str().to_owned()),
            Self::Diff(tab) => TabIdentity::Diff { checkout: tab.checkout.path.clone(), path: tab.request.saved().path },
            Self::Commit(tab) => TabIdentity::Commit { project: tab.project.as_str().to_owned(), sha: tab.sha.as_str().to_owned() },
            Self::GitLog(tab) => TabIdentity::GitLog(tab.project.as_str().to_owned()),
            Self::Settings(_) => TabIdentity::Settings,
            Self::Failed { .. } | Self::Hub(_) | Self::RunConfigs(_) => return None,
            Self::Scratch(scratch) => TabIdentity::Scratch { project: scratch.project().as_str().to_owned(), name: scratch.name().to_owned() },
            Self::RunValues(tab) => TabIdentity::RunValues {
                checkout: tab.request.checkout.path.clone(),
                folder: tab.request.folder.clone(),
                key: tab.request.key.clone(),
            },
            Self::Unloaded(SavedTab::Hub) => return None,
            Self::Unloaded(saved) => TabIdentity::of_saved(saved)?,
        };
        Some(identity)
    }

    /// The checkout this tab belongs to: a document's (deepest containing checkout), a process's, an overview's;
    /// a project's tab (overview, Git log, commit) stands for the project's last focused checkout, else its main one.
    pub fn checkout(&self, workspace: &Workspace) -> Option<CheckoutRef> {
        match self {
            Self::Document(file) => workspace.checkout_of_path(file.path()),
            Self::Process(id) => workspace.processes().get(*id).and_then(|process| process.checkout.clone()),
            Self::ProjectOverview(overview) => project_checkout(workspace, &overview.project),
            Self::Diff(tab) => Some(tab.checkout.clone()),
            Self::Commit(tab) => project_checkout(workspace, &tab.project),
            Self::GitLog(tab) => project_checkout(workspace, &tab.project),
            // where it runs
            Self::Scratch(scratch) => workspace.scratch_checkout(scratch.project()),
            Self::Settings(_) | Self::Failed { .. } | Self::Hub(_) => None,
            Self::RunValues(tab) => Some(tab.request.checkout.clone()),
            Self::RunConfigs(tab) => Some(tab.checkout.clone()),
            Self::Unloaded(saved) => unloaded_checkout(saved, workspace),
        }
    }

    /// Checkouts whose status this tab displays; a window activation re-reads only those.
    pub fn shown_statuses(&self, workspace: &Workspace) -> Vec<CheckoutRef> {
        match self {
            Self::ProjectOverview(overview) if overview.section == ProjectSection::Worktrees => workspace
                .project(&overview.project)
                .map(|project| project.checkouts.iter().map(|checkout| project.checkout_ref(checkout)).collect())
                .unwrap_or_default(),
            Self::Document(_)
            | Self::Process(_)
            | Self::ProjectOverview(_)
            | Self::Diff(_)
            | Self::Commit(_)
            | Self::GitLog(_)
            | Self::Settings(_)
            | Self::Failed { .. }
            | Self::Scratch(_)
            | Self::RunValues(_)
            | Self::RunConfigs(_)
            | Self::Unloaded(_)
            | Self::Hub(_) => Vec::new(),
        }
    }

    /// How often, while the window is active, the statuses this tab shows are re-read.
    pub fn poll_interval(&self) -> Option<Duration> {
        match self {
            Self::ProjectOverview(overview) if overview.section == ProjectSection::Worktrees => Some(WORKTREES_POLL),
            Self::Document(_)
            | Self::Process(_)
            | Self::ProjectOverview(_)
            | Self::Diff(_)
            | Self::Commit(_)
            | Self::GitLog(_)
            | Self::Settings(_)
            | Self::Failed { .. }
            | Self::Scratch(_)
            | Self::RunValues(_)
            | Self::RunConfigs(_)
            | Self::Unloaded(_)
            | Self::Hub(_) => None,
        }
    }

    /// The file this tab shows, registered for changes on disk; a project overview's Scratches section, its folder.
    pub fn watched(&self) -> Option<Watched> {
        match self {
            Self::Document(file) => Some(Watched::File(file.path().to_path_buf())),
            Self::Scratch(scratch) => Some(Watched::File(scratch.path().to_path_buf())),
            Self::ProjectOverview(overview) if overview.section == ProjectSection::Scratches => {
                scratch::store_of(&overview.project).ok().map(|store| Watched::Folder(store.dir().to_path_buf()))
            }
            Self::Diff(diff) => diff.edited_path().map(Watched::File),
            Self::Process(_)
            | Self::ProjectOverview(_)
            | Self::Commit(_)
            | Self::GitLog(_)
            | Self::Settings(_)
            | Self::Failed { .. }
            | Self::RunValues(_)
            | Self::RunConfigs(_)
            | Self::Unloaded(_)
            | Self::Hub(_) => None,
        }
    }

    /// What a re-read of the file at `path` compares it with, when this tab edits that file (a file tab, or a diff
    /// tab editing its new side).
    pub fn edited_stamp(&self, path: &Path, cx: &App) -> Option<Option<Stamp>> {
        match self {
            Self::Document(file) if file.path() == path => Some(file.stamp(cx)),
            Self::Diff(diff) if diff.edited_path().as_deref() == Some(path) => diff.edited_stamp(cx),
            _ => None,
        }
    }

    /// What re-reading the file this tab edits found.
    pub fn take_reread(&mut self, reread: Reread, cx: &mut App) {
        match self {
            Self::Document(file) => file.take_reread(reread, cx),
            Self::Diff(diff) => diff.take_reread(reread, cx),
            Self::Process(_)
            | Self::ProjectOverview(_)
            | Self::Commit(_)
            | Self::GitLog(_)
            | Self::Settings(_)
            | Self::Failed { .. }
            | Self::Scratch(_)
            | Self::RunValues(_)
            | Self::RunConfigs(_)
            | Self::Unloaded(_)
            | Self::Hub(_) => {}
        }
    }

    /// The project this tab belongs to; Close project closes it.
    pub fn project(&self, workspace: &Workspace) -> Option<ProjectId> {
        match self {
            Self::Scratch(scratch) => Some(scratch.project().clone()),
            _ => self.checkout(workspace).map(|checkout| checkout.project),
        }
    }

    /// `<checkout label or project name> — <title>`, or the title alone outside projects; a scratch names its project,
    /// a file shows its full path in place of the title.
    pub fn tooltip(&self, workspace: &Workspace) -> String {
        if let Self::Diff(tab) = self {
            return tab.tooltip.clone();
        }
        let title = match self {
            Self::Document(file) => file.path().display().to_string(),
            _ => self.title(workspace),
        };
        let owner = match self {
            Self::ProjectOverview(_) | Self::Diff(_) | Self::Settings(_) | Self::Failed { .. } | Self::Unloaded(_) | Self::Hub(_) => None,
            Self::Document(_) | Self::Process(_) | Self::RunValues(_) | Self::RunConfigs(_) => {
                self.checkout(workspace).map(|checkout| workspace.checkout_label(&checkout))
            }
            Self::Commit(CommitTab { project, .. }) | Self::GitLog(GitLogTab { project, .. }) => {
                workspace.project(project).map(|project| project.name.clone())
            }
            Self::Scratch(scratch) => Some(scratch.project_name().to_owned()),
        };
        match owner {
            Some(owner) => format!("{owner} — {title}"),
            None => title,
        }
    }

    /// What takes the keyboard while this tab is shown; `None` leaves it to the workspace.
    pub fn focus_handle(&self, workspace: &Workspace, cx: &App) -> Option<FocusHandle> {
        match self {
            Self::Process(id) => workspace.processes().get(*id).map(|process| process.view.focus_handle()),
            Self::Scratch(scratch) => scratch.focus_handle(cx),
            Self::Document(file) => Some(file.panel.focus_handle(cx)),
            // a diff of uncommitted changes edits its new side
            Self::Diff(diff) => diff.editing_panel(cx).map(|panel| panel.focus_handle(cx)),
            Self::RunValues(tab) => tab.fields.first().map(|field| field.input.focus_handle(cx)),
            Self::Hub(hub) => Some(hub.focus_handle(cx)),
            // the Worktrees list's filter keeps the keyboard
            Self::ProjectOverview(ProjectOverview { section: ProjectSection::Worktrees, git: Some(views), .. }) => Some(views.list.focus_handle(cx)),
            Self::ProjectOverview(_)
            | Self::Commit(_)
            | Self::GitLog(_)
            | Self::Settings(_)
            | Self::Failed { .. }
            | Self::RunConfigs(_)
            | Self::Unloaded(_) => None,
        }
    }

    /// The checkout colour a tab is underlined with (2 px): a diff's; `None` for the others.
    pub fn underline(&self, workspace: &Workspace, theme: Theme) -> Option<u32> {
        match self {
            Self::Diff(tab) => workspace
                .project(&tab.checkout.project)
                .and_then(|project| project.checkout(&tab.checkout.path))
                .map(|checkout| checkout_colour(checkout, theme)),
            _ => None,
        }
    }

    fn render(&self, id: TabId, workspace: &Workspace, window: &mut Window, cx: &mut Context<Workspace>) -> AnyElement {
        let theme = crate::theme::tabs(cx);
        match self {
            Self::Document(file) => file.render(theme, cx),
            Self::Process(process) => match workspace.processes().get(*process) {
                Some(process) => match process.link() {
                    Some(url) => div()
                        .flex()
                        .flex_col()
                        .size_full()
                        .child(crate::run::link_bar::link_bar(process.id, "tab", url, theme, cx))
                        .child(div().flex().flex_1().min_h_0().child(process.view.render(theme, window, cx)))
                        .into_any_element(),
                    None => process.view.render(theme, window, cx).into_any_element(),
                },
                None => centered_message("This terminal has ended", theme.muted).into_any_element(),
            },
            Self::ProjectOverview(overview) => overview::render_project_overview(id, overview, workspace, cx),
            Self::Diff(tab) => tab.body.clone().into_any_element(),
            Self::Commit(tab) => tab.body.clone().into_any_element(),
            Self::GitLog(tab) => tab.body.clone().into_any_element(),
            Self::Settings(view) => settings_page::render(view, theme, cx).into_any_element(),
            Self::Failed { error, .. } => centered_message(error.clone(), theme.error).into_any_element(),
            Self::Scratch(scratch) => scratch.view().clone().into_any_element(),
            Self::RunValues(tab) => run_values::render_values_tab(id, tab, cx),
            Self::RunConfigs(tab) => render_run_configs_tab(id, tab, workspace, cx),
            Self::Unloaded(_) => centered_message("Loading…", theme.muted).into_any_element(),
            Self::Hub(hub) => workspace.render_hub_tab(id, hub, cx),
        }
    }
}

/// A saved tab's icon, before it loads and in the hub's Recent.
pub(crate) fn saved_icon(saved: &SavedTab) -> &'static str {
    match saved {
        SavedTab::File { .. } | SavedTab::Scratch { .. } => icons::file_icon(&saved.title()),
        SavedTab::Terminal { .. } => icons::TERMINAL,
        SavedTab::ProjectOverview { .. } => icons::PROJECT_FOLDER,
        SavedTab::WorktreeOverview { .. } => icons::BRANCH,
        SavedTab::GitLog { .. } => icons::HISTORY,
        SavedTab::Commit { .. } => icons::COMMIT,
        SavedTab::Diff { .. } => icons::CHANGES,
        SavedTab::Settings { .. } => icons::SETTINGS,
        SavedTab::Hub => icons::LOCATE,
    }
}

fn unloaded_checkout(saved: &SavedTab, workspace: &Workspace) -> Option<CheckoutRef> {
    match saved {
        SavedTab::File { path, .. } | SavedTab::WorktreeOverview { checkout: path, .. } | SavedTab::Diff { checkout: path, .. } => {
            workspace.checkout_of_path(path)
        }
        SavedTab::Scratch { project, .. } | SavedTab::ProjectOverview { project, .. } | SavedTab::GitLog { project, .. } | SavedTab::Commit { project, .. } => {
            ProjectId::parse(project).ok().and_then(|project| project_checkout(workspace, &project))
        }
        SavedTab::Terminal { .. } | SavedTab::Settings { .. } | SavedTab::Hub => None,
    }
}

/// The project's last focused checkout, else its main one.
fn project_checkout(workspace: &Workspace, project: &ProjectId) -> Option<CheckoutRef> {
    workspace.last_focused_checkout(project).or_else(|| workspace.project(project).map(|project| project.checkout_ref(project.main_checkout())))
}

/// The editor pane: the tab row, always shown so the window can be dragged and closed, and the shown tab.
pub fn render_pane(workspace: &Workspace, window: &mut Window, cx: &mut Context<Workspace>) -> Div {
    let theme = crate::theme::tabs(cx);
    let tabs = workspace.tabs();
    let body = match tabs.active_id().zip(tabs.active()) {
        Some((id, tab)) => tab.render(id, workspace, window, cx),
        None => centered_message("Select a file in the sidebar, or press Alt+F12 for a terminal", theme.muted).into_any_element(),
    };
    div()
        .flex()
        .flex_col()
        .flex_1()
        .h_full()
        .min_w_0()
        // captured: tab chips and views stop their presses; what takes the keys in its own handler keeps them
        .capture_any_mouse_down(cx.listener(|workspace, event: &MouseDownEvent, window, cx| {
            if event.button == MouseButton::Left {
                workspace.press_tabs(window, cx);
            }
        }))
        // anywhere the pointer goes, outside the window too: a tab released away from the row is detached
        .on_drag_move(cx.listener(|workspace, event: &DragMoveEvent<DraggedTab>, window, cx| {
            let pane = event.bounds;
            let row_height = cx.global::<Theme>().sizes.tabs.row_height.px();
            let row = Bounds::new(pane.origin, size(pane.size.width, row_height));
            let body = Bounds::from_corners(row.bottom_left(), pane.bottom_right());
            workspace.track_tab_drag(event.drag(cx).id, event.event.position, TabDragArea { row, body }, window, cx);
        }))
        .child(render_tab_row(workspace, theme, window, cx))
        .map(|pane| match theme.panels.shown() {
            // the tab row sits on the frame; the body below it is the panel
            true => pane.child(div().flex().flex_col().flex_1().min_h_0().bg(rgb(theme.background)).rounded_b(theme.panels.radius.px()).overflow_hidden().child(body)),
            false => pane.child(body),
        })
}

/// The top of the right column: tabs (scrolling sideways when they do not fit: Shift+wheel or a touchpad),
/// then Settings and the window buttons. Its empty space moves the window.
fn render_tab_row(workspace: &Workspace, theme: Theme, window: &Window, cx: &mut Context<Workspace>) -> Stateful<Div> {
    let tabs = workspace.tabs();
    let active = tabs.active_id();
    let last = tabs.row_len().saturating_sub(1);
    // a gap stays marked after a drag ends outside the window, until the next drag
    let drop_gap = workspace.tab_drop_gap().filter(|_| cx.has_active_drag());
    let marker = move |place: usize| match drop_gap {
        Some(gap) if gap == place => Some(Side::Left),
        Some(gap) if gap == place + 1 && place == last => Some(Side::Right),
        _ => None,
    };
    // past the last tab: a dragged tab goes last
    let sizes = &theme.sizes.tabs;
    let end = div().id("tab-strip-end").flex_1().h_full().min_w(sizes.gap.px()).on_drag_move(cx.listener(
        move |workspace, event: &DragMoveEvent<DraggedTab>, _window, cx| {
            if event.bounds.contains(&event.event.position) {
                workspace.hover_tab_drop(event.drag(cx).id, Some(last + 1), cx);
            }
        },
    ));
    let strip = div()
        .id("tab-strip")
        .flex()
        .flex_1()
        .min_w_0()
        .h_full()
        .items_center()
        .gap(sizes.gap.px())
        .px(sizes.strip_padding_x.px())
        .overflow_x_scroll()
        .track_scroll(workspace.tab_scroll())
        .on_drag_move(cx.listener(|workspace, event: &DragMoveEvent<DraggedTab>, _window, cx| {
            if event.bounds.contains(&event.event.position) {
                return;
            }
            workspace.hover_tab_drop(event.drag(cx).id, None, cx);
        }))
        .on_drop(cx.listener(|workspace, dragged: &DraggedTab, _window, cx| workspace.drop_dragged_tab(dragged.id, cx)))
        .children(tabs.row().enumerate().map(|(place, (id, tab))| {
            render_tab(Chip { id, place, tab, shown: Some(id) == active, temporary: tabs.is_temporary(id), marker: marker(place) }, workspace, cx)
        }))
        .child(end);
    let restart = workspace.restart_script().map(|_| restart_button(theme, cx));
    let tools = div()
        .flex()
        .flex_none()
        .items_center()
        .gap(sizes.tools_gap.px())
        .h_full()
        .pl(sizes.tools_padding_left.px())
        .children(restart)
        .child(settings_button(theme, cx))
        .children(window_controls(theme, window, cx));
    workspace
        .window_drag()
        .apply(div().id("tab-row"))
        .flex()
        .flex_none()
        .items_center()
        .h(sizes.row_height.px())
        .when(!theme.panels.shown(), |row| row.bg(rgb(theme.sidebar)))
        .border_b(sizes.row_border.px())
        .border_color(rgb(theme.border))
        .window_control_area(WindowControlArea::Drag)
        .child(strip)
        .child(tools)
        .children(crate::tab_menu::render_tab_menu(workspace, theme, cx))
}

/// A detached tab's window: a row with the tab, Move to main window and the window buttons, then the tab. `None`
/// once the tab is closed; else the tab's title too, for the window's.
pub fn render_detached(workspace: &Workspace, id: TabId, drag: &WindowDrag, window: &mut Window, cx: &mut Context<Workspace>) -> Option<(String, Div)> {
    let tab = workspace.tabs().get(id)?;
    let theme = crate::theme::tabs(cx);
    let title = tab.title(workspace);
    let sizes = &theme.sizes.tabs;
    let chip = div()
        .flex()
        .flex_none()
        .items_center()
        .gap(sizes.tab_inner_gap.px())
        .h(sizes.tab_height.px())
        .px(sizes.tab_padding_left.px())
        .rounded(sizes.tab_radius.px())
        .border(sizes.tab_border.px())
        .text_size(sizes.tab_text_size.px())
        .child(tab.icon(workspace, theme))
        .children(tab.progress(workspace).and_then(|progress| progress_bar(progress, theme)))
        .child(div().max_w(sizes.title_max_width.px()).overflow_hidden().text_ellipsis().whitespace_nowrap().child(title.clone()))
        .children(tab.underline(workspace, theme).map(|colour| underline(colour, theme)))
        .relative();
    // the window's only tab is always the shown one, so its chip marks the window having the keyboard instead
    let chip = if window.is_window_active() {
        chip.bg(rgb(theme.tab_active)).border_color(rgb(theme.tab_active_border))
    } else {
        chip.border_color(transparent_black())
    };
    let tools = div()
        .flex()
        .flex_none()
        .items_center()
        .gap(sizes.tools_gap.px())
        .h_full()
        .pl(sizes.tools_padding_left.px())
        .child(attach_button(id, theme, cx))
        .children(window_controls(theme, window, cx));
    let row = drag
        .apply(div().id("detached-tab-row"))
        .flex()
        .flex_none()
        .items_center()
        .h(sizes.row_height.px())
        .bg(rgb(theme.sidebar))
        .border_b(sizes.row_border.px())
        .border_color(rgb(theme.border))
        .window_control_area(WindowControlArea::Drag)
        .child(
            div()
                .flex()
                .flex_1()
                .min_w_0()
                .px(sizes.strip_padding_x.px())
                .when(cfg!(target_os = "macos"), |strip| strip.pl(px(TRAFFIC_LIGHTS_WIDTH)))
                .child(chip),
        )
        .child(tools);
    let body = tab.render(id, workspace, window, cx);
    Some((title, div().flex().flex_col().flex_1().h_full().min_w_0().child(row).child(body)))
}

fn attach_button(id: TabId, theme: Theme, cx: &mut Context<Workspace>) -> Stateful<Div> {
    div()
        .id("attach-tab")
        .p(theme.sizes.tabs.toolbar_button_padding.px())
        .rounded(theme.sizes.tabs.toolbar_button_radius.px())
        .cursor_pointer()
        .hover(|style| style.bg(rgb(theme.hover)))
        .child(img(icons::embedded(theme.appearance, icons::DOCK)).size(theme.sizes.tabs.tab_icon_size.px()))
        .tooltip(tooltip::text_tooltip("Move to main window"))
        .on_mouse_down(MouseButton::Left, |_event, _window, cx| cx.stop_propagation())
        .on_click(target("attach-tab".to_owned(), cx.listener(move |workspace, _event, window, cx| workspace.attach_tab(id, window, cx)), cx))
}

/// A tab of the tab row: its place in the row, whether it is the shown one, and the side a dragged tab would
/// land on.
struct Chip<'a> {
    id: TabId,
    place: usize,
    tab: &'a Tab,
    shown: bool,
    temporary: bool,
    marker: Option<Side>,
}

#[derive(Clone, Copy)]
enum Side {
    Left,
    Right,
}

/// A tab being dragged to another place in the tab row; it draws itself under the pointer, unless the preview
/// of the window it would detach into is shown instead.
#[derive(Clone)]
struct DraggedTab {
    id: TabId,
    title: SharedString,
    hidden: Rc<Cell<bool>>,
}

impl Render for DraggedTab {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        if self.hidden.get() {
            return div();
        }
        let sizes = &theme.sizes.tabs;
        div()
            .flex()
            .items_center()
            .h(sizes.tab_height.px())
            .px(sizes.tab_padding_left.px())
            .rounded(sizes.tab_radius.px())
            .border(sizes.tab_border.px())
            .border_color(rgb(theme.tab_active_border))
            .bg(rgb(theme.tab_active))
            .opacity(sizes.drag_chip_opacity.0)
            .text_size(sizes.tab_text_size.px())
            .text_color(rgb(theme.text))
            .max_w(sizes.title_max_width.px())
            .overflow_hidden()
            .text_ellipsis()
            .whitespace_nowrap()
            .child(self.title.clone())
    }
}

/// A tab as in the IntelliJ New UI: the shown one is a filled, outlined chip; the close button shows on it and
/// on hover only.
fn render_tab(chip: Chip<'_>, workspace: &Workspace, cx: &mut Context<Workspace>) -> Stateful<Div> {
    let theme = crate::theme::tabs(cx);
    let Chip { id, place, tab, shown, temporary, marker } = chip;
    let group = SharedString::from(format!("tab-{}", id.value()));
    let sizes = &theme.sizes.tabs;
    let close = div()
        .id(("close-tab", id.value()))
        .flex_none()
        .rounded(sizes.close_radius.px())
        .cursor_pointer()
        .hover(|style| style.bg(rgb(theme.hover)))
        .child(img(icons::embedded(theme.appearance, icons::CLOSE)).size(sizes.tab_icon_size.px()))
        .on_click(cx.listener(move |workspace, _event, window, cx| {
            cx.stop_propagation();
            workspace.close_tab(id, window, cx);
        }));
    let close = if shown { close } else { close.invisible().group_hover(group.clone(), |style| style.visible()) };
    let title_text = SharedString::from(tab.title(workspace));
    let title = div().max_w(sizes.title_max_width.px()).overflow_hidden().text_ellipsis().whitespace_nowrap();
    let title = match temporary.then(|| temporary_marker(cx)) {
        Some(TabMarker::Italic) => title.italic().child(title_text.clone()),
        Some(TabMarker::Prefix(prefix)) => title.child(format!("{prefix} {title_text}")),
        None => title.child(title_text.clone()),
    };
    let chip = div()
        .id(("tab", id.value()))
        .group(group)
        .flex()
        .flex_none()
        .items_center()
        .gap(sizes.tab_inner_gap.px())
        .h(sizes.tab_height.px())
        .pl(sizes.tab_padding_left.px())
        .pr(sizes.tab_padding_right.px())
        .rounded(sizes.tab_radius.px())
        .border(sizes.tab_border.px())
        .text_size(sizes.tab_text_size.px())
        .cursor_pointer()
        .child(tab.icon(workspace, theme))
        .children(tab.progress(workspace).and_then(|progress| progress_bar(progress, theme)))
        .child(title)
        .child(close)
        // a child bar: the chip's own border is its outline
        .children(tab.underline(workspace, theme).map(|colour| underline(colour, theme)))
        .relative()
        .tooltip(tooltip::text_tooltip(tab.tooltip(workspace)))
        .children(marker.map(|side| drop_marker(side, theme)))
        .on_drag(DraggedTab { id, title: title_text, hidden: workspace.drag_chip_hidden() }, |dragged, _offset, _window, cx| cx.new(|_| dragged.clone()))
        // the left half of a tab is the gap before it, the right half the gap after it
        .on_drag_move(cx.listener(move |workspace, event: &DragMoveEvent<DraggedTab>, _window, cx| {
            if event.bounds.contains(&event.event.position) {
                let gap = if event.event.position.x < event.bounds.center().x { place } else { place + 1 };
                workspace.hover_tab_drop(event.drag(cx).id, Some(gap), cx);
            }
        }));
    // for MCP's click: opens the default menu, as a right press does, at the pointer
    drop(target(format!("tab-menu:{}", id.value()), cx.listener(move |workspace, _event, window, cx| workspace.open_tab_menu(id, crate::workspace::DEFAULT_MENU, window, cx)), cx));
    // a press on a tab must not start a window drag: the bindings stop it there
    let handle = cx.entity().downgrade();
    let chip = mouse_binding::on_mouse_triggers(chip, format!("tab:{}", id.value()), move |trigger, window, cx| {
        // the workspace is gone only while the window closes
        let _ = handle.update(cx, |workspace, cx| workspace.run_tab_mouse(id, trigger, window, cx));
    });
    if shown {
        return chip.bg(rgb(theme.tab_active)).border_color(rgb(theme.tab_active_border));
    }
    chip.border_color(transparent_black()).hover(|style| style.bg(rgb(theme.hover)))
}

/// How a temporary tab's title shows it; italic until the scripts have loaded.
fn temporary_marker(cx: &App) -> TabMarker {
    cx.try_global::<TemporaryTabs>().map_or(TabMarker::Italic, |tabs| tabs.marker.clone())
}

/// The checkout colour bar under a tab.
fn underline(colour: u32, theme: Theme) -> Div {
    let sizes = &theme.sizes.tabs;
    let inset = sizes.underline_inset.px();
    div().absolute().bottom_0().left(inset).right(inset).h(sizes.underline_height.px()).rounded(sizes.underline_radius.px()).bg(rgb(colour))
}

/// The bar in the gap beside a tab where a dragged tab would land; centred on the gap, outside the tab.
fn drop_marker(side: Side, theme: Theme) -> Div {
    let sizes = &theme.sizes.tabs;
    let offset = px(-(sizes.gap.0 / 2. + sizes.drop_marker_width.0 / 2.) - sizes.tab_border.0);
    let bar = div()
        .absolute()
        .top(sizes.drop_marker_inset.px())
        .bottom(sizes.drop_marker_inset.px())
        .w(sizes.drop_marker_width.px())
        .rounded(sizes.drop_marker_radius.px())
        .bg(rgb(theme.accent));
    match side {
        Side::Left => bar.left(offset),
        Side::Right => bar.right(offset),
    }
}

/// Shown with `--restart`.
fn restart_button(theme: Theme, cx: &mut Context<Workspace>) -> Stateful<Div> {
    div()
        .id("restart")
        .p(theme.sizes.tabs.toolbar_button_padding.px())
        .rounded(theme.sizes.tabs.toolbar_button_radius.px())
        .cursor_pointer()
        .hover(|style| style.bg(rgb(theme.hover)))
        .child(img(icons::embedded(theme.appearance, icons::RESTART)).size(theme.sizes.tabs.toolbar_icon_size.px()))
        .tooltip(tooltip::text_tooltip("Restart IVE"))
        .on_mouse_down(MouseButton::Left, |_event, _window, cx| cx.stop_propagation())
        .on_click(target("restart".to_owned(), cx.listener(|workspace, _event, window, cx| workspace.restart(window, cx)), cx))
}

fn settings_button(theme: Theme, cx: &mut Context<Workspace>) -> Stateful<Div> {
    div()
        .id("open-settings")
        .p(theme.sizes.tabs.toolbar_button_padding.px())
        .rounded(theme.sizes.tabs.toolbar_button_radius.px())
        .cursor_pointer()
        .hover(|style| style.bg(rgb(theme.hover)))
        .child(img(icons::embedded(theme.appearance, icons::SETTINGS_20)).size(theme.sizes.tabs.toolbar_icon_size.px()))
        .tooltip(tooltip::text_tooltip("Settings"))
        .on_mouse_down(MouseButton::Left, |_event, _window, cx| cx.stop_propagation())
        .on_click(target("settings".to_owned(), cx.listener(|workspace, _event, window, cx| workspace.open_settings(window, cx)), cx))
}

fn centered_message(message: impl Into<SharedString>, color: u32) -> Div {
    div()
        .flex()
        .flex_1()
        .justify_center()
        .items_center()
        .text_color(rgb(color))
        .child(message.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::saved_tabs::PanelState;

    fn unloaded(saved: SavedTab) -> Tab {
        Tab::Unloaded(saved)
    }

    #[test]
    fn should_count_restored_tabs_as_open_for_what_they_show() {
        // given
        let file = unloaded(SavedTab::File { path: "/p/a.rs".into(), panel: PanelState::default() });
        let settings = unloaded(SavedTab::Settings { page: "Appearance".to_owned() });
        let overview = unloaded(SavedTab::ProjectOverview { project: "p".to_owned(), section: "Git".to_owned() });
        let scratch = unloaded(SavedTab::Scratch { project: "p".to_owned(), name: "n.md".to_owned(), panel: PanelState::default() });
        let log = unloaded(SavedTab::GitLog { project: "p".to_owned(), view: "Log".to_owned() });
        let sha = "0123456789abcdef0123456789abcdef01234567";
        let commit = unloaded(SavedTab::Commit { project: "p".to_owned(), sha: sha.to_owned(), title: "t".to_owned(), panel: PanelState::default() });
        let p = ProjectId::new("p");

        // when
        let found = [
            file.shows(Path::new("/p/a.rs")),
            settings.is_settings(),
            overview.is_project_overview(&p),
            scratch.shows_scratch(&crate::scratch::ScratchRef { project: p.clone(), name: "n.md".to_owned() }),
            log.is_git_log(&p),
            commit.is_commit(&p, sha),
        ];

        // then
        assert_eq!(found, [true; 6]);
        assert!(!file.shows(Path::new("/p/b.rs")));
    }

    #[test]
    fn should_open_restored_and_failed_tabs_permanent() {
        // given
        let settings = unloaded(SavedTab::Settings { page: "Appearance".to_owned() });
        let overview = unloaded(SavedTab::ProjectOverview { project: "p".to_owned(), section: "Git".to_owned() });
        let failed = Tab::Failed { title: "a.rs".to_owned(), error: "gone".into() };

        // when
        let kinds = [settings.temporary_kind(), overview.temporary_kind(), failed.temporary_kind()];

        // then
        assert_eq!(kinds, [None, None, None]);
    }
    #[test]
    fn should_count_a_restored_hub_as_the_hub() {
        // given
        let restored = unloaded(SavedTab::Hub);

        // when
        let is_hub = restored.is_hub();

        // then
        assert!(is_hub);
        assert!(!unloaded(SavedTab::Settings { page: "Appearance".to_owned() }).is_hub());
    }
}
