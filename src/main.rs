#[allow(dead_code)] // ages and dates for parts 2 and 3
mod age;
mod api;
mod args;
mod assets;
mod build_info;
mod click_targets;
mod clipboard;
mod codeview_host;
mod colors;
mod disk_text;
mod dragged_files;
mod editor;
mod file_tab;
mod file_write;
mod favicon;
mod file_tree;
mod frame;
mod glow;
mod git;
mod http_server;
mod hub;
mod icons;
mod ive_toml;
#[cfg(target_os = "linux")]
mod memory_watch;
mod mouse_binding;
mod mcp;
mod menu;
mod menu_view;
mod open_project;
mod os_open;
mod overview;
mod paging;
mod pasted;
mod path_text;
mod paths;
mod process;
#[cfg(unix)]
mod process_directory;
#[cfg(unix)]
mod process_stop;
mod profile;
mod progress_bar;
mod retry;
mod run;
mod project;
mod project_list;
mod prompt;
mod saved_tabs;
mod scratch;
mod script;
mod settings;
mod settings_page;
#[cfg(unix)]
mod signals;
mod sidebar;
mod state;
mod tab_preview;
mod tab_window;
mod tabs;
mod third_party;
mod trace;
mod terminal;
mod terminal_use;
mod text_input;
mod theme;
mod theme_config;
mod tooltip;
mod update;
mod watch;
mod window_capture;
mod window_controls;
mod window_drag;
mod window_state;
mod workspace;

#[cfg(unix)]
use std::ffi::OsString;
use std::process::ExitCode;

use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender, unbounded};
use gpui::{App, Bounds, prelude::*};

use args::{Args, McpPort};
use codeview::{document, parallel, scrollbar, syntax};
use assets::Assets;
use api::Call;
use mcp::Token;
use settings::SettingsStore;
use workspace::{Launch, OnQuit, Startup, Workspace};

fn main() -> ExitCode {
    #[cfg(all(target_os = "linux", target_env = "gnu"))]
    cap_malloc_arenas();
    #[cfg(unix)]
    if let Some(code) = run_pty_host() {
        return code;
    }
    // a run configuration's static server; nothing of IVE itself starts in this process
    if std::env::args_os().nth(1).is_some_and(|flag| flag == "--serve") {
        return ExitCode::from(u8::try_from(run::serve::main()).unwrap_or(1));
    }
    trace::init();
    let args = match Args::parse(std::env::args_os().skip(1)) {
        Ok(args) => args,
        Err(error) => {
            eprintln!("ive: {error}\n{}", args::USAGE);
            return ExitCode::from(2);
        }
    };
    profile::init(args.profile.clone());
    if let Err(error) = check_restart(args.restart.as_deref()) {
        eprintln!("ive: {error}");
        return ExitCode::from(2);
    }
    delete_old_pastes();
    #[cfg(target_os = "linux")]
    memory_watch::start();
    #[cfg(windows)]
    update::install::delete_set_aside_exe();
    if let Some(pid) = args.replacing {
        wait_for_replaced(pid);
    }
    let start = Start::of(&args);
    let (calls, api_calls) = unbounded();
    if let Err(error) = tracing::info_span!("serve_mcp").in_scope(|| serve_mcp(args.mcp, start, &calls)) {
        eprintln!("ive: {error}");
        return ExitCode::FAILURE;
    }
    let log = script::log::ScriptLog::new(script::log::default_path(), script::log::MAX_LOG_BYTES);
    let setup = script::Setup { dir: script::default_dir(), theme: saved_user_theme() };
    let scripts = tracing::info_span!("start_scripts").in_scope(|| script::start(setup, log, calls));
    let state_path = profile::current().state_path();
    let saved = tracing::info_span!("load_state").in_scope(|| load_state(state_path.as_deref()));
    let startup = Startup {
        launch: Launch::from_path(args.path),
        saved,
        state_path,
        api_calls,
        scripts,
        #[cfg(unix)]
        hosted: tracing::info_span!("claim_terminals").in_scope(|| claim_terminals(start)),
        signals: forward_signals(),
        restart_script: args.restart.and_then(|script| std::fs::canonicalize(script).ok()),
        on_quit: if args.stop_processes_on_quit { OnQuit::StopProcesses } else { OnQuit::Ask },
    };
    run(startup, args.background);
    ExitCode::SUCCESS
}

fn load_state(path: Option<&std::path::Path>) -> state::SavedState {
    path.map(state::load).map_or_else(state::SavedState::default, |loaded| {
        if let Some(problem) = loaded.problem {
            eprintln!("ive: {problem}");
        }
        loaded.state
    })
}

/// glibc gives threads that allocate at once arenas of their own, up to 8 per core, and keeps what they freed:
/// restoring the atlassian projects (git reads on ~30 background threads) left 470 MiB resident instead of 245.
/// `MALLOC_ARENA_MAX` in the environment still wins. Call before any thread starts.
#[cfg(all(target_os = "linux", target_env = "gnu"))]
fn cap_malloc_arenas() {
    const MAX_ARENAS: libc::c_int = 4;
    if std::env::var_os("MALLOC_ARENA_MAX").is_none() {
        // SAFETY: mallopt only sets an allocator parameter; no other thread exists yet to allocate meanwhile
        unsafe { libc::mallopt(libc::M_ARENA_MAX, MAX_ARENAS) };
    }
}

/// Off the startup path: the paste folder can hold many images.
fn delete_old_pastes() {
    std::thread::spawn(|| {
        if let Err(error) = pasted::delete_old_images() {
            eprintln!("ive: cannot delete old pasted images: {error}");
        }
    });
}

fn run(startup: Startup, background: bool) {
    let store = tracing::info_span!("load_settings").in_scope(load_settings);
    let application = tracing::info_span!("start_platform").in_scope(|| gpui_platform::application().with_assets(Assets));
    #[cfg(target_os = "macos")]
    application.on_reopen(workspace::show_main_window);
    application.run(move |cx: &mut App| {
        trace::milestone("platform running");
        tracing::info_span!("load_fonts").in_scope(|| assets::load_fonts(cx));
        cx.set_global(theme::Fonts::default());
        cx.set_global(store);
        cx.set_global(click_targets::ClickTargets::default());
        // where gpui would draw its own fallback prompt; elsewhere the OS's prompt stays
        if cfg!(target_os = "linux") {
            cx.set_prompt_builder(prompt::build);
        }
        cx.set_global(theme_config::defaults());
        theme::apply(cx.window_appearance(), cx);
        text_input::bind_keys(cx);
        codeview_host::install(cx);
        #[cfg(target_os = "macos")]
        workspace::add_app_menu(cx);
        let saved = startup.saved.window.and_then(|window| window.usable_bounds(&window_state::screens(cx)));
        let bounds = saved.unwrap_or_else(|| Bounds::centered(None, window_state::DEFAULT_SIZE, cx));
        // a first start leaves the place to the window manager, which knows the monitor the user works on
        let options = gpui::WindowOptions { focus: !background, keep_position: saved.is_some(), ..window_controls::window_options(bounds) };
        let open = |window: &mut gpui::Window, cx: &mut App| {
            // next-frame callbacks run before that frame draws: the second one runs once the first is on screen
            window.on_next_frame(|window, _| window.on_next_frame(|_, _| trace::milestone("first frame drawn")));
            cx.new(|cx| Workspace::new(startup, window, cx))
        };
        tracing::info_span!("open_window").in_scope(|| cx.open_window(options, open)).expect("failed to open main window");
        if !background {
            cx.activate(true);
        }
    });
}

/// SIGTERM, SIGINT and SIGUSR1 as messages, so IVE asks before quitting; `None` (dying at once, as before) where they cannot
/// be caught.
fn forward_signals() -> Option<UnboundedReceiver<i32>> {
    #[cfg(unix)]
    match signals::forward() {
        Ok(signals) => return Some(signals),
        Err(error) => eprintln!("ive: cannot catch SIGTERM, SIGINT and SIGUSR1: {error}; they end IVE at once"),
    }
    None
}

/// `ive --pty-host …` runs the terminal host CLI instead of IVE: IVE starts each terminal's host this way, and
/// people use it to list, attach or end sessions outside IVE. Nothing of IVE itself starts in this process.
#[cfg(unix)]
fn run_pty_host() -> Option<ExitCode> {
    let args: Vec<OsString> = std::env::args_os().skip(1).collect();
    match pty_host_args(&args)? {
        Ok(rest) => Some(ptyhost::cli::main(&rest, &pty_host_launcher())),
        Err(error) => {
            eprintln!("ive --pty-host: {error}");
            Some(ExitCode::from(2))
        }
    }
}

/// The arguments after `--pty-host`, when IVE runs as the terminal host CLI; `None` leaves every argument, UTF-8
/// or not, to IVE.
#[cfg(unix)]
fn pty_host_args(args: &[OsString]) -> Option<Result<Vec<String>, String>> {
    let (first, rest) = args.split_first()?;
    let utf8 = |arg: &OsString| arg.clone().into_string().map_err(|arg| format!("argument {} is not UTF-8", arg.to_string_lossy()));
    (first == PTY_HOST_FLAG).then(|| rest.iter().map(utf8).collect())
}

/// Starts hosts through `/proc/self/exe`, which still names this binary after a rebuild replaced its file.
#[cfg(target_os = "linux")]
pub fn pty_host_launcher() -> ptyhost::host::Launcher {
    ptyhost::host::Launcher { exe: "/proc/self/exe".into(), args: vec![PTY_HOST_FLAG.into()] }
}

/// macOS has no `/proc/self/exe`; the path IVE started from names the same binary until a build replaces it.
#[cfg(target_os = "macos")]
pub fn pty_host_launcher() -> ptyhost::host::Launcher {
    let exe = std::env::current_exe().unwrap_or_else(|_| std::env::args_os().next().unwrap_or_default().into());
    ptyhost::host::Launcher { exe, args: vec![PTY_HOST_FLAG.into()] }
}

#[cfg(unix)]
const PTY_HOST_FLAG: &str = "--pty-host";

/// `--restart` needs SIGUSR1, so Unix, and a start script IVE may run.
fn check_restart(script: Option<&std::path::Path>) -> Result<(), String> {
    let Some(script) = script else {
        return Ok(());
    };
    #[cfg(unix)]
    return args::check_restart_script(script);
    #[cfg(not(unix))]
    Err(format!("--restart {} needs SIGUSR1, which only Unix has", script.display()))
}

/// An update restart starts this IVE at once; the one it replaces saves its tabs as it quits, which this one reads.
fn wait_for_replaced(pid: u32) {
    let limit = retry::RESTART_INTERVAL * retry::RESTART_ATTEMPTS;
    if update::restart::wait_for_exit(pid, limit) == update::restart::Exit::StillRunning {
        eprintln!("ive: IVE {pid}, which this one replaces, still runs after {} s; starting anyway", limit.as_secs());
    }
}

/// Whether this IVE may be replacing another, which still holds the terminals and the MCP port for a moment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Start {
    Fresh,
    /// `--restart` (the restart script may have started this IVE) or `--replacing` (an update restart did)
    Replacing,
}

impl Start {
    fn of(args: &Args) -> Self {
        match (&args.restart, args.replacing) {
            (None, None) => Self::Fresh,
            _ => Self::Replacing,
        }
    }
}

/// An IVE replacing another waits for it to let go of the terminals.
#[cfg(unix)]
fn claim_terminals(start: Start) -> Option<terminal::hosted::HostedTerminals> {
    match start {
        Start::Replacing => terminal::hosted::HostedTerminals::claim_after_restart(profile::current()),
        Start::Fresh => terminal::hosted::HostedTerminals::claim(profile::current()),
    }
}

/// Serves MCP unless `--no-mcp`, printing how to connect; `Err` when IVE must not start without it. An IVE replacing
/// another waits up to 10 s for it to free the port.
fn serve_mcp(mcp: McpPort, start: Start, calls: &UnboundedSender<Call>) -> Result<(), String> {
    let given = match mcp {
        McpPort::Off => return Ok(()),
        McpPort::Remembered => None,
        McpPort::Given(port) => Some(port),
    };
    let env_token = Token::from_env().transpose().map_err(|error| format!("{}: {error}", mcp::TOKEN_ENV))?;
    let path = profile::current().mcp_path();
    let attempts = match start {
        Start::Replacing => retry::RESTART_ATTEMPTS,
        Start::Fresh => 1,
    };
    let request = mcp::endpoint::Request { given, path: path.as_deref(), env_token, attempts };
    let opened = mcp::endpoint::open(request, |port, token| mcp::start(port, token.clone(), calls.clone()))?;
    for line in opened.lines {
        eprintln!("ive: {line}");
    }
    if let Some((url, token)) = opened.served {
        terminal::env::announce_mcp(url, token.value().to_owned());
    }
    Ok(())
}

/// The theme the settings picked, for the scripts' first load; IVE's own when they do not load, which
/// [`load_settings`] reports.
fn saved_user_theme() -> settings::ThemeChoice {
    let saved = settings::default_path().and_then(|path| settings::Settings::load(&path).ok());
    saved.map(|settings| settings.user_theme).unwrap_or_default()
}

/// Saved settings, or defaults when the file does not load; the reason goes to stderr too, since no window exists
/// yet, and the Settings tab shows it.
fn load_settings() -> SettingsStore {
    let store = SettingsStore::open(settings::default_path());
    if let Some(error) = store.load_error() {
        eprintln!("ive: {error}; using default settings until the file is fixed");
    }
    store
}

#[cfg(all(test, unix))]
mod tests {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    use super::*;

    #[test]
    fn should_leave_non_utf8_ive_arguments_to_ive() {
        // given
        let args = vec![OsString::from_vec(b"caf\xe9.txt".to_vec())];

        // when
        let host = pty_host_args(&args);

        // then
        assert_eq!(host, None);
    }

    #[test]
    fn should_pass_host_arguments_on() {
        // given
        let args = ["--pty-host", "ls"].map(OsString::from).to_vec();

        // when
        let host = pty_host_args(&args);

        // then
        assert_eq!(host, Some(Ok(vec!["ls".to_owned()])));
    }

    #[test]
    fn should_reject_non_utf8_host_argument() {
        // given
        let args = vec![OsString::from("--pty-host"), OsString::from_vec(b"\xff".to_vec())];

        // when
        let host = pty_host_args(&args);

        // then
        assert!(matches!(host, Some(Err(_))));
    }
}
