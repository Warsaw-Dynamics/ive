use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::time::Duration;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use futures::StreamExt;
use futures::channel::mpsc::UnboundedReceiver;
use futures::channel::oneshot;
use gpui::{App, AsyncWindowContext, ClickEvent, Context, Keystroke, Modifiers, ScrollStrategy, Task, WeakEntity, Window};
use serde_json::json;

use super::runs::{RunInputs, RunValues, read_run_values};
use super::{AfterOpen, Workspace};
use crate::run::wallet::OsWallet;
use crate::document::Document;
use crate::editor::{EditorView, Tab};
use crate::click_targets::ClickTargets;
use crate::api::{Call, Reply, Request, WindowInput};
use crate::window_capture;
use crate::script::state::{ScriptState, script_status_json};
use crate::open_project::OpenProject;
use crate::process::{Process, ProcessId, RunId};
use crate::run::file::{ConfigKind, RunFile};
use crate::run::values_tab::{RunRequest, ShowRun};
use crate::sidebar::rows::{RunRow, RunsView, TreeRow};
use crate::hub::model::HubState;
use crate::script::keymap::Builtin;
use crate::tabs::TabId;
use crate::terminal::{ScreenCell, TerminalView};

const CLOSED: &str = "the IVE window is closed";
/// A window the OS does not let draw (minimized, under a full-screen app) never draws; waits for a frame end then.
const FRAME_WAIT: Duration = Duration::from_millis(500);

/// Answers MCP calls on the gpui thread, one at a time, until the server or the window goes away.
pub(super) fn answer_calls(mut calls: UnboundedReceiver<Call>, window: &mut Window, cx: &mut Context<Workspace>) -> Task<()> {
    cx.spawn_in(window, async move |workspace, cx| {
        while let Some(Call { request, reply }) = calls.next().await {
            if let Request::NextFrame = request {
                // answered from the frame callback; if the window is gone, dropping `reply` tells the server
                let _ = workspace.update_in(cx, |_, window, _| answer_after_next_frame(reply, window));
                continue;
            }
            let answer = answer(&workspace, request, cx).await;
            // the agent may have given up waiting; nothing to do then
            let _ = reply.send(answer);
        }
    })
}

async fn answer(workspace: &WeakEntity<Workspace>, request: Request, cx: &mut AsyncWindowContext) -> Reply {
    match request {
        Request::WindowInput(input) => window_input(workspace, input, cx).await,
        Request::OpenProject { path } => open_project(workspace, path, cx).await,
        Request::RunConfig { folder, key } => run_config(workspace, folder, key, cx).await,
        Request::Screenshot => screenshot(cx).await,
        request => workspace.update_in(cx, |workspace, window, cx| workspace.answer(request, window, cx)).unwrap_or_else(|_| Err(CLOSED.to_owned())),
    }
}

/// Input for the whole window, dispatched outside the workspace's update (the handlers it reaches update the
/// workspace themselves) and answered once the window has drawn its result, so the next call sees the targets
/// that result drew. A click on a tree row out of view scrolls the row into view first.
async fn window_input(workspace: &WeakEntity<Workspace>, input: WindowInput, cx: &mut AsyncWindowContext) -> Reply {
    if let WindowInput::Click(name) = &input
        && workspace.update_in(cx, |workspace, _window, cx| workspace.scroll_to_target(name, cx)).unwrap_or(false)
    {
        next_frame(cx).await;
    }
    let answer = cx.update(|window, cx| dispatch(input, window, cx)).unwrap_or_else(|_| Err(CLOSED.to_owned()));
    next_frame(cx).await;
    answer
}

/// MCP `screenshot`: the main window drawn to an image on the gpui thread, encoded off it.
async fn screenshot(cx: &mut AsyncWindowContext) -> Reply {
    let image = cx.update(|window, _cx| window_capture::capture(window)).map_err(|_| CLOSED.to_owned())??;
    let png = cx.background_executor().spawn(async move { window_capture::encode_png(&image) }).await?;
    Ok(json!(BASE64.encode(png)))
}

/// MCP `run_config`: reads the values on the background executor (the wallet may block), then runs.
async fn run_config(workspace: &WeakEntity<Workspace>, folder: PathBuf, key: String, cx: &mut AsyncWindowContext) -> Reply {
    let inputs = workspace.update_in(cx, |workspace, _window, _cx| workspace.run_config_inputs(&folder, key)).map_err(|_| CLOSED.to_owned())?;
    let (request, inputs) = inputs?;
    let profile = crate::profile::current().name().to_owned();
    let read = cx.background_executor().spawn(async move { read_run_values(&inputs, &OsWallet::new(&profile)) }).await;
    workspace.update_in(cx, |workspace, window, cx| workspace.run_config_with(request, read, window, cx)).map_err(|_| CLOSED.to_owned())?
}

/// Opens a folder's project as Open Folder… does; answers with its id once it is open.
async fn open_project(workspace: &WeakEntity<Workspace>, path: PathBuf, cx: &mut AsyncWindowContext) -> Reply {
    let opening = workspace.update_in(cx, |workspace, window, cx| {
        let path = workspace.resolve(&path);
        workspace.open_folder_task(path, AfterOpen::ProjectOverview, window, cx)
    });
    let Ok(opening) = opening else {
        return Err(CLOSED.to_owned());
    };
    opening.await.map(|id| json!({ "project": id.as_str() }))
}

/// Waits for the window to draw a fresh frame (two frame starts, as `answer_after_next_frame`), at most 500 ms.
async fn next_frame(cx: &mut AsyncWindowContext) {
    let (drawn, frame) = oneshot::channel::<()>();
    let asked = cx.update(|window, _cx| {
        window.refresh();
        window.on_next_frame(move |window, _| {
            window.refresh();
            window.on_next_frame(move |_, _| {
                let _ = drawn.send(());
            });
        });
    });
    if asked.is_ok() {
        let timeout = cx.background_executor().timer(FRAME_WAIT);
        futures::future::select(frame, std::pin::pin!(timeout)).await;
    }
}

/// gpui runs next-frame callbacks as a frame starts, before it is drawn; the start of the frame after that is
/// the first moment the changed frame is certainly on screen.
fn answer_after_next_frame(reply: oneshot::Sender<Reply>, window: &mut Window) {
    window.refresh();
    window.on_next_frame(move |window, _| {
        window.refresh();
        window.on_next_frame(move |_, _| {
            let _ = reply.send(Ok(json!("drawn")));
        });
    });
}

fn dispatch(input: WindowInput, window: &mut Window, cx: &mut App) -> Reply {
    match input {
        WindowInput::Click(name) => {
            let listener = cx.global::<ClickTargets>().get(&name).ok_or_else(|| format!("no target {name:?} on screen; see list_targets"))?;
            listener(&ClickEvent::default(), window, cx);
            window.refresh();
            Ok(json!("clicked"))
        }
        WindowInput::Keys(keys) => {
            for key in &keys {
                let keystroke = Keystroke::parse(key).map_err(|error| format!("invalid key {key:?}: {error}"))?;
                window.dispatch_keystroke(keystroke, cx);
            }
            window.refresh();
            Ok(json!("ok"))
        }
        WindowInput::Text(text) => {
            for character in text.chars() {
                let typed = character.to_string();
                window.dispatch_keystroke(Keystroke { modifiers: Modifiers::default(), key: typed.clone(), key_char: Some(typed) }, cx);
            }
            window.refresh();
            Ok(json!("ok"))
        }
    }
}

impl Workspace {
    fn answer(&mut self, request: Request, window: &mut Window, cx: &mut Context<Self>) -> Reply {
        match request {
            Request::ListTabs => Ok(self.tab_list()),
            Request::ActiveTab => Ok(self.active_tab()),
            Request::ScriptStatus => Ok(script_status_json(cx.global::<ScriptState>())),
            Request::SendToActive { text } => self.send_to_active(&text).map(|()| json!("ok")),
            Request::ListProjects => Ok(self.project_list()),
            Request::OpenFile { path } => self.open_file(&path, window, cx).map(|id| json!({ "tab": id.value() })),
            Request::OpenTerminal { command, directory } => {
                self.open_terminal_running(command, directory, window, cx).map(|id| json!({ "tab": id.value() }))
            }
            Request::ActivateTab { tab } => self.tab_id(tab).map(|id| self.activate_tab(id, window, cx)).map(|()| json!("ok")),
            Request::CloseTab { tab } => self.tab_id(tab).map(|id| self.close_tab(id, window, cx)).map(|()| json!("ok")),
            Request::StopTab { tab } => self.tab_id(tab).map(|id| self.run_tab_command(Builtin::StopProgram, id, window, cx)).map(|()| json!("ok")),
            Request::OpenHub { hub } => parse_hub_state(hub).map(|state| json!({ "tab": self.open_hub(Some(state), window, cx).value() })),
            Request::DetachTab { tab } => self.detach_tab_by_request(self.tab_id(tab)?, window, cx).map(|()| json!("ok")),
            Request::AttachTab { tab } => self.attach_tab_by_request(self.tab_id(tab)?, window, cx).map(|()| json!("ok")),
            Request::SendText { tab, text } => self.send_text_to_tab(self.tab_id(tab)?, &text, cx).map(|()| json!("ok")),
            Request::SendKeys { tab, keys } => self.terminal(tab)?.send_keys(&keys).map(|()| json!("ok")),
            Request::SendMouse { tab, gesture } => {
                let to_program = self.terminal(tab)?.send_mouse(&gesture, cx)?;
                window.refresh();
                Ok(json!({ "to": if to_program { "program" } else { "ive" } }))
            }
            Request::TabMouse { tab, triggers } => self.tab_mouse_by_request(self.tab_id(tab)?, triggers, window, cx).map(|()| json!("ok")),
            Request::ReadScreen { tab } => {
                let screen = self.terminal(tab)?.screen_text();
                serde_json::to_value(screen).map_err(|error| format!("cannot encode the screen: {error}"))
            }
            Request::ReadOutput { tab, lines } => Ok(json!({ "lines": self.terminal(tab)?.output_text(lines) })),
            Request::ShowProcess { process } => self.show_process_tab(process, window, cx).map(|id| json!({ "tab": id.value() })),
            Request::ListTargets => Ok(self.target_names(cx)),
            Request::ListRunConfigs { folder } => self.list_run_configs(&self.resolve(&folder)),

            Request::NextFrame | Request::WindowInput(_) | Request::OpenProject { .. } | Request::RunConfig { .. } | Request::Screenshot => {
                Err("this request is answered before it reaches the workspace".to_owned())
            }
        }
    }

    /// What `click` can press now: the targets drawn in the last frame, and those of every tree row, drawn or not.
    fn target_names(&self, cx: &App) -> serde_json::Value {
        let mut names: BTreeSet<String> = cx.global::<ClickTargets>().names().into_iter().collect();
        names.extend(self.tree.rows.iter().flat_map(TreeRow::target_names));
        json!(names)
    }

    /// Scrolls the tree to the row with click target `name` when the row is out of view (not drawn, so its
    /// targets are not registered); says whether it scrolled.
    fn scroll_to_target(&self, name: &str, cx: &App) -> bool {
        if cx.global::<ClickTargets>().get(name).is_some() {
            return false;
        }
        let row = self.tree.rows.iter().position(|row| row.target_names().iter().any(|target| target == name));
        let Some(index) = row else {
            return false;
        };
        self.tree.list.list_handle().scroll_to_item(index, ScrollStrategy::Center);
        true
    }

    fn tab_list(&self) -> serde_json::Value {
        let shown = self.tabs.active_id();
        let tabs: Vec<_> = self
            .tabs
            .iter()
            .map(|(id, tab)| {
                let checkout = tab.checkout(self).map(|checkout| checkout.path);
                let detached = self.tabs.is_detached(id);
                let temporary = self.tabs.is_temporary(id);
                let mut listed = json!({ "id": id.value(), "kind": tab.kind(), "title": tab.title(self), "checkout": checkout, "shown": Some(id) == shown, "detached": detached, "temporary": temporary });
                self.add_opened_from_hub(id, &mut listed);
                listed
            })
            .collect();
        json!(tabs)
    }

    /// Tab `id` as a bound function's `context` gets it: `{ tab, kind, checkout }`, as `active_tab` reports a tab.
    pub(crate) fn tab_context(&self, id: TabId) -> serde_json::Value {
        let tab = self.tabs.get(id);
        let checkout = tab.and_then(|tab| tab.checkout(self)).map(|checkout| checkout.path);
        active_tab_json(tab.map(|tab| (id.value(), tab.kind(), checkout.as_deref())))
    }

    /// The tab of process `process`'s terminal as a bound function's `context`, with the clicked cell's `column` and
    /// `line` (from 0, as `read_screen` counts).
    pub(crate) fn terminal_context(&self, process: ProcessId, cell: ScreenCell) -> serde_json::Value {
        let id = self.tabs.find(|tab| tab.shows_process(process));
        let mut context = id.map_or_else(|| active_tab_json(None), |id| self.tab_context(id));
        context["column"] = json!(cell.column);
        context["line"] = json!(cell.line);
        context
    }

    fn active_tab(&self) -> serde_json::Value {
        let shown = self.tabs.active_id().and_then(|id| Some((id, self.tabs.get(id)?)));
        let checkout = shown.and_then(|(_, tab)| tab.checkout(self)).map(|checkout| checkout.path);
        let mut answer = active_tab_json(shown.map(|(id, tab)| (id.value(), tab.kind(), checkout.as_deref())));
        if let Some((id, _)) = shown {
            self.add_opened_from_hub(id, &mut answer);
        }
        answer
    }

    /// `opened_from_hub: { seconds_ago, hub }` on the tab just opened from the hub, until another tab is shown.
    fn add_opened_from_hub(&self, id: TabId, tab: &mut serde_json::Value) {
        if let Some(opened) = self.hub_return.as_ref().filter(|opened| opened.tab == id) {
            let hub = serde_json::to_value(&opened.state).unwrap_or_else(|_| json!({}));
            tab["opened_from_hub"] = json!({ "seconds_ago": opened.at.elapsed().as_secs_f64(), "hub": hub });
        }
    }

    /// Types `text` into the shown tab, as on the keyboard, when it is a process tab.
    pub(super) fn send_to_active(&self, text: &str) -> Result<(), String> {
        let shown = self.tabs.active_id().ok_or_else(|| "no tab is shown".to_owned())?;
        match self.tabs.get(shown) {
            Some(Tab::Process(_)) => self.send_text_to_process(shown, text),
            Some(other) => Err(format!("the shown tab is a {}, not a process", other.kind())),
            None => Err("no tab is shown".to_owned()),
        }
    }

    fn project_list(&self) -> serde_json::Value {
        json!(self.projects.iter().map(|open| self.project_json(open)).collect::<Vec<_>>())
    }

    fn project_json(&self, open: &OpenProject) -> serde_json::Value {
        let project = &open.project;
        let checkouts: Vec<_> = project
            .checkouts
            .iter()
            .map(|checkout| {
                // null while the status is unread, not 0
                let status = checkout.status.as_ref();
                json!({
                    "path": checkout.path, "label": checkout.label, "branch": checkout.branch, "main": checkout.is_main,
                    "changes": status.map(|status| status.changes),
                    "ahead": status.map(|status| status.ahead), "behind": status.map(|status| status.behind),
                    "base": status.and_then(|status| status.base.as_ref()),
                    "hidden": open.is_hidden(&checkout.path),
                })
            })
            .collect();
        let processes: Vec<_> = self.processes.in_project(&project.id).into_iter().map(|process| self.process_json(process)).collect();
        json!({
            "id": project.id.as_str(), "name": project.name, "path": project.root,
            "git": matches!(project.kind, crate::project::ProjectKind::Git), "missing": open.missing,
            "error": project.refresh_error, "checkouts": checkouts, "processes": processes,
        })
    }

    fn process_json(&self, process: &Process<TerminalView>) -> serde_json::Value {
        let tab = self.tabs.find(|tab| tab.shows_process(process.id)).map(TabId::value);
        let mut described = json!({
            "id": process.id.value(), "tab": tab, "kind": process.kind.name(), "title": process.title(),
            "progress": process.progress().describe(),
            "checkout": process.checkout.as_ref().map(|checkout| &checkout.path),
        });
        if let Some(origin) = &process.run {
            described["folder"] = json!(origin.folder);
            described["config"] = json!(origin.key);
            described["live"] = json!(process.is_live());
        }
        described
    }

    /// MCP `list_run_configs`: `folder`'s `.run.toml` read now, with the states of its runs.
    fn list_run_configs(&self, folder: &Path) -> Reply {
        let checkout = self.checkout_of_path(folder).ok_or_else(|| format!("{} is not in an open project", folder.display()))?;
        match super::runs::load_for_run(folder) {
            Ok(file) => Ok(run_configs_json(folder, &file, &self.shown_commands(&checkout, folder), &self.runs_in(&checkout.path, folder))),
            Err(error) => Ok(json!({ "folder": folder, "error": error })),
        }
    }

    /// What MCP `run_config` reads off the UI thread: the request and the inputs of its values.
    fn run_config_inputs(&self, folder: &Path, key: String) -> Result<(RunRequest, RunInputs), String> {
        let folder = self.resolve(folder);
        let checkout = self.checkout_of_path(&folder).ok_or_else(|| format!("{} is not in an open project", folder.display()))?;
        let request = RunRequest { checkout, folder, key, show: ShowRun::Tab };
        let inputs = self.run_inputs(&request);
        Ok((request, inputs))
    }

    /// MCP `run_config`, once the values are read: runs `key` as Run in the tree does, or names the values it lacks;
    /// an agent never types secrets.
    fn run_config_with(&mut self, request: RunRequest, read: Result<RunValues, String>, window: &mut Window, cx: &mut Context<Self>) -> Reply {
        let missing = read.as_ref().map_err(Clone::clone)?.1.missing.clone();
        if !missing.is_empty() {
            return Ok(json!({ "missing": missing }));
        }
        let (checkout, folder, key) = (request.checkout.path.clone(), request.folder.clone(), request.key.clone());
        // as Run in the tree: a live or waiting run is shown, not started a second time
        match super::runs::blocking_run(&self.runs_in(&checkout, &folder), &key) {
            Some(id) => self.show_run(id, window, cx),
            None => self.continue_run(request, read, window, cx),
        }
        let run = self.runs_in(&checkout, &folder).into_iter().find(|run| run.live && run.key == key);
        Ok(match run.map(|run| run.id) {
            Some(RunId::Process(id)) => json!({ "process": id.value() }),
            Some(RunId::Pending(_)) => json!({ "process": null, "waiting": true }),
            None => json!({ "process": null }),
        })
    }

    /// A path as the tools take it: absolute, or relative to the current checkout.
    fn resolve(&self, path: &Path) -> PathBuf {
        match self.current_checkout() {
            Some(checkout) if path.is_relative() => checkout.path.join(path),
            _ => path.to_path_buf(),
        }
    }

    /// Shows `path` in its tab, opening one when needed.
    pub(super) fn open_file(&mut self, path: &Path, window: &mut Window, cx: &mut Context<Self>) -> Result<TabId, String> {
        let path = self.resolve(path);
        let path = path.canonicalize().map_err(|error| format!("cannot open {}: {error}", path.display()))?;
        if let Some(open) = self.tabs.find(|tab| tab.shows(&path)) {
            self.activate_tab(open, window, cx);
            return Ok(open);
        }
        let document = Document::open(&path).map_err(|error| error.to_string())?;
        Ok(self.open_tab(Tab::Document(EditorView::new(document)), window, cx))
    }

    /// Starts a terminal in the checkout holding `directory` (the current checkout without one) and types
    /// `command` into it.
    fn open_terminal_running(
        &mut self,
        command: Option<String>,
        directory: Option<PathBuf>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Result<TabId, String> {
        let checkout = match directory {
            Some(directory) => {
                let directory = self.resolve(&directory);
                let directory = directory.canonicalize().map_err(|error| format!("cannot use {}: {error}", directory.display()))?;
                let checkout = self.checkout_of_path(&directory);
                Some(checkout.ok_or_else(|| format!("no open project contains {}", directory.display()))?)
            }
            None => self.current_checkout(),
        };
        let id = self.open_terminal_in(checkout, window, cx);
        if let Some(command) = command {
            // the shell reads it once started; the PTY holds it until then. A shell that could not start left a
            // failed tab, which says why.
            let _ = self.send_text_to_process(id, &format!("{command}\r"));
        }
        Ok(id)
    }

    fn show_process_tab(&mut self, process: u64, window: &mut Window, cx: &mut Context<Self>) -> Result<TabId, String> {
        let id = self.processes.id_of(process).ok_or_else(|| format!("no process {process}; see list_projects"))?;
        self.show_process(id, window, cx);
        self.tabs.find(|tab| tab.shows_process(id)).ok_or_else(|| format!("process {process} has no tab"))
    }

    fn tab_id(&self, tab: u64) -> Result<TabId, String> {
        self.tabs.iter().map(|(id, _)| id).find(|id| id.value() == tab).ok_or_else(|| format!("no tab {tab}; see list_tabs"))
    }

    fn terminal(&self, tab: u64) -> Result<&TerminalView, String> {
        match self.tabs.get(self.tab_id(tab)?) {
            Some(Tab::Process(id)) => self.processes.get(*id).map(|process| &process.view).ok_or_else(|| format!("tab {tab} has ended")),
            Some(other) => Err(format!("tab {tab} is a {}, not a process", other.kind())),
            None => Err(format!("no tab {tab}; see list_tabs")),
        }
    }
}

/// The hub state `open_hub` restores: what a tab's `opened_from_hub.hub` holds; `{}` is a fresh hub.
fn parse_hub_state(hub: serde_json::Value) -> Result<HubState, String> {
    serde_json::from_value(hub).map_err(|error| format!("open_hub: the hub state is not one open_hub made: {error}"))
}

fn active_tab_json(shown: Option<(u64, &str, Option<&Path>)>) -> serde_json::Value {
    match shown {
        Some((tab, kind, checkout)) => json!({ "tab": tab, "kind": kind, "checkout": checkout }),
        None => json!({ "tab": null, "kind": null, "checkout": null }),
    }
}

/// A `.run.toml`'s configurations for MCP: key, name, kind, the command as rows show it, and the live run's state
/// and process.
fn run_configs_json(folder: &Path, file: &RunFile, shown: &HashMap<String, String>, runs: &[RunRow]) -> serde_json::Value {
    let configurations: Vec<serde_json::Value> = file
        .configs
        .iter()
        .map(|config| {
            let (kind, written) = match &config.kind {
                ConfigKind::Run(run) => ("run", run.command.clone()),
                ConfigKind::Serve(serve) => ("serve", format!("serve {} → :{}", serve.root, serve.port)),
            };
            let live = runs.iter().find(|run| run.live && run.key == config.key);
            let process = live.and_then(|run| match run.id {
                RunId::Process(id) => Some(id.value()),
                RunId::Pending(_) => None,
            });
            json!({
                "key": config.key, "name": config.name, "kind": kind, "command": shown.get(&config.key).cloned().unwrap_or(written),
                "state": live.map(|run| run.status.clone()), "process": process,
            })
        })
        .collect();
    json!({ "folder": folder, "configurations": configurations })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_describe_run_configurations() {
        // given
        let file = crate::run::file::parse("[run.build]\nname = \"Build\"\ncommand = \"make\"\n\n[serve.web]\nroot = \"site\"\nport = 4200\n").unwrap();
        let shown = std::collections::HashMap::from([("build".to_owned(), "make".to_owned())]);
        let runs = vec![crate::sidebar::rows::RunRow {
            id: crate::process::RunId::Process(crate::process::ProcessId::for_tests(7)),
            key: "web".to_owned(),
            name: "web".to_owned(),
            serve: true,
            state: crate::sidebar::rows::RunBadge::Running,
            status: "running".to_owned(),
            label: "● running · :4200".to_owned(),
            live: true,
            link: Some("http://127.0.0.1:4200/".to_owned()),
        }];

        // when
        let described = run_configs_json(Path::new("/r/misc"), &file, &shown, &runs);

        // then
        assert_eq!(described, json!({
            "folder": "/r/misc",
            "configurations": [
                { "key": "build", "name": "Build", "kind": "run", "command": "make", "state": null, "process": null },
                { "key": "web", "name": "web", "kind": "serve", "command": "serve site → :4200", "state": "running", "process": 7 },
            ],
        }));
    }

    #[test]
    fn should_describe_no_active_tab() {
        // when
        let described = active_tab_json(None);

        // then
        assert_eq!(described, json!({ "tab": null, "kind": null, "checkout": null }));
    }

    #[test]
    fn should_describe_active_terminal() {
        // when
        let described = active_tab_json(Some((3, "process", Some(Path::new("/p")))));

        // then
        assert_eq!(described, json!({ "tab": 3, "kind": "process", "checkout": "/p" }));
    }

    #[test]
    fn should_reject_a_hub_state_open_hub_did_not_make() {
        // when
        let error = parse_hub_state(serde_json::json!({ "scope": 3 })).unwrap_err();

        // then
        assert!(error.contains("open_hub"), "{error}");
    }

    #[test]
    fn should_open_a_fresh_hub_for_an_empty_state() {
        // when
        let state = parse_hub_state(serde_json::json!({})).unwrap();

        // then
        assert_eq!(state, crate::hub::model::HubState::default());
    }
}
