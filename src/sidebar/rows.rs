//! The tree's rows from projects, processes, expansion and paging; pure, so it is tested without gpui.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::folders::{ChangeFolderRow, ChangeRow, ChangeSource, CheckoutGit, CommitRow, CommitsMoreRow, FolderKind, FolderRow, GitFolders, NoteRow};
use super::states::{EntryState, entry_mark, entry_state};

use crate::age::Clock;
use crate::file_tree::{Entry, EntryKind, RunState};
use crate::git::status::TreeStatus;
use crate::git::types::{GitPath, Mark, Sha};
use crate::overview::file_list::FileLayout;
use crate::open_project::{OpenProject, PagedList};
use crate::process::{ProcessId, ProcessKind, Processes, RunId};
use crate::terminal::Progress;
use crate::project::{Checkout, CheckoutRef, ProjectId, ProjectKind};
use crate::script::open_folders::OpenFolders;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeRow {
    pub depth: usize,
    pub item: TreeItem,
    /// what the row's click targets are named after (see [`TreeRow::target_names`])
    pub target: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TreeItem {
    Project(ProjectRow),
    Checkout(CheckoutRow),
    /// Processes, Uncommitted, Since base, Commits, Files
    Folder(FolderRow),
    Process(ProcessRow),
    /// a run in Processes, live or exited
    LiveRun(RunRow),
    Change(ChangeRow),
    ChangeFolder(ChangeFolderRow),
    Commit(CommitRow),
    CommitsMore(CommitsMoreRow),
    Note(NoteRow),
    Entry(EntryRow),
    More(MoreRow),
}

/// A run, as [`RunsView`] reports it: on its configuration's row in the tree, a row of the folder overview's list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunRow {
    pub id: RunId,
    /// its configuration's key
    pub key: String,
    pub name: String,
    pub serve: bool,
    pub state: RunBadge,
    /// the full state: `running`, `exited 1`, `before: build failed (exit 1)`
    pub status: String,
    /// what its configuration's row says: `● running · :4200`, `… waiting for Build`, `✕ exit 1`
    pub label: String,
    /// running or waiting, rather than exited
    pub live: bool,
    /// a running static server's page
    pub link: Option<String>,
}

/// A run's state, for its colour and actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunBadge {
    Running,
    Waiting,
    Exited(i32),
    Stopped,
    /// a `before` step failed
    Failed,
}

/// The runs the tree shows, from the workspace; tests make them up.
pub trait RunsView {
    /// The runs started from `folder` in `checkout`, newest first.
    fn runs_in(&self, checkout: &Path, folder: &Path) -> Vec<RunRow>;
    /// Every run of `checkout`, from every folder: the started ones oldest first, then the pending ones.
    fn runs_of(&self, checkout: &Path) -> Vec<RunRow>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckoutRow {
    pub checkout: CheckoutRef,
    pub label: String,
    pub expanded: bool,
    /// the main checkout, which cannot be hidden
    pub is_main: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessRow {
    pub id: ProcessId,
    pub kind: ProcessKind,
    pub title: String,
    pub progress: Progress,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryRow {
    pub checkout: CheckoutRef,
    pub entry: Entry,
    pub expanded: bool,
    /// a broken `.run.toml`'s error, shown as a warning after its name
    pub warning: Option<String>,
    /// a folder with a loaded `.run.toml`
    pub run_file: bool,
    pub state: EntryState,
    /// a file's git status; folders have none
    pub mark: Option<Mark>,
}

/// What file rows are drawn by besides their names: each shown checkout's git status (by checkout path; none
/// until it was read), and the folder names excluded besides gitignored folders; which folders the user left alone
/// are open; and what the git folders list:
/// each checkout's git data (none until read), the Flat/Tree layout, and the clock commit ages count from.
pub struct TreeLook<'a> {
    pub statuses: &'a HashMap<PathBuf, TreeStatus>,
    pub excluded: &'a [String],
    pub open_folders: &'a OpenFolders,
    pub git: &'a HashMap<PathBuf, CheckoutGit>,
    pub layout: FileLayout,
    pub clock: Clock,
}

/// "Show 10 more · N of M" under a paged list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoreRow {
    pub project: ProjectId,
    pub list: PagedList,
    pub total: usize,
    pub shown: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectRow {
    pub id: ProjectId,
    pub name: String,
    pub initials: String,
    pub expanded: bool,
    pub tag: Option<ProjectTag>,
    pub running: usize,
    /// the last refresh failed with this; shown as a warning icon
    pub error: Option<String>,
}

/// What follows the project name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectTag {
    /// the label of a project's only checkout: its branch, or `HEAD detached at <sha>`
    Branch(String),
    NoGit,
    Missing,
}

/// What a row stands for, to keep it selected across rebuilds.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TreeKey {
    Project(ProjectId),
    Checkout(PathBuf),
    Folder(PathBuf, FolderKind),
    Process(ProcessId),
    LiveRun(RunId),
    Change(PathBuf, ChangeSource, GitPath),
    ChangeFolder(PathBuf, ChangeSource, String),
    Commit(PathBuf, Sha),
    CommitsMore(PathBuf),
    Note(PathBuf, Sha),
    Entry(PathBuf),
    More(PagedList),
}

impl TreeRow {
    pub fn key(&self) -> TreeKey {
        match &self.item {
            TreeItem::Project(row) => TreeKey::Project(row.id.clone()),
            TreeItem::Checkout(row) => TreeKey::Checkout(row.checkout.path.clone()),
            TreeItem::Folder(row) => TreeKey::Folder(row.checkout.path.clone(), row.kind),
            TreeItem::Process(row) => TreeKey::Process(row.id),
            TreeItem::LiveRun(row) => TreeKey::LiveRun(row.id),
            TreeItem::Change(row) => TreeKey::Change(row.checkout.path.clone(), row.source.clone(), row.change.path.clone()),
            TreeItem::ChangeFolder(row) => TreeKey::ChangeFolder(row.checkout.path.clone(), row.source.clone(), row.path.clone()),
            TreeItem::Commit(row) => TreeKey::Commit(row.checkout.path.clone(), row.sha.clone()),
            TreeItem::CommitsMore(row) => TreeKey::CommitsMore(row.checkout.path.clone()),
            TreeItem::Note(row) => TreeKey::Note(row.checkout.path.clone(), row.sha.clone()),
            TreeItem::Entry(row) => TreeKey::Entry(row.entry.path.clone()),
            TreeItem::More(row) => TreeKey::More(row.list.clone()),
        }
    }

    /// The names MCP's `click` reaches this row's actions by, whether the row is drawn or not.
    pub fn target_names(&self) -> Vec<String> {
        let verbs: &[&str] = match &self.item {
            TreeItem::Project(row) if row.tag == Some(ProjectTag::Missing) => &["close-project"],
            TreeItem::Project(_) => &["project", "toggle", "close-project"],
            TreeItem::Checkout(row) if row.is_main => &["toggle"],
            TreeItem::Checkout(_) => &["toggle", "hide-worktree"],
            // its click opens the Run configurations tab, its chevron folds it
            TreeItem::Folder(row) if row.kind == FolderKind::Processes && row.empty => &["folder", "new-terminal"],
            TreeItem::Folder(row) if row.kind == FolderKind::Processes => &["folder", "toggle", "new-terminal"],
            TreeItem::Folder(_) => &["folder"],
            TreeItem::Process(_) => &["process", "stop-process"],
            TreeItem::LiveRun(row) if row.link.is_some() => &["live-run", "open-link", "copy-link", "restart-process", "stop-process"],
            TreeItem::LiveRun(row) if row.live => &["live-run", "restart-process", "stop-process"],
            TreeItem::LiveRun(_) => &["live-run", "start-process", "remove-process"],
            TreeItem::Change(_) => &["change"],
            TreeItem::ChangeFolder(_) => &["change-folder"],
            TreeItem::Commit(_) => &["commit"],
            TreeItem::CommitsMore(_) => &["commits-more"],
            TreeItem::Note(_) => &[],
            TreeItem::Entry(_) => &["entry"],
            TreeItem::More(_) => &["more", "all"],
        };
        verbs.iter().map(|verb| target_name(verb, &self.target)).collect()
    }
}

/// A click target's name: `project:atlassian`, `toggle:atlassian/dev`, `entry:/r/src`.
pub fn target_name(verb: &str, row: &str) -> String {
    format!("{verb}:{row}")
}

/// Every visible row, top to bottom. A project with one checkout (or no git) has no worktree level: its
/// Processes row and files sit directly under it.
pub fn build<'a, V>(projects: impl IntoIterator<Item = &'a OpenProject>, processes: &Processes<V>, runs: &dyn RunsView, look: &TreeLook) -> Vec<TreeRow> {
    let mut rows = Vec::new();
    let tree = Sources { processes, runs, look };
    for project in projects {
        push_project(&mut rows, project, &tree);
    }
    rows
}

/// What the rows are built from besides the projects.
struct Sources<'a, V> {
    processes: &'a Processes<V>,
    runs: &'a dyn RunsView,
    look: &'a TreeLook<'a>,
}

fn push_project<V>(rows: &mut Vec<TreeRow>, open: &OpenProject, tree: &Sources<V>) {
    let item = TreeItem::Project(project_row(open, tree.processes));
    rows.push(TreeRow { depth: 0, item, target: open.project.id.to_string() });
    if open.shows_children() {
        push_children(rows, open, tree);
    }
}

/// The rows below a project's own: its checkouts with a worktree level, else its only checkout's content.
fn push_children<V>(rows: &mut Vec<TreeRow>, open: &OpenProject, tree: &Sources<V>) {
    let project = &open.project;
    if open.has_worktree_level() {
        return push_checkouts(rows, open, tree);
    }
    push_checkout_content(rows, open, &project.checkout_ref(project.main_checkout()), 1, tree);
}

fn push_checkouts<V>(rows: &mut Vec<TreeRow>, open: &OpenProject, tree: &Sources<V>) {
    let project = &open.project;
    let list = PagedList::Checkouts(project.id.clone());
    let listed: Vec<&Checkout> = open.listed_checkouts().collect();
    let total = listed.len();
    let shown = open.paging(&list).visible(total);
    for checkout in listed.into_iter().take(shown) {
        let checkout_ref = project.checkout_ref(checkout);
        let expanded = open.is_checkout_expanded(&checkout.path);
        let target = checkout_target(open, checkout);
        let item = TreeItem::Checkout(CheckoutRow { checkout: checkout_ref.clone(), label: checkout.label.clone(), expanded, is_main: checkout.is_main });
        rows.push(TreeRow { depth: 1, item, target });
        if expanded {
            push_checkout_content(rows, open, &checkout_ref, 2, tree);
        }
    }
    push_more(rows, open, list, total, 1);
}

/// A plain folder's git folders list nothing.
static NO_GIT: std::sync::LazyLock<CheckoutGit> = std::sync::LazyLock::new(CheckoutGit::default);

fn git_folders<'a>(open: &'a OpenProject, checkout: &'a CheckoutRef, target: &'a str, look: &'a TreeLook) -> GitFolders<'a> {
    let git = match open.project.kind {
        ProjectKind::Plain => Some(&*NO_GIT),
        ProjectKind::Git => look.git.get(&checkout.path),
    };
    GitFolders { checkout, target, folders: open.folders(&checkout.path), git, layout: look.layout, clock: look.clock }
}

/// What Processes lists: terminals and Claude Code, and the runs.
fn processes_count<V>(checkout: &CheckoutRef, tree: &Sources<V>) -> usize {
    tree.processes.by_checkout(&checkout.path).len() + tree.runs.runs_of(&checkout.path).len()
}

/// A checkout's folders: those switched on, in order.
fn push_checkout_content<V>(rows: &mut Vec<TreeRow>, open: &OpenProject, checkout: &CheckoutRef, depth: usize, tree: &Sources<V>) {
    let found = open.project.checkout(&checkout.path);
    let target = found.map_or_else(String::new, |found| checkout_target(open, found));
    let git = git_folders(open, checkout, &target, tree.look);
    let shown = git.states(processes_count(checkout, tree)).into_iter().filter(|state| state.shown).map(|state| (state.kind, state.count));
    for (kind, count) in shown {
        let expanded = git.folders.is_expanded(kind, tree.look.open_folders);
        let nothing = git.nothing(kind);
        // an empty Processes has no chevron, as an empty git folder; its + still starts a terminal
        let empty = nothing.is_some() || (kind == FolderKind::Processes && count == Some(0));
        let item = TreeItem::Folder(FolderRow { checkout: checkout.clone(), kind, count, expanded, empty, nothing });
        rows.push(TreeRow { depth, item, target: format!("{target}/{}", kind.name()) });
        match kind {
            _ if !expanded || empty => {}
            FolderKind::Processes => push_processes(rows, open, checkout, depth + 1, tree),
            FolderKind::Files => push_files(rows, open, checkout, depth + 1, tree),
            _ => rows.extend(git.rows(kind, depth + 1)),
        }
    }
}

/// Terminals and Claude Code, ten at a time, then every run.
fn push_processes<V>(rows: &mut Vec<TreeRow>, open: &OpenProject, checkout: &CheckoutRef, depth: usize, tree: &Sources<V>) {
    let running = tree.processes.by_checkout(&checkout.path);
    let list = PagedList::Processes(checkout.path.clone());
    let shown = open.paging(&list).visible(running.len());
    for process in &running[..shown] {
        // Stop removes the row once the process is gone; until then it says so
        let title = if tree.processes.is_closing(process.id) { format!("{} · stopping…", process.title()) } else { process.title() };
        let item = TreeItem::Process(ProcessRow { id: process.id, kind: process.kind, title, progress: process.progress() });
        rows.push(TreeRow { depth, item, target: process.id.value().to_string() });
    }
    push_more(rows, open, list, running.len(), depth);
    for run in tree.runs.runs_of(&checkout.path) {
        let target = run.id.target();
        rows.push(TreeRow { depth, item: TreeItem::LiveRun(run), target });
    }
}

fn push_files<V>(rows: &mut Vec<TreeRow>, open: &OpenProject, checkout: &CheckoutRef, depth: usize, tree: &Sources<V>) {
    let unread = TreeStatus::default();
    let status = tree.look.statuses.get(&checkout.path).unwrap_or(&unread);
    for row in open.files(&checkout.path).map(|files| files.rows()).unwrap_or_default() {
        let target = entry_target(open, checkout, &row.entry);
        // a .run.toml at the checkout's root is ignored: the root has no folder row
        let at_root = row.entry.path.parent() == Some(checkout.path.as_path());
        let warning = match &row.entry.run {
            RunState::Broken(error) if row.entry.kind == EntryKind::File && !at_root => Some(error.to_string()),
            _ => None,
        };
        let run_file = row.entry.kind == EntryKind::Folder && matches!(row.entry.run, RunState::Loaded(_));
        let (state, mark) = entry_look(&row.entry, &checkout.path, status, tree.look.excluded);
        let item = TreeItem::Entry(EntryRow { checkout: checkout.clone(), entry: row.entry.clone(), expanded: row.expanded, warning, run_file, state, mark });
        rows.push(TreeRow { depth: depth + row.depth, item, target });
    }
}

/// An entry's state and git mark; the `@hidden` group row is plain.
fn entry_look(entry: &Entry, checkout: &Path, status: &TreeStatus, excluded: &[String]) -> (EntryState, Option<Mark>) {
    let Ok(relative) = entry.path.strip_prefix(checkout) else {
        return (EntryState::Plain, None);
    };
    if entry.kind == EntryKind::HiddenGroup {
        return (EntryState::Plain, None);
    }
    let is_dir = entry.expands();
    (entry_state(relative, is_dir, status, excluded), entry_mark(relative, is_dir, status))
}

fn push_more(rows: &mut Vec<TreeRow>, open: &OpenProject, list: PagedList, total: usize, depth: usize) {
    let shown = open.paging(&list).visible(total);
    if shown < total {
        let target = match &list {
            PagedList::Checkouts(project) => project.to_string(),
            PagedList::Processes(checkout) => checkout.display().to_string(),
        };
        rows.push(TreeRow { depth, item: TreeItem::More(MoreRow { project: open.project.id.clone(), list, total, shown }), target });
    }
}

/// `<project id>/<checkout's target label>`.
fn checkout_target(open: &OpenProject, checkout: &Checkout) -> String {
    format!("{}/{}", open.project.id, open.project.target_label(checkout))
}

/// The entry's path; a file a worktree inside this checkout also shows (`.claude/worktrees/x/…` in the main
/// checkout's tree) adds `@<this checkout's label>`, so both rows have targets of their own.
fn entry_target(open: &OpenProject, checkout: &CheckoutRef, entry: &Entry) -> String {
    let deepest = open.project.checkout_containing(&entry.path).map(|found| found.path.as_path());
    let path = entry.path.display();
    match open.project.checkout(&checkout.path) {
        Some(found) if deepest != Some(checkout.path.as_path()) => format!("{path}@{}", open.project.target_label(found)),
        _ => path.to_string(),
    }
}

fn project_row<V>(open: &OpenProject, processes: &Processes<V>) -> ProjectRow {
    let project = &open.project;
    let tag = match (open.missing, open.loaded, project.kind) {
        (true, _, _) => Some(ProjectTag::Missing),
        (false, false, _) => None,
        (false, true, ProjectKind::Plain) => Some(ProjectTag::NoGit),
        (false, true, ProjectKind::Git) if open.has_worktree_level() => None,
        (false, true, ProjectKind::Git) => Some(ProjectTag::Branch(project.main_checkout().label.clone())),
    };
    ProjectRow {
        id: project.id.clone(),
        name: project.name.clone(),
        initials: initials(&project.name),
        expanded: open.expanded,
        tag,
        running: processes.in_project(&project.id).iter().filter(|process| process.is_live()).count(),
        error: project.refresh_error.clone(),
    }
}

/// Up to two letters for a project badge: the first letter of the first two words, as IntelliJ does.
pub fn initials(name: &str) -> String {
    let words = name.split(|c: char| !c.is_alphanumeric()).filter(|word| !word.is_empty());
    let letters: String = words.take(2).filter_map(|word| word.chars().next()).flat_map(char::to_uppercase).collect();
    if letters.is_empty() {
        return "?".to_owned();
    }
    letters
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::fs;
    use std::path::Path;

    use super::*;
    use crate::age::When;
    use crate::git::log::CommitInfo;
    use crate::git::status::Change;
    use crate::git::worktrees::BaseBranch;
    use crate::project::{Checkout, CheckoutStatus, ProjectScan};
    use crate::sidebar::folders::SinceGit;

    fn checkout(path: &Path, is_main: bool, branch: &str) -> Checkout {
        Checkout {
            label: if is_main { branch.to_owned() } else { path.file_name().unwrap().to_string_lossy().into_owned() },
            path: path.to_path_buf(),
            branch: Some(branch.to_owned()),
            head: None,
            is_main,
            status: Some(CheckoutStatus::default()),
        }
    }

    fn git_project(id: &str, root: &Path, worktrees: usize) -> OpenProject {
        let mut checkouts = vec![checkout(root, true, "dev")];
        checkouts.extend((0..worktrees).map(|n| checkout(&root.join(format!("wt{n:02}")), false, "feature")));
        let mut open = OpenProject::placeholder(ProjectId::new(id), root.to_path_buf());
        open.apply(1, Ok(ProjectScan { root: root.to_path_buf(), kind: ProjectKind::Git, checkouts, error: None, git_dir: None }));
        open
    }

    /// No status read and no exclusion rules.
    fn no_look() -> TreeLook<'static> {
        static NONE: std::sync::LazyLock<HashMap<PathBuf, TreeStatus>> = std::sync::LazyLock::new(HashMap::new);
        static NO_DATA: std::sync::LazyLock<HashMap<PathBuf, CheckoutGit>> = std::sync::LazyLock::new(HashMap::new);
        TreeLook { statuses: &NONE, excluded: &[], open_folders: &crate::script::open_folders::ALL_OPEN, git: &NO_DATA, layout: FileLayout::Flat, clock: Clock::new(DAY_2, 0) }
    }

    /// Two days after the epoch: the tests' commits are a day old.
    const DAY_2: i64 = 2 * 86_400;

    fn entry_states(rows: &[TreeRow]) -> Vec<(String, EntryState, Option<Mark>)> {
        rows.iter()
            .filter_map(|row| match &row.item {
                TreeItem::Entry(entry) => Some((entry.entry.name.clone(), entry.state, entry.mark)),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn should_mark_excluded_folder_row() {
        // given
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join("target")).unwrap();
        fs::write(dir.path().join("debug.log"), "").unwrap();
        let mut project = git_project("forge", dir.path(), 0);
        project.toggle();
        project.read_files_now();
        let ignored = TreeStatus { ignored: HashSet::from([PathBuf::from("debug.log")]), marks: HashMap::new(), ..TreeStatus::default() };
        let statuses = HashMap::from([(dir.path().to_path_buf(), ignored)]);
        let excluded = ["target".to_owned()];

        // when
        let rows = build(&[project], &Processes::<()>::default(), &FakeRuns::default(), &TreeLook { statuses: &statuses, excluded: &excluded, ..no_look() });

        // then
        assert_eq!(entry_states(&rows), [
            ("target".to_owned(), EntryState::Excluded, None),
            ("debug.log".to_owned(), EntryState::Ignored, None),
        ]);
    }

    #[test]
    fn should_keep_git_colour_inside_excluded_folder() {
        // given
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join("target")).unwrap();
        fs::write(dir.path().join("target/x.txt"), "").unwrap();
        let mut project = git_project("forge", dir.path(), 0);
        project.toggle();
        project.read_files_now();
        let expanded = project.toggle_folder(dir.path(), &dir.path().join("target"));
        project.apply_listings(&expanded.iter().map(|folder| crate::file_tree::read_folder(folder)).collect::<Vec<_>>());
        let modified = TreeStatus { ignored: HashSet::new(), marks: HashMap::from([(PathBuf::from("target/x.txt"), Mark::Modified)]), ..TreeStatus::default() };
        let statuses = HashMap::from([(dir.path().to_path_buf(), modified)]);
        let excluded = ["target".to_owned()];

        // when
        let rows = build(&[project], &Processes::<()>::default(), &FakeRuns::default(), &TreeLook { statuses: &statuses, excluded: &excluded, ..no_look() });

        // then
        assert_eq!(entry_states(&rows), [
            ("target".to_owned(), EntryState::Excluded, None),
            ("x.txt".to_owned(), EntryState::Excluded, Some(Mark::Modified)),
        ]);
    }

    fn outline<V>(projects: &[OpenProject], processes: &Processes<V>) -> Vec<String> {
        outline_rows(&build(projects, processes, &FakeRuns::default(), &no_look()))
    }

    fn outline_rows(rows: &[TreeRow]) -> Vec<String> {
        rows.iter()
            .map(|row| {
                let text = match &row.item {
                    TreeItem::Project(project) => format!("{} {:?} {}", project.name, project.tag, project.running),
                    TreeItem::Checkout(checkout) => format!("checkout {}", checkout.label),
                    TreeItem::Folder(folder) => {
                        let count = folder.count.map_or(String::new(), |count| format!(" {count}"));
                        let chevron = if folder.empty { " (no chevron)" } else { "" };
                        format!("{}{count}{chevron}", folder.kind.title())
                    }
                    TreeItem::Process(process) => format!("process {}", process.title),
                    TreeItem::LiveRun(run) => format!("run {} {}", run.name, run.label),
                    TreeItem::Entry(entry) => entry.entry.name.clone(),
                    TreeItem::Change(change) => {
                        let folder = change.folder.as_ref().map_or(String::new(), |folder| format!(" ({folder})"));
                        format!("{} {}{folder}", change.change.path.file_name(), change.change.mark.letter())
                    }
                    TreeItem::ChangeFolder(folder) => format!("{}/ {}", folder.label, folder.files),
                    TreeItem::Commit(commit) => format!("{} · {}", commit.label, commit.meta),
                    TreeItem::CommitsMore(more) => more.text.clone(),
                    TreeItem::Note(note) => format!("! {}", note.text),
                    TreeItem::More(more) => format!("more {}/{}", more.shown, more.total),
                };
                format!("{}{text}", "  ".repeat(row.depth))
            })
            .collect()
    }

    #[test]
    fn should_put_processes_and_files_of_single_checkout_project_under_the_project() {
        // given
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("CLAUDE.md"), "").unwrap();
        let mut project = git_project("forge", dir.path(), 0);
        project.toggle();
        project.read_files_now();

        // when
        let rows = outline(&[project], &Processes::<()>::default());

        // then
        assert!(rows[0].ends_with(" Some(Branch(\"dev\")) 0"), "{rows:?}");
        assert_eq!(rows[1..], ["  Processes 0 (no chevron)", "  Files", "    CLAUDE.md"]);
    }

    #[test]
    fn should_show_the_label_of_a_detached_single_checkout_as_its_pill() {
        // given
        let mut project = git_project("forge", Path::new("/forge"), 0);
        let mut detached = project.project.checkouts.clone();
        detached[0].branch = None;
        detached[0].label = "HEAD detached at 0123456".to_owned();
        project.apply(2, Ok(ProjectScan { root: PathBuf::from("/forge"), kind: ProjectKind::Git, checkouts: detached, error: None, git_dir: None }));

        // when
        let rows = outline(&[project], &Processes::<()>::default());

        // then
        assert_eq!(rows, ["forge Some(Branch(\"HEAD detached at 0123456\")) 0"]);
    }

    #[test]
    fn should_show_no_git_tag_and_no_worktree_level_for_plain_project() {
        // given
        let mut project = OpenProject::placeholder(ProjectId::new("notes"), PathBuf::from("/n/notes"));
        project.apply(1, Ok(ProjectScan::plain(PathBuf::from("/n/notes"))));
        project.toggle();

        // when
        let rows = outline(&[project], &Processes::<()>::default());

        // then
        assert_eq!(rows, ["notes Some(NoGit) 0", "  Processes 0 (no chevron)", "  Files"]);
    }

    #[test]
    fn should_leave_hidden_worktrees_out_of_the_tree() {
        // given
        let mut project = git_project("p", Path::new("/p"), 2);
        project.toggle();
        project.hide(Path::new("/p/wt00"));

        // when
        let rows = outline(&[project], &Processes::<()>::default());

        // then
        assert_eq!(rows[1..], ["  checkout dev", "  checkout wt01"]);
    }

    #[test]
    fn should_drop_the_worktree_level_when_only_the_main_checkout_is_listed() {
        // given
        let mut project = git_project("p", Path::new("/p"), 2);
        project.toggle();
        project.hide(Path::new("/p/wt00"));
        project.hide(Path::new("/p/wt01"));

        // when
        let rows = build(&[project], &Processes::<()>::default(), &FakeRuns::default(), &no_look());

        // then
        assert!(matches!(rows[0].item, TreeItem::Project(_)));
        assert!(!rows.iter().any(|row| matches!(row.item, TreeItem::Checkout(_))));
    }

    #[test]
    fn should_name_a_hide_target_on_worktree_rows_only() {
        // given
        let mut project = git_project("p", Path::new("/p"), 1);
        project.toggle();

        // when
        let rows = build(&[project], &Processes::<()>::default(), &FakeRuns::default(), &no_look());

        // then
        let hides = |row: &TreeRow| row.target_names().iter().any(|name| name.starts_with("hide-worktree:"));
        let checkouts: Vec<bool> = rows.iter().filter(|row| matches!(row.item, TreeItem::Checkout(_))).map(hides).collect();
        assert_eq!(checkouts, [false, true]);
    }

    #[test]
    fn should_page_checkouts_ten_at_a_time() {
        // given
        let mut project = git_project("big", Path::new("/big"), 14);
        project.toggle();

        // when
        let rows = outline(&[project], &Processes::<()>::default());

        // then
        assert_eq!(rows.len(), 1 + 10 + 1);
        assert_eq!((rows[1].as_str(), rows[11].as_str()), ("  checkout dev", "  more 10/15"));
    }

    #[test]
    fn should_page_far_enough_to_show_a_revealed_checkout() {
        // given
        let mut project = git_project("big", Path::new("/big"), 24);

        // when
        project.reveal(Path::new("/big/wt12"));

        // then
        let rows = outline(&[project], &Processes::<()>::default());
        assert!(rows.contains(&"  checkout wt12".to_owned()), "{rows:?}");
        assert_eq!(rows.last().map(String::as_str), Some("  more 20/25"));
    }

    #[test]
    fn should_count_processes_of_checkout_and_project() {
        // given
        let mut project = git_project("atlassian", Path::new("/atlassian"), 1);
        project.toggle();
        project.toggle_checkout(Path::new("/atlassian/wt00"));
        let mut processes: Processes<()> = Processes::default();
        let worktree = Some(CheckoutRef { project: project.project.id.clone(), path: PathBuf::from("/atlassian/wt00") });
        processes.start(worktree.clone(), |_| Ok::<_, ()>(())).unwrap();
        let claude = processes.start(worktree, |_| Ok::<_, ()>(())).unwrap();
        processes.set_kind(claude, ProcessKind::ClaudeCode);
        processes.set_title(claude, "✳ Claude Code".to_owned());

        // when
        let rows = outline(&[project], &processes);

        // then
        assert_eq!(
            rows,
            ["atlassian None 2", "  checkout dev", "  checkout wt00", "    Processes 2", "      process Terminal", "      process ✳ Claude Code", "    Files"]
        );
    }

    #[test]
    fn should_count_only_live_runs_as_running() {
        // given
        let project = git_project("atlassian", Path::new("/atlassian"), 0);
        let mut processes: Processes<()> = Processes::default();
        let checkout = Some(CheckoutRef { project: ProjectId::new("atlassian"), path: PathBuf::from("/atlassian") });
        let origin = |key: &str| crate::process::RunOrigin {
            folder: PathBuf::from("/atlassian/misc"),
            key: key.to_owned(),
            name: key.to_owned(),
            url: None,
            life: crate::process::RunLife::Running,
        };
        processes.start_run(checkout.clone(), origin("app"), |_| Ok::<_, ()>(())).unwrap();
        let build = processes.start_run(checkout, origin("build"), |_| Ok::<_, ()>(())).unwrap();
        processes.set_exited(build, 0);

        // when
        let rows = outline(&[project], &processes);

        // then
        assert_eq!(rows, ["atlassian Some(Branch(\"dev\")) 1"]);
    }

    #[test]
    fn should_show_missing_project_without_children() {
        // given
        let saved = crate::state::SavedProject {
            path: PathBuf::from("/nonexistent/ive"),
            id: "gone".to_owned(),
            expanded: true,
            expanded_checkouts: Vec::new(),
            last_checkout: None,
            folders: Vec::new(),
            hidden_worktrees: Vec::new(),
            tabs_folded: false,
            worktree_order: Vec::new(),
        };
        let mut project = OpenProject::restored(&saved, ProjectId::new("gone"));
        let gone = std::io::Error::from(std::io::ErrorKind::NotFound);
        project.apply(1, Err(crate::project::ProjectError::Unreadable { path: saved.path.clone(), source: gone }));

        // when
        let rows = outline(&[project], &Processes::<()>::default());

        // then
        assert_eq!(rows, ["ive Some(Missing) 0"]);
    }

    #[test]
    fn should_name_the_click_targets_of_every_row_drawn_or_not() {
        // given
        let mut project = git_project("atlassian", Path::new("/a"), 1);
        project.toggle();
        project.toggle_checkout(Path::new("/a/wt00"));
        let mut processes: Processes<()> = Processes::default();
        let worktree = Some(CheckoutRef { project: ProjectId::new("atlassian"), path: PathBuf::from("/a/wt00") });
        let id = processes.start(worktree, |_| Ok::<_, ()>(())).unwrap().value();

        // when
        let names: Vec<String> = build(&[project], &processes, &FakeRuns::default(), &no_look()).iter().flat_map(TreeRow::target_names).collect();

        // then
        assert_eq!(names, [
            "project:atlassian".to_owned(),
            "toggle:atlassian".to_owned(),
            "close-project:atlassian".to_owned(),
            "toggle:atlassian/dev".to_owned(),
            "toggle:atlassian/wt00".to_owned(),
            "hide-worktree:atlassian/wt00".to_owned(),
            "folder:atlassian/wt00/processes".to_owned(),
            "toggle:atlassian/wt00/processes".to_owned(),
            "new-terminal:atlassian/wt00/processes".to_owned(),
            format!("process:{id}"),
            format!("stop-process:{id}"),
            "folder:atlassian/wt00/files".to_owned(),
        ]);
    }

    #[test]
    fn should_tell_apart_a_worktree_file_shown_in_both_trees() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        fs::create_dir_all(root.join("wt")).unwrap();
        fs::write(root.join("wt/a.txt"), "").unwrap();
        let mut project = git_project("r", &root, 0);
        let checkouts = vec![checkout(&root, true, "dev"), checkout(&root.join("wt"), false, "feature")];
        project.apply(2, Ok(ProjectScan { root: root.clone(), kind: ProjectKind::Git, checkouts, error: None, git_dir: None }));
        project.reveal(&root);
        project.reveal(&root.join("wt"));
        project.read_files_now();
        project.toggle_folder(&root, &root.join("wt"));
        project.apply_listings(&[crate::file_tree::read_folder(&root.join("wt"))]);

        // when
        let names: Vec<String> = build(&[project], &Processes::<()>::default(), &FakeRuns::default(), &no_look()).iter().flat_map(TreeRow::target_names).collect();

        // then
        let file = root.join("wt/a.txt").display().to_string();
        assert!(names.contains(&format!("entry:{file}@dev")), "{names:?}");
        assert!(names.contains(&format!("entry:{file}")), "{names:?}");
    }

    /// Runs the tests make up, by folder.
    #[derive(Default)]
    struct FakeRuns(Vec<(PathBuf, RunRow)>);

    impl RunsView for FakeRuns {
        fn runs_in(&self, _checkout: &Path, folder: &Path) -> Vec<RunRow> {
            self.0.iter().filter(|(at, _)| at == folder).map(|(_, run)| run.clone()).collect()
        }

        fn runs_of(&self, _checkout: &Path) -> Vec<RunRow> {
            self.0.iter().map(|(_, run)| run.clone()).collect()
        }
    }

    fn run_row(key: &str, name: &str, state: RunBadge, live: bool, id: u64) -> RunRow {
        let label = match state {
            RunBadge::Exited(code) => format!("exit {code}"),
            _ => format!("{state:?}"),
        };
        RunRow { id: RunId::Pending(id), key: key.to_owned(), name: name.to_owned(), serve: false, state, status: String::new(), label, live, link: None }
    }

    const MISC: &str = "[run.build]\nname = \"Build misc\"\ncommand = \"make\"\n\n[run.app]\ncommand = \"make run\"\nbefore = [\"build\"]\n";

    /// A one-checkout project at a temporary root holding `misc/` with `run_file` as its `.run.toml`, expanded down
    /// to `misc/`.
    fn run_project(run_file: &str) -> (tempfile::TempDir, PathBuf, OpenProject) {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        fs::create_dir_all(root.join("misc")).unwrap();
        fs::write(root.join("misc/.run.toml"), run_file).unwrap();
        fs::write(root.join("misc/pom.xml"), "").unwrap();
        let mut project = git_project("r", &root, 0);
        project.toggle();
        project.read_files_now();
        project.toggle_folder(&root, &root.join("misc"));
        project.apply_listings(&[crate::file_tree::read_folder(&root.join("misc"))]);
        (dir, root, project)
    }

    fn run_outline(projects: &[OpenProject], runs: &FakeRuns) -> Vec<String> {
        outline_of(&build(projects, &Processes::<()>::default(), runs, &no_look()))
    }

    /// The rows below Processes and Files, as names indented by depth.
    fn outline_of(rows: &[TreeRow]) -> Vec<String> {
        rows.iter()
            .skip(3)
            .map(|row| {
                let text = match &row.item {
                    TreeItem::Entry(entry) => format!("{}{}", entry.entry.name, entry.warning.as_ref().map_or("", |_| " ⚠")),
                    _ => "?".to_owned(),
                };
                format!("{}{text}", "  ".repeat(row.depth - 2))
            })
            .collect()
    }

    #[test]
    fn should_not_show_run_node_in_a_folder_with_a_run_file() {
        // given
        let (_dir, _root, project) = run_project(MISC);

        // when
        let rows = build(std::slice::from_ref(&project), &Processes::<()>::default(), &FakeRuns::default(), &no_look());

        // then
        assert_eq!(outline_of(&rows), ["misc", "  .run.toml", "  pom.xml"]);
        let run_files: Vec<bool> = rows.iter().filter_map(|row| match &row.item {
            TreeItem::Entry(entry) if entry.entry.name == "misc" => Some(entry.run_file),
            _ => None,
        }).collect();
        assert_eq!(run_files, [true]);
    }

    #[test]
    fn should_not_show_run_node_for_a_broken_file_and_warn_on_the_file() {
        // given
        let (_dir, _root, project) = run_project("[run.Build]\ncommand = \"make\"\n");

        // when
        let rows = build(std::slice::from_ref(&project), &Processes::<()>::default(), &FakeRuns::default(), &no_look());

        // then
        let outline = run_outline(std::slice::from_ref(&project), &FakeRuns::default());
        assert_eq!(outline, ["misc", "  .run.toml ⚠", "  pom.xml"]);
        let folder = rows.iter().find_map(|row| match &row.item {
            TreeItem::Entry(entry) if entry.entry.name == "misc" => Some(entry.run_file),
            _ => None,
        });
        assert_eq!(folder, Some(false));
    }

    #[test]
    fn should_ignore_a_run_file_at_the_checkout_root() {
        // given
        let (_dir, root, mut project) = run_project(MISC);
        fs::write(root.join(".run.toml"), "broken = [").unwrap();
        project.apply_listings(&[crate::file_tree::read_folder(&root)]);

        // when
        let rows = run_outline(&[project], &FakeRuns::default());

        // then
        assert_eq!(rows[..1], ["misc"]);
        assert_eq!(rows.last().map(String::as_str), Some(".run.toml"));
    }

    #[test]
    fn should_name_run_click_targets() {
        // given
        let (_dir, root, project) = run_project(MISC);
        let misc = root.join("misc");
        let runs = FakeRuns(vec![
            (misc.clone(), RunRow { id: RunId::Pending(7), ..run_row("app", "app", RunBadge::Waiting, true, 7) }),
            (misc.clone(), run_row("build", "Build misc", RunBadge::Exited(0), false, 8)),
        ]);

        // when
        let names: Vec<String> = build(&[project], &Processes::<()>::default(), &runs, &no_look()).iter().flat_map(TreeRow::target_names).collect();

        // then
        assert!(names.iter().all(|name| !name.starts_with("run-node:") && !name.starts_with("run:")), "{names:?}");
        for expected in ["live-run:pending-8", "start-process:pending-8", "remove-process:pending-8", "live-run:pending-7", "restart-process:pending-7", "stop-process:pending-7"] {
            assert!(names.contains(&expected.to_owned()), "{expected} not in {names:?}");
        }
    }

    #[test]
    fn should_take_first_letters_of_first_two_words() {
        // given
        let names = ["ive-big", "atlassian", "my cool project", "żółw_app", "--"];

        // when
        let badges: Vec<_> = names.iter().map(|name| initials(name)).collect();

        // then
        assert_eq!(badges, ["IB", "A", "MC", "ŻA", "?"]);
    }

    fn change(mark: Mark, path: &str) -> Change {
        Change { mark, path: GitPath::from_bytes(path.as_bytes()), old_path: None }
    }

    fn sha(n: u8) -> Sha {
        Sha::parse(&format!("{n:02x}").repeat(20)).unwrap()
    }

    fn commit(n: u8, subject: &str) -> CommitInfo {
        CommitInfo { sha: sha(n), subject: subject.to_owned(), author: "kb".to_owned(), when: When { seconds: 86_400, offset_minutes: 0 }, parents: Vec::new() }
    }

    fn since(files: Vec<Change>, commits: Vec<CommitInfo>) -> SinceGit {
        let base = BaseBranch { name: "dev".to_owned(), reference: "refs/heads/dev".to_owned() };
        SinceGit { base, fork: sha(0xff), files, commits, more: false }
    }

    /// Project `forge` at `/forge`, expanded, with one checkout on branch `dev`, its files not read.
    fn single() -> OpenProject {
        let mut project = git_project("forge", Path::new("/forge"), 0);
        project.toggle();
        project
    }

    fn folder_outline(project: &OpenProject, git: Option<CheckoutGit>, layout: FileLayout) -> Vec<String> {
        let data: HashMap<PathBuf, CheckoutGit> = git.into_iter().map(|git| (PathBuf::from("/forge"), git)).collect();
        let look = TreeLook { git: &data, layout, ..no_look() };
        outline_rows(&build(std::slice::from_ref(project), &Processes::<()>::default(), &FakeRuns::default(), &look)[1..])
    }

    fn states(project: &OpenProject, git: Option<CheckoutGit>) -> Vec<(FolderKind, Option<usize>, bool)> {
        let data: HashMap<PathBuf, CheckoutGit> = git.into_iter().map(|git| (PathBuf::from("/forge"), git)).collect();
        let look = TreeLook { git: &data, ..no_look() };
        let checkout = project.project.checkout_ref(project.project.main_checkout());
        let states = git_folders(project, &checkout, "forge/dev", &look).states(0);
        states.iter().map(|state| (state.kind, state.count, state.shown)).collect()
    }

    fn show_all(project: &mut OpenProject) {
        for kind in [FolderKind::Uncommitted, FolderKind::SinceBase, FolderKind::Commits] {
            project.folders_mut(Path::new("/forge")).toggle_shown(kind);
        }
    }

    fn some_git() -> CheckoutGit {
        let commits = vec![commit(1, "[GEN-1] App - second step"), commit(2, "[GEN-1] App - first step")];
        let files = vec![change(Mark::Modified, "src/a.rs"), change(Mark::Added, "src/b.rs")];
        CheckoutGit { changes: vec![change(Mark::Deleted, "old.txt")], since: Some(since(files, commits)), commit_files: HashMap::new() }
    }

    #[test]
    fn should_show_each_git_folder_switched_on() {
        // given
        let mut project = single();
        show_all(&mut project);

        // when
        let rows = folder_outline(&project, Some(some_git()), FileLayout::Flat);

        // then
        assert_eq!(rows, [
            "  Processes 0 (no chevron)",
            "  Uncommitted 1",
            "    old.txt D",
            "  Since base 2",
            "    a.rs M (src)",
            "    b.rs A (src)",
            "  Commits 2",
            "    second step · 0101010 · yesterday",
            "    first step · 0202020 · yesterday",
            "  Files",
        ]);
    }

    #[test]
    fn should_mark_switched_on_folders() {
        // given
        let mut project = single();
        project.folders_mut(Path::new("/forge")).toggle_shown(FolderKind::Commits);

        // when
        let states = states(&project, Some(some_git()));

        // then
        assert_eq!(states, [
            (FolderKind::Processes, Some(0), true),
            (FolderKind::Uncommitted, Some(1), false),
            (FolderKind::SinceBase, Some(2), false),
            (FolderKind::Commits, Some(2), true),
            (FolderKind::Files, None, true),
        ]);
    }

    #[test]
    fn should_leave_out_files_switched_off() {
        // given
        let mut project = single();
        project.folders_mut(Path::new("/forge")).toggle_shown(FolderKind::Files);

        // when
        let (rows, states) = (folder_outline(&project, None, FileLayout::Flat), states(&project, None));

        // then
        assert_eq!(rows, ["  Processes 0 (no chevron)"]);
        assert_eq!(states[4], (FolderKind::Files, None, false));
    }

    /// What each folder row of `/forge` says it lacks: `(kind, mark, tip)`.
    fn nothings(project: &OpenProject, git: CheckoutGit) -> Vec<(FolderKind, Option<(&'static str, String)>)> {
        let data = HashMap::from([(PathBuf::from("/forge"), git)]);
        let look = TreeLook { git: &data, ..no_look() };
        let rows = build(std::slice::from_ref(project), &Processes::<()>::default(), &FakeRuns::default(), &look);
        rows.iter()
            .filter_map(|row| match &row.item {
                TreeItem::Folder(folder) => Some((folder.kind, folder.nothing.as_ref().map(|nothing| (nothing.mark, nothing.tip.clone())))),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn should_keep_empty_git_folders_and_say_why_they_are_empty() {
        // given
        let mut project = single();
        show_all(&mut project);
        let clean = CheckoutGit { changes: Vec::new(), since: Some(since(Vec::new(), Vec::new())), commit_files: HashMap::new() };

        // when
        let (rows, states, nothings) =
            (folder_outline(&project, Some(clean.clone()), FileLayout::Flat), states(&project, Some(clean.clone())), nothings(&project, clean));

        // then
        assert_eq!(rows, ["  Processes 0 (no chevron)", "  Uncommitted 0 (no chevron)", "  Since base 0 (no chevron)", "  Commits 0 (no chevron)", "  Files"]);
        assert_eq!(states[1..4], [(FolderKind::Uncommitted, Some(0), true), (FolderKind::SinceBase, Some(0), true), (FolderKind::Commits, Some(0), true)]);
        assert_eq!(nothings[1..4], [
            (FolderKind::Uncommitted, Some(("0", "Nothing uncommitted".to_owned()))),
            (FolderKind::SinceBase, Some(("0", "No changes since dev".to_owned()))),
            (FolderKind::Commits, Some(("0", "No commits since dev".to_owned()))),
        ]);
        assert_eq!(nothings[0], (FolderKind::Processes, None));
    }

    #[test]
    fn should_show_since_base_and_commits_without_a_base_as_empty() {
        // given
        let mut project = single();
        show_all(&mut project);
        let no_base = CheckoutGit { changes: vec![change(Mark::Modified, "a.txt")], since: None, commit_files: HashMap::new() };

        // when
        let (states, nothings) = (states(&project, Some(no_base.clone())), nothings(&project, no_base));

        // then
        assert_eq!(states[1..4], [(FolderKind::Uncommitted, Some(1), true), (FolderKind::SinceBase, None, true), (FolderKind::Commits, None, true)]);
        let no_base = Some(("no base", "No base branch to compare with".to_owned()));
        assert_eq!(nothings[1..4], [(FolderKind::Uncommitted, None), (FolderKind::SinceBase, no_base.clone()), (FolderKind::Commits, no_base)]);
    }

    #[test]
    fn should_show_folders_without_counts_and_empty_until_git_is_read() {
        // given
        let mut project = single();
        project.folders_mut(Path::new("/forge")).toggle_shown(FolderKind::Uncommitted);

        // when
        let (rows, states) = (folder_outline(&project, None, FileLayout::Flat), states(&project, None));

        // then
        assert_eq!(rows, ["  Processes 0 (no chevron)", "  Uncommitted", "  Files"]);
        assert_eq!(states[1..4], [(FolderKind::Uncommitted, None, true), (FolderKind::SinceBase, None, false), (FolderKind::Commits, None, false)]);
    }

    #[test]
    fn should_group_changed_files_by_folder_in_the_tree_layout() {
        // given
        let mut project = single();
        project.folders_mut(Path::new("/forge")).toggle_shown(FolderKind::Uncommitted);
        let changes = vec![change(Mark::Modified, "src/ui/a.rs"), change(Mark::Deleted, "src/ui/b.rs"), change(Mark::Added, "README.md")];
        let git = CheckoutGit { changes, since: None, commit_files: HashMap::new() };

        // when
        let (flat, tree) = (folder_outline(&project, Some(git.clone()), FileLayout::Flat), folder_outline(&project, Some(git), FileLayout::Tree));

        // then
        assert_eq!(flat[1..5], ["  Uncommitted 3", "    README.md A", "    a.rs M (src/ui)", "    b.rs D (src/ui)"]);
        assert_eq!(tree[1..6], ["  Uncommitted 3", "    src/ui/ 2", "      a.rs M", "      b.rs D", "    README.md A"]);
    }

    #[test]
    fn should_fold_a_change_folder() {
        // given
        let mut project = single();
        let folders = project.folders_mut(Path::new("/forge"));
        folders.toggle_shown(FolderKind::Uncommitted);
        folders.set_change_folder(ChangeSource::Uncommitted, "src/ui".to_owned(), false);
        let git = CheckoutGit { changes: vec![change(Mark::Modified, "src/ui/a.rs")], since: None, commit_files: HashMap::new() };

        // when
        let rows = folder_outline(&project, Some(git), FileLayout::Tree);

        // then
        assert_eq!(rows[1..3], ["  Uncommitted 1", "    src/ui/ 1"]);
        assert_eq!(rows[3], "  Files");
    }

    #[test]
    fn should_open_only_the_change_folders_that_fit_in_twenty_rows() {
        // given
        let mut project = single();
        project.folders_mut(Path::new("/forge")).toggle_shown(FolderKind::SinceBase);
        let mut files: Vec<Change> = (0..30).map(|n| change(Mark::Added, &format!("big/{n:02}.rs"))).collect();
        files.extend(["src/ui/a.rs", "src/ui/b.rs"].map(|path| change(Mark::Modified, path)));
        let git = CheckoutGit { changes: Vec::new(), since: Some(since(files, Vec::new())), commit_files: HashMap::new() };

        // when
        let rows = folder_outline(&project, Some(git), FileLayout::Tree);

        // then
        assert_eq!(rows[1..6], ["  Since base 32", "    big/ 30", "    src/ui/ 2", "      a.rs M", "      b.rs M"]);
        assert_eq!(rows[6], "  Files");
    }

    #[test]
    fn should_unfold_a_commit_into_its_files_or_why_they_could_not_be_read() {
        // given
        let mut project = single();
        let folders = project.folders_mut(Path::new("/forge"));
        folders.toggle_shown(FolderKind::Commits);
        folders.toggle_commit(sha(1));
        folders.toggle_commit(sha(2));
        let mut git = some_git();
        git.commit_files.insert(sha(1), Ok(vec![change(Mark::Renamed, "lib/c.rs")]));
        git.commit_files.insert(sha(2), Err("object not found".to_owned()));

        // when
        let rows = folder_outline(&project, Some(git), FileLayout::Flat);

        // then
        assert_eq!(rows[2..7].iter().map(|row| row.split(" · ").next().unwrap_or_default()).collect::<Vec<_>>(), [
            "    second step",
            "      c.rs R (lib)",
            "    first step",
            "      ! object not found",
            "  Files",
        ]);
    }

    #[test]
    fn should_list_ten_commits_then_load_more_in_place() {
        // given
        let mut project = single();
        project.folders_mut(Path::new("/forge")).toggle_shown(FolderKind::Commits);
        let commits: Vec<CommitInfo> = (1..=32).map(|n| commit(n, &format!("step {n}"))).collect();
        let git = CheckoutGit { changes: Vec::new(), since: Some(since(vec![change(Mark::Added, "a")], commits)), commit_files: HashMap::new() };

        // when
        let first = folder_outline(&project, Some(git.clone()), FileLayout::Flat);
        project.folders_mut(Path::new("/forge")).show_more_commits();
        let second = folder_outline(&project, Some(git.clone()), FileLayout::Flat);
        project.folders_mut(Path::new("/forge")).show_more_commits();
        let third = folder_outline(&project, Some(git), FileLayout::Flat);

        // then
        assert_eq!((first.len(), first[12].as_str()), (1 + 1 + 10 + 1 + 1, "    Load 20 more · 10 of 32"));
        assert_eq!((second.len(), second[32].as_str()), (1 + 1 + 30 + 1 + 1, "    Load 2 more · 30 of 32"));
        assert_eq!(third.len(), 1 + 1 + 32 + 1);
    }

    #[test]
    fn should_keep_a_folded_folder_row_without_its_rows() {
        // given
        let mut project = single();
        project.read_files_now();
        project.folders_mut(Path::new("/forge")).toggle_folded(FolderKind::Files, &crate::script::open_folders::ALL_OPEN);

        // when
        let rows = folder_outline(&project, None, FileLayout::Flat);

        // then
        assert_eq!(rows, ["  Processes 0 (no chevron)", "  Files"]);
    }

    #[test]
    fn should_list_live_and_exited_runs_in_processes() {
        // given
        let (_dir, root, project) = run_project(MISC);
        let misc = root.join("misc");
        let running = RunRow { label: "● running".to_owned(), ..run_row("app", "app", RunBadge::Running, true, 1) };
        let exited = RunRow { label: "✕ exit 1".to_owned(), ..run_row("build", "build", RunBadge::Exited(1), false, 2) };
        let runs = FakeRuns(vec![(misc.clone(), running), (misc, exited)]);

        // when
        let rows = outline_rows(&build(&[project], &Processes::<()>::default(), &runs, &no_look()));

        // then
        assert_eq!(rows[1..4], ["  Processes 2", "    run app ● running", "    run build ✕ exit 1"]);
    }

    #[test]
    fn should_name_the_link_targets_of_a_running_server() {
        // given
        let server = RunRow { serve: true, link: Some("http://127.0.0.1:4200/".to_owned()), ..run_row("web", "web", RunBadge::Running, true, 1) };
        let row = TreeRow { depth: 2, item: TreeItem::LiveRun(server), target: "7".to_owned() };

        // when
        let names = row.target_names();

        // then
        assert_eq!(names, ["live-run:7", "open-link:7", "copy-link:7", "restart-process:7", "stop-process:7"]);
    }

    #[test]
    fn should_name_folder_and_change_targets() {
        // given
        let mut project = single();
        project.folders_mut(Path::new("/forge")).toggle_shown(FolderKind::Commits);
        project.folders_mut(Path::new("/forge")).toggle_commit(sha(1));
        let mut git = some_git();
        git.commit_files.insert(sha(1), Ok(vec![change(Mark::Modified, "lib/c.rs")]));
        let data = HashMap::from([(PathBuf::from("/forge"), git)]);

        // when
        let names: Vec<String> = build(&[project], &Processes::<()>::default(), &FakeRuns::default(), &TreeLook { git: &data, ..no_look() })
            .iter()
            .flat_map(TreeRow::target_names)
            .collect();

        // then
        let short = sha(1).short().to_owned();
        for name in [
            "folder:forge/dev/commits".to_owned(),
            format!("commit:forge/dev/{short}"),
            format!("change:forge/dev/{short}:lib/c.rs"),
            "folder:forge/dev/files".to_owned(),
        ] {
            assert!(names.contains(&name), "{name} not in {names:?}");
        }
    }
}
