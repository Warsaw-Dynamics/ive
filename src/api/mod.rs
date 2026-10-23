//! IVE's API: what MCP and scripts may ask of the workspace. Callers on other threads send a [`Call`]; the
//! workspace answers it on the gpui thread.

mod client;

use std::path::PathBuf;

use futures::channel::oneshot;

use crate::script::keymap::MouseTrigger;
use crate::terminal::Gesture;

pub use client::ask;

/// What a caller (MCP, a script) asks of the workspace.
#[derive(Debug)]
pub enum Request {
    ListTabs,
    /// the tab shown: id, kind and checkout, or nulls when none is
    ActiveTab,
    /// types `text` into the shown tab when it is a process tab
    SendToActive { text: String },
    /// how the last load of `init.rn` went, and the latest script log lines
    ScriptStatus,
    /// the open notices, newest first, and the app log's path
    Notices,
    ListProjects,
    OpenFile { path: PathBuf },
    /// in the checkout holding `directory`, or the current checkout as Alt+F12 picks it
    OpenTerminal { command: Option<String>, directory: Option<PathBuf> },
    ActivateTab { tab: u64 },
    CloseTab { tab: u64 },
    /// as Ctrl+Shift+Q: a process tab's process ends for good and the tab closes; another tab just closes
    StopTab { tab: u64 },
    /// shows the hub, temporary, in a state from a tab's `opened_from_hub` (`{}`: as Ctrl+Shift+E)
    OpenHub { hub: serde_json::Value },
    /// opens a tab of the tab row in a window of its own
    DetachTab { tab: u64 },
    /// puts a detached tab back into the tab row
    AttachTab { tab: u64 },
    SendText { tab: u64, text: String },
    SendKeys { tab: u64, keys: Vec<String> },
    SendMouse { tab: u64, gesture: Gesture },
    /// mouse presses and releases on a tab of the tab row, run through the tab mouse bindings
    TabMouse { tab: u64, triggers: Vec<MouseTrigger> },
    ReadScreen { tab: u64 },
    /// the whole output as text lines, scrollback included; only the last `lines` when given
    ReadOutput { tab: u64, lines: Option<usize> },
    /// shows the process's tab, opening one when it was closed
    ShowProcess { process: u64 },
    /// a folder's `.run.toml`: its configurations and their states, or why it does not load
    ListRunConfigs { folder: PathBuf },
    /// runs (or restarts) configuration `key` of `folder`; never asks for values, only names the missing ones
    RunConfig { folder: PathBuf, key: String },
    /// as Open Folder… with this folder picked; answered once the project is open
    OpenProject { path: PathBuf },
    /// what `WindowInput::Click` can press: the targets on screen and those of every tree row
    ListTargets,
    /// input for the whole window, dispatched outside the workspace's update
    WindowInput(WindowInput),
    /// answered once the window has drawn a fresh frame, so a screenshot shows the latest changes
    NextFrame,
    /// the main window as last drawn, a PNG in base64
    Screenshot,
}

/// Input for whatever is on screen or has the keyboard.
#[derive(Debug, PartialEq, Eq)]
pub enum WindowInput {
    /// the click target of that name, as `Request::ListTargets` names them
    Click(String),
    Keys(Vec<String>),
    Text(String),
}

/// The answer as JSON for the tool result, or a message saying why the request failed.
pub type Reply = Result<serde_json::Value, String>;

/// A request and where its answer goes.
pub struct Call {
    pub request: Request,
    pub reply: oneshot::Sender<Reply>,
}
