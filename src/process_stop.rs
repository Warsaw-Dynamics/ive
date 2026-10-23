//! Ending a process for good: the process groups to signal and whether any of them still runs.

/// The groups a process's Stop ends: the one its program (a shell, a run's command) leads, and the foreground
/// program's when that is another (Claude Code, vim).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Targets {
    leader: libc::pid_t,
    foreground: Option<libc::pid_t>,
}

impl Targets {
    /// `None` without a program pid (a session not started yet): there is nothing to signal or wait for.
    pub fn new(program: Option<u32>, foreground: Option<libc::pid_t>) -> Option<Self> {
        let leader = libc::pid_t::try_from(program?).ok().filter(|pid| *pid > 0)?;
        Some(Self { leader, foreground: foreground.filter(|group| *group > 0 && *group != leader) })
    }

    /// The program's pid, as the user knows it.
    pub fn pid(&self) -> libc::pid_t {
        self.leader
    }

    /// Sends `signal` to every target group; one gone already makes it fail harmlessly.
    pub fn signal(&self, signal: libc::c_int) {
        for group in self.groups() {
            // SAFETY: kill only sends a signal
            unsafe { libc::kill(-group, signal) };
        }
    }

    pub fn terminate(&self) {
        self.signal(libc::SIGTERM);
    }

    pub fn kill(&self) {
        self.signal(libc::SIGKILL);
    }

    /// Whether a group's leader still runs; a zombie (exited, not reaped yet) does not.
    pub fn alive(&self) -> bool {
        self.groups().any(runs)
    }

    fn groups(&self) -> impl Iterator<Item = libc::pid_t> {
        std::iter::once(self.leader).chain(self.foreground)
    }
}

#[cfg(target_os = "linux")]
fn runs(pid: libc::pid_t) -> bool {
    let Ok(stat) = std::fs::read_to_string(format!("/proc/{pid}/stat")) else {
        return false;
    };
    // the state follows the command in parentheses, which may itself hold ") "
    let state = stat.rfind(") ").and_then(|at| stat[at + 2..].chars().next());
    !matches!(state, Some('Z' | 'X') | None)
}

// `kill(pid, 0)` succeeds on a zombie too, so the process's status is read instead; a zombie's may not be read at all
#[cfg(target_os = "macos")]
fn runs(pid: libc::pid_t) -> bool {
    let mut info = std::mem::MaybeUninit::<libc::proc_bsdinfo>::zeroed();
    let size = std::mem::size_of::<libc::proc_bsdinfo>() as libc::c_int;
    // SAFETY: proc_pidinfo writes at most `size` bytes into `info`, which is that large
    let read = unsafe { libc::proc_pidinfo(pid, libc::PROC_PIDTBSDINFO, 0, info.as_mut_ptr().cast(), size) };
    // SAFETY: zeroed is a valid proc_bsdinfo, and a full read filled it
    read == size && unsafe { info.assume_init() }.pbi_status != libc::SZOMB
}

#[cfg(test)]
mod tests {
    use std::os::unix::process::{CommandExt, ExitStatusExt};
    use std::process::{Child, Command};
    use std::time::{Duration, Instant};

    use super::*;

    fn group(script: &str) -> Child {
        Command::new("sh").args(["-c", script]).process_group(0).spawn().unwrap()
    }

    fn targets(child: &Child) -> Targets {
        Targets::new(Some(child.id()), None).unwrap()
    }

    fn wait_until_dead(targets: &Targets) -> bool {
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            if !targets.alive() {
                return true;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        false
    }

    #[test]
    fn should_see_a_running_group_alive() {
        // given
        let mut child = group("sleep 30");

        // when
        let alive = targets(&child).alive();

        // then
        assert!(alive);
        child.kill().unwrap();
        child.wait().unwrap();
    }

    #[test]
    fn should_count_an_unreaped_exit_as_gone() {
        // given
        let mut child = group("exit 0");
        let targets = targets(&child);

        // when
        let gone = wait_until_dead(&targets);

        // then
        assert!(gone, "a zombie counts as stopped");
        child.wait().unwrap();
    }

    #[test]
    fn should_stop_a_group_with_sigterm() {
        // given
        let mut child = group("sleep 30");

        // when
        targets(&child).signal(libc::SIGTERM);

        // then
        assert_eq!(child.wait().unwrap().signal(), Some(libc::SIGTERM));
    }

    #[test]
    fn should_keep_a_group_that_ignores_sigterm_until_sigkill() {
        // given
        let mut child = group("trap '' TERM; while :; do sleep 1; done");
        let targets = targets(&child);
        std::thread::sleep(Duration::from_millis(100));

        // when
        targets.signal(libc::SIGTERM);
        std::thread::sleep(Duration::from_millis(200));
        let after_term = targets.alive();
        targets.signal(libc::SIGKILL);

        // then
        assert!(after_term);
        assert_eq!(child.wait().unwrap().signal(), Some(libc::SIGKILL));
    }

    #[test]
    fn should_have_no_targets_without_a_pid() {
        // when
        let targets = Targets::new(None, Some(42));

        // then
        assert_eq!(targets, None);
    }

    #[test]
    fn should_not_signal_the_leader_twice_as_foreground() {
        // when
        let targets = Targets::new(Some(100), Some(100)).unwrap();

        // then
        assert_eq!(targets.groups().collect::<Vec<_>>(), [100]);
    }
}
