//! How a terminal's program ended, and what IVE shows for it: a failure that says what ran, where, how it ended and
//! what it printed last, or nothing when it simply closed.

use std::process::ExitStatus;
use std::time::Duration;

use super::view::TerminalState;
use crate::failure::Failure;

/// A shell that fails within this long after start most likely could not start at all.
const QUICK_EXIT: Duration = Duration::from_secs(2);

/// Whether IVE saw the shell start, which tells whether a quick exit means it could not start.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Start {
    New,
    Adopted,
}

/// What ran and where, for the failure's details.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Started {
    pub command: String,
    pub folder: String,
}

/// A hosted session whose connection broke for good: its id, to attach to it, and the error that broke it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lost {
    pub id: String,
    pub reason: String,
}

/// Everything known when a terminal's program ended.
pub struct ExitFacts {
    pub status: Option<ExitStatus>,
    pub elapsed: Duration,
    pub start: Start,
    pub lost: Option<Lost>,
    pub started: Started,
    /// its last output lines, oldest first
    pub output: Vec<String>,
}

/// A shell's end: lost, killed by a signal, or failed right after start shows a failure; any other exit closes the tab,
/// as in IntelliJ. A signal IVE sent itself is told apart by the caller.
pub fn shell_exit(facts: ExitFacts) -> TerminalState {
    if let Some(lost) = facts.lost {
        return TerminalState::Failed(lost_failure("Lost the terminal; it still runs", lost));
    }
    if let Some(signal) = facts.status.and_then(signal_of) {
        return TerminalState::Failed(ended("The shell was killed by a signal", signal_name(signal), facts.started, facts.output));
    }
    match facts.status {
        Some(status) if !status.success() && facts.start == Start::New && facts.elapsed < QUICK_EXIT => {
            TerminalState::Failed(ended("The shell exited right after start", format!("exit code {}", exit_code_of(Some(status))), facts.started, facts.output))
        }
        _ => TerminalState::Closed,
    }
}

/// A run's end: it stays with its exit code, however quickly it failed; one whose host was lost may still run.
pub fn run_exit(facts: ExitFacts) -> TerminalState {
    match facts.lost {
        Some(lost) => TerminalState::Failed(lost_failure("Lost the run; it still runs", lost)),
        None => TerminalState::Exited(exit_code_of(facts.status)),
    }
}

/// A shell that could not start: what, where, and what runs it (`runs_in`, e.g. a terminal host).
pub fn start_failure(started: Started, runs_in: &str, error: &std::io::Error) -> Failure {
    Failure::new(format!("Cannot start a shell: {error}")).with("Command", started.command).with("Folder", started.folder).with("Runs in", runs_in.to_owned())
}

fn lost_failure(summary: &str, lost: Lost) -> Failure {
    Failure::new(summary).with("Cause", lost.reason).with("Attach", format!("ive --pty-host attach {}", lost.id))
}

fn ended(summary: &str, exit: String, started: Started, output: Vec<String>) -> Failure {
    Failure::new(summary).with("Command", started.command).with("Folder", started.folder).with("Exit", exit).with_output(output)
}

/// The exit code, or 128 + the signal that killed the program, as shells report it; -1 when unknown.
#[cfg(unix)]
pub fn exit_code_of(status: Option<ExitStatus>) -> i32 {
    status.and_then(|status| status.code().or_else(|| signal_of(status).map(|signal| 128 + signal))).unwrap_or(-1)
}

#[cfg(not(unix))]
pub fn exit_code_of(status: Option<ExitStatus>) -> i32 {
    status.and_then(|status| status.code()).unwrap_or(-1)
}

#[cfg(unix)]
fn signal_of(status: ExitStatus) -> Option<i32> {
    use std::os::unix::process::ExitStatusExt;
    status.signal()
}

#[cfg(not(unix))]
fn signal_of(_status: ExitStatus) -> Option<i32> {
    None
}

/// A signal by name and number, as `SIGSEGV (11)`; `signal 77` for one without a common name.
pub fn signal_name(signal: i32) -> String {
    match signal_short_name(signal) {
        Some(name) => format!("{name} ({signal})"),
        None => format!("signal {signal}"),
    }
}

#[cfg(not(unix))]
pub fn signal_short_name(_signal: i32) -> Option<&'static str> {
    None
}

/// A common signal's name, as `SIGKILL`.
#[cfg(unix)]
pub fn signal_short_name(signal: i32) -> Option<&'static str> {
    let names = [
        (libc::SIGHUP, "SIGHUP"), (libc::SIGINT, "SIGINT"), (libc::SIGQUIT, "SIGQUIT"), (libc::SIGILL, "SIGILL"),
        (libc::SIGTRAP, "SIGTRAP"), (libc::SIGABRT, "SIGABRT"), (libc::SIGBUS, "SIGBUS"), (libc::SIGFPE, "SIGFPE"),
        (libc::SIGKILL, "SIGKILL"), (libc::SIGUSR1, "SIGUSR1"), (libc::SIGSEGV, "SIGSEGV"), (libc::SIGUSR2, "SIGUSR2"),
        (libc::SIGPIPE, "SIGPIPE"), (libc::SIGALRM, "SIGALRM"), (libc::SIGTERM, "SIGTERM"), (libc::SIGXCPU, "SIGXCPU"),
        (libc::SIGXFSZ, "SIGXFSZ"), (libc::SIGSYS, "SIGSYS"),
    ];
    names.iter().find(|(number, _)| *number == signal).map(|(_, name)| *name)
}

#[cfg(test)]
mod tests {
    use std::os::unix::process::ExitStatusExt;

    use super::*;

    fn facts(status: Option<ExitStatus>, elapsed: Duration, start: Start) -> ExitFacts {
        let started = Started { command: "/bin/zsh".to_owned(), folder: "/p".to_owned() };
        ExitFacts { status, elapsed, start, lost: None, started, output: vec!["zsh: bad option".to_owned()] }
    }

    fn report(state: TerminalState) -> Option<String> {
        match state {
            TerminalState::Failed(failure) => Some(failure.report("Terminal")),
            TerminalState::Closed | TerminalState::Running | TerminalState::Exited(_) => None,
        }
    }

    #[test]
    fn should_give_the_exit_code_or_128_plus_the_signal() {
        // given
        let statuses = [Some(ExitStatus::from_raw(1 << 8)), Some(ExitStatus::from_raw(libc::SIGTERM)), None];

        // when
        let codes = statuses.map(exit_code_of);

        // then
        assert_eq!(codes, [1, 143, -1]);
    }

    #[test]
    fn should_keep_last_output_when_shell_fails_at_start() {
        // given
        let facts = facts(Some(ExitStatus::from_raw(1 << 8)), Duration::from_millis(100), Start::New);

        // when
        let report = report(shell_exit(facts)).expect("a failure");

        // then
        assert!(report.contains("right after start"), "{report}");
        assert!(report.contains("Command: /bin/zsh\nFolder: /p\nExit: exit code 1\n"), "{report}");
        assert!(report.contains("zsh: bad option"), "{report}");
    }

    #[test]
    fn should_name_signal_of_crashed_shell() {
        // given
        let facts = facts(Some(ExitStatus::from_raw(libc::SIGSEGV)), Duration::from_secs(600), Start::Adopted);

        // when
        let report = report(shell_exit(facts)).expect("a failure");

        // then
        assert!(report.contains("SIGSEGV (11)"), "{report}");
    }

    #[test]
    fn should_close_on_exit_code_after_start() {
        // given
        let facts = facts(Some(ExitStatus::from_raw(1 << 8)), Duration::from_secs(3), Start::New);

        // when
        let state = shell_exit(facts);

        // then
        assert!(matches!(state, TerminalState::Closed));
    }

    #[test]
    fn should_close_adopted_session_that_exits_quickly() {
        // given
        let facts = facts(Some(ExitStatus::from_raw(1 << 8)), Duration::from_millis(100), Start::Adopted);

        // when
        let state = shell_exit(facts);

        // then
        assert!(matches!(state, TerminalState::Closed));
    }

    #[test]
    fn should_say_why_terminal_was_lost() {
        // given
        let lost = Lost { id: "t1a2b3".to_owned(), reason: "Connection reset by peer (os error 104)".to_owned() };
        let facts = ExitFacts { lost: Some(lost), ..facts(None, Duration::from_secs(60), Start::Adopted) };

        // when
        let report = report(shell_exit(facts)).expect("a failure");

        // then
        assert!(report.contains("Cause: Connection reset by peer (os error 104)\n"), "{report}");
        assert!(report.contains("Attach: ive --pty-host attach t1a2b3\n"), "{report}");
    }

    #[test]
    fn should_keep_a_run_that_fails_right_after_start() {
        // given
        let facts = facts(Some(ExitStatus::from_raw(127 << 8)), Duration::from_millis(100), Start::New);

        // when
        let state = run_exit(facts);

        // then
        assert!(matches!(state, TerminalState::Exited(127)));
    }

    #[test]
    fn should_say_what_could_not_start() {
        // given
        let started = Started { command: "/bin/zsh".to_owned(), folder: "/p".to_owned() };
        let error = std::io::Error::from(std::io::ErrorKind::NotFound);

        // when
        let report = start_failure(started, "a terminal host, so it outlives IVE", &error).report("Terminal");

        // then
        assert!(report.starts_with("Terminal: Cannot start a shell: entity not found\n\nCommand: /bin/zsh\nFolder: /p\nRuns in: a terminal host, so it outlives IVE\n"), "{report}");
    }

    #[test]
    fn should_name_known_signals() {
        // given
        let signals = [libc::SIGKILL, libc::SIGSEGV, 77];

        // when
        let names = signals.map(signal_name);

        // then
        assert_eq!(names, ["SIGKILL (9)".to_owned(), "SIGSEGV (11)".to_owned(), "signal 77".to_owned()]);
    }
}
