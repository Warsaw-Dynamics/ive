//! Stopping a run: its whole process group, so what its shell started stops too.

use std::time::Duration;

/// How long a stopped run gets to exit before it is killed.
const STOP_GRACE: Duration = Duration::from_secs(5);

/// Sends SIGTERM to the process group `leader` leads, and SIGKILL once [`STOP_GRACE`] passed with it still there.
#[cfg(unix)]
pub fn stop_group(leader: u32) {
    let Ok(group) = libc::pid_t::try_from(leader) else { return };
    // SAFETY: kill only sends a signal; a group gone already makes it fail harmlessly
    unsafe { libc::kill(-group, libc::SIGTERM) };
    std::thread::spawn(move || {
        std::thread::sleep(STOP_GRACE);
        // SAFETY: as above; signal 0 only checks the group still exists
        unsafe {
            if libc::kill(-group, 0) == 0 {
                libc::kill(-group, libc::SIGKILL);
            }
        }
    });
}

/// Ends what is left in `job` once [`STOP_GRACE`] passed after the run's hang-up.
#[cfg(windows)]
pub fn stop_job(job: std::sync::Arc<crate::job::Job>) {
    std::thread::spawn(move || {
        std::thread::sleep(STOP_GRACE);
        job.terminate();
    });
}

#[cfg(all(test, unix))]
mod tests {
    use std::os::unix::process::{CommandExt, ExitStatusExt};
    use std::process::Command;

    use super::*;

    #[test]
    fn should_stop_a_process_group_with_sigterm() {
        // given
        let mut child = Command::new("sh").args(["-c", "sleep 30"]).process_group(0).spawn().unwrap();

        // when
        stop_group(child.id());

        // then
        let status = child.wait().unwrap();
        assert_eq!(status.signal(), Some(libc::SIGTERM));
    }
}
