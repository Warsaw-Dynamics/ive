//! The staging folder beside the install: the staged release, the lock that keeps two IVEs from staging or swapping
//! at once, and the list of IVEs running from the install.

use std::fs::{self, OpenOptions};
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::install::Install;

pub(super) const STAGED_NAME: &str = "staged.toml";
const LOCK_NAME: &str = "lock";
const RUNNING_DIR: &str = "running";
/// `staged.toml` is a few lines; anything bigger is not ours.
const MAX_STAGED_BYTES: u64 = 4096;

/// The executable or `.app` inside an extracted release (`<extracted>/<archive folder>/…`) that replaces `install`.
pub fn find_staged_install(extracted: &Path, install: &Install) -> Result<PathBuf, String> {
    let missing = || format!("the release extracted to {} has no IVE in it", extracted.display());
    let folders = fs::read_dir(extracted).map_err(|error| format!("cannot list {}: {error}", extracted.display()))?;
    let candidates = folders.filter_map(Result::ok).map(|entry| entry.path()).filter(|path| path.is_dir());
    candidates.into_iter().find_map(|folder| staged_in(&folder, install)).ok_or_else(missing)
}

fn staged_in(folder: &Path, install: &Install) -> Option<PathBuf> {
    match install {
        Install::Binary(_) => Some(folder.join(BINARY_NAME)).filter(|binary| binary.is_file()),
        Install::App(_) => fs::read_dir(folder).ok()?.filter_map(Result::ok).map(|entry| entry.path()).find(|path| path.extension().is_some_and(|extension| extension == "app")),
    }
}

#[cfg(windows)]
const BINARY_NAME: &str = "ive.exe";
#[cfg(not(windows))]
const BINARY_NAME: &str = "ive";

/// The terminal-host protocol the IVE `binary` speaks, by asking it; `None` when it cannot tell.
pub fn staged_protocol(binary: &Path) -> Option<u16> {
    let output = std::process::Command::new(binary).args(["--pty-host", "protocol"]).output().ok()?;
    output.status.success().then(|| String::from_utf8_lossy(&output.stdout).trim().parse().ok()).flatten()
}

/// The terminal-host protocol of this build; none on Windows, which has no terminal host.
pub fn own_protocol() -> Option<u16> {
    #[cfg(unix)]
    return Some(ptyhost::protocol::PROTOCOL);
    #[cfg(not(unix))]
    None
}

/// A release ready to swap in: its tag, where it is, and the terminal-host protocol it speaks (Windows has none).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Staged {
    pub tag: String,
    pub path: PathBuf,
    pub protocol: Option<u16>,
}

impl Staged {
    /// The record in staging folder `dir`; `None` when there is none, or it is not one IVE wrote.
    pub fn load(dir: &Path) -> Option<Self> {
        let path = dir.join(STAGED_NAME);
        let small = fs::metadata(&path).ok()?.len() <= MAX_STAGED_BYTES;
        let staged: Self = small.then(|| fs::read_to_string(&path).ok()).flatten().and_then(|text| toml::from_str(&text).ok())?;
        staged.path.exists().then_some(staged)
    }

    pub fn save(&self, dir: &Path) -> Result<(), String> {
        let path = dir.join(STAGED_NAME);
        let text = toml::to_string(self).map_err(|error| format!("cannot write {}: {error}", path.display()))?;
        fs::write(&path, text).map_err(|error| format!("cannot write {}: {error}", path.display()))
    }
}

/// The staging lock of this process; dropping it lets go.
#[derive(Debug)]
pub struct Lock {
    path: PathBuf,
}

impl Lock {
    /// Takes the lock in staging folder `dir`, taking it over from a process that is gone; `None` while a live one
    /// holds it.
    pub fn take(dir: &Path) -> Result<Option<Self>, String> {
        let path = dir.join(LOCK_NAME);
        if let Some(lock) = create_lock(&path)? {
            return Ok(Some(lock));
        }
        let holder = fs::read_to_string(&path).ok().and_then(|text| text.trim().parse::<u32>().ok());
        if holder.is_some_and(is_alive) {
            return Ok(None);
        }
        let _ = fs::remove_file(&path);
        create_lock(&path)
    }
}

fn create_lock(path: &Path) -> Result<Option<Lock>, String> {
    let created = OpenOptions::new().write(true).create_new(true).open(path);
    match created {
        Ok(mut file) => {
            file.write_all(std::process::id().to_string().as_bytes()).map_err(|error| format!("cannot write {}: {error}", path.display()))?;
            Ok(Some(Lock { path: path.to_path_buf() }))
        }
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => Ok(None),
        Err(error) => Err(format!("cannot create {}: {error}", path.display())),
    }
}

impl Drop for Lock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

/// This IVE's entry in the install's list of running IVEs; dropping it removes it.
#[derive(Debug)]
pub struct Running {
    path: PathBuf,
}

impl Running {
    pub fn register(dir: &Path) -> Result<Self, String> {
        let running = dir.join(RUNNING_DIR);
        fs::create_dir_all(&running).map_err(|error| format!("cannot create {}: {error}", running.display()))?;
        let path = running.join(std::process::id().to_string());
        fs::write(&path, "").map_err(|error| format!("cannot write {}: {error}", path.display()))?;
        Ok(Self { path })
    }
}

impl Drop for Running {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

/// Whether an IVE other than this one runs from the install of staging folder `dir`; entries of processes that are
/// gone are removed.
pub fn others_running(dir: &Path) -> bool {
    let Ok(entries) = fs::read_dir(dir.join(RUNNING_DIR)) else {
        return false;
    };
    let own = std::process::id();
    let pids: Vec<(PathBuf, u32)> = entries.filter_map(Result::ok).filter_map(|entry| Some((entry.path(), entry.file_name().to_str()?.parse().ok()?))).collect();
    let mut others = false;
    for (path, pid) in pids.into_iter().filter(|(_, pid)| *pid != own) {
        if is_alive(pid) {
            others = true;
            continue;
        }
        let _ = fs::remove_file(path);
    }
    others
}

#[cfg(unix)]
pub fn is_alive(pid: u32) -> bool {
    let Ok(pid) = libc::pid_t::try_from(pid) else {
        return false;
    };
    // SAFETY: signal 0 only checks that the process exists and may be signalled
    let result = unsafe { libc::kill(pid, 0) };
    result == 0 || io::Error::last_os_error().raw_os_error() == Some(libc::EPERM)
}

/// `tasklist` lists the process while it runs.
#[cfg(not(unix))]
pub fn is_alive(pid: u32) -> bool {
    let filter = format!("PID eq {pid}");
    let output = std::process::Command::new("tasklist").args(["/FI", &filter, "/NH"]).output();
    output.is_ok_and(|output| String::from_utf8_lossy(&output.stdout).contains(&pid.to_string()))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;
    use std::process::Command;

    use super::*;
    use crate::update::install::Install;

    fn dead_pid() -> u32 {
        let mut child = Command::new("true").spawn().unwrap();
        child.wait().unwrap();
        child.id()
    }

    fn write(path: &Path, text: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    #[test]
    fn should_find_the_binary_in_an_extracted_release() {
        // given
        let dir = tempfile::tempdir().unwrap();
        write(&dir.path().join("ive-linux-x86_64-abc/ive"), "new");
        let install = Install::Binary(PathBuf::from("/opt/ive/ive"));

        // when
        let found = find_staged_install(dir.path(), &install);

        // then
        assert_eq!(found, Ok(dir.path().join("ive-linux-x86_64-abc/ive")));
    }

    #[test]
    fn should_find_the_app_in_an_extracted_release() {
        // given
        let dir = tempfile::tempdir().unwrap();
        write(&dir.path().join("ive-macos-arm64-abc/IVE.app/Contents/MacOS/ive"), "new");
        let install = Install::App(PathBuf::from("/Applications/IVE.app"));

        // when
        let found = find_staged_install(dir.path(), &install);

        // then
        assert_eq!(found, Ok(dir.path().join("ive-macos-arm64-abc/IVE.app")));
    }

    #[test]
    fn should_report_an_extracted_release_without_ive() {
        // given
        let dir = tempfile::tempdir().unwrap();
        write(&dir.path().join("ive-linux-x86_64-abc/README"), "");
        let install = Install::Binary(PathBuf::from("/opt/ive/ive"));

        // when
        let found = find_staged_install(dir.path(), &install);

        // then
        assert!(found.unwrap_err().contains(&dir.path().display().to_string()));
    }

    #[cfg(unix)]
    #[test]
    fn should_read_the_protocol_a_binary_prints() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let binary = dir.path().join("ive");
        // written by a child: a file this process wrote can still be open in a child another test forks meanwhile,
        // and executing it then fails with ETXTBSY
        let script = "printf '#!/bin/sh\\n[ \"$1 $2\" = \"--pty-host protocol\" ] && echo 7\\n' > \"$0\" && chmod 755 \"$0\"";
        assert!(Command::new("sh").arg("-c").arg(script).arg(&binary).status().unwrap().success());

        // when
        let protocol = staged_protocol(&binary);

        // then
        assert_eq!(protocol, Some(7));
    }

    #[cfg(unix)]
    #[test]
    fn should_have_no_protocol_from_a_binary_that_cannot_tell() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let binary = dir.path().join("missing");

        // when
        let protocol = staged_protocol(&binary);

        // then
        assert_eq!(protocol, None);
    }

    #[test]
    fn should_round_trip_the_staged_record() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let staged = Staged { tag: "ive-0.1.0-abc".to_owned(), path: dir.path().join("x/ive"), protocol: Some(1) };
        write(&staged.path, "new");

        // when
        staged.save(dir.path()).unwrap();
        let loaded = Staged::load(dir.path());

        // then
        assert_eq!(loaded, Some(staged));
    }

    #[test]
    fn should_forget_a_staged_record_whose_release_is_gone() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let staged = Staged { tag: "ive-0.1.0-abc".to_owned(), path: dir.path().join("x/ive"), protocol: None };
        staged.save(dir.path()).unwrap();

        // when
        let loaded = Staged::load(dir.path());

        // then
        assert_eq!(loaded, None);
    }

    #[test]
    fn should_take_over_a_lock_of_a_dead_process() {
        // given
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(LOCK_NAME), dead_pid().to_string()).unwrap();

        // when
        let lock = Lock::take(dir.path()).unwrap();

        // then
        assert!(lock.is_some());
        assert_eq!(fs::read_to_string(dir.path().join(LOCK_NAME)).unwrap(), std::process::id().to_string());
        drop(lock);
        assert!(!dir.path().join(LOCK_NAME).exists());
    }

    #[test]
    fn should_not_take_a_lock_held_by_a_live_process() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let held = Lock::take(dir.path()).unwrap();

        // when
        let second = Lock::take(dir.path()).unwrap();

        // then
        assert!(held.is_some());
        assert!(second.is_none());
    }

    #[test]
    fn should_ignore_running_entries_of_dead_processes() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let dead = dead_pid();
        write(&dir.path().join(RUNNING_DIR).join(dead.to_string()), "");
        let own = Running::register(dir.path()).unwrap();

        // when
        let others = others_running(dir.path());

        // then
        assert!(!others);
        assert!(!dir.path().join(RUNNING_DIR).join(dead.to_string()).exists());
        drop(own);
        assert!(!dir.path().join(RUNNING_DIR).join(std::process::id().to_string()).exists());
    }

    #[test]
    fn should_see_another_running_ive() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let mut other = Command::new("sleep").arg("5").spawn().unwrap();
        write(&dir.path().join(RUNNING_DIR).join(other.id().to_string()), "");

        // when
        let others = others_running(dir.path());

        // then
        other.kill().unwrap();
        other.wait().unwrap();
        assert!(others);
    }
}
