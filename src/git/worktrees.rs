//! The checkouts of a repository and how each stands: branch, change count, distance from its base.
//! Read only: status runs without `update_index`, so nothing here writes the index or takes locks.

use std::path::{Path, PathBuf};
use std::time::Instant;

use git2::{Branch, BranchType, ErrorCode, Oid, Repository};

use super::repo::{self, GitError, short_sha};
use super::status;
use crate::parallel::in_parallel;
use crate::project::{Checkout, CheckoutStatus, Project, Statuses, folder_name};

/// The main checkout of the repository holding `folder` (a worktree, a subfolder, or the checkout itself);
/// `None` when `folder` is in no repository, or in a bare one.
pub fn main_checkout_of(folder: &Path) -> Result<Option<PathBuf>, GitError> {
    let repository = match Repository::discover(folder) {
        Ok(repository) => repository,
        Err(error) if error.code() == ErrorCode::NotFound => return Ok(None),
        Err(error) => return Err(GitError::new("find the repository", folder, error)),
    };
    if repository.is_bare() {
        return Ok(None);
    }
    let common = repository.commondir();
    // a worktree's common dir is the main checkout's `.git`; a submodule's lies under the superproject's
    // `.git/modules`, and its work tree is then configured in it
    let main = if common.file_name().is_some_and(|name| name == ".git") {
        common.parent().map(Path::to_path_buf)
    } else {
        repo::open(common)?.workdir().map(Path::to_path_buf)
    };
    Ok(main.map(|main| main.canonicalize().unwrap_or(main)))
}

/// The common git dir of the repository whose main checkout is `main`, canonical: `<main>/.git` but for
/// `--separate-git-dir` repositories.
pub fn common_dir(main: &Path) -> Result<PathBuf, GitError> {
    let common = repo::open(main)?.commondir().to_path_buf();
    Ok(common.canonicalize().unwrap_or(common))
}

/// The checkouts of the repository whose main checkout is `main`: the main one, then each linked worktree
/// whose folder still exists, in `git worktree list` order. Status is read for the checkouts `statuses` names, from
/// a read started at or after `since` (see [`status::shared`]).
pub fn read_checkouts(main: &Path, statuses: &Statuses, since: Instant) -> Result<Vec<Checkout>, GitError> {
    let repository = repo::open(main)?;
    let main_head = read_head(&repository, main)?;
    let base = worktree_base(&repository, main_head.branch.as_deref());
    let mut dirs = vec![(main.to_path_buf(), upstream_of(&repository, main_head.branch.as_deref()))];
    dirs.extend(linked_worktrees(&repository, main)?.into_iter().map(|dir| (dir, base.clone())));
    // a status takes ~130 ms on the atlassian monorepo; one after another they held up a project's first show
    let reading = Reading { main, statuses, since };
    let reads = in_parallel(&dirs, |(dir, base)| read_checkout(dir, base.clone(), &reading));
    let mut checkouts = Vec::with_capacity(reads.len());
    for ((dir, _), read) in dirs.iter().zip(reads) {
        match read {
            Ok(checkout) => checkouts.push(checkout),
            Err(error) if dir == main || dir.exists() => return Err(error),
            // deleted between listing and reading: skipped, as a pruned one
            Err(_) => {}
        }
    }
    Ok(checkouts)
}

/// The branch `checkout` is compared with: the main checkout's upstream (`origin/dev`) for the main checkout,
/// and for the others too unless it has none, then the main checkout's branch (`dev`). Reads git; run it on the
/// background executor.
pub fn base_of(project: &Project, checkout: &Checkout) -> Option<BaseBranch> {
    let main = project.main_checkout();
    let repository = repo::open(&main.path).ok()?;
    let base = if checkout.is_main {
        upstream_of(&repository, main.branch.as_deref())
    } else {
        worktree_base(&repository, main.branch.as_deref())
    };
    base.map(|base| BaseBranch { name: base.name, reference: base.reference })
}

/// A base branch as shown (`origin/dev`) and its full reference name (`refs/remotes/origin/dev`), which a tag
/// of the same short name cannot shadow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaseBranch {
    pub name: String,
    pub reference: String,
}

/// A checkout's label: the main checkout shows its branch, others their folder name, which stays put when
/// the branch in them changes.
pub fn checkout_label(dir: &Path, main: &Path, branch: Option<&str>, head: Option<Oid>) -> String {
    if dir != main {
        return folder_name(dir);
    }
    match (branch, head) {
        (Some(branch), _) => branch.to_owned(),
        (None, Some(oid)) => format!("HEAD detached at {}", short_sha(oid)),
        (None, None) => "HEAD".to_owned(),
    }
}

struct Head {
    branch: Option<String>,
    oid: Option<Oid>,
}

/// What every checkout of one [`read_checkouts`] shares.
struct Reading<'a> {
    main: &'a Path,
    statuses: &'a Statuses,
    since: Instant,
}

#[derive(Clone)]
struct Base {
    name: String,
    reference: String,
    oid: Oid,
}

#[tracing::instrument(level = "debug", skip_all, fields(dir = %dir.display()))]
fn read_checkout(dir: &Path, base: Option<Base>, reading: &Reading<'_>) -> Result<Checkout, GitError> {
    let repository = repo::open(dir)?;
    let head = read_head(&repository, dir)?;
    let status = reading.statuses.includes(dir).then(|| read_status(&repository, dir, head.oid, base, reading.since)).transpose()?;
    Ok(Checkout {
        path: dir.to_path_buf(),
        label: checkout_label(dir, reading.main, head.branch.as_deref(), head.oid),
        head: head.oid.map(|oid| oid.to_string()),
        branch: head.branch,
        is_main: dir == reading.main,
        status,
    })
}

/// Change count and distance to the base: the slow part of a refresh (150 ms on the atlassian monorepo).
fn read_status(repository: &Repository, dir: &Path, head: Option<Oid>, base: Option<Base>, since: Instant) -> Result<CheckoutStatus, GitError> {
    let changes = status::shared(dir, since)?.changes.len();
    let (ahead, behind) = distance(repository, dir, head, base.as_ref())?;
    Ok(CheckoutStatus { changes, ahead, behind, base: base.map(|base| base.name) })
}

fn read_head(repository: &Repository, dir: &Path) -> Result<Head, GitError> {
    match repository.head() {
        Ok(head) => Ok(Head {
            branch: head.is_branch().then(|| head.shorthand().ok().map(str::to_owned)).flatten(),
            oid: head.target(),
        }),
        Err(error) if error.code() == ErrorCode::UnbornBranch => Ok(Head { branch: unborn_branch(repository), oid: None }),
        Err(error) => Err(GitError::new("read HEAD", dir, error)),
    }
}

/// The branch HEAD names in a repository without commits.
fn unborn_branch(repository: &Repository) -> Option<String> {
    let head = repository.find_reference("HEAD").ok()?;
    let target = head.symbolic_target().ok().flatten()?;
    Some(target.strip_prefix("refs/heads/").unwrap_or(target).to_owned())
}

/// Worktree folders, skipping those `validate` rejects (folder deleted outside git: prunable).
fn linked_worktrees(repository: &Repository, main: &Path) -> Result<Vec<PathBuf>, GitError> {
    let names = repository.worktrees().map_err(|error| GitError::new("list the worktrees", main, error))?;
    let dirs = names
        .iter()
        .filter_map(|name| name.ok().flatten())
        .filter_map(|name| repository.find_worktree(name).ok())
        .filter(|worktree| worktree.validate().is_ok())
        .map(|worktree| worktree.path().canonicalize().unwrap_or_else(|_| worktree.path().to_path_buf()))
        .collect();
    Ok(dirs)
}

fn upstream_of(repository: &Repository, branch: Option<&str>) -> Option<Base> {
    let local = repository.find_branch(branch?, BranchType::Local).ok()?;
    base_from(&local.upstream().ok()?)
}

/// A worktree merged and pushed to `origin/dev` from its own folder is level with it while the main checkout's
/// `dev` lags until pulled; comparing with the local branch would count the pushed commits as ahead.
fn worktree_base(repository: &Repository, branch: Option<&str>) -> Option<Base> {
    upstream_of(repository, branch).or_else(|| local_branch(repository, branch))
}

fn local_branch(repository: &Repository, branch: Option<&str>) -> Option<Base> {
    base_from(&repository.find_branch(branch?, BranchType::Local).ok()?)
}

fn base_from(branch: &Branch<'_>) -> Option<Base> {
    let reference = branch.get();
    Some(Base { name: branch.name().ok().flatten()?.to_owned(), reference: reference.name().ok()?.to_owned(), oid: reference.target()? })
}

#[tracing::instrument(level = "debug", skip_all)]
fn distance(repository: &Repository, dir: &Path, head: Option<Oid>, base: Option<&Base>) -> Result<(u32, u32), GitError> {
    let (Some(head), Some(base)) = (head, base) else {
        return Ok((0, 0));
    };
    let (ahead, behind) =
        repository.graph_ahead_behind(head, base.oid).map_err(|error| GitError::new("compare HEAD with the base", dir, error))?;
    Ok((saturate(ahead), saturate(behind)))
}

fn saturate(count: usize) -> u32 {
    u32::try_from(count).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::git::testing::{TestRepo, add_worktree, commit, write};
    use crate::project::{ProjectId, ProjectKind, ProjectScan};

    #[test]
    fn should_label_main_checkout_by_branch_or_detached_head_and_others_by_folder() {
        // given
        let oid = Oid::from_str("0123456789abcdef0123456789abcdef01234567").unwrap();
        let (main, worktree) = (Path::new("/r"), Path::new("/r/.claude/worktrees/GEN-817-ive"));

        // when
        let labels = [
            checkout_label(main, main, Some("dev"), Some(oid)),
            checkout_label(main, main, None, Some(oid)),
            checkout_label(main, main, None, None),
            checkout_label(worktree, main, Some("other-branch"), Some(oid)),
        ];

        // then
        assert_eq!(labels, ["dev", "HEAD detached at 0123456", "HEAD", "GEN-817-ive"]);
    }

    #[test]
    fn should_name_unborn_branch_of_new_repository() {
        // given
        let test = TestRepo::init();

        // when
        let checkouts = read_checkouts(&test.root, &Statuses::All, Instant::now()).unwrap();

        // then
        assert_eq!((checkouts[0].label.as_str(), checkouts[0].branch.as_deref(), checkouts[0].head.as_deref()), ("main", Some("main"), None));
    }

    #[test]
    fn should_count_untracked_files_in_new_folders_as_changes() {
        // given
        let test = TestRepo::init();
        commit(&test.repo, "a.txt", "one");
        write(&test.repo, "a.txt", "two");
        write(&test.repo, "new/deep/b.txt", "new");
        write(&test.repo, "new/deep/c.txt", "new");

        // when
        let checkouts = read_checkouts(&test.root, &Statuses::All, Instant::now()).unwrap();

        // then
        // without recursing into untracked folders, `new/` would count once
        assert_eq!(checkouts[0].status.as_ref().map(|status| status.changes), Some(3));
    }

    #[test]
    fn should_read_status_only_of_the_checkouts_asked_for() {
        // given
        let test = TestRepo::init();
        commit(&test.repo, "a.txt", "one");
        let worktree = test.sibling("feature");
        add_worktree(&test.repo, "feature", &worktree);
        let statuses = Statuses::Only([worktree.clone()].into());

        // when
        let checkouts = read_checkouts(&test.root, &statuses, Instant::now()).unwrap();

        // then
        let read: Vec<_> = checkouts.iter().map(|checkout| (checkout.label.as_str(), checkout.status.is_some())).collect();
        assert_eq!(read, [("main", false), ("feature", true)]);
    }

    #[test]
    fn should_skip_worktree_whose_folder_was_deleted() {
        // given
        let test = TestRepo::init();
        commit(&test.repo, "a.txt", "one");
        let kept = test.sibling("kept");
        let gone = test.sibling("gone");
        add_worktree(&test.repo, "kept", &kept);
        add_worktree(&test.repo, "gone", &gone);
        fs::remove_dir_all(&gone).unwrap();

        // when
        let checkouts = read_checkouts(&test.root, &Statuses::All, Instant::now()).unwrap();

        // then
        assert_eq!(checkouts.iter().map(|checkout| checkout.path.clone()).collect::<Vec<_>>(), [test.root.clone(), kept]);
    }

    #[test]
    fn should_see_worktree_added_after_the_first_read() {
        // given
        let test = TestRepo::init();
        commit(&test.repo, "a.txt", "one");
        let before = read_checkouts(&test.root, &Statuses::All, Instant::now()).unwrap().len();

        // when
        add_worktree(&test.repo, "later", &test.sibling("later"));
        let after = read_checkouts(&test.root, &Statuses::All, Instant::now()).unwrap().len();

        // then
        assert_eq!((before, after), (1, 2));
    }

    #[test]
    fn should_count_worktree_commits_ahead_and_behind_main_branch() {
        // given
        let test = TestRepo::init();
        commit(&test.repo, "a.txt", "one");
        let worktree = add_worktree(&test.repo, "feature", &test.sibling("feature"));
        commit(&worktree, "b.txt", "feature 1");
        commit(&worktree, "c.txt", "feature 2");
        commit(&test.repo, "d.txt", "main moved on");

        // when
        let checkouts = read_checkouts(&test.root, &Statuses::All, Instant::now()).unwrap();

        // then
        let status = checkouts[1].status.clone().unwrap();
        assert_eq!((status.ahead, status.behind, status.base.as_deref()), (2, 1, Some("main")));
        assert_eq!(checkouts[0].status.clone().unwrap().base, None, "main has no upstream");
    }

    #[test]
    fn should_use_upstream_as_base_of_main_checkout() {
        // given
        let test = TestRepo::init();
        let pushed = commit(&test.repo, "a.txt", "one");
        commit(&test.repo, "b.txt", "unpushed");
        test.repo.remote("origin", "https://example.com/repo.git").unwrap();
        test.repo.reference("refs/remotes/origin/main", pushed, false, "test").unwrap();
        test.repo.find_branch("main", BranchType::Local).unwrap().set_upstream(Some("origin/main")).unwrap();
        let checkouts = read_checkouts(&test.root, &Statuses::All, Instant::now()).unwrap();
        let project = Project::new(
            ProjectId::new("repo"),
            ProjectScan { root: test.root.clone(), kind: ProjectKind::Git, checkouts, error: None, git_dir: None },
        );

        // when
        let base = base_of(&project, project.main_checkout());

        // then
        let status = project.main_checkout().status.clone().unwrap();
        let base = base.expect("a base");
        assert_eq!((base.name.as_str(), base.reference.as_str(), status.ahead, status.behind), ("origin/main", "refs/remotes/origin/main", 1, 0));
    }

    #[test]
    fn should_compare_worktree_with_upstream_when_main_branch_lags_behind_it() {
        // given
        let test = TestRepo::init();
        let stale = commit(&test.repo, "a.txt", "one");
        let worktree = add_worktree(&test.repo, "feature", &test.sibling("feature"));
        let pushed = commit(&worktree, "b.txt", "merged and pushed");
        test.repo.remote("origin", "https://example.com/repo.git").unwrap();
        test.repo.reference("refs/remotes/origin/main", pushed, false, "test").unwrap();
        test.repo.find_branch("main", BranchType::Local).unwrap().set_upstream(Some("origin/main")).unwrap();

        // when
        let checkouts = read_checkouts(&test.root, &Statuses::All, Instant::now()).unwrap();

        // then
        let status = checkouts[1].status.clone().unwrap();
        assert_eq!((status.ahead, status.behind, status.base.as_deref()), (0, 0, Some("origin/main")));
        assert_eq!(test.repo.find_branch("main", BranchType::Local).unwrap().get().target(), Some(stale));
    }

    #[test]
    fn should_find_the_common_git_dir_of_the_main_checkout() {
        // given
        let test = TestRepo::init();

        // when
        let dir = common_dir(&test.root).unwrap();

        // then
        assert_eq!(dir, test.root.join(".git"));
    }

    #[test]
    fn should_find_main_checkout_from_inside_a_worktree() {
        // given
        let test = TestRepo::init();
        commit(&test.repo, "a.txt", "one");
        let worktree = test.sibling("feature");
        add_worktree(&test.repo, "feature", &worktree);

        // when
        let main = main_checkout_of(&worktree).unwrap();

        // then
        assert_eq!(main, Some(test.root.clone()));
    }
}
