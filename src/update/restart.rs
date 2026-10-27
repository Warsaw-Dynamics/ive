//! When IVE may restart onto a staged update, and how it starts the new IVE.

use std::ffi::OsString;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

use super::install::Install;

/// What decides whether IVE restarts onto a staged update now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RestartConditions {
    pub staged: bool,
    /// since the last key, mouse or MCP input
    pub idle: Duration,
    pub idle_limit: Duration,
    /// another IVE runs from the install, so it cannot be swapped
    pub others_running: bool,
    /// the staged build's terminal hosts are ones this build's terminals can be adopted by
    pub protocol_matches: bool,
    pub unsaved: bool,
    pub prompt_shown: bool,
    /// processes run that a restart would end: this IVE does not keep its terminals
    pub terminals_at_risk: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestartVerdict {
    Restart,
    /// not now, and why
    Wait(&'static str),
}

/// Whether being idle restarts IVE onto the staged update.
pub fn idle_verdict(conditions: &RestartConditions) -> RestartVerdict {
    match button_verdict(conditions) {
        RestartVerdict::Restart if conditions.unsaved => RestartVerdict::Wait("unsaved edits"),
        RestartVerdict::Restart if conditions.idle < conditions.idle_limit => RestartVerdict::Wait("not idle"),
        verdict => verdict,
    }
}

/// Whether Restart to update can restart IVE: it saves edits first and needs no idle time.
pub fn button_verdict(conditions: &RestartConditions) -> RestartVerdict {
    if !conditions.staged {
        return RestartVerdict::Wait("no update staged");
    }
    if conditions.others_running {
        return RestartVerdict::Wait("another IVE runs from this install");
    }
    if !conditions.protocol_matches {
        return RestartVerdict::Wait("terminals would not carry over; it applies on quit");
    }
    if conditions.prompt_shown {
        return RestartVerdict::Wait("the quit prompt shows");
    }
    if conditions.terminals_at_risk {
        return RestartVerdict::Wait("running processes would end");
    }
    RestartVerdict::Restart
}

/// Whether the new IVE takes focus: a click on Restart to update does, an idle restart does not.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Take,
    Keep,
}

/// Flags of IVE followed by a value, which is not a folder to open.
const VALUE_FLAGS: [&str; 3] = ["--mcp", "--profile", "--restart"];

/// The new IVE's arguments: this one's, with a relative folder made absolute against `cwd` (macOS's `open` does not
/// keep the folder IVE runs in), `--replacing <pid>`, and `--background` for [`Focus::Keep`].
pub fn relaunch_args(original: &[OsString], cwd: &Path, pid: u32, focus: Focus) -> Vec<OsString> {
    let mut args = Vec::with_capacity(original.len() + 3);
    let mut arguments = original.iter();
    while let Some(argument) = arguments.next() {
        let text = argument.to_string_lossy();
        match text.as_ref() {
            "--background" => {}
            "--replacing" => drop(arguments.next()),
            flag if flag.starts_with("--replacing=") => {}
            flag if VALUE_FLAGS.contains(&flag) => {
                args.push(argument.clone());
                args.extend(arguments.next().cloned());
            }
            flag if flag.starts_with("--") => args.push(argument.clone()),
            _ => args.push(cwd.join(argument).into_os_string()),
        }
    }
    args.extend(["--replacing".into(), pid.to_string().into()]);
    if focus == Focus::Keep {
        args.push("--background".into());
    }
    args
}

/// Starts the IVE at `install` with `args`, apart from this one so it outlives it.
pub fn relaunch(install: &Install, args: &[OsString], focus: Focus) -> Result<(), String> {
    let mut command = launch_command(install, args, focus);
    command.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    detach(&mut command);
    command.spawn().map(drop).map_err(|error| format!("cannot start the updated IVE at {}: {error}", install.path().display()))
}

/// macOS starts an app through `open`, so the Dock shows it as IVE; `-g` keeps it from coming to the front, which
/// IVE's own `--background` cannot stop.
fn launch_command(install: &Install, args: &[OsString], focus: Focus) -> Command {
    match install {
        Install::App(app) => {
            let mut command = Command::new("open");
            command.arg("-n");
            if focus == Focus::Keep {
                command.arg("-g");
            }
            command.arg("-a").arg(app).arg("--args").args(args);
            command
        }
        Install::Binary(exe) => {
            let mut command = Command::new(exe);
            command.args(args);
            command
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Exit {
    Exited,
    StillRunning,
}

/// How often [`wait_for_exit`] looks.
const EXIT_POLL: Duration = Duration::from_millis(100);

/// Waits up to `limit` for process `pid` to exit: an IVE replacing another reads the tabs it saves as it quits.
pub fn wait_for_exit(pid: u32, limit: Duration) -> Exit {
    let deadline = std::time::Instant::now() + limit;
    while super::staging::is_alive(pid) {
        if std::time::Instant::now() >= deadline {
            return Exit::StillRunning;
        }
        std::thread::sleep(EXIT_POLL);
    }
    Exit::Exited
}

/// A process group of its own: the old IVE's terminal closing does not hang it up.
#[cfg(unix)]
fn detach(command: &mut Command) {
    use std::os::unix::process::CommandExt;
    command.process_group(0);
}

#[cfg(not(unix))]
fn detach(_command: &mut Command) {}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::Duration;

    use super::*;

    fn ready() -> RestartConditions {
        RestartConditions {
            staged: true,
            idle: Duration::from_secs(31 * 60),
            idle_limit: Duration::from_secs(30 * 60),
            others_running: false,
            protocol_matches: true,
            unsaved: false,
            prompt_shown: false,
            terminals_at_risk: false,
        }
    }

    fn os(args: &[&str]) -> Vec<OsString> {
        args.iter().map(OsString::from).collect()
    }

    #[test]
    fn should_restart_when_idle_and_nothing_refuses() {
        // given
        let conditions = ready();

        // when
        let verdict = idle_verdict(&conditions);

        // then
        assert_eq!(verdict, RestartVerdict::Restart);
    }

    #[test]
    fn should_wait_while_unsaved_edits_exist() {
        // given
        let conditions = RestartConditions { unsaved: true, ..ready() };

        // when
        let verdict = idle_verdict(&conditions);

        // then
        assert_eq!(verdict, RestartVerdict::Wait("unsaved edits"));
    }

    #[test]
    fn should_wait_for_other_ives_on_the_install() {
        // given
        let conditions = RestartConditions { others_running: true, ..ready() };

        // when
        let verdicts = [idle_verdict(&conditions), button_verdict(&conditions)];

        // then
        assert_eq!(verdicts, [RestartVerdict::Wait("another IVE runs from this install"); 2]);
    }

    #[test]
    fn should_wait_for_a_protocol_change_to_apply_on_quit() {
        // given
        let conditions = RestartConditions { protocol_matches: false, ..ready() };

        // when
        let verdicts = [idle_verdict(&conditions), button_verdict(&conditions)];

        // then
        assert_eq!(verdicts, [RestartVerdict::Wait("terminals would not carry over; it applies on quit"); 2]);
    }

    #[test]
    fn should_wait_while_terminals_would_end() {
        // given
        let conditions = RestartConditions { terminals_at_risk: true, ..ready() };

        // when
        let verdict = idle_verdict(&conditions);

        // then
        assert_eq!(verdict, RestartVerdict::Wait("running processes would end"));
    }

    #[test]
    fn should_wait_before_the_idle_limit() {
        // given
        let conditions = RestartConditions { idle: Duration::from_secs(29 * 60), ..ready() };

        // when
        let verdicts = [idle_verdict(&conditions), button_verdict(&conditions)];

        // then
        assert_eq!(verdicts, [RestartVerdict::Wait("not idle"), RestartVerdict::Restart]);
    }

    #[test]
    fn should_let_the_button_restart_with_unsaved_edits_it_saves_first() {
        // given
        let conditions = RestartConditions { unsaved: true, idle: Duration::ZERO, ..ready() };

        // when
        let verdict = button_verdict(&conditions);

        // then
        assert_eq!(verdict, RestartVerdict::Restart);
    }

    #[test]
    fn should_have_nothing_to_restart_onto_without_a_staged_update() {
        // given
        let conditions = RestartConditions { staged: false, ..ready() };

        // when
        let verdicts = [idle_verdict(&conditions), button_verdict(&conditions)];

        // then
        assert_eq!(verdicts, [RestartVerdict::Wait("no update staged"); 2]);
    }

    #[cfg(unix)]
    #[test]
    fn should_wait_for_the_replaced_ive_to_exit() {
        // given
        let mut child = std::process::Command::new("sleep").arg("0.3").spawn().unwrap();
        let pid = child.id();
        let reaper = std::thread::spawn(move || child.wait());
        let started = std::time::Instant::now();

        // when
        let exited = wait_for_exit(pid, Duration::from_secs(5));

        // then
        assert_eq!(exited, Exit::Exited);
        assert!(started.elapsed() >= Duration::from_millis(250));
        reaper.join().unwrap().unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn should_stop_waiting_after_the_limit() {
        // given
        let mut child = std::process::Command::new("sleep").arg("5").spawn().unwrap();

        // when
        let exited = wait_for_exit(child.id(), Duration::from_millis(300));

        // then
        child.kill().unwrap();
        child.wait().unwrap();
        assert_eq!(exited, Exit::StillRunning);
    }

    #[test]
    fn should_open_an_app_in_the_background_on_an_idle_restart() {
        // given
        let install = Install::App(PathBuf::from("/Applications/IVE.app"));
        let args = os(&["--replacing", "42"]);

        // when
        let keep = launch_command(&install, &args, Focus::Keep);
        let take = launch_command(&install, &args, Focus::Take);

        // then
        let args_of = |command: &Command| command.get_args().map(OsString::from).collect::<Vec<_>>();
        assert_eq!(args_of(&keep), os(&["-n", "-g", "-a", "/Applications/IVE.app", "--args", "--replacing", "42"]));
        assert_eq!(args_of(&take), os(&["-n", "-a", "/Applications/IVE.app", "--args", "--replacing", "42"]));
    }

    #[test]
    fn should_build_relaunch_arguments_without_duplicates() {
        // given
        let original = os(&["--profile", "work", "--background", "--replacing", "12", "--replacing=13", "code", "--mcp", "7461"]);

        // when
        let args = relaunch_args(&original, Path::new("/home/me"), 42, Focus::Take);

        // then
        let code = Path::new("/home/me").join("code").into_os_string();
        assert_eq!(args, [os(&["--profile", "work"]), vec![code], os(&["--mcp", "7461", "--replacing", "42"])].concat());
    }

    #[test]
    fn should_keep_focus_out_of_an_idle_restart() {
        // given
        let original = os(&["/abs/folder"]);

        // when
        let args = relaunch_args(&original, Path::new("/home/me"), 42, Focus::Keep);

        // then
        assert_eq!(args, os(&["/abs/folder", "--replacing", "42", "--background"]));
    }

    #[test]
    fn should_keep_flag_values_that_look_like_paths() {
        // given
        let original = os(&["--restart", "restart.sh", "--profile", "work"]);

        // when
        let args = relaunch_args(&original, Path::new("/home/me"), 42, Focus::Take);

        // then
        assert_eq!(args, os(&["--restart", "restart.sh", "--profile", "work", "--replacing", "42"]));
    }
}
