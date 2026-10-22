use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::path::{Path, PathBuf};

use git2::{Commit, Oid, Repository};

use super::diff::{self, FileDiff};
use super::repo::{self, GitError};
use super::types::{GitPath, Sha, failed, head_tree, lossy};
use crate::age::When;

/// Commits read for the Since and Commits views; a branch this far ahead is not reviewed in IVE.
pub const SINCE_LIMIT: usize = 1_000;
const STASH_REF: &str = "refs/stash";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitInfo {
    pub sha: Sha,
    pub subject: String,
    pub author: String,
    pub when: When,
    pub parents: Vec<Sha>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitDetail {
    pub info: CommitInfo,
    pub message: String,
    pub files: Vec<FileDiff>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stash {
    pub index: usize,
    pub message: String,
    pub when: When,
}

/// The commits between a fork point and HEAD, as the Since and Commits views list them. It depends on HEAD and
/// the fork point only, so a reload with both unchanged reuses it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct History {
    pub head: Sha,
    pub fork: Sha,
    /// newest first, at most [`SINCE_LIMIT`]
    pub commits: Vec<CommitInfo>,
    /// the branch has more commits than [`SINCE_LIMIT`]
    pub more: bool,
    /// files each of `commits` changed against its first parent (renames counted once), in the same order
    pub file_counts: Vec<usize>,
    /// for each path, how many of `commits` changed it
    pub touched: HashMap<GitPath, usize>,
}

/// Which commits a log walks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommitRange {
    /// HEAD's history
    Head,
    /// every local branch, remote branch and tag
    AllRefs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Page {
    pub skip: usize,
    pub limit: usize,
}

/// Newest commit first; empty for an unborn HEAD. Reads only as far as the page ends.
pub fn commits(dir: &Path, range: &CommitRange, page: Page) -> Result<Vec<CommitInfo>, GitError> {
    let fail = failed("read the log", dir);
    let repo = repo::open(dir)?;
    let tips = match range {
        CommitRange::Head => head_tip(&repo),
        CommitRange::AllRefs => ref_tips(&repo),
    }
    .map_err(&fail)?;
    DateWalk::new(&repo, tips)
        .and_then(|walk| walk.skip(page.skip).take(page.limit).map(|commit| commit.map(|commit| info(&commit))).collect())
        .map_err(&fail)
}

/// The commits of HEAD's history that changed `path` (relative, a file or a folder), newest first; a file is followed
/// across renames. Reads only as far as the page ends.
pub fn path_history(dir: &Path, path: &Path, page: Page) -> Result<Vec<CommitInfo>, GitError> {
    let fail = failed("read the history of a path", dir);
    let repo = repo::open(dir)?;
    let walk = DateWalk::new(&repo, head_tip(&repo).map_err(&fail)?).map_err(&fail)?;
    let mut tracked = path.to_path_buf();
    let mut touching = Vec::new();
    for commit in walk {
        let commit = commit.map_err(&fail)?;
        let Some(change) = path_change(&repo, &commit, &tracked).map_err(&fail)? else {
            continue;
        };
        if let PathChange::RenamedFrom(old) = change {
            tracked = old;
        }
        touching.push(commit);
        if touching.len() == page.skip + page.limit {
            break;
        }
    }
    Ok(touching.iter().skip(page.skip).map(info).collect())
}

/// How a commit changed a path against its first parent.
enum PathChange {
    Changed,
    /// added under this name by renaming the one given
    RenamedFrom(PathBuf),
}

/// How `commit` changed `path` against its first parent; `None` when it did not.
fn path_change(repo: &Repository, commit: &Commit, path: &Path) -> Result<Option<PathChange>, git2::Error> {
    // the root: every commit
    if path.as_os_str().is_empty() {
        return Ok(Some(PathChange::Changed));
    }
    let entry = |tree: Option<&git2::Tree>| tree.and_then(|tree| tree.get_path(path).ok()).map(|entry| entry.id());
    let tree = commit.tree()?;
    let parent = commit.parents().next().map(|parent| parent.tree()).transpose()?;
    let (now, before) = (entry(Some(&tree)), entry(parent.as_ref()));
    if now == before {
        return Ok(None);
    }
    let added_file = before.is_none() && tree.get_path(path).is_ok_and(|entry| entry.kind() == Some(git2::ObjectType::Blob));
    match (added_file, parent) {
        (true, Some(parent)) => Ok(Some(renamed_from(repo, &parent, &tree, path)?.map_or(PathChange::Changed, PathChange::RenamedFrom))),
        _ => Ok(Some(PathChange::Changed)),
    }
}

/// The name `path` had in `before` when `after` renamed it.
fn renamed_from(repo: &Repository, before: &git2::Tree, after: &git2::Tree, path: &Path) -> Result<Option<PathBuf>, git2::Error> {
    let mut diff = repo.diff_tree_to_tree(Some(before), Some(after), None)?;
    diff.find_similar(Some(git2::DiffFindOptions::new().renames(true)))?;
    let renamed = diff.deltas().find(|delta| delta.status() == git2::Delta::Renamed && delta.new_file().path() == Some(path));
    Ok(renamed.and_then(|delta| delta.old_file().path().map(Path::to_path_buf)))
}

/// The commits from `fork` (excluded) to `head`, newest first, at most [`SINCE_LIMIT`]; each is diffed once against
/// its first parent for the counts.
pub fn history(dir: &Path, head: &Sha, fork: &Sha) -> Result<History, GitError> {
    let fail = failed("read the log", dir);
    let repo = repo::open(dir)?;
    let walk = BranchWalk::start(&repo, head.oid().map_err(&fail)?, fork.oid().map_err(&fail)?).map_err(&fail)?;
    let (commits, more) = walk.commits(SINCE_LIMIT).map_err(&fail)?;
    let mut history = History { head: head.clone(), fork: fork.clone(), commits: Vec::new(), more, file_counts: Vec::new(), touched: HashMap::new() };
    for commit in commits {
        let changes = diff::commit_deltas(&repo, &commit).map_err(&fail)?;
        history.file_counts.push(changes.len());
        for change in changes {
            *history.touched.entry(change.path).or_insert(0) += 1;
        }
        history.commits.push(info(&commit));
    }
    Ok(history)
}

pub fn commit(dir: &Path, sha: &Sha) -> Result<CommitDetail, GitError> {
    let fail = failed("read a commit", dir);
    let repo = repo::open(dir)?;
    let commit = repo.find_commit(sha.oid().map_err(&fail)?).map_err(&fail)?;
    let files = diff::files_of(diff::commit_diff(&repo, &commit, &mut diff::options()).map_err(&fail)?).map_err(&fail)?;
    Ok(CommitDetail { info: info(&commit), message: lossy(commit.message_bytes()).trim_end().to_owned(), files })
}

/// Stashes are shared by every worktree: the reflog of `refs/stash`, newest first. Messages that are not UTF-8
/// show lossily.
pub fn stashes(dir: &Path) -> Result<Vec<Stash>, GitError> {
    let fail = failed("read the stashes", dir);
    let repo = repo::open(dir)?;
    let reflog = repo.reflog(STASH_REF).map_err(&fail)?;
    reflog
        .iter()
        .enumerate()
        .map(|(index, entry)| {
            let message = entry.message_bytes().map(lossy).unwrap_or_default();
            repo.find_commit(entry.id_new()).map(|commit| Stash { index, message, when: when(&commit) })
        })
        .collect::<Result<_, git2::Error>>()
        .map_err(&fail)
}

/// HEAD's commit; none while HEAD is unborn.
fn head_tip(repo: &Repository) -> Result<Vec<Oid>, git2::Error> {
    if head_tree(repo)?.is_none() {
        return Ok(Vec::new());
    }
    Ok(vec![repo.head()?.peel_to_commit()?.id()])
}

/// The commits local branches, remote branches and tags point at.
fn ref_tips(repo: &Repository) -> Result<Vec<Oid>, git2::Error> {
    let mut tips = Vec::new();
    for reference in repo.references()? {
        let reference = reference?;
        let listed = reference.is_branch() || reference.is_remote() || reference.is_tag();
        // a tag of a tree or blob has no history
        if listed && let Ok(commit) = reference.peel_to_commit() {
            tips.push(commit.id());
        }
    }
    Ok(tips)
}

/// Commits by committer time, newest first. A commit's parents are queued when it comes out, so a walk reads only
/// as far as it is iterated (libgit2's sorted revwalk reads all reachable history before its first commit).
struct DateQueue<'r> {
    repo: &'r Repository,
    queue: BinaryHeap<(i64, Reverse<usize>, Oid)>,
    queued: usize,
}

impl<'r> DateQueue<'r> {
    fn new(repo: &'r Repository) -> Self {
        Self { repo, queue: BinaryHeap::new(), queued: 0 }
    }

    fn push(&mut self, oid: Oid) -> Result<(), git2::Error> {
        let time = self.repo.find_commit(oid)?.time().seconds();
        self.queued += 1;
        self.queue.push((time, Reverse(self.queued), oid));
        Ok(())
    }

    fn oids(&self) -> impl Iterator<Item = Oid> + '_ {
        self.queue.iter().map(|(_, _, oid)| *oid)
    }

    /// The newest commit. Of several made in the same second, one no other of them descends from: a branch tip of the
    /// same second as its parent's tip (a script, a rebase) would otherwise come out before its child.
    fn pop(&mut self) -> Result<Option<Oid>, git2::Error> {
        let Some(newest) = self.queue.pop() else {
            return Ok(None);
        };
        let mut same = vec![newest];
        while let Some(next) = self.queue.peek().filter(|next| next.0 == newest.0).copied() {
            self.queue.pop();
            same.push(next);
        }
        let mut chosen = 0;
        for (index, candidate) in same.iter().enumerate() {
            if self.is_newest(candidate.2, &same)? {
                chosen = index;
                break;
            }
        }
        let picked = same.swap_remove(chosen);
        self.queue.extend(same);
        Ok(Some(picked.2))
    }

    fn is_newest(&self, oid: Oid, same: &[(i64, Reverse<usize>, Oid)]) -> Result<bool, git2::Error> {
        for (_, _, other) in same {
            if *other != oid && self.repo.graph_descendant_of(*other, oid)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

/// The commits reachable from some tips, newest first.
struct DateWalk<'r> {
    queue: DateQueue<'r>,
    seen: HashSet<Oid>,
}

impl<'r> DateWalk<'r> {
    fn new(repo: &'r Repository, tips: Vec<Oid>) -> Result<Self, git2::Error> {
        let mut walk = Self { queue: DateQueue::new(repo), seen: HashSet::new() };
        for tip in tips {
            walk.queue_commit(tip)?;
        }
        Ok(walk)
    }

    fn queue_commit(&mut self, oid: Oid) -> Result<(), git2::Error> {
        if self.seen.insert(oid) {
            self.queue.push(oid)?;
        }
        Ok(())
    }

    fn next_commit(&mut self) -> Result<Option<Commit<'r>>, git2::Error> {
        let Some(oid) = self.queue.pop()? else {
            return Ok(None);
        };
        let commit = self.queue.repo.find_commit(oid)?;
        for parent in commit.parent_ids() {
            self.queue_commit(parent)?;
        }
        Ok(Some(commit))
    }
}

impl<'r> Iterator for DateWalk<'r> {
    type Item = Result<Commit<'r>, git2::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_commit().transpose()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Side {
    /// reached from HEAD only
    Branch,
    /// reached from the fork point
    Fork,
}

/// The commits `head` has and `fork` does not. Both sides walk by date together and the walk ends once only commits
/// `fork` reaches are queued, so it reads the branch and a little of what lies behind the fork point, never all
/// history.
struct BranchWalk<'r> {
    queue: DateQueue<'r>,
    sides: HashMap<Oid, Side>,
}

impl<'r> BranchWalk<'r> {
    fn start(repo: &'r Repository, head: Oid, fork: Oid) -> Result<Self, git2::Error> {
        let mut walk = Self { queue: DateQueue::new(repo), sides: HashMap::new() };
        walk.queue_commit(head, Side::Branch)?;
        walk.queue_commit(fork, Side::Fork)?;
        Ok(walk)
    }

    /// Newest first, at most `limit`, and whether there are more. A commit listed early and found reachable from
    /// `fork` later (clocks out of step) is dropped again.
    fn commits(mut self, limit: usize) -> Result<(Vec<Commit<'r>>, bool), git2::Error> {
        let mut listed = Vec::new();
        while listed.len() <= limit && self.branch_queued() {
            let Some(oid) = self.queue.pop()? else {
                break;
            };
            let side = self.side(oid);
            let commit = self.queue.repo.find_commit(oid)?;
            for parent in commit.parent_ids() {
                self.queue_commit(parent, side)?;
            }
            if side == Side::Branch {
                listed.push(commit);
            }
        }
        listed.retain(|commit| self.side(commit.id()) == Side::Branch);
        let more = listed.len() > limit;
        listed.truncate(limit);
        Ok((listed, more))
    }

    fn side(&self, oid: Oid) -> Side {
        self.sides.get(&oid).copied().unwrap_or(Side::Fork)
    }

    fn branch_queued(&self) -> bool {
        self.queue.oids().any(|oid| self.side(oid) == Side::Branch)
    }

    /// A commit both sides reach belongs to the fork point's, and so does every commit it leads to that was seen.
    fn queue_commit(&mut self, commit: Oid, side: Side) -> Result<(), git2::Error> {
        match (self.sides.get(&commit), side) {
            (None, _) => {
                self.sides.insert(commit, side);
                self.queue.push(commit)?;
            }
            (Some(Side::Branch), Side::Fork) => self.hide_from(commit)?,
            (Some(_), _) => {}
        }
        Ok(())
    }

    fn hide_from(&mut self, start: Oid) -> Result<(), git2::Error> {
        let mut stack = vec![start];
        while let Some(oid) = stack.pop() {
            if self.sides.insert(oid, Side::Fork) == Some(Side::Branch) {
                stack.extend(self.queue.repo.find_commit(oid)?.parent_ids().filter(|parent| self.sides.contains_key(parent)));
            }
        }
        Ok(())
    }
}

fn info(commit: &Commit<'_>) -> CommitInfo {
    CommitInfo {
        sha: Sha::of(commit.id()),
        subject: commit.summary_bytes().map(lossy).unwrap_or_default(),
        author: lossy(commit.author().name_bytes()),
        when: when(commit),
        parents: commit.parent_ids().map(Sha::of).collect(),
    }
}

fn when(commit: &Commit<'_>) -> When {
    let time = commit.time();
    When { seconds: time.seconds(), offset_minutes: time.offset_minutes() }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::git::testing::TestRepo;

    const ALL: Page = Page { skip: 0, limit: 100 };

    fn subjects(commits: &[CommitInfo]) -> Vec<&str> {
        commits.iter().map(|commit| commit.subject.as_str()).collect()
    }

    /// main: base; topic: a+b, then a.
    fn topic_repo() -> (TestRepo, Sha, Sha) {
        let repo = TestRepo::init();
        repo.write("a.txt", "0\n");
        let fork = Sha::of(repo.commit("base"));
        repo.branch("topic");
        repo.checkout("topic");
        repo.write("a.txt", "1\n");
        repo.write("b.txt", "1\n");
        repo.commit("a and b");
        repo.write("a.txt", "2\n");
        let head = Sha::of(repo.commit("only a"));
        (repo, head, fork)
    }

    /// `count` commits on main, oldest first.
    fn linear_repo(count: usize) -> (TestRepo, Vec<Oid>) {
        let repo = TestRepo::init();
        let commits = (0..count)
            .map(|n| {
                repo.write("f.txt", &format!("{n}\n"));
                repo.commit(&format!("c{n}"))
            })
            .collect();
        (repo, commits)
    }

    #[test]
    fn should_list_commits_since_the_fork_point_newest_first_with_file_counts() {
        // given
        let (repo, head, fork) = topic_repo();

        // when
        let history = history(repo.path(), &head, &fork).expect("history");

        // then
        assert_eq!((subjects(&history.commits), history.file_counts, history.more), (vec!["only a", "a and b"], vec![1, 2], false));
    }

    #[test]
    fn should_count_commits_touching_each_path() {
        // given
        let (repo, head, fork) = topic_repo();

        // when
        let touched = history(repo.path(), &head, &fork).expect("history").touched;

        // then
        let mut counts: Vec<_> = touched.iter().map(|(path, count)| (path.display(), *count)).collect();
        counts.sort();
        assert_eq!(counts, [("a.txt".to_owned(), 2), ("b.txt".to_owned(), 1)]);
    }

    fn whole() -> Page {
        Page { skip: 0, limit: 100 }
    }

    #[test]
    fn should_list_only_commits_touching_the_path() {
        // given
        let repo = TestRepo::init();
        repo.write("a.txt", "a\n");
        repo.commit("add a");
        repo.write("b.txt", "b\n");
        repo.commit("add b");
        repo.write("a.txt", "a2\n");
        repo.commit("change a");

        // when
        let commits = path_history(repo.path(), Path::new("a.txt"), whole()).expect("history");

        // then
        assert_eq!(subjects(&commits), ["change a", "add a"]);
    }

    #[test]
    fn should_list_commits_touching_a_folder() {
        // given
        let repo = TestRepo::init();
        repo.write("src/a.rs", "a\n");
        repo.commit("add src");
        repo.write("README.md", "r\n");
        repo.commit("readme");
        repo.write("src/b.rs", "b\n");
        repo.commit("more src");

        // when
        let commits = path_history(repo.path(), Path::new("src"), whole()).expect("history");

        // then
        assert_eq!(subjects(&commits), ["more src", "add src"]);
    }

    #[test]
    fn should_list_every_commit_for_the_root() {
        // given
        let repo = TestRepo::init();
        repo.write("a.txt", "a\n");
        repo.commit("one");
        repo.write("b.txt", "b\n");
        repo.commit("two");

        // when
        let commits = path_history(repo.path(), Path::new(""), whole()).expect("history");

        // then
        assert_eq!(subjects(&commits), ["two", "one"]);
    }

    #[test]
    fn should_follow_a_renamed_file() {
        // given
        let repo = TestRepo::init();
        repo.write("old.txt", "line 1\nline 2\nline 3\nline 4\nline 5\n");
        repo.commit("add old");
        repo.write("old.txt", "line 1\nline 2\nline 3\nline 4\nline 5\nline 6\n");
        repo.commit("grow old");
        repo.rename("old.txt", "new.txt");
        repo.commit("rename");
        repo.write("other.txt", "x\n");
        repo.commit("unrelated");

        // when
        let commits = path_history(repo.path(), Path::new("new.txt"), whole()).expect("history");

        // then
        assert_eq!(subjects(&commits), ["rename", "grow old", "add old"]);
    }

    #[test]
    fn should_count_a_renamed_file_once() {
        // given
        let repo = TestRepo::init();
        repo.write("old.txt", "line 1\nline 2\nline 3\nline 4\nline 5\n");
        let fork = Sha::of(repo.commit("base"));
        repo.rename("old.txt", "new.txt");
        let head = Sha::of(repo.commit("rename"));

        // when
        let history = history(repo.path(), &head, &fork).expect("history");

        // then
        let touched: Vec<_> = history.touched.keys().map(GitPath::display).collect();
        assert_eq!((history.file_counts, touched), (vec![1], vec!["new.txt".to_owned()]));
    }

    #[test]
    fn should_stop_at_the_limit_and_say_there_are_more() {
        // given
        let (repo, commits) = linear_repo(5);
        let walk = || BranchWalk::start(&repo.repo, commits[4], commits[0]).expect("walk");

        // when
        let (cut, more) = walk().commits(3).expect("walk");
        let (whole, no_more) = walk().commits(4).expect("walk");

        // then
        assert_eq!((cut.len(), more, whole.len(), no_more), (3, true, 4, false));
    }

    #[test]
    fn should_read_no_further_than_the_page_or_the_fork_point() {
        // given
        let (repo, oids) = linear_repo(6);
        // the root commit is gone, so a walk that reads that far fails
        repo.remove_object(oids[0]);

        // when
        let page = commits(repo.path(), &CommitRange::Head, Page { skip: 1, limit: 2 }).expect("log");
        let since = history(repo.path(), &Sha::of(oids[5]), &Sha::of(oids[2])).expect("history");

        // then
        assert_eq!((subjects(&page), subjects(&since.commits)), (vec!["c4", "c3"], vec!["c5", "c4", "c3"]));
    }

    #[test]
    fn should_read_merge_commit_with_parents_and_first_parent_files() {
        // given
        let repo = TestRepo::init();
        repo.write("base.txt", "base\n");
        repo.commit("root");
        repo.branch("side");
        repo.checkout("side");
        repo.write("side.txt", "side\n");
        let side = repo.commit("on side");
        repo.checkout("main");
        repo.write("main.txt", "main\n");
        let main = repo.commit("on main");
        repo.write("side.txt", "side\n");
        let merge = Sha::of(repo.merge("merge side\n\nbody line", side));

        // when
        let detail = commit(repo.path(), &merge).expect("commit");

        // then
        let files: Vec<_> = detail.files.iter().map(|file| file.path.display()).collect();
        assert_eq!(detail.info.parents, [Sha::of(main), Sha::of(side)]);
        assert_eq!((detail.info.subject.as_str(), detail.message.as_str(), files), ("merge side", "merge side\n\nbody line", vec!["side.txt".to_owned()]));
    }

    #[test]
    fn should_list_no_commits_and_no_stashes_for_unborn_head() {
        // given
        let repo = TestRepo::init();

        // when
        let log = commits(repo.path(), &CommitRange::Head, ALL).expect("log");
        let stashed = stashes(repo.path()).expect("stashes");

        // then
        assert_eq!((log.len(), stashed.len()), (0, 0));
    }

    #[test]
    fn should_walk_all_branches_by_time_in_pages() {
        // given
        let repo = TestRepo::init();
        repo.write("f.txt", "1\n");
        repo.commit("c1");
        repo.write("f.txt", "2\n");
        repo.commit("c2");
        repo.branch("other");
        repo.write("f.txt", "3\n");
        repo.commit("c3");
        repo.checkout("other");
        repo.write("f.txt", "4\n");
        repo.commit("c4");

        // when
        let page = commits(repo.path(), &CommitRange::AllRefs, Page { skip: 1, limit: 2 }).expect("log");

        // then
        assert_eq!(subjects(&page), ["c3", "c2"]);
    }

    #[test]
    fn should_list_a_child_before_its_parent_made_in_the_same_second() {
        // given
        let repo = TestRepo::init();
        let signature = git2::Signature::new("Test Author", "test@example.com", &git2::Time::new(1_700_000_000, 0)).unwrap();
        let tree = repo.repo.find_tree(repo.repo.index().unwrap().write_tree().unwrap()).unwrap();
        let first = repo.repo.commit(Some("HEAD"), &signature, &signature, "first", &tree, &[]).unwrap();
        let parent = repo.repo.find_commit(first).unwrap();
        repo.repo.commit(Some("HEAD"), &signature, &signature, "second", &tree, &[&parent]).unwrap();
        // listed before `main`, so its older tip is queued first
        repo.repo.branch("a-first", &parent, false).unwrap();

        // when
        let log = commits(repo.path(), &CommitRange::AllRefs, ALL).expect("log");

        // then
        assert_eq!(subjects(&log), ["second", "first"]);
    }

    #[test]
    fn should_list_stashes_newest_first_and_show_other_bytes_lossily() {
        // given
        let mut repo = TestRepo::init();
        repo.write("f.txt", "0\n");
        repo.commit("base");
        repo.write("f.txt", "1\n");
        repo.stash("first");
        repo.write("f.txt", "2\n");
        repo.stash("second");
        let log = repo.repo.path().join("logs").join(STASH_REF);
        let bytes = fs::read(&log).expect("stash log");
        let at = bytes.windows(5).position(|window| window == b"first").expect("first");
        fs::write(&log, [&bytes[..at], b"f\xe9rst", &bytes[at + 5..]].concat()).expect("stash log");

        // when
        let stashed = stashes(repo.path()).expect("stashes");

        // then
        let listed: Vec<_> = stashed.iter().map(|stash| (stash.index, stash.message.as_str())).collect();
        assert_eq!(listed, [(0, "On main: second"), (1, "On main: f\u{fffd}rst")]);
    }
}
