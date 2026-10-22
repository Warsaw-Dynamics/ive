use std::io;
use std::net::TcpListener;
use std::path::PathBuf;
use std::time::Duration;

use futures::channel::mpsc::UnboundedSender;
use gpui::Modifiers;
use hyper::body::Incoming;
use hyper::{Request as HttpRequest, StatusCode, header};
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ContentBlock, Implementation, ServerCapabilities, ServerConfig};
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::streamable_http_server::{StreamableHttpServerConfig, StreamableHttpService};
use rmcp::{ErrorData, ServerHandler, schemars, tool, tool_handler, tool_router};
use serde::Deserialize;

use super::Token;
use crate::api::{self, Call, Request, WindowInput};
use crate::http_server::{self, HttpResponse};
use crate::script::keymap::{MouseButton, MouseKind, MouseTrigger};
use crate::terminal::{Button, Gesture, GestureKind, ScreenCell};

/// Bounds on what one call may send; an agent sending more is most likely stuck in a loop.
const MAX_TEXT_BYTES: usize = 64 * 1024;
const MAX_KEYS: usize = 256;
/// Longer than any key name (`ctrl-alt-shift-pageup`).
const MAX_KEY_BYTES: usize = 64;
const MAX_PATH_BYTES: usize = 4096;
/// A target name holds at most a path and a label.
const MAX_TARGET_BYTES: usize = 2 * MAX_PATH_BYTES;
const MAX_SCROLL_LINES: i32 = 100;
const FRAME_WAIT: Duration = Duration::from_millis(500);

const INSTRUCTIONS: &str = "Drives the IVE editor window. Tab ids come from list_tabs; projects, their checkouts \
and processes from list_projects. Process tabs (terminals, Claude Code) take send_text (end with \\r to press \
Enter), send_keys (e.g. [\"ctrl-c\"]) and send_mouse (clicks, drags, wheel on screen cells); read_screen returns \
their visible text, read_output all they printed (or its last lines). Scratch tabs take send_text too (\\r is a line break; the scratch saves itself 500 ms later). \
screenshot shows the whole window. list_targets names what click can press on screen \
(tree rows, overview sections and cards, git views and their rows, scratch rows and buttons, prompt buttons); \
window_keys and window_text type into whatever has the keyboard (click new-worktree first to type a branch). \
show_process brings back the tab of a process, by its id from list_projects. Run configurations (.run.toml in a \
folder): list_run_configs lists them, run_config runs one. The hub (Ctrl+Shift+E, open_hub) lists recent tabs, changes \
and processes and searches every file: window_text types its query, window_keys its keys, click hub-row:<n>.";

/// Starts the MCP server on `127.0.0.1:port` (0: any free port) and returns the port it listens on; requests need
/// `Authorization: Bearer <token>`. Binding happens before returning, so a taken port is reported to the caller.
pub fn start(port: u16, token: Token, calls: UnboundedSender<Call>) -> io::Result<u16> {
    let listener = TcpListener::bind(("127.0.0.1", port))?;
    let bound = listener.local_addr()?.port();
    listener.set_nonblocking(true)?;
    let runtime = tokio::runtime::Builder::new_multi_thread().worker_threads(2).enable_all().build()?;
    std::thread::Builder::new().name("mcp".to_owned()).spawn(move || {
        if let Err(error) = runtime.block_on(serve(listener, token, calls)) {
            eprintln!("ive: MCP server stopped: {error}");
        }
    })?;
    Ok(bound)
}

pub fn url(port: u16) -> String {
    format!("http://127.0.0.1:{port}/mcp")
}

async fn serve(listener: TcpListener, token: Token, calls: UnboundedSender<Call>) -> io::Result<()> {
    // loopback Host only (the default) and no browser Origin at all: web pages must not reach a local shell
    let config = StreamableHttpServerConfig::default().enforce_origin_validation();
    let service: StreamableHttpService<IveServer, LocalSessionManager> =
        StreamableHttpService::new(move || Ok(IveServer::new(calls.clone())), Default::default(), config);
    let endpoint = Endpoint { service, token };
    let listener = tokio::net::TcpListener::from_std(listener)?;
    http_server::serve(listener, "ive: MCP server", move |request| endpoint.clone().answer(request)).await;
    Ok(())
}

/// `/mcp` behind the bearer token.
#[derive(Clone)]
struct Endpoint {
    service: StreamableHttpService<IveServer, LocalSessionManager>,
    token: Token,
}

impl Endpoint {
    async fn answer(self, request: HttpRequest<Incoming>) -> HttpResponse {
        if self.authorized(&request) {
            return self.routed(request).await;
        }
        http_server::text(StatusCode::UNAUTHORIZED, "IVE MCP: missing or wrong bearer token")
    }

    async fn routed(self, request: HttpRequest<Incoming>) -> HttpResponse {
        if request.uri().path() == "/mcp" {
            return self.service.handle(request).await;
        }
        http_server::text(StatusCode::NOT_FOUND, "IVE MCP: the endpoint is /mcp")
    }

    fn authorized(&self, request: &HttpRequest<Incoming>) -> bool {
        request.headers().get(header::AUTHORIZATION).and_then(|value| value.to_str().ok()).is_some_and(|value| self.token.matches_header(value))
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct TabArgs {
    #[schemars(description = "tab id from list_tabs")]
    tab: u64,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct OpenHubArgs {
    #[schemars(description = "the hub state from a tab's opened_from_hub.hub; leave out for a fresh hub, as Ctrl+Shift+E; {\"new_terminal\": true} asks where a new terminal goes, as Ctrl+Shift+T")]
    hub: Option<serde_json::Value>,
}

/// More than a terminal keeps (its scrollback plus a screen).
const MAX_OUTPUT_LINES: u64 = 20_000;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ReadOutputArgs {
    #[schemars(description = "tab id from list_tabs")]
    tab: u64,
    #[schemars(description = "only the last this many lines (1-20000); all of them when left out")]
    lines: Option<u64>,
}

impl ReadOutputArgs {
    fn last(&self) -> Result<Option<usize>, String> {
        match self.lines {
            None => Ok(None),
            Some(lines @ 1..=MAX_OUTPUT_LINES) => Ok(Some(lines as usize)),
            Some(lines) => Err(format!("lines must be 1-{MAX_OUTPUT_LINES}, got {lines}")),
        }
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct OpenFileArgs {
    #[schemars(description = "file path, absolute or relative to the current checkout (the shown tab's, as Alt+F12 picks it)")]
    path: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct OpenTerminalArgs {
    #[schemars(description = "command to run in the new shell, e.g. \"cargo test\"; Enter is pressed for you")]
    command: Option<String>,
    #[schemars(description = "a folder in an open project, absolute or relative to the current checkout; the shell \
starts in the checkout holding it. Default: the current checkout (the shown tab's, as Alt+F12 picks it)")]
    directory: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct SendTextArgs {
    #[schemars(description = "process, file or scratch tab id from list_tabs")]
    tab: u64,
    #[schemars(description = "text typed as is; end with \\r to press Enter")]
    text: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct OpenProjectArgs {
    #[schemars(description = "a folder, absolute or relative to the current checkout: a repository's main checkout, \
any worktree or subfolder of it, or a plain folder")]
    path: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ShowProcessArgs {
    #[schemars(description = "process id from list_projects")]
    process: u64,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct RunFolderArgs {
    #[schemars(description = "the folder holding the .run.toml, absolute or relative to the current checkout")]
    folder: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct RunConfigArgs {
    #[schemars(description = "the folder holding the .run.toml, absolute or relative to the current checkout")]
    folder: String,
    #[schemars(description = "the configuration's key, as [run.<key>] or [serve.<key>] names it")]
    key: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ClickArgs {
    #[schemars(description = "a name from list_targets, e.g. \"section:Worktrees\" or \"prompt:Cancel\"")]
    target: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct WindowKeysArgs {
    #[schemars(description = "keys pressed in order, written like key bindings: enter, escape, backspace, alt-f12")]
    keys: Vec<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct WindowTextArgs {
    #[schemars(description = "text typed as is into whatever has the keyboard")]
    text: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct SendKeysArgs {
    #[schemars(description = "process tab id from list_tabs")]
    tab: u64,
    #[schemars(description = "keys pressed in order, written like key bindings: enter, escape, up, ctrl-c, alt-b, f5")]
    keys: Vec<String>,
}

#[derive(Debug, Clone, Copy, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
enum MouseAction {
    Click,
    DoubleClick,
    TripleClick,
    Press,
    Release,
    Move,
    Scroll,
}

#[derive(Debug, Clone, Copy, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
enum MouseButtonArg {
    Left,
    Middle,
    Right,
}

#[derive(Debug, Clone, Copy, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
enum ModifierArg {
    Shift,
    Alt,
    Ctrl,
}

#[derive(Debug, Clone, Copy, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
enum TabMouseAction {
    Click,
    DoubleClick,
    Press,
    Release,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct TabMouseArgs {
    #[schemars(description = "tab id from list_tabs, of a tab in the tab row")]
    tab: u64,
    #[schemars(description = "click (a press and a release), double_click, press, release")]
    action: TabMouseAction,
    #[schemars(description = "left (default), middle or right")]
    button: Option<MouseButtonArg>,
    #[schemars(description = "keys held: shift, alt, ctrl")]
    modifiers: Option<Vec<ModifierArg>>,
}

impl TabMouseArgs {
    /// The presses and releases a real mouse gives the tab, in order.
    fn triggers(&self) -> Vec<MouseTrigger> {
        let button = match self.button.unwrap_or(MouseButtonArg::Left) {
            MouseButtonArg::Left => MouseButton::Left,
            MouseButtonArg::Middle => MouseButton::Middle,
            MouseButtonArg::Right => MouseButton::Right,
        };
        let mods = held(self.modifiers.as_deref().unwrap_or_default());
        let trigger = |kind, streak| MouseTrigger { kind, button, streak, mods, mouse_reporting: false };
        match self.action {
            TabMouseAction::Click => vec![trigger(MouseKind::Down, 1), trigger(MouseKind::Up, 1)],
            TabMouseAction::DoubleClick => vec![trigger(MouseKind::Down, 1), trigger(MouseKind::Up, 1), trigger(MouseKind::Down, 2), trigger(MouseKind::Up, 2)],
            TabMouseAction::Press => vec![trigger(MouseKind::Down, 1)],
            TabMouseAction::Release => vec![trigger(MouseKind::Up, 1)],
        }
    }
}

fn held(modifiers: &[ModifierArg]) -> Modifiers {
    Modifiers {
        shift: modifiers.iter().any(|m| matches!(m, ModifierArg::Shift)),
        alt: modifiers.iter().any(|m| matches!(m, ModifierArg::Alt)),
        control: modifiers.iter().any(|m| matches!(m, ModifierArg::Ctrl)),
        ..Modifiers::default()
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct SendMouseArgs {
    #[schemars(description = "process tab id from list_tabs")]
    tab: u64,
    #[schemars(description = "click, double_click (selects a word), triple_click (a line), press, release, \
move (drags while a button is pressed), scroll")]
    action: MouseAction,
    #[schemars(description = "screen column, 0 at the left, as in read_screen")]
    column: u16,
    #[schemars(description = "screen row, 0 at the top, as in read_screen")]
    line: u16,
    #[schemars(description = "left (default), middle or right; for click, press and release")]
    button: Option<MouseButtonArg>,
    #[schemars(description = "keys held: shift, alt, ctrl. Shift makes the mouse select text even when the program tracks it")]
    modifiers: Option<Vec<ModifierArg>>,
    #[schemars(description = "for scroll: lines, positive scrolls up, negative down")]
    lines: Option<i32>,
}

impl SendMouseArgs {
    fn gesture(&self) -> Result<Gesture, String> {
        let button = match self.button.unwrap_or(MouseButtonArg::Left) {
            MouseButtonArg::Left => Button::Left,
            MouseButtonArg::Middle => Button::Middle,
            MouseButtonArg::Right => Button::Right,
        };
        let kind = match self.action {
            MouseAction::Click => GestureKind::Click { button, clicks: 1 },
            MouseAction::DoubleClick => GestureKind::Click { button, clicks: 2 },
            MouseAction::TripleClick => GestureKind::Click { button, clicks: 3 },
            MouseAction::Press => GestureKind::Press(button),
            MouseAction::Release => GestureKind::Release(button),
            MouseAction::Move => GestureKind::Move,
            MouseAction::Scroll => GestureKind::Scroll(self.scroll_lines()?),
        };
        let modifiers = held(self.modifiers.as_deref().unwrap_or_default());
        Ok(Gesture { kind, cell: ScreenCell { column: usize::from(self.column), line: usize::from(self.line) }, modifiers })
    }

    fn scroll_lines(&self) -> Result<i32, String> {
        match self.lines {
            Some(0) | None => Err("scroll needs lines: positive scrolls up, negative down".to_owned()),
            Some(lines) if lines.abs() > MAX_SCROLL_LINES => Err(format!("lines is {lines}; the limit is ±{MAX_SCROLL_LINES}")),
            Some(lines) => Ok(lines),
        }
    }
}

/// One MCP session; every session shares the same channel to the workspace.
#[derive(Clone)]
struct IveServer {
    calls: UnboundedSender<Call>,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl IveServer {
    fn new(calls: UnboundedSender<Call>) -> Self {
        Self { calls, tool_router: Self::tool_router() }
    }

    #[tool(description = "List open tabs: id, kind (document, process, project-overview, diff, \
commit, git-log, scratch, settings, failed, run-values, run-configs, hub), title, checkout (path, or null), which one is shown, which are \
detached (each in a window of its own), which one is temporary, and on a tab just opened from the hub \
opened_from_hub { seconds_ago, hub } (gone once another tab is shown; hub is for open_hub).")]
    async fn list_tabs(&self) -> Result<CallToolResult, ErrorData> {
        Ok(self.ask(Request::ListTabs).await)
    }

    #[tool(description = "The shown tab: id, kind and checkout; null fields when no tab is shown.")]
    async fn active_tab(&self) -> Result<CallToolResult, ErrorData> {
        Ok(self.ask(Request::ActiveTab).await)
    }

    #[tool(description = "Scripts from ~/.config/ive: whether init.rn loaded (load: loaded, failed, no-init), its \
errors with file and line, how many bindings it made, and the latest log lines (printed output, run-time errors).")]
    async fn script_status(&self) -> Result<CallToolResult, ErrorData> {
        Ok(self.ask(Request::ScriptStatus).await)
    }

    #[tool(description = "List open projects with their checkouts (path, label, branch, changes, ahead, behind; hidden: a worktree the user hid \
from the tree and the hub) and \
processes (id for show_process, tab id when a tab shows it, kind terminal, claude-code or run, title, checkout; a run \
also its folder, config and whether it is live).")]
    async fn list_projects(&self) -> Result<CallToolResult, ErrorData> {
        Ok(self.ask(Request::ListProjects).await)
    }

    #[tool(description = "Open a file in a tab, or show its tab when already open. Returns the tab id.")]
    async fn open_file(&self, Parameters(args): Parameters<OpenFileArgs>) -> Result<CallToolResult, ErrorData> {
        if let Some(refused) = too_long("path", &args.path, MAX_PATH_BYTES) {
            return Ok(refused);
        }
        Ok(self.ask(Request::OpenFile { path: PathBuf::from(args.path) }).await)
    }

    #[tool(description = "Start a terminal in a checkout and show it in a tab, optionally running a command. Returns the tab id.")]
    async fn open_terminal(&self, Parameters(args): Parameters<OpenTerminalArgs>) -> Result<CallToolResult, ErrorData> {
        let command = args.command.as_ref().and_then(|command| too_long("command", command, MAX_TEXT_BYTES));
        let directory = args.directory.as_ref().and_then(|directory| too_long("directory", directory, MAX_PATH_BYTES));
        if let Some(refused) = command.or(directory) {
            return Ok(refused);
        }
        Ok(self.ask(Request::OpenTerminal { command: args.command, directory: args.directory.map(PathBuf::from) }).await)
    }

    #[tool(description = "Show a tab.")]
    async fn activate_tab(&self, Parameters(args): Parameters<TabArgs>) -> Result<CallToolResult, ErrorData> {
        Ok(self.ask(Request::ActivateTab { tab: args.tab }).await)
    }

    #[tool(description = "Close a tab; a process keeps running after its tab closes (list_projects still shows it).")]
    async fn close_tab(&self, Parameters(args): Parameters<TabArgs>) -> Result<CallToolResult, ErrorData> {
        Ok(self.ask(Request::CloseTab { tab: args.tab }).await)
    }

    #[tool(description = "Stop a tab, as Ctrl+Shift+Q: a process tab's process ends for good (SIGTERM, SIGKILL 5 s \
later) and the tab closes; any other tab just closes.")]
    async fn stop_tab(&self, Parameters(args): Parameters<TabArgs>) -> Result<CallToolResult, ErrorData> {
        Ok(self.ask(Request::StopTab { tab: args.tab }).await)
    }

    #[tool(description = "Show the hub (Ctrl+Shift+E), temporary: the one open, or a new one; with hub (a tab's \
opened_from_hub.hub) in that state: query, scope, worktree question, selected row. Answers { tab }.")]
    async fn open_hub(&self, Parameters(args): Parameters<OpenHubArgs>) -> Result<CallToolResult, ErrorData> {
        Ok(self.ask(Request::OpenHub { hub: args.hub.unwrap_or_else(|| serde_json::json!({})) }).await)
    }

    #[tool(description = "Open a tab of the tab row in a window of its own, as dragging it out of the row does.")]
    async fn detach_tab(&self, Parameters(args): Parameters<TabArgs>) -> Result<CallToolResult, ErrorData> {
        Ok(self.ask(Request::DetachTab { tab: args.tab }).await)
    }

    #[tool(description = "Put a detached tab back into the tab row and show it there; its window closes.")]
    async fn attach_tab(&self, Parameters(args): Parameters<TabArgs>) -> Result<CallToolResult, ErrorData> {
        Ok(self.ask(Request::AttachTab { tab: args.tab }).await)
    }

    #[tool(description = "Type text into a process, file or scratch tab, as if typed on the keyboard. End with \\r to \
press Enter.")]
    async fn send_text(&self, Parameters(args): Parameters<SendTextArgs>) -> Result<CallToolResult, ErrorData> {
        if args.text.len() > MAX_TEXT_BYTES {
            return Ok(failure(format!("text is {} bytes; the limit is {MAX_TEXT_BYTES}", args.text.len())));
        }
        Ok(self.ask(Request::SendText { tab: args.tab, text: args.text }).await)
    }

    #[tool(description = "Press keys in a process tab, in order: enter, escape, tab, up, pageup, ctrl-c, ctrl-d, alt-b, f1-f12.")]
    async fn send_keys(&self, Parameters(args): Parameters<SendKeysArgs>) -> Result<CallToolResult, ErrorData> {
        if let Some(refused) = too_many_keys(&args.keys) {
            return Ok(refused);
        }
        Ok(self.ask(Request::SendKeys { tab: args.tab, keys: args.keys }).await)
    }

    #[tool(description = "Use the mouse in a process tab, on a screen cell. Goes to the program when it tracks the mouse \
(mc, Claude Code, vim); otherwise selects text (click, double_click, triple_click, then move to extend) or scrolls \
history. Returns {\"to\": \"program\"} or {\"to\": \"ive\"}.")]
    async fn send_mouse(&self, Parameters(args): Parameters<SendMouseArgs>) -> Result<CallToolResult, ErrorData> {
        match args.gesture() {
            Ok(gesture) => Ok(self.ask(Request::SendMouse { tab: args.tab, gesture }).await),
            Err(reason) => Ok(failure(reason)),
        }
    }

    #[tool(description = "Use the mouse on a tab of the tab row, as a real click on it: the tab mouse bindings \
(init.rn's, then ive.rn's) decide what it does, e.g. a right click opens the tab's menu (at the pointer), a middle \
click closes it. Modifiers pick other bindings, such as a Shift+right click's menu.")]
    async fn tab_mouse(&self, Parameters(args): Parameters<TabMouseArgs>) -> Result<CallToolResult, ErrorData> {
        Ok(self.ask(Request::TabMouse { tab: args.tab, triggers: args.triggers() }).await)
    }

    #[tool(description = "Read the visible text of a process tab: one string per screen row, plus the cursor position.")]
    async fn read_screen(&self, Parameters(args): Parameters<TabArgs>) -> Result<CallToolResult, ErrorData> {
        Ok(self.ask(Request::ReadScreen { tab: args.tab }).await)
    }

    #[tool(description = "Read what a process tab printed, scrollback included, as text lines (a long line wrapped \
on screen is one line): all of it, or only the last `lines`. Prefer it to read_screen for a command's result.")]
    async fn read_output(&self, Parameters(args): Parameters<ReadOutputArgs>) -> Result<CallToolResult, ErrorData> {
        match args.last() {
            Ok(lines) => Ok(self.ask(Request::ReadOutput { tab: args.tab, lines }).await),
            Err(reason) => Ok(failure(reason)),
        }
    }

    #[tool(description = "Open a folder's project, as Open Folder… does with it picked: added to the sidebar (or \
shown when open) with its project overview. Answers once it is open, with {\"project\": id}, or why it could not \
open.")]
    async fn open_project(&self, Parameters(args): Parameters<OpenProjectArgs>) -> Result<CallToolResult, ErrorData> {
        if let Some(refused) = too_long("path", &args.path, MAX_PATH_BYTES) {
            return Ok(refused);
        }
        Ok(self.ask(Request::OpenProject { path: PathBuf::from(args.path) }).await)
    }

    #[tool(description = "Show the tab of a process (id from list_projects), opening it again when it was closed. \
Returns the tab id.")]
    async fn show_process(&self, Parameters(args): Parameters<ShowProcessArgs>) -> Result<CallToolResult, ErrorData> {
        Ok(self.ask(Request::ShowProcess { process: args.process }).await)
    }

    #[tool(description = "A folder's run configurations (.run.toml) read now: key, name, kind (run or serve), command \
(variables filled in, secrets as ${name}), state of its live run and its process id; or the file's error.")]
    async fn list_run_configs(&self, Parameters(args): Parameters<RunFolderArgs>) -> Result<CallToolResult, ErrorData> {
        Ok(self.ask(Request::ListRunConfigs { folder: PathBuf::from(args.folder) }).await)
    }

    #[tool(description = "Run a configuration of a folder's .run.toml, its before steps first, as Run in the tree does; \
a live one is shown instead. Answers with its process id, or with the variables it needs and has no value for \
(missing), which only the user enters, in IVE.")]
    async fn run_config(&self, Parameters(args): Parameters<RunConfigArgs>) -> Result<CallToolResult, ErrorData> {
        Ok(self.ask(Request::RunConfig { folder: PathBuf::from(args.folder), key: args.key }).await)
    }

    #[tool(description = "List what click can press now, by name. Sidebar: open-project, recent:<id>, open-folder, \
project:<id>, toggle:<id>, close-project:<id>, toggle:<id>/<label> (a worktree), \
badge:<id>/<label>/<folder> (shows or hides a worktree folder: processes, uncommitted, since-base, commits), \
folder:<id>/<label>/<folder> (folds it; also files; processes opens its Run configurations tab), \
toggle:<id>/<label>/processes (folds it), new-terminal:<id>/<label>/processes, process:<n>, \
stop-process:<n> (ends it for good), live-run:<n>, change:<id>/<label>/<source>:<path> (its diff), \
change-folder:<id>/<label>/<source>:<path>/, commit:<id>/<label>/<sha> (folds its files), \
commits-more:<id>/<label> (Load 20 more), \
entry:<path>, more:<list>, all:<list> (a label two checkouts share is <label>@<path>; a file a worktree inside \
the main checkout shows twice is entry:<path>@<label> in the main checkout's tree). Tab row: tab-menu:<tab> opens a tab's main menu, then tab-menu:keep, tab-menu:stop (a process tab: ends its process for good), tab-menu:kill (when a menu names it), \
tab-menu:close, tab-menu:close-others, tab-menu:close-left, tab-menu:close-right. Tabs: settings, \
theme:<name>, settings-page:<page>. \
Project overview: section:<label>, card:<label> (reveals the worktree in the tree). Project Git section and Git log: open-git, log:<sha>, git-log:Log, \
git-log:Stashes, git-log:Worktrees, git-log:Branches, load-more. Diff and commit tabs: reload-diff, \
diff-file:<path>. Run configurations tab (<folder> checkout-relative): run-configs-filter, \
run-configs-layout:flat, run-configs-layout:tree, run-configs-folder:<folder>, open-run-file:<folder>, \
run-config:<folder>#<key> (shows its run), run:<folder>#<key> (Run, the list stays), run-configs-restart:<n>, \
run-configs-stop:<n>, run-configs-start:<n>, run-configs-remove:<n>. Hub: hub-query, hub-row:<n> (from 1, as numbered), hub-worktree:<checkout> (the worktree \
question), hub-scope:<all|project id|checkout path>, hub-scope-picker, hub-excluded. New worktree card: new-worktree, branch:<name>, branch-more, branch-all. Scratches section: \
new-scratch, scratch-search, scratch-name (the open name input), scratch:<name>, rename-scratch:<name>, \
delete-scratch:<name>, scratch-group:<group>, scratch-more:<group>, scratch-all:<group>. Scratch tabs: \
run-scratch, scratch-reload, scratch-overwrite, scratch-save-again, scratch-close. File and scratch tabs whose save found the \
file changed on disk: save-reload, save-overwrite. Prompts: prompt:<button>. \
<sha> is the 7-character short sha. Every tree row is listed, also those scrolled out of view; the rest only \
while on screen.")]
    async fn list_targets(&self) -> Result<CallToolResult, ErrorData> {
        Ok(self.ask(Request::ListTargets).await)
    }

    #[tool(description = "Click a target named by list_targets, as a mouse click on it would; a tree row out of view \
is scrolled to first. Answers once the window has drawn the result.")]
    async fn click(&self, Parameters(args): Parameters<ClickArgs>) -> Result<CallToolResult, ErrorData> {
        if let Some(refused) = too_long("target", &args.target, MAX_TARGET_BYTES) {
            return Ok(refused);
        }
        Ok(self.ask(Request::WindowInput(WindowInput::Click(args.target))).await)
    }

    #[tool(description = "Press keys in whatever has the keyboard in the IVE window (a prompt, a terminal, the \
workspace's key bindings), in order: enter, escape, alt-f12.")]
    async fn window_keys(&self, Parameters(args): Parameters<WindowKeysArgs>) -> Result<CallToolResult, ErrorData> {
        if let Some(refused) = too_many_keys(&args.keys) {
            return Ok(refused);
        }
        Ok(self.ask(Request::WindowInput(WindowInput::Keys(args.keys))).await)
    }

    #[tool(description = "Type text into whatever has the keyboard in the IVE window.")]
    async fn window_text(&self, Parameters(args): Parameters<WindowTextArgs>) -> Result<CallToolResult, ErrorData> {
        if args.text.len() > MAX_TEXT_BYTES {
            return Ok(failure(format!("text is {} bytes; the limit is {MAX_TEXT_BYTES}", args.text.len())));
        }
        Ok(self.ask(Request::WindowInput(WindowInput::Text(args.text))).await)
    }

    #[tool(description = "Capture the IVE window as a PNG, as IVE last drew it. Works while other windows cover it or it is minimized.")]
    async fn screenshot(&self) -> Result<CallToolResult, ErrorData> {
        // without this, a screenshot right after send_keys shows the frame from before the keys; a window the OS
        // does not let draw (minimized, under a full-screen app) never answers, so the wait is bounded
        let _ = tokio::time::timeout(FRAME_WAIT, self.ask(Request::NextFrame)).await;
        Ok(screenshot_result(api::ask(&self.calls, Request::Screenshot).await))
    }
}

/// The workspace answers a screenshot with the PNG as base64, ready for the image block.
fn screenshot_result(reply: api::Reply) -> CallToolResult {
    match reply {
        Ok(serde_json::Value::String(png)) => CallToolResult::success(vec![ContentBlock::image(png, "image/png")]),
        Ok(other) => failure(format!("the workspace answered a screenshot with {other} instead of a PNG")),
        Err(reason) => failure(reason),
    }
}

impl IveServer {
    /// Hands `request` to the workspace on the gpui thread and waits for its answer.
    async fn ask(&self, request: Request) -> CallToolResult {
        match api::ask(&self.calls, request).await {
            Ok(value) => CallToolResult::success(vec![ContentBlock::text(value.to_string())]),
            Err(reason) => failure(reason),
        }
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for IveServer {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("ive", env!("CARGO_PKG_VERSION")))
            .with_instructions(INSTRUCTIONS)
    }
}

/// A failed tool call the agent sees and can react to, unlike a protocol error.
fn failure(reason: String) -> CallToolResult {
    CallToolResult::error(vec![ContentBlock::text(reason)])
}

/// A failure when `value` is longer than `limit` bytes.
fn too_long(what: &str, value: &str, limit: usize) -> Option<CallToolResult> {
    (value.len() > limit).then(|| failure(format!("{what} is {} bytes; the limit is {limit}", value.len())))
}

/// A failure when a call sends more than 256 keys or a key name longer than 64 bytes.
fn too_many_keys(keys: &[String]) -> Option<CallToolResult> {
    if keys.len() > MAX_KEYS {
        return Some(failure(format!("{} keys sent; the limit is {MAX_KEYS} per call", keys.len())));
    }
    keys.iter().find_map(|key| too_long("a key name", key, MAX_KEY_BYTES))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(json: serde_json::Value) -> SendMouseArgs {
        serde_json::from_value(json).unwrap_or_else(|error| panic!("bad send_mouse arguments: {error}"))
    }

    #[test]
    fn should_answer_screenshot_with_the_png_the_workspace_drew() {
        // given
        let reply: crate::api::Reply = Ok(serde_json::json!("iVBORw0KGgo="));

        // when
        let result = screenshot_result(reply);

        // then
        assert_eq!(result.is_error, Some(false));
        assert_eq!(result.content[0].as_image().map(|image| image.data.as_str()), Some("iVBORw0KGgo="));
    }

    #[test]
    fn should_fail_screenshot_when_the_workspace_answers_no_picture() {
        // given
        let reply: crate::api::Reply = Ok(serde_json::json!({ "tab": 1 }));

        // when
        let result = screenshot_result(reply);

        // then
        assert_eq!(result.is_error, Some(true));
    }

    #[test]
    fn should_turn_send_mouse_arguments_into_a_gesture() {
        // given
        let request = args(serde_json::json!({
            "tab": 1, "action": "double_click", "column": 3, "line": 7, "button": "right", "modifiers": ["ctrl", "shift"]
        }));

        // when
        let gesture = request.gesture().expect("valid gesture");

        // then
        assert_eq!(gesture.kind, GestureKind::Click { button: Button::Right, clicks: 2 });
        assert_eq!(gesture.cell, ScreenCell { column: 3, line: 7 });
        assert!(gesture.modifiers.control && gesture.modifiers.shift && !gesture.modifiers.alt);
    }

    #[test]
    fn should_turn_a_shift_right_click_on_a_tab_into_a_press_and_a_release() {
        // given
        let request: TabMouseArgs = serde_json::from_value(serde_json::json!({ "tab": 3, "action": "click", "button": "right", "modifiers": ["shift"] })).unwrap();

        // when
        let triggers = request.triggers();

        // then
        let shift = Modifiers { shift: true, ..Modifiers::default() };
        let trigger = |kind| MouseTrigger { kind, button: MouseButton::Right, streak: 1, mods: shift, mouse_reporting: false };
        assert_eq!(triggers, [trigger(MouseKind::Down), trigger(MouseKind::Up)]);
    }

    #[test]
    fn should_turn_a_double_click_on_a_tab_into_two_streaks() {
        // given
        let request: TabMouseArgs = serde_json::from_value(serde_json::json!({ "tab": 3, "action": "double_click" })).unwrap();

        // when
        let streaks: Vec<(MouseKind, u8)> = request.triggers().iter().map(|trigger| (trigger.kind, trigger.streak)).collect();

        // then
        assert_eq!(streaks, [(MouseKind::Down, 1), (MouseKind::Up, 1), (MouseKind::Down, 2), (MouseKind::Up, 2)]);
    }

    #[test]
    fn should_reject_scroll_without_lines_or_beyond_the_limit() {
        // given
        let missing = args(serde_json::json!({ "tab": 1, "action": "scroll", "column": 0, "line": 0 }));
        let huge = args(serde_json::json!({ "tab": 1, "action": "scroll", "column": 0, "line": 0, "lines": -1000 }));
        let fine = args(serde_json::json!({ "tab": 1, "action": "scroll", "column": 0, "line": 0, "lines": -3 }));

        // when
        let results = [missing.gesture(), huge.gesture(), fine.gesture()].map(|result| result.map(|gesture| gesture.kind));

        // then
        assert!(results[0].is_err() && results[1].is_err());
        assert_eq!(results[2], Ok(GestureKind::Scroll(-3)));
    }

    #[test]
    fn should_read_all_output_or_a_bounded_number_of_last_lines() {
        // given
        let read = |json| serde_json::from_value::<ReadOutputArgs>(json).expect("valid read_output arguments");
        let requests = [
            read(serde_json::json!({ "tab": 1 })),
            read(serde_json::json!({ "tab": 1, "lines": 50 })),
            read(serde_json::json!({ "tab": 1, "lines": 0 })),
            read(serde_json::json!({ "tab": 1, "lines": 1_000_000 })),
        ];

        // when
        let results = requests.map(|request| request.last());

        // then
        assert_eq!((&results[0], &results[1]), (&Ok(None), &Ok(Some(50))));
        assert!(results[2].is_err() && results[3].is_err());
    }
}
