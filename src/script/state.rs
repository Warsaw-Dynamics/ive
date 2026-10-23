//! The scripts as the gpui thread knows them: the keymap key handlers read, the last load's outcome and where the
//! script log is. Plain data only; Rune stays on the script thread.

use futures::channel::mpsc::UnboundedSender;
use gpui::{App, Global, KeyDownEvent, Keystroke};

use super::keymap::{Action, Builtin, Keymap, MouseTrigger, Place, Scope};
use super::log::ScriptLog;
use super::{ScriptCommand, ScriptEvent};
use crate::settings::ThemeChoice;

/// Log lines `script_status` returns: enough for the last run, small enough for an agent to read.
const STATUS_LOG_LINES: usize = 50;

/// How the last load of `init.rn` went.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadStatus {
    NoInit,
    Loaded { bindings: usize },
    /// the bindings of the last good load stay in effect
    Failed { message: String, errors: usize },
}

pub struct ScriptState {
    keymap: Keymap,
    load: LoadStatus,
    log: ScriptLog,
    commands: UnboundedSender<ScriptCommand>,
    /// the user themes the last load found
    themes: Vec<String>,
}

/// How long a menu condition may take before its item counts as hidden.
pub const TEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

impl Global for ScriptState {}

impl ScriptState {
    pub fn new(commands: UnboundedSender<ScriptCommand>, log: ScriptLog) -> Self {
        Self { keymap: Keymap::empty(), load: LoadStatus::NoInit, log, commands, themes: Vec::new() }
    }

    pub fn apply(&mut self, event: ScriptEvent) {
        match event {
            ScriptEvent::Loaded { keymap, has_init, themes, .. } => {
                self.themes = themes;
                self.load = if has_init { LoadStatus::Loaded { bindings: keymap.len() } } else { LoadStatus::NoInit };
                self.keymap = keymap;
            }
            ScriptEvent::LoadFailed { message, errors, themes } => {
                self.themes = themes;
                self.load = LoadStatus::Failed { message, errors };
            }
            // the workspace shows the log, or hands the answer to the open menu; nothing to keep
            ScriptEvent::RunFailed | ScriptEvent::Tested { .. } => {}
        }
    }

    pub fn find_key(&self, keystroke: &Keystroke, place: Place) -> Option<Action> {
        self.keymap.find_key(keystroke, place).cloned()
    }

    pub fn find_mouse(&self, scope: Scope, trigger: &MouseTrigger) -> Option<Action> {
        self.keymap.find_mouse(scope, trigger).cloned()
    }

    /// Asks the script thread whether menu condition `function` holds on `context`; the answer comes back as
    /// `ScriptEvent::Tested` for `request`.
    pub fn test(&self, generation: u64, function: usize, context: serde_json::Value, request: u64) {
        let command = ScriptCommand::Test { generation, function, context, request, timeout: TEST_TIMEOUT };
        // the script thread is gone only when it could not start; the menu's item then stays pending until it closes
        let _ = self.commands.unbounded_send(command);
    }

    /// Asks the script thread to run a bound function on `context` (the shown tab when `None`).
    pub fn invoke(&self, generation: u64, function: usize, context: Option<serde_json::Value>) {
        // the script thread is gone only when it could not start, which it logged
        let _ = self.commands.unbounded_send(ScriptCommand::Invoke { generation, function, context });
    }

    pub fn reload(&self) {
        let _ = self.commands.unbounded_send(ScriptCommand::Reload);
    }

    /// Loads the scripts again over `theme`.
    pub fn use_theme(&self, theme: ThemeChoice) {
        let _ = self.commands.unbounded_send(ScriptCommand::UseTheme(theme));
    }

    pub fn themes(&self) -> &[String] {
        &self.themes
    }

    pub fn load(&self) -> &LoadStatus {
        &self.load
    }

    pub fn log(&self) -> &ScriptLog {
        &self.log
    }
}

/// `script_status` as MCP returns it: the load's outcome, the log file and its latest lines, oldest first.
pub fn script_status_json(state: &ScriptState) -> serde_json::Value {
    let (load, bindings, error) = match state.load() {
        LoadStatus::NoInit => ("no-init", None, None),
        LoadStatus::Loaded { bindings } => ("loaded", Some(*bindings), None),
        LoadStatus::Failed { message, .. } => ("failed", None, Some(message.as_str())),
    };
    let log = state.log();
    serde_json::json!({ "load": load, "bindings": bindings, "error": error, "log_file": log.path(), "log": log.tail(STATUS_LOG_LINES) })
}

/// The action bound to `event` pressed at `place`; `None` when no binding takes the key. A held key repeats `Send`
/// and built-in actions but runs a function only once: its repeats do nothing.
pub fn bound_action(event: &KeyDownEvent, place: Place, cx: &App) -> Option<Action> {
    let action = cx.try_global::<ScriptState>()?.find_key(&event.keystroke, place)?;
    match action {
        Action::Invoke { .. } if event.is_held => Some(Action::Builtin(Builtin::Nop)),
        action => Some(action),
    }
}

/// The bindings in effect; `None` before the scripts' state exists.
pub fn keymap(cx: &App) -> Option<&Keymap> {
    cx.try_global::<ScriptState>().map(|state| &state.keymap)
}

/// The action bound to a mouse gesture in `scope`; `None` when no binding takes it.
pub fn bound_mouse(scope: Scope, trigger: &MouseTrigger, cx: &App) -> Option<Action> {
    cx.try_global::<ScriptState>()?.find_mouse(scope, trigger)
}

/// Asks the script thread to run a bound function on `context` (the shown tab when `None`).
pub fn invoke(generation: u64, function: usize, context: Option<serde_json::Value>, cx: &App) {
    if let Some(state) = cx.try_global::<ScriptState>() {
        state.invoke(generation, function, context);
    }
}

#[cfg(test)]
mod tests {
    use futures::channel::mpsc::unbounded;
    use gpui::Keystroke;
    use tempfile::TempDir;

    use super::*;
    use crate::script::keymap::{Action, Binding, Scope, Trigger};
    use crate::script::temporary_tabs::{TabMarker, TemporaryTabs};
    use crate::theme::Fonts;

    fn send_binding(key: &str) -> Keymap {
        let binding = Binding { trigger: Trigger::Key(Keystroke::parse(key).unwrap()), scope: Scope::Window, action: Action::Send("x".to_owned()) };
        Keymap::new(vec![binding]).unwrap()
    }

    fn loaded(keymap: Keymap) -> ScriptEvent {
        let temporary_tabs = TemporaryTabs { enabled: true, kinds: Vec::new(), marker: TabMarker::Italic };
        ScriptEvent::Loaded {
            keymap,
            fonts: Fonts::default(),
            theme: Box::new(crate::theme_config::defaults()),
            excluded: Default::default(),
            temporary_tabs,
            menus: Default::default(),
            quit_countdown: crate::script::quit_countdown::parse(&serde_json::from_value(serde_json::json!(10)).unwrap()).unwrap(),
            open_folders: Default::default(),
            has_init: true,
            themes: Vec::new(),
        }
    }

    fn new_state() -> (TempDir, ScriptState) {
        let dir = tempfile::tempdir().unwrap();
        let log = ScriptLog::new(dir.path().join("scripts.log"), 64 * 1024);
        (dir, ScriptState::new(unbounded().0, log))
    }

    #[test]
    fn should_keep_keymap_when_load_fails() {
        // given
        let (_dir, mut state) = new_state();
        state.apply(loaded(send_binding("f5")));

        // when
        state.apply(ScriptEvent::LoadFailed { message: "init.rn:1:1: broken".to_owned(), errors: 1, themes: Vec::new() });

        // then
        assert_eq!(state.find_key(&Keystroke::parse("f5").unwrap(), Place::Elsewhere), Some(Action::Send("x".to_owned())));
        assert!(matches!(state.load(), LoadStatus::Failed { errors: 1, .. }));
    }

    #[test]
    fn should_describe_failed_load() {
        // given
        let (_dir, mut state) = new_state();
        state.apply(ScriptEvent::LoadFailed { message: "init.rn:2:19: broken".to_owned(), errors: 1, themes: Vec::new() });

        // when
        let status = script_status_json(&state);

        // then
        assert_eq!(status["load"], "failed");
        assert_eq!(status["error"], "init.rn:2:19: broken");
        assert_eq!(status["bindings"], serde_json::Value::Null);
        assert!(status["log_file"].as_str().unwrap().ends_with("scripts.log"));
    }

    #[test]
    fn should_describe_no_init() {
        // given
        let (_dir, state) = new_state();

        // when
        let status = script_status_json(&state);

        // then
        assert_eq!(status["load"], "no-init");
    }

    #[test]
    fn should_return_last_50_log_lines() {
        // given
        let (_dir, state) = new_state();
        for line in 0..60 {
            state.log().append(&format!("line {line}"));
        }

        // when
        let status = script_status_json(&state);

        // then
        let log = status["log"].as_array().unwrap();
        assert_eq!(log.len(), 50);
        assert!(log[0].as_str().unwrap().ends_with("line 10"));
        assert!(log[49].as_str().unwrap().ends_with("line 59"));
    }

    #[test]
    fn should_keep_theme_names_from_load() {
        // given
        let (_dir, mut state) = new_state();
        let event = match loaded(send_binding("f5")) {
            ScriptEvent::Loaded { keymap, fonts, theme, excluded, temporary_tabs, menus, quit_countdown, open_folders, has_init, .. } => {
                let themes = vec!["a".to_owned(), "b".to_owned()];
                ScriptEvent::Loaded { keymap, fonts, theme, excluded, temporary_tabs, menus, quit_countdown, open_folders, has_init, themes }
            }
            event => event,
        };

        // when
        state.apply(event);

        // then
        assert_eq!(state.themes(), ["a", "b"]);
    }

    #[test]
    fn should_count_bindings_of_a_load() {
        // given
        let (_dir, mut state) = new_state();

        // when
        state.apply(loaded(send_binding("f5")));

        // then
        assert!(matches!(state.load(), LoadStatus::Loaded { bindings: 1 }));
    }
}
