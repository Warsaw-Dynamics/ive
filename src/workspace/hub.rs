//! The hub tab in the workspace: opening it, its keys, what it lists, and the searches over the file index.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use futures::StreamExt;
use gpui::{AnyElement, App, Context, Keystroke, ScrollHandle, Window, prelude::*};

use super::Workspace;
use crate::editor::{Tab, saved_icon};
use crate::hub::history::{History, Shown, TabIdentity};
use crate::hub::matcher::Query;
use crate::hub::model::{Effect, HubState, Mode, mode};
use crate::hub::content::{self, ContentFound, ContentQuery, Ignored, Picked, Root, Search};
use crate::hub::rows::{List, Located, content_list, list, thousands};
use crate::hub::search::{Found, search};
use crate::hub::sources::{BackTab, FreeBranch, HubAction, HubScope, Item, Place, ProjectInfo, Sources, StateIn, Target, WorktreeInfo};
use codeview::diff::Headers;
use codeview::panel::Cast;
use crate::codeview_host::{DiffOrigin, diff_panel, serve, serve_texts};
use crate::git::diff::{FileDiff, FilePaths, Sides};
use crate::git::refs::BranchName;
use crate::git::repo::GitError;
use crate::git::types::GitPath;
use crate::git::worktree_add::{add_worktree, worktree_dir};
use crate::workspace::projects::AfterOpen;
use crate::hub::preview::{PROCESS_PREVIEW_LINES, read_text};
use crate::hub::view::{Drawn, HubTab, Preview, escape_does, render_hub};
use crate::run::file::ConfigKind;
use crate::hub::{corpus::Corpus, key_label};
use crate::icons;
use crate::overview::{ProjectSection, project_card_tags};
use crate::process::ProcessId;
use crate::project::{CheckoutRef, ProjectId, ProjectKind};
use crate::scratch::search::{Generations, Ticket};
use crate::script::excluded::Excluded;
use crate::script::keymap::{Action, Builtin, Place as KeyPlace, Scope};
use crate::script::state::{invoke, keymap};
use crate::settings::ThemePreference;
use super::processes::ProgramEnding;
use crate::run::values_tab::{RunRequest, ShowRun};
use crate::saved_tabs::SavedTab;
use crate::scratch::{self, ScratchRef};
use crate::sidebar::rows::RunsView;
use crate::tabs::TabId;
use crate::text_input::{self, TextInput, TextInputEvent};

const QUERY_TARGET: &str = "hub-query";
const PLACEHOLDER: &str = "Files, or > actions, @ processes, # worktrees, : scope";
const NEW_TERMINAL_PLACEHOLDER: &str = "New terminal in… type a project, worktree or branch";

/// What the selected row's preview needs.
enum Wanted {
    Ready(Preview),
    Text { title: String, path: PathBuf, line: Option<usize> },
    Diff { title: String, dir: PathBuf, path: String },
}

impl Wanted {
    /// The same key, the same preview: it is not read again while the selection stays.
    fn key(&self) -> String {
        match self {
            Self::Ready(_) => "ready".to_owned(),
            Self::Text { path, line, .. } => format!("text:{}:{line:?}", path.display()),
            Self::Diff { dir, path, .. } => format!("diff:{}:{path}", dir.display()),
        }
    }
}

/// A preview read off the UI thread.
enum Loaded {
    Text { title: String, first: usize, lines: Vec<String>, line: Option<usize> },
    Diff { title: String, dir: PathBuf, diff: FileDiff },
    Message(String),
}

/// The line a files query or `:42` names.
fn hub_line(state: &HubState) -> Option<usize> {
    let line = match mode(state) {
        Mode::Files { line } => line,
        Mode::Line(line) => Some(line),
        _ => None,
    };
    line.and_then(|line| usize::try_from(line).ok())
}

/// The tab just opened from the hub, and the hub as it was: `opened_from_hub` until another tab is shown.
pub(super) struct HubReturn {
    pub(super) tab: TabId,
    pub(super) at: Instant,
    pub(super) state: HubState,
}

/// What a key pressed in the hub's query does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum KeyUse {
    Hub(Builtin),
    Invoke { generation: u64, function: usize },
    /// to the query field, then the window's bindings
    Pass,
}

/// The hub takes its own actions, except keys the field keeps (Backspace only while nothing is typed).
pub(super) fn hub_key_use(action: Option<&Action>, keystroke: &Keystroke, typed_nothing: bool) -> KeyUse {
    match action {
        Some(Action::Builtin(builtin)) if builtin.is_hub_command() => {
            let widens = *builtin == Builtin::HubWiden && typed_nothing;
            if text_input::keeps_key(keystroke) && !widens { KeyUse::Pass } else { KeyUse::Hub(*builtin) }
        }
        // a function bound in the hub scope, or a window one: the field keeps its typing keys
        Some(Action::Invoke { generation, function }) if !text_input::keeps_key(keystroke) => KeyUse::Invoke { generation: *generation, function: *function },
        _ => KeyUse::Pass,
    }
}

/// What `Effect::Open` asks for.
struct OpenRow {
    target: Target,
    checkout: Option<CheckoutRef>,
    line: Option<u32>,
    behind: bool,
}

/// A Recent row's tab now: open, closed but reopenable, its process still running without a tab, or gone.
/// An open tab and a running process carry their title now, which a terminal's program may have changed since.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Presence {
    Open { tab: TabId, title: String },
    Closed,
    Process { id: ProcessId, title: String },
    Gone,
}

impl Workspace {
    pub(super) fn hub_tab_id(&self) -> Option<TabId> {
        self.tabs.find(Tab::is_hub)
    }

    /// Ctrl+Shift+E: shows the hub, the one there is or a new one, its query given the keyboard; `state` as
    /// `ive::open_hub` restores it.
    pub fn open_hub(&mut self, state: Option<HubState>, window: &mut Window, cx: &mut Context<Self>) -> TabId {
        self.index_due_checkouts(cx);
        if let Some(id) = self.hub_tab_id() {
            // a restored hub loads as it is shown
            self.activate_tab(id, window, cx);
            let fitted = state.map(|state| state.fitted(&self.hub_sources(None, cx)));
            if let Some(Tab::Hub(hub)) = self.tabs.get_mut(id) {
                match fitted {
                    Some(fitted) => hub.state = fitted,
                    None => hub.state.show_as_hub(),
                }
                if self.worktree_creations.is_empty() {
                    hub.notice = None;
                }
                hub.sync_input(cx);
            }
            self.start_hub_search(id, cx);
            self.settle_hub(id, cx);
            return id;
        }
        // a hub coming back opens right of the tab it came from, in the slot of the tab just closed
        let returned = state.as_ref().and_then(|state| state.back).filter(|back| self.tabs.get(*back).is_some() && !self.tabs.is_detached(*back));
        if let Some(back) = returned {
            self.tabs.activate(back);
        }
        let back = self.tabs.active_id();
        let fresh = || HubState { scope: self.hub_scope.clone(), ..HubState::default() };
        let state = state.unwrap_or_else(fresh).fitted(&self.hub_sources(back, cx));
        let hub = self.new_hub(state, back, window, cx);
        let id = self.open_tab(Tab::Hub(Box::new(hub)), window, cx);
        self.start_hub_search(id, cx);
        self.settle_hub(id, cx);
        id
    }

    /// Ctrl+Shift+T: the hub asking where a new terminal goes.
    pub fn open_terminal_picker(&mut self, window: &mut Window, cx: &mut Context<Self>) -> TabId {
        let state = HubState { new_terminal: true, scope: self.hub_scope.clone(), ..HubState::default() };
        self.open_hub(Some(state), window, cx)
    }

    pub(super) fn new_hub(&mut self, state: HubState, back: Option<TabId>, window: &mut Window, cx: &mut Context<Self>) -> HubTab {
        let query = state.pick.as_ref().map_or(&state.query, |pick| &pick.query).clone();
        let input = cx.new(|cx| {
            let mut input = TextInput::new(cx);
            input.set_target(QUERY_TARGET);
            input.placeholder(PLACEHOLDER);
            input.set_text(query, cx);
            input
        });
        let events = cx.subscribe_in(&input, window, |workspace, _input, event: &TextInputEvent, window, cx| {
            if let (TextInputEvent::Changed(text), Some(id)) = (event, workspace.hub_tab_id()) {
                workspace.hub_typed(id, text.clone(), window, cx);
            }
        });
        HubTab {
            state,
            back,
            input,
            found: None,
            content: None,
            searched: self.corpus.clone(),
            search: None,
            generations: Generations::default(),
            list_scroll: ScrollHandle::new(),
            preview: Preview::None,
            preview_key: None,
            preview_task: None,
            notice: None,
            _input_events: events,
        }
    }

    /// A key while the hub's query has the keyboard, before the field sees it: the hub's own bindings, except the
    /// keys the field keeps (Backspace only on a non-empty query). True when the hub took it.
    pub(super) fn hub_keystroke(&mut self, keystroke: &Keystroke, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let Some((id, Tab::Hub(hub))) = self.tabs.active_id().and_then(|id| Some((id, self.tabs.get(id)?))) else {
            return false;
        };
        let typed_nothing = hub.state.query.is_empty() && hub.state.pick.as_ref().is_none_or(|pick| pick.query.is_empty());
        let action = keymap(cx).and_then(|keymap| keymap.find_key(keystroke, KeyPlace::Hub)).cloned();
        match hub_key_use(action.as_ref(), keystroke, typed_nothing) {
            KeyUse::Hub(builtin) => self.hub_action(id, builtin, window, cx),
            KeyUse::Invoke { generation, function } => invoke(generation, function, None, cx),
            KeyUse::Pass => return false,
        }
        true
    }

    pub fn hub_action(&mut self, id: TabId, action: Builtin, window: &mut Window, cx: &mut Context<Self>) {
        let Some(Tab::Hub(hub)) = self.tabs.get(id) else {
            return;
        };
        let sources = self.hub_sources(hub.back, cx);
        let list = self.hub_list(hub, &sources);
        let Some(Tab::Hub(hub)) = self.tabs.get_mut(id) else {
            return;
        };
        let effect = hub.state.act(action, &list, &sources);
        if hub.state.excluded {
            self.index_ignored_files(cx);
        }
        self.apply_hub_effect(id, effect, window, cx);
    }

    pub fn hub_typed(&mut self, id: TabId, text: String, window: &mut Window, cx: &mut Context<Self>) {
        let Some(Tab::Hub(hub)) = self.tabs.get_mut(id) else {
            return;
        };
        if self.worktree_creations.is_empty() {
            hub.notice = None;
        }
        let effect = hub.state.typed(text);
        self.apply_hub_effect(id, effect, window, cx);
    }

    /// A click on row `index`: as Enter on it.
    pub fn hub_open_row(&mut self, id: TabId, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(Tab::Hub(hub)) = self.tabs.get_mut(id) {
            match &mut hub.state.pick {
                Some(pick) => pick.selected = index,
                None => hub.state.selected = index,
            }
        }
        self.hub_action(id, Builtin::HubOpen, window, cx);
    }

    /// A click on a scope chip.
    pub fn hub_pick_scope(&mut self, id: TabId, scope: HubScope, window: &mut Window, cx: &mut Context<Self>) {
        let Some(Tab::Hub(hub)) = self.tabs.get(id) else {
            return;
        };
        let sources = self.hub_sources(hub.back, cx);
        let Some(Tab::Hub(hub)) = self.tabs.get_mut(id) else {
            return;
        };
        let effect = hub.state.pick_scope(scope, &sources);
        self.apply_hub_effect(id, effect, window, cx);
    }

    fn apply_hub_effect(&mut self, id: TabId, effect: Effect, window: &mut Window, cx: &mut Context<Self>) {
        match effect {
            Effect::Nothing => return,
            Effect::Redraw => {}
            Effect::Search => self.start_hub_search(id, cx),
            Effect::Leave => return self.leave_hub(id, window, cx),
            Effect::Open { target: Target::NewWorktreeTerminal { project, branch, remote }, .. } => return self.add_worktree_from_hub(id, project, branch, remote, window, cx),
            Effect::Open { target, checkout, line, behind } => return self.open_from_hub(id, OpenRow { target, checkout, line, behind }, window, cx),
            Effect::TypePath(path) => self.type_path_into_terminal(&path),
            Effect::CopyPath(path) => cx.write_to_clipboard(gpui::ClipboardItem::new_string(path)),
            Effect::Stop(target) => self.stop_from_hub(&target, window, cx),
        }
        self.settle_hub(id, cx);
    }

    /// Stop on a row: its process ends as a process tab's Stop ends it; the hub stays, the row reading "stopping".
    fn stop_from_hub(&mut self, target: &Target, window: &mut Window, cx: &mut Context<Self>) {
        let process = match target {
            Target::Process(id) => Some(*id),
            Target::Tab(tab) => match self.tabs.get(*tab) {
                Some(Tab::Process(id)) => Some(*id),
                _ => None,
            },
            _ => None,
        };
        if let Some(id) = process {
            self.stop_process(id, ProgramEnding::Stop, window, cx);
        }
    }

    /// After a process ended: its row left the hub's list, so the selection and the preview follow the rows that stay.
    pub(super) fn settle_open_hub(&mut self, cx: &mut Context<Self>) {
        if let Some(id) = self.hub_tab_id() {
            self.settle_hub(id, cx);
        }
    }

    /// After a change: the field shows the state's text, the selection fits the rows and is in view, and the
    /// scope is remembered for the next Ctrl+Shift+E.
    fn settle_hub(&mut self, id: TabId, cx: &mut Context<Self>) {
        let Some(Tab::Hub(hub)) = self.tabs.get(id) else {
            return;
        };
        let sources = self.hub_sources(hub.back, cx);
        let list = self.hub_list(hub, &sources);
        let Some(Tab::Hub(hub)) = self.tabs.get_mut(id) else {
            return;
        };
        hub.state.fit_selection(&list);
        hub.sync_input(cx);
        let placeholder = if hub.state.new_terminal { NEW_TERMINAL_PLACEHOLDER } else { PLACEHOLDER };
        hub.input.update(cx, |input, _cx| input.placeholder(placeholder));
        let selected = hub.state.pick.as_ref().map_or(hub.state.selected, |pick| pick.selected);
        hub.list_scroll.scroll_to_item(child_index(&list, selected));
        self.hub_scope = hub.state.scope.clone();
        self.show_selected_preview(id, &list, &sources, cx);
        cx.notify();
    }

    /// A process changed its title: the shown hub's preview of it is drawn again under the new one. Only the
    /// preview, so the list keeps where the user scrolled it.
    pub(super) fn retitle_hub_preview(&mut self, cx: &mut Context<Self>) {
        let Some(Tab::Hub(hub)) = self.tabs.active() else {
            return;
        };
        let Some(id) = self.hub_tab_id() else {
            return;
        };
        let sources = self.hub_sources(hub.back, cx);
        let list = self.hub_list(hub, &sources);
        self.show_selected_preview(id, &list, &sources, cx);
    }

    fn show_selected_preview(&mut self, id: TabId, list: &List, sources: &Sources, cx: &mut Context<Self>) {
        let Some(Tab::Hub(hub)) = self.tabs.get(id) else {
            return;
        };
        let selected = hub.state.pick.as_ref().map_or(hub.state.selected, |pick| pick.selected);
        let line = hub_line(&hub.state);
        let wanted = list.rows.get(selected).map(|row| self.preview_of(row, &line, sources));
        self.show_hub_preview(id, wanted, cx);
    }

    /// What the selected row's preview needs: its key, and how to get it.
    fn preview_of(&self, row: &crate::hub::rows::Row, line: &Option<usize>, sources: &Sources) -> Wanted {
        let item = &row.item;
        let in_checkout = |relative: &str| -> Option<PathBuf> {
            let checkout = match &item.place {
                Place::One(checkout) => checkout.path.clone(),
                Place::Several { project, checkouts } => sources.by_recency(project, Some(checkouts)).first()?.checkout.path.clone(),
                Place::Nowhere => return None,
            };
            Some(checkout.join(relative))
        };
        let text = |path: PathBuf, line: Option<usize>| Wanted::Text { title: path.display().to_string(), path, line };
        let message = |message: String| Wanted::Ready(Preview::Message(message));
        match &item.target {
            Target::File { path, .. } => in_checkout(path).map_or_else(|| message(item.title.clone()), |path| text(path, *line)),
            Target::Reopen(SavedTab::File { path, .. }) => text(path.clone(), *line),
            Target::Tab(id) => match self.tabs.get(*id) {
                Some(Tab::Document(file)) => text(file.path().to_path_buf(), *line),
                Some(Tab::Process(process)) => self.process_preview(*process),
                _ => message(item.title.clone()),
            },
            Target::Scratch(scratch) => match scratch::store_of(&scratch.project) {
                Ok(store) => text(store.path(&scratch.name), None),
                Err(error) => message(error),
            },
            Target::Change { checkout, path } => Wanted::Diff { title: format!("{path} · uncommitted"), dir: checkout.path.clone(), path: path.clone() },
            Target::Process(process) => self.process_preview(*process),
            Target::RunConfig { folder, key, .. } => self.config_preview(item, folder, key, sources),
            Target::Worktree(checkout) => self.worktree_preview(checkout, sources),
            Target::Line(number) => match sources.back.as_ref().and_then(|back| self.tabs.get(back.tab)) {
                Some(Tab::Document(file)) => text(file.path().to_path_buf(), usize::try_from(*number).ok()),
                _ => message(crate::hub::rows::LINE_NEEDS_TEXT.to_owned()),
            },
            Target::Reopen(saved) => message(format!("{} · Enter opens it again", saved.title())),
            Target::Action(_) | Target::Scope(_) | Target::Branch { .. } => message(format!("{} · {}", item.title, item.detail)),
            Target::NewTerminal(checkout) => self.new_terminal_preview(checkout),
            Target::NewWorktreeTerminal { project, branch, remote } => self.new_worktree_preview(project, branch, remote.as_deref()),
            Target::Hit { path, line, .. } => text(path.clone(), usize::try_from(*line).ok()),
        }
    }

    fn process_preview(&self, id: ProcessId) -> Wanted {
        match self.processes.get(id) {
            Some(process) => Wanted::Ready(Preview::Lines { title: process.title(), lines: process.view.output_text(Some(PROCESS_PREVIEW_LINES)), mono: true }),
            None => Wanted::Ready(Preview::Message("This process has ended".to_owned())),
        }
    }

    /// A configuration's live run's last lines there, else its command.
    fn config_preview(&self, item: &Item, folder: &str, key: &str, sources: &Sources) -> Wanted {
        let checkout = match &item.place {
            Place::One(checkout) => Some(checkout.path.clone()),
            Place::Several { project, checkouts } => sources.by_recency(project, Some(checkouts)).first().map(|worktree| worktree.checkout.path.clone()),
            Place::Nowhere => None,
        };
        let Some(checkout) = checkout else {
            return Wanted::Ready(Preview::Message(item.title.clone()));
        };
        let runs = self.processes.runs_in(&checkout, &checkout.join(folder));
        if let Some(run) = runs.into_iter().find(|process| process.is_live() && process.run.as_ref().is_some_and(|origin| origin.key == key)) {
            return self.process_preview(run.id);
        }
        let file = self.hub_index.files(&checkout).and_then(|read| read.configs.iter().find(|(at, _)| at == folder).map(|(_, file)| file.clone()));
        let command = file.and_then(|file| file.configs.iter().find(|config| config.key == key).map(|config| match &config.kind {
            ConfigKind::Run(spec) => spec.command.clone(),
            ConfigKind::Serve(_) => "a static server (serve)".to_owned(),
        }));
        Wanted::Ready(Preview::Lines { title: format!("{folder}/.run.toml · {key}"), lines: command.into_iter().collect(), mono: true })
    }

    /// What runs in the checkout a new terminal would join.
    fn new_terminal_preview(&self, checkout: &CheckoutRef) -> Wanted {
        let running: Vec<String> = self.processes.by_checkout(&checkout.path).iter().map(|process| format!("  {}", process.title())).collect();
        let lines = match running.is_empty() {
            true => vec!["Nothing running there yet.".to_owned()],
            false => std::iter::once("Running there:".to_owned()).chain(running).collect(),
        };
        let title = format!("New terminal in {} · {}", self.checkout_label(checkout), checkout.path.display());
        Wanted::Ready(Preview::Lines { title, lines, mono: false })
    }

    /// What Enter on a branch row does, and where its worktree's folder comes from.
    fn new_worktree_preview(&self, project: &ProjectId, branch: &str, remote: Option<&str>) -> Wanted {
        let Some(main_dir) = self.project(project).map(|project| project.main_checkout().path.clone()) else {
            return Wanted::Ready(Preview::Message("This project is no longer open".to_owned()));
        };
        let folder = match crate::ive_toml::worktrees_folder(&main_dir) {
            Ok(folder) => folder,
            Err(error) => return Wanted::Ready(Preview::Message(error)),
        };
        let name = BranchName::Local(branch.to_owned());
        let dir = worktree_dir(&main_dir, &folder, &name);
        let from = match folder.as_os_str() == crate::ive_toml::DEFAULT_WORKTREES {
            true => format!("(the default {}; set worktrees in {})", crate::ive_toml::DEFAULT_WORKTREES, crate::ive_toml::FILE_NAME),
            false => format!("(worktrees = \"{}\" in {})", folder.display(), crate::ive_toml::FILE_NAME),
        };
        let mut lines = Vec::new();
        if let Some(remote) = remote {
            lines.push(format!("Only on {remote}: first a local {branch} tracking {remote}/{branch}"));
        }
        lines.extend([format!("A worktree in {}", dir.display()), from, "then a terminal there".to_owned()]);
        Wanted::Ready(Preview::Lines { title: format!("{branch} · no worktree yet"), lines, mono: false })
    }

    /// Enter on a branch row in new terminal mode: its worktree, created off the UI thread while the hub says so,
    /// then the folder opened with a terminal in it; a failure is said in the hub, or in a tab once the hub is gone.
    /// A branch being created already is not created again.
    fn add_worktree_from_hub(&mut self, hub: TabId, project: ProjectId, branch: String, remote: Option<String>, window: &mut Window, cx: &mut Context<Self>) {
        let Some(main_dir) = self.project(&project).map(|open| open.main_checkout().path.clone()) else {
            return;
        };
        if !self.worktree_creations.start(&project, &branch) {
            return;
        }
        if let Some(Tab::Hub(open)) = self.tabs.get_mut(hub) {
            open.notice = Some(format!("Creating a worktree of {branch}…"));
        }
        let name = match remote {
            Some(remote) => BranchName::Remote { remote, branch: branch.clone() },
            None => BranchName::Local(branch.clone()),
        };
        cx.spawn_in(window, async move |this, cx| {
            let result = cx.background_executor().spawn(async move { add_worktree(&main_dir, &name) }).await;
            this.update_in(cx, |workspace, window, cx| workspace.hub_worktree_added(&project, &branch, result, window, cx)).ok();
        })
        .detach();
        self.settle_hub(hub, cx);
    }

    fn hub_worktree_added(&mut self, project: &ProjectId, branch: &str, result: Result<PathBuf, GitError>, window: &mut Window, cx: &mut Context<Self>) {
        self.worktree_creations.finish(project, branch);
        let notice = match result {
            Ok(path) => {
                self.open_folder(path, AfterOpen::NewTerminal, window, cx);
                self.refresh_project(project, cx);
                None
            }
            Err(error) => {
                eprintln!("ive: {error}");
                Some(format!("Cannot create a worktree of {branch}: {}", error.message))
            }
        };
        // the hub there is now: the one that asked may be closed, or opened again
        let Some(hub) = self.hub_tab_id() else {
            if let Some(error) = notice {
                self.open_tab(Tab::Failed { title: "New worktree".to_owned(), error: error.into() }, window, cx);
            }
            return;
        };
        if let Some(Tab::Hub(open)) = self.tabs.get_mut(hub) {
            open.notice = notice;
        }
        self.settle_hub(hub, cx);
    }

    fn worktree_preview(&self, checkout: &CheckoutRef, sources: &Sources) -> Wanted {
        let Some(worktree) = sources.worktree(checkout) else {
            return Wanted::Ready(Preview::Message("This worktree is no longer open".to_owned()));
        };
        let mut lines = vec![checkout.path.display().to_string()];
        lines.extend(worktree.branch.clone().map(|branch| format!("branch {branch}")));
        lines.extend(worktree.tags.iter().cloned());
        Wanted::Ready(Preview::Lines { title: worktree.label.clone(), lines, mono: false })
    }

    /// Shows `wanted` in the preview, reading a file or a diff off the UI thread; the same row again keeps it.
    fn show_hub_preview(&mut self, id: TabId, wanted: Option<Wanted>, cx: &mut Context<Self>) {
        let key = wanted.as_ref().map(Wanted::key);
        let Some(Tab::Hub(hub)) = self.tabs.get_mut(id) else {
            return;
        };
        if key.is_some() && hub.preview_key == key && !matches!(wanted, Some(Wanted::Ready(_))) {
            return;
        }
        hub.preview_key = key;
        let Some(wanted) = wanted else {
            hub.preview = Preview::None;
            hub.preview_task = None;
            return;
        };
        let reading = match wanted {
            Wanted::Ready(preview) => {
                hub.preview = preview;
                hub.preview_task = None;
                return;
            }
            Wanted::Text { title, path, line } => cx.background_executor().spawn(async move {
                match read_text(&path, line.and_then(|line| u32::try_from(line).ok())) {
                    Ok((first, lines)) => Loaded::Text { title, first, lines, line },
                    Err(error) => Loaded::Message(error),
                }
            }),
            Wanted::Diff { title, dir, path } => cx.background_executor().spawn(async move {
                let file = FilePaths { path: GitPath::from_bytes(path.as_bytes()), old_path: None };
                match crate::git::diff::uncommitted(&dir, &file) {
                    Ok(Some(diff)) => Loaded::Diff { title, dir, diff },
                    Ok(None) => Loaded::Message("No changes any more".to_owned()),
                    Err(error) => Loaded::Message(error.to_string()),
                }
            }),
        };
        hub.preview = Preview::Loading;
        let key = hub.preview_key.clone();
        hub.preview_task = Some(cx.spawn(async move |this, cx| {
            let loaded = reading.await;
            this.update(cx, |workspace, cx| workspace.hub_preview_loaded(id, key, loaded, cx)).ok();
        }));
    }

    fn hub_preview_loaded(&mut self, id: TabId, key: Option<String>, loaded: Loaded, cx: &mut Context<Self>) {
        let preview = match loaded {
            Loaded::Text { title, first, lines, line } => Preview::Text { title, first, lines, line },
            Loaded::Message(message) => Preview::Message(message),
            Loaded::Diff { title, dir, diff } => {
                let origin = DiffOrigin { dir, sides: Sides::Uncommitted };
                let name = diff.path.display();
                let panel = diff_panel(vec![diff], Headers::Hidden, cx);
                serve(&panel, name, cx);
                serve_texts(&panel, origin, cx);
                Preview::Diff { title, panel }
            }
        };
        if let Some(Tab::Hub(hub)) = self.tabs.get_mut(id)
            && hub.preview_key == key
        {
            hub.preview = preview;
            cx.notify();
        }
    }

    /// Escape on an empty query: the tab shown before, a temporary hub going as it is shown; without one, the hub
    /// closes.
    fn leave_hub(&mut self, id: TabId, window: &mut Window, cx: &mut Context<Self>) {
        let back = match self.tabs.get(id) {
            Some(Tab::Hub(hub)) => hub.back.filter(|back| *back != id && self.tabs.get(*back).is_some()),
            _ => None,
        };
        match back {
            Some(back) => self.activate_tab(back, window, cx),
            None => self.close_tab(id, window, cx),
        }
    }

    /// Runs the files search for the hub's query, its best rows arriving as each chunk is matched; a newer query
    /// stops it.
    pub(super) fn start_hub_search(&mut self, id: TabId, cx: &mut Context<Self>) {
        let Some(Tab::Hub(hub)) = self.tabs.get(id) else {
            return;
        };
        let filter = hub.state.filter(&self.hub_sources(hub.back, cx));
        let corpus = self.corpus.clone();
        let Some(Tab::Hub(hub)) = self.tabs.get_mut(id) else {
            return;
        };
        let ticket = hub.generations.next();
        if let Some(query) = hub.state.content_query() {
            let roots = self.content_roots(id);
            return self.start_content_search(id, ContentSearch { roots, query, ticket }, cx);
        }
        let Some(Tab::Hub(hub)) = self.tabs.get_mut(id) else {
            return;
        };
        hub.content = None;
        let Some(text) = hub.state.search_text() else {
            hub.found = None;
            hub.search = None;
            return cx.notify();
        };
        let query = Query::parse(text);
        let (sender, mut found) = futures::channel::mpsc::unbounded::<Found>();
        let scanned = (corpus.clone(), ticket.clone());
        let scan = cx.background_executor().spawn(async move {
            let (corpus, ticket) = scanned;
            search(&corpus, &query, &filter, &ticket, |partial| drop(sender.unbounded_send(partial.clone())));
        });
        hub.search = Some(cx.spawn(async move |this, cx| {
            let _scan = scan;
            while let Some(result) = found.next().await {
                if ticket.is_stale() {
                    return;
                }
                let corpus = corpus.clone();
                if this.update(cx, |workspace, cx| workspace.hub_found(id, result, corpus, cx)).is_err() {
                    return;
                }
            }
        }));
    }

    /// Runs a contents search for the hub, its lines arriving as they are found; a newer query stops it.
    fn start_content_search(&mut self, id: TabId, search: ContentSearch, cx: &mut Context<Self>) {
        let excluded = cx.try_global::<Excluded>().cloned().unwrap_or_default();
        let Some(Tab::Hub(hub)) = self.tabs.get_mut(id) else {
            return;
        };
        hub.found = None;
        // the last query's lines are not this one's: "Searching…" until the first of these arrive
        hub.content = None;
        let ignored = if hub.state.excluded { Ignored::Searched } else { Ignored::Skipped };
        let ContentSearch { roots, query, ticket } = search;
        let (sender, mut found) = futures::channel::mpsc::unbounded::<ContentFound>();
        let scanned = ticket.clone();
        let scan = cx.background_executor().spawn(async move {
            let search = Search { roots: &roots, query: &query, ignored, excluded: &excluded };
            let sending = std::sync::Mutex::new(sender.clone());
            let done = content::search(&search, &scanned, |partial| drop(sending.lock().map(|sender| sender.unbounded_send(partial.clone()))));
            if let Some(done) = done {
                drop(sender.unbounded_send(done));
            }
        });
        hub.search = Some(cx.spawn(async move |this, cx| {
            let _scan = scan;
            while let Some(result) = found.next().await {
                if ticket.is_stale() {
                    return;
                }
                if this.update(cx, |workspace, cx| workspace.hub_content_found(id, result, cx)).is_err() {
                    return;
                }
            }
        }));
        cx.notify();
    }

    fn hub_content_found(&mut self, id: TabId, found: ContentFound, cx: &mut Context<Self>) {
        if let Some(Tab::Hub(hub)) = self.tabs.get_mut(id) {
            hub.content = Some(found);
        }
        self.settle_hub(id, cx);
    }

    /// Where a contents search looks: the files and folders picked in the tree, else the checkouts of the hub's scope
    /// and, for a project or all of them, their scratches.
    fn content_roots(&self, id: TabId) -> Vec<Root> {
        let Some(Tab::Hub(hub)) = self.tabs.get(id) else {
            return Vec::new();
        };
        if !hub.state.paths.is_empty() {
            return hub.state.paths.iter().map(|path| Root { path: path.clone(), picked: Picked::FromTree }).collect();
        }
        let in_scope = |open: &&crate::open_project::OpenProject| match &hub.state.scope {
            HubScope::All => true,
            HubScope::Project(project) => open.project.id == *project,
            HubScope::Worktree(checkout) => open.project.id == checkout.project,
        };
        let mut roots = Vec::new();
        for open in self.projects.iter().filter(in_scope) {
            let checkouts = open.listed_checkouts().filter(|checkout| !matches!(&hub.state.scope, HubScope::Worktree(only) if only.path != checkout.path));
            roots.extend(checkouts.map(|checkout| Root { path: checkout.path.clone(), picked: Picked::Scope }));
            let scratches = (!matches!(hub.state.scope, HubScope::Worktree(_))).then(|| scratch::store_of(&open.project.id).ok()).flatten();
            roots.extend(scratches.map(|store| Root { path: store.dir().to_path_buf(), picked: Picked::Scope }));
        }
        roots
    }

    /// A contents search's file in its checkout, for its row.
    fn locate_hit(&self, path: &Path) -> Located {
        match self.checkout_ref_of(path) {
            Some(checkout) => {
                let relative = path.strip_prefix(&checkout.path).map_or_else(|_| path.display().to_string(), |relative| relative.display().to_string());
                Located { project: Some(checkout.project.clone()), place: Place::One(checkout), path: relative }
            }
            None => Located { place: Place::Nowhere, project: None, path: path.display().to_string() },
        }
    }

    fn hub_found(&mut self, id: TabId, found: Found, corpus: Arc<Corpus>, cx: &mut Context<Self>) {
        if let Some(Tab::Hub(hub)) = self.tabs.get_mut(id) {
            hub.found = Some(found);
            hub.searched = corpus;
        }
        self.settle_hub(id, cx);
    }

    pub(super) fn hub_list(&self, hub: &HubTab, sources: &Sources) -> List {
        let mut list = match mode(&hub.state) {
            Mode::Content => content_list(&hub.state, hub.content.as_ref(), &|path| self.locate_hit(path)),
            _ => list(&hub.state, sources, hub.found.as_ref().map(|found| (&*hub.searched, found)), &WorkspaceStates(self)),
        };
        if hub.notice.is_some() {
            list.message.clone_from(&hub.notice);
        }
        list
    }

    pub(crate) fn render_hub_tab(&self, id: TabId, hub: &HubTab, cx: &mut Context<Self>) -> AnyElement {
        let sources = self.hub_sources(hub.back, cx);
        let list = self.hub_list(hub, &sources);
        let drawn = Drawn { list, strip: self.key_strip(&hub.state, cx), chips: self.scope_chips(&hub.state, &sources), excluded_note: self.excluded_note(&hub.state) };
        render_hub(id, hub, drawn, self, cx)
    }

    fn scope_chips(&self, state: &HubState, sources: &Sources) -> Vec<(String, HubScope)> {
        let label = |scope: &HubScope| match scope {
            HubScope::All if sources.hidden.is_empty() => "All".to_owned(),
            HubScope::All => sources.all_label(),
            HubScope::Project(id) => self.project(id).map_or_else(|| id.to_string(), |project| project.name.clone()),
            HubScope::Worktree(checkout) => self.checkout_label(checkout),
        };
        state.stops(sources).into_iter().map(|scope| (label(&scope), scope)).collect()
    }

    fn excluded_note(&self, state: &HubState) -> Option<String> {
        (!state.excluded && self.corpus.excluded > 0).then(|| format!("{} excluded not searched", thousands(self.corpus.excluded)))
    }

    /// The keys of the hub's state as bound (a key changed in `init.rn` shows as changed); unbound ones are left out.
    fn key_strip(&self, state: &HubState, cx: &App) -> Vec<(String, &'static str)> {
        let Some(keymap) = keymap(cx) else {
            return Vec::new();
        };
        let key = |builtin: Builtin| keymap.keys_for(Scope::Hub, &Action::Builtin(builtin)).first().map(|keystroke| key_label::label(keystroke));
        let moves = key(Builtin::HubUp).zip(key(Builtin::HubDown)).map(|(up, down)| format!("{up}{down}"));
        let mut strip: Vec<(Option<String>, &'static str)> = vec![(moves, "move")];
        if mode(state) == Mode::Content {
            strip.extend([
                (key(Builtin::HubOpen), "open"),
                (key(Builtin::HubOpenBehind), "behind"),
                (key(Builtin::HubTypePath), "to terminal"),
                (key(Builtin::HubCopyPath), "copy path"),
                (key(Builtin::HubToggleRegex), if state.matching.is_regex() { "regex on" } else { "regex" }),
                (key(Builtin::HubToggleWholeWord), if state.matching.is_whole_word() { "words on" } else { "words" }),
                (key(Builtin::HubToggleExcluded), if state.excluded { "excluded on" } else { "excluded" }),
                (key(Builtin::HubToggleContent), "names"),
                (key(Builtin::HubBack), escape_does(state)),
            ]);
        } else if mode(state) == Mode::Pick {
            strip.extend([(key(Builtin::HubOpen), "open there"), (key(Builtin::HubOpenBehind), "behind"), (key(Builtin::HubBack), "back")]);
        } else if mode(state) == Mode::NewTerminal {
            strip.extend([(key(Builtin::HubOpen), "new terminal there"), (key(Builtin::HubBack), escape_does(state)), (key(Builtin::HubToggleChanged), "hub")]);
        } else {
            strip.extend([
                (key(Builtin::HubOpen), "open"),
                (key(Builtin::HubOpenBehind), "behind"),
                (key(Builtin::HubOpenRow(1)).map(|first| format!("{first}…9")), "row"),
                (key(Builtin::HubTypePath), "to terminal"),
                (key(Builtin::HubCopyPath), "copy path"),
                (key(Builtin::CopyFullPath), "full path"),
                (key(Builtin::HubStop), "stop"),
                (key(Builtin::HubCycleScope), "scope"),
                (key(Builtin::HubToggleExcluded), "excluded"),
                (key(Builtin::HubToggleChanged), "changed"),
                (key(Builtin::HubBack), escape_does(state)),
            ]);
        }
        strip.into_iter().filter_map(|(keys, does)| keys.map(|keys| (keys, does))).collect()
    }

    /// Everything the hub lists but the files.
    pub(super) fn hub_sources(&self, back: Option<TabId>, cx: &App) -> Sources {
        let open_checkouts: Vec<PathBuf> = self.projects.iter().flat_map(|open| open.listed_checkouts().map(|checkout| checkout.path.clone())).collect();
        let shown = self.shown_projects();
        let hidden = self.projects.iter().map(|open| open.project.id.clone()).filter(|id| !shown.contains(id)).collect();
        Sources {
            hidden,
            projects: self.hub_projects(),
            recent: recent_items(&self.history, |shown| self.presence(shown), |path| self.checkout_of_path(path)),
            changed: self.changed_items(),
            running: self.process_items(true),
            processes: self.process_items(false),
            configs: self.config_items(),
            branches: self.free_branches(),
            actions: action_items(),
            back: back.and_then(|back| self.back_tab(back, cx)),
            loading: open_checkouts.iter().any(|path| self.hub_index.files(path).is_none()),
        }
        .without_hidden()
    }

    /// Open projects in the corpus' order, which its entries index; projects not in it yet after them.
    fn hub_projects(&self) -> Vec<ProjectInfo> {
        let in_corpus = self.corpus_projects.iter().map(|id| self.project_info(id).unwrap_or_else(|| ProjectInfo { id: id.clone(), name: id.to_string(), git: false, worktrees: Vec::new() }));
        let later = self.projects.iter().filter(|open| !self.corpus_projects.contains(&open.project.id)).filter_map(|open| self.project_info(&open.project.id));
        in_corpus.chain(later).collect()
    }

    fn project_info(&self, id: &ProjectId) -> Option<ProjectInfo> {
        let open = self.open_project(id)?;
        let project = &open.project;
        let worktrees = open
            .listed_checkouts()
            .map(|checkout| {
                let running = self.processes.iter().filter(|process| process.is_live() && process.checkout.as_ref().is_some_and(|at| at.path == checkout.path)).count();
                WorktreeInfo {
                    checkout: project.checkout_ref(checkout),
                    label: checkout.label.clone(),
                    branch: checkout.branch.clone(),
                    tags: project_card_tags(checkout, project.kind, running),
                    last_used: self.worktree_use.last_used(&checkout.path, &self.history),
                    running,
                }
            })
            .collect();
        Some(ProjectInfo { id: id.clone(), name: project.name.clone(), git: project.kind == ProjectKind::Git, worktrees })
    }

    fn presence(&self, shown: &Shown) -> Presence {
        if let Some((tab, open)) = self.tabs.iter().find(|(_, tab)| tab.identity(self).as_ref() == Some(&shown.identity)) {
            return Presence::Open { tab, title: open.title(self) };
        }
        let running = |process: &crate::process::Process| Presence::Process { id: process.id, title: process.title() };
        match &shown.identity {
            TabIdentity::Terminal { session } => {
                let process = self.processes.iter().find(|process| process.view.hosted_id() == Some(session.as_str()) && process.is_live());
                process.map_or(Presence::Gone, running)
            }
            TabIdentity::Process(number) => {
                let process = self.processes.iter().find(|process| process.id.value() == *number);
                process.map_or(Presence::Gone, running)
            }
            _ if shown.saved.is_some() => Presence::Closed,
            _ => Presence::Gone,
        }
    }

    fn changed_items(&self) -> Vec<Item> {
        let mut items = Vec::new();
        for open in &self.projects {
            for checkout in open.listed_checkouts() {
                let Some(read) = self.hub_index.files(&checkout.path) else {
                    continue;
                };
                let checkout = open.project.checkout_ref(checkout);
                for change in read.changes.iter().filter(|change| !change.path.is_folder()) {
                    let path = change.path.display();
                    let name = change.path.file_name();
                    items.push(Item {
                        detail: change.path.folder(),
                        place: Place::One(checkout.clone()),
                        project: Some(checkout.project.clone()),
                        mark: Some(change.mark),
                        path: Some(path.clone()),
                        ..Item::new(Target::Change { checkout: checkout.clone(), path }, name.clone(), icons::file_icon(&name))
                    });
                }
            }
        }
        items
    }

    fn process_items(&self, live_only: bool) -> Vec<Item> {
        let processes = self.processes.iter().filter(|process| !live_only || process.is_live());
        let item = |process: &crate::process::Process| {
            let state = match &process.run {
                Some(origin) if !process.is_live() => format!("ended · {}", origin.name),
                Some(_) => "running".to_owned(),
                None => "live".to_owned(),
            };
            let detail = process.checkout.as_ref().map_or_else(String::new, |checkout| self.checkout_label(checkout));
            Item {
                detail,
                state: Some(state),
                place: process.checkout.clone().map_or(Place::Nowhere, Place::One),
                project: process.checkout.as_ref().map(|checkout| checkout.project.clone()),
                ..Item::new(Target::Process(process.id), process.title(), process.icon())
            }
        };
        processes.map(item).collect()
    }

    fn back_tab(&self, back: TabId, _cx: &App) -> Option<BackTab> {
        let tab = self.tabs.get(back)?;
        let checkout = tab.checkout(self);
        let project = tab.project(self);
        Some(BackTab { tab: back, project, checkout, is_text: matches!(tab, Tab::Document(_)) })
    }

    /// `@`: each project's run configurations, one row per folder and key, in the checkouts that have it.
    fn config_items(&self) -> Vec<Item> {
        let mut items: Vec<Item> = Vec::new();
        for open in &self.projects {
            let project = &open.project;
            let mut seen: Vec<(String, String, Vec<PathBuf>, String)> = Vec::new();
            for checkout in open.listed_checkouts() {
                let Some(read) = self.hub_index.files(&checkout.path) else {
                    continue;
                };
                for (folder, file) in read.configs.iter() {
                    for config in &file.configs {
                        match seen.iter_mut().find(|(at, key, _, _)| at == folder && *key == config.key) {
                            Some((_, _, checkouts, _)) => checkouts.push(checkout.path.clone()),
                            None => seen.push((folder.clone(), config.key.clone(), vec![checkout.path.clone()], config.name.clone())),
                        }
                    }
                }
            }
            for (folder, key, checkouts, name) in seen {
                let place = match checkouts.as_slice() {
                    [one] => Place::One(CheckoutRef { project: project.id.clone(), path: one.clone() }),
                    _ => Place::Several { project: project.id.clone(), checkouts },
                };
                let detail = if folder.is_empty() { project.name.clone() } else { format!("{} · {folder}", project.name) };
                let target = Target::RunConfig { project: project.id.clone(), folder, key };
                items.push(Item { detail, place, project: Some(project.id.clone()), ..Item::new(target, name, icons::RUN_CONFIG) });
            }
        }
        items
    }

    /// The branches without a worktree of every git project, each project's newest first.
    fn free_branches(&self) -> Vec<FreeBranch> {
        let mut free = Vec::new();
        for open in &self.projects {
            let project = &open.project;
            let Some(read) = self.hub_index.files(&project.main_checkout().path) else {
                continue;
            };
            free.extend(read.branches.iter().map(|branch| FreeBranch {
                project: project.id.clone(),
                local: branch.local.clone(),
                shown: branch.shown.clone(),
                remote: branch.remote.clone(),
                when: branch.when,
            }));
        }
        free
    }

    /// Opens what a row stands for. Behind: the hub stays shown and the opened tab, permanent, sits right of it.
    fn open_from_hub(&mut self, hub: TabId, row: OpenRow, window: &mut Window, cx: &mut Context<Self>) {
        if !row.behind {
            // taken before opening: a temporary hub goes as the row's tab is shown
            let state = match self.tabs.get(hub) {
                Some(Tab::Hub(open)) => HubState { back: open.back, ..open.state.clone() },
                _ => HubState::default(),
            };
            self.open_target(row, window, cx);
            // closing it with the key soon brings the hub back as it was (ive.rn's close_back_to_hub)
            self.hub_return = self.tabs.active_id().filter(|opened| *opened != hub).map(|tab| HubReturn { tab, at: Instant::now(), state });
            return;
        }
        let temporary = self.tabs.is_temporary(hub);
        self.tabs.keep(hub);
        let before: Vec<TabId> = self.tabs.iter().map(|(id, _)| id).collect();
        self.open_target(row, window, cx);
        if let Some(opened) = self.tabs.active_id().filter(|opened| *opened != hub) {
            self.tabs.keep(opened);
            self.tabs.activate(hub);
            if !before.contains(&opened) {
                self.tabs.place_behind(opened);
            }
        }
        if temporary {
            self.tabs.make_temporary(hub);
        }
        self.tab_shown(window, cx);
    }

    fn open_target(&mut self, row: OpenRow, window: &mut Window, cx: &mut Context<Self>) {
        let OpenRow { target, checkout, line, .. } = row;
        let line = line.and_then(|line| usize::try_from(line).ok());
        match target {
            Target::Tab(id) => self.activate_tab(id, window, cx),
            Target::Reopen(saved) => self.reopen(&saved, line, window, cx),
            Target::Process(id) => self.show_process(id, window, cx),
            Target::File { path, .. } | Target::Change { path, .. } => {
                if let Some(checkout) = checkout {
                    self.open_document_at(&checkout.path.join(path), line, window, cx);
                }
            }
            Target::Scratch(scratch) => {
                if let Err(error) = self.open_scratch(&scratch, window, cx) {
                    eprintln!("ive: cannot open scratch {}: {error}", scratch.name);
                }
            }
            Target::RunConfig { folder, key, .. } => {
                if let Some(checkout) = checkout {
                    self.run_from_hub(checkout, &folder, key, window, cx);
                }
            }
            Target::Action(action) => self.run_hub_action(action, window, cx),
            Target::Worktree(checkout) => self.reveal_checkout(&checkout, cx),
            Target::Branch { project, name } => self.start_worktree_of(&project, &name, window, cx),
            Target::Line(line) => self.go_to_line_of_back(usize::try_from(line).unwrap_or(usize::MAX), window, cx),
            // the model takes scope rows itself
            Target::Scope(_) => {}
            Target::NewTerminal(checkout) => {
                self.reveal_checkout(&checkout, cx);
                self.open_terminal_in(Some(checkout), window, cx);
            }
            // the hub adds the worktree first (`add_worktree_from_hub`)
            Target::NewWorktreeTerminal { .. } => {}
            Target::Hit { path, line, .. } => self.open_document_at(&path, usize::try_from(line).ok(), window, cx),
        }
    }

    /// Shows the file's tab, opening it when needed, at `line` (from 1) marked.
    pub(crate) fn open_document_at(&mut self, path: &Path, line: Option<usize>, window: &mut Window, cx: &mut Context<Self>) {
        self.open_document(path, window, cx);
        if let (Some(line), Some(shown)) = (line, self.tabs.active_id()) {
            self.cast_to_document_tab(shown, Cast::RevealLine(line), cx);
        }
    }

    fn reopen(&mut self, saved: &SavedTab, line: Option<usize>, window: &mut Window, cx: &mut Context<Self>) {
        match saved {
            SavedTab::File { path, .. } => self.open_document_at(path, line, window, cx),
            SavedTab::Settings { .. } => self.open_settings(window, cx),
            SavedTab::Hub => {
                self.open_hub(None, window, cx);
            }
            _ => match self.loaded_tab(saved, window, cx) {
                Some(tab) => {
                    self.open_tab(tab, window, cx);
                }
                None => eprintln!("ive: {} cannot be opened again: what it showed is gone", saved.title()),
            },
        }
    }

    /// A configuration's live run there, else a new run of it.
    fn run_from_hub(&mut self, checkout: CheckoutRef, folder: &str, key: String, window: &mut Window, cx: &mut Context<Self>) {
        let folder = checkout.path.join(folder);
        let live = self.processes.runs_in(&checkout.path, &folder).into_iter().find(|process| process.is_live() && process.run.as_ref().is_some_and(|origin| origin.key == key));
        match live.map(|process| process.id) {
            Some(process) => self.show_process(process, window, cx),
            None => self.request_run(RunRequest { checkout, folder, key, show: ShowRun::Tab }, window, cx),
        }
    }

    fn run_hub_action(&mut self, action: HubAction, window: &mut Window, cx: &mut Context<Self>) {
        let back = self.hub_back();
        let checkout = back.and_then(|back| self.tabs.get(back)).and_then(|tab| tab.checkout(self)).or_else(|| self.current_checkout());
        let project = checkout.as_ref().map(|checkout| checkout.project.clone());
        match action {
            HubAction::NewTerminal => {
                self.open_terminal_in(checkout, window, cx);
            }
            HubAction::NewScratch => {
                if let Some(project) = project {
                    self.new_scratch_in(project, window, cx);
                }
            }
            HubAction::GitLog => {
                if let Some(project) = project {
                    self.open_git_log_tab(&project, window, cx);
                }
            }
            HubAction::ProjectOverview => {
                if let Some(project) = project {
                    self.show_project_overview(&project, window, cx);
                }
            }
            HubAction::CloseOtherTabs => {
                if let Some(back) = back {
                    self.run_tab_command(Builtin::CloseOtherTabs, back, window, cx);
                }
            }
            HubAction::Settings => self.open_settings(window, cx),
            HubAction::Theme(preference) => self.set_theme(preference, window, cx),
        }
    }

    /// The tab shown before the hub, while it is open.
    fn hub_back(&self) -> Option<TabId> {
        match self.hub_tab_id().and_then(|id| self.tabs.get(id)) {
            Some(Tab::Hub(hub)) => hub.back.filter(|back| self.tabs.get(*back).is_some()),
            _ => None,
        }
    }

    fn new_scratch_in(&mut self, project: ProjectId, window: &mut Window, cx: &mut Context<Self>) {
        let existing = scratch::store_of(&project).and_then(|store| store.list().map_err(|error| error.to_string())).unwrap_or_default();
        let names: Vec<String> = existing.into_iter().map(|info| info.name).collect();
        let scratch = ScratchRef { project, name: free_scratch_name(&names) };
        if let Err(error) = self.create_scratch(&scratch, window, cx) {
            eprintln!("ive: cannot create scratch {}: {error}", scratch.name);
        }
    }

    /// A branch without a worktree: its project page's Worktrees list, the branch typed and its row selected.
    fn start_worktree_of(&mut self, project: &ProjectId, branch: &str, window: &mut Window, cx: &mut Context<Self>) {
        let Some(tab) = self.show_project_overview(project, window, cx) else {
            return;
        };
        self.update_tab(tab, |tab, cx| {
            if let Tab::ProjectOverview(overview) = tab {
                overview.select(ProjectSection::Worktrees, cx);
            }
        }, cx);
        self.section_shown(tab, window, cx);
        if let Some(views) = self.project_views(project, window, cx) {
            views.list.update(cx, |list, cx| list.start_with(branch, cx));
        }
    }

    /// `:42`: the tab shown before the hub, at line 42 marked.
    fn go_to_line_of_back(&mut self, line: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(back) = self.hub_back() else {
            return;
        };
        self.cast_to_document_tab(back, Cast::RevealLine(line), cx);
        self.activate_tab(back, window, cx);
    }

    /// Alt+Enter: the path pasted into the most recently used terminal, relative to its directory when it works in
    /// the path's checkout, without Enter.
    fn type_path_into_terminal(&self, path: &str) {
        let Some(terminal) = self.last_terminal() else {
            return;
        };
        if let Some(process) = self.processes.get(terminal) {
            process.view.paste(&self.typed_for_process(&[std::path::PathBuf::from(path)], terminal));
        }
    }
}

/// A contents search to start: where, what, and its ticket.
struct ContentSearch {
    roots: Vec<Root>,
    query: ContentQuery,
    ticket: Ticket,
}

/// The rows of a worktree question in that worktree: `● open` for a file open there.
struct WorkspaceStates<'a>(&'a Workspace);

impl StateIn for WorkspaceStates<'_> {
    fn state_in(&self, target: &Target, checkout: &Path) -> Option<String> {
        match target {
            Target::File { path, .. } => {
                let full = checkout.join(path);
                self.0.tabs.find(|tab| tab.shows(&full)).map(|_| "● open".to_owned())
            }
            Target::RunConfig { folder, key, .. } => {
                let runs = self.0.runs_in(checkout, &checkout.join(folder));
                runs.into_iter().find(|run| run.key == *key).map(|run| run.label)
            }
            _ => None,
        }
    }
}

/// Recent rows from the history, newest first: tabs of projects no longer open and processes that ended are left
/// out; a closed tab is greyed and reopens. `locate` gives a checkout path's open checkout: a file's is looked up
/// again from its path, since the one recorded may predate its project's worktrees being read.
pub(super) fn recent_items(history: &History, presence: impl Fn(&Shown) -> Presence, locate: impl Fn(&Path) -> Option<CheckoutRef>) -> Vec<Item> {
    let mut items = Vec::new();
    for shown in history.newest_first().filter(|shown| shown.identity != TabIdentity::Hub) {
        let at = match &shown.identity {
            TabIdentity::File(path) => Some(path.as_path()),
            _ => shown.checkout.as_deref(),
        };
        let located = at.map(&locate);
        if located.as_ref().is_some_and(Option::is_none) {
            continue;
        }
        let checkout = located.flatten();
        let project = checkout.as_ref().map(|checkout| checkout.project.clone());
        let (target, title, state, greyed) = match presence(shown) {
            Presence::Open { tab, title } => (Target::Tab(tab), title, Some("● open".to_owned()), false),
            Presence::Process { id, title } => (Target::Process(id), title, None, true),
            Presence::Closed => match &shown.saved {
                Some(saved) => (Target::Reopen(saved.clone()), shown.title.clone(), None, true),
                None => continue,
            },
            Presence::Gone => continue,
        };
        let relative = match (&shown.identity, &checkout) {
            (TabIdentity::File(path), Some(checkout)) => path.strip_prefix(&checkout.path).ok().map(|relative| relative.to_string_lossy().into_owned()),
            _ => None,
        };
        let place = checkout.map_or(Place::Nowhere, Place::One);
        let detail = relative.as_deref().and_then(|relative| relative.rsplit_once('/').map(|(folder, _)| folder.to_owned())).unwrap_or_else(|| kind_detail(&shown.identity).to_owned());
        let icon = shown.saved.as_ref().map_or(icons::RUN, saved_icon);
        items.push(Item { detail, state, place, project, age: Some(shown.shown_at), greyed, path: relative, ..Item::new(target, title, icon) });
    }
    items
}

fn kind_detail(identity: &TabIdentity) -> &'static str {
    match identity {
        TabIdentity::File(_) => "",
        TabIdentity::Scratch { .. } => "scratch",
        TabIdentity::Terminal { .. } => "terminal",
        TabIdentity::Process(_) => "run",
        TabIdentity::ProjectOverview(_) => "project",
        TabIdentity::GitLog(_) => "Git log",
        TabIdentity::Commit { .. } => "commit",
        TabIdentity::Diff { .. } => "uncommitted change",
        TabIdentity::Settings => "settings",
        TabIdentity::RunValues { .. } => "run values",
        TabIdentity::Hub => "",
    }
}

fn action_items() -> Vec<Item> {
    let actions = [
        (HubAction::NewTerminal, "New terminal", "in the worktree of the tab shown before", icons::TERMINAL),
        (HubAction::NewScratch, "New scratch", "in the project of the tab shown before", icons::FILE_TEXT),
        (HubAction::GitLog, "Open Git log", "of the project", icons::HISTORY),
        (HubAction::ProjectOverview, "Open project overview", "of the project", icons::PROJECT_FOLDER),
        (HubAction::CloseOtherTabs, "Close Other Tabs", "of the tab shown before", icons::CLOSE),
        (HubAction::Settings, "Settings", "", icons::SETTINGS),
        (HubAction::Theme(ThemePreference::Dark), "Theme: Dark", "", icons::SETTINGS),
        (HubAction::Theme(ThemePreference::Light), "Theme: Light", "", icons::SETTINGS),
        (HubAction::Theme(ThemePreference::System), "Theme: System", "", icons::SETTINGS),
    ];
    actions.into_iter().map(|(action, title, detail, icon)| Item { detail: detail.to_owned(), ..Item::new(Target::Action(action), title, icon) }).collect()
}

/// The scroll container's child of row `row`: the message, the section headers and the more lines of the sections
/// above come before it.
fn child_index(list: &List, row: usize) -> usize {
    let headers = list.sections.iter().take_while(|section| section.rows.start <= row).count();
    let mores = list.sections.iter().filter(|section| section.rows.end <= row && section.more.is_some()).count();
    row + headers + mores + usize::from(list.message.is_some())
}

/// `untitled-<n>`, the first `n` from 1 no scratch has.
fn free_scratch_name(existing: &[String]) -> String {
    (1..).map(|number| format!("untitled-{number}")).find(|name| !existing.contains(name)).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::saved_tabs::{PanelState, SavedTab};

    fn shown(identity: TabIdentity, saved: Option<SavedTab>, checkout: Option<&str>, at: i64) -> Shown {
        Shown { identity, saved, checkout: checkout.map(PathBuf::from), title: format!("t{at}"), shown_at: at }
    }

    #[test]
    fn should_leave_out_recent_rows_of_ended_processes_and_closed_projects() {
        // given
        let deleted = SavedTab::File { path: PathBuf::from("/p/gone.rs"), panel: PanelState::default() };
        let mut history = History::default();
        history.shown(shown(TabIdentity::Terminal { session: "ended".to_owned() }, Some(SavedTab::Terminal { session: "ended".to_owned() }), Some("/p"), 1));
        history.shown(shown(TabIdentity::File(PathBuf::from("/closed/a.rs")), None, Some("/closed"), 2));
        history.shown(shown(TabIdentity::File(PathBuf::from("/p/gone.rs")), Some(deleted.clone()), Some("/p"), 3));
        let presence = |shown: &Shown| match shown.identity {
            TabIdentity::Terminal { .. } => Presence::Gone,
            _ => Presence::Closed,
        };
        let locate = |path: &Path| path.starts_with("/p").then(|| CheckoutRef { project: ProjectId::parse("p").unwrap(), path: PathBuf::from("/p") });

        // when
        let items = recent_items(&history, presence, locate);

        // then
        assert_eq!(items.iter().map(|item| (item.target.clone(), item.greyed, item.path.clone())).collect::<Vec<_>>(), [(Target::Reopen(deleted), true, Some("gone.rs".to_owned()))]);
    }

    #[test]
    fn should_count_headers_and_more_lines_above_a_row() {
        // given
        let section = |rows: std::ops::Range<usize>, more: Option<&str>| crate::hub::rows::Section { title: "p".to_owned(), rows, note: None, more: more.map(str::to_owned) };
        let list = List { sections: vec![section(0..2, Some("… 3 more branches")), section(2..4, None)], rows: Vec::new(), more: 0, counter: None, message: None };

        // when
        let (first, third) = (child_index(&list, 0), child_index(&list, 2));

        // then
        assert_eq!((first, third), (1, 5));
    }

    #[test]
    fn should_group_thousands() {
        // when
        let grouped = [0, 999, 1000, 381_877, 1_234_567].map(thousands);

        // then
        assert_eq!(grouped, ["0", "999", "1,000", "381,877", "1,234,567"]);
    }

    #[test]
    fn should_name_a_new_scratch_after_the_first_free_number() {
        // given
        let existing = ["untitled-1".to_owned(), "untitled-3".to_owned(), "notes".to_owned()];

        // when
        let name = free_scratch_name(&existing);

        // then
        assert_eq!(name, "untitled-2");
    }

    #[test]
    fn should_place_a_recent_file_in_the_worktree_that_holds_it_now() {
        // given
        let saved = SavedTab::File { path: PathBuf::from("/p/wt/src/a.rs"), panel: PanelState::default() };
        let mut history = History::default();
        // recorded before the project's worktrees were read: the main checkout
        history.shown(shown(TabIdentity::File(PathBuf::from("/p/wt/src/a.rs")), Some(saved), Some("/p"), 1));
        let project = ProjectId::parse("p").unwrap();
        let locate = |path: &Path| {
            let at = if path.starts_with("/p/wt") { "/p/wt" } else { "/p" };
            Some(CheckoutRef { project: project.clone(), path: PathBuf::from(at) })
        };

        // when
        let items = recent_items(&history, |_| Presence::Closed, locate);

        // then
        let worktree = CheckoutRef { project: project.clone(), path: PathBuf::from("/p/wt") };
        assert_eq!((items[0].place.clone(), items[0].path.clone()), (Place::One(worktree), Some("src/a.rs".to_owned())));
    }

    #[test]
    fn should_title_a_recent_terminal_as_its_program_titles_it_now() {
        // given
        let session = "s1".to_owned();
        let mut history = History::default();
        // restored from saved tabs, where a terminal is titled before its program sets one
        history.shown(Shown { title: "Terminal".to_owned(), ..shown(TabIdentity::Terminal { session: session.clone() }, Some(SavedTab::Terminal { session }), None, 1) });
        let presence = |_: &Shown| Presence::Process { id: ProcessId::for_tests(4), title: "✳ Hub preview".to_owned() };

        // when
        let items = recent_items(&history, presence, |_| None);

        // then
        assert_eq!(items.iter().map(|item| item.title.as_str()).collect::<Vec<_>>(), ["✳ Hub preview"]);
    }

    #[test]
    fn should_leave_the_hub_out_of_recent() {
        // given
        let mut history = History::default();
        history.shown(shown(TabIdentity::Hub, Some(SavedTab::Hub), None, 1));

        // when
        let items = recent_items(&history, |_| Presence::Closed, |_| None);

        // then
        assert!(items.is_empty());
    }

    #[test]
    fn should_run_a_function_bound_in_the_hub() {
        // given
        let function = Action::Invoke { generation: 1, function: 3 };

        // when
        let taken = hub_key_use(Some(&function), &Keystroke::parse("ctrl-o").unwrap(), false);

        // then
        assert_eq!(taken, KeyUse::Invoke { generation: 1, function: 3 });
    }

    #[test]
    fn should_leave_typing_to_the_query_and_take_backspace_only_when_empty() {
        // given
        let widen = Action::Builtin(Builtin::HubWiden);
        let backspace = Keystroke::parse("backspace").unwrap();

        // when
        let typed = hub_key_use(Some(&widen), &backspace, false);
        let empty = hub_key_use(Some(&widen), &backspace, true);
        let window_key = hub_key_use(Some(&Action::Builtin(Builtin::OpenSettings)), &Keystroke::parse("ctrl-,").unwrap(), true);

        // then
        assert_eq!((typed, empty, window_key), (KeyUse::Pass, KeyUse::Hub(Builtin::HubWiden), KeyUse::Pass));
    }
}
