//! Other processes on Windows, read through sysinfo: what a shell started, and a process's command line and directory.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex, PoisonError};
use std::time::{Duration, Instant};

use sysinfo::{Pid, Process, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

/// What a pseudoconsole starts to host its console; never a program the user started.
const CONSOLE_HOSTS: [&str; 2] = ["conhost.exe", "OpenConsole.exe"];

/// How long a list of the processes serves the questions after it. Every refresh, of one process too, has Windows
/// list all of them with their threads, and a poll asks about each terminal several times: one list answers a poll.
const LIST_FRESH: Duration = Duration::from_millis(200);

/// Kept between reads: sysinfo opens a handle to each process it sees for the first time, which a fresh list would
/// do for every process on every poll.
struct Processes {
    system: System,
    listed_at: Option<Instant>,
}

impl Processes {
    /// Lists every process again, unless the list is younger than [`LIST_FRESH`].
    fn list(&mut self) {
        if self.listed_at.is_some_and(|at| at.elapsed() < LIST_FRESH) {
            return;
        }
        self.system.refresh_processes_specifics(ProcessesToUpdate::All, ProcessRefreshKind::new());
        self.listed_at = Some(Instant::now());
    }
}

static PROCESSES: LazyLock<Mutex<Processes>> = LazyLock::new(|| Mutex::new(Processes { system: System::new(), listed_at: None }));

/// The youngest live child of `parent` other than a console host, as of at most [`LIST_FRESH`] ago; `None` when
/// it has none or is gone.
pub fn youngest_child(parent: u32) -> Option<u32> {
    let mut processes = PROCESSES.lock().unwrap_or_else(PoisonError::into_inner);
    processes.list();
    let processes = &processes.system;
    let parent = Pid::from_u32(parent);
    let started = processes.process(parent)?.start_time();
    processes
        .processes()
        .values()
        // an older one names a parent that ended and whose id Windows gave to the shell since
        .filter(|process| process.parent() == Some(parent) && process.start_time() >= started)
        .filter(|process| !CONSOLE_HOSTS.iter().any(|host| process.name().eq_ignore_ascii_case(host)))
        .max_by_key(|process| process.start_time())
        .map(|process| process.pid().as_u32())
}

/// `read` of process `pid` with its command line, which is read from the process the first time only: it stays
/// while the process lives. `None` once the process is gone.
pub fn read_command<R>(pid: u32, read: impl FnOnce(&Process) -> R) -> Option<R> {
    let mut processes = PROCESSES.lock().unwrap_or_else(PoisonError::into_inner);
    let pid = Pid::from_u32(pid);
    if processes.system.process(pid).is_none_or(|process| process.cmd().is_empty()) {
        let kind = ProcessRefreshKind::new().with_cmd(UpdateKind::Always);
        processes.system.refresh_processes_specifics(ProcessesToUpdate::Some(&[pid]), kind);
    }
    processes.system.process(pid).map(read)
}

/// The working directory of each of `pids` now, in one read for all of them; a process that is gone, or that
/// Windows refuses to read (another user's, or elevated), has none.
pub fn directories_of(pids: &[u32]) -> HashMap<u32, PathBuf> {
    let mut processes = PROCESSES.lock().unwrap_or_else(PoisonError::into_inner);
    let asked: Vec<Pid> = pids.iter().map(|pid| Pid::from_u32(*pid)).collect();
    let kind = ProcessRefreshKind::new().with_cwd(UpdateKind::Always);
    processes.system.refresh_processes_specifics(ProcessesToUpdate::Some(&asked), kind);
    let directory = |pid: &Pid| Some((pid.as_u32(), processes.system.process(*pid)?.cwd()?.to_path_buf()));
    asked.iter().filter_map(directory).collect()
}

#[cfg(test)]
mod tests {
    use std::process::{Command, Stdio};
    use std::time::{Duration, Instant};

    use super::*;

    #[test]
    fn should_find_the_program_a_shell_started() {
        // given
        let mut shell = Command::new("cmd").args(["/D", "/C", "ping -n 30 127.0.0.1"]).stdout(Stdio::null()).spawn().unwrap();
        let start = Instant::now();
        let mut child = None;

        // when
        while child.is_none() && start.elapsed() < Duration::from_secs(10) {
            std::thread::sleep(Duration::from_millis(50));
            child = youngest_child(shell.id());
        }

        // then
        let name = child.and_then(|pid| read_command(pid, |process| process.name().to_string_lossy().to_lowercase()));
        assert_eq!(name.as_deref(), Some("ping.exe"));
        child.and_then(|pid| read_command(pid, Process::kill));
        shell.kill().unwrap();
        shell.wait().unwrap();
    }
}
