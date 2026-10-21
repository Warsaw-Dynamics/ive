use std::path::Path;

use git2::ErrorCode;

use super::repo::{self, GitError};
use super::types::{Sha, failed, head_tree};
use super::worktrees::{BaseBranch, base_of};
use crate::project::{Checkout, Project};

const NO_COMMIT: &str = "no commit yet";

/// Where a branch left its base, or why that cannot be told.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ForkPoint {
    /// with the HEAD and base branch tip it was computed for
    At { fork: Sha, head: Sha, base_tip: Sha },
    Missing(String),
}

/// The checkout's base branch (`base_of`), or the reason it has none.
pub fn base(project: &Project, checkout: &Checkout) -> Result<BaseBranch, String> {
    match base_of(project, checkout) {
        Some(base) if checkout.head.is_some() => Ok(base),
        _ => Err(no_base_reason(project.main_checkout(), checkout)),
    }
}

pub fn no_base_reason(main: &Checkout, checkout: &Checkout) -> String {
    if checkout.head.is_none() {
        return NO_COMMIT.to_owned();
    }
    match (checkout.is_main, &main.branch) {
        (true, Some(branch)) => format!("{branch} has no upstream"),
        (true, None) => "HEAD is detached".to_owned(),
        (false, None) => "the main checkout is detached".to_owned(),
        (false, Some(branch)) => format!("{branch} cannot be read"),
    }
}

/// `merge_base(base, HEAD)`; `known` is handed back without one while HEAD and the base's tip are where it was
/// computed, as each poll of a worktree's git folders asks again.
pub fn fork_point(dir: &Path, base: &BaseBranch, known: Option<&ForkPoint>) -> Result<ForkPoint, GitError> {
    let fail = failed("find the fork point", dir);
    let repo = repo::open(dir)?;
    if head_tree(&repo).map_err(&fail)?.is_none() {
        return Ok(ForkPoint::Missing(NO_COMMIT.to_owned()));
    }
    let Ok(base_commit) = repo.find_reference(&base.reference).and_then(|reference| reference.peel_to_commit()) else {
        return Ok(ForkPoint::Missing(format!("{} not found", base.name)));
    };
    let head_oid = repo.head().and_then(|head| head.peel_to_commit()).map_err(&fail)?.id();
    let (head, base_tip) = (Sha::of(head_oid), Sha::of(base_commit.id()));
    if let Some(known @ ForkPoint::At { head: known_head, base_tip: known_tip, .. }) = known
        && (known_head, known_tip) == (&head, &base_tip)
    {
        return Ok(known.clone());
    }
    match repo.merge_base(head_oid, base_commit.id()) {
        Ok(fork) => Ok(ForkPoint::At { fork: Sha::of(fork), head, base_tip }),
        Err(error) if error.code() == ErrorCode::NotFound => Ok(ForkPoint::Missing(format!("no commit in common with {}", base.name))),
        Err(error) => Err(fail(error)),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::git::testing::TestRepo;

    fn checkout(branch: Option<&str>, head: Option<&str>, is_main: bool) -> Checkout {
        Checkout {
            path: PathBuf::from("/repo"),
            label: branch.unwrap_or("HEAD").to_owned(),
            branch: branch.map(str::to_owned),
            head: head.map(str::to_owned),
            is_main,
            status: None,
        }
    }

    fn local(name: &str) -> BaseBranch {
        BaseBranch { name: name.to_owned(), reference: format!("refs/heads/{name}") }
    }

    #[test]
    fn should_explain_why_a_checkout_has_no_base() {
        // given
        let main = checkout(Some("dev"), Some("abc1234"), true);
        let detached_main = checkout(None, Some("abc1234"), true);
        let worktree = checkout(Some("topic"), Some("abc1234"), false);
        let unborn = checkout(Some("topic"), None, false);

        // when
        let reasons = [
            no_base_reason(&main, &main),
            no_base_reason(&detached_main, &detached_main),
            no_base_reason(&detached_main, &worktree),
            no_base_reason(&main, &unborn),
        ];

        // then
        assert_eq!(reasons, ["dev has no upstream", "HEAD is detached", "the main checkout is detached", "no commit yet"]);
    }

    #[test]
    fn should_find_where_a_branch_left_its_base() {
        // given
        let repo = TestRepo::init();
        repo.write("f.txt", "0\n");
        let fork = Sha::of(repo.commit("base"));
        repo.branch("topic");
        repo.write("f.txt", "main moved on\n");
        let base_tip = Sha::of(repo.commit("on main"));
        repo.checkout("topic");
        repo.write("f.txt", "topic\n");
        let head = Sha::of(repo.commit("on topic"));

        // when
        let found = fork_point(repo.path(), &local("main"), None).expect("fork point");

        // then
        assert_eq!(found, ForkPoint::At { fork, head, base_tip });
    }

    #[test]
    fn should_keep_a_known_fork_point_until_head_or_the_base_moves() {
        // given
        let repo = TestRepo::init();
        repo.write("f.txt", "0\n");
        let fork = Sha::of(repo.commit("base"));
        repo.branch("topic");
        repo.checkout("topic");
        repo.write("f.txt", "topic\n");
        let head = Sha::of(repo.commit("on topic"));
        // a fork point no merge base would give: only reuse hands it back
        let known = ForkPoint::At { fork: head.clone(), head: head.clone(), base_tip: fork.clone() };

        // when
        let kept = fork_point(repo.path(), &local("main"), Some(&known)).expect("kept");
        repo.write("f.txt", "topic again\n");
        let moved = Sha::of(repo.commit("again"));
        let recomputed = fork_point(repo.path(), &local("main"), Some(&known)).expect("recomputed");

        // then
        assert_eq!([kept, recomputed], [known.clone(), ForkPoint::At { fork: fork.clone(), head: moved, base_tip: fork }]);
    }

    #[test]
    fn should_say_why_no_fork_point_exists() {
        // given
        let empty = TestRepo::init();
        let repo = TestRepo::init();
        repo.write("f.txt", "0\n");
        repo.commit("base");

        // when
        let remote = BaseBranch { name: "origin/main".to_owned(), reference: "refs/remotes/origin/main".to_owned() };
        let found = [fork_point(repo.path(), &remote, None).expect("missing base"), fork_point(empty.path(), &local("main"), None).expect("unborn")];

        // then
        assert_eq!(found, [ForkPoint::Missing("origin/main not found".to_owned()), ForkPoint::Missing("no commit yet".to_owned())]);
    }

    #[test]
    fn should_take_the_base_branch_over_a_tag_of_the_same_name() {
        // given
        let repo = TestRepo::init();
        repo.write("f.txt", "0\n");
        let fork = Sha::of(repo.commit("base"));
        repo.branch("dev");
        repo.write("f.txt", "1\n");
        repo.commit("later");
        repo.tag("dev");
        repo.write("f.txt", "2\n");
        let head = Sha::of(repo.commit("on main"));

        // when
        let found = fork_point(repo.path(), &local("dev"), None).expect("fork point");

        // then
        assert_eq!(found, ForkPoint::At { fork: fork.clone(), head, base_tip: fork });
    }
}
