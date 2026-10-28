use std::cell::Cell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;

use codeview::document::{Reread, Stamp};
use codeview::panel::Panel;

use gpui::{
    AnyElement, App, AppContext, Bounds, Context, Div, DragMoveEvent, Entity, FocusHandle, MouseButton, MouseDownEvent, SharedString, Focusable,
    Stateful, Window, WindowControlArea, div, img, prelude::*, px, rgb, size, transparent_black,
};

use crate::click_targets::target;
use crate::colors::{mixed, tinted};
use crate::colors::checkout_colour;
use crate::failure::Failure;
use crate::favicon_badge::{Badge, badged, favicon_or};
pub use crate::file_tab::FileTab;
use crate::image_tab::ImageTab;
use crate::hub::history::TabIdentity;
use crate::hub::view::HubTab;
use crate::icons;
use crate::mouse_binding;
use crate::overview::commit_tab::CommitTab;
use crate::overview::diff_tab::DiffTab;
use crate::overview::git_log_tab::{GitLogTab, LogOf};
use crate::overview::{self, ProjectOverview, ProjectSection};
use crate::process::{ProcessId, ProcessKind};
use crate::conflicts_tab::{ConflictsTab, render_conflicts_tab};
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
use crate::tab_groups::{self, CueKind, FramePart, RowItem, Spread, TabFacts, TabGroup};
use crate::tabs::TabId;
use crate::theme::Theme;
use crate::tooltip;
use crate::watch::Watched;
use crate::window_controls::{TRAFFIC_LIGHTS_WIDTH, above_title_bar, window_controls};
use crate::window_drag::WindowDrag;
use crate::workspace::{TabDragArea, Workspace};

/// Notifications cannot see edits in collapsed folders, which the Worktrees cards count.
const WORKTREES_POLL: Duration = Duration::from_secs(5);

/// What a tab in the editor pane shows. Parts 2 and 3 add variants with their arms in each method below.
pub enum Tab {
    Document(FileTab),
    /// an image file, shown, not edited
    Image(ImageTab),
    /// a process owned by the workspace's process list; closing the tab keeps the process
    Process(ProcessId),
    ProjectOverview(ProjectOverview),
    /// one file's diff, frozen when it opened
    Diff(DiffTab),
    Commit(CommitTab),
    /// one per project
    GitLog(GitLogTab),
    Settings(SettingsView),
    Failed { title: String, failure: Failure },
    /// a scratch of a project, in its editor
    Scratch(ScratchTab),
    /// the values a run needs and has none of; never saved
    RunValues(RunValuesTab),
    /// every run configuration of a checkout; at most one per checkout, never saved
    RunConfigs(RunConfigsTab),
    /// a copy or move with files in the way, asking before it writes; never saved
    Conflicts(Box<ConflictsTab>),
    /// a tab restored at start, loaded when first shown
    Unloaded(SavedTab),
    /// Ctrl+Shift+E: recent tabs, changes, processes and every file; at most one
    Hub(Box<HubTab>),
}

impl Tab {
    /// A tab of the file at `path`: an image tab for an image, else a file tab of its text; why it cannot be read.
    pub fn of_file(path: &Path, cx: &mut App) -> Result<Self, String> {
        if crate::image_tab::is_image(path) {
            std::fs::metadata(path).map_err(|error| format!("cannot read {}: {error}", path.display()))?;
            return Ok(Self::Image(ImageTab::new(path.to_path_buf())));
        }
        let opened = crate::document::Document::open_stamped(path).map_err(|error| error.to_string())?;
        Ok(Self::Document(FileTab::new(opened, cx)))
    }

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
            Self::Image(image) => image.path() == path,
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
            Self::GitLog(tab) => &tab.project == id && tab.of == LogOf::Project,
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
            Self::Image(_) => "image",
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
            Self::Conflicts(_) => "conflicts",
            Self::Unloaded(saved) => saved.kind(),
            Self::Hub(_) => "hub",
        }
    }

    pub fn title(&self, workspace: &Workspace) -> String {
        match self {
            Self::Document(file) => file.file_name(),
            Self::Image(image) => image.file_name(),
            Self::Process(id) => workspace.processes().get(*id).map_or_else(|| "Terminal".to_owned(), |process| process.title()),
            Self::ProjectOverview(overview) => {
                workspace.project(&overview.project).map_or_else(|| overview.project.to_string(), |project| project.name.clone())
            }
            Self::Diff(tab) => tab.title.clone(),
            Self::Commit(tab) => tab.title.clone(),
            Self::GitLog(GitLogTab { of: LogOf::Path { path, .. }, .. }) => {
                format!("History · {}", path.file_name().map_or_else(|| path.display().to_string(), |name| name.to_string_lossy().into_owned()))
            }
            Self::GitLog(_) => "Git log".to_owned(),
            Self::Settings(_) => "Settings".to_owned(),
            Self::Failed { title, .. } => title.clone(),
            Self::Scratch(scratch) => scratch.name().to_owned(),
            Self::RunValues(tab) => tab.title(),
            Self::RunConfigs(_) => "Run configurations".to_owned(),
            Self::Conflicts(tab) => tab.title(),
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

    /// Its icon, with the attention dot while its process waits for the user; `behind` is the chip's colour.
    pub fn marked_icon(&self, workspace: &Workspace, theme: Theme, behind: u32) -> AnyElement {
        match self.attention(workspace) {
            Some(_) => crate::attention_dot::with_dot(self.icon(workspace, theme), theme, behind).into_any_element(),
            None => self.icon(workspace, theme),
        }
    }

    /// What its process said when it called for the user, until they look.
    fn attention(&self, workspace: &Workspace) -> Option<String> {
        match self {
            Self::Process(id) => workspace.processes().get(*id).and_then(|process| process.attention()).map(|attention| attention.text.clone()),
            _ => None,
        }
    }

    fn icon_name(&self, workspace: &Workspace) -> &'static str {
        match self {
            Self::Document(file) => icons::file_icon(&file.file_name()),
            Self::Image(image) => icons::file_icon(&image.file_name()),
            Self::Process(id) => workspace.processes().get(*id).map_or(ProcessKind::Terminal.icon(), |process| process.icon()),
            Self::ProjectOverview(_) => icons::PROJECT_FOLDER,
            Self::Diff(_) => icons::CHANGES,
            Self::Commit(_) => icons::COMMIT,
            Self::GitLog(_) => icons::HISTORY,
            Self::Settings(_) => icons::SETTINGS,
            Self::Failed { title, .. } => icons::file_icon(title),
            Self::Scratch(scratch) => icons::file_icon(scratch.name()),
            Self::RunValues(_) | Self::RunConfigs(_) => icons::RUN_CONFIG,
            Self::Conflicts(_) => icons::WARNING,
            Self::Unloaded(saved) => saved_icon(saved),
            Self::Hub(_) => icons::LOCATE,
        }
    }

    /// What `state.toml` keeps to open this tab again; `None` for a tab that is not saved: a failed one, or a
    /// terminal that ends with IVE.
    /// The panel whose scroll and layout the saved tab keeps, for a file, diff or commit tab (a scratch saves its
    /// own).
    pub fn saved_panel(&self, cx: &App) -> Option<Entity<Panel>> {
        match self {
            Self::Document(file) => Some(file.panel.clone()),
            Self::Diff(tab) => Some(tab.panel(cx)),
            Self::Commit(tab) => Some(tab.panel(cx)),
            _ => None,
        }
    }

    pub fn saved(&self, workspace: &Workspace, cx: &App) -> Option<SavedTab> {
        let tab = match self {
            Self::Document(file) => SavedTab::File { path: file.path().to_path_buf(), panel: file.panel.read(cx).state() },
            Self::Image(image) => SavedTab::File { path: image.path().to_path_buf(), panel: Default::default() },
            Self::Scratch(scratch) => {
                SavedTab::Scratch { project: scratch.project().as_str().to_owned(), name: scratch.name().to_owned(), panel: scratch.panel_state(cx) }
            }
            Self::Process(id) => SavedTab::Terminal { session: workspace.processes().get(*id)?.view.hosted_id()?.to_owned() },
            Self::ProjectOverview(overview) => {
                SavedTab::ProjectOverview { project: overview.project.as_str().to_owned(), section: overview.section.label().to_owned() }
            }
            // a path's history is not restored
            Self::GitLog(GitLogTab { of: LogOf::Path { .. }, .. }) => return None,
            Self::GitLog(tab) => SavedTab::GitLog { project: tab.project.as_str().to_owned(), view: tab.body.read(cx).section_name().to_owned() },
            Self::Commit(tab) => {
                let panel = tab.panel_state(cx);
                SavedTab::Commit { project: tab.project.as_str().to_owned(), sha: tab.sha.as_str().to_owned(), title: tab.title.clone(), panel }
            }
            Self::Diff(tab) => SavedTab::Diff { checkout: tab.checkout.path.clone(), request: tab.request.saved(), panel: tab.panel_state(cx) },
            Self::Settings(view) => SavedTab::Settings { page: view.page_name().to_owned() },
            Self::Hub(_) => SavedTab::Hub,
            Self::Failed { .. } | Self::RunValues(_) | Self::RunConfigs(_) | Self::Conflicts(_) => return None,
            Self::Unloaded(saved) => saved.clone(),
        };
        Some(tab)
    }

    /// What the hub's history knows this tab by; `None` for a tab it does not list (a failed one).
    pub fn identity(&self, workspace: &Workspace) -> Option<TabIdentity> {
        let identity = match self {
            Self::Document(file) => TabIdentity::File(file.path().to_path_buf()),
            Self::Image(image) => TabIdentity::File(image.path().to_path_buf()),
            Self::Process(id) => match workspace.processes().get(*id).and_then(|process| process.view.hosted_id()) {
                Some(session) => TabIdentity::Terminal { session: session.to_owned() },
                None => TabIdentity::Process(id.value()),
            },
            Self::ProjectOverview(overview) => TabIdentity::ProjectOverview(overview.project.as_str().to_owned()),
            Self::Diff(tab) => TabIdentity::Diff { checkout: tab.checkout.path.clone(), path: tab.request.saved().path },
            Self::Commit(tab) => TabIdentity::Commit { project: tab.project.as_str().to_owned(), sha: tab.sha.as_str().to_owned() },
            Self::GitLog(GitLogTab { of: LogOf::Path { .. }, .. }) => return None,
            Self::GitLog(tab) => TabIdentity::GitLog(tab.project.as_str().to_owned()),
            Self::Settings(_) => TabIdentity::Settings,
            Self::Failed { .. } | Self::Hub(_) | Self::RunConfigs(_) | Self::Conflicts(_) => return None,
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
            Self::Image(image) => workspace.checkout_of_path(image.path()),
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
            Self::Conflicts(tab) => Some(tab.plan.operation.target.checkout.clone()),
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
            | Self::Image(_)
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
            | Self::Conflicts(_)
            | Self::Unloaded(_)
            | Self::Hub(_) => Vec::new(),
        }
    }

    /// How often, while the window is active, the statuses this tab shows are re-read.
    pub fn poll_interval(&self) -> Option<Duration> {
        match self {
            Self::ProjectOverview(overview) if overview.section == ProjectSection::Worktrees => Some(WORKTREES_POLL),
            Self::Document(_)
            | Self::Image(_)
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
            | Self::Conflicts(_)
            | Self::Unloaded(_)
            | Self::Hub(_) => None,
        }
    }

    /// The file this tab shows, registered for changes on disk; a project overview's Scratches section, its folder.
    pub fn watched(&self) -> Option<Watched> {
        match self {
            Self::Document(file) => Some(Watched::File(file.path().to_path_buf())),
            Self::Image(image) => Some(Watched::File(image.path().to_path_buf())),
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
            | Self::Conflicts(_)
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

    /// Something changed in a git dir: a file tab reads its change markers' base again, as HEAD may have moved.
    pub fn git_changed(&self, git_dir: &Path, cx: &mut App) {
        if let Self::Document(file) = self {
            file.git_changed(git_dir, cx);
        }
    }

    /// The file at `path` changed or went: an image tab of it reads it again.
    pub fn file_changed(&self, path: &Path, cx: &mut App) {
        if let Self::Image(image) = self
            && image.path() == path
        {
            image.reload(cx);
        }
    }

    /// What re-reading the file this tab edits found.
    pub fn take_reread(&mut self, reread: Reread, cx: &mut App) {
        match self {
            Self::Document(file) => file.take_reread(reread, cx),
            Self::Diff(diff) => diff.take_reread(reread, cx),
            Self::Image(_)
            | Self::Process(_)
            | Self::ProjectOverview(_)
            | Self::Commit(_)
            | Self::GitLog(_)
            | Self::Settings(_)
            | Self::Failed { .. }
            | Self::Scratch(_)
            | Self::RunValues(_)
            | Self::RunConfigs(_)
            | Self::Conflicts(_)
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

    /// The path whose module owns this tab: a file's, a diff's (relative to its checkout), a process's working folder
    /// or its run's folder; `None` for a tab of the whole project or of none.
    pub fn owner_path(&self, workspace: &Workspace) -> Option<PathBuf> {
        match self {
            Self::Document(file) => Some(file.path().to_path_buf()),
            Self::Image(image) => Some(image.path().to_path_buf()),
            Self::Diff(tab) => Some(PathBuf::from(tab.request.file.path.display())),
            Self::Process(id) => {
                let process = workspace.processes().get(*id)?;
                process.directory().map(Path::to_path_buf).or_else(|| process.run.as_ref().map(|run| run.folder.clone()))
            }
            Self::Unloaded(saved) => saved.owner_path(),
            _ => None,
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
            Self::Image(image) => image.path().display().to_string(),
            _ => self.title(workspace),
        };
        let owner = match self {
            Self::ProjectOverview(_) | Self::Diff(_) | Self::Settings(_) | Self::Failed { .. } | Self::Unloaded(_) | Self::Hub(_) => None,
            Self::Document(_) | Self::Image(_) | Self::Process(_) | Self::RunValues(_) | Self::RunConfigs(_) | Self::Conflicts(_) => {
                self.checkout(workspace).map(|checkout| workspace.checkout_label(&checkout))
            }
            Self::Commit(CommitTab { project, .. }) | Self::GitLog(GitLogTab { project, .. }) => {
                workspace.project(project).map(|project| project.name.clone())
            }
            Self::Scratch(scratch) => Some(scratch.project_name().to_owned()),
        };
        let tooltip = match owner {
            Some(owner) => format!("{owner} — {title}"),
            None => title,
        };
        match self.attention(workspace) {
            Some(said) => format!("{tooltip}\n● {said}"),
            None => tooltip,
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
            Self::Image(_)
            | Self::ProjectOverview(_)
            | Self::Commit(_)
            | Self::GitLog(_)
            | Self::Settings(_)
            | Self::Failed { .. }
            | Self::RunConfigs(_)
            | Self::Conflicts(_)
            | Self::Unloaded(_) => None,
        }
    }


    fn render(&self, id: TabId, workspace: &Workspace, window: &mut Window, cx: &mut Context<Workspace>) -> AnyElement {
        let theme = crate::theme::tabs(cx);
        match self {
            Self::Document(file) => file.render(theme, cx),
            Self::Image(image) => image.render(theme),
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
            Self::Settings(view) => settings_page::render(settings_page::Page { view, update_status: workspace.update_status() }, theme, cx).into_any_element(),
            Self::Failed { title, failure } => div().id("failure").flex_1().min_h_0().overflow_y_scroll().child(failure.render(&id.value().to_string(), title, cx)).into_any_element(),
            Self::Scratch(scratch) => scratch.view().clone().into_any_element(),
            Self::RunValues(tab) => run_values::render_values_tab(id, tab, cx),
            Self::RunConfigs(tab) => render_run_configs_tab(id, tab, workspace, cx),
            Self::Conflicts(tab) => render_conflicts_tab(id, tab, workspace, cx),
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
    let facts = workspace.tab_group_facts();
    let groups = workspace.tab_groups(&facts);
    let items = tab_groups::row_items(&groups);
    let places: HashMap<TabId, usize> = tabs.row().enumerate().map(|(place, (id, _))| (id, place)).collect();
    // the row's last drawn tab, which the groups need not have last in the tab order
    let last = items.iter().rev().find_map(|item| match item {
        RowItem::Tab(_, id) => places.get(id).copied(),
        RowItem::Head(_) => None,
    });
    let last = last.unwrap_or(0);
    let view = GroupView {
        // a gap stays marked after a drag ends outside the window, until the next drag
        drop: DropGap { gap: workspace.tab_drop_gap().filter(|_| cx.has_active_drag()), last },
        shown: tabs.active_id(),
        places,
        spread: Spread { worktrees: tab_groups::spread_worktrees(&facts), owners: tab_groups::spread_owners(&facts) },
        groups,
        facts: facts.iter().map(|fact| (fact.id, fact)).collect(),
        theme,
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
    // each item pads half the gap between tabs on either side, so a group's frame runs unbroken under its items
    let parts: Vec<FramePart> = items.iter().map(|item| tab_groups::frame_part(*item, &view.groups)).collect();
    let previous: Vec<Option<FramePart>> = std::iter::once(None).chain(parts.iter().copied().map(Some)).collect();
    let strip = div()
        .id("tab-strip")
        .flex()
        .flex_1()
        .min_w_0()
        .h_full()
        .items_center()
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
        .children(items.into_iter().zip(parts).zip(previous).filter_map(|((item, part), before)| {
            let element = render_row_item(item, &view, workspace, cx)?;
            Some(framed(element, Framing { part, before, active: view.holds_shown(item) }, theme))
        }))
        .child(end);
    let restart = workspace.restart_script().map(|_| restart_button(theme, cx));
    let update = workspace.update_to_restart_onto().map(|tag| update_button(tag.to_owned(), theme, cx));
    let tools = div()
        .flex()
        .flex_none()
        .items_center()
        .gap(sizes.tools_gap.px())
        .h_full()
        .pl(sizes.tools_padding_left.px())
        .children(update)
        .children(restart)
        .children(crate::waiting_menu::waiting_button(workspace, theme, cx))
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
        .children(crate::menu_view::render_menu(workspace, theme, window, cx))
        .children(crate::tab_group_menu::render(workspace, window, cx))
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
        .child(tab.marked_icon(workspace, theme, theme.sidebar))
        .children(tab.progress(workspace).and_then(|progress| progress_bar(progress, theme)))
        .child(div().max_w(sizes.title_max_width.px()).overflow_hidden().text_ellipsis().whitespace_nowrap().child(title.clone()))
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
        .map(above_title_bar)
        .p(theme.sizes.tabs.toolbar_button_padding.px())
        .rounded(theme.sizes.tabs.toolbar_button_radius.px())
        .cursor_pointer()
        .hover(|style| style.bg(rgb(theme.hover)))
        .child(img(icons::embedded(theme.appearance, icons::DOCK)).size(theme.sizes.tabs.tab_icon_size.px()))
        .tooltip(tooltip::text_tooltip("Move to main window"))
        .on_mouse_down(MouseButton::Left, |_event, _window, cx| cx.stop_propagation())
        .on_click(target("attach-tab".to_owned(), cx.listener(move |workspace, _event, window, cx| workspace.attach_tab(id, window, cx)), cx))
}

/// What the tab row's items are drawn with: where a dragged tab would land, each tab's place in the row, what the
/// row spans, its groups and the tabs' facts.
struct GroupView<'a> {
    drop: DropGap,
    shown: Option<TabId>,
    places: HashMap<TabId, usize>,
    spread: Spread,
    groups: Vec<TabGroup>,
    facts: HashMap<TabId, &'a TabFacts>,
    theme: Theme,
}

/// The gap a dragged tab would land in, and the last tab's place.
#[derive(Clone, Copy)]
struct DropGap {
    gap: Option<usize>,
    last: usize,
}

impl DropGap {
    /// The side of the tab at `place` its drop marker shows on.
    fn marker(self, place: usize) -> Option<Side> {
        match self.gap {
            Some(gap) if gap == place => Some(Side::Left),
            Some(gap) if gap == place + 1 && place == self.last => Some(Side::Right),
            _ => None,
        }
    }
}

/// How an item draws its part of its group's frame: the part, the one before it (a group after a frame keeps the
/// gap between groups), and whether its group holds the shown tab.
struct Framing {
    part: FramePart,
    before: Option<FramePart>,
    active: bool,
}

/// An item of the row in its part of its group's frame: the head opens it (left, top, bottom), a tab carries its top
/// and bottom, the group's last tab closes it (right, top, bottom); the frame in the shown tab's outline colour round
/// the group holding it. Borders take their width inside, so the paddings leave it out.
fn framed(element: AnyElement, framing: Framing, theme: Theme) -> AnyElement {
    let sizes = &theme.sizes.tabs;
    let half = px(sizes.gap.0 / 2.);
    let (border, radius) = (sizes.group_border.px(), sizes.group_radius.px());
    let colour = if framing.active { theme.tab_active_border } else { theme.border };
    let part = div().flex().flex_none().items_center().px(half).child(element);
    let gapped = |part: Div| match framing.before {
        Some(_) => part.ml(sizes.group_gap.px()),
        None => part,
    };
    let frame = |part: Div| part.h(sizes.group_height.px()).border_t(border).border_b(border).border_color(rgb(colour));
    match framing.part {
        FramePart::Unframed if framing.before == Some(FramePart::Close) => gapped(part).into_any_element(),
        FramePart::Unframed => part.into_any_element(),
        FramePart::Open => gapped(frame(part)).border_l(border).rounded_l(radius).pl(sizes.group_padding_left.px() - border).into_any_element(),
        FramePart::Inside => frame(part).into_any_element(),
        FramePart::Close => frame(part).border_r(border).rounded_r(radius).pr(sizes.group_padding_right.px() - border).into_any_element(),
    }
}

/// One item of the tab row, drawn: a group's head or a tab; the row scrolls to a tab by its item's index.
fn render_row_item(item: RowItem, view: &GroupView<'_>, workspace: &Workspace, cx: &mut Context<Workspace>) -> Option<AnyElement> {
    match item {
        RowItem::Head(group) => group_head(&view.groups[group], workspace, cx).map(IntoElement::into_any_element),
        RowItem::Tab(group, id) => {
            let tabs = workspace.tabs();
            let (tab, place, fact) = (tabs.get(id)?, *view.places.get(&id)?, *view.facts.get(&id)?);
            let look = view.look(fact, &view.groups[group], workspace);
            let fill = match (view.shown == Some(id), tab_groups::outlines_shown_tab(&view.groups[group])) {
                (true, true) => Fill::Shown,
                (true, false) => Fill::ShownInFrame,
                (false, _) => Fill::Rest,
            };
            let chip = Chip { id, place, tab, fill, temporary: tabs.is_temporary(id), marker: view.drop.marker(place), look };
            Some(render_tab(chip, workspace, cx).into_any_element())
        }
    }
}

/// A group's head: the chevron folds it; then the project's favicon (or initials), name and, folded, its count.
fn group_head(group: &TabGroup, workspace: &Workspace, cx: &mut Context<Workspace>) -> Option<Stateful<Div>> {
    let theme = crate::theme::tabs(cx);
    let sizes = &theme.sizes.tabs;
    let project = group.project.clone()?;
    let name = workspace.project(&project).map_or_else(|| project.to_string(), |found| found.name.clone());
    let toggled = project.clone();
    let chevron = if group.is_folded() { icons::CHEVRON_RIGHT } else { icons::CHEVRON_DOWN };
    let fold = div()
        .id(SharedString::from(format!("tab-group-fold-{}", project.as_str())))
        .flex_none()
        .p(px(3.))
        .rounded(sizes.close_radius.px())
        .cursor_pointer()
        .hover(|style| style.bg(rgb(theme.hover)))
        .child(img(icons::embedded(theme.appearance, chevron)).size(px(12.)))
        .on_click(target(format!("tab-group-fold:{}", project.as_str()), cx.listener(move |workspace, _event, _window, cx| workspace.toggle_tab_group(&toggled, cx)), cx));
    let letters = crate::sidebar::rows::initials(&name);
    let initials = move || div().px(px(2.)).rounded(px(3.)).bg(rgb(theme.hover)).text_size(px(8.)).child(letters.clone()).into_any_element();
    let icon = match workspace.project_icon(&project) {
        Some(path) => favicon_or(&path, px(14.), initials).into_any_element(),
        None => initials(),
    };
    let count = group.is_folded().then(|| div().px(px(5.)).rounded(sizes.group_count_radius.px()).bg(rgb(theme.hover)).text_size(px(11.)).child(group.tabs.len().to_string()));
    let label = div().max_w(px(140.)).overflow_hidden().text_ellipsis().whitespace_nowrap().child(name);
    let opened = project.clone();
    let row_height = sizes.row_height.px();
    let open = cx.listener(move |workspace, event: &gpui::ClickEvent, _window, cx| {
        workspace.open_group_menu(opened.clone(), gpui::point(event.position().x, row_height), cx);
    });
    let named = div()
        .id(SharedString::from(format!("tab-group-name-{}", project.as_str())))
        .flex()
        .items_center()
        .gap(px(5.))
        .h(sizes.group_head_height.px())
        .pl(px(3.))
        .pr(px(6.))
        .rounded(px(6.))
        .cursor_pointer()
        .hover(|style| style.bg(rgb(theme.hover)).text_color(rgb(theme.text)))
        .child(icon)
        .child(label)
        .children(count)
        .on_click(target(format!("tab-group-menu:{}", project.as_str()), open, cx));
    Some(
        div()
            .id(SharedString::from(format!("tab-group-{}", project.as_str())))
            .flex()
            .flex_none()
            .items_center()
            .text_size(sizes.group_head_text_size.px())
            .text_color(rgb(theme.muted))
            // a press on the head must not start a window drag
            .on_mouse_down(MouseButton::Left, |_event, _window, cx| cx.stop_propagation())
            .child(fold)
            .child(named),
    )
}

impl GroupView<'_> {
    /// Whether the item's group holds the shown tab.
    fn holds_shown(&self, item: RowItem) -> bool {
        let (RowItem::Head(group) | RowItem::Tab(group, _)) = item;
        self.shown.is_some_and(|shown| self.groups[group].tabs.contains(&shown))
    }

    /// How a tab shows where it belongs, from its facts and its group: the main checkout has no colour, its label
    /// is muted.
    fn look(&self, fact: &TabFacts, group: &TabGroup, workspace: &Workspace) -> ChipLook {
        let checkout = fact.project.clone().zip(fact.checkout.clone()).map(|(project, path)| CheckoutRef { project, path });
        let colour = checkout.as_ref().filter(|_| !fact.main).and_then(|checkout| workspace.checkout(checkout)).map(|found| checkout_colour(found, self.theme));
        let cue = match (tab_groups::chip_cue(fact, group, self.spread), checkout.as_ref()) {
            (CueKind::Tint, _) => colour.map_or(ChipCue::Plain, ChipCue::Tint),
            (CueKind::Label, Some(checkout)) => ChipCue::Label { text: workspace.checkout_label(checkout), colour: colour.unwrap_or(self.theme.muted) },
            (CueKind::Label | CueKind::Plain, _) => ChipCue::Plain,
        };
        let badge = fact.icon.clone().filter(|_| tab_groups::shows_badge(fact, group, self.spread));
        ChipLook { cue, badge }
    }
}

/// A tab's worktree cue as drawn: its worktree's colour as background, or its worktree's name after its title.
enum ChipCue {
    Plain,
    Tint(u32),
    Label { text: String, colour: u32 },
}

/// How a tab shows where it belongs: its worktree's cue, and its owner's favicon for the badge.
struct ChipLook {
    cue: ChipCue,
    badge: Option<PathBuf>,
}

/// A tab of the tab row: its place in the row, how it is filled, the side a dragged tab would land on, and how it
/// shows where it belongs.
struct Chip<'a> {
    id: TabId,
    place: usize,
    tab: &'a Tab,
    fill: Fill,
    temporary: bool,
    marker: Option<Side>,
    look: ChipLook,
}

/// How a chip is filled: the shown one filled and outlined, or only filled in a folded group's frame, which outlines
/// it already; the others not.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Fill {
    Rest,
    Shown,
    ShownInFrame,
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
    let Chip { id, place, tab, fill, temporary, marker, look } = chip;
    let shown = fill != Fill::Rest;
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
        // the attention dot on the icon's top right, the owner's badge on its lower right
        .child(badged(tab.marked_icon(workspace, theme, chip_ring(shown, theme)), look.badge.map(|path| Badge { path, ring: chip_ring(shown, theme) }), theme))
        .children(tab.progress(workspace).and_then(|progress| progress_bar(progress, theme)))
        .child(title)
        .children(worktree_label(&look.cue, theme))
        .child(close)
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
    let chip = mouse_binding::on_mouse_triggers(above_title_bar(chip), format!("tab:{}", id.value()), move |trigger, window, cx| {
        // the workspace is gone only while the window closes
        let _ = handle.update(cx, |workspace, cx| workspace.run_tab_mouse(id, trigger, window, cx));
    });
    filled(chip, fill, &look.cue, theme)
}

/// A chip's fill and outline: the shown one filled and outlined (not in a folded group's frame), a worktree's
/// tinted with its colour.
fn filled(chip: Stateful<Div>, fill: Fill, cue: &ChipCue, theme: Theme) -> Stateful<Div> {
    let sizes = &theme.sizes.tabs;
    let outline = |colour: gpui::Hsla| if fill == Fill::Shown { colour } else { transparent_black() };
    match (fill, cue) {
        (Fill::Shown | Fill::ShownInFrame, ChipCue::Tint(colour)) => {
            chip.bg(rgb(mixed(theme.tab_active, *colour, sizes.tint_shown.0))).border_color(outline(tinted(*colour, sizes.tint_outline.0).into()))
        }
        (Fill::Shown | Fill::ShownInFrame, _) => chip.bg(rgb(theme.tab_active)).border_color(outline(rgb(theme.tab_active_border).into())),
        (Fill::Rest, ChipCue::Tint(colour)) => {
            let hover = tinted(*colour, sizes.tint_hover.0);
            chip.bg(tinted(*colour, sizes.tint.0)).border_color(transparent_black()).hover(move |style| style.bg(hover))
        }
        (Fill::Rest, _) => chip.border_color(transparent_black()).hover(|style| style.bg(rgb(theme.hover))),
    }
}

/// The ring cutting a badge out of its icon: the shown chip's fill, else the tab row's.
fn chip_ring(shown: bool, theme: Theme) -> u32 {
    if shown { theme.tab_active } else { theme.sidebar }
}

/// A folded group's tab's worktree name, after its title.
fn worktree_label(cue: &ChipCue, theme: Theme) -> Option<Div> {
    let ChipCue::Label { text, colour } = cue else {
        return None;
    };
    Some(div().flex_none().max_w(px(120.)).overflow_hidden().text_ellipsis().whitespace_nowrap().text_size(theme.sizes.tabs.group_head_text_size.px()).text_color(rgb(*colour)).child(text.clone()))
}

/// How a temporary tab's title shows it; italic until the scripts have loaded.
fn temporary_marker(cx: &App) -> TabMarker {
    cx.try_global::<TemporaryTabs>().map_or(TabMarker::Italic, |tabs| tabs.marker.clone())
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
        .map(above_title_bar)
        .p(theme.sizes.tabs.toolbar_button_padding.px())
        .rounded(theme.sizes.tabs.toolbar_button_radius.px())
        .cursor_pointer()
        .hover(|style| style.bg(rgb(theme.hover)))
        .child(img(icons::embedded(theme.appearance, icons::RESTART)).size(theme.sizes.tabs.toolbar_icon_size.px()))
        .tooltip(tooltip::text_tooltip("Restart IVE"))
        .on_mouse_down(MouseButton::Left, |_event, _window, cx| cx.stop_propagation())
        .on_click(target("restart".to_owned(), cx.listener(|workspace, _event, window, cx| workspace.restart(window, cx)), cx))
}

/// Shown while a release is staged: restarts onto it now instead of on quit or when idle.
fn update_button(tag: String, theme: Theme, cx: &mut Context<Workspace>) -> Stateful<Div> {
    div()
        .id("update-restart")
        .flex()
        .items_center()
        .gap(theme.sizes.tabs.toolbar_button_padding.px())
        .p(theme.sizes.tabs.toolbar_button_padding.px())
        .rounded(theme.sizes.tabs.toolbar_button_radius.px())
        .cursor_pointer()
        .text_size(theme.sizes.tabs.tab_text_size.px())
        .text_color(rgb(theme.accent))
        .hover(|style| style.bg(rgb(theme.hover)))
        .child(img(icons::embedded(theme.appearance, icons::RESTART)).size(theme.sizes.tabs.toolbar_icon_size.px()))
        .child("Restart to update")
        .tooltip(tooltip::text_tooltip(format!("Restart IVE onto {tag}; terminals keep running")))
        .on_mouse_down(MouseButton::Left, |_event, _window, cx| cx.stop_propagation())
        .on_click(target("update-restart".to_owned(), cx.listener(|workspace, _event, window, cx| workspace.restart_to_update(window, cx)), cx))
}

fn settings_button(theme: Theme, cx: &mut Context<Workspace>) -> Stateful<Div> {
    div()
        .id("open-settings")
        .map(above_title_bar)
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
        let failed = Tab::Failed { title: "a.rs".to_owned(), failure: Failure::new("gone") };

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
