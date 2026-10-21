//! IVE's API as Rune functions in the `ive` module: the same operations, names and arguments as the MCP tools.
//! Each sends a [`Call`] to the workspace and returns `Ok(answer)` or `Err(reason)`.

use std::future::Future;
use std::path::PathBuf;

use futures::channel::mpsc::UnboundedSender;
use rune::{ContextError, Module, Value};

use crate::api::{self, Call, Request, WindowInput};

type Answer = Result<Value, String>;

pub fn module(calls: UnboundedSender<Call>) -> Result<Module, ContextError> {
    let mut module = Module::with_crate("ive")?;
    let api = Api(calls);
    let a = api.clone();
    module.function("list_tabs", move || a.ask(Request::ListTabs)).build()?;
    let a = api.clone();
    module.function("list_projects", move || a.ask(Request::ListProjects)).build()?;
    let a = api.clone();
    module.function("active_tab", move || a.ask(Request::ActiveTab)).build()?;
    let a = api.clone();
    module.function("open_file", move |path: String| a.ask(Request::OpenFile { path: PathBuf::from(path) })).build()?;
    let a = api.clone();
    module.function("open_terminal", move |options: Value| a.open_terminal(options)).build()?;
    let a = api.clone();
    module.function("activate_tab", move |tab: u64| a.ask(Request::ActivateTab { tab })).build()?;
    let a = api.clone();
    module.function("close_tab", move |tab: u64| a.ask(Request::CloseTab { tab })).build()?;
    let a = api.clone();
    module.function("stop_tab", move |tab: u64| a.ask(Request::StopTab { tab })).build()?;
    let a = api.clone();
    module.function("open_hub", move |hub: Value| a.open_hub(hub)).build()?;
    let a = api.clone();
    module.function("detach_tab", move |tab: u64| a.ask(Request::DetachTab { tab })).build()?;
    let a = api.clone();
    module.function("attach_tab", move |tab: u64| a.ask(Request::AttachTab { tab })).build()?;
    let a = api.clone();
    module.function("send_text", move |tab: u64, text: String| a.ask(Request::SendText { tab, text })).build()?;
    let a = api.clone();
    module.function("send_keys", move |tab: u64, keys: Vec<String>| a.ask(Request::SendKeys { tab, keys })).build()?;
    let a = api.clone();
    module.function("read_screen", move |tab: u64| a.ask(Request::ReadScreen { tab })).build()?;
    let a = api.clone();
    module
        .function("read_output", move |tab: u64, lines: Option<usize>| a.ask(Request::ReadOutput { tab, lines }))
        .build()?;
    let a = api.clone();
    module.function("open_project", move |path: String| a.ask(Request::OpenProject { path: PathBuf::from(path) })).build()?;
    let a = api.clone();
    module.function("show_process", move |process: u64| a.ask(Request::ShowProcess { process })).build()?;
    let a = api.clone();
    module.function("list_run_configs", move |folder: String| a.ask(Request::ListRunConfigs { folder: PathBuf::from(folder) })).build()?;
    let a = api.clone();
    module.function("run_config", move |folder: String, key: String| a.ask(Request::RunConfig { folder: PathBuf::from(folder), key })).build()?;
    let a = api.clone();
    module.function("list_targets", move || a.ask(Request::ListTargets)).build()?;
    let a = api.clone();
    module.function("click", move |name: String| a.ask(Request::WindowInput(WindowInput::Click(name)))).build()?;
    let a = api.clone();
    module.function("window_keys", move |keys: Vec<String>| a.ask(Request::WindowInput(WindowInput::Keys(keys)))).build()?;
    let a = api.clone();
    module.function("window_text", move |text: String| a.ask(Request::WindowInput(WindowInput::Text(text)))).build()?;
    let a = api;
    module.function("send_to_active", move |text: String| a.ask(Request::SendToActive { text })).build()?;
    Ok(module)
}

#[derive(Clone)]
struct Api(UnboundedSender<Call>);

impl Api {
    fn ask(&self, request: Request) -> impl Future<Output = Answer> + use<> {
        let calls = self.0.clone();
        async move { to_rune(api::ask(&calls, request).await?) }
    }

    /// `open_hub(state)`: a tab's `opened_from_hub.hub`, or `#{}`.
    fn open_hub(&self, hub: Value) -> impl Future<Output = Answer> + use<> {
        let request = serde_json::to_value(&hub).map(|hub| Request::OpenHub { hub }).map_err(|error| format!("open_hub takes the hub state of opened_from_hub: {error}"));
        let api = self.clone();
        async move { api.ask(request?).await }
    }

    /// `open_terminal(#{ command, directory })`, both optional.
    fn open_terminal(&self, options: Value) -> impl Future<Output = Answer> + use<> {
        let request = terminal_options(&options).map(|(command, directory)| Request::OpenTerminal { command, directory });
        let api = self.clone();
        async move { api.ask(request?).await }
    }
}

fn terminal_options(options: &Value) -> Result<(Option<String>, Option<PathBuf>), String> {
    let options = serde_json::to_value(options).map_err(|error| format!("open_terminal takes #{{ command, directory }}: {error}"))?;
    let field = |name: &str| options.get(name).and_then(serde_json::Value::as_str).map(str::to_owned);
    Ok((field("command"), field("directory").map(PathBuf::from)))
}

fn to_rune(answer: serde_json::Value) -> Answer {
    serde_json::from_value(answer).map_err(|error| format!("cannot pass IVE's answer to the script: {error}"))
}
