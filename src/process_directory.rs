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

/// Elsewhere IVE does not read other processes' directories; only `OSC 7` reports count.
#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub fn directory_of(_pid: libc::pid_t) -> Option<PathBuf> {
    None
}

#[cfg(test)]
mod tests {
    use std::process::{Child, Command};

    use super::*;

    fn pid(child: &Child) -> libc::pid_t {
        libc::pid_t::try_from(child.id()).unwrap()
    }

    #[test]
    fn should_read_directory_of_a_running_process() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let mut child = Command::new("sleep").arg("30").current_dir(dir.path()).spawn().unwrap();

        // when
        let directory = directory_of(pid(&child));

        // then
        assert_eq!(directory, Some(dir.path().canonicalize().unwrap()));
        child.kill().unwrap();
        child.wait().unwrap();
    }

    #[test]
    fn should_have_no_directory_once_the_process_is_gone() {
        // given
        let mut child = Command::new("true").spawn().unwrap();
        child.wait().unwrap();

        // when
        let directory = directory_of(pid(&child));

        // then
        assert_eq!(directory, None);
    }
}
