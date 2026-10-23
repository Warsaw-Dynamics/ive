//! Other processes on Windows, read through sysinfo: what a shell started, and a process's command line and directory.

use std::sync::{LazyLock, Mutex, PoisonError};

use sysinfo::{Pid, Process, ProcessRefreshKind, ProcessesToUpdate, System};

/// What a pseudoconsole starts to host its console; never a program the user started.
const CONSOLE_HOSTS: [&str; 2] = ["conhost.exe", "OpenConsole.exe"];

/// Kept between reads: sysinfo opens a handle to each process it sees for the first time, which a fresh list would
/// do for every process on every poll.
static PROCESSES: LazyLock<Mutex<System>> = LazyLock::new(|| Mutex::new(System::new()));

/// The youngest live child of `parent` other than a console host; `None` when it has none or is gone.
pub fn youngest_child(parent: u32) -> Option<u32> {
    let mut processes = PROCESSES.lock().unwrap_or_else(PoisonError::into_inner);
    processes.refresh_processes_specifics(ProcessesToUpdate::All, ProcessRefreshKind::new());
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

/// `read` of process `pid` once what `kind` names is read afresh; `None` once the process is gone.
pub fn read<R>(pid: u32, kind: ProcessRefreshKind, read: impl FnOnce(&Process) -> R) -> Option<R> {
    let mut processes = PROCESSES.lock().unwrap_or_else(PoisonError::into_inner);
    let pid = Pid::from_u32(pid);
    processes.refresh_processes_specifics(ProcessesToUpdate::Some(&[pid]), kind);
    processes.process(pid).map(read)
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
        let name = child.and_then(|pid| read(pid, ProcessRefreshKind::new(), |process| process.name().to_string_lossy().to_lowercase()));
        assert_eq!(name.as_deref(), Some("ping.exe"));
        child.and_then(|pid| read(pid, ProcessRefreshKind::new(), Process::kill));
        shell.kill().unwrap();
        shell.wait().unwrap();
    }
}
