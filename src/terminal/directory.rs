//! The working directory a program reports with `OSC 7;file://<host>/<path>`, as WezTerm, kitty, foot, VTE and
//! iTerm2 read it.

use std::path::PathBuf;

use alacritty_terminal::vte::{Parser, Perform};

/// Longest path a report may name, as Linux's `PATH_MAX`.
const MAX_PATH: usize = 4096;

/// A program's last working-directory report; `number` counts reports from 1, so a reader can tell a new one from
/// one it saw.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectoryReport {
    pub number: u64,
    pub path: PathBuf,
    /// who led the foreground when it arrived; `None` for replayed output, which may predate the program now running
    pub leader: Option<i32>,
}

/// What one foreground poll saw of a terminal: who leads its foreground, the last report, and the leader's own
/// directory.
pub struct Sample {
    pub leader: i32,
    pub report: Option<DirectoryReport>,
    pub process_cwd: Option<PathBuf>,
}

/// A terminal's current directory across polls: the last report while the program that made it leads the
/// foreground, else the leader's own directory, so a report outlives neither its program nor a newer one.
#[derive(Default)]
pub struct DirectoryTracker {
    /// the number of the last report taken
    seen: Option<u64>,
    /// the report in force, with the leader it came from
    bound: Option<(i32, PathBuf)>,
    current: Option<PathBuf>,
}

impl DirectoryTracker {
    /// The current directory after `sample`, when it differs from the one last returned.
    pub fn observe(&mut self, sample: Sample) -> Option<PathBuf> {
        if let Some(report) = sample.report.filter(|report| Some(report.number) != self.seen) {
            self.seen = Some(report.number);
            if let Some(leader) = report.leader {
                self.bound = Some((leader, report.path));
            }
        }
        let reported = self.bound.as_ref().filter(|(leader, _)| *leader == sample.leader).map(|(_, path)| path.clone());
        if reported.is_none() {
            self.bound = None;
        }
        let directory = reported.or(sample.process_cwd)?;
        if self.current.as_ref() == Some(&directory) {
            return None;
        }
        self.current = Some(directory.clone());
        Some(directory)
    }
}

/// Finds working-directory reports in a program's output, also those split across reads. Like the progress
/// scanner, it runs beside the terminal's own parser, which ignores `OSC 7`.
pub struct DirectoryScanner {
    parser: Parser,
    reports: LastReport,
}

impl DirectoryScanner {
    /// A scanner accepting reports from `host` (this machine's name), from `localhost` and with no host.
    pub fn new(host: Option<String>) -> Self {
        Self { parser: Parser::new(), reports: LastReport { host, last: None } }
    }

    /// The last report completed in `bytes`, if any.
    pub fn scan(&mut self, bytes: &[u8]) -> Option<PathBuf> {
        self.parser.advance(&mut self.reports, bytes);
        self.reports.last.take()
    }
}

struct LastReport {
    host: Option<String>,
    last: Option<PathBuf>,
}

impl LastReport {
    /// The path of `file://<host>/<path>` when the host is this machine; `None` for anything else.
    fn path_of(&self, url: &[u8]) -> Option<PathBuf> {
        let rest = url.strip_prefix(b"file://")?;
        let slash = rest.iter().position(|byte| *byte == b'/')?;
        let (host, path) = rest.split_at(slash);
        let local = host.is_empty() || host == b"localhost" || self.host.as_deref().is_some_and(|name| name.as_bytes() == host);
        if !local {
            return None;
        }
        let decoded = percent_decoded(path)?;
        if decoded.len() > MAX_PATH {
            return None;
        }
        path_from_bytes(decoded)
    }
}

impl Perform for LastReport {
    fn osc_dispatch(&mut self, params: &[&[u8]], _bell_terminated: bool) {
        let [b"7", rest @ ..] = params else {
            return;
        };
        // the parser splits at `;`, which a path may hold
        if let Some(path) = self.path_of(&rest.join(&b';')) {
            self.last = Some(path);
        }
    }
}

/// `bytes` with each `%XX` turned into its byte; `None` for a malformed escape.
fn percent_decoded(bytes: &[u8]) -> Option<Vec<u8>> {
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut rest = bytes;
    while let [first, tail @ ..] = rest {
        if *first == b'%' {
            let hex = std::str::from_utf8(tail.get(..2)?).ok()?;
            decoded.push(u8::from_str_radix(hex, 16).ok()?);
            rest = &tail[2..];
        } else {
            decoded.push(*first);
            rest = tail;
        }
    }
    Some(decoded)
}

#[cfg(unix)]
fn path_from_bytes(bytes: Vec<u8>) -> Option<PathBuf> {
    use std::os::unix::ffi::OsStringExt;
    Some(PathBuf::from(std::ffi::OsString::from_vec(bytes))).filter(|path| path.is_absolute())
}

#[cfg(not(unix))]
fn path_from_bytes(bytes: Vec<u8>) -> Option<PathBuf> {
    String::from_utf8(bytes).ok().map(PathBuf::from).filter(|path| path.is_absolute())
}

/// This machine's name, as programs put it in their reports.
#[cfg(unix)]
pub fn host_name() -> Option<String> {
    let mut buffer = [0u8; 256];
    // SAFETY: gethostname writes at most `buffer.len()` bytes into the buffer
    if unsafe { libc::gethostname(buffer.as_mut_ptr().cast(), buffer.len()) } != 0 {
        return None;
    }
    let end = buffer.iter().position(|byte| *byte == 0)?;
    String::from_utf8(buffer[..end].to_vec()).ok()
}

#[cfg(not(unix))]
pub fn host_name() -> Option<String> {
    None
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn scanner() -> DirectoryScanner {
        DirectoryScanner::new(Some("kbpc".to_owned()))
    }

    fn sample(leader: i32, report: Option<(u64, &str, Option<i32>)>, process_cwd: &str) -> Sample {
        Sample {
            leader,
            report: report.map(|(number, path, reporter)| DirectoryReport { number, path: PathBuf::from(path), leader: reporter }),
            process_cwd: Some(PathBuf::from(process_cwd)),
        }
    }

    #[test]
    fn should_use_process_directory_without_report() {
        // given
        let mut tracker = DirectoryTracker::default();

        // when
        let observed = [tracker.observe(sample(10, None, "/a")), tracker.observe(sample(10, None, "/a"))];

        // then
        assert_eq!(observed, [Some(PathBuf::from("/a")), None]);
    }

    #[test]
    fn should_prefer_report_while_its_reporter_leads() {
        // given
        let mut tracker = DirectoryTracker::default();

        // when
        let observed = [tracker.observe(sample(10, Some((1, "/r", Some(10))), "/a")), tracker.observe(sample(10, Some((1, "/r", Some(10))), "/b"))];

        // then
        assert_eq!(observed, [Some(PathBuf::from("/r")), None]);
    }

    #[test]
    fn should_drop_report_when_foreground_changes() {
        // given
        let mut tracker = DirectoryTracker::default();
        tracker.observe(sample(10, Some((1, "/r", Some(10))), "/a"));

        // when
        let observed = [tracker.observe(sample(11, Some((1, "/r", Some(10))), "/shell")), tracker.observe(sample(10, Some((1, "/r", Some(10))), "/shell"))];

        // then
        assert_eq!(observed, [Some(PathBuf::from("/shell")), None]);
    }

    #[test]
    fn should_take_a_new_report_from_the_shell() {
        // given
        let mut tracker = DirectoryTracker::default();
        tracker.observe(sample(10, Some((1, "/r", Some(10))), "/a"));
        tracker.observe(sample(11, Some((1, "/r", Some(10))), "/shell"));

        // when
        let observed = tracker.observe(sample(11, Some((2, "/s2", Some(11))), "/shell"));

        // then
        assert_eq!(observed, Some(PathBuf::from("/s2")));
    }

    #[test]
    fn should_ignore_report_of_a_program_no_longer_leading() {
        // given
        let mut tracker = DirectoryTracker::default();

        // when
        // the shell's prompt reported, then Claude Code took the foreground before the poll
        let observed = tracker.observe(sample(11, Some((1, "/shell", Some(10))), "/claude"));

        // then
        assert_eq!(observed, Some(PathBuf::from("/claude")));
    }

    #[test]
    fn should_never_bind_a_replayed_report() {
        // given
        let mut tracker = DirectoryTracker::default();

        // when
        let observed = tracker.observe(sample(11, Some((1, "/before-restart", None)), "/claude"));

        // then
        assert_eq!(observed, Some(PathBuf::from("/claude")));
    }

    #[test]
    fn should_read_report_with_bel_and_st() {
        // given
        let mut scanner = scanner();

        // when
        let reports = [scanner.scan(b"\x1b]7;file:///tmp/a\x07"), scanner.scan(b"\x1b]7;file://kbpc/tmp/b\x1b\\")];

        // then
        assert_eq!(reports, [Some(PathBuf::from("/tmp/a")), Some(PathBuf::from("/tmp/b"))]);
    }

    #[test]
    fn should_read_report_split_across_reads() {
        // given
        let mut scanner = scanner();

        // when
        let first = scanner.scan(b"x\x1b]7;file:///t");
        let second = scanner.scan(b"mp/c\x07");

        // then
        assert_eq!((first, second), (None, Some(PathBuf::from("/tmp/c"))));
    }

    #[test]
    fn should_decode_percent_and_keep_semicolons() {
        // given
        let mut scanner = scanner();

        // when
        let report = scanner.scan(b"\x1b]7;file:///tmp/a%20b;c\x07");

        // then
        assert_eq!(report, Some(PathBuf::from("/tmp/a b;c")));
    }

    #[test]
    fn should_accept_localhost_and_ignore_foreign_host() {
        // given
        let mut scanner = scanner();

        // when
        let reports = [scanner.scan(b"\x1b]7;file://localhost/x\x07"), scanner.scan(b"\x1b]7;file://other/x\x07")];

        // then
        assert_eq!(reports, [Some(PathBuf::from("/x")), None]);
    }

    #[test]
    fn should_ignore_relative_or_overlong_paths_and_other_osc() {
        // given
        let mut scanner = scanner();
        let overlong = format!("\x1b]7;file:///{}\x07", "a".repeat(MAX_PATH + 1));

        // when
        let reports = [
            scanner.scan(b"\x1b]7;file://relative\x07"),
            scanner.scan(overlong.as_bytes()),
            scanner.scan(b"\x1b]9;4;1;5\x07"),
            scanner.scan(b"\x1b]7;http://kbpc/x\x07"),
        ];

        // then
        assert_eq!(reports, [None, None, None, None]);
    }
}
