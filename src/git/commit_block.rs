//! Commits one change of a file and nothing else: HEAD's version of the file with only that change in it. Runs the
//! `git` command, so the user's hooks, signing and config apply; the commit is built in a temporary index, so other
//! staged files stay staged and out of it.

use std::fmt;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, PoisonError};
use std::time::{SystemTime, UNIX_EPOCH};

/// The new content of one file to commit on top of HEAD.
pub struct BlockCommit {
    pub path: PathBuf,
    /// the HEAD the content was made from: a commit refuses when HEAD is another by now
    pub head_id: String,
    /// the whole file in working-tree form: HEAD's text with the one change applied
    pub text: Vec<u8>,
    pub message: String,
}

/// Why the change is not committed; each says it for the user.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    EmptyMessage,
    NotInRepository(String),
    NoHead,
    HeadMoved,
    /// a merge, rebase, cherry-pick or revert, named
    InProgress(&'static str),
    StagedChanges,
    StagedNew,
    NotInHead,
    NotAFile,
    /// a `filter` or `working-tree-encoding` attribute applies to the file
    Filtered,
    /// HEAD moved between the check and the commit, and taking the commit off it again failed: `commit` stays on
    /// `parent`
    UndoFailed { commit: String, parent: String, stderr: String },
    /// the `git` command could not be started
    GitMissing(String),
    /// a `git` command failed: what it was doing, and its stderr
    Git { doing: &'static str, stderr: String },
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyMessage => write!(f, "Write a commit message first"),
            Self::NotInRepository(reason) => write!(f, "The file is not in a git work tree: {reason}"),
            Self::NoHead => write!(f, "The repository has no commit yet"),
            Self::HeadMoved => write!(f, "HEAD moved since the changes were found; try again once they are updated"),
            Self::InProgress(what) => write!(f, "A {what} is in progress; finish it first"),
            Self::StagedChanges => write!(f, "The file has staged changes; commit or unstage them from git"),
            Self::StagedNew => write!(f, "The file is staged as new; commit it from git"),
            Self::NotInHead => write!(f, "The file is not in HEAD"),
            Self::NotAFile => write!(f, "The file is not a regular file in HEAD"),
            Self::Filtered => write!(f, "The file uses a git filter; commit it from git"),
            Self::UndoFailed { commit, parent, stderr } => write!(
                f,
                "HEAD moved while committing: commit {commit} went on top of {parent}, not on the HEAD the change was \
                 made from, and undoing it failed ({}). If HEAD is still {commit}, undo it with `git reset --soft \
                 {parent}`; your files stay as they are",
                stderr.trim()
            ),
            Self::GitMissing(reason) => write!(f, "Cannot run git: {reason}"),
            Self::Git { doing, stderr } => write!(f, "git failed to {doing}: {}", stderr.trim()),
        }
    }
}

/// Commits `commit.text` as the new content of its file with message `commit.message`, on HEAD, and nothing else.
/// The file's index entry follows the commit when it had nothing staged. A commit git puts on another HEAD than
/// `commit.head_id` is taken off again ([`Refusal::HeadMoved`]); the work tree is never touched.
pub fn commit_block(git: &Git, commit: &BlockCommit) -> Result<(), Refusal> {
    if commit.message.trim().is_empty() {
        return Err(Refusal::EmptyMessage);
    }
    // IVE's commits one at a time: each checks HEAD and commits on it before the next looks
    let _alone = COMMITTING.lock().unwrap_or_else(PoisonError::into_inner);
    let place = git.locate(&commit.path)?;
    if git.head(&place)? != commit.head_id {
        return Err(Refusal::HeadMoved);
    }
    git.check_nothing_in_progress(&place)?;
    let entry = git.head_entry(&place, &commit.head_id)?;
    git.check_unfiltered(&place)?;
    git.check_nothing_staged(&place)?;
    let blob = git.store(&place, &commit.text)?;
    let made = git.commit_entry(&place, &Entry { blob, ..entry.clone() }, commit)?;
    git.check_parent(&place, &made, &commit.head_id)?;
    if let Err(refusal) = git.follow_in_index(&place, &entry, &made.id) {
        eprintln!("ive: committed {}, but its index entry stayed as it was: {refusal}", place.name);
    }
    Ok(())
}

/// How to run the `git` command: the environment it gets on top of IVE's.
pub struct Git {
    env: Vec<(String, String)>,
    /// runs right before `git commit`, where another commit may land
    #[cfg(test)]
    before_commit: Box<dyn Fn() + Send + Sync>,
}

/// A file in its repository: the work tree's top folder and the file's name there, with `/`.
struct Place {
    top: PathBuf,
    name: String,
}

/// A file's entry in a tree or an index.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Entry {
    mode: String,
    blob: String,
}

/// A commit IVE made, and the parent git gave it.
struct Made {
    id: String,
    parent: String,
}

/// One run of `git` in a work tree.
struct Call<'a> {
    args: &'a [&'a str],
    stdin: Option<&'a [u8]>,
    /// the index to use instead of the repository's
    index: Option<&'a Path>,
    /// what a commit's reflog entry says it was, instead of `commit`
    reflog_action: Option<&'a str>,
}

impl<'a> Call<'a> {
    fn of(args: &'a [&'a str]) -> Self {
        Self { args, stdin: None, index: None, reflog_action: None }
    }
}

/// What each operation in progress leaves in the git dir, and its name.
const IN_PROGRESS: [(&str, &str); 5] = [
    ("MERGE_HEAD", "merge"),
    ("rebase-merge", "rebase"),
    ("rebase-apply", "rebase"),
    ("CHERRY_PICK_HEAD", "cherry-pick"),
    ("REVERT_HEAD", "revert"),
];

/// Where git finds a repository, which IVE may have been started with (from a hook, `git rebase -x`): the
/// repository is always the file's.
const INHERITED: [&str; 7] = [
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_INDEX_FILE",
    "GIT_PREFIX",
    "GIT_OBJECT_DIRECTORY",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    "GIT_COMMON_DIR",
];

/// How many of HEAD's newest reflog entries are searched for IVE's commit.
const REFLOG_SEARCHED: &str = "64";

/// Numbers the commits of this process: their temporary indexes and reflog actions.
static COMMITS: AtomicU64 = AtomicU64::new(0);

/// Held while committing.
static COMMITTING: Mutex<()> = Mutex::new(());

impl Git {
    /// The `git` on PATH, with IVE's environment.
    pub fn installed() -> Self {
        Self {
            env: Vec::new(),
            #[cfg(test)]
            before_commit: Box::new(|| ()),
        }
    }

    /// The `git` on PATH with `env` added.
    #[cfg(test)]
    pub fn with_env(env: Vec<(String, String)>) -> Self {
        Self { env, before_commit: Box::new(|| ()) }
    }

    fn locate(&self, path: &Path) -> Result<Place, Refusal> {
        let folder = path.parent().unwrap_or(path);
        let top = self.text(folder, Call::of(&["rev-parse", "--show-toplevel"]), "find the work tree").map_err(not_in_repository)?;
        let not_inside = |reason: String| Refusal::NotInRepository(format!("{}: {reason}", path.display()));
        let top = fs::canonicalize(top.trim_end_matches('\n')).map_err(|error| not_inside(error.to_string()))?;
        let file = fs::canonicalize(folder).map_err(|error| not_inside(error.to_string()))?.join(path.file_name().unwrap_or_default());
        let relative = file.strip_prefix(&top).map_err(|_| not_inside(format!("outside {}", top.display())))?;
        let name = relative.components().map(|part| part.as_os_str().to_string_lossy()).collect::<Vec<_>>().join("/");
        Ok(Place { top, name })
    }

    fn head(&self, place: &Place) -> Result<String, Refusal> {
        let output = self.output(&place.top, Call::of(&["rev-parse", "--verify", "--quiet", "HEAD^{commit}"]))?;
        if output.status.success() {
            return Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned());
        }
        Err(Refusal::NoHead)
    }

    fn check_nothing_in_progress(&self, place: &Place) -> Result<(), Refusal> {
        let args: Vec<&str> = IN_PROGRESS.iter().flat_map(|(file, _)| ["--git-path", file]).collect();
        let paths = self.text(&place.top, Call::of(&[&["rev-parse"], args.as_slice()].concat()), "find the git dir")?;
        let found = paths.lines().zip(IN_PROGRESS).find(|(path, _)| place.top.join(path).exists());
        match found {
            Some((_, (_, name))) => Err(Refusal::InProgress(name)),
            None => Ok(()),
        }
    }

    /// The file's entry in commit `head`; refused for a file it lacks or holds as no regular file.
    fn head_entry(&self, place: &Place, head: &str) -> Result<Entry, Refusal> {
        match self.tree_entry(place, head)? {
            Some(entry) => Ok(entry),
            None => Err(self.absence(place)?),
        }
    }

    /// The file's entry in commit `commit`'s tree; `None` when it has none, refused when it is no regular file.
    fn tree_entry(&self, place: &Place, commit: &str) -> Result<Option<Entry>, Refusal> {
        let listed = self.text(&place.top, Call::of(&["ls-tree", "-z", commit, "--", &place.name]), "read a commit's tree")?;
        let Some(line) = listed.split('\0').find(|line| !line.is_empty()) else {
            return Ok(None);
        };
        // <mode> SP <type> SP <object> TAB <name>
        let fields: Vec<&str> = line.split(['\t', ' ']).collect();
        match fields.as_slice() {
            [mode @ ("100644" | "100755"), "blob", blob, ..] => Ok(Some(Entry { mode: (*mode).to_owned(), blob: (*blob).to_owned() })),
            _ => Err(Refusal::NotAFile),
        }
    }

    /// Why HEAD lacks the file: it is staged as new, or not tracked at all.
    fn absence(&self, place: &Place) -> Result<Refusal, Refusal> {
        match self.index_entry(place)? {
            Some(_) => Ok(Refusal::StagedNew),
            None => Ok(Refusal::NotInHead),
        }
    }

    /// The file's entry in the repository's index (stage 0); `None` when it has none.
    fn index_entry(&self, place: &Place) -> Result<Option<Entry>, Refusal> {
        let listed = self.text(&place.top, Call::of(&["ls-files", "-s", "-z", "--", &place.name]), "read the index")?;
        let Some(line) = listed.split('\0').find(|line| !line.is_empty()) else {
            return Ok(None);
        };
        // <mode> SP <object> SP <stage> TAB <name>
        let fields: Vec<&str> = line.split(['\t', ' ']).collect();
        match fields.as_slice() {
            [mode, blob, "0", ..] => Ok(Some(Entry { mode: (*mode).to_owned(), blob: (*blob).to_owned() })),
            _ => Err(Refusal::StagedChanges),
        }
    }

    /// Refuses a file git filters or re-encodes: git's form of the text may not be the one IVE compared with HEAD.
    fn check_unfiltered(&self, place: &Place) -> Result<(), Refusal> {
        let args = ["check-attr", "-z", "filter", "working-tree-encoding", "--", &place.name];
        let listed = self.text(&place.top, Call::of(&args), "read the file's attributes")?;
        // <path> NUL <attribute> NUL <value> NUL, for each attribute
        let fields: Vec<&str> = listed.split('\0').collect();
        if fields.as_chunks::<3>().0.iter().all(|[_, _, value]| matches!(*value, "unspecified" | "unset")) {
            return Ok(());
        }
        Err(Refusal::Filtered)
    }

    fn check_nothing_staged(&self, place: &Place) -> Result<(), Refusal> {
        let output = self.output(&place.top, Call::of(&["diff", "--cached", "--quiet", "--", &place.name]))?;
        match output.status.code() {
            Some(0) => Ok(()),
            Some(1) => Err(Refusal::StagedChanges),
            _ => Err(failure("compare the index with HEAD", &output)),
        }
    }

    /// Stores `text` as the file's content, through its clean filters (line endings); its blob id.
    fn store(&self, place: &Place, text: &[u8]) -> Result<String, Refusal> {
        let path = format!("--path={}", place.name);
        let args = ["hash-object", "-w", &path, "--stdin"];
        let call = Call { stdin: Some(text), ..Call::of(&args) };
        Ok(self.text(&place.top, call, "store the new content")?.trim().to_owned())
    }

    /// Commits the tree of `commit.head_id` with the file's entry `entry` and `commit.message`, through a temporary
    /// index; git puts it on HEAD, whichever commit that is by then.
    fn commit_entry(&self, place: &Place, entry: &Entry, commit: &BlockCommit) -> Result<Made, Refusal> {
        // unique beyond this process: an earlier IVE with the same process id left its own in the reflog
        let started = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis();
        let run = format!("{}-{}-{started}", std::process::id(), COMMITS.fetch_add(1, Ordering::Relaxed));
        let name = format!("ive-commit-index-{run}");
        let path = self.text(&place.top, Call::of(&["rev-parse", "--git-path", &name]), "find the git dir")?;
        let index = TempIndex(place.top.join(path.trim_end_matches('\n')));
        // the tree checked, not HEAD's: a commit landing meanwhile is not reverted by ours, it becomes our parent
        self.text(&place.top, index.call(&["read-tree", &commit.head_id]), "read HEAD into a temporary index")?;
        let info = format!("{},{},{}", entry.mode, entry.blob, place.name);
        self.text(&place.top, index.call(&["update-index", "--cacheinfo", &info]), "put the change in the temporary index")?;
        #[cfg(test)]
        (self.before_commit)();
        let action = format!("commit (ive {run})");
        let args = ["commit", "--quiet", "-m", &commit.message];
        let call = Call { reflog_action: Some(&action), ..index.call(&args) };
        self.text(&place.top, call, "commit")?;
        self.made(place, &action)
    }

    /// The commit whose entry in HEAD's reflog says `action` first, with its parent; HEAD's when HEAD keeps no
    /// reflog. Hooks inherit `action`: a post-commit hook's commits say it too, after ours.
    fn made(&self, place: &Place, action: &str) -> Result<Made, Refusal> {
        let args = ["log", "--walk-reflogs", "--no-show-signature", "-n", REFLOG_SEARCHED, "--format=%H %P%x09%gs", "HEAD"];
        let reflog = self.output(&place.top, Call::of(&args))?;
        let listed = String::from_utf8_lossy(&reflog.stdout);
        if listed.trim().is_empty() {
            let ids = self.text(&place.top, Call::of(&["rev-parse", "HEAD", "HEAD^1"]), "read the new commit")?;
            return made_of(ids.split_whitespace()).ok_or_else(|| missing(&ids));
        }
        let subject = format!("{action}: ");
        let ours = listed.lines().filter_map(|line| line.split_once('\t')).rfind(|(_, said)| said.starts_with(&subject));
        ours.and_then(|(ids, _)| made_of(ids.split(' '))).ok_or_else(|| missing(&listed))
    }

    /// Takes `made` off HEAD when git put it on another commit than `head_id`: one that landed after the check.
    fn check_parent(&self, place: &Place, made: &Made, head_id: &str) -> Result<(), Refusal> {
        if made.parent == head_id {
            return Ok(());
        }
        // only while HEAD is still `made`, and recorded in the reflog; the work tree and the index stay
        let args = ["update-ref", "-m", "ive: undo commit on moved HEAD", "HEAD", &made.parent, &made.id];
        let undone = self.output(&place.top, Call::of(&args))?;
        if undone.status.success() {
            return Err(Refusal::HeadMoved);
        }
        let stderr = String::from_utf8_lossy(&undone.stderr).into_owned();
        let stranded = Refusal::UndoFailed { commit: made.id.clone(), parent: made.parent.clone(), stderr };
        eprintln!("ive: {stranded}");
        Err(stranded)
    }

    /// Moves the file's index entry from `old` (HEAD's before the commit) to its entry in commit `made`, as hooks
    /// left it, so the committed change does not show as staged backwards; an entry holding other content stays.
    fn follow_in_index(&self, place: &Place, old: &Entry, made: &str) -> Result<(), Refusal> {
        if self.index_entry(place)?.as_ref() != Some(old) {
            return Ok(());
        }
        // a hook may have taken the file out of the commit
        let Some(new) = self.tree_entry(place, made)? else {
            return Ok(());
        };
        let info = format!("{},{},{}", new.mode, new.blob, place.name);
        self.text(&place.top, Call::of(&["update-index", "--cacheinfo", &info]), "update the index")?;
        Ok(())
    }

    /// The stdout of `call` run in `dir`, which must succeed.
    fn text(&self, dir: &Path, call: Call<'_>, doing: &'static str) -> Result<String, Refusal> {
        let output = self.output(dir, call)?;
        if output.status.success() {
            return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
        }
        Err(failure(doing, &output))
    }

    /// Runs `call` in `dir` and waits for it.
    fn output(&self, dir: &Path, call: Call<'_>) -> Result<Output, Refusal> {
        let mut command = self.command(dir, &call);
        command.stdin(if call.stdin.is_some() { Stdio::piped() } else { Stdio::null() });
        let mut child = command.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().map_err(|error| Refusal::GitMissing(error.to_string()))?;
        if let (Some(text), Some(mut stdin)) = (call.stdin, child.stdin.take()) {
            // a failed write shows as git's own failure below
            let _ = stdin.write_all(text);
        }
        child.wait_with_output().map_err(|error| Refusal::GitMissing(error.to_string()))
    }

    /// `git` running `call` in `dir` on the repository there, pathspecs taken literally.
    fn command(&self, dir: &Path, call: &Call<'_>) -> Command {
        let mut command = Command::new("git");
        command.arg("--literal-pathspecs").arg("-C").arg(dir).args(call.args);
        for name in INHERITED {
            command.env_remove(name);
        }
        command.envs(self.env.iter().map(|(name, value)| (name, value)));
        if let Some(index) = call.index {
            command.env("GIT_INDEX_FILE", index);
        }
        if let Some(action) = call.reflog_action {
            command.env("GIT_REFLOG_ACTION", action);
        }
        command
    }
}

/// A commit and its parent: the first two of `ids`.
fn made_of<'a>(mut ids: impl Iterator<Item = &'a str>) -> Option<Made> {
    let id = ids.next()?.to_owned();
    let parent = ids.next()?.to_owned();
    Some(Made { id, parent })
}

/// The commit just made is not in `listed`, what git said of HEAD.
fn missing(listed: &str) -> Refusal {
    Refusal::Git { doing: "find the new commit and its parent", stderr: format!("not in {listed:?}") }
}

fn failure(doing: &'static str, output: &Output) -> Refusal {
    Refusal::Git { doing, stderr: String::from_utf8_lossy(&output.stderr).into_owned() }
}

fn not_in_repository(refusal: Refusal) -> Refusal {
    match refusal {
        Refusal::Git { stderr, .. } => Refusal::NotInRepository(stderr.trim().to_owned()),
        other => other,
    }
}

/// A temporary index file, removed with its lock when dropped.
struct TempIndex(PathBuf);

impl TempIndex {
    /// `args` run with this index instead of the repository's.
    fn call<'a>(&'a self, args: &'a [&'a str]) -> Call<'a> {
        Call { index: Some(&self.0), ..Call::of(args) }
    }
}

impl Drop for TempIndex {
    fn drop(&mut self) {
        let mut lock = self.0.clone().into_os_string();
        lock.push(".lock");
        let lock = PathBuf::from(lock);
        for path in [&self.0, &lock] {
            // absent when git failed before writing it
            let _ = fs::remove_file(path);
        }
    }
}

#[cfg(test)]
#[path = "commit_block_tests.rs"]
mod tests;
