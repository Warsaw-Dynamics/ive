use std::collections::HashMap;
use std::path::PathBuf;

/// The working directory of process `pid`; `None` once it is gone.
#[cfg(target_os = "linux")]
pub fn directory_of(pid: libc::pid_t) -> Option<PathBuf> {
    std::fs::read_link(format!("/proc/{pid}/cwd")).ok()
}

/// The working directory of process `pid`; `None` once it is gone.
#[cfg(target_os = "macos")]
pub fn directory_of(pid: libc::pid_t) -> Option<PathBuf> {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    let mut info = std::mem::MaybeUninit::<libc::proc_vnodepathinfo>::zeroed();
    let size = std::mem::size_of::<libc::proc_vnodepathinfo>() as libc::c_int;
    // SAFETY: proc_pidinfo writes at most `size` bytes into `info`, which is that large
    let read = unsafe { libc::proc_pidinfo(pid, libc::PROC_PIDVNODEPATHINFO, 0, info.as_mut_ptr().cast(), size) };
    if read != size {
        return None;
    }
    // SAFETY: zeroed is a valid proc_vnodepathinfo, and a full read filled it
    let info = unsafe { info.assume_init() };
    let path: Vec<u8> = info.pvi_cdir.vip_path.as_flattened().iter().map(|&byte| byte as u8).take_while(|&byte| byte != 0).collect();
    (!path.is_empty()).then(|| PathBuf::from(OsStr::from_bytes(&path)))
}

/// The working directory of process `pid`, read from its memory; `None` once it is gone, or when Windows refuses
/// (a process of another user, or elevated).
#[cfg(all(windows, test))]
pub fn directory_of(pid: i32) -> Option<PathBuf> {
    directories_of(&[pid]).remove(&pid)
}

/// The working directory of each of `pids` that has one to read, all in one read of the process list: on Windows
/// each read lists every process.
#[cfg(windows)]
pub fn directories_of(pids: &[i32]) -> HashMap<i32, PathBuf> {
    let asked: Vec<u32> = pids.iter().filter_map(|pid| u32::try_from(*pid).ok()).collect();
    let directories = crate::windows_processes::directories_of(&asked);
    directories.into_iter().filter_map(|(pid, directory)| Some((i32::try_from(pid).ok()?, directory))).collect()
}

/// The working directory of each of `pids` that has one to read.
#[cfg(not(windows))]
pub fn directories_of(pids: &[libc::pid_t]) -> HashMap<libc::pid_t, PathBuf> {
    pids.iter().filter_map(|pid| Some((*pid, directory_of(*pid)?))).collect()
}

/// Elsewhere IVE does not read other processes' directories; only `OSC 7` reports count.
#[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
pub fn directory_of(_pid: libc::pid_t) -> Option<PathBuf> {
    None
}

#[cfg(test)]
mod tests {
    use std::process::{Child, Command, Stdio};

    use super::*;

    fn pid(child: &Child) -> i32 {
        i32::try_from(child.id()).unwrap()
    }

    /// A program that runs half a minute.
    fn lasting() -> Command {
        if cfg!(windows) {
            let mut ping = Command::new("ping");
            ping.args(["-n", "30", "127.0.0.1"]).stdout(Stdio::null());
            return ping;
        }
        let mut sleep = Command::new("sleep");
        sleep.arg("30");
        sleep
    }

    fn brief() -> Command {
        if cfg!(windows) {
            let mut cmd = Command::new("cmd");
            cmd.args(["/D", "/C", "exit"]);
            return cmd;
        }
        Command::new("true")
    }

    #[test]
    fn should_read_directory_of_a_running_process() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let mut child = lasting().current_dir(dir.path()).spawn().unwrap();

        // when
        let directory = directory_of(pid(&child));

        // then
        assert_eq!(directory, Some(dunce::canonicalize(dir.path()).unwrap()));
        child.kill().unwrap();
        child.wait().unwrap();
    }

    #[test]
    fn should_have_no_directory_once_the_process_is_gone() {
        // given
        let mut child = brief().spawn().unwrap();
        child.wait().unwrap();

        // when
        let directory = directory_of(pid(&child));

        // then
        assert_eq!(directory, None);
    }
}
