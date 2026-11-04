//! `~/.config/ive/<profile>/schedule.toml`: the scheduled commands and their history. It holds commands, so on Unix
//! only its owner may read it.

use std::fs;
use std::io;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use super::Schedule;

/// Far more than 100 commands and a week of history take.
const MAX_FILE_BYTES: u64 = 4 * 1024 * 1024;
static STAGING: AtomicU64 = AtomicU64::new(0);

/// The schedule at `path`, and why the file was set aside when it could not be used; no file is an empty schedule.
pub fn load(path: &Path) -> (Schedule, Option<String>) {
    match read(path) {
        Ok(schedule) => (schedule, None),
        Err(reason) => (Schedule::default(), Some(crate::state::set_aside(path, &reason))),
    }
}

fn read(path: &Path) -> Result<Schedule, String> {
    let bytes = match fs::metadata(path) {
        Ok(metadata) => metadata.len(),
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Schedule::default()),
        Err(error) => return Err(error.to_string()),
    };
    if bytes > MAX_FILE_BYTES {
        return Err(format!("{bytes} bytes, the limit is {MAX_FILE_BYTES}"));
    }
    let text = fs::read_to_string(path).map_err(|error| error.to_string())?;
    toml::from_str(&text).map_err(|error| format!("line {}: {}", crate::toml_error::line_of(error.span(), &text), error.message().trim_end()))
}

/// Writes `schedule` to `path` through a staging file of its own, creating the folder.
pub fn save(schedule: &Schedule, path: &Path) -> Result<(), String> {
    let failed = |error: &dyn std::fmt::Display| format!("cannot save {}: {error}", path.display());
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|error| failed(&error))?;
    }
    let text = toml::to_string(schedule).map_err(|error| failed(&error))?;
    let staging = path.with_extension(format!("toml.{}-{}.tmp", std::process::id(), STAGING.fetch_add(1, Ordering::Relaxed)));
    write_private(&staging, &text).map_err(|error| failed(&error))?;
    crate::retry::replace_file(&staging, path).map_err(|error| {
        let _ = fs::remove_file(&staging);
        failed(&error)
    })
}

#[cfg(unix)]
fn write_private(path: &Path, text: &str) -> io::Result<()> {
    use std::io::Write as _;
    use std::os::unix::fs::OpenOptionsExt as _;
    let mut file = fs::OpenOptions::new().write(true).create(true).truncate(true).mode(0o600).open(path)?;
    file.write_all(text.as_bytes())
}

#[cfg(not(unix))]
fn write_private(path: &Path, text: &str) -> io::Result<()> {
    fs::write(path, text)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::schedule::{NewEntry, Target};

    fn one_command() -> Schedule {
        let mut schedule = Schedule::default();
        let target = Target { project: PathBuf::from("/p"), branch: "main".to_owned(), worktree: None, source: Some("dev".to_owned()) };
        let new = NewEntry { name: "check".to_owned(), description: "why".to_owned(), command: "claude \"go\"".to_owned(), target, at: 100, repeat: None };
        schedule.add(new, 50).unwrap();
        schedule
    }

    #[test]
    fn should_read_back_what_it_saved() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("schedule.toml");
        let schedule = one_command();
        // when
        save(&schedule, &path).unwrap();
        let (read, problem) = load(&path);
        // then
        assert_eq!((read, problem), (schedule, None));
    }

    #[cfg(unix)]
    #[test]
    fn should_let_only_the_owner_read_it() {
        // given
        use std::os::unix::fs::PermissionsExt as _;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("schedule.toml");
        // when
        save(&one_command(), &path).unwrap();
        // then
        assert_eq!(fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o600);
    }

    #[test]
    fn should_set_aside_a_file_that_does_not_load() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("schedule.toml");
        fs::write(&path, "[[command]]\nid = \"x\"\n").unwrap();
        // when
        let (read, problem) = load(&path);
        // then
        assert_eq!(read, Schedule::default());
        assert!(problem.unwrap().contains("moved it to schedule.toml.bad"));
        assert!(dir.path().join("schedule.toml.bad").exists());
    }
}
