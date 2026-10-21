//! Which program has a terminal: the PTY's foreground process group and the command of its leader.

use std::fs::File;
use std::os::fd::AsRawFd;
use std::path::Path;
use std::sync::Arc;

use alacritty_terminal::tty::Pty;

/// Reads the foreground process group of one PTY; cheap to clone and usable on any thread.
#[derive(Debug, Clone)]
pub struct ForegroundProbe {
    shell: libc::pid_t,
    source: Source,
}

#[derive(Debug, Clone)]
enum Source {
    /// a duplicate of the PTY master, so the probe works whatever the PTY thread does with its own
    Master(Arc<File>),
    /// the shell's process status, for a terminal whose master another process holds (a host)
    Proc,
}

/// A process's command name (`comm`) and `argv[0]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramCommand {
    pub comm: String,
    pub argv0: String,
}

impl ForegroundProbe {
    /// A probe for the shell `pty` runs; `None` when the master cannot be duplicated (out of descriptors).
    pub fn of(pty: &Pty) -> Option<Self> {
        let master = pty.file().try_clone().ok()?;
        let shell = libc::pid_t::try_from(pty.child().id()).ok()?;
        Some(Self { shell, source: Source::Master(Arc::new(master)) })
    }

    /// A probe for the shell `shell`, read from its process status: the terminal belongs to another process.
    pub fn of_program(shell: u32) -> Self {
        Self { shell: shell as libc::pid_t, source: Source::Proc }
    }

    fn foreground_group(&self) -> libc::pid_t {
        match &self.source {
            // SAFETY: reads terminal state of a descriptor `master` keeps open
            Source::Master(master) => unsafe { libc::tcgetpgrp(master.as_raw_fd()) },
            Source::Proc => terminal_group_of(self.shell).unwrap_or(-1),
        }
    }

    /// macOS starts an in-process shell through `login`; a host starts the shell itself.
    fn is_shell(&self, group: libc::pid_t) -> bool {
        group == self.shell || matches!(self.source, Source::Master(_)) && is_shell_under_login(group, self.shell)
    }

    /// The foreground process group while a program other than the shell has the terminal.
    pub fn program_group(&self) -> Option<libc::pid_t> {
        let group = self.foreground_group();
        match group {
            ..=0 => None,
            shell if self.is_shell(shell) => None,
            program => Some(program),
        }
    }

    /// The command of the foreground group's leader, unless the shell itself has the terminal.
    pub fn program(&self) -> Option<ProgramCommand> {
        self.program_group().and_then(command_of)
    }

    /// Who leads the foreground: the program's group, or the shell itself.
    pub fn leader(&self) -> libc::pid_t {
        self.program_group().unwrap_or(self.shell)
    }

    pub fn runs_claude_code(&self) -> bool {
        self.program().is_some_and(|command| is_claude_code(&command))
    }
}

#[cfg(target_os = "linux")]
fn terminal_group_of(pid: libc::pid_t) -> Option<libc::pid_t> {
    std::fs::read_to_string(format!("/proc/{pid}/stat")).ok().as_deref().and_then(tpgid_of)
}

#[cfg(target_os = "macos")]
fn terminal_group_of(pid: libc::pid_t) -> Option<libc::pid_t> {
    let mut info = std::mem::MaybeUninit::<libc::proc_bsdinfo>::zeroed();
    let size = std::mem::size_of::<libc::proc_bsdinfo>() as libc::c_int;
    // SAFETY: proc_pidinfo writes at most `size` bytes into `info`, which is that large
    let read = unsafe { libc::proc_pidinfo(pid, libc::PROC_PIDTBSDINFO, 0, info.as_mut_ptr().cast(), size) };
    if read != size {
        return None;
    }
    // SAFETY: zeroed is a valid proc_bsdinfo, and a full read filled it
    let group = unsafe { info.assume_init() }.e_tpgid;
    libc::pid_t::try_from(group).ok().filter(|group| *group > 0)
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn terminal_group_of(_pid: libc::pid_t) -> Option<libc::pid_t> {
    None
}

/// The terminal's foreground group (`tpgid`, field 8) from a `/proc/<pid>/stat` line; the command name before it
/// may hold spaces and parentheses, so fields count from the last `)`.
#[cfg(target_os = "linux")]
fn tpgid_of(stat: &str) -> Option<libc::pid_t> {
    let fields = &stat[stat.rfind(')')? + 1..];
    fields.split_whitespace().nth(5)?.parse().ok().filter(|group| *group > 0)
}

/// Claude Code: `comm` is `claude`, or `argv[0]` is a `claude` file, or a version the native installer keeps
/// under `~/.local/share/claude/versions/<version>`.
pub fn is_claude_code(command: &ProgramCommand) -> bool {
    let file_name = Path::new(&command.argv0).file_name().and_then(|name| name.to_str());
    command.comm == "claude" || file_name == Some("claude") || command.argv0.contains("/claude/versions/")
}

/// macOS: alacritty starts the shell through `login -flp`, so the PTY's child is `login`, and the shell, its
/// child, leads a group of its own.
#[cfg(target_os = "macos")]
fn is_shell_under_login(group: libc::pid_t, login: libc::pid_t) -> bool {
    ps_field(group, "ppid=").and_then(|parent| parent.parse::<libc::pid_t>().ok()) == Some(login)
}

/// Elsewhere the PTY's child is the shell itself.
#[cfg(not(target_os = "macos"))]
fn is_shell_under_login(_group: libc::pid_t, _child: libc::pid_t) -> bool {
    false
}

#[cfg(target_os = "linux")]
fn command_of(pid: libc::pid_t) -> Option<ProgramCommand> {
    let comm = std::fs::read_to_string(format!("/proc/{pid}/comm")).ok()?;
    let cmdline = std::fs::read(format!("/proc/{pid}/cmdline")).ok()?;
    let argv0 = cmdline.split(|byte| *byte == 0).next().unwrap_or_default();
    Some(ProgramCommand { comm: comm.trim_end().to_owned(), argv0: String::from_utf8_lossy(argv0).into_owned() })
}

#[cfg(target_os = "macos")]
fn command_of(pid: libc::pid_t) -> Option<ProgramCommand> {
    // `comm=` is the executable path on macOS; two calls, since paths with spaces make one line ambiguous
    let comm = ps_field(pid, "comm=")?;
    let args = ps_field(pid, "args=")?;
    let comm = Path::new(&comm).file_name().map_or(comm.clone(), |name| name.to_string_lossy().into_owned());
    Some(ProgramCommand { comm, argv0: args.split_whitespace().next().unwrap_or_default().to_owned() })
}

#[cfg(target_os = "macos")]
fn ps_field(pid: libc::pid_t, field: &str) -> Option<String> {
    let output = std::process::Command::new("ps").args(["-o", field, "-p", &pid.to_string()]).output().ok()?;
    output.status.success().then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn command_of(_pid: libc::pid_t) -> Option<ProgramCommand> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn command(comm: &str, argv0: &str) -> ProgramCommand {
        ProgramCommand { comm: comm.to_owned(), argv0: argv0.to_owned() }
    }

    #[test]
    fn should_recognise_claude_code_by_comm_argv0_or_versions_path() {
        // given
        let commands = [
            command("claude", "node"),
            command("node", "/usr/local/bin/claude"),
            command("2.1.3", "/home/kb/.local/share/claude/versions/2.1.3"),
            command("zsh", "-zsh"),
            command("vim", "vim"),
            command("claude-dev", "claude-dev"),
        ];

        // when
        let detected = commands.map(|command| is_claude_code(&command));

        // then
        assert_eq!(detected, [true, true, true, false, false, false]);
    }

    #[cfg(target_os = "linux")]
    mod pty {
        use std::io::Write;
        use std::time::{Duration, Instant};

        use alacritty_terminal::event::WindowSize;
        use alacritty_terminal::tty::{self, Options, Shell};

        use super::super::*;
        use crate::process_directory::directory_of;

        fn interactive_sh() -> Pty {
            // job control (interactive `sh`) puts each command in its own foreground group, as login shells do
            let options = Options { shell: Some(Shell::new("/bin/sh".to_owned(), vec!["-i".to_owned()])), ..Options::default() };
            tty::new(&options, WindowSize { num_lines: 24, num_cols: 80, cell_width: 8, cell_height: 16 }, 0).unwrap()
        }

        fn wait_for<T>(mut probe: impl FnMut() -> Option<T>) -> Option<T> {
            let deadline = Instant::now() + Duration::from_secs(5);
            while Instant::now() < deadline {
                if let Some(found) = probe() {
                    return Some(found);
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            None
        }

        #[test]
        fn should_see_command_started_from_the_shell_in_the_foreground() {
            // given
            let pty = interactive_sh();
            let probe = ForegroundProbe::of(&pty).unwrap();
            let shell_first = probe.program().is_none();

            // when
            pty.file().write_all(b"sleep 5\n").unwrap();
            let program = wait_for(|| probe.program());

            // then
            assert!(shell_first);
            assert_eq!(program.map(|command| command.comm), Some("sleep".to_owned()));
        }

        #[test]
        fn should_see_foreground_command_through_proc_without_the_master() {
            // given
            let pty = interactive_sh();
            let probe = ForegroundProbe::of_program(pty.child().id());
            let shell_first = probe.program().is_none();

            // when
            pty.file().write_all(b"sleep 5\n").unwrap();
            let program = wait_for(|| probe.program());

            // then
            assert!(shell_first);
            assert_eq!(program.map(|command| command.comm), Some("sleep".to_owned()));
        }

        #[test]
        fn should_read_directory_of_foreground_leader() {
            // given
            let dir = tempfile::tempdir().unwrap();
            let pty = interactive_sh();
            let probe = ForegroundProbe::of(&pty).unwrap();

            // when
            pty.file().write_all(format!("cd {} && sleep 5\n", dir.path().display()).as_bytes()).unwrap();
            wait_for(|| probe.program().filter(|command| command.comm == "sleep"));
            let directory = directory_of(probe.leader());

            // then
            assert_eq!(directory, Some(dir.path().canonicalize().unwrap()));
        }

        #[test]
        fn should_read_foreground_group_from_proc_stat() {
            // given
            let stat = "123 (my (odd) cmd) S 1 123 123 34816 4567 4194560 0 0";

            // when
            let group = tpgid_of(stat);

            // then
            assert_eq!(group, Some(4567));
        }
    }
}
