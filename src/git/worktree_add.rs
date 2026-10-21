//! New worktree: IVE's only change to a repository.

use std::fs;
use std::path::{Path, PathBuf};

use git2::{BranchType, Repository, WorktreeAddOptions};

use super::refs::BranchName;
use super::repo::{self, GitError};
use super::types::failed;
use crate::ive_toml::worktrees_folder;

const OPERATION: &str = "add a worktree";
/// `topic`, `topic-2`, … `topic-100`; past that something else is wrong.
const MAX_NAME_TRIES: usize = 100;

/// `<main checkout>/<folder>/<branch with / replaced by ->`; `folder` as [`worktrees_folder`] reads it.
pub fn worktree_dir(main_dir: &Path, folder: &Path, branch: &BranchName) -> PathBuf {
    main_dir.join(folder).join(branch.local_name().replace('/', "-"))
}

/// Adds a worktree of `branch` in [`worktree_dir`], its folder from `.ive.toml`. A remote branch without a local one
/// first gets a local branch tracking it. Nothing is written when the folder exists or `.ive.toml` is refused; a
/// failure undoes what was written.
pub fn add_worktree(main_dir: &Path, branch: &BranchName) -> Result<PathBuf, GitError> {
    let folder = worktrees_folder(main_dir).map_err(|error| refused(main_dir, error))?;
    let dir = worktree_dir(main_dir, &folder, branch);
    if dir.exists() {
        return Err(refused(main_dir, format!("{} already exists", dir.display())));
    }
    let fail = failed(OPERATION, main_dir);
    let repo = repo::open(main_dir)?;
    let folder = dir.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default();
    let name = free_name(&repo, &folder).ok_or_else(|| refused(main_dir, format!("no free worktree name like {folder}")))?;
    // libgit2 creates only the last folder of the path
    if let Some(parent) = dir.parent() {
        fs::create_dir_all(parent).map_err(|error| refused(main_dir, format!("cannot create {}: {error}", parent.display())))?;
    }
    let created = local_branch(&repo, branch).map_err(&fail)?;
    let place = Place { name, dir };
    match add(&repo, &place, branch.local_name()) {
        Ok(()) => place.dir.canonicalize().map_err(|error| refused(main_dir, format!("cannot read {}: {error}", place.dir.display()))),
        Err(error) => {
            undo(&repo, &place, created);
            Err(fail(error))
        }
    }
}

/// Where a new worktree goes: its folder, and its name under `.git/worktrees`.
struct Place {
    name: String,
    dir: PathBuf,
}

/// Whether [`add_worktree`] made the local branch, so a failure deletes it again.
enum LocalBranch {
    Existing,
    Created(String),
}

fn local_branch(repo: &Repository, branch: &BranchName) -> Result<LocalBranch, git2::Error> {
    let BranchName::Remote { remote, branch: name } = branch else {
        return Ok(LocalBranch::Existing);
    };
    if repo.find_branch(name, BranchType::Local).is_ok() {
        return Ok(LocalBranch::Existing);
    }
    let tip = repo.find_branch(&branch.display(), BranchType::Remote)?.get().peel_to_commit()?;
    let mut local = repo.branch(name, &tip, false)?;
    if let Err(error) = local.set_upstream(Some(&format!("{remote}/{name}"))) {
        // best effort: the error that matters is the upstream one
        let _ = local.delete();
        return Err(error);
    }
    Ok(LocalBranch::Created(name.clone()))
}

fn add(repo: &Repository, place: &Place, branch: &str) -> Result<(), git2::Error> {
    let reference = repo.find_branch(branch, BranchType::Local)?.into_reference();
    let mut options = WorktreeAddOptions::new();
    options.reference(Some(&reference));
    repo.worktree(&place.name, &place.dir, Some(&options)).map(|_| ())
}

/// The folder name, or the first of `-2`, `-3`, … not yet under `.git/worktrees`.
fn free_name(repo: &Repository, folder: &str) -> Option<String> {
    let taken = repo.commondir().join("worktrees");
    (1..=MAX_NAME_TRIES)
        .map(|n| if n == 1 { folder.to_owned() } else { format!("{folder}-{n}") })
        .find(|name| !taken.join(name).exists())
}

/// libgit2 leaves `.git/worktrees/<name>` and a half-made folder behind when it fails midway. The name was free
/// before this run, so its entry is this run's; the folder is only when libgit2 linked it to that entry (or left
/// it empty): a folder that appeared between the check and libgit2's `mkdir` is someone else's.
fn undo(repo: &Repository, place: &Place, created: LocalBranch) {
    let entry = repo.commondir().join("worktrees").join(&place.name);
    if links_to(&place.dir, &entry) {
        remove(&place.dir);
    }
    // only succeeds on an empty folder
    let _ = fs::remove_dir(&place.dir);
    remove(&entry);
    if let LocalBranch::Created(branch) = created {
        delete_branch(repo, &branch);
    }
}

/// Whether `dir/.git` names `entry` as its git dir, as libgit2 writes it for a new worktree.
fn links_to(dir: &Path, entry: &Path) -> bool {
    let Ok(text) = fs::read_to_string(dir.join(".git")) else {
        return false;
    };
    let linked = text.trim_end().strip_prefix("gitdir: ").map(|path| Path::new(path).canonicalize());
    matches!((linked, entry.canonicalize()), (Some(Ok(linked)), Ok(entry)) if linked == entry)
}

fn remove(leftover: &Path) {
    if leftover.exists() && let Err(error) = fs::remove_dir_all(leftover) {
        eprintln!("ive: cannot remove {}: {error}", leftover.display());
    }
}

fn delete_branch(repo: &Repository, branch: &str) {
    let deleted = repo.find_branch(branch, BranchType::Local).and_then(|mut local| local.delete());
    if let Err(error) = deleted {
        eprintln!("ive: cannot delete branch {branch} after a failed worktree: {}", error.message());
    }
}

fn refused(main_dir: &Path, message: String) -> GitError {
    GitError { operation: OPERATION.to_owned(), dir: main_dir.to_path_buf(), message }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::testing::TestRepo;

    fn remote_feature() -> BranchName {
        BranchName::Remote { remote: "origin".to_owned(), branch: "feature/x".to_owned() }
    }

    /// `repo` with remote `origin` whose branch `feature/x` adds feature.txt.
    fn repo_with_origin() -> (TestRepo, TestRepo) {
        let origin = TestRepo::init();
        origin.write("base.txt", "base\n");
        origin.commit("origin root");
        origin.branch("feature/x");
        origin.checkout("feature/x");
        origin.write("feature.txt", "feature\n");
        origin.commit("feature");
        let repo = TestRepo::init();
        repo.write("main.txt", "main\n");
        repo.commit("root");
        repo.fetch_from("origin", &origin);
        (repo, origin)
    }

    #[test]
    fn should_add_worktree_from_local_branch() {
        // given
        let repo = TestRepo::init();
        repo.write("f.txt", "f\n");
        repo.commit("root");
        repo.branch("topic");

        // when
        let dir = add_worktree(repo.path(), &BranchName::Local("topic".to_owned())).expect("worktree");

        // then
        let opened = Repository::open(&dir).expect("open worktree");
        assert_eq!(dir, repo.path().join(".worktree/topic"));
        assert_eq!((opened.head().expect("HEAD").shorthand().ok(), dir.join("f.txt").is_file()), (Some("topic"), true));
    }

    #[test]
    fn should_add_the_worktree_in_the_configured_folder() {
        // given
        let repo = TestRepo::init();
        repo.write("f.txt", "f\n");
        repo.commit("root");
        repo.branch("feature/x");
        fs::write(repo.path().join(".ive.toml"), "worktrees = \"wt\"\n").expect(".ive.toml");

        // when
        let dir = add_worktree(repo.path(), &BranchName::Local("feature/x".to_owned())).expect("worktree");

        // then
        assert_eq!(dir, repo.path().join("wt/feature-x"));
    }

    #[test]
    fn should_refuse_a_folder_outside_the_checkout() {
        // given
        let repo = TestRepo::init();
        repo.write("f.txt", "f\n");
        repo.commit("root");
        repo.branch("topic");
        fs::write(repo.path().join(".ive.toml"), "worktrees = \"../away\"\n").expect(".ive.toml");

        // when
        let result = add_worktree(repo.path(), &BranchName::Local("topic".to_owned()));

        // then
        let message = result.expect_err("refused").message;
        assert!(message.contains("\"../away\""), "{message}");
        assert_eq!(repo.repo.worktrees().expect("worktrees").len(), 0);
    }

    #[test]
    fn should_create_tracking_branch_for_remote_branch() {
        // given
        let (repo, _origin) = repo_with_origin();

        // when
        let dir = add_worktree(repo.path(), &remote_feature()).expect("worktree");

        // then
        let local = repo.repo.find_branch("feature/x", BranchType::Local).expect("local branch");
        let upstream = local.upstream().expect("upstream");
        assert_eq!(dir, repo.path().join(".worktree/feature-x"));
        assert_eq!((upstream.name().ok().flatten(), dir.join("feature.txt").is_file()), (Some("origin/feature/x"), true));
    }

    #[test]
    fn should_use_existing_local_branch_when_remote_one_is_picked() {
        // given
        let (repo, _origin) = repo_with_origin();
        repo.branch("feature/x");
        let local_tip = repo.repo.find_branch("feature/x", BranchType::Local).expect("local").get().target();

        // when
        let dir = add_worktree(repo.path(), &remote_feature()).expect("worktree");

        // then
        let head = Repository::open(&dir).expect("open").head().expect("HEAD").target();
        assert_eq!(head, local_tip);
    }

    #[test]
    fn should_write_nothing_when_the_folder_exists() {
        // given
        let (repo, _origin) = repo_with_origin();
        fs::create_dir_all(repo.path().join(".worktree/feature-x")).expect("folder");

        // when
        let result = add_worktree(repo.path(), &remote_feature());

        // then
        let message = result.expect_err("refused").message;
        assert!(message.ends_with("feature-x already exists"), "{message}");
        assert!(repo.repo.find_branch("feature/x", BranchType::Local).is_err());
        assert_eq!(repo.repo.worktrees().expect("worktrees").len(), 0);
    }

    #[test]
    fn should_pick_next_name_when_git_already_has_one() {
        // given
        let repo = TestRepo::init();
        repo.write("f.txt", "f\n");
        repo.commit("root");
        repo.branch("topic");
        fs::create_dir_all(repo.path().join(".git/worktrees/topic")).expect("stale entry");

        // when
        let dir = add_worktree(repo.path(), &BranchName::Local("topic".to_owned())).expect("worktree");

        // then
        let worktree = repo.repo.find_worktree("topic-2").expect("named topic-2");
        assert_eq!(worktree.path().canonicalize().expect("canonical"), dir);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn should_remove_leftovers_and_created_branch_when_adding_fails() {
        // given
        use std::os::unix::fs::PermissionsExt;
        if unsafe { libc::geteuid() } == 0 {
            return; // root ignores the read-only folder this test relies on
        }
        let (repo, _origin) = repo_with_origin();
        let parent = repo.path().join(".worktree");
        fs::create_dir_all(&parent).expect("parent");
        fs::set_permissions(&parent, fs::Permissions::from_mode(0o555)).expect("read-only");

        // when
        let result = add_worktree(repo.path(), &remote_feature());

        // then
        fs::set_permissions(&parent, fs::Permissions::from_mode(0o755)).expect("writable again");
        assert!(result.is_err());
        assert!(!repo.path().join(".git/worktrees/feature-x").exists());
        assert!(repo.repo.find_branch("feature/x", BranchType::Local).is_err());
    }

    #[test]
    fn should_remove_the_half_made_folder_when_the_checkout_fails() {
        // given
        let repo = TestRepo::init();
        repo.write("f.txt", "f\n");
        repo.commit("root");
        repo.branch("broken");
        repo.checkout("broken");
        repo.write("lost.txt", "its blob goes missing\n");
        let tip = repo.commit("lost blob");
        repo.checkout("main");
        let blob = repo.repo.find_commit(tip).and_then(|commit| commit.tree()).expect("tree").get_name("lost.txt").expect("entry").id();
        repo.remove_object(blob);

        // when
        let result = add_worktree(repo.path(), &BranchName::Local("broken".to_owned()));

        // then
        assert!(result.is_err());
        assert!(!repo.path().join(".worktree/broken").exists());
        assert!(!repo.path().join(".git/worktrees/broken").exists());
    }

    #[test]
    fn should_keep_a_folder_this_run_did_not_make_when_cleaning_up() {
        // given
        let repo = TestRepo::init();
        repo.write("f.txt", "f\n");
        repo.commit("root");
        let place = Place { name: "topic".to_owned(), dir: repo.path().join(".worktree/topic") };
        fs::create_dir_all(&place.dir).expect("folder");
        fs::write(place.dir.join("notes.txt"), "someone else's\n").expect("file");

        // when
        undo(&repo.repo, &place, LocalBranch::Existing);

        // then
        assert!(place.dir.join("notes.txt").is_file());
    }
}
