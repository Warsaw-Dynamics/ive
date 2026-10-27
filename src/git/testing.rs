//! Temp repositories built with git2 itself, so tests need no `git` command.

use std::cell::Cell;
use std::fs;
use std::path::{Path, PathBuf};

use git2::build::CheckoutBuilder;
use git2::{BranchType, Commit, IndexAddOption, ObjectType, Oid, Repository, RepositoryInitOptions, Signature, Time, WorktreeAddOptions};

/// Commits made through [`TestRepo`]'s methods are one minute apart from here on, so logs sorted by time are
/// deterministic.
const FIRST_COMMIT_TIME: i64 = 1_700_000_000;

/// A repository in a temp folder whose HEAD names `main`, whatever `init.defaultBranch` says.
pub struct TestRepo {
    _dir: tempfile::TempDir,
    pub root: PathBuf,
    pub repo: Repository,
    clock: Cell<i64>,
}

impl TestRepo {
    pub fn init() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let root = dunce::canonicalize(dir.path()).unwrap().join("repo");
        let mut options = RepositoryInitOptions::new();
        options.initial_head("main");
        let repo = Repository::init_opts(&root, &options).unwrap();
        // a user's global `core.autocrlf=true` (common on Windows) would make HEAD's checked-out text CRLF
        repo.config().unwrap().set_bool("core.autocrlf", false).unwrap();
        Self { _dir: dir, root, repo, clock: Cell::new(FIRST_COMMIT_TIME) }
    }

    /// A folder next to the repository, for worktrees and plain folders.
    pub fn sibling(&self, name: &str) -> PathBuf {
        self.root.parent().unwrap().join(name)
    }

    pub fn path(&self) -> &Path {
        &self.root
    }

    pub fn write(&self, path: &str, content: &str) {
        self.write_bytes(Path::new(path), content.as_bytes());
    }

    pub fn write_bytes(&self, path: &Path, bytes: &[u8]) {
        let full = self.root.join(path);
        fs::create_dir_all(full.parent().unwrap()).unwrap();
        fs::write(full, bytes).unwrap();
    }

    pub fn remove(&self, path: &str) {
        fs::remove_file(self.root.join(path)).unwrap();
    }

    pub fn rename(&self, from: &str, to: &str) {
        fs::rename(self.root.join(from), self.root.join(to)).unwrap();
    }

    /// Stages one path, as `git add <path>`.
    pub fn stage(&self, path: &str) {
        let mut index = self.repo.index().unwrap();
        index.add_path(Path::new(path)).unwrap();
        index.write().unwrap();
    }

    /// Stages additions, modifications and deletions, as `git add -A`.
    pub fn stage_all(&self) {
        let mut index = self.repo.index().unwrap();
        index.add_all(["*"], IndexAddOption::DEFAULT, None).unwrap();
        index.update_all(["*"], None).unwrap();
        index.write().unwrap();
    }

    /// Commits the whole work tree on the current branch.
    pub fn commit(&self, message: &str) -> Oid {
        self.stage_all();
        let head = self.head_commit();
        let parents: Vec<&Commit<'_>> = head.iter().collect();
        self.commit_with_parents(message, &parents)
    }

    /// Commits the whole work tree with `other` as second parent, as a merge does.
    pub fn merge(&self, message: &str, other: Oid) -> Oid {
        self.stage_all();
        let head = self.head_commit().unwrap();
        let other = self.repo.find_commit(other).unwrap();
        self.commit_with_parents(message, &[&head, &other])
    }

    /// A branch at HEAD; HEAD stays where it is.
    pub fn branch(&self, name: &str) {
        let head = self.head_commit().unwrap();
        self.repo.branch(name, &head, false).unwrap();
    }

    /// Switches HEAD and the work tree to local branch `name`.
    pub fn checkout(&self, name: &str) {
        self.repo.set_head(&format!("refs/heads/{name}")).unwrap();
        self.repo.checkout_head(Some(CheckoutBuilder::new().force())).unwrap();
    }

    /// An annotated tag at HEAD.
    pub fn tag(&self, name: &str) {
        let head = self.repo.head().and_then(|head| head.peel(ObjectType::Commit)).unwrap();
        self.repo.tag(name, &head, &self.signature(), name, false).unwrap();
    }

    /// Merges local branch `name` into the work tree and index without committing, as `git merge --no-commit`
    /// does; clashing changes leave conflicts.
    // its one test needs a symlink, so it runs on Unix only
    #[cfg_attr(windows, allow(dead_code))]
    pub fn merge_into_work_tree(&self, name: &str) {
        let tip = self.repo.find_branch(name, BranchType::Local).unwrap().into_reference();
        let annotated = self.repo.reference_to_annotated_commit(&tip).unwrap();
        self.repo.merge(&[&annotated], None, None).unwrap();
    }

    /// Deletes a loose object, as a damaged or partial clone lacks it.
    pub fn remove_object(&self, oid: Oid) {
        let hex = oid.to_string();
        fs::remove_file(self.repo.path().join("objects").join(&hex[..2]).join(&hex[2..])).unwrap();
    }

    pub fn stash(&mut self, message: &str) {
        let signature = self.signature();
        self.repo.stash_save(&signature, message, None).unwrap();
    }

    /// Adds `other` as remote `name` and fetches its branches into `refs/remotes/<name>/*`.
    pub fn fetch_from(&self, name: &str, other: &TestRepo) {
        let url = other.path().to_str().unwrap();
        let mut remote = self.repo.remote(name, url).unwrap();
        // libgit2's local transport works without the network features IVE leaves out
        remote.fetch(&[format!("refs/heads/*:refs/remotes/{name}/*")], None, None).unwrap();
    }

    fn commit_with_parents(&self, message: &str, parents: &[&Commit<'_>]) -> Oid {
        let mut index = self.repo.index().unwrap();
        let tree = self.repo.find_tree(index.write_tree().unwrap()).unwrap();
        let signature = self.signature();
        self.repo.commit(Some("HEAD"), &signature, &signature, message, &tree, parents).unwrap()
    }

    fn signature(&self) -> Signature<'static> {
        let time = self.clock.get();
        self.clock.set(time + 60);
        Signature::new("Test Author", "test@example.com", &Time::new(time, 0)).unwrap()
    }

    fn head_commit(&self) -> Option<Commit<'_>> {
        self.repo.head().ok()?.peel_to_commit().ok()
    }
}

/// Writes `file` in `repo`'s work tree and commits it on HEAD.
pub fn commit(repo: &Repository, file: &str, content: &str) -> Oid {
    write(repo, file, content);
    let mut index = repo.index().unwrap();
    index.add_path(Path::new(file)).unwrap();
    index.write().unwrap();
    let tree = repo.find_tree(index.write_tree().unwrap()).unwrap();
    let signature = Signature::now("IVE test", "ive@example.com").unwrap();
    let parent = repo.head().ok().and_then(|head| head.peel_to_commit().ok());
    let parents: Vec<&Commit> = parent.iter().collect();
    repo.commit(Some("HEAD"), &signature, &signature, &format!("change {file}"), &tree, &parents).unwrap()
}

/// Writes `file` in `repo`'s work tree without staging it.
pub fn write(repo: &Repository, file: &str, content: &str) {
    let path = repo.workdir().unwrap().join(file);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

/// Adds a worktree at `path` on a new branch `branch` from HEAD, and opens it.
pub fn add_worktree(repo: &Repository, branch: &str, path: &Path) -> Repository {
    let head = repo.head().unwrap().peel_to_commit().unwrap();
    let reference = repo.branch(branch, &head, false).unwrap().into_reference();
    let mut options = WorktreeAddOptions::new();
    options.reference(Some(&reference));
    let name = path.file_name().unwrap().to_str().unwrap();
    repo.worktree(name, path, Some(&options)).unwrap();
    Repository::open(path).unwrap()
}
