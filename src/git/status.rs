use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock, Mutex, OnceLock, PoisonError};
use std::time::{Duration, Instant};

use git2::{Index, Status, StatusEntry, StatusOptions};

use super::repo::{self, GitError};
use super::types::{GitPath, Mark, failed};

/// A path that differs between HEAD and the work tree, staged or not.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    pub mark: Mark,
    pub path: GitPath,
    /// the name before a rename
    pub old_path: Option<GitPath>,
}

/// The checkout's uncommitted changes, untracked files included, sorted by path. A nested repository or
/// checkout (a worktree under `.claude/worktrees` that git does not ignore) is one untracked folder row.
#[cfg(test)]
pub fn changes(dir: &Path) -> Result<Vec<Change>, GitError> {
    read(dir).map(|read| Arc::unwrap_or_clone(read.changes))
}

fn change(entry: &StatusEntry<'_>) -> Option<Change> {
    let mark = mark(entry.status())?;
    let staged = entry.head_to_index();
    let unstaged = entry.index_to_workdir();
    // `entry.path()` is the old name of a rename; the new name is on the newest side
    let path = unstaged.as_ref().or(staged.as_ref()).and_then(|delta| delta.new_file().path_bytes()).unwrap_or(entry.path_bytes());
    let old = staged.as_ref().or(unstaged.as_ref()).and_then(|delta| delta.old_file().path_bytes());
    let old_path = old.filter(|old| mark == Mark::Renamed && *old != path).map(GitPath::from_bytes);
    Some(Change { mark, path: GitPath::from_bytes(path), old_path })
}

/// What the file tree draws from git, by path relative to the checkout: each changed file's mark, the folders git
/// does not track (everything in them is unversioned), and what git ignores. A folder git ignores or does not track
/// is one entry, never its contents.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TreeStatus {
    pub marks: HashMap<PathBuf, Mark>,
    pub untracked: HashSet<PathBuf>,
    pub ignored: HashSet<PathBuf>,
}

/// The checkout's [`TreeStatus`].
#[cfg(test)]
pub fn tree_status(dir: &Path) -> Result<TreeStatus, GitError> {
    read(dir).map(|read| Arc::unwrap_or_clone(read.tree))
}

/// A checkout's status, read once for every view showing it: the changes, untracked files one by one, and the
/// file tree's [`TreeStatus`].
#[derive(Debug, Clone)]
pub struct StatusRead {
    pub changes: Arc<Vec<Change>>,
    pub tree: Arc<TreeStatus>,
}

/// Reads [`StatusRead`]: untracked folders are walked, ignored ones are one entry each.
#[tracing::instrument(level = "debug", name = "git_status", skip_all, fields(dir = %dir.display()))]
pub fn read(dir: &Path) -> Result<StatusRead, GitError> {
    let fail = failed("read the changes", dir);
    let repo = repo::open(dir)?;
    let mut options = StatusOptions::new();
    options
        .include_untracked(true)
        .recurse_untracked_dirs(true)
        .include_ignored(true)
        .recurse_ignored_dirs(false)
        .exclude_submodules(true)
        .renames_head_to_index(true)
        .renames_index_to_workdir(true);
    let statuses = repo.statuses(Some(&mut options)).map_err(&fail)?;
    let mut changes = Vec::new();
    let mut ignored = Vec::new();
    for entry in statuses.iter() {
        if entry.status().is_ignored() {
            ignored.push(relative_path(entry.path_bytes()));
        } else if let Some(change) = change(&entry) {
            changes.push(change);
        }
    }
    changes.sort_by(|a, b| a.path.cmp(&b.path));
    let index = repo.index().map_err(&fail)?;
    let tree = tree_of(&changes, ignored, &index);
    Ok(StatusRead { changes: Arc::new(changes), tree: Arc::new(tree) })
}

/// The tree's view of a walked status, as a status that does not walk untracked folders reports it: a folder the
/// index has nothing in is one untracked entry when something untracked is in it, else one ignored entry.
fn tree_of(changes: &[Change], ignored: Vec<PathBuf>, index: &Index) -> TreeStatus {
    let mut tracked = TrackedFolders { index, known: HashMap::new() };
    let mut status = TreeStatus::default();
    let mut holding_untracked: HashSet<PathBuf> = HashSet::new();
    for change in changes {
        let path = relative_path(change.path.as_bytes());
        if change.mark != Mark::Untracked {
            status.marks.insert(path, change.mark);
            continue;
        }
        holding_untracked.extend(folders_of(&path).map(Path::to_path_buf));
        match (tracked.topmost_untracked(&path, |_| true), change.path.is_folder()) {
            (Some(folder), _) => status.untracked.insert(folder),
            (None, true) => status.untracked.insert(path),
            (None, false) => status.marks.insert(path, Mark::Untracked).is_none(),
        };
    }
    for path in ignored {
        if folders_of(&path).any(|folder| status.untracked.contains(folder)) {
            continue;
        }
        let folder = tracked.topmost_untracked(&path, |folder| !holding_untracked.contains(folder));
        status.ignored.insert(folder.unwrap_or(path));
    }
    status
}

/// `path`'s folders, the topmost last; without the checkout itself.
fn folders_of(path: &Path) -> impl DoubleEndedIterator<Item = &Path> {
    let folders: Vec<&Path> = path.ancestors().skip(1).filter(|folder| !folder.as_os_str().is_empty()).collect();
    folders.into_iter()
}

/// Whether the index has anything under a folder, asked once per folder.
struct TrackedFolders<'a> {
    index: &'a Index,
    known: HashMap<PathBuf, bool>,
}

impl TrackedFolders<'_> {
    /// The topmost folder of `path` that the index has nothing in and `also` accepts, as git meets it walking down.
    fn topmost_untracked(&mut self, path: &Path, also: impl Fn(&Path) -> bool) -> Option<PathBuf> {
        let found = folders_of(path).rev().find(|folder| also(folder) && !self.has_tracked(folder));
        found.map(Path::to_path_buf)
    }

    fn has_tracked(&mut self, folder: &Path) -> bool {
        let index = self.index;
        *self.known.entry(folder.to_path_buf()).or_insert_with(|| {
            let mut prefix = folder.as_os_str().as_bytes().to_vec();
            prefix.push(b'/');
            index.find_prefix(prefix).is_ok()
        })
    }
}

/// How long a read serves the readers that accept it.
const KEEP_READS: Duration = Duration::from_secs(10);

struct SharedRead {
    started: Instant,
    result: Arc<OnceLock<Result<StatusRead, GitError>>>,
}

static READS: LazyLock<Mutex<HashMap<PathBuf, SharedRead>>> = LazyLock::new(Mutex::default);

/// `dir`'s status from a read that started at or after `since`, waiting for one in progress; otherwise reads it.
/// The tree, its git folders, the hub and the project's row each want a checkout's status after the same change,
/// and on the atlassian monorepo each read takes ~130 ms.
pub fn shared(dir: &Path, since: Instant) -> Result<StatusRead, GitError> {
    let (result, mine) = claim(dir, since);
    if mine {
        let read = std::panic::catch_unwind(|| read(dir));
        let failed = || Err(GitError { operation: "read the changes".to_owned(), dir: dir.to_path_buf(), message: "the read panicked".to_owned() });
        // waiting readers get an error rather than waiting forever
        let _ = result.set(read.as_ref().map_or_else(|_| failed(), Clone::clone));
        if let Err(panic) = read {
            std::panic::resume_unwind(panic);
        }
    }
    result.wait().clone()
}

/// The read to share, and whether the caller starts it.
fn claim(dir: &Path, since: Instant) -> (Arc<OnceLock<Result<StatusRead, GitError>>>, bool) {
    let mut reads = READS.lock().unwrap_or_else(PoisonError::into_inner);
    let now = Instant::now();
    reads.retain(|_, read| now.duration_since(read.started) < KEEP_READS);
    if let Some(read) = reads.get(dir).filter(|read| read.started >= since) {
        return (read.result.clone(), false);
    }
    let result = Arc::new(OnceLock::new());
    reads.insert(dir.to_path_buf(), SharedRead { started: now, result: result.clone() });
    (result, true)
}

/// git's path bytes as a relative path, without the `/` git puts after a folder.
fn relative_path(bytes: &[u8]) -> PathBuf {
    let bytes = bytes.strip_suffix(b"/").unwrap_or(bytes);
    PathBuf::from(OsStr::from_bytes(bytes))
}

/// `None` for entries that are not changes (current, ignored, unreadable).
fn mark(status: Status) -> Option<Mark> {
    if status.is_conflicted() {
        return Some(Mark::Conflicted);
    }
    if status.intersects(Status::INDEX_RENAMED | Status::WT_RENAMED) {
        return Some(Mark::Renamed);
    }
    if status.intersects(Status::INDEX_TYPECHANGE | Status::WT_TYPECHANGE) {
        return Some(Mark::TypeChange);
    }
    if status.is_wt_new() {
        return Some(Mark::Untracked);
    }
    if status.is_index_new() {
        return Some(Mark::Added);
    }
    if status.intersects(Status::INDEX_DELETED | Status::WT_DELETED) {
        return Some(Mark::Deleted);
    }
    status.intersects(Status::INDEX_MODIFIED | Status::WT_MODIFIED).then_some(Mark::Modified)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::testing::{TestRepo, add_worktree};

    fn marks(changes: &[Change]) -> Vec<(&'static str, String)> {
        changes.iter().map(|change| (change.mark.letter(), change.path.display())).collect()
    }

    #[test]
    fn should_list_modified_added_deleted_and_untracked_changes() {
        // given
        let repo = TestRepo::init();
        repo.write("a.txt", "a\n");
        repo.write("b.txt", "b\n");
        repo.commit("first");
        repo.write("a.txt", "a changed\n");
        repo.remove("b.txt");
        repo.write("new.txt", "staged\n");
        repo.stage("new.txt");
        repo.write("new folder/x.txt", "untracked\n");
        repo.write_bytes(Path::new("data.bin"), &[0, 1, 2, 0]);

        // when
        let changes = changes(repo.path()).expect("changes");

        // then
        assert_eq!(marks(&changes), [
            ("M", "a.txt".to_owned()),
            ("D", "b.txt".to_owned()),
            ("?", "data.bin".to_owned()),
            ("?", "new folder/x.txt".to_owned()),
            ("A", "new.txt".to_owned()),
        ]);
    }

    #[test]
    fn should_report_renames_with_both_names_staged_or_not() {
        // given
        let repo = TestRepo::init();
        let text = "line 1\nline 2\nline 3\nline 4\nline 5\n";
        repo.write("staged.txt", text);
        repo.write("unstaged.txt", &text.replace("line", "row"));
        repo.commit("first");
        repo.rename("staged.txt", "staged-new.txt");
        repo.stage_all();
        repo.rename("unstaged.txt", "unstaged-new.txt");

        // when
        let changes = changes(repo.path()).expect("changes");

        // then
        let renames: Vec<_> = changes
            .iter()
            .map(|change| (change.mark, change.path.display(), change.old_path.as_ref().map(GitPath::display)))
            .collect();
        assert_eq!(renames, [
            (Mark::Renamed, "staged-new.txt".to_owned(), Some("staged.txt".to_owned())),
            (Mark::Renamed, "unstaged-new.txt".to_owned(), Some("unstaged.txt".to_owned())),
        ]);
    }

    #[test]
    fn should_report_ignored_folder_without_its_contents() {
        // given
        let repo = TestRepo::init();
        repo.write(".gitignore", "target/\n*.log\n");
        repo.commit("first");
        repo.write("target/a/b.class", "x\n");
        repo.write("target/c.class", "x\n");
        repo.write("debug.log", "x\n");

        // when
        let status = tree_status(repo.path()).expect("status");

        // then
        assert_eq!(status.ignored, HashSet::from([PathBuf::from("target"), PathBuf::from("debug.log")]));
    }

    #[test]
    fn should_report_untracked_folder_without_its_contents() {
        // given
        let repo = TestRepo::init();
        repo.write("a.txt", "a\n");
        repo.commit("first");
        repo.write("node_modules/left-pad/index.js", "x\n");
        repo.write("node_modules/b.js", "x\n");
        repo.write("notes.txt", "x\n");

        // when
        let status = tree_status(repo.path()).expect("status");

        // then
        assert_eq!(status.untracked, HashSet::from([PathBuf::from("node_modules")]));
        let marks: Vec<String> = status.marks.keys().map(|path| path.display().to_string()).collect();
        assert_eq!(marks, ["notes.txt"]);
    }

    #[test]
    fn should_share_a_read_started_since_the_caller_asked_and_read_again_for_a_later_one() {
        // given
        let repo = TestRepo::init();
        repo.write("a.txt", "a\n");
        repo.commit("first");
        let asked = Instant::now();
        let first = shared(repo.path(), asked).expect("first read");

        // when
        repo.write("b.txt", "b\n");
        let shared_read = shared(repo.path(), asked).expect("shared read");
        let fresh = shared(repo.path(), Instant::now()).expect("fresh read");

        // then
        assert!(Arc::ptr_eq(&first.changes, &shared_read.changes));
        assert_eq!((first.changes.len(), marks(&fresh.changes)), (0, vec![("?", "b.txt".to_owned())]));
    }

    #[test]
    fn should_report_a_folder_holding_only_ignored_files_as_ignored() {
        // given
        let repo = TestRepo::init();
        repo.write(".gitignore", "*.cache\n");
        repo.write("src/a.txt", "a\n");
        repo.commit("first");
        repo.write("build/out/x.cache", "x\n");
        repo.write("src/y.cache", "y\n");

        // when
        let status = tree_status(repo.path()).expect("status");

        // then
        assert_eq!(status.ignored, HashSet::from([PathBuf::from("build"), PathBuf::from("src/y.cache")]));
    }

    #[test]
    fn should_report_the_topmost_untracked_folder_inside_a_tracked_one_and_not_what_it_ignores() {
        // given
        let repo = TestRepo::init();
        repo.write(".gitignore", "*.log\n");
        repo.write("src/a.txt", "a\n");
        repo.commit("first");
        repo.write("src/new/deep/b.txt", "b\n");
        repo.write("src/new/c.log", "c\n");

        // when
        let status = tree_status(repo.path()).expect("status");

        // then
        assert_eq!((status.untracked, status.ignored, status.marks.len()), (HashSet::from([PathBuf::from("src/new")]), HashSet::new(), 0));
    }

    #[test]
    fn should_mark_modified_untracked_and_renamed_files_for_the_tree() {
        // given
        let repo = TestRepo::init();
        repo.write("a.txt", "a\n");
        repo.write("old.txt", "line 1\nline 2\nline 3\n");
        repo.commit("first");
        repo.write("a.txt", "a changed\n");
        repo.write("dir/b.txt", "b\n");
        repo.rename("old.txt", "new.txt");
        repo.stage_all();

        // when
        let status = tree_status(repo.path()).expect("status");

        // then
        let marks: HashMap<_, _> = status.marks.iter().map(|(path, mark)| (path.display().to_string(), *mark)).collect();
        assert_eq!(marks, HashMap::from([
            ("a.txt".to_owned(), Mark::Modified),
            ("dir/b.txt".to_owned(), Mark::Added),
            ("new.txt".to_owned(), Mark::Renamed),
        ]));
    }

    #[test]
    fn should_list_files_of_a_repository_without_commits() {
        // given
        let repo = TestRepo::init();
        repo.write("a.txt", "a\n");

        // when
        let changes = changes(repo.path()).expect("changes");

        // then
        assert_eq!(marks(&changes), [("?", "a.txt".to_owned())]);
    }

    #[test]
    fn should_list_a_nested_checkout_as_one_untracked_folder() {
        // given
        let repo = TestRepo::init();
        repo.write("a.txt", "a\n");
        repo.commit("first");
        let nested = repo.path().join(".claude/worktrees/topic");
        std::fs::create_dir_all(nested.parent().unwrap()).unwrap();
        let worktree = add_worktree(&repo.repo, "topic", &nested);
        std::fs::write(nested.join("inside.txt"), "x\n").unwrap();

        // when
        let changes = changes(repo.path()).expect("changes");

        // then
        assert_eq!(marks(&changes), [("?", ".claude/worktrees/topic/".to_owned())]);
        assert!(changes[0].path.is_folder() && worktree.workdir().is_some());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn should_keep_non_utf8_path_bytes() {
        // given
        use std::os::unix::ffi::OsStrExt;
        let repo = TestRepo::init();
        repo.write_bytes(Path::new(std::ffi::OsStr::from_bytes(b"caf\xe9.txt")), b"x\n");

        // when
        let changes = changes(repo.path()).expect("changes");

        // then
        assert_eq!(changes[0].path.as_bytes(), b"caf\xe9.txt");
    }

    #[cfg(unix)]
    #[test]
    fn should_mark_conflicts_type_changes_and_staged_files_changed_again() {
        // given
        let repo = TestRepo::init();
        repo.write("clash.txt", "base\n");
        repo.write("link.txt", "a file\n");
        repo.commit("base");
        repo.branch("side");
        repo.checkout("side");
        repo.write("clash.txt", "side\n");
        repo.commit("on side");
        repo.checkout("main");
        repo.write("clash.txt", "main\n");
        repo.commit("on main");
        repo.merge_into_work_tree("side");
        repo.remove("link.txt");
        std::os::unix::fs::symlink("clash.txt", repo.path().join("link.txt")).unwrap();
        repo.write("new.txt", "staged\n");
        repo.stage("new.txt");
        repo.write("new.txt", "staged, then changed\n");

        // when
        let changes = changes(repo.path()).expect("changes");

        // then
        assert_eq!(marks(&changes), [("U", "clash.txt".to_owned()), ("T", "link.txt".to_owned()), ("A", "new.txt".to_owned())]);
    }
}

