//! The script log: a plain text file IVE shows in an ordinary file tab. Load results, compile and run-time errors
//! and `println!` output go there, one timestamped line each.

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::age::{self, Clock};

/// Past this size the log keeps only its newer half.
pub const MAX_LOG_BYTES: u64 = 1024 * 1024;

/// `scripts.log` in the profile's folder of the OS state directory (`~/.local/state/ive/<profile>/` on Linux),
/// else of its config directory (macOS has no state directory), else of the temp directory.
pub fn default_path() -> PathBuf {
    let base = dirs::state_dir().or_else(dirs::config_dir).unwrap_or_else(std::env::temp_dir);
    base.join("ive").join(crate::profile::current().name()).join("scripts.log")
}

#[derive(Debug, Clone)]
pub struct ScriptLog {
    path: PathBuf,
    max_bytes: u64,
}

impl ScriptLog {
    pub fn new(path: PathBuf, max_bytes: u64) -> Self {
        Self { path, max_bytes }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Appends each line of `text` with the local time. A failure goes to stderr: the log is where script errors
    /// go, so it cannot report its own.
    pub fn append(&self, text: &str) {
        if let Err(error) = self.try_append(text) {
            eprintln!("ive: cannot write the script log {}: {error}; it said: {text}", self.path.display());
        }
    }

    /// The last `count` lines, oldest first; none when the file cannot be read.
    pub fn tail(&self, count: usize) -> Vec<String> {
        let text = fs::read_to_string(&self.path).unwrap_or_default();
        let lines: Vec<&str> = text.lines().collect();
        lines[lines.len().saturating_sub(count)..].iter().map(|line| (*line).to_owned()).collect()
    }

    fn try_append(&self, text: &str) -> io::Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let time = age::date_time(Clock::system().when());
        let stamped: String = text.lines().map(|line| format!("{time} {line}\n")).collect();
        let mut file = OpenOptions::new().create(true).append(true).open(&self.path)?;
        file.write_all(stamped.as_bytes())?;
        if file.metadata()?.len() > self.max_bytes {
            self.keep_newer_half()?;
        }
        Ok(())
    }

    /// Keeps the whole lines in the newer half of the limit.
    fn keep_newer_half(&self) -> io::Result<()> {
        let text = fs::read_to_string(&self.path)?;
        let budget = usize::try_from(self.max_bytes / 2).unwrap_or(usize::MAX);
        let cut = text.len().saturating_sub(budget);
        let start = text[cut..].find('\n').map_or(text.len(), |newline| cut + newline + 1);
        fs::write(&self.path, &text[start..])
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn should_append_lines_with_time() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let log = ScriptLog::new(dir.path().join("ive").join("scripts.log"), 1024);

        // when
        log.append("init.rn loaded: 2 bindings");
        log.append("f8: no tab is shown");

        // then
        let written = fs::read_to_string(log.path()).unwrap();
        let lines: Vec<&str> = written.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].starts_with("20") && lines[0].ends_with(" init.rn loaded: 2 bindings"), "{}", lines[0]);
        assert!(lines[1].ends_with(" f8: no tab is shown"), "{}", lines[1]);
    }

    #[test]
    fn should_keep_newer_half_when_over_limit() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let log = ScriptLog::new(dir.path().join("scripts.log"), 400);

        // when
        for number in 0..40 {
            log.append(&format!("line {number:02}"));
        }

        // then
        let written = fs::read_to_string(log.path()).unwrap();
        assert!(written.len() <= 400, "{} bytes", written.len());
        assert!(written.ends_with("line 39\n"));
        assert!(!written.contains("line 00"));
        assert!(written.lines().all(|line| line.contains("line ")), "a cut line: {written}");
    }

    #[test]
    fn should_return_last_lines() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let log = ScriptLog::new(dir.path().join("scripts.log"), 4096);
        for number in 0..5 {
            log.append(&format!("line {number}"));
        }

        // when
        let tail = log.tail(2);

        // then
        assert_eq!(tail.len(), 2);
        assert!(tail[0].ends_with("line 3") && tail[1].ends_with("line 4"), "{tail:?}");
    }

    #[test]
    fn should_return_no_lines_without_file() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let log = ScriptLog::new(dir.path().join("scripts.log"), 4096);

        // when
        let tail = log.tail(50);

        // then
        assert!(tail.is_empty());
    }
}
