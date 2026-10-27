use std::collections::BTreeSet;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::git::GitError;
use crate::git::worktrees;

/// Longest id; leaves room in a folder name for part 3's scratches and a `-NN` suffix.
const MAX_ID_LEN: usize = 64;
const MAX_SLUG_BASE_LEN: usize = 56;

/// Stable id of a project, kept in `state.toml`; part 3 names the project's scratch folder after it.
/// Checked again when read back (the hub's state goes through scripts): `parse`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct ProjectId(String);

impl TryFrom<String> for ProjectId {
    type Error = String;

    fn try_from(value: String) -> Result<Self, String> {
        Self::parse(&value)
    }
}

impl From<ProjectId> for String {
    fn from(id: ProjectId) -> String {
        id.0
    }
}

impl ProjectId {
    /// Checks an id read from `state.toml`: 1–64 of `[a-z0-9._-]`, not only dots, since it becomes a folder name.
    pub fn parse(value: &str) -> Result<Self, String> {
        let allowed = value.chars().all(is_slug_char);
        let only_dots = value.chars().all(|c| c == '.');
        if value.is_empty() || value.len() > MAX_ID_LEN || only_dots || !allowed {
            return Err(format!("invalid project id {value:?}: expected 1-{MAX_ID_LEN} of a-z 0-9 . _ -, not only dots"));
        }
        Ok(Self(value.to_owned()))
    }

    /// A new id for a project in folder `name`, unused by `taken`: `atlassian`, then `atlassian-2`, `atlassian-3`, …
    pub fn slug(name: &str, taken: &[&ProjectId]) -> Self {
        let base: String = name
            .to_lowercase()
            .chars()
            .map(|c| if is_slug_char(c) { c } else { '-' })
            .take(MAX_SLUG_BASE_LEN)
            .collect();
        let base = if base.chars().all(|c| c == '.') { "project".to_owned() } else { base };
        let is_taken = |candidate: &str| taken.iter().any(|id| id.as_str() == candidate);
        let free = std::iter::once(base.clone())
            .chain((2..).map(|n| format!("{base}-{n}")))
            .find(|candidate| !is_taken(candidate))
            .expect("an unbounded sequence of candidates has a free one");
        Self(free)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// For tests: an id that must be valid.
    #[cfg(test)]
    pub fn new(slug: impl Into<String>) -> Self {
        let slug = slug.into();
        Self::parse(&slug).unwrap_or_else(|reason| panic!("{reason}"))
    }
}

impl fmt::Display for ProjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

fn is_slug_char(c: char) -> bool {
    c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '.' | '_' | '-')
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectKind {
    Git,
    Plain,
}

/// A folder the user opened: a git repository (identified by its main checkout) or a plain folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    pub id: ProjectId,
    pub name: String,
    /// main checkout, or the plain folder; canonical
    pub root: PathBuf,
    pub kind: ProjectKind,
    /// never empty; the main checkout first
    pub checkouts: Vec<Checkout>,
    /// why the last refresh failed; the checkouts are then the last ones read
    pub refresh_error: Option<String>,
    /// the repository's common git dir (`<main checkout>/.git`), watched for branch and worktree changes
    pub git_dir: Option<PathBuf>,
}

/// A working directory of a project; its identity is its canonical path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Checkout {
    pub path: PathBuf,
    /// the branch for the main checkout (`dev`, `HEAD detached at 1a2b3c4`), the folder name for others
    pub label: String,
    /// `None` when HEAD is detached; an unborn HEAD has the branch it names
    pub branch: Option<String>,
    /// full hex sha of HEAD; `None` before the first commit
    pub head: Option<String>,
    pub is_main: bool,
    /// `None` for a plain project
    pub status: Option<CheckoutStatus>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CheckoutStatus {
    /// work tree and index entries that differ from HEAD, untracked files included
    pub changes: usize,
    pub ahead: u32,
    pub behind: u32,
    /// `origin/dev` for the main checkout, `dev` for a worktree; `None` without one
    pub base: Option<String>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, serde::Serialize, serde::Deserialize)]
pub struct CheckoutRef {
    pub project: ProjectId,
    pub path: PathBuf,
}

/// What reading a folder found; becomes a [`Project`] once it has an id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectScan {
    pub root: PathBuf,
    pub kind: ProjectKind,
    /// the main checkout first; an empty list is read as a plain project
    pub checkouts: Vec<Checkout>,
    /// set when git could not be read; the folder is then shown as plain
    pub error: Option<String>,
    pub git_dir: Option<PathBuf>,
}

impl ProjectScan {
    /// A folder outside any repository: one checkout, the folder itself.
    pub fn plain(root: PathBuf) -> Self {
        let checkout = Checkout {
            label: folder_name(&root),
            path: root.clone(),
            branch: None,
            head: None,
            is_main: true,
            status: None,
        };
        Self { root, kind: ProjectKind::Plain, checkouts: vec![checkout], error: None, git_dir: None }
    }
}

impl Project {
    pub fn new(id: ProjectId, scan: ProjectScan) -> Self {
        let mut project = Self {
            id,
            name: folder_name(&scan.root),
            root: scan.root.clone(),
            kind: ProjectKind::Plain,
            checkouts: Vec::new(),
            refresh_error: None,
            git_dir: None,
        };
        project.apply(scan);
        project
    }

    /// Replaces root, kind and checkouts with what a refresh read, in one step; the root follows the folder into
    /// a repository it became part of, and a checkout whose status was not read keeps its last one.
    pub fn apply(&mut self, scan: ProjectScan) {
        let mut scan = if scan.checkouts.is_empty() { ProjectScan { error: scan.error, ..ProjectScan::plain(scan.root) } } else { scan };
        for checkout in scan.checkouts.iter_mut().filter(|checkout| checkout.status.is_none()) {
            checkout.status = self.checkout(&checkout.path).and_then(|last| last.status.clone());
        }
        self.name = folder_name(&scan.root);
        self.root = scan.root;
        self.kind = scan.kind;
        self.checkouts = scan.checkouts;
        self.refresh_error = scan.error;
        self.git_dir = scan.git_dir;
    }

    pub fn main_checkout(&self) -> &Checkout {
        &self.checkouts[0]
    }

    pub fn checkout(&self, path: &Path) -> Option<&Checkout> {
        self.checkouts.iter().find(|checkout| checkout.path == path)
    }

    /// The checkout whose folder holds `path`; with nested checkouts (worktrees under `.claude/worktrees`), the
    /// deepest one.
    pub fn checkout_containing(&self, path: &Path) -> Option<&Checkout> {
        self.checkouts
            .iter()
            .filter(|checkout| path.starts_with(&checkout.path))
            .max_by_key(|checkout| checkout.path.components().count())
    }

    pub fn checkout_ref(&self, checkout: &Checkout) -> CheckoutRef {
        CheckoutRef { project: self.id.clone(), path: checkout.path.clone() }
    }

    /// How click targets name `checkout`: its label, or `<label>@<path>` while another checkout of the project
    /// has the same label (a worktree folder named like the main checkout's branch).
    pub fn target_label(&self, checkout: &Checkout) -> String {
        let shared = self.checkouts.iter().filter(|other| other.label == checkout.label).count() > 1;
        if shared {
            return format!("{}@{}", checkout.label, checkout.path.display());
        }
        checkout.label.clone()
    }
}

/// Whose status a refresh reads; checkouts and branches are always listed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statuses {
    All,
    /// these checkout folders only
    Only(BTreeSet<PathBuf>),
}

impl Statuses {
    pub fn includes(&self, checkout: &Path) -> bool {
        match self {
            Self::All => true,
            Self::Only(paths) => paths.contains(checkout),
        }
    }

    /// What two refreshes read together.
    pub fn union(self, other: Self) -> Self {
        match (self, other) {
            (Self::Only(mut paths), Self::Only(more)) => {
                paths.extend(more);
                Self::Only(paths)
            }
            _ => Self::All,
        }
    }
}

/// Reads `folder` as a project: the repository it is in (any worktree or subfolder of it), or the plain folder,
/// with the status of the checkouts `statuses` names, read at or after `since`. Reads git; run it on the background
/// executor.
#[tracing::instrument(level = "debug", name = "scan_project", skip_all, fields(folder = %folder.display()))]
pub fn scan(folder: &Path, statuses: &Statuses, since: Instant) -> Result<ProjectScan, ProjectError> {
    let folder = dunce::canonicalize(folder).map_err(|source| ProjectError::Unreadable { path: folder.to_path_buf(), source })?;
    if folder.is_file() {
        return Err(ProjectError::NotADirectory(folder));
    }
    let main = match worktrees::main_checkout_of(&folder) {
        Ok(Some(main)) => main,
        Ok(None) => return Ok(ProjectScan::plain(folder)),
        Err(error) => return Err(ProjectError::Git(error)),
    };
    let checkouts = worktrees::read_checkouts(&main, statuses, since).map_err(ProjectError::Git)?;
    let git_dir = worktrees::common_dir(&main).map_err(ProjectError::Git)?;
    Ok(ProjectScan { root: main, kind: ProjectKind::Git, checkouts, error: None, git_dir: Some(git_dir) })
}

/// Reads a folder the user opens: as [`scan`], but a folder whose git cannot be read opens as a plain project
/// with the error on its row. A refresh uses `scan`, so a failing read keeps the last checkouts.
pub fn scan_opened(folder: &Path) -> Result<ProjectScan, ProjectError> {
    match scan(folder, &Statuses::All, Instant::now()) {
        Err(ProjectError::Git(error)) => {
            let folder = dunce::canonicalize(folder).map_err(|source| ProjectError::Unreadable { path: folder.to_path_buf(), source })?;
            Ok(ProjectScan { error: Some(error.to_string()), ..ProjectScan::plain(folder) })
        }
        scanned => scanned,
    }
}

/// The last component of `path`, or the whole path for `/`.
pub fn folder_name(path: &Path) -> String {
    path.file_name().map_or_else(|| path.display().to_string(), |name| name.to_string_lossy().into_owned())
}

#[derive(Debug)]
pub enum ProjectError {
    NotADirectory(PathBuf),
    Unreadable { path: PathBuf, source: io::Error },
    PickerFailed(String),
    Git(GitError),
    /// the limit of open projects
    TooMany(usize),
    /// `state.toml` holds paths as UTF-8 text
    NotUtf8(PathBuf),
}

impl fmt::Display for ProjectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotADirectory(path) => write!(f, "not a folder: {}", path.display()),
            Self::Unreadable { path, source } => write!(f, "cannot read {}: {source}", path.display()),
            Self::PickerFailed(reason) => write!(f, "folder picker failed: {reason}"),
            Self::Git(error) => error.fmt(f),
            Self::TooMany(limit) => write!(f, "cannot open another project: {limit} are open, the limit; close one first"),
            Self::NotUtf8(path) => write!(f, "cannot open {}: IVE remembers projects by UTF-8 paths, and this one is not", path.display()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::testing::{TestRepo, add_worktree, commit};

    fn checkout(path: &str, is_main: bool) -> Checkout {
        Checkout { path: PathBuf::from(path), label: folder_name(Path::new(path)), branch: None, head: None, is_main, status: None }
    }

    fn project(checkouts: Vec<Checkout>) -> Project {
        let root = checkouts[0].path.clone();
        Project::new(ProjectId::new("p"), ProjectScan { root, kind: ProjectKind::Git, checkouts, error: None, git_dir: None })
    }

    #[test]
    fn should_pick_deepest_checkout_containing_path() {
        // given
        let project = project(vec![checkout("/r", true), checkout("/r/.claude/worktrees/x", false)]);

        // when
        let found = ["/r/src/a.rs", "/r/.claude/worktrees/x/src/a.rs", "/other/a.rs"]
            .map(|path| project.checkout_containing(Path::new(path)).map(|checkout| checkout.path.clone()));

        // then
        assert_eq!(found, [Some(PathBuf::from("/r")), Some(PathBuf::from("/r/.claude/worktrees/x")), None]);
    }

    #[test]
    fn should_keep_the_last_status_of_checkouts_whose_status_was_not_read() {
        // given
        let status = CheckoutStatus { changes: 3, ahead: 1, behind: 0, base: Some("main".to_owned()) };
        let mut read = vec![checkout("/r", true), checkout("/r/wt", false)];
        read[1].status = Some(status.clone());
        let mut project = project(read);

        // when
        project.apply(ProjectScan { root: PathBuf::from("/r"), kind: ProjectKind::Git, checkouts: vec![checkout("/r", true), checkout("/r/wt", false)], error: None, git_dir: None });

        // then
        assert_eq!(project.checkouts[1].status, Some(status));
    }

    #[test]
    fn should_follow_a_plain_folder_into_the_repository_it_became_part_of() {
        // given
        let mut project = Project::new(ProjectId::new("notes"), ProjectScan::plain(PathBuf::from("/a/notes")));

        // when
        project.apply(ProjectScan { root: PathBuf::from("/a"), kind: ProjectKind::Git, checkouts: vec![checkout("/a", true)], error: None, git_dir: None });

        // then
        let read = (project.root.as_path(), project.name.as_str(), project.main_checkout().path.as_path());
        assert_eq!(read, (Path::new("/a"), "a", Path::new("/a")));
    }

    #[test]
    fn should_name_checkouts_for_click_targets_by_label_and_by_path_when_labels_clash() {
        // given
        let mut main = checkout("/r", true);
        main.label = "dev".to_owned();
        let project = project(vec![main, checkout("/r/wt/dev", false), checkout("/r/wt/x", false)]);

        // when
        let names: Vec<String> = project.checkouts.iter().map(|checkout| project.target_label(checkout)).collect();

        // then
        assert_eq!(names, ["dev@/r", "dev@/r/wt/dev", "x"]);
    }

    #[test]
    fn should_slug_folder_name_and_number_clashes() {
        // given
        let taken = [ProjectId::new("app"), ProjectId::new("app-2")];
        let taken: Vec<&ProjectId> = taken.iter().collect();

        // when
        let ids = ["App", "My Notes", "żółw_app", "..", "atlassian"].map(|name| ProjectId::slug(name, &taken).0);

        // then
        assert_eq!(ids, ["app-3", "my-notes", "---w_app", "project", "atlassian"]);
    }

    #[test]
    fn should_reject_ids_that_are_no_safe_folder_name() {
        // given
        let values = ["", "..", "a/b", "Upper", &"x".repeat(65), "ok-1.2_3"];

        // when
        let valid = values.map(|value| ProjectId::parse(value).is_ok());

        // then
        assert_eq!(valid, [false, false, false, false, false, true]);
    }

    #[test]
    fn should_open_main_checkout_project_from_subfolder_of_worktree() {
        // given
        let test = TestRepo::init();
        commit(&test.repo, "README.md", "hello");
        let worktree = test.sibling("feature");
        add_worktree(&test.repo, "feature", &worktree);
        std::fs::create_dir_all(worktree.join("src")).unwrap();

        // when
        let scan = scan(&worktree.join("src"), &Statuses::All, Instant::now()).unwrap();

        // then
        assert_eq!((scan.root, scan.kind), (test.root.clone(), ProjectKind::Git));
        assert_eq!(scan.checkouts.iter().map(|checkout| checkout.label.as_str()).collect::<Vec<_>>(), ["main", "feature"]);
    }

    #[test]
    fn should_open_folder_outside_repositories_as_plain_project() {
        // given
        let dir = tempfile::tempdir().unwrap();

        // when
        let scan = scan(dir.path(), &Statuses::All, Instant::now()).unwrap();

        // then
        assert_eq!(scan.kind, ProjectKind::Plain);
        assert_eq!(scan.checkouts.len(), 1);
        assert_eq!(scan.checkouts[0].status, None);
    }

    #[test]
    fn should_reject_missing_folder_with_its_path_in_message() {
        // given
        let missing = Path::new("/nonexistent/ive-project");

        // when
        let error = scan(missing, &Statuses::All, Instant::now()).err().unwrap().to_string();

        // then
        assert!(error.starts_with("cannot read /nonexistent/ive-project:"), "{error}");
    }

    #[test]
    fn should_fail_a_scan_when_git_cannot_be_read_but_open_the_folder_as_plain() {
        // given
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(".git"), "not a git file").unwrap();

        // when
        let refreshed = scan(dir.path(), &Statuses::All, Instant::now());
        let opened = scan_opened(dir.path()).unwrap();

        // then
        assert!(matches!(refreshed, Err(ProjectError::Git(_))), "{refreshed:?}");
        assert_eq!(opened.kind, ProjectKind::Plain);
        assert!(opened.error.is_some_and(|error| error.starts_with("cannot find the repository of ")), "{:?}", scan_opened(dir.path()));
    }
}
