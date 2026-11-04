//! Where a scheduled command runs: the checkout with its branch, or a worktree IVE makes first. Reads and writes
//! git, so it runs off the UI thread.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use git2::{BranchType, Repository};

use super::Target;
use crate::git::refs::BranchName;
use crate::git::worktree_add::{add_worktree_at, add_worktree_from, worktree_dir};
use crate::ive_toml::worktrees_folder;

const ORIGIN: &str = "origin";
/// A fetch that takes longer is stuck (a password prompt nobody sees, a dead network).
const FETCH_TIMEOUT: Duration = Duration::from_secs(120);
const FETCH_POLL: Duration = Duration::from_millis(100);

/// What starting a command takes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Plan {
    /// the checkout with the branch
    In { path: PathBuf, main: bool },
    /// the branch exists (local, or on `origin` only) without a worktree: one in `dir`
    AddWorktree { branch: BranchName, dir: PathBuf },
    /// no such branch: fetch `source`, start the branch there, then its worktree in `dir`
    NewBranch { source: String, dir: PathBuf },
    Cannot(String),
}

impl Plan {
    /// One line for the tab: where it runs, or what Run does first, or why it cannot.
    pub fn text(&self, branch: &str) -> String {
        let shown = |path: &Path| crate::paths::tilde_home(path);
        match self {
            Self::In { path, main: true } => format!("in {} (main checkout)", shown(path)),
            Self::In { path, main: false } => format!("in {}", shown(path)),
            Self::AddWorktree { branch: BranchName::Remote { remote, .. }, dir } => {
                format!("Run first: a local {branch} tracking {remote}/{branch}, then new worktree {}", shown(dir))
            }
            Self::AddWorktree { dir, .. } => format!("Run first: new worktree {}", shown(dir)),
            Self::NewBranch { source, dir } => {
                format!("Run first: fetch {ORIGIN}, then new branch {branch} from {ORIGIN}/{source}, then new worktree {}", shown(dir))
            }
            Self::Cannot(reason) => reason.clone(),
        }
    }

    pub fn is_runnable(&self) -> bool {
        !matches!(self, Self::Cannot(_))
    }
}

/// What starting `target` takes, read from the repository now.
pub fn plan(target: &Target) -> Plan {
    let main = &target.project;
    if !main.is_dir() {
        return Plan::Cannot(format!("The project {} is gone", main.display()));
    }
    let Ok(repo) = Repository::open(main) else {
        return Plan::Cannot(format!("The project {} is no longer a git repository", main.display()));
    };
    if let Some((path, is_main)) = checkout_with(&repo, &target.branch) {
        return Plan::In { path, main: is_main };
    }
    let dir = match new_worktree_dir(target) {
        Ok(dir) => dir,
        Err(reason) => return Plan::Cannot(reason),
    };
    if repo.find_branch(&target.branch, BranchType::Local).is_ok() {
        return Plan::AddWorktree { branch: BranchName::Local(target.branch.clone()), dir };
    }
    if repo.find_branch(&format!("{ORIGIN}/{}", target.branch), BranchType::Remote).is_ok() {
        return Plan::AddWorktree { branch: BranchName::Remote { remote: ORIGIN.to_owned(), branch: target.branch.clone() }, dir };
    }
    match &target.source {
        Some(source) => Plan::NewBranch { source: source.clone(), dir },
        None => Plan::Cannot(format!("No branch {} and no source_branch to start it from", target.branch)),
    }
}

/// Does what `plan` says and returns the checkout's folder.
pub fn prepare(plan: &Plan, target: &Target) -> Result<PathBuf, String> {
    let main = &target.project;
    match plan {
        Plan::In { path, .. } => Ok(path.clone()),
        Plan::AddWorktree { branch, dir } => add_worktree_at(main, branch, dir).map_err(|error| error.to_string()),
        Plan::NewBranch { source, dir } => {
            // a failed fetch is no reason to stop while the source is known: its last fetched tip will do
            let fetched = fetch(main, source);
            let repo = Repository::open(main).map_err(|error| format!("cannot open {}: {}", main.display(), error.message()))?;
            let start = start_of(&repo, source).map_err(|missing| match fetched {
                Err(error) => format!("{missing}; {error}"),
                Ok(()) => missing,
            })?;
            add_worktree_from(main, &target.branch, start, dir).map_err(|error| error.to_string())
        }
        Plan::Cannot(reason) => Err(reason.clone()),
    }
}

/// The checkout of `repo` that has `branch`, and whether it is the main one.
fn checkout_with(repo: &Repository, branch: &str) -> Option<(PathBuf, bool)> {
    let on = |repo: &Repository| repo.head().ok().and_then(|head| head.shorthand().ok().map(str::to_owned)).as_deref() == Some(branch);
    if on(repo) {
        return repo.workdir().map(|dir| (dir.to_path_buf(), true));
    }
    let names = repo.worktrees().ok()?;
    names.iter().flatten().flatten().filter_map(|name| repo.find_worktree(name).ok()).find_map(|worktree| {
        let opened = Repository::open_from_worktree(&worktree).ok()?;
        on(&opened).then(|| (worktree.path().to_path_buf(), false))
    })
}

/// `worktree` (relative to the main checkout), else the `.ive.toml` folder's `<branch, / as ->`.
fn new_worktree_dir(target: &Target) -> Result<PathBuf, String> {
    if let Some(dir) = &target.worktree {
        return Ok(target.project.join(dir));
    }
    let folder = worktrees_folder(&target.project)?;
    Ok(worktree_dir(&target.project, &folder, &BranchName::Local(target.branch.clone())))
}

/// `origin/<source>`, else the local `<source>`.
fn start_of(repo: &Repository, source: &str) -> Result<git2::Oid, String> {
    let remote = repo.find_branch(&format!("{ORIGIN}/{source}"), BranchType::Remote);
    let branch = remote.or_else(|_| repo.find_branch(source, BranchType::Local));
    branch
        .ok()
        .and_then(|branch| branch.get().peel_to_commit().ok())
        .map(|commit| commit.id())
        .ok_or_else(|| format!("source branch {source} is found neither on {ORIGIN} nor locally"))
}

/// `git fetch origin <source>` with the git CLI, so the user's credentials apply; at most [`FETCH_TIMEOUT`].
fn fetch(main: &Path, source: &str) -> Result<(), String> {
    let mut child = Command::new("git")
        .arg("-C")
        .arg(main)
        .args(["fetch", "--quiet", ORIGIN, source])
        // no prompt nobody would see
        .env("GIT_TERMINAL_PROMPT", "0")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("cannot run git fetch: {error}"))?;
    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) if status.success() => return Ok(()),
            Ok(Some(status)) => return Err(format!("git fetch {ORIGIN} {source} failed ({status}): {}", stderr_of(child))),
            Ok(None) if started.elapsed() > FETCH_TIMEOUT => {
                let _ = child.kill();
                return Err(format!("git fetch {ORIGIN} {source} took over {} s", FETCH_TIMEOUT.as_secs()));
            }
            Ok(None) => std::thread::sleep(FETCH_POLL),
            Err(error) => return Err(format!("git fetch {ORIGIN} {source}: {error}")),
        }
    }
}

fn stderr_of(child: std::process::Child) -> String {
    child.wait_with_output().map(|output| String::from_utf8_lossy(&output.stderr).trim().to_owned()).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::testing::TestRepo;

    fn target(repo: &TestRepo, branch: &str, source: Option<&str>) -> Target {
        Target { project: repo.path().to_path_buf(), branch: branch.to_owned(), worktree: None, source: source.map(str::to_owned) }
    }

    fn repo_on_main() -> TestRepo {
        let repo = TestRepo::init();
        repo.write("f.txt", "f\n");
        repo.commit("root");
        repo
    }

    #[test]
    fn should_run_in_the_main_checkout_on_its_branch() {
        // given
        let repo = repo_on_main();
        let head = Repository::open(repo.path()).unwrap().head().unwrap().shorthand().unwrap().to_owned();
        // when
        let plan = plan(&target(&repo, &head, None));
        // then
        assert!(matches!(plan, Plan::In { main: true, .. }), "{plan:?}");
    }

    #[test]
    fn should_add_a_worktree_for_a_branch_without_one_then_use_it() {
        // given
        let repo = repo_on_main();
        repo.branch("topic");
        let wanted = target(&repo, "topic", None);
        // when
        let first = plan(&wanted);
        let made = prepare(&first, &wanted).unwrap();
        let second = plan(&wanted);
        // then
        assert_eq!(first, Plan::AddWorktree { branch: BranchName::Local("topic".to_owned()), dir: repo.path().join(".worktree/topic") });
        assert_eq!(second, Plan::In { path: made, main: false });
    }

    #[test]
    fn should_start_a_new_branch_from_the_local_source_without_origin() {
        // given
        let repo = repo_on_main();
        repo.branch("dev");
        let wanted = target(&repo, "GEN-900-retry", Some("dev"));
        // when
        let plan = plan(&wanted);
        let made = prepare(&plan, &wanted).unwrap();
        // then
        assert!(matches!(plan, Plan::NewBranch { .. }), "{plan:?}");
        let opened = Repository::open(&made).unwrap();
        assert_eq!(opened.head().unwrap().shorthand().ok(), Some("GEN-900-retry"));
    }

    #[test]
    fn should_say_why_it_cannot_run() {
        // given
        let repo = repo_on_main();
        let gone = Target { project: PathBuf::from("/nowhere/at/all"), branch: "x".to_owned(), worktree: None, source: None };
        // when
        let no_source = plan(&target(&repo, "missing", None));
        let no_project = plan(&gone);
        let missing_source = prepare(&plan(&target(&repo, "missing", Some("nope"))), &target(&repo, "missing", Some("nope")));
        // then
        assert_eq!(no_source, Plan::Cannot("No branch missing and no source_branch to start it from".to_owned()));
        assert_eq!(no_project, Plan::Cannot("The project /nowhere/at/all is gone".to_owned()));
        assert!(missing_source.unwrap_err().contains("source branch nope is found neither"));
    }
}
