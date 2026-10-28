//! The script thread: owns the Rune engine and every Rune value, loads and reloads `init.rn`, runs bound functions
//! as tasks so a slow one does not hold up the next, and writes what happens to the script log.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::rc::Rc;

use futures::StreamExt;
use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender, unbounded};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context as TaskContext, Poll};

use rune::runtime::{Function, Future as RuneFuture, VmError, budget};
use rune::Value;
use rune::alloc::clone::TryClone;

use super::defaults_file;
use super::engine::{Engine, Loaded, describe, is_out_of_budget};
use super::excluded::Excluded;
use super::open_folders::OpenFolders;
use super::quit_countdown::QuitCountdown;
use super::fingerprint::fingerprint;
use super::keymap::Keymap;
use super::menus::Menus;
use super::temporary_tabs::TemporaryTabs;
use crate::theme::Fonts;
use crate::settings::ThemeChoice;
use crate::theme_config::{self, ThemeSpec, user};
use super::log::ScriptLog;
use crate::api::{self, Call, Request};

/// What the script thread tells the gpui thread.
#[derive(Debug)]
pub enum ScriptEvent {
    /// `ive.rn` and `init.rn` (when `has_init`) loaded; these bindings and fonts replace the previous ones
    Loaded {
        keymap: Keymap,
        fonts: Fonts,
        theme: Box<ThemeSpec>,
        excluded: Excluded,
        hidden: super::hidden::Hidden,
        temporary_tabs: TemporaryTabs,
        menus: Box<Menus>,
        quit_countdown: QuitCountdown,
        open_folders: OpenFolders,
        drop_modifiers: crate::transfer::DropModifiers,
        delete: super::delete_mode::DeleteMode,
        has_init: bool,
        /// the user themes to pick from, by name
        themes: Vec<String>,
    },
    /// the load failed (the log says why); the previous bindings stay
    LoadFailed { message: String, errors: usize, themes: Vec<String> },
    /// a bound function failed; the log says why
    RunFailed,
    /// the menu condition asked as `request` answered
    Tested { request: u64, answer: bool },
}

/// What the gpui thread asks of the script thread.
#[derive(Debug)]
pub enum ScriptCommand {
    /// the scripts folder changed: load again when a `.rn` file or a theme did
    Reload,
    /// the user picked this theme: load again with it
    UseTheme(ThemeChoice),
    /// a trigger bound to function `function` of load `generation` fired; `context` is what it fired on, `None`
    /// for the shown tab
    Invoke { generation: u64, function: usize, context: Option<serde_json::Value> },
    /// a menu opened on `context`: does the condition function `function` of load `generation` hold? Answered as
    /// [`ScriptEvent::Tested`], false when it errs or takes longer than `timeout`
    Test { generation: u64, function: usize, context: serde_json::Value, request: u64, timeout: Duration },
}

/// The gpui thread's ends of the script thread's channels, and the log it writes.
pub struct Scripts {
    pub events: UnboundedReceiver<ScriptEvent>,
    pub commands: UnboundedSender<ScriptCommand>,
    pub log: ScriptLog,
}

/// Where the scripts are, and the theme picked at start.
pub struct Setup {
    /// the profile folder; none when the OS reports no config folder
    pub dir: Option<PathBuf>,
    pub theme: ThemeChoice,
}

/// Starts the script thread on `setup`, writing to `log`, and loads `init.rn` once at start. The copies of `ive.rn`
/// and `theme.toml` are written before it returns, so tabs restored next show them.
pub fn start(setup: Setup, log: ScriptLog, calls: UnboundedSender<Call>) -> Scripts {
    let Setup { dir, theme } = setup;
    write_defaults(dir.as_deref(), &log);
    let (events, event_receiver) = unbounded();
    let (commands, command_receiver) = unbounded();
    let thread = Thread { dir, theme, log: log.clone(), calls, events };
    let spawned = std::thread::Builder::new().name("ive-scripts".to_owned()).spawn(move || run(thread, command_receiver));
    if let Err(error) = spawned {
        eprintln!("ive: cannot start the script thread: {error}; scripts are off");
    }
    Scripts { events: event_receiver, commands, log }
}

/// What the script thread is given.
struct Thread {
    dir: Option<PathBuf>,
    theme: ThemeChoice,
    log: ScriptLog,
    calls: UnboundedSender<Call>,
    events: UnboundedSender<ScriptEvent>,
}

fn run(thread: Thread, commands: UnboundedReceiver<ScriptCommand>) {
    let started = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| format!("cannot start the script runtime: {error}"))
        .and_then(|runtime| Ok((runtime, Engine::new(thread.calls.clone())?)));
    let (runtime, engine) = match started {
        Ok(started) => started,
        // a load that could not happen; the bindings stay empty
        Err(message) => return fail_start(&thread, message),
    };
    let Thread { dir, theme, log, calls, events } = thread;
    let mut runner = Runner { dir, theme, engine: Rc::new(engine), log, calls, events, current: None, fingerprint: BTreeMap::new(), loads: 0 };
    let local = tokio::task::LocalSet::new();
    local.block_on(&runtime, runner.serve(commands));
}

/// Writes `ive.rn` and `theme.toml` for the user to read, once per run: a reload leaves them, and any edit of them,
/// alone.
fn write_defaults(dir: Option<&Path>, log: &ScriptLog) {
    let Some(dir) = dir else {
        return log.append("the OS reports no config folder, so only IVE's default bindings apply");
    };
    if let Err(error) = defaults_file::write(dir) {
        log.append(&error);
    }
}

fn fail_start(thread: &Thread, message: String) {
    let message = format!("scripts are off: {message}");
    thread.log.append(&message);
    let _ = thread.events.unbounded_send(ScriptEvent::LoadFailed { message, errors: 1, themes: Vec::new() });
}

/// The loaded scripts and what the thread needs to load them again.
struct Runner {
    dir: Option<PathBuf>,
    /// the picked theme, under `ive.rn` and `init.rn`
    theme: ThemeChoice,
    engine: Rc<Engine>,
    log: ScriptLog,
    calls: UnboundedSender<Call>,
    events: UnboundedSender<ScriptEvent>,
    /// the load in effect and its generation; a failed reload keeps it
    current: Option<(u64, Loaded)>,
    fingerprint: BTreeMap<String, u64>,
    loads: u64,
}

impl Runner {
    async fn serve(&mut self, mut commands: UnboundedReceiver<ScriptCommand>) {
        self.load();
        while let Some(command) = commands.next().await {
            match command {
                ScriptCommand::Reload => self.reload(),
                ScriptCommand::UseTheme(theme) => {
                    self.theme = theme;
                    self.load();
                }
                ScriptCommand::Invoke { generation, function, context } => self.invoke(generation, function, context),
                ScriptCommand::Test { generation, function, context, request, timeout } => self.test(Asked { generation, function, request, timeout }, context),
            }
        }
    }

    fn reload(&mut self) {
        let unchanged = self.dir.as_deref().map(fingerprint).is_some_and(|now| now == self.fingerprint);
        if unchanged {
            return;
        }
        self.load();
    }

    fn load(&mut self) {
        let dir = self.dir.clone();
        if let Some(dir) = &dir {
            self.fingerprint = fingerprint(dir);
        }
        self.loads += 1;
        match self.theme_base(dir.as_deref()) {
            Ok(theme) => self.load_with(dir.as_deref(), theme),
            Err(message) => self.theme_failed(dir.as_deref(), message),
        }
    }

    fn load_with(&mut self, dir: Option<&Path>, theme: Value) {
        let loaded = self.engine.load(dir, theme, self.loads);
        // what init printed comes before what came of it
        self.drain_output();
        match loaded {
            Ok(loaded) => self.apply(loaded),
            Err(message) => self.fail(message),
        }
    }

    /// A broken theme fails the load; at start there is nothing to keep, so the scripts load over the default theme,
    /// and when they fail too, one failure names both.
    fn theme_failed(&mut self, dir: Option<&Path>, message: String) {
        if self.current.is_some() {
            return self.fail(message);
        }
        let loaded = self.engine.load(dir, theme_config::default_value(), self.loads);
        self.drain_output();
        match loaded {
            Ok(loaded) => {
                self.apply(loaded);
                self.fail(message);
            }
            Err(scripts_error) => self.fail(format!("{message}\n{scripts_error}")),
        }
    }

    /// Tells the gpui thread the load failed, with the themes there are now, so one can be picked to get out of it.
    fn send_failed(&self, message: String) {
        let themes = self.dir.as_deref().map(user::names).unwrap_or_default();
        self.send(ScriptEvent::LoadFailed { errors: message.lines().count(), message, themes });
    }

    /// `config.theme` before the scripts run: the default with the picked theme merged over it.
    fn theme_base(&self, dir: Option<&Path>) -> Result<Value, String> {
        let Some(dir) = dir else {
            return Ok(theme_config::default_value());
        };
        let base = user::base(dir, &self.theme)?;
        if let (user::Base::Missing, ThemeChoice::File(name)) = (&base, &self.theme) {
            self.log.append(&format!("theme \"{name}\": no file {}/{name}.toml; using the default", user::THEMES_DIR));
        }
        Ok(base.value())
    }

    fn apply(&mut self, loaded: Loaded) {
        if loaded.has_init {
            self.log.append(&format!("init.rn loaded: {} bindings", loaded.keymap.len()));
        }
        let event = ScriptEvent::Loaded {
            keymap: loaded.keymap.clone(),
            fonts: loaded.fonts.clone(),
            theme: Box::new(loaded.theme.clone()),
            excluded: loaded.excluded.clone(),
            hidden: loaded.hidden.clone(),
            temporary_tabs: loaded.temporary_tabs.clone(),
            menus: Box::new(loaded.menus.clone()),
            quit_countdown: loaded.quit_countdown,
            open_folders: loaded.open_folders.clone(),
            drop_modifiers: loaded.drop_modifiers,
            delete: loaded.delete,
            has_init: loaded.has_init,
            themes: self.dir.as_deref().map(user::names).unwrap_or_default(),
        };
        self.current = Some((self.loads, loaded));
        self.send(event);
    }

    /// A failed load keeps the bindings in effect; at start there are none yet, so `ive.rn`'s alone apply, over the
    /// picked theme when it loads.
    fn fail(&mut self, message: String) {
        self.log.append(&message);
        if self.current.is_none() {
            let theme = self.theme_base(self.dir.clone().as_deref()).unwrap_or_else(|_| theme_config::default_value());
            match self.engine.load(None, theme, self.loads) {
                Ok(defaults) => self.apply(defaults),
                Err(defaults_error) => self.log.append(&format!("IVE's own ive.rn did not load, so there are no bindings: {defaults_error}")),
            }
        }
        self.send_failed(message);
    }

    fn invoke(&self, generation: u64, function: usize, context: Option<serde_json::Value>) {
        let bound = self.current.as_ref().filter(|(current, _)| *current == generation).and_then(|(_, loaded)| {
            let key = loaded.keymap.invoked(function).or_else(|| loaded.menus.function_label(function))?;
            Some((key, loaded.functions.get(function)?.try_clone().ok()?))
        });
        let Some((key, function)) = bound else {
            return self.log.append("ignored key from an older script load");
        };
        let (calls, events, engine, log) = (self.calls.clone(), self.events.clone(), self.engine.clone(), self.log.clone());
        tokio::task::spawn_local(async move {
            let result = call_bound(&function, &calls, context).await;
            for line in engine.drain_output() {
                log.append(&line);
            }
            if let Err(error) = result {
                log.append(&format!("{key}: {error}"));
                let _ = events.unbounded_send(ScriptEvent::RunFailed);
            }
        });
    }

    /// Runs a menu condition as a task of its own and sends its answer; one of an older load answers false.
    fn test(&self, asked: Asked, context: serde_json::Value) {
        let bound = self.current.as_ref().filter(|(current, _)| *current == asked.generation).and_then(|(_, loaded)| {
            let label = loaded.menus.function_label(asked.function)?;
            Some((label, loaded.functions.get(asked.function)?.try_clone().ok()?))
        });
        let Some((label, function)) = bound else {
            return self.send(ScriptEvent::Tested { request: asked.request, answer: false });
        };
        let (events, engine, log) = (self.events.clone(), self.engine.clone(), self.log.clone());
        tokio::task::spawn_local(async move {
            let answer = tokio::time::timeout(asked.timeout, call_test(&function, context)).await;
            for line in engine.drain_output() {
                log.append(&line);
            }
            let answer = match answer {
                Ok(Ok(answer)) => answer,
                Ok(Err(error)) => {
                    log.append(&format!("{label}: {error}"));
                    false
                }
                Err(_) => {
                    log.append(&format!("{label}: did not answer within {} ms, so the item stays hidden", asked.timeout.as_millis()));
                    false
                }
            };
            let _ = events.unbounded_send(ScriptEvent::Tested { request: asked.request, answer });
        });
    }

    fn drain_output(&self) {
        for line in self.engine.drain_output() {
            self.log.append(&line);
        }
    }

    fn send(&self, event: ScriptEvent) {
        // the gpui side is gone only while IVE quits
        let _ = self.events.unbounded_send(event);
    }
}

/// A bound function runs this many Rune instructions between two awaits at most: a loop that never awaits is
/// stopped, so it cannot hold the script thread (and every later key and reload) forever, while one that awaits
/// (a watcher sleeping between polls) runs as long as it likes.
const STEP_BUDGET: usize = 50_000_000;

/// Calls a bound function with `context` (the shown tab when `None`), awaiting it when async; its `Err` is the error.
async fn call_bound(function: &Function, calls: &UnboundedSender<Call>, context: Option<serde_json::Value>) -> Result<(), String> {
    let context = match context {
        Some(context) => Some(context),
        None => api::ask(calls, Request::ActiveTab).await.ok(),
    };
    let context = context.and_then(|context| serde_json::from_value::<Value>(context).ok());
    let context = context.unwrap_or_else(Value::empty);
    let returned = budget::with(STEP_BUDGET, || function.call::<Value>((context,))).call().into_result().map_err(run_error)?;
    let returned = match rune::from_value::<RuneFuture>(returned.clone()) {
        Ok(future) => PerPoll(future).await.into_result().map_err(run_error)?,
        Err(_) => returned,
    };
    match rune::from_value::<Result<Value, Value>>(returned) {
        Ok(Err(error)) => Err(describe(error)),
        _ => Ok(()),
    }
}

/// Which menu condition to ask, and for how long.
struct Asked {
    generation: u64,
    function: usize,
    request: u64,
    timeout: Duration,
}

/// Calls a menu condition with `context`, awaiting it when async: `true` or `Ok(true)` holds, any other value does not;
/// a run-time error or an `Err` is the error.
async fn call_test(function: &Function, context: serde_json::Value) -> Result<bool, String> {
    let context = serde_json::from_value::<Value>(context).unwrap_or_else(|_| Value::empty());
    let returned = budget::with(STEP_BUDGET, || function.call::<Value>((context,))).call().into_result().map_err(run_error)?;
    let returned = match rune::from_value::<RuneFuture>(returned.clone()) {
        Ok(future) => PerPoll(future).await.into_result().map_err(run_error)?,
        Err(_) => returned,
    };
    match rune::from_value::<Result<Value, Value>>(returned.clone()) {
        Ok(Ok(value)) => Ok(rune::from_value::<bool>(value).unwrap_or(false)),
        Ok(Err(error)) => Err(describe(error)),
        Err(_) => Ok(rune::from_value::<bool>(returned).unwrap_or(false)),
    }
}

fn run_error(error: VmError) -> String {
    let error = error.to_string();
    if is_out_of_budget(&error) {
        return format!("ran over {STEP_BUDGET} instructions without awaiting and was stopped; does it loop forever?");
    }
    error
}

/// A future whose every poll gets the full [`STEP_BUDGET`]; Rune's own budget would count across polls.
struct PerPoll<F>(F);

impl<F: Future + Unpin> Future for PerPoll<F> {
    type Output = F::Output;

    fn poll(mut self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<F::Output> {
        let inner = &mut self.0;
        budget::with(STEP_BUDGET, || Pin::new(inner).poll(cx)).call()
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};

    use futures::StreamExt;
    use futures::channel::mpsc::{UnboundedReceiver, unbounded};
    use futures::executor::block_on;
    use serde_json::json;

    use super::*;
    use crate::script::keymap::Place;
    use crate::api::{Call, Reply, Request, WindowInput};
    use crate::theme_config::Rgb;
    use crate::theme_config::user::THEMES_DIR;

    const WAIT: Duration = Duration::from_secs(5);

    /// Answers the script thread's calls as `answer` says and records each request.
    fn responder(answer: fn(&Request) -> Reply) -> (UnboundedSender<Call>, Arc<Mutex<Vec<String>>>) {
        let (calls, mut received): (UnboundedSender<Call>, UnboundedReceiver<Call>) = unbounded();
        let seen = Arc::new(Mutex::new(Vec::new()));
        let recorded = seen.clone();
        std::thread::spawn(move || {
            while let Some(call) = block_on(received.next()) {
                recorded.lock().unwrap().push(format!("{:?}", call.request));
                let _ = call.reply.send(answer(&call.request));
            }
        });
        (calls, seen)
    }

    fn answer_terminal(request: &Request) -> Reply {
        match request {
            Request::ActiveTab => Ok(json!({ "tab": 1, "kind": "process", "checkout": "/p" })),
            _ => Ok(json!("ok")),
        }
    }

    fn answer_no_tab(request: &Request) -> Reply {
        match request {
            Request::ActiveTab => Ok(json!({ "tab": null, "kind": null, "checkout": null })),
            Request::SendToActive { .. } => Err("no tab is shown".to_owned()),
            _ => Ok(json!("ok")),
        }
    }

    fn next_event(scripts: &mut Scripts, within: Duration) -> Option<ScriptEvent> {
        let deadline = Instant::now() + within;
        while Instant::now() < deadline {
            if let Ok(event) = scripts.events.try_recv() {
                return Some(event);
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        None
    }

    /// The next event that is not a run-time failure.
    fn next_load(scripts: &mut Scripts) -> ScriptEvent {
        loop {
            match next_event(scripts, WAIT).expect("no load event") {
                ScriptEvent::RunFailed => continue,
                event => return event,
            }
        }
    }

    /// The first log line containing `needle`, waiting for it to be written.
    fn logged(log: &ScriptLog, needle: &str) -> String {
        let deadline = Instant::now() + WAIT;
        while Instant::now() < deadline {
            if let Some(line) = log.tail(100).into_iter().find(|line| line.contains(needle)) {
                return line;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        panic!("no log line with {needle:?}: {:?}", log.tail(100));
    }

    /// IVE writes `ive.rn` read-only; a user who edits it makes it writable first.
    fn make_writable(path: &Path) {
        let mut permissions = fs::metadata(path).unwrap().permissions();
        #[cfg(unix)]
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o644);
        #[cfg(windows)]
        #[allow(clippy::permissions_set_readonly_false)]
        permissions.set_readonly(false);
        fs::set_permissions(path, permissions).unwrap();
    }

    fn log_in(dir: &Path) -> ScriptLog {
        ScriptLog::new(dir.join("log").join("scripts.log"), 64 * 1024)
    }

    fn builtin(dir: &Path) -> Setup {
        Setup { dir: Some(dir.to_path_buf()), theme: ThemeChoice::Builtin }
    }

    fn start_with(dir: &Path, init: &str, answer: fn(&Request) -> Reply) -> (Scripts, Arc<Mutex<Vec<String>>>, ScriptLog) {
        fs::write(dir.join("init.rn"), init).unwrap();
        let (calls, seen) = responder(answer);
        let log = log_in(dir);
        let mut scripts = start(builtin(dir), log.clone(), calls);
        assert!(matches!(next_load(&mut scripts), ScriptEvent::Loaded { .. }));
        (scripts, seen, log)
    }

    /// Runs the first function `init.rn` binds; `ive.rn`'s own come before it.
    fn invoke(scripts: &Scripts, generation: u64) {
        invoke_function(scripts, generation, defaults_functions());
    }

    fn invoke_function(scripts: &Scripts, generation: u64, function: usize) {
        scripts.commands.unbounded_send(ScriptCommand::Invoke { generation, function, context: None }).unwrap();
    }

    /// Asks the first function `init.rn` names whether it holds, with `timeout`, and waits for the answer.
    fn ask_test(scripts: &mut Scripts, generation: u64, timeout: Duration) -> bool {
        let command = ScriptCommand::Test { generation, function: defaults_functions(), context: json!({ "paths": [] }), request: 7, timeout };
        scripts.commands.unbounded_send(command).unwrap();
        loop {
            match next_event(scripts, WAIT).expect("no answer") {
                ScriptEvent::Tested { request: 7, answer } => return answer,
                _ => continue,
            }
        }
    }

    fn when_test(body: &str) -> String {
        format!("pub fn init(config) {{ config.tree_menus.main = [#{{ label: \"X\", action: act::CopyFullPath, when: check }}]; }}\n{body}")
    }

    fn defaults_functions() -> usize {
        super::super::engine::Engine::new(unbounded().0).unwrap().load(None, crate::theme_config::default_value(), 1).unwrap().functions.len()
    }

    fn wait_for(seen: &Arc<Mutex<Vec<String>>>, needle: &str) -> bool {
        let deadline = Instant::now() + WAIT;
        while Instant::now() < deadline {
            if seen.lock().unwrap().iter().any(|request| request.contains(needle)) {
                return true;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        false
    }

    const CLICK_RESTART: &str = "pub fn init(config) { config.keys = [#{ key: \"r\", mods: \"CTRL|ALT\", action: restart }]; }\n\
        async fn restart(context) { ive::click(\"restart\").await?; Ok(()) }";

    #[test]
    fn should_not_reload_when_no_rn_file_changed() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let (mut scripts, _, _) = start_with(dir.path(), "pub fn init(config) {}", answer_terminal);

        // when
        fs::write(dir.path().join("state.toml"), "version = 2").unwrap();
        scripts.commands.unbounded_send(ScriptCommand::Reload).unwrap();

        // then
        assert!(next_event(&mut scripts, Duration::from_millis(500)).is_none());
    }

    #[test]
    fn should_reload_changed_init() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let (mut scripts, _, _) = start_with(dir.path(), r#"pub fn init(config) { config.keys = [#{ key: "F5", action: act::SendString("a") }]; }"#, answer_terminal);

        // when
        fs::write(dir.path().join("init.rn"), r#"pub fn init(config) { config.keys = [#{ key: "F5", action: act::SendString("a") }, #{ key: "F6", action: act::SendString("b") }]; }"#).unwrap();
        scripts.commands.unbounded_send(ScriptCommand::Reload).unwrap();

        // then
        match next_load(&mut scripts) {
            ScriptEvent::Loaded { keymap, .. } => assert!(keymap.find_key(&gpui::Keystroke::parse("f6").unwrap(), Place::Elsewhere).is_some()),
            _ => panic!("reload failed"),
        }
    }

    #[test]
    fn should_keep_previous_load_when_reload_fails() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let (mut scripts, seen, _) = start_with(dir.path(), CLICK_RESTART, answer_terminal);
        fs::write(dir.path().join("init.rn"), "pub fn init(config) {").unwrap();
        scripts.commands.unbounded_send(ScriptCommand::Reload).unwrap();
        assert!(matches!(next_load(&mut scripts), ScriptEvent::LoadFailed { .. }));

        // when
        invoke(&scripts, 1);

        // then
        assert!(wait_for(&seen, "Click(\"restart\")"));
    }

    #[test]
    fn should_send_theme_with_load() {
        // given
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("init.rn"), r##"pub fn init(config) { config.theme.dark.tree = "#202124"; }"##).unwrap();
        let (calls, _) = responder(answer_terminal);

        // when
        let mut scripts = start(builtin(dir.path()), log_in(dir.path()), calls);

        // then
        match next_load(&mut scripts) {
            ScriptEvent::Loaded { theme, .. } => assert_eq!(theme.dark.tree, crate::theme_config::Rgb(0x202124)),
            _ => panic!("load failed"),
        }
    }

    fn start_themed(dir: &Path, theme: &str) -> Scripts {
        let (calls, _) = responder(answer_terminal);
        let setup = Setup { dir: Some(dir.to_path_buf()), theme: ThemeChoice::File(theme.to_owned()) };
        start(setup, log_in(dir), calls)
    }

    fn write_theme(dir: &Path, name: &str, text: &str) {
        fs::create_dir_all(dir.join(THEMES_DIR)).unwrap();
        fs::write(dir.join(THEMES_DIR).join(format!("{name}.toml")), text).unwrap();
    }

    fn loaded_tree(event: ScriptEvent) -> Rgb {
        match event {
            ScriptEvent::Loaded { theme, .. } => theme.dark.tree,
            event => panic!("expected a load, got {event:?}"),
        }
    }

    const NORD: &str = "[dark]\ntree = \"#112233\"\n";
    const BROKEN: &str = "[dark]\ntre = \"#000000\"\n";
    const DEFAULT_TREE: Rgb = Rgb(0x191a1c);

    #[test]
    fn should_start_with_picked_theme() {
        // given
        let dir = tempfile::tempdir().unwrap();
        write_theme(dir.path(), "nord", NORD);

        // when
        let mut scripts = start_themed(dir.path(), "nord");

        // then
        match next_load(&mut scripts) {
            ScriptEvent::Loaded { theme, themes, .. } => {
                assert_eq!(theme.dark.tree, Rgb(0x112233));
                assert_eq!(themes, ["nord"]);
            }
            event => panic!("expected a load, got {event:?}"),
        }
    }

    #[test]
    fn should_let_init_override_picked_theme() {
        // given
        let dir = tempfile::tempdir().unwrap();
        write_theme(dir.path(), "nord", NORD);
        fs::write(dir.path().join("init.rn"), r##"pub fn init(config) { config.theme.dark.tree = "#202124"; }"##).unwrap();

        // when
        let mut scripts = start_themed(dir.path(), "nord");

        // then
        assert_eq!(loaded_tree(next_load(&mut scripts)), Rgb(0x202124));
    }

    #[test]
    fn should_keep_init_bindings_when_theme_broken_at_start() {
        // given
        let dir = tempfile::tempdir().unwrap();
        write_theme(dir.path(), "nord", BROKEN);
        fs::write(dir.path().join("init.rn"), r#"pub fn init(config) { config.keys = [#{ key: "F5", action: act::SendString("a") }]; }"#).unwrap();

        // when
        let mut scripts = start_themed(dir.path(), "nord");

        // then
        match next_load(&mut scripts) {
            ScriptEvent::Loaded { keymap, theme, .. } => {
                assert!(keymap.find_key(&gpui::Keystroke::parse("f5").unwrap(), Place::Elsewhere).is_some());
                assert_eq!(theme.dark.tree, DEFAULT_TREE);
            }
            event => panic!("expected a load, got {event:?}"),
        }
        match next_load(&mut scripts) {
            ScriptEvent::LoadFailed { message, .. } => assert!(message.starts_with("themes/nord.toml: "), "{message}"),
            event => panic!("expected the theme's failure, got {event:?}"),
        }
    }

    #[test]
    fn should_keep_previous_theme_when_theme_breaks_on_reload() {
        // given
        let dir = tempfile::tempdir().unwrap();
        write_theme(dir.path(), "nord", NORD);
        let mut scripts = start_themed(dir.path(), "nord");
        assert_eq!(loaded_tree(next_load(&mut scripts)), Rgb(0x112233));

        // when
        write_theme(dir.path(), "nord", BROKEN);
        scripts.commands.unbounded_send(ScriptCommand::Reload).unwrap();

        // then
        assert!(matches!(next_load(&mut scripts), ScriptEvent::LoadFailed { .. }));
        assert!(next_event(&mut scripts, Duration::from_millis(500)).is_none());
    }

    #[test]
    fn should_load_picked_theme_on_use_theme() {
        // given
        let dir = tempfile::tempdir().unwrap();
        write_theme(dir.path(), "nord", NORD);
        let (calls, _) = responder(answer_terminal);
        let mut scripts = start(builtin(dir.path()), log_in(dir.path()), calls);
        assert_eq!(loaded_tree(next_load(&mut scripts)), DEFAULT_TREE);

        // when
        scripts.commands.unbounded_send(ScriptCommand::UseTheme(ThemeChoice::File("nord".to_owned()))).unwrap();

        // then
        assert_eq!(loaded_tree(next_load(&mut scripts)), Rgb(0x112233));
    }

    #[test]
    fn should_use_default_when_picked_theme_missing() {
        // given
        let dir = tempfile::tempdir().unwrap();

        // when
        let mut scripts = start_themed(dir.path(), "gone");

        // then
        assert_eq!(loaded_tree(next_load(&mut scripts)), DEFAULT_TREE);
        logged(&log_in(dir.path()), "theme \"gone\": no file themes/gone.toml; using the default");
    }

    #[test]
    fn should_report_both_errors_when_theme_and_init_broken_at_start() {
        // given
        let dir = tempfile::tempdir().unwrap();
        write_theme(dir.path(), "nord", BROKEN);
        fs::write(dir.path().join("init.rn"), "pub fn init(config) {").unwrap();

        // when
        let mut scripts = start_themed(dir.path(), "nord");

        // then
        assert!(matches!(next_load(&mut scripts), ScriptEvent::Loaded { .. }));
        match next_load(&mut scripts) {
            ScriptEvent::LoadFailed { message, .. } => assert!(message.contains("themes/nord.toml: ") && message.contains("init.rn"), "{message}"),
            event => panic!("expected the failure, got {event:?}"),
        }
        assert!(next_event(&mut scripts, Duration::from_millis(500)).is_none());
    }

    #[test]
    fn should_list_themes_while_load_fails() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let (calls, _) = responder(answer_terminal);
        fs::write(dir.path().join("init.rn"), "pub fn init(config) {").unwrap();
        let mut scripts = start(builtin(dir.path()), log_in(dir.path()), calls);
        assert!(matches!(next_load(&mut scripts), ScriptEvent::Loaded { .. }));
        assert!(matches!(next_load(&mut scripts), ScriptEvent::LoadFailed { .. }));

        // when
        write_theme(dir.path(), "fixed", NORD);
        scripts.commands.unbounded_send(ScriptCommand::Reload).unwrap();

        // then
        match next_load(&mut scripts) {
            ScriptEvent::LoadFailed { themes, .. } => assert_eq!(themes, ["fixed"]),
            event => panic!("expected the failure, got {event:?}"),
        }
    }

    #[test]
    fn should_keep_previous_theme_when_init_indexes_missing_table() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let (mut scripts, _, _) = start_with(dir.path(), r##"pub fn init(config) { config.theme.dark.tree = "#202124"; }"##, answer_terminal);
        fs::write(dir.path().join("init.rn"), "pub fn init(config) { config.theme.sizes.tre.row_height = 1.0; }").unwrap();

        // when
        scripts.commands.unbounded_send(ScriptCommand::Reload).unwrap();

        // then
        match next_load(&mut scripts) {
            ScriptEvent::LoadFailed { message, .. } => assert!(message.starts_with("init.rn:1:"), "{message}"),
            _ => panic!("the broken reload loaded"),
        }
        assert!(next_event(&mut scripts, Duration::from_millis(500)).is_none());
    }

    #[test]
    fn should_ignore_invoke_from_an_older_load() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let (mut scripts, seen, log) = start_with(dir.path(), CLICK_RESTART, answer_terminal);
        fs::write(dir.path().join("init.rn"), CLICK_RESTART.replace(r#"key: "r""#, r#"key: "s""#)).unwrap();
        scripts.commands.unbounded_send(ScriptCommand::Reload).unwrap();
        assert!(matches!(next_load(&mut scripts), ScriptEvent::Loaded { .. }));

        // when
        invoke(&scripts, 1);

        // then
        logged(&log, "older script load");
        assert!(!seen.lock().unwrap().iter().any(|request| request.contains("Click")));
    }

    #[test]
    fn should_call_api_from_bound_function() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let (scripts, seen, _) = start_with(dir.path(), CLICK_RESTART, answer_terminal);

        // when
        invoke(&scripts, 1);

        // then
        let expected = format!("{:?}", Request::WindowInput(WindowInput::Click("restart".to_owned())));
        assert!(wait_for(&seen, &expected));
    }

    #[test]
    fn should_run_a_function_named_only_by_a_menu() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let init = "pub fn init(config) { config.tree_menus.main = [#{ label: \"Restart\", action: restart }]; }\n\
            async fn restart(context) { ive::click(\"restart\").await?; Ok(()) }";
        let (scripts, seen, _) = start_with(dir.path(), init, answer_terminal);

        // when
        invoke(&scripts, 1);

        // then
        let expected = format!("{:?}", Request::WindowInput(WindowInput::Click("restart".to_owned())));
        assert!(wait_for(&seen, &expected));
    }

    #[test]
    fn should_answer_a_sync_test_true() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let (mut scripts, _, _) = start_with(dir.path(), &when_test("fn check(context) { context.paths.len() == 0 }"), answer_terminal);

        // when
        let answer = ask_test(&mut scripts, 1, WAIT);

        // then
        assert!(answer);
    }

    #[test]
    fn should_answer_an_async_test_after_it_awaits() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let body = "async fn check(context) { time::sleep(time::Duration::from_millis(50)).await; Ok(true) }";
        let (mut scripts, _, _) = start_with(dir.path(), &when_test(body), answer_terminal);

        // when
        let answer = ask_test(&mut scripts, 1, WAIT);

        // then
        assert!(answer);
    }

    #[test]
    fn should_answer_false_and_log_when_a_test_errs() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let (mut scripts, _, log) = start_with(dir.path(), &when_test("fn check(context) { Err(\"no deploy key\") }"), answer_terminal);

        // when
        let answer = ask_test(&mut scripts, 1, WAIT);

        // then
        assert!(!answer);
        assert!(logged(&log, "no deploy key").contains("the condition of X"));
    }

    #[test]
    fn should_answer_false_after_the_timeout() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let body = "async fn check(context) { time::sleep(time::Duration::from_secs(60)).await; Ok(true) }";
        let (mut scripts, _, log) = start_with(dir.path(), &when_test(body), answer_terminal);

        // when
        let answer = ask_test(&mut scripts, 1, Duration::from_millis(100));

        // then
        assert!(!answer);
        assert!(logged(&log, "the condition of X").contains("did not answer"));
    }

    #[test]
    fn should_answer_false_for_an_older_generation() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let (mut scripts, _, _) = start_with(dir.path(), &when_test("fn check(context) { true }"), answer_terminal);

        // when
        let answer = ask_test(&mut scripts, 0, WAIT);

        // then
        assert!(!answer);
    }

    #[test]
    fn should_log_error_returned_by_bound_function() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let init = "pub fn init(config) { config.keys = [#{ key: \"x\", mods: \"CTRL|ALT\", action: send_x }]; }\n\
            async fn send_x(context) { ive::send_to_active(\"x\").await?; Ok(()) }";
        let (mut scripts, _, log) = start_with(dir.path(), init, answer_no_tab);

        // when
        invoke(&scripts, 1);

        // then
        assert!(matches!(next_event(&mut scripts, WAIT), Some(ScriptEvent::RunFailed)));
        assert!(logged(&log, "no tab is shown").contains("ctrl-alt-x"));
    }

    #[test]
    fn should_pass_active_tab_as_context() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let init = "pub fn init(config) { config.keys = [#{ key: \"F7\", action: show }]; }\nasync fn show(context) { Err(context.kind) }";
        let (scripts, _, log) = start_with(dir.path(), init, answer_terminal);

        // when
        invoke(&scripts, 1);

        // then
        logged(&log, "f7: process");
    }

    #[test]
    fn should_stop_bound_function_that_never_yields() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let init = "pub fn init(config) { config.keys = [#{ key: \"F8\", action: spin }]; }\nasync fn spin(context) { loop {} }";
        let (mut scripts, _, log) = start_with(dir.path(), init, answer_terminal);

        // when
        invoke(&scripts, 1);

        // then
        assert!(logged(&log, "instructions").contains("f8"));
        fs::write(dir.path().join("init.rn"), r#"pub fn init(config) { config.keys = [#{ key: "F5", action: act::SendString("a") }]; }"#).unwrap();
        scripts.commands.unbounded_send(ScriptCommand::Reload).unwrap();
        assert!(matches!(next_load(&mut scripts), ScriptEvent::Loaded { .. }));
    }

    fn has_open_terminal(event: &ScriptEvent) -> bool {
        let alt_f12 = gpui::Keystroke::parse("alt-f12").unwrap();
        matches!(event, ScriptEvent::Loaded { keymap, .. } if keymap.find_key(&alt_f12, Place::Elsewhere).is_some())
    }

    #[test]
    fn should_write_default_copies_before_start_returns() {
        // given
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(super::super::engine::IVE_RN_NAME), "// an older IVE's copy").unwrap();
        let (calls, _) = responder(answer_terminal);

        // when
        let _scripts = start(builtin(dir.path()), log_in(dir.path()), calls);

        // then
        let written = fs::read_to_string(dir.path().join(super::super::engine::IVE_RN_NAME)).unwrap();
        assert_eq!(written, super::super::engine::IVE_RN);
        assert!(dir.path().join(defaults_file::THEME_NAME).exists());
    }

    #[test]
    fn should_load_defaults_without_init() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let (calls, _) = responder(answer_terminal);

        // when
        let mut scripts = start(builtin(dir.path()), log_in(dir.path()), calls);

        // then
        let event = next_load(&mut scripts);
        assert!(matches!(event, ScriptEvent::Loaded { has_init: false, .. }));
        assert!(has_open_terminal(&event));
    }

    #[test]
    fn should_keep_defaults_when_init_fails_at_start() {
        // given
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("init.rn"), "pub fn init(config) {").unwrap();
        let (calls, _) = responder(answer_terminal);

        // when
        let mut scripts = start(builtin(dir.path()), log_in(dir.path()), calls);

        // then
        assert!(has_open_terminal(&next_load(&mut scripts)));
        assert!(matches!(next_load(&mut scripts), ScriptEvent::LoadFailed { .. }));
    }

    #[test]
    fn should_pass_given_context_to_function() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let init = "pub fn init(config) { config.keys = [#{ key: \"F7\", action: show }]; }\nasync fn show(context) { Err(context.row) }";
        let (scripts, _, log) = start_with(dir.path(), init, answer_terminal);

        // when
        let context = Some(serde_json::json!({ "row": "entry:x" }));
        scripts.commands.unbounded_send(ScriptCommand::Invoke { generation: 1, function: defaults_functions(), context }).unwrap();

        // then
        logged(&log, "entry:x");
    }

    #[test]
    fn should_write_ive_rn_next_to_init() {
        // given
        let dir = tempfile::tempdir().unwrap();

        // when
        let (_scripts, _, _) = start_with(dir.path(), "pub fn init(config) {}", answer_terminal);

        // then
        assert_eq!(fs::read_to_string(dir.path().join("ive.rn")).unwrap(), crate::script::engine::IVE_RN);
    }

    #[test]
    fn should_leave_edited_ive_rn_alone_on_reload() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let (mut scripts, _, _) = start_with(dir.path(), "pub fn init(config) {}", answer_terminal);
        let ive_rn = dir.path().join("ive.rn");
        make_writable(&ive_rn);
        fs::write(&ive_rn, "// the user's edit").unwrap();

        // when
        fs::write(dir.path().join("init.rn"), "pub fn init(config) { }").unwrap();
        scripts.commands.unbounded_send(ScriptCommand::Reload).unwrap();

        // then
        assert!(matches!(next_load(&mut scripts), ScriptEvent::Loaded { .. }));
        assert_eq!(fs::read_to_string(&ive_rn).unwrap(), "// the user's edit");
    }

    #[test]
    fn should_log_failed_load() {
        // given
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("init.rn"), "pub fn init(config) {\n    config.keys = [#{ key: \"F5\" action: act::Nop }];\n}").unwrap();
        let (calls, _) = responder(answer_terminal);
        let log = log_in(dir.path());

        // when
        let mut scripts = start(builtin(dir.path()), log.clone(), calls);

        // then
        assert!(matches!(next_load(&mut scripts), ScriptEvent::Loaded { has_init: false, .. }));
        assert!(matches!(next_load(&mut scripts), ScriptEvent::LoadFailed { errors: 1, .. }));
        logged(&log, "init.rn:2:");
    }

    #[test]
    fn should_log_printed_output() {
        // given
        let dir = tempfile::tempdir().unwrap();

        // when
        let (_scripts, _, log) = start_with(dir.path(), r#"pub fn init(config) { println!("hello from init"); }"#, answer_terminal);

        // then
        logged(&log, "hello from init");
    }

    #[test]
    fn should_log_init_output_before_load_result() {
        // given
        let dir = tempfile::tempdir().unwrap();

        // when
        let (_scripts, _, log) = start_with(dir.path(), r#"pub fn init(config) { println!("hello from init"); }"#, answer_terminal);

        // then
        logged(&log, "init.rn loaded");
        let lines = log.tail(10);
        let printed = lines.iter().position(|line| line.contains("hello from init"));
        let loaded = lines.iter().position(|line| line.contains("init.rn loaded"));
        assert!(printed < loaded, "{lines:?}");
    }

    fn answer_opened_from_hub(seconds_ago: f64) -> Reply {
        Ok(json!({ "tab": 4, "kind": "document", "checkout": "/p", "opened_from_hub": { "seconds_ago": seconds_ago, "hub": { "query": "theme" } } }))
    }

    fn answer_fresh_from_hub(request: &Request) -> Reply {
        match request {
            Request::ActiveTab => answer_opened_from_hub(2.0),
            _ => Ok(json!("ok")),
        }
    }

    fn answer_stale_from_hub(request: &Request) -> Reply {
        match request {
            Request::ActiveTab => answer_opened_from_hub(12.0),
            _ => Ok(json!("ok")),
        }
    }

    /// `close_back_to_hub`, the first function `ive.rn` binds.
    const CLOSE_BACK_TO_HUB: usize = 0;

    #[test]
    fn should_reopen_the_hub_when_a_tab_from_it_closes_quickly() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let (scripts, seen, _) = start_with(dir.path(), "pub fn init(config) {}", answer_fresh_from_hub);

        // when
        invoke_function(&scripts, 1, CLOSE_BACK_TO_HUB);

        // then
        assert!(wait_for(&seen, "OpenHub"));
        let requests = seen.lock().unwrap().clone();
        let order: Vec<&str> = requests.iter().map(|request| request.split([' ', '{']).next().unwrap_or_default()).collect();
        assert_eq!(order, ["ActiveTab", "CloseTab", "OpenHub"]);
        assert!(requests[2].contains("theme"), "{}", requests[2]);
    }

    #[test]
    fn should_not_reopen_the_hub_after_ten_seconds() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let (scripts, seen, _) = start_with(dir.path(), "pub fn init(config) {}", answer_stale_from_hub);

        // when
        invoke_function(&scripts, 1, CLOSE_BACK_TO_HUB);

        // then
        assert!(wait_for(&seen, "CloseTab"));
        std::thread::sleep(Duration::from_millis(200));
        assert!(!seen.lock().unwrap().iter().any(|request| request.contains("OpenHub")));
    }

    #[test]
    fn should_close_plainly_without_opened_from_hub() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let (scripts, seen, _) = start_with(dir.path(), "pub fn init(config) {}", answer_terminal);

        // when
        invoke_function(&scripts, 1, CLOSE_BACK_TO_HUB);

        // then
        assert!(wait_for(&seen, "CloseTab"));
        std::thread::sleep(Duration::from_millis(200));
        assert!(!seen.lock().unwrap().iter().any(|request| request.contains("OpenHub")));
    }

    #[test]
    fn should_stop_the_tab_before_going_back_to_the_hub() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let (scripts, seen, _) = start_with(dir.path(), "pub fn init(config) {}", answer_fresh_from_hub);

        // when
        invoke_function(&scripts, 1, CLOSE_BACK_TO_HUB + 1);

        // then
        assert!(wait_for(&seen, "OpenHub"));
        assert!(seen.lock().unwrap().iter().any(|request| request.starts_with("StopTab")));
    }

    #[test]
    fn should_close_or_stop_nothing_when_no_tab_is_shown() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let (mut scripts, seen, _) = start_with(dir.path(), "pub fn init(config) {}", answer_no_tab);

        // when
        invoke_function(&scripts, 1, CLOSE_BACK_TO_HUB);
        invoke_function(&scripts, 1, CLOSE_BACK_TO_HUB + 1);

        // then
        assert!(next_event(&mut scripts, Duration::from_millis(300)).is_none());
        let requests = seen.lock().unwrap().clone();
        assert_eq!(requests.iter().filter(|request| request.starts_with("ActiveTab")).count(), 2, "{requests:?}");
        assert!(!requests.iter().any(|request| request.starts_with("CloseTab") || request.starts_with("StopTab")), "{requests:?}");
    }
}
