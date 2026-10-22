use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::fs;
use std::ops::Range;
use std::path::Path;
use std::rc::Rc;
use std::time::{Duration, SystemTime};

use gpui::{
    AnyElement, App, Bounds, Context, CursorStyle, DispatchPhase, Div, DragMoveEvent, FocusHandle, MouseButton, MouseDownEvent, MouseMoveEvent,
    MouseUpEvent, Pixels, Point, SharedString, Focusable, Stateful, StyledText, WeakEntity, Window, WindowControlArea, canvas, div, img, prelude::*, px,
    rgb, size, transparent_black,
};

use crate::click_targets::target;
use crate::clipboard;
use crate::colors::{ColorKey, ResolvedScheme, checkout_colour};
use crate::document::{DisplayLine, Document, Source};
use crate::hub::history::TabIdentity;
use crate::hub::view::HubTab;
use crate::icons;
use crate::line_view::{self, LineScroll, Metrics};
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
use crate::saved_tabs::{SavedTab, ScrollPosition};
use crate::project::{CheckoutRef, ProjectId};
use crate::scratch::{self, ScratchRef, ScratchTab};
use crate::scratch::icon::scratch_icon;
use crate::script::keymap::ClipboardTarget;
use crate::script::temporary_tabs::{TabKind, TabMarker, TemporaryTabs};
use crate::scrollbar::{ScrollState, horizontal_scrollbar, vertical_scrollbar};
use crate::settings_page::{self, SettingsView};
use crate::syntax::{self, Cancel, Highlight, Language, MAX_HIGHLIGHT_BYTES, PARSE_BUDGET, ParseError, Syntax};
use crate::tabs::TabId;
use crate::text_selection::{TextSelection, whole_line, word_around};
use crate::theme::{self, Theme};
use crate::tooltip;
use crate::watch::Watched;
use crate::window_controls::{TRAFFIC_LIGHTS_WIDTH, window_controls};
use crate::window_drag::WindowDrag;
use crate::workspace::{TabDragArea, Workspace};

/// Columns kept between the pointer and the view's edge when a selection drag scrolls sideways.
const DRAG_MARGIN_CHARS: f32 = 2.;
/// Notifications cannot see edits in collapsed folders, which the Worktrees cards count.
const WORKTREES_POLL: Duration = Duration::from_secs(5);

/// What a tab in the editor pane shows. Parts 2 and 3 add variants with their arms in each method below.
pub enum Tab {
    Document(EditorView),
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
            Self::Document(editor) => editor.document.path() == path,
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
            Self::Document(editor) => editor.document.file_name(),
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
            Self::Document(editor) => icons::file_icon(&editor.document.file_name()),
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
            Self::Document(editor) => SavedTab::File { path: editor.path().to_path_buf(), scroll: editor.scroll_position() },
            Self::Scratch(scratch) => {
                let (caret, scroll) = scratch.view_state(cx);
                SavedTab::Scratch { project: scratch.project().as_str().to_owned(), name: scratch.name().to_owned(), caret, scroll }
            }
            Self::Process(id) => SavedTab::Terminal { session: workspace.processes().get(*id)?.view.hosted_id()?.to_owned() },
            Self::ProjectOverview(overview) => {
                SavedTab::ProjectOverview { project: overview.project.as_str().to_owned(), section: overview.section.label().to_owned() }
            }
            Self::GitLog(tab) => SavedTab::GitLog { project: tab.project.as_str().to_owned(), view: tab.body.read(cx).section_name().to_owned() },
            Self::Commit(tab) => {
                SavedTab::Commit { project: tab.project.as_str().to_owned(), sha: tab.sha.as_str().to_owned(), title: tab.title.clone() }
            }
            Self::Diff(tab) => SavedTab::Diff { checkout: tab.checkout.path.clone(), request: tab.request.saved() },
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
            Self::Document(editor) => TabIdentity::File(editor.path().to_path_buf()),
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
            Self::Document(editor) => workspace.checkout_of_path(editor.document.path()),
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
            Self::Document(editor) => Some(Watched::File(editor.path().to_path_buf())),
            Self::Scratch(scratch) => Some(Watched::File(scratch.path().to_path_buf())),
            Self::ProjectOverview(overview) if overview.section == ProjectSection::Scratches => {
                scratch::store_of(&overview.project).ok().map(|store| Watched::Folder(store.dir().to_path_buf()))
            }
            Self::Process(_)
            | Self::ProjectOverview(_)
            | Self::Diff(_)
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
            Self::Document(editor) => editor.path().display().to_string(),
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
            Self::RunValues(tab) => tab.fields.first().map(|field| field.input.focus_handle(cx)),
            Self::Hub(hub) => Some(hub.focus_handle(cx)),
            // the Worktrees list's filter keeps the keyboard
            Self::ProjectOverview(ProjectOverview { section: ProjectSection::Worktrees, git: Some(views), .. }) => Some(views.list.focus_handle(cx)),
            Self::Document(_)
            | Self::ProjectOverview(_)
            | Self::Diff(_)
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
            Self::Document(editor) => editor.render_selectable(id, theme, window, cx).into_any_element(),
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

/// A read-only view of a document that draws only the lines on screen.
/// It does its own scrolling instead of using GPUI's list: with 10M lines the list's f32 pixel
/// positions (200M px) are only accurate to 16 px, so rows would overlap; here the position is a
/// line number plus a fraction.
pub struct EditorView {
    document: Document,
    scroll: Rc<LineScroll>,
    scrollbars: ScrollState,
    /// size and modification time when read; a reload of an unchanged file is skipped
    stamp: Option<Stamp>,
    /// a bar above the text: "deleted on disk", or why a reload failed
    notice: Option<SharedString>,
    /// set once the background parse finished; plain text until then
    syntax: Option<Syntax>,
    /// stops the background parse when this view goes away or its text is re-read
    cancel: Cancel,
    highlights: HighlightCache,
    /// what the mouse or Select All marked; dropped when the text is re-read, since it holds file offsets
    selection: Option<TextSelection>,
    /// a left-button drag that started in the text goes on
    dragging: bool,
    /// where the last frame put the view, for the mouse
    placed: Rc<Cell<Option<Placed>>>,
    /// the line `file:42` went to (from 0), drawn marked until the next press
    marked: Option<usize>,
}

/// The view's top-left corner in the window and its font metrics, as last painted.
#[derive(Clone, Copy)]
struct Placed {
    origin: Point<Pixels>,
    metrics: Metrics,
}

/// The line under the pointer and the file offset nearest to it.
struct Hit {
    index: usize,
    line: DisplayLine,
    column: usize,
    offset: usize,
}

/// A background parse of one document.
pub struct ParseJob {
    source: Source,
    language: Language,
    cancel: Cancel,
}

impl ParseJob {
    /// Set when the view went away or re-read its text; a result finished before that is stale then.
    pub fn cancel(&self) -> Cancel {
        self.cancel.clone()
    }

    pub fn run(self) -> Result<Syntax, ParseError> {
        syntax::parse(self.source.as_bytes(), self.language, &self.cancel, PARSE_BUDGET)
    }
}

impl Drop for EditorView {
    fn drop(&mut self) {
        self.cancel.cancel();
    }
}

impl EditorView {
    pub fn new(document: Document) -> Self {
        let scroll = Rc::new(LineScroll::new(document.line_count()));
        let scrollbars = ScrollState::new(scroll.clone());
        Self {
            stamp: stamp(document.path()),
            document,
            scroll,
            scrollbars,
            notice: None,
            syntax: None,
            cancel: Cancel::default(),
            highlights: HighlightCache::default(),
            selection: None,
            dragging: false,
            placed: Rc::default(),
            marked: None,
        }
    }

    /// Scrolls line `line` (from 1; past the end, the last) a third of the way down the view and marks it.
    pub fn reveal_line(&mut self, line: usize) {
        let index = line.saturating_sub(1).min(self.document.line_count().saturating_sub(1));
        let top = (index as f64 - self.scroll.viewport_lines() / 3.).max(0.).floor();
        self.scroll.set_position(ScrollPosition { top_line: top, left: 0. });
        self.marked = Some(index);
    }

    /// The marked line, from 1.
    #[cfg(test)]
    pub fn marked_line(&self) -> Option<usize> {
        self.marked.map(|index| index + 1)
    }

    /// A left press: places the start of a selection, selects a word on a double click and a line on a triple
    /// one; with Shift, extends the selection.
    pub fn press(&mut self, event: &MouseDownEvent) {
        self.marked = None;
        let Some(hit) = self.hit(event.position) else {
            return;
        };
        let bytes = self.document.bytes();
        let selection = match event.click_count {
            2 => TextSelection::of(word_around(bytes, &hit.line, hit.offset)),
            3.. => TextSelection::of(whole_line(bytes, &hit.line)),
            _ if event.modifiers.shift => self.selection.unwrap_or(TextSelection::at(hit.offset)).extended_to(hit.offset),
            _ => TextSelection::at(hit.offset),
        };
        self.selection = Some(selection);
        self.dragging = true;
    }

    /// A move of the pointer anywhere in the window; while a drag that began in the text goes on, the selection
    /// follows it, and past the view's edges scrolls lines and columns into view. False when nothing changed.
    pub fn drag_to(&mut self, point: Point<Pixels>) -> bool {
        let Some(selection) = self.selection.filter(|_| self.dragging) else {
            return false;
        };
        let (Some(hit), Some(placed)) = (self.hit(point), self.placed.get()) else {
            return false;
        };
        self.scroll.reveal_line(hit.index);
        let char_width = placed.metrics.char_width;
        self.scroll.reveal_x(f64::from(f32::from(char_width * hit.column as f32)), f64::from(f32::from(char_width * DRAG_MARGIN_CHARS)));
        self.selection = Some(selection.extended_to(hit.offset));
        true
    }

    pub fn release(&mut self) {
        self.dragging = false;
    }

    pub fn select_all(&mut self) {
        self.selection = Some(TextSelection::of(0..self.document.byte_len()));
    }

    /// Puts the selected text in `target`; a selection too large to copy is refused, with the reason on stderr.
    pub fn copy(&self, target: ClipboardTarget, cx: &App) {
        let Some(selection) = self.selection else {
            return;
        };
        match selection.text(self.document.bytes()) {
            Ok(text) if text.is_empty() => {}
            Ok(text) => clipboard::write(target, text, cx),
            Err(reason) => eprintln!("ive: {} not copied: {reason}", self.document.path().display()),
        }
    }

    /// The line under window point `point` (the first or last one above or below the text) and the column
    /// boundary nearest to it.
    fn hit(&self, point: Point<Pixels>) -> Option<Hit> {
        let Placed { origin, metrics } = self.placed.get()?;
        let row = f64::from(f32::from(point.y - origin.y) / f32::from(metrics.line_height)) + self.scroll.top();
        let index = (row.max(0.).floor() as usize).min(self.document.line_count().saturating_sub(1));
        let line = self.document.lines(index..index + 1).pop()?;
        let x = f64::from(f32::from(point.x - origin.x - metrics.gutter_width())) + self.scroll.left();
        let column = (x / f64::from(f32::from(metrics.char_width))).max(0.).round() as usize;
        let offset = line.offset_at(column);
        Some(Hit { index, line, column, offset })
    }

    pub fn path(&self) -> &Path {
        self.document.path()
    }

    /// Scrolls to the last line, where a log's newest lines are.
    pub fn reveal_end(&self) {
        self.scroll.reveal_line(self.document.line_count().saturating_sub(1));
    }

    pub fn scroll_position(&self) -> ScrollPosition {
        self.scroll.position()
    }

    pub fn restore_scroll(&self, position: ScrollPosition) {
        self.scroll.set_position(position);
    }

    /// The parse this document needs, if it gets highlighting at all: a known language, at most 8 MB.
    pub fn parse_job(&self) -> Option<ParseJob> {
        let language = Language::for_path(self.document.path(), self.document.bytes())?;
        (self.document.byte_len() <= MAX_HIGHLIGHT_BYTES).then(|| ParseJob {
            source: self.document.source(),
            language,
            cancel: self.cancel.clone(),
        })
    }

    pub fn set_syntax(&mut self, syntax: Syntax) {
        self.syntax = Some(syntax);
        self.highlights = HighlightCache::default();
    }

    /// Highlights of what `line` displays, queried per line so a huge line costs only its shown part, and once
    /// while the line stays on screen.
    fn highlights_of(&self, line: &DisplayLine) -> Rc<[Highlight]> {
        let Some(syntax) = &self.syntax else {
            return Rc::from([]);
        };
        let bytes = line.file_bytes();
        self.highlights.get_or_query(bytes.start, || syntax::spans(syntax, self.document.bytes(), bytes.clone()))
    }

    /// What a re-read compares the file with; none while a notice shows, so the next re-read reads.
    pub fn stamp(&self) -> Option<Stamp> {
        match self.notice {
            Some(_) => None,
            None => self.stamp,
        }
    }

    /// Takes what re-reading the file after a change on disk found: new text keeps the scroll position (clamped
    /// to the new length) and shows plain until it is parsed again (`parse_job`); a file that is gone or cannot be
    /// read keeps the old text, with a note saying why.
    pub fn take_reread(&mut self, reread: Reread) {
        match reread {
            Reread::Unchanged => {}
            Reread::Changed(document, stamp) => {
                self.scroll.set_line_count(document.line_count());
                self.document = document;
                self.stamp = stamp;
                self.notice = None;
                self.selection = None;
                self.dragging = false;
                self.forget_syntax();
            }
            Reread::Gone => self.mark_deleted(),
            Reread::Failed(reason) => self.notice = Some(format!("cannot reload: {reason}").into()),
        }
    }

    /// The old tree's offsets do not fit new text, and a parse still running for it is stale.
    fn forget_syntax(&mut self) {
        self.cancel.cancel();
        self.cancel = Cancel::default();
        self.syntax = None;
        self.highlights = HighlightCache::default();
    }

    /// The file was removed or renamed away; the text stays until it comes back.
    pub fn mark_deleted(&mut self) {
        self.notice = Some("deleted on disk".into());
    }

    /// The view in document tab `id`, where the mouse selects text.
    pub fn render_selectable(&self, id: TabId, theme: Theme, window: &mut Window, cx: &mut Context<Workspace>) -> impl IntoElement {
        let press = cx.listener(move |workspace, event: &MouseDownEvent, window, cx| workspace.press_document(id, event, window, cx));
        let release = cx.listener(move |workspace, _event: &MouseUpEvent, _window, cx| release_document(workspace, id, cx));
        let release_out = cx.listener(move |workspace, _event: &MouseUpEvent, _window, cx| release_document(workspace, id, cx));
        let workspace = cx.entity().downgrade();
        let placed = self.placed.clone();
        self.render_with(theme, window, cx, move |text, metrics| {
            text.cursor(CursorStyle::IBeam)
                .on_mouse_down(MouseButton::Left, press)
                .on_mouse_up(MouseButton::Left, release)
                // a drag that ends outside the text still ends
                .on_mouse_up_out(MouseButton::Left, release_out)
                .child(follow_drag(id, metrics, placed, workspace))
        })
    }

    pub fn render(&self, theme: Theme, window: &mut Window, cx: &App) -> impl IntoElement {
        self.render_with(theme, window, cx, |text, _metrics| text)
    }

    /// `interact` adds the mouse handling to the text area.
    fn render_with(
        &self,
        theme: Theme,
        window: &mut Window,
        cx: &App,
        interact: impl FnOnce(Stateful<Div>, Metrics) -> Stateful<Div>,
    ) -> impl IntoElement {
        let scheme = cx.global::<ResolvedScheme>();
        let metrics = Metrics::measure(window, cx, self.document.line_count());
        let top = self.scroll.top();
        let first = top.floor() as usize;
        let visible = self.scroll.viewport_lines().ceil() as usize + 1;
        let lines = self.document.lines(first..first + visible);
        self.highlights.start_frame();
        let widest_chars = lines.iter().map(|line| line.text.chars().count()).max().unwrap_or(0);
        self.scroll.set_content_width(metrics.text_width(widest_chars));
        let shift = px(self.scroll.left() as f32);
        let rows: Vec<Div> = lines
            .iter()
            .enumerate()
            .map(|(offset, line)| {
                let text = styled_line(line, &self.highlights_of(line), scheme);
                let selected = self.selection.and_then(|selection| selection.columns_on(line));
                let marked = self.marked == Some(first + offset);
                render_line(first + offset, Row { text, selected, marked }, shift, &metrics, theme)
            })
            .collect();
        let wheel_scroll = self.scroll.clone();
        let line_height = metrics.line_height;
        let text = div()
            .id("editor")
            .relative()
            .flex_1()
            .min_h_0()
            .overflow_hidden()
            .font_family(theme::mono_font(cx))
            .text_size(metrics.font_size)
            .line_height(metrics.line_height)
            .text_color(rgb(scheme.get(ColorKey::TEXT).foreground.unwrap_or(theme.text)))
            .on_scroll_wheel(move |event, window, _cx| wheel_scroll.scroll_by_wheel(event, line_height, window))
            .child(line_view::measure_viewport(self.scroll.clone(), metrics));
        let text = interact(text, metrics)
            .child(
                div()
                    .absolute()
                    .left_0()
                    .right_0()
                    .top(-metrics.line_height * (top - first as f64) as f32)
                    .children(rows),
            )
            .child(vertical_scrollbar(&self.scrollbars, &theme::view_style(&theme)))
            .child(horizontal_scrollbar(&self.scrollbars, &theme::view_style(&theme)));
        let notice = self.notice.clone().map(|notice| {
            let sizes = &theme.sizes.editor;
            div()
                .flex_none()
                .px(sizes.notice_padding_x.px())
                .py(sizes.notice_padding_y.px())
                .bg(rgb(theme.hover))
                .text_size(sizes.notice_text_size.px())
                .text_color(rgb(theme.error))
                .child(notice)
        });
        div().flex().flex_col().flex_1().min_h_0().children(notice).child(text)
    }
}

fn release_document(workspace: &mut Workspace, id: TabId, cx: &mut Context<Workspace>) {
    workspace.update_document_tab(
        id,
        |editor| {
            editor.release();
            false
        },
        cx,
    );
}

/// Records where the view was painted, and follows a selection drag over the whole window: an element's own
/// mouse-move listener hears only moves over it.
fn follow_drag(id: TabId, metrics: Metrics, into: Rc<Cell<Option<Placed>>>, workspace: WeakEntity<Workspace>) -> impl IntoElement {
    canvas(
        |_bounds, _window, _cx| {},
        move |bounds, (), window, _cx| {
            into.set(Some(Placed { origin: bounds.origin, metrics }));
            window.on_mouse_event(move |event: &MouseMoveEvent, phase, _window, cx| {
                if phase == DispatchPhase::Bubble && event.pressed_button == Some(MouseButton::Left) {
                    // the workspace is gone only while the window closes
                    let _ = workspace.update(cx, |workspace, cx| workspace.update_document_tab(id, |editor| editor.drag_to(event.position), cx));
                }
            });
        },
    )
    .absolute()
    .size_full()
}

/// Highlights of the lines drawn in the last frame, by the file offset each starts at; repaints and horizontal
/// scrolling reuse them instead of querying the tree again.
#[derive(Default)]
struct HighlightCache {
    current: RefCell<HashMap<usize, Rc<[Highlight]>>>,
    previous: RefCell<HashMap<usize, Rc<[Highlight]>>>,
}

impl HighlightCache {
    /// Lines not asked for until the next frame starts are forgotten then.
    fn start_frame(&self) {
        let current = self.current.take();
        self.previous.replace(current);
    }

    fn get_or_query(&self, line_start: usize, query: impl FnOnce() -> Vec<Highlight>) -> Rc<[Highlight]> {
        let known = self.current.borrow().get(&line_start).cloned();
        let highlights = known
            .or_else(|| self.previous.borrow_mut().remove(&line_start))
            .unwrap_or_else(|| Rc::from(query()));
        self.current.borrow_mut().insert(line_start, highlights.clone());
        highlights
    }
}

/// A file's size and modification time: when both are as before, the file is not read again.
pub type Stamp = (u64, SystemTime);

/// What re-reading an open document found.
pub enum Reread {
    /// size and modification time as when it was last read
    Unchanged,
    Changed(Document, Option<Stamp>),
    Gone,
    Failed(String),
}

/// Re-reads the document at `path` unless its stamp is still `known`. Reads the disk; run it on the background
/// executor.
pub fn reread(path: &Path, known: Option<Stamp>) -> Reread {
    let now = stamp(path);
    if now.is_some() && now == known {
        return Reread::Unchanged;
    }
    match Document::open(path) {
        Ok(document) => Reread::Changed(document, now),
        Err(error) if path.exists() => Reread::Failed(error.to_string()),
        Err(_) => Reread::Gone,
    }
}

fn stamp(path: &Path) -> Option<Stamp> {
    let metadata = fs::metadata(path).ok()?;
    Some((metadata.len(), metadata.modified().ok()?))
}

fn styled_line(line: &DisplayLine, highlights: &[Highlight], scheme: &ResolvedScheme) -> StyledText {
    let styles = line.place(highlights).into_iter().map(|(range, key)| (range, scheme.get(key).highlight_style()));
    StyledText::new(line.text.clone()).with_highlights(styles)
}

/// A line's text and the columns of it that are selected.
struct Row {
    text: StyledText,
    selected: Option<Range<usize>>,
    /// the line `file:42` went to
    marked: bool,
}

fn render_line(index: usize, row: Row, shift: Pixels, metrics: &Metrics, theme: Theme) -> Div {
    let number = format!("{:>width$}", index + 1, width = metrics.number_chars);
    div()
        .flex()
        .h(metrics.line_height)
        .when(row.marked, |line| line.bg(rgb(theme.hover)))
        .child(
            div()
                .flex_none()
                .w(metrics.gutter_width())
                .pl(metrics.char_width)
                .text_color(rgb(theme.muted))
                .child(number),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .overflow_hidden()
                .child(div().relative().left(-shift).whitespace_nowrap().children(row.selected.map(|columns| selection_bar(columns, metrics, theme))).child(row.text)),
        )
}

/// The selection's background behind `columns` of a line; glyphs of the monospace font sit on the column grid.
fn selection_bar(columns: Range<usize>, metrics: &Metrics, theme: Theme) -> Div {
    div()
        .absolute()
        .top_0()
        .h(metrics.line_height)
        .left(metrics.char_width * columns.start as f32)
        .w(metrics.char_width * columns.len() as f32)
        .bg(rgb(theme.selected))
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

    fn unloaded(saved: SavedTab) -> Tab {
        Tab::Unloaded(saved)
    }

    #[test]
    fn should_count_restored_tabs_as_open_for_what_they_show() {
        // given
        let file = unloaded(SavedTab::File { path: "/p/a.rs".into(), scroll: ScrollPosition::default() });
        let settings = unloaded(SavedTab::Settings { page: "Appearance".to_owned() });
        let overview = unloaded(SavedTab::ProjectOverview { project: "p".to_owned(), section: "Git".to_owned() });
        let scratch = unloaded(SavedTab::Scratch {
            project: "p".to_owned(),
            name: "n.md".to_owned(),
            caret: crate::saved_tabs::CaretPosition::default(),
            scroll: ScrollPosition::default(),
        });
        let log = unloaded(SavedTab::GitLog { project: "p".to_owned(), view: "Log".to_owned() });
        let sha = "0123456789abcdef0123456789abcdef01234567";
        let commit = unloaded(SavedTab::Commit { project: "p".to_owned(), sha: sha.to_owned(), title: "t".to_owned() });
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
    use std::cell::Cell;

    fn editor_for(name: &str, text: &[u8]) -> (tempfile::TempDir, EditorView) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(name);
        fs::write(&path, text).unwrap();
        let editor = EditorView::new(Document::open(&path).unwrap());
        (dir, editor)
    }

    #[test]
    fn should_query_each_shown_line_once_across_repaints() {
        // given
        let cache = HighlightCache::default();
        let queries = Cell::new(0);
        let query = || {
            queries.set(queries.get() + 1);
            Vec::new()
        };

        // when
        for _ in 0..3 {
            cache.start_frame();
            cache.get_or_query(0, query);
            cache.get_or_query(10, query);
        }

        // then
        assert_eq!(queries.get(), 2);
    }

    #[test]
    fn should_forget_lines_not_shown_in_the_last_frame() {
        // given
        let cache = HighlightCache::default();
        let queries = Cell::new(0);
        let query = || {
            queries.set(queries.get() + 1);
            Vec::new()
        };
        cache.start_frame();
        cache.get_or_query(0, query);
        cache.start_frame();
        cache.get_or_query(10, query);

        // when
        cache.start_frame();
        cache.get_or_query(0, query);

        // then
        assert_eq!(queries.get(), 3);
    }

    #[test]
    fn should_parse_only_known_languages_up_to_the_limit() {
        // given
        let small = editor_for("data.json", b"{}");
        let big = editor_for("big.json", &vec![b' '; MAX_HIGHLIGHT_BYTES + 1]);
        let log = editor_for("server.log", b"plain");

        // when
        let jobs = [small.1.parse_job().is_some(), big.1.parse_job().is_some(), log.1.parse_job().is_some()];

        // then
        assert_eq!(jobs, [true, false, false]);
    }

    #[test]
    fn should_colour_java_line_with_tab_and_crlf() {
        // given
        let (_dir, mut editor) = editor_for("A.java", b"class A {\r\n\tint x;\r\n}\r\n");
        editor.set_syntax(editor.parse_job().unwrap().run().unwrap());

        // when
        let line = editor.document.lines(1..2).remove(0);
        let placed = line.place(&editor.highlights_of(&line));

        // then
        let int = placed.iter().find(|(range, _)| &line.text[range.clone()] == "int").map(|(_, key)| key.id());
        assert_eq!((line.text.as_ref(), int), ("    int x;", Some("JAVA_KEYWORD")));
    }

    #[test]
    fn should_bound_highlighting_to_displayed_part_of_long_line() {
        // given
        let json = format!("[{}1]", "1, ".repeat(1_000_000));
        let (_dir, mut editor) = editor_for("long.json", json.as_bytes());
        editor.set_syntax(editor.parse_job().unwrap().run().unwrap());
        let line = editor.document.lines(0..1).remove(0);

        // when
        let highlights = editor.highlights_of(&line);

        // then
        let end = highlights.iter().map(|highlight| highlight.bytes.end).max().unwrap();
        assert!(end <= line.file_bytes().end, "{end} > {:?}", line.file_bytes());
        assert!(line.file_bytes().len() <= crate::document::MAX_LINE_CHARS * 4);
    }

    #[test]
    fn should_reload_a_changed_file_and_say_when_it_is_deleted() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("notes.md");
        fs::write(&path, "one\ntwo\n").unwrap();
        let mut editor = EditorView::new(Document::open(&path).unwrap());
        fs::write(&path, "one\ntwo\nthree\nfour\n").unwrap();

        // when
        editor.take_reread(reread(&path, editor.stamp()));
        let lines = editor.document.line_count();
        fs::remove_file(&path).unwrap();
        editor.take_reread(reread(&path, editor.stamp()));

        // then
        assert_eq!((lines, editor.notice.as_ref().map(SharedString::as_ref)), (4, Some("deleted on disk")));
        assert_eq!(editor.document.line_count(), 4, "the text stays");
    }

    #[test]
    fn should_not_read_a_file_whose_size_and_modification_time_are_unchanged() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("notes.md");
        fs::write(&path, "one\n").unwrap();
        let editor = EditorView::new(Document::open(&path).unwrap());

        // when
        let reread = reread(&path, editor.stamp());

        // then
        assert!(matches!(reread, Reread::Unchanged));
    }

    #[test]
    fn should_mark_the_revealed_line() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.txt");
        std::fs::write(&path, (1..=100).map(|line| format!("line {line}\n")).collect::<String>()).unwrap();
        let mut editor = EditorView::new(Document::open(&path).unwrap());

        // when
        editor.reveal_line(42);

        // then
        let top = editor.scroll_position().top_line;
        assert_eq!(editor.marked_line(), Some(42));
        assert!(top <= 41. && 41. < top + editor.scroll.viewport_lines(), "top {top}");
    }

    #[test]
    fn should_keep_a_line_past_the_end_inside_the_text() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.txt");
        std::fs::write(&path, "one\ntwo\n").unwrap();
        let mut editor = EditorView::new(Document::open(&path).unwrap());

        // when
        editor.reveal_line(500);

        // then
        assert_eq!(editor.marked_line(), Some(editor.document.line_count()));
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
