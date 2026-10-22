use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use git2::{Commit, Diff, DiffDelta, DiffFindOptions, DiffOptions, ErrorCode, FileMode, Patch, Repository, Tree};

use super::blob_filter;
use super::repo::{self, GitError};
use super::status::{Change, relative_path};
use super::types::{GitPath, Mark, Sha, failed, head_tree, lossy, mark_of_delta};

pub use codeview::diff::{Cut, DiffBody, DiffLine, FileDiff, Hunk, LineKind};
pub use codeview::diff::{FILE_LINE_LIMIT, MAX_FILE_BYTES, VIEW_LINE_LIMIT};
use codeview::changes::ChangeBase;
use codeview::document::Source;
use codeview::syntax::MAX_HIGHLIGHT_BYTES;

const CONTEXT_LINES: u32 = 3;
const TAB: &str = "    ";
/// Like git, a NUL byte in the first 8000 bytes marks a file as binary.
const BINARY_SNIFF_BYTES: usize = 8000;

/// Whether a diff from a tree reaches into the work tree (staged, unstaged and untracked changes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Uncommitted {
    Excluded,
    Included,
}

/// One file of a diff by its names: libgit2 pairs a rename only when both are in the pathspec.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FilePaths {
    pub path: GitPath,
    /// the name before a rename
    pub old_path: Option<GitPath>,
}

impl FilePaths {
    pub fn of(change: &Change) -> Self {
        Self { path: change.path.clone(), old_path: change.old_path.clone() }
    }
}

/// Where a Since diff starts, and whether it ends at HEAD or in the work tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SinceRange {
    pub fork: Sha,
    pub uncommitted: Uncommitted,
}

#[derive(Debug, Clone, Copy)]
struct Limits {
    file: usize,
    view: usize,
}

const LIMITS: Limits = Limits { file: FILE_LINE_LIMIT, view: VIEW_LINE_LIMIT };
const ONE_FILE: Limits = Limits { file: FILE_LINE_LIMIT, view: FILE_LINE_LIMIT };

/// HEAD → work tree through the index for one change.
pub fn uncommitted(dir: &Path, file: &FilePaths) -> Result<Option<FileDiff>, GitError> {
    let fail = failed("read the uncommitted diff", dir);
    let repo = repo::open(dir)?;
    let head = head_tree(&repo).map_err(&fail)?;
    let mut options = options();
    restrict(with_work_tree(&mut options), file);
    let mut diff = repo.diff_tree_to_workdir_with_index(head.as_ref(), Some(&mut options)).map_err(&fail)?;
    find_renames(&mut diff).map_err(&fail)?;
    Ok(collect(&diff, ONE_FILE).map_err(&fail)?.into_iter().find(|found| found.path == file.path))
}

/// The files changed from `fork` to `head`, names and marks only (content is read for rename detection alone),
/// sorted by path. It depends on the two commits only, so a caller keeps it while neither moves.
pub fn committed_changes(dir: &Path, fork: &Sha, head: &Sha) -> Result<Vec<Change>, GitError> {
    let fail = failed("read the diff since the fork point", dir);
    let repo = repo::open(dir)?;
    let tree = |sha: &Sha| repo.find_commit(sha.oid()?)?.tree();
    let (fork_tree, head_tree) = (tree(fork).map_err(&fail)?, tree(head).map_err(&fail)?);
    let mut diff = repo.diff_tree_to_tree(Some(&fork_tree), Some(&head_tree), Some(&mut options())).map_err(&fail)?;
    find_renames(&mut diff).map_err(&fail)?;
    Ok(sorted(changes_of(&diff)))
}

/// Since with uncommitted changes, from what [`committed_changes`] and `status::changes` listed: each uncommitted
/// change applied on top of the file's committed one, so the work tree is not read a second time. The marks are
/// those of fork → work tree; a file changed back to its fork content stays listed, and its diff says so.
pub fn with_uncommitted(committed: &[Change], uncommitted: &[Change]) -> Vec<Change> {
    let mut by_path: BTreeMap<GitPath, Change> = committed.iter().map(|change| (change.path.clone(), change.clone())).collect();
    for change in uncommitted {
        let before = by_path.remove(change.old_path.as_ref().unwrap_or(&change.path));
        if let Some(combined) = combined(before, change) {
            by_path.insert(combined.path.clone(), combined);
        }
    }
    by_path.into_values().collect()
}

/// One file of Since, read when it is shown.
pub fn since_file(dir: &Path, range: &SinceRange, file: &FilePaths) -> Result<Option<FileDiff>, GitError> {
    let fail = failed("read the diff since the fork point", dir);
    let repo = repo::open(dir)?;
    let mut options = options();
    restrict(&mut options, file);
    let fork_tree = repo.find_commit(range.fork.oid().map_err(&fail)?).and_then(|commit| commit.tree()).map_err(&fail)?;
    let mut diff = match range.uncommitted {
        Uncommitted::Excluded => repo.diff_tree_to_tree(Some(&fork_tree), head_tree(&repo).map_err(&fail)?.as_ref(), Some(&mut options)),
        Uncommitted::Included => repo.diff_tree_to_workdir_with_index(Some(&fork_tree), Some(with_work_tree(&mut options))),
    }
    .map_err(&fail)?;
    find_renames(&mut diff).map_err(&fail)?;
    Ok(collect(&diff, ONE_FILE).map_err(&fail)?.into_iter().find(|found| found.path == file.path))
}

/// The files a commit changed against its first parent, names and marks only, sorted by path.
pub fn commit_changes(dir: &Path, sha: &Sha) -> Result<Vec<Change>, GitError> {
    let fail = failed("read the files of a commit", dir);
    let repo = repo::open(dir)?;
    let commit = repo.find_commit(sha.oid().map_err(&fail)?).map_err(&fail)?;
    commit_deltas(&repo, &commit).map(sorted).map_err(&fail)
}

/// Which two texts a diff's files are diffed between.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Sides {
    /// HEAD → the work tree
    Uncommitted,
    /// the fork point → HEAD, or → the work tree
    Since(SinceRange),
    /// the commit's first parent (nothing for a root commit) → the commit
    Commit(Sha),
}

/// The whole old and new text of `file` between `sides`, for colouring its diff; a side is `None` when the file is
/// missing there, binary, or larger than `MAX_HIGHLIGHT_BYTES`.
pub fn texts(dir: &Path, sides: &Sides, file: &FilePaths) -> Result<(Option<Source>, Option<Source>), GitError> {
    let fail = failed("read the texts of a diff", dir);
    let repo = repo::open(dir)?;
    let (old_tree, new_side) = side_trees(&repo, sides).map_err(&fail)?;
    let old = blob_text(&repo, old_tree.as_ref(), file.old_path.as_ref().unwrap_or(&file.path)).map_err(&fail)?;
    let new = match new_side {
        NewSide::Tree(tree) => blob_text(&repo, tree.as_ref(), &file.path).map_err(&fail)?,
        NewSide::WorkTree => work_tree_text(&repo, &file.path),
    };
    Ok((old, new))
}

/// An uncommitted diff of a file as IVE edits it: HEAD's text of it against the text IVE read or saved.
pub struct EditedDiff {
    /// HEAD's text, when it is one to colour
    pub old: Option<Source>,
    pub body: DiffBody,
}

/// The uncommitted diff of `file` whose work tree text is `new` (as IVE read or saved it, not as the work tree holds it
/// now): against HEAD's text as a checkout writes it (its filters applied), so a line `new` did not change is context.
/// A file HEAD lacks is all added; one HEAD has as other than a regular file, or too large, is an error.
pub fn edited_uncommitted(dir: &Path, file: &FilePaths, new: &[u8]) -> Result<EditedDiff, GitError> {
    let fail = failed("diff the edited text with HEAD", dir);
    let repo = repo::open(dir)?;
    let old = checked_out_head(&repo, file).map_err(&fail)?;
    let body = buffers_body(&old, new).map_err(&fail)?;
    let binary = memchr::memchr(0, &old[..old.len().min(BINARY_SNIFF_BYTES)]).is_some();
    let colourable = old.len() <= MAX_HIGHLIGHT_BYTES && !binary;
    Ok(EditedDiff { old: colourable.then(|| Source::from_vec(old)), body })
}

/// The diff of two texts of one file, as [`uncommitted`] reads it.
fn buffers_body(old: &[u8], new: &[u8]) -> Result<DiffBody, git2::Error> {
    let patch = Patch::from_buffers(old, None, new, None, Some(&mut options()))?;
    patch_body(&patch, Budget { lines: FILE_LINE_LIMIT, cut: Cut::FileLimit(FILE_LINE_LIMIT) })
}

/// HEAD's text of `file` (by its name before a rename) as a checkout to its name writes it; empty when HEAD lacks it.
fn checked_out_head(repo: &Repository, file: &FilePaths) -> Result<Vec<u8>, git2::Error> {
    let Some(tree) = head_tree(repo)? else {
        return Ok(Vec::new());
    };
    let name = file.old_path.as_ref().unwrap_or(&file.path);
    let entry = match tree.get_path(&relative_path(name.as_bytes())) {
        Ok(entry) => entry,
        Err(error) if error.code() == ErrorCode::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };
    if entry.filemode() != i32::from(FileMode::Blob) && entry.filemode() != i32::from(FileMode::BlobExecutable) {
        return Err(git2::Error::from_str(&format!("{} is not a regular file in HEAD", name.display())));
    }
    let (size, _) = repo.odb()?.read_header(entry.id())?;
    if size as u64 > MAX_FILE_BYTES {
        return Err(git2::Error::from_str(&format!("{} is too large in HEAD ({size} bytes)", name.display())));
    }
    blob_filter::checked_out(&repo.find_blob(entry.id())?, &file.path)
}

/// What change markers compare the work tree file at `path` with: its text in HEAD as a checkout writes it (as
/// [`edited_uncommitted`] diffs it), with HEAD's commit id. A file staged as new (HEAD, or an unborn branch, lacks
/// it and the index has it) has an empty text, so every line shows as added, as in the Uncommitted diff.
/// [`ChangeBase::Untracked`] when no repository's work tree holds the file, neither HEAD nor the index has it, or
/// HEAD's entry is not a regular file, is binary or is larger than `MAX_HIGHLIGHT_BYTES`.
#[cfg(test)]
pub fn change_base(path: &Path) -> Result<ChangeBase, GitError> {
    Ok(change_base_since(path, &BaseAsk { known: "", git_dir: None })?.unwrap_or(ChangeBase::Untracked))
}

/// What [`change_base_since`] is asked.
pub struct BaseAsk<'a> {
    /// the `head_id` the caller last got: an answer with the same one is skipped
    pub known: &'a str,
    /// the git dir that changed: a file of another repository is skipped
    pub git_dir: Option<&'a Path>,
}

/// [`change_base`], or `None` when `ask` says the caller has the answer: the file is of another git dir, or HEAD
/// (and whether it holds the file) is as the caller last saw it, which is told without reading HEAD's blob.
pub fn change_base_since(path: &Path, ask: &BaseAsk<'_>) -> Result<Option<ChangeBase>, GitError> {
    let Some((repo, name)) = repository_of(path)? else {
        return Ok(Some(ChangeBase::Untracked));
    };
    if ask.git_dir.is_some_and(|dir| !is_git_dir_of(&repo, dir)) {
        return Ok(None);
    }
    let workdir = repo.workdir().map_or_else(|| path.to_path_buf(), Path::to_path_buf);
    let fail = failed("read HEAD's text of a file", &workdir);
    let (head_id, blob) = match head_entry(&repo, &name).map_err(&fail)? {
        HeadEntry::Absent => return Ok(Some(ChangeBase::Untracked)),
        HeadEntry::StagedNew { head_id } if head_id == ask.known => return Ok(None),
        HeadEntry::StagedNew { head_id } => return Ok(Some(ChangeBase::Tracked { head: Source::from_vec(Vec::new()), head_id })),
        HeadEntry::Blob { head_id, .. } if head_id == ask.known => return Ok(None),
        HeadEntry::Blob { head_id, blob } => (head_id, blob),
    };
    let blob = repo.find_blob(blob).map_err(&fail)?;
    if blob.is_binary() {
        return Ok(Some(ChangeBase::Untracked));
    }
    let text = blob_filter::checked_out(&blob, &name).map_err(&fail)?;
    if text.len() > MAX_HIGHLIGHT_BYTES {
        return Ok(Some(ChangeBase::Untracked));
    }
    Ok(Some(ChangeBase::Tracked { head: Source::from_vec(text), head_id }))
}

/// Whether `dir` is the git dir of `repo`, or of the repository a linked work tree belongs to.
fn is_git_dir_of(repo: &Repository, dir: &Path) -> bool {
    let wanted = fs::canonicalize(dir).unwrap_or_else(|_| dir.to_path_buf());
    [repo.path(), repo.commondir()].iter().any(|own| fs::canonicalize(own).is_ok_and(|own| own == wanted))
}

/// The repository whose work tree holds the file at `path`, found from its folder up, and the file's name in it;
/// `None` when there is none.
fn repository_of(path: &Path) -> Result<Option<(Repository, GitPath)>, GitError> {
    let folder = path.parent().unwrap_or(path);
    let repo = match Repository::discover(folder) {
        Ok(repo) => repo,
        Err(error) if error.code() == ErrorCode::NotFound => return Ok(None),
        Err(error) => return Err(GitError::new("find the repository", folder, error)),
    };
    let (Some(workdir), Ok(file)) = (repo.workdir().and_then(|dir| fs::canonicalize(dir).ok()), fs::canonicalize(path)) else {
        return Ok(None);
    };
    let name = file.strip_prefix(&workdir).ok().map(|relative| GitPath::from_bytes(relative.as_os_str().as_encoded_bytes()));
    Ok(name.map(|name| (repo, name)))
}

/// What HEAD and the index hold at a file's name.
enum HeadEntry {
    /// a regular file small enough to diff in HEAD, with HEAD's commit id
    Blob { head_id: String, blob: git2::Oid },
    /// not in HEAD (or there is no commit yet) but in the index; `head_id` is HEAD's commit id, or `unborn`
    StagedNew { head_id: String },
    Absent,
}

fn head_entry(repo: &Repository, name: &GitPath) -> Result<HeadEntry, git2::Error> {
    let commit = match repo.head() {
        Ok(head) => Some(head.peel_to_commit()?),
        Err(error) if matches!(error.code(), ErrorCode::UnbornBranch | ErrorCode::NotFound) => None,
        Err(error) => return Err(error),
    };
    let head_id = commit.as_ref().map_or_else(|| "unborn".to_owned(), |commit| commit.id().to_string());
    let entry = match &commit {
        Some(commit) => match commit.tree()?.get_path(&relative_path(name.as_bytes())) {
            Ok(entry) => Some(entry),
            Err(error) if error.code() == ErrorCode::NotFound => None,
            Err(error) => return Err(error),
        },
        None => None,
    };
    let Some(entry) = entry else {
        let staged = repo.index()?.get_path(&relative_path(name.as_bytes()), 0).is_some();
        return Ok(if staged { HeadEntry::StagedNew { head_id } } else { HeadEntry::Absent });
    };
    let regular = entry.filemode() == i32::from(FileMode::Blob) || entry.filemode() == i32::from(FileMode::BlobExecutable);
    let (size, _) = repo.odb()?.read_header(entry.id())?;
    Ok(if regular && size <= MAX_HIGHLIGHT_BYTES { HeadEntry::Blob { head_id, blob: entry.id() } } else { HeadEntry::Absent })
}

/// Where the work tree of the repository at `dir` keeps `path`.
pub fn work_tree_path(dir: &Path, path: &GitPath) -> Result<PathBuf, GitError> {
    let repo = repo::open(dir)?;
    let workdir = repo.workdir().ok_or_else(|| GitError::new("find the work tree", dir, git2::Error::from_str("a bare repository has none")))?;
    Ok(workdir.join(relative_path(path.as_bytes())))
}

/// Where a diff's new side is.
enum NewSide<'r> {
    /// `None`: no tree at all, as before the first commit
    Tree(Option<Tree<'r>>),
    WorkTree,
}

/// The old side's tree (`None` before a root commit or the first commit) and where the new side is.
fn side_trees<'r>(repo: &'r Repository, sides: &Sides) -> Result<(Option<Tree<'r>>, NewSide<'r>), git2::Error> {
    let commit_tree = |sha: &Sha| repo.find_commit(sha.oid()?)?.tree();
    Ok(match sides {
        Sides::Uncommitted => (head_tree(repo)?, NewSide::WorkTree),
        Sides::Since(SinceRange { fork, uncommitted: Uncommitted::Included }) => (Some(commit_tree(fork)?), NewSide::WorkTree),
        Sides::Since(SinceRange { fork, uncommitted: Uncommitted::Excluded }) => (Some(commit_tree(fork)?), NewSide::Tree(head_tree(repo)?)),
        Sides::Commit(sha) => {
            let commit = repo.find_commit(sha.oid()?)?;
            let parent = match commit.parent_count() {
                0 => None,
                _ => Some(commit.parent(0)?.tree()?),
            };
            (parent, NewSide::Tree(Some(commit.tree()?)))
        }
    })
}

/// The text of the file at `path` in `tree`, when it is one to colour: a regular file, not binary, small enough.
fn blob_text(repo: &Repository, tree: Option<&Tree<'_>>, path: &GitPath) -> Result<Option<Source>, git2::Error> {
    let Some(tree) = tree else {
        return Ok(None);
    };
    let entry = match tree.get_path(&relative_path(path.as_bytes())) {
        Ok(entry) => entry,
        Err(error) if error.code() == ErrorCode::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    // a symbolic link's blob is its target, a submodule's entry a commit
    if entry.filemode() != i32::from(FileMode::Blob) && entry.filemode() != i32::from(FileMode::BlobExecutable) {
        return Ok(None);
    }
    // the size first: a blob too large to colour is not read at all
    let (size, _) = repo.odb()?.read_header(entry.id())?;
    if size > MAX_HIGHLIGHT_BYTES {
        return Ok(None);
    }
    let blob = repo.find_blob(entry.id())?;
    Ok((!blob.is_binary()).then(|| Source::from_vec(blob.content().to_vec())))
}

/// The work tree's file at `path`, when it is one to colour; `None` also when it cannot be read (it went away).
fn work_tree_text(repo: &Repository, path: &GitPath) -> Option<Source> {
    let file = repo.workdir()?.join(relative_path(path.as_bytes()));
    let metadata = fs::symlink_metadata(&file).ok()?;
    if !metadata.is_file() || metadata.len() > MAX_HIGHLIGHT_BYTES as u64 {
        return None;
    }
    let bytes = fs::read(&file).ok()?;
    let binary = memchr::memchr(0, &bytes[..bytes.len().min(BINARY_SNIFF_BYTES)]).is_some();
    (bytes.len() <= MAX_HIGHLIGHT_BYTES && !binary).then(|| Source::from_vec(bytes))
}

/// One file of a commit, read when it is shown.
pub fn commit_file(dir: &Path, sha: &Sha, file: &FilePaths) -> Result<Option<FileDiff>, GitError> {
    let fail = failed("read the diff of a commit", dir);
    let repo = repo::open(dir)?;
    let commit = repo.find_commit(sha.oid().map_err(&fail)?).map_err(&fail)?;
    let mut options = options();
    restrict(&mut options, file);
    let mut diff = commit_diff(&repo, &commit, &mut options).map_err(&fail)?;
    find_renames(&mut diff).map_err(&fail)?;
    Ok(collect(&diff, ONE_FILE).map_err(&fail)?.into_iter().find(|found| found.path == file.path))
}

/// A commit against its first parent; a root commit against the empty tree.
pub(crate) fn commit_diff<'r>(repo: &'r Repository, commit: &Commit<'_>, options: &mut DiffOptions) -> Result<Diff<'r>, git2::Error> {
    let tree = commit.tree()?;
    let parent = match commit.parent_count() {
        0 => None,
        _ => Some(commit.parent(0)?.tree()?),
    };
    repo.diff_tree_to_tree(parent.as_ref(), Some(&tree), Some(options))
}

/// A commit's changed files after rename detection, so a renamed file counts once.
pub(crate) fn commit_deltas(repo: &Repository, commit: &Commit<'_>) -> Result<Vec<Change>, git2::Error> {
    let mut diff = commit_diff(repo, commit, &mut options())?;
    find_renames(&mut diff)?;
    Ok(changes_of(&diff))
}

/// Renames found, then cut to the limits: a commit tab shows all its files in one view.
pub(crate) fn files_of(mut diff: Diff<'_>) -> Result<Vec<FileDiff>, git2::Error> {
    find_renames(&mut diff)?;
    collect(&diff, LIMITS)
}

pub(crate) fn options() -> DiffOptions {
    let mut options = DiffOptions::new();
    options
        .context_lines(CONTEXT_LINES)
        .include_typechange(true)
        .ignore_submodules(true)
        .max_size(MAX_FILE_BYTES as i64);
    options
}

fn with_work_tree(options: &mut DiffOptions) -> &mut DiffOptions {
    options.include_untracked(true).show_untracked_content(true).recurse_untracked_dirs(true)
}

/// Limits a diff to these exact paths: no fnmatch, so `a[1].txt` means that file.
fn restrict(options: &mut DiffOptions, file: &FilePaths) {
    options.disable_pathspec_match(true).pathspec(file.path.as_bytes());
    if let Some(old) = &file.old_path {
        options.pathspec(old.as_bytes());
    }
}

fn find_renames(diff: &mut Diff<'_>) -> Result<(), git2::Error> {
    diff.find_similar(Some(DiffFindOptions::new().renames(true).for_untracked(true)))
}

fn changes_of(diff: &Diff<'_>) -> Vec<Change> {
    diff.deltas()
        .map(|delta| {
            let (path, old_path) = names(&delta);
            Change { mark: mark_of_delta(delta.status()), path, old_path }
        })
        .collect()
}

fn sorted(mut changes: Vec<Change>) -> Vec<Change> {
    changes.sort_by(|a, b| a.path.cmp(&b.path));
    changes
}

/// `after` (HEAD → work tree) on top of `before` (fork → HEAD) for one file.
fn combined(before: Option<Change>, after: &Change) -> Option<Change> {
    let Some(before) = before else {
        return Some(after.clone());
    };
    let mark = match (before.mark, after.mark) {
        // added since the fork point and deleted again: no change at all
        (Mark::Added, Mark::Deleted) => return None,
        (Mark::Added, _) => Mark::Added,
        (Mark::Deleted, Mark::Untracked | Mark::Added) => Mark::Modified,
        (_, Mark::Deleted) => return Some(Change { mark: Mark::Deleted, path: before.old_path.unwrap_or(before.path), old_path: None }),
        (_, Mark::TypeChange | Mark::Conflicted) => after.mark,
        (Mark::Renamed, _) | (_, Mark::Renamed) => Mark::Renamed,
        (mark, _) => mark,
    };
    // a rename names the file as it was at the fork point; renamed back, it is a modification
    let old_path = before.old_path.or_else(|| after.old_path.clone()).filter(|old| mark == Mark::Renamed && *old != after.path);
    let mark = match (mark, &old_path) {
        (Mark::Renamed, None) => Mark::Modified,
        (mark, _) => mark,
    };
    Some(Change { mark, path: after.path.clone(), old_path })
}

fn collect(diff: &Diff<'_>, limits: Limits) -> Result<Vec<FileDiff>, git2::Error> {
    let mut left = limits.view;
    let mut files = Vec::with_capacity(diff.deltas().len());
    for index in 0..diff.deltas().len() {
        let Some(delta) = diff.get_delta(index) else {
            continue;
        };
        let budget = match left < limits.file {
            true => Budget { lines: left, cut: Cut::ViewLimit(limits.view) },
            false => Budget { lines: limits.file, cut: Cut::FileLimit(limits.file) },
        };
        let body = body(diff, index, budget)?;
        let (path, old_path) = names(&delta);
        let file = FileDiff { path, old_path, mark: mark_of_delta(delta.status()), body };
        left -= file.line_count();
        files.push(file);
    }
    Ok(files)
}

fn names(delta: &DiffDelta<'_>) -> (GitPath, Option<GitPath>) {
    let new = delta.new_file().path_bytes();
    let old = delta.old_file().path_bytes();
    let path = GitPath::from_bytes(new.or(old).unwrap_or_default());
    let old_path = old.filter(|old| Some(*old) != new).map(GitPath::from_bytes);
    (path, old_path)
}

/// How many lines one file may still show, and why it stops there.
struct Budget {
    lines: usize,
    cut: Cut,
}

fn body(diff: &Diff<'_>, index: usize, budget: Budget) -> Result<DiffBody, git2::Error> {
    if budget.lines == 0 {
        return Ok(DiffBody::Text { hunks: Vec::new(), cut: Some(budget.cut) });
    }
    // libgit2 gives no patch for binary and unchanged entries
    let Some(patch) = Patch::from_diff(diff, index)? else {
        return Ok(diff.get_delta(index).map_or(DiffBody::Text { hunks: Vec::new(), cut: None }, |delta| binary_or_empty(&delta)));
    };
    patch_body(&patch, budget)
}

fn patch_body(patch: &Patch<'_>, budget: Budget) -> Result<DiffBody, git2::Error> {
    let delta = patch.delta();
    if delta.flags().is_binary() {
        let size = delta.new_file().size().max(delta.old_file().size());
        return Ok(if size > MAX_FILE_BYTES { DiffBody::TooLarge } else { DiffBody::Binary });
    }
    text(patch, budget)
}

fn binary_or_empty(delta: &DiffDelta<'_>) -> DiffBody {
    match delta.flags().is_binary() {
        true => DiffBody::Binary,
        false => DiffBody::Text { hunks: Vec::new(), cut: None },
    }
}

fn text(patch: &Patch<'_>, budget: Budget) -> Result<DiffBody, git2::Error> {
    let Budget { lines: limit, cut } = budget;
    let mut hunks = Vec::with_capacity(patch.num_hunks());
    let mut shown = 0;
    for hunk_index in 0..patch.num_hunks() {
        let (hunk, line_count) = patch.hunk(hunk_index)?;
        let mut lines = Vec::with_capacity(line_count.min(limit - shown));
        for line_index in 0..line_count {
            if shown == limit {
                hunks.push(Hunk { header: header(hunk.header()), lines });
                return Ok(DiffBody::Text { hunks, cut: Some(cut) });
            }
            lines.push(line(&patch.line_in_hunk(hunk_index, line_index)?));
            shown += 1;
        }
        hunks.push(Hunk { header: header(hunk.header()), lines });
    }
    Ok(DiffBody::Text { hunks, cut: None })
}

fn header(bytes: &[u8]) -> String {
    lossy(bytes).trim_end().to_owned()
}

fn line(line: &git2::DiffLine<'_>) -> DiffLine {
    let kind = match line.origin() {
        '+' => LineKind::Added,
        '-' => LineKind::Removed,
        '=' | '>' | '<' => LineKind::NoNewline,
        _ => LineKind::Context,
    };
    // libgit2 numbers the "no newline" marker like the line before it; the gutter leaves it blank
    match kind {
        LineKind::NoNewline => DiffLine { kind, old: None, new: None, text: "\\ No newline at end of file".to_owned() },
        _ => DiffLine { kind, old: line.old_lineno(), new: line.new_lineno(), text: lossy(line.content()).trim_end_matches(['\n', '\r']).replace('\t', TAB) },
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::git::status;
    use crate::git::testing::TestRepo;

    fn lines(file: &FileDiff) -> Vec<(LineKind, Option<u32>, Option<u32>, String)> {
        let DiffBody::Text { hunks, .. } = &file.body else { panic!("not a text diff: {:?}", file.body) };
        hunks.iter().flat_map(|hunk| &hunk.lines).map(|line| (line.kind, line.old, line.new, line.text.clone())).collect()
    }

    fn path(text: &str) -> GitPath {
        GitPath::from_bytes(text.as_bytes())
    }

    fn file(text: &str) -> FilePaths {
        FilePaths { path: path(text), old_path: None }
    }

    fn renamed(old: &str, new: &str) -> FilePaths {
        FilePaths { path: path(new), old_path: Some(path(old)) }
    }

    fn change(mark: Mark, name: &str, old: Option<&str>) -> Change {
        Change { mark, path: path(name), old_path: old.map(path) }
    }

    fn summary(changes: &[Change]) -> Vec<(&'static str, String, Option<String>)> {
        changes.iter().map(|change| (change.mark.letter(), change.path.display(), change.old_path.as_ref().map(GitPath::display))).collect()
    }

    #[test]
    fn should_diff_head_to_work_tree_with_staged_and_unstaged_changes_together() {
        // given
        let repo = TestRepo::init();
        repo.write("a.txt", "one\ntwo\nthree\n");
        repo.commit("first");
        repo.write("a.txt", "one\nTWO\nthree\n");
        repo.stage("a.txt");
        repo.write("a.txt", "one\nTWO\nthree\nfour\n");

        // when
        let found = uncommitted(repo.path(), &file("a.txt")).expect("diff").expect("a.txt changed");

        // then
        assert_eq!(lines(&found), [
            (LineKind::Context, Some(1), Some(1), "one".to_owned()),
            (LineKind::Removed, Some(2), None, "two".to_owned()),
            (LineKind::Added, None, Some(2), "TWO".to_owned()),
            (LineKind::Context, Some(3), Some(3), "three".to_owned()),
            (LineKind::Added, None, Some(4), "four".to_owned()),
        ]);
    }

    #[test]
    fn should_number_lines_across_hunks_and_mark_a_missing_newline() {
        // given
        let repo = TestRepo::init();
        let numbered: Vec<String> = (1..=20).map(|n| format!("line {n}")).collect();
        repo.write("a.txt", &format!("{}\n", numbered.join("\n")));
        repo.commit("first");
        let mut changed = numbered.clone();
        changed[1] = "line two".to_owned();
        changed[18] = "line nineteen".to_owned();
        repo.write("a.txt", &changed.join("\n"));

        // when
        let found = uncommitted(repo.path(), &file("a.txt")).expect("diff").expect("a.txt changed");

        // then
        let DiffBody::Text { hunks, .. } = &found.body else { panic!("not text") };
        let headers: Vec<_> = hunks.iter().map(|hunk| hunk.header.as_str()).collect();
        let all = lines(&found);
        assert_eq!(headers, ["@@ -1,5 +1,5 @@", "@@ -16,5 +16,5 @@ line 15"]);
        assert_eq!(all[all.len() - 4..], [
            (LineKind::Removed, Some(20), None, "line 20".to_owned()),
            (LineKind::Added, None, Some(19), "line nineteen".to_owned()),
            (LineKind::Added, None, Some(20), "line 20".to_owned()),
            (LineKind::NoNewline, None, None, "\\ No newline at end of file".to_owned()),
        ]);
    }

    #[test]
    fn should_show_untracked_file_as_all_added() {
        // given
        let repo = TestRepo::init();
        repo.write("new folder/x.txt", "a\nb\n");

        // when
        let found = uncommitted(repo.path(), &file("new folder/x.txt")).expect("diff").expect("untracked file");

        // then
        assert_eq!((found.mark, lines(&found)), (Mark::Untracked, vec![
            (LineKind::Added, None, Some(1), "a".to_owned()),
            (LineKind::Added, None, Some(2), "b".to_owned()),
        ]));
    }

    #[test]
    fn should_diff_a_rename_against_its_old_name() {
        // given
        let repo = TestRepo::init();
        repo.write("old.txt", "line 1\nline 2\nline 3\nline 4\nline 5\n");
        repo.commit("first");
        repo.rename("old.txt", "new.txt");
        repo.write("new.txt", "line 1\nline 2\nline 3\nline 4\nline five\n");

        // when
        let found = uncommitted(repo.path(), &renamed("old.txt", "new.txt")).expect("diff").expect("rename");

        // then
        assert_eq!((found.mark, found.old_path.as_ref().map(GitPath::display)), (Mark::Renamed, Some("old.txt".to_owned())));
        assert!(lines(&found).contains(&(LineKind::Removed, Some(5), None, "line 5".to_owned())));
    }

    #[test]
    fn should_treat_paths_with_glob_characters_literally() {
        // given
        let repo = TestRepo::init();
        repo.write("a[1].txt", "x\n");
        repo.write("a1.txt", "x\n");
        repo.commit("first");
        repo.write("a[1].txt", "y\n");
        repo.write("a1.txt", "y\n");
        let mut options = options();
        restrict(with_work_tree(&mut options), &file("a[1].txt"));

        // when
        let head = head_tree(&repo.repo).expect("HEAD");
        let diff = repo.repo.diff_tree_to_workdir_with_index(head.as_ref(), Some(&mut options)).expect("diff");

        // then
        let paths: Vec<_> = diff.deltas().filter_map(|delta| delta.new_file().path_bytes().map(lossy)).collect();
        assert_eq!(paths, ["a[1].txt"]);
    }

    #[test]
    fn should_give_a_nested_checkout_folder_no_diff_of_its_own() {
        // given
        let repo = TestRepo::init();
        repo.write("a.txt", "a\n");
        repo.commit("first");
        let nested = repo.path().join(".claude/worktrees/topic");
        fs::create_dir_all(nested.parent().unwrap()).unwrap();
        crate::git::testing::add_worktree(&repo.repo, "topic", &nested);

        // when
        let found = uncommitted(repo.path(), &file(".claude/worktrees/topic/")).expect("diff");

        // then
        assert_eq!(found.map(|found| found.body), Some(DiffBody::Text { hunks: Vec::new(), cut: None }));
    }

    #[test]
    fn should_not_load_binary_or_too_large_files() {
        // given
        let repo = TestRepo::init();
        repo.write_bytes(Path::new("data.bin"), &[0, 1, 2, 0, 3]);
        repo.write("big.txt", &"x\n".repeat(5_500_000));

        // when
        let bodies = ["data.bin", "big.txt"].map(|name| uncommitted(repo.path(), &file(name)).expect("diff").expect("untracked").body);

        // then
        assert_eq!(bodies, [DiffBody::Binary, DiffBody::TooLarge]);
    }

    #[test]
    fn should_cut_a_file_diff_at_20000_lines() {
        // given
        let repo = TestRepo::init();
        let text: String = (0..25_000).map(|n| format!("line {n}\n")).collect();
        repo.write("long.txt", &text);

        // when
        let found = uncommitted(repo.path(), &file("long.txt")).expect("diff").expect("untracked");

        // then
        let DiffBody::Text { cut, .. } = &found.body else { panic!("not text") };
        assert_eq!((found.line_count(), cut.clone()), (20_000, Some(Cut::FileLimit(20_000))));
    }

    #[test]
    fn should_cut_later_files_once_the_view_budget_is_spent() {
        // given
        let repo = TestRepo::init();
        repo.write("1.txt", "a\nb\nc\n");
        repo.write("2.txt", "a\nb\nc\nd\ne\n");
        repo.write("3.txt", "a\n");
        let mut options = options();
        options.include_untracked(true).show_untracked_content(true);
        let diff = repo.repo.diff_tree_to_workdir_with_index(None, Some(&mut options)).expect("diff");

        // when
        let files = collect(&diff, Limits { file: 4, view: 6 }).expect("files");

        // then
        let summary: Vec<_> = files.iter().map(|file| (file.line_count(), cut_of(file))).collect();
        assert_eq!(summary, [(3, None), (3, Some(Cut::ViewLimit(6))), (0, Some(Cut::ViewLimit(6)))]);
    }

    fn cut_of(file: &FileDiff) -> Option<Cut> {
        match &file.body {
            DiffBody::Text { cut, .. } => cut.clone(),
            DiffBody::Binary | DiffBody::TooLarge => None,
        }
    }

    #[test]
    fn should_list_since_fork_with_and_without_uncommitted_changes_and_diff_one_file() {
        // given
        let repo = TestRepo::init();
        repo.write("f.txt", "base\n");
        repo.write("g.txt", "base\n");
        let fork = Sha::of(repo.commit("base"));
        repo.branch("topic");
        repo.checkout("topic");
        repo.write("f.txt", "topic\n");
        let head = Sha::of(repo.commit("on topic"));
        repo.write("g.txt", "uncommitted\n");
        repo.write("u.txt", "untracked\n");

        // when
        let committed = committed_changes(repo.path(), &fork, &head).expect("committed");
        let with_work_tree = with_uncommitted(&committed, &status::changes(repo.path()).expect("status"));
        let range = SinceRange { fork, uncommitted: Uncommitted::Included };
        let g = since_file(repo.path(), &range, &file("g.txt")).expect("diff").expect("g.txt changed");

        // then
        assert_eq!(summary(&committed), [("M", "f.txt".to_owned(), None)]);
        assert_eq!(summary(&with_work_tree), [("M", "f.txt".to_owned(), None), ("M", "g.txt".to_owned(), None), ("?", "u.txt".to_owned(), None)]);
        assert!(lines(&g).contains(&(LineKind::Added, None, Some(1), "uncommitted".to_owned())));
    }

    #[test]
    fn should_combine_committed_and_uncommitted_changes_as_fork_to_work_tree() {
        // given
        let committed = [
            change(Mark::Added, "added-then-deleted.txt", None),
            change(Mark::Added, "added-then-changed.txt", None),
            change(Mark::Deleted, "deleted-then-created.txt", None),
            change(Mark::Renamed, "b.txt", Some("a.txt")),
            change(Mark::Modified, "m.txt", None),
            change(Mark::Renamed, "back.txt", Some("start.txt")),
        ];
        let uncommitted = [
            change(Mark::Deleted, "added-then-deleted.txt", None),
            change(Mark::Modified, "added-then-changed.txt", None),
            change(Mark::Untracked, "deleted-then-created.txt", None),
            change(Mark::Deleted, "b.txt", None),
            change(Mark::Renamed, "n.txt", Some("m.txt")),
            change(Mark::Renamed, "start.txt", Some("back.txt")),
            change(Mark::Untracked, "new.txt", None),
        ];

        // when
        let combined = with_uncommitted(&committed, &uncommitted);

        // then
        assert_eq!(summary(&combined), [
            ("D", "a.txt".to_owned(), None),
            ("A", "added-then-changed.txt".to_owned(), None),
            ("M", "deleted-then-created.txt".to_owned(), None),
            ("R", "n.txt".to_owned(), Some("m.txt".to_owned())),
            ("?", "new.txt".to_owned(), None),
            ("M", "start.txt".to_owned(), None),
        ]);
    }

    #[test]
    fn should_list_files_of_a_merge_against_its_first_parent_and_of_a_root_against_nothing() {
        // given
        let repo = TestRepo::init();
        repo.write("base.txt", "base\n");
        let root = Sha::of(repo.commit("root"));
        repo.branch("side");
        repo.checkout("side");
        repo.write("side.txt", "side\n");
        let side = repo.commit("on side");
        repo.checkout("main");
        repo.write("main.txt", "main\n");
        repo.commit("on main");
        repo.write("side.txt", "side\n");
        let merge = Sha::of(repo.merge("merge side", side));

        // when
        let of_merge = commit_changes(repo.path(), &merge).expect("merge files");
        let of_root = commit_changes(repo.path(), &root).expect("root files");

        // then
        assert_eq!(summary(&of_merge), [("A", "side.txt".to_owned(), None)]);
        assert_eq!(summary(&of_root), [("A", "base.txt".to_owned(), None)]);
    }

    #[test]
    fn should_read_one_file_of_a_commit_with_both_names_of_a_rename() {
        // given
        let repo = TestRepo::init();
        repo.write("old.txt", "line 1\nline 2\nline 3\nline 4\nline 5\n");
        repo.write("other.txt", "other\n");
        repo.commit("first");
        repo.rename("old.txt", "new.txt");
        repo.write("new.txt", "line 1\nline 2\nline 3\nline 4\nline five\n");
        repo.write("other.txt", "changed\n");
        let sha = Sha::of(repo.commit("rename and change"));
        let files = commit_changes(repo.path(), &sha).expect("files");

        // when
        let found = commit_file(repo.path(), &sha, &FilePaths::of(&files[0])).expect("diff").expect("renamed file");

        // then
        assert_eq!(summary(&files), [("R", "new.txt".to_owned(), Some("old.txt".to_owned())), ("M", "other.txt".to_owned(), None)]);
        assert_eq!((found.mark, found.line_count()), (Mark::Renamed, 5));
    }

    #[test]
    fn should_leave_the_index_alone_while_reading() {
        // given
        let repo = TestRepo::init();
        repo.write("a.txt", "a\n");
        let fork = Sha::of(repo.commit("first"));
        repo.write("a.txt", "a\n");
        repo.write("b.txt", "b\n");
        let index = repo.repo.path().join("index");
        let before = (fs::read(&index).expect("index"), fs::metadata(&index).and_then(|meta| meta.modified()).expect("mtime"));

        // when
        status::changes(repo.path()).expect("status");
        uncommitted(repo.path(), &file("b.txt")).expect("diff");
        since_file(repo.path(), &SinceRange { fork, uncommitted: Uncommitted::Included }, &file("a.txt")).expect("diff");

        // then
        let after = (fs::read(&index).expect("index"), fs::metadata(&index).and_then(|meta| meta.modified()).expect("mtime"));
        assert!(before == after, "the index changed");
        assert!(!repo.repo.path().join("index.lock").exists());
    }

    fn edited_lines(diff: EditedDiff) -> Vec<(LineKind, Option<u32>, Option<u32>, String)> {
        lines(&FileDiff { path: path("a.txt"), old_path: None, mark: Mark::Modified, body: diff.body })
    }

    #[test]
    fn should_diff_an_edited_text_against_heads_text_as_a_checkout_writes_it() {
        // given
        let repo = TestRepo::init();
        repo.write(".gitattributes", "*.txt text eol=crlf\n");
        repo.write("a.txt", "one\ntwo\n");
        repo.commit("first");

        // when
        let found = edited_uncommitted(repo.path(), &file("a.txt"), b"one\r\nTWO\r\n").expect("diff");

        // then
        let old = found.old.as_ref().map(|old| old.as_bytes().to_vec());
        assert_eq!(old, Some(b"one\r\ntwo\r\n".to_vec()));
        assert_eq!(edited_lines(found), [
            (LineKind::Context, Some(1), Some(1), "one".to_owned()),
            (LineKind::Removed, Some(2), None, "two".to_owned()),
            (LineKind::Added, None, Some(2), "TWO".to_owned()),
        ]);
    }

    #[test]
    fn should_answer_the_head_text_in_working_tree_form() {
        // given
        let repo = TestRepo::init();
        repo.write(".gitattributes", "*.txt text eol=crlf\n");
        repo.write("a.txt", "one\ntwo\n");
        repo.write("big.txt", &"x\n".repeat(MAX_HIGHLIGHT_BYTES / 2 + 1));
        repo.commit("first");
        repo.write("new.txt", "new\n");
        let outside = tempfile::tempdir().unwrap();
        fs::write(outside.path().join("plain.txt"), "plain\n").unwrap();

        // when
        let tracked = change_base(&repo.root.join("a.txt")).expect("read");
        let others = ["new.txt", "big.txt"].map(|name| change_base(&repo.root.join(name)).expect("read"));
        let plain = change_base(&outside.path().join("plain.txt")).expect("read");

        // then
        let ChangeBase::Tracked { head, head_id } = tracked else {
            panic!("a.txt is in HEAD");
        };
        assert_eq!(head.as_bytes(), b"one\r\ntwo\r\n");
        assert_eq!(head_id, repo.repo.head().unwrap().target().unwrap().to_string());
        assert!(others.iter().chain([&plain]).all(|base| matches!(base, ChangeBase::Untracked)));
    }

    #[test]
    fn should_mark_every_line_of_a_staged_new_file_as_added() {
        // given
        let unborn = TestRepo::init();
        unborn.write("staged.txt", "one\n");
        unborn.stage("staged.txt");
        unborn.write("loose.txt", "two\n");
        let born = TestRepo::init();
        born.write("a.txt", "a\n");
        let first = born.commit("first");
        born.write("staged.txt", "one\n");
        born.stage("staged.txt");

        // when
        let before_commit = change_base(&unborn.root.join("staged.txt")).expect("read");
        let loose = change_base(&unborn.root.join("loose.txt")).expect("read");
        let after_commit = change_base(&born.root.join("staged.txt")).expect("read");

        // then
        let ChangeBase::Tracked { head, head_id } = before_commit else {
            panic!("a staged file is tracked");
        };
        assert_eq!((head.as_bytes(), head_id.as_str()), (b"".as_slice(), "unborn"));
        assert!(matches!(loose, ChangeBase::Untracked));
        let ChangeBase::Tracked { head, head_id } = after_commit else {
            panic!("a staged file is tracked");
        };
        assert_eq!((head.as_bytes(), head_id), (b"".as_slice(), first.to_string()));
    }

    #[test]
    fn should_skip_the_head_text_when_the_head_id_is_known_or_the_git_dir_is_another() {
        // given
        let repo = TestRepo::init();
        repo.write("a.txt", "a\n");
        let head = repo.commit("first").to_string();
        let other = TestRepo::init();
        let file = repo.root.join("a.txt");
        let ask = |known: &str, git_dir: Option<&Path>| change_base_since(&file, &BaseAsk { known, git_dir }).expect("read");

        // when
        let known = ask(&head, None);
        let stale = ask("old", Some(repo.repo.path()));
        let elsewhere = ask("old", Some(other.repo.path()));

        // then
        assert!(known.is_none() && elsewhere.is_none());
        assert!(matches!(stale, Some(ChangeBase::Tracked { .. })));
    }

    #[test]
    fn should_diff_an_edited_text_head_lacks_as_all_added() {
        // given
        let repo = TestRepo::init();
        repo.write("other.txt", "x\n");
        repo.commit("first");

        // when
        let found = edited_uncommitted(repo.path(), &file("a.txt"), b"new\n").expect("diff");

        // then
        assert_eq!(edited_lines(found), [(LineKind::Added, None, Some(1), "new".to_owned())]);
    }

    #[test]
    fn should_diff_an_edited_text_against_a_binary_head_as_binary() {
        // given
        let repo = TestRepo::init();
        repo.write_bytes(Path::new("a.txt"), b"one\0two\n");
        repo.commit("first");

        // when
        let found = edited_uncommitted(repo.path(), &file("a.txt"), b"one\n").expect("diff");

        // then
        assert_eq!((found.old.is_none(), found.body), (true, DiffBody::Binary));
    }

    /// Both sides of `paths` between `sides`, as text; `None` for a side without one.
    fn both(repo: &TestRepo, sides: &Sides, paths: &FilePaths) -> (Option<String>, Option<String>) {
        let text = |side: Option<Source>| side.map(|source| String::from_utf8(source.as_bytes().to_vec()).expect("utf-8"));
        let (old, new) = texts(repo.path(), sides, paths).expect("texts");
        (text(old), text(new))
    }

    #[test]
    fn should_read_the_old_and_new_text_of_each_source() {
        // given
        let repo = TestRepo::init();
        repo.write("a.txt", "fork\n");
        let root = Sha::of(repo.commit("root"));
        repo.write("a.txt", "head\n");
        let head = Sha::of(repo.commit("head"));
        repo.write("a.txt", "work tree\n");
        let since = |uncommitted| Sides::Since(SinceRange { fork: root.clone(), uncommitted });

        // when
        let found = [Sides::Uncommitted, since(Uncommitted::Excluded), since(Uncommitted::Included), Sides::Commit(head), Sides::Commit(root.clone())]
            .map(|sides| both(&repo, &sides, &file("a.txt")));

        // then
        let pair = |old: Option<&str>, new: &str| (old.map(str::to_owned), Some(new.to_owned()));
        assert_eq!(found, [
            pair(Some("head\n"), "work tree\n"),
            pair(Some("fork\n"), "head\n"),
            pair(Some("fork\n"), "work tree\n"),
            pair(Some("fork\n"), "head\n"),
            pair(None, "fork\n"),
        ]);
    }

    #[test]
    fn should_read_a_renamed_files_old_side_by_its_old_name_and_leave_out_a_missing_side() {
        // given
        let repo = TestRepo::init();
        repo.write("old.txt", "kept\n");
        repo.write("gone.txt", "deleted\n");
        repo.commit("first");
        repo.rename("old.txt", "new.txt");
        repo.remove("gone.txt");
        repo.write("added.txt", "added\n");

        // when
        let found = [renamed("old.txt", "new.txt"), file("gone.txt"), file("added.txt")].map(|paths| both(&repo, &Sides::Uncommitted, &paths));

        // then
        let text = |text: &str| Some(text.to_owned());
        assert_eq!(found, [(text("kept\n"), text("kept\n")), (text("deleted\n"), None), (None, text("added\n"))]);
    }

    #[test]
    fn should_leave_out_binary_sides_and_sides_too_large_to_colour() {
        // given
        let repo = TestRepo::init();
        repo.write_bytes(Path::new("data.bin"), &[0, 1, 2, 0, 3]);
        repo.write("big.txt", &"x\n".repeat(MAX_HIGHLIGHT_BYTES / 2 + 1));
        repo.commit("first");
        repo.write_bytes(Path::new("data.bin"), &[0, 1, 2, 0, 4]);
        repo.write("big.txt", &"y\n".repeat(MAX_HIGHLIGHT_BYTES / 2 + 1));

        // when
        let found = [file("data.bin"), file("big.txt")].map(|paths| both(&repo, &Sides::Uncommitted, &paths));

        // then
        assert_eq!(found, [(None, None), (None, None)]);
    }
}
