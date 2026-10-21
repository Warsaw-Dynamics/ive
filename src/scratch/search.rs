//! Searching scratches by name and content.

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::SystemTime;

use super::groups::name_matches;
use super::store::ScratchInfo;

/// Content search reads this much of each scratch.
pub const CONTENT_PROBE_BYTES: u64 = 256 * 1024;

/// Whether one scratch matched, as it was when searched.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Answer {
    pub name: String,
    pub modified: SystemTime,
    pub matched: bool,
}

/// For each scratch, whether its name or first `CONTENT_PROBE_BYTES` contain `query`, ignoring case; `None` as
/// soon as `ticket` is no longer the newest search. Reads files: run it on the background executor.
pub fn search(dir: &Path, scratches: &[ScratchInfo], query: &str, ticket: &Ticket) -> Option<Vec<Answer>> {
    let needle = query.to_lowercase();
    let mut answers = Vec::with_capacity(scratches.len());
    for scratch in scratches {
        if ticket.is_stale() {
            return None;
        }
        let matched = name_matches(&scratch.name, query) || content_contains(&dir.join(&scratch.name), &needle);
        answers.push(Answer { name: scratch.name.clone(), modified: scratch.modified, matched });
    }
    Some(answers)
}

fn content_contains(path: &Path, needle: &str) -> bool {
    let mut probe = Vec::new();
    let read = File::open(path).and_then(|file| file.take(CONTENT_PROBE_BYTES).read_to_end(&mut probe));
    read.is_ok() && String::from_utf8_lossy(&probe).to_lowercase().contains(needle)
}

/// The answers for one query, kept while it stays: a new listing searches only scratches that are new or were
/// modified since (an IVE save re-reads one file, not the folder).
#[derive(Debug, Default)]
pub struct Found {
    seen: HashMap<String, Answer>,
}

impl Found {
    /// The scratches without an answer for their current modification time.
    pub fn stale(&self, scratches: &[ScratchInfo]) -> Vec<ScratchInfo> {
        scratches.iter().filter(|scratch| self.answer(scratch).is_none()).cloned().collect()
    }

    pub fn record(&mut self, answers: Vec<Answer>) {
        self.seen.extend(answers.into_iter().map(|answer| (answer.name.clone(), answer)));
    }

    /// The names that match, in the listing's order; a scratch not searched yet matches by its name.
    pub fn names(&self, scratches: &[ScratchInfo], query: &str) -> Vec<String> {
        let matches = |scratch: &&ScratchInfo| self.answer(scratch).map_or_else(|| name_matches(&scratch.name, query), |answer| answer.matched);
        scratches.iter().filter(matches).map(|scratch| scratch.name.clone()).collect()
    }

    fn answer(&self, scratch: &ScratchInfo) -> Option<&Answer> {
        self.seen.get(&scratch.name).filter(|answer| answer.modified == scratch.modified)
    }
}

/// Numbers searches so that only the newest one's result applies and an older one stops reading.
#[derive(Debug, Clone, Default)]
pub struct Generations {
    latest: Arc<AtomicU64>,
}

impl Generations {
    pub fn next(&self) -> Ticket {
        let number = self.latest.fetch_add(1, Ordering::SeqCst) + 1;
        Ticket { latest: self.latest.clone(), number }
    }
}

/// One search's number; it can be checked from the background executor.
#[derive(Debug, Clone)]
pub struct Ticket {
    latest: Arc<AtomicU64>,
    number: u64,
}

impl Ticket {
    /// A newer search has started.
    pub fn is_stale(&self) -> bool {
        self.latest.load(Ordering::SeqCst) != self.number
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::Duration;

    use super::*;

    fn info(name: &str) -> ScratchInfo {
        ScratchInfo { name: name.to_owned(), modified: SystemTime::UNIX_EPOCH, first_line: String::new() }
    }

    fn matched(answers: Option<Vec<Answer>>) -> Vec<String> {
        answers.unwrap().into_iter().filter(|answer| answer.matched).map(|answer| answer.name).collect()
    }

    #[test]
    fn should_find_scratches_by_content_ignoring_case() {
        // given
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("idp-users.sql"), "SELECT Email FROM users").unwrap();
        fs::write(dir.path().join("notes.md"), "nothing here").unwrap();

        // when
        let found = search(dir.path(), &[info("idp-users.sql"), info("notes.md")], "email", &Generations::default().next());

        // then
        assert_eq!(matched(found), ["idp-users.sql"]);
    }

    #[test]
    fn should_match_names_even_when_the_file_cannot_be_read() {
        // given
        let dir = tempfile::tempdir().unwrap();

        // when
        let found = search(dir.path(), &[info("Notes.md")], "notes", &Generations::default().next());

        // then
        assert_eq!(matched(found), ["Notes.md"]);
    }

    #[test]
    fn should_not_look_past_the_first_256_kib() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let cap = CONTENT_PROBE_BYTES as usize;
        fs::write(dir.path().join("inside.txt"), "a".repeat(cap - 6) + "needle").unwrap();
        fs::write(dir.path().join("beyond.txt"), "a".repeat(cap) + "needle").unwrap();

        // when
        let found = search(dir.path(), &[info("inside.txt"), info("beyond.txt")], "needle", &Generations::default().next());

        // then
        assert_eq!(matched(found), ["inside.txt"]);
    }

    #[test]
    fn should_stop_a_search_once_a_newer_one_starts() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let generations = Generations::default();
        let stale = generations.next();
        let newest = generations.next();

        // when
        let found = search(dir.path(), &[info("a.md")], "a", &stale);

        // then
        assert_eq!((found, stale.is_stale(), newest.is_stale()), (None, true, false));
    }

    #[test]
    fn should_search_again_only_new_or_changed_scratches() {
        // given
        let mut found = Found::default();
        found.record(vec![
            Answer { name: "kept.md".to_owned(), modified: SystemTime::UNIX_EPOCH, matched: true },
            Answer { name: "saved.md".to_owned(), modified: SystemTime::UNIX_EPOCH, matched: false },
        ]);
        let mut saved = info("saved.md");
        saved.modified += Duration::from_secs(1);
        let listing = [info("kept.md"), saved, info("new.md")];

        // when
        let stale: Vec<String> = found.stale(&listing).into_iter().map(|scratch| scratch.name).collect();

        // then
        assert_eq!(stale, ["saved.md", "new.md"]);
    }

    #[test]
    fn should_match_scratches_not_searched_yet_by_name() {
        // given
        let mut found = Found::default();
        found.record(vec![Answer { name: "by-content.sql".to_owned(), modified: SystemTime::UNIX_EPOCH, matched: true }]);
        let listing = [info("by-content.sql"), info("users.md"), info("notes.md")];

        // when
        let names = found.names(&listing, "user");

        // then
        assert_eq!(names, ["by-content.sql", "users.md"]);
    }
}
