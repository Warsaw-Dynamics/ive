//! Commits one change of a file and nothing else: HEAD's version of the file with only that change in it. Runs the
//! `git` command, so the user's hooks, signing and config apply; the commit is built in a temporary index, so other
//! staged files stay staged and out of it.

use std::fmt;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

/// The new content of one file to commit on top of HEAD.
pub struct BlockCommit {
    pub path: PathBuf,
    /// the HEAD the content was made from: a commit refuses when HEAD is another by now
    pub head_id: String,
    /// the whole file in working-tree form: HEAD's text with the one change applied
    pub text: Vec<u8>,
    pub message: String,
}

/// Why nothing was committed; each says it for the user.
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
            Self::GitMissing(reason) => write!(f, "Cannot run git: {reason}"),
            Self::Git { doing, stderr } => write!(f, "git failed to {doing}: {}", stderr.trim()),
        }
    }
}

/// Commits `commit.text` as the new content of its file with message `commit.message`, on HEAD, and nothing else.
/// The file's index entry follows the commit when it had nothing staged.
pub fn commit_block(git: &Git, commit: &BlockCommit) -> Result<(), Refusal> {
    if commit.message.trim().is_empty() {
        return Err(Refusal::EmptyMessage);
    }
    let place = git.locate(&commit.path)?;
    if git.head(&place)? != commit.head_id {
        return Err(Refusal::HeadMoved);
    }
    git.check_nothing_in_progress(&place)?;
    let entry = git.head_entry(&place)?;
    git.check_nothing_staged(&place)?;
    let blob = git.store(&place, &commit.text)?;
    let new = Entry { blob, ..entry.clone() };
    git.commit_entry(&place, &new, &commit.message)?;
    if let Err(refusal) = git.follow_in_index(&place, &entry, &new) {
        eprintln!("ive: committed {}, but its index entry stayed as it was: {refusal}", place.name);
    }
    Ok(())
}

/// How to run the `git` command: the environment it gets on top of IVE's.
pub struct Git {
    env: Vec<(String, String)>,
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

/// One run of `git` in a work tree.
struct Call<'a> {
    args: &'a [&'a str],
    stdin: Option<&'a [u8]>,
    /// the index to use instead of the repository's
    index: Option<&'a Path>,
}

impl<'a> Call<'a> {
    fn of(args: &'a [&'a str]) -> Self {
        Self { args, stdin: None, index: None }
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

/// Numbers the temporary indexes of this process.
static TEMP_INDEXES: AtomicU64 = AtomicU64::new(0);

impl Git {
    /// The `git` on PATH, with IVE's environment.
    pub fn installed() -> Self {
        Self { env: Vec::new() }
    }

    /// The `git` on PATH with `env` added.
    #[cfg(test)]
    pub fn with_env(env: Vec<(String, String)>) -> Self {
        Self { env }
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

    /// The file's entry in HEAD; refused for a file HEAD lacks or holds as no regular file.
    fn head_entry(&self, place: &Place) -> Result<Entry, Refusal> {
        let listed = self.text(&place.top, Call::of(&["ls-tree", "-z", "HEAD", "--", &place.name]), "read HEAD's tree")?;
        let Some(line) = listed.split('\0').find(|line| !line.is_empty()) else {
            return Err(self.absence(place)?);
        };
        // <mode> SP <type> SP <object> TAB <name>
        let fields: Vec<&str> = line.split(['\t', ' ']).collect();
        match fields.as_slice() {
            [mode @ ("100644" | "100755"), "blob", blob, ..] => Ok(Entry { mode: (*mode).to_owned(), blob: (*blob).to_owned() }),
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

    fn check_nothing_staged(&self, place: &Place) -> Result<(), Refusal> {
        let output = self.output(&place.top, Call::of(&["diff", "--cached", "--quiet", "--", &place.name]))?;
        match output.status.code() {
            Some(0) => Ok(()),
            Some(1) => Err(Refusal::StagedChanges),
            _ => Err(failure("compare the index with HEAD", &output)),
        }
    }

    /// Stores `text` as the file's content, through its clean filters (line endings, `filter=`); its blob id.
    fn store(&self, place: &Place, text: &[u8]) -> Result<String, Refusal> {
        let path = format!("--path={}", place.name);
        let args = ["hash-object", "-w", &path, "--stdin"];
        let call = Call { stdin: Some(text), ..Call::of(&args) };
        Ok(self.text(&place.top, call, "store the new content")?.trim().to_owned())
    }

    /// Commits HEAD's tree with the file's entry `entry`, through a temporary index.
    fn commit_entry(&self, place: &Place, entry: &Entry, message: &str) -> Result<(), Refusal> {
        let name = format!("ive-commit-index-{}-{}", std::process::id(), TEMP_INDEXES.fetch_add(1, Ordering::Relaxed));
        let path = self.text(&place.top, Call::of(&["rev-parse", "--git-path", &name]), "find the git dir")?;
        let index = TempIndex(place.top.join(path.trim_end_matches('\n')));
        self.text(&place.top, index.call(&["read-tree", "HEAD"]), "read HEAD into a temporary index")?;
        let info = format!("{},{},{}", entry.mode, entry.blob, place.name);
        self.text(&place.top, index.call(&["update-index", "--cacheinfo", &info]), "put the change in the temporary index")?;
        self.text(&place.top, index.call(&["commit", "--quiet", "-m", message]), "commit")?;
        Ok(())
    }

    /// Moves the file's index entry from `old` (HEAD's before the commit) to `new` (the committed one), so the
    /// committed change does not show as staged backwards; an entry holding other content stays.
    fn follow_in_index(&self, place: &Place, old: &Entry, new: &Entry) -> Result<(), Refusal> {
        if self.index_entry(place)?.as_ref() != Some(old) {
            return Ok(());
        }
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

    /// Runs `call` in `dir`, pathspecs taken literally, and waits for it.
    fn output(&self, dir: &Path, call: Call<'_>) -> Result<Output, Refusal> {
        let mut command = Command::new("git");
        command.arg("--literal-pathspecs").arg("-C").arg(dir).args(call.args);
        command.envs(self.env.iter().map(|(name, value)| (name, value)));
        if let Some(index) = call.index {
            command.env("GIT_INDEX_FILE", index);
        }
        command.stdin(if call.stdin.is_some() { Stdio::piped() } else { Stdio::null() });
        let mut child = command.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().map_err(|error| Refusal::GitMissing(error.to_string()))?;
        if let (Some(text), Some(mut stdin)) = (call.stdin, child.stdin.take()) {
            // a failed write shows as git's own failure below
            let _ = stdin.write_all(text);
        }
        child.wait_with_output().map_err(|error| Refusal::GitMissing(error.to_string()))
    }
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
        let lock = self.0.with_extension("lock");
        for path in [&self.0, &lock] {
            // absent when git failed before writing it
            let _ = fs::remove_file(path);
        }
    }
}

#[cfg(test)]
#[path = "commit_block_tests.rs"]
mod tests;
