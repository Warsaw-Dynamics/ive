use std::collections::BTreeMap;
use std::path::Path;

use git2::{Commit, Diff, DiffDelta, DiffFindOptions, DiffOptions, Patch, Repository};

use super::repo::{self, GitError};
use super::status::Change;
use super::types::{GitPath, Mark, Sha, failed, head_tree, lossy, mark_of_delta};

pub use codeview::diff::{Cut, DiffBody, DiffLine, FileDiff, Hunk, LineKind};
pub use codeview::diff::{FILE_LINE_LIMIT, MAX_FILE_BYTES, VIEW_LINE_LIMIT};

const CONTEXT_LINES: u32 = 3;
const TAB: &str = "    ";

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
    let delta = patch.delta();
    if delta.flags().is_binary() {
        let size = delta.new_file().size().max(delta.old_file().size());
        return Ok(if size > MAX_FILE_BYTES { DiffBody::TooLarge } else { DiffBody::Binary });
    }
    text(&patch, budget)
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
}
