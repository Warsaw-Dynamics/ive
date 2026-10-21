//! What the tree's git folders list of a worktree: its uncommitted changes and, against its base branch, the files
//! and commits since the fork point. Names and marks only; diffs are read when a file is opened.

use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use super::base::{self, ForkPoint};
use super::diff;
use super::log::{self, History};
use super::repo::GitError;
use super::status::{self, Change};
use super::types::Sha;
use super::worktrees::BaseBranch;

#[derive(Debug, Clone)]
pub struct WorktreeData {
    pub changes: Vec<Change>,
    /// `Err` says why there is no base branch or fork point
    pub since: Result<SinceData, String>,
}

/// What Since base and Commits list for one fork point and HEAD.
#[derive(Debug, Clone)]
pub struct SinceData {
    pub base: BaseBranch,
    pub fork: Sha,
    /// the base branch's tip the fork point was found from
    pub base_tip: Sha,
    /// fork point → HEAD; kept with `history` while neither moves
    pub committed: Vec<Change>,
    /// fork point → work tree: `committed` with the uncommitted changes on top
    pub with_uncommitted: Vec<Change>,
    pub history: History,
}

impl SinceData {
    fn reusable(&self, fork: &Sha, head: &Sha) -> bool {
        self.fork == *fork && self.history.head == *head
    }

    fn fork_point(&self) -> ForkPoint {
        ForkPoint::At { fork: self.fork.clone(), head: self.history.head.clone(), base_tip: self.base_tip.clone() }
    }
}

/// Where the branch leaves its base, and the uncommitted changes to put on top.
struct ForkAt<'a> {
    base: &'a BaseBranch,
    fork: Sha,
    head: Sha,
    base_tip: Sha,
    changes: &'a [Change],
}

/// Reads a worktree's lists; run it on the background executor. `previous` is reused while HEAD and the base's tip
/// stay, so a reload then reads a status only, one started at or after `status_since` (see [`status::shared`]).
#[tracing::instrument(level = "debug", skip_all, fields(dir = %dir.display()))]
pub fn load_worktree(dir: &Path, base: &Result<BaseBranch, String>, previous: Option<&SinceData>, status_since: Instant) -> Result<WorktreeData, GitError> {
    let changes = status::shared(dir, status_since)?.changes;
    let since = match base {
        Err(reason) => Err(reason.clone()),
        Ok(base) => match base::fork_point(dir, base, previous.map(SinceData::fork_point).as_ref())? {
            ForkPoint::Missing(reason) => Err(reason),
            ForkPoint::At { fork, head, base_tip } => Ok(since_data(dir, ForkAt { base, fork, head, base_tip, changes: &changes }, previous)?),
        },
    };
    Ok(WorktreeData { changes: Arc::unwrap_or_clone(changes), since })
}

fn since_data(dir: &Path, at: ForkAt<'_>, previous: Option<&SinceData>) -> Result<SinceData, GitError> {
    let (committed, history) = match previous.filter(|previous| previous.reusable(&at.fork, &at.head)) {
        Some(previous) => (previous.committed.clone(), previous.history.clone()),
        None => (diff::committed_changes(dir, &at.fork, &at.head)?, log::history(dir, &at.head, &at.fork)?),
    };
    let with_uncommitted = diff::with_uncommitted(&committed, at.changes);
    Ok(SinceData { base: at.base.clone(), fork: at.fork, base_tip: at.base_tip, committed, with_uncommitted, history })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::testing::TestRepo;

    fn main() -> Result<BaseBranch, String> {
        Ok(BaseBranch { name: "main".to_owned(), reference: "refs/heads/main".to_owned() })
    }

    fn names(changes: &[Change]) -> Vec<String> {
        changes.iter().map(|change| change.path.display()).collect()
    }

    /// main: base; topic (checked out): f.txt changed in one commit; g.txt changed in the work tree.
    fn topic_repo() -> TestRepo {
        let repo = TestRepo::init();
        repo.write("f.txt", "0\n");
        repo.write("g.txt", "0\n");
        repo.commit("base");
        repo.branch("topic");
        repo.checkout("topic");
        repo.write("f.txt", "1\n");
        repo.commit("on topic");
        repo.write("g.txt", "dirty\n");
        repo
    }

    #[test]
    fn should_load_changes_files_and_commits_against_the_base() {
        // given
        let repo = topic_repo();

        // when
        let data = load_worktree(repo.path(), &main(), None, Instant::now()).expect("load");

        // then
        let since = data.since.as_ref().expect("a base");
        let files = (names(&since.committed), names(&since.with_uncommitted));
        assert_eq!((data.changes.len(), files, since.history.commits.len()), (1, (vec!["f.txt".to_owned()], vec!["f.txt".to_owned(), "g.txt".to_owned()]), 1));
    }

    #[test]
    fn should_explain_a_missing_base_and_keep_uncommitted_working() {
        // given
        let repo = topic_repo();

        // when
        let data = load_worktree(repo.path(), &Err("dev has no upstream".to_owned()), None, Instant::now()).expect("load");

        // then
        assert_eq!((data.changes.len(), data.since.err()), (1, Some("dev has no upstream".to_owned())));
    }

    #[test]
    fn should_reuse_history_and_committed_files_while_head_and_fork_point_stay() {
        // given
        let repo = topic_repo();
        let first = load_worktree(repo.path(), &main(), None, Instant::now()).expect("load");
        let mut stale = first.since.expect("a base");
        stale.history.commits[0].subject = "kept from the last load".to_owned();
        stale.committed.clear();

        // when
        repo.write("h.txt", "new\n");
        let kept = load_worktree(repo.path(), &main(), Some(&stale), Instant::now()).expect("load");
        repo.commit("moved on");
        let reread = load_worktree(repo.path(), &main(), Some(&stale), Instant::now()).expect("load");

        // then
        let seen = |data: &WorktreeData| data.since.as_ref().map(|since| (since.history.commits[0].subject.clone(), since.committed.len(), names(&since.with_uncommitted))).expect("a base");
        assert_eq!(seen(&kept), ("kept from the last load".to_owned(), 0, vec!["g.txt".to_owned(), "h.txt".to_owned()]));
        assert_eq!(seen(&reread), ("moved on".to_owned(), 3, vec!["f.txt".to_owned(), "g.txt".to_owned(), "h.txt".to_owned()]));
    }
}
