//! What the hub knows of each checkout's files and changes, and which checkouts are due for a read: kept by the
//! workspace, read off the UI thread.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::files::CheckoutFiles;
use crate::git::status::Change;
use crate::run::file::RunFile;

/// A checkout read longer ago than this is read again when the hub opens: files appear in folders no watcher sees.
pub const STALE_INDEX: Duration = Duration::from_secs(30);
/// A checkout that changed is read again at most this often: a build writing files must not keep a read running.
pub const REREAD_FLOOR: Duration = Duration::from_secs(2);

/// One read of a checkout.
#[derive(Debug, Clone, Default)]
pub struct Read {
    pub files: Arc<CheckoutFiles>,
    /// `None` when not asked for
    pub ignored: Option<Arc<CheckoutFiles>>,
    pub changes: Arc<Vec<Change>>,
    /// the `.run.toml` files: folder (checkout-relative, `` at the top) and what it declares
    pub configs: Arc<Vec<(String, Arc<RunFile>)>>,
    /// read in the main checkout only: branches without a worktree, as git prints them
    pub branches: Arc<Vec<Branch>>,
}

/// A branch without a worktree: its name, the local name a worktree of it would check out, the remote of a branch
/// only a remote has, and its last commit (seconds since the epoch).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Branch {
    pub shown: String,
    pub local: String,
    pub remote: Option<String>,
    pub when: i64,
}

#[derive(Debug, Default)]
struct Indexed {
    read: Option<Read>,
    read_at: Option<Instant>,
    /// why the last read failed, until one succeeds
    failure: Option<String>,
    dirty: bool,
    reading: bool,
}

#[derive(Debug, Default)]
pub struct HubIndex {
    checkouts: HashMap<PathBuf, Indexed>,
    /// the Excluded switch was on once this session: ignored files are read from then on
    ignored: bool,
}

impl HubIndex {
    /// Of `checkouts`, those to read now: never read, changed since, or read longer than [`STALE_INDEX`] ago; none
    /// being read already.
    pub fn due(&self, checkouts: &[PathBuf], now: Instant) -> Vec<PathBuf> {
        let due = |path: &&PathBuf| match self.checkouts.get(*path) {
            None => true,
            Some(indexed) if indexed.reading => false,
            Some(indexed) => {
                let age = indexed.read_at.map(|at| now.saturating_duration_since(at));
                let stale = age.is_none_or(|age| age > STALE_INDEX);
                let changed = indexed.dirty && age.is_none_or(|age| age >= REREAD_FLOOR);
                changed || stale || indexed.read.as_ref().is_some_and(|read| self.ignored && read.ignored.is_none())
            }
        };
        checkouts.iter().filter(due).cloned().collect()
    }

    /// Only the checkouts of `due` that were never read: what opening a project reads at once.
    pub fn unread(&self, checkouts: &[PathBuf]) -> Vec<PathBuf> {
        checkouts.iter().filter(|path| !self.checkouts.contains_key(*path)).cloned().collect()
    }

    pub fn started(&mut self, checkout: &Path) {
        let indexed = self.checkouts.entry(checkout.to_path_buf()).or_default();
        indexed.reading = true;
        indexed.dirty = false;
    }

    /// A read of a checkout `started` gave this; one forgotten meanwhile (its project closed) is dropped.
    pub fn read(&mut self, checkout: &Path, read: Read, at: Instant) {
        if let Some(indexed) = self.checkouts.get_mut(checkout) {
            indexed.read = Some(read);
            indexed.read_at = Some(at);
            indexed.failure = None;
            indexed.reading = false;
        }
    }

    /// A read failed with `error`: the checkout is read again next time.
    pub fn failed(&mut self, checkout: &Path, error: String) {
        if let Some(indexed) = self.checkouts.get_mut(checkout) {
            indexed.reading = false;
            indexed.dirty = true;
            indexed.failure = Some(error);
        }
    }

    /// Why the last read of `checkout` failed; `None` once one succeeded.
    pub fn failure(&self, checkout: &Path) -> Option<&str> {
        self.checkouts.get(checkout)?.failure.as_deref()
    }

    pub fn mark_dirty(&mut self, checkout: &Path) {
        if let Some(indexed) = self.checkouts.get_mut(checkout) {
            indexed.dirty = true;
        }
    }

    pub fn want_ignored(&mut self) {
        self.ignored = true;
    }

    pub fn wants_ignored(&self) -> bool {
        self.ignored
    }

    pub fn retain(&mut self, open: &[PathBuf]) {
        self.checkouts.retain(|path, _| open.contains(path));
    }

    pub fn files(&self, checkout: &Path) -> Option<&Read> {
        self.checkouts.get(checkout)?.read.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    fn path(name: &str) -> PathBuf {
        PathBuf::from(format!("/{name}"))
    }

    #[test]
    fn should_mark_only_stale_unread_or_dirty_checkouts_for_reading() {
        // given
        let now = Instant::now();
        let mut index = HubIndex::default();
        for (name, ago) in [("old", 31), ("fresh", 5), ("dirty", 5), ("reading", 40)] {
            index.started(&path(name));
            index.read(&path(name), Read::default(), now - Duration::from_secs(ago));
        }
        index.mark_dirty(&path("dirty"));
        index.started(&path("reading"));
        let checkouts = ["old", "fresh", "dirty", "reading", "never"].map(path);

        // when
        let due = index.due(&checkouts, now);

        // then
        assert_eq!(due, ["old", "dirty", "never"].map(path));
    }

    #[test]
    fn should_keep_ignored_files_once_asked_for() {
        // given
        let mut index = HubIndex::default();

        // when
        let before = index.wants_ignored();
        index.want_ignored();

        // then
        assert_eq!((before, index.wants_ignored()), (false, true));
    }

    #[test]
    fn should_forget_checkouts_no_longer_open() {
        // given
        let mut index = HubIndex::default();
        for name in ["a", "b"] {
            index.started(&path(name));
            index.read(&path(name), Read::default(), Instant::now());
        }

        // when
        index.retain(&[path("b")]);

        // then
        assert_eq!((index.files(&path("a")).is_none(), index.files(&path("b")).is_some()), (true, true));
    }

    #[test]
    fn should_wait_before_reading_a_changed_checkout_again() {
        // given
        let now = Instant::now();
        let mut index = HubIndex::default();
        for (name, ago) in [("just", 1), ("while", 3)] {
            index.started(&path(name));
            index.read(&path(name), Read::default(), now - Duration::from_secs(ago));
            index.mark_dirty(&path(name));
        }

        // when
        let due = index.due(&[path("just"), path("while")], now);

        // then
        assert_eq!(due, [path("while")]);
    }

    #[test]
    fn should_not_bring_back_a_checkout_forgotten_while_it_was_read() {
        // given
        let mut index = HubIndex::default();
        index.started(&path("a"));
        index.retain(&[]);

        // when
        index.read(&path("a"), Read::default(), Instant::now());
        index.failed(&path("a"), "gone".to_owned());

        // then
        assert!(index.files(&path("a")).is_none());
        assert_eq!(index.unread(&[path("a")]), [path("a")]);
    }

    #[test]
    fn should_keep_why_a_read_failed_until_one_succeeds() {
        // given
        let mut index = HubIndex::default();
        index.started(&path("a"));

        // when
        index.failed(&path("a"), "Permission denied".to_owned());
        let failed = index.failure(&path("a")).map(str::to_owned);
        index.started(&path("a"));
        index.read(&path("a"), Read::default(), Instant::now());

        // then
        assert_eq!(failed.as_deref(), Some("Permission denied"));
        assert_eq!(index.failure(&path("a")), None);
    }
}
