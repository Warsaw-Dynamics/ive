//! What IVE remembers between runs that is not a preference: the open and recent projects, tabs and the window.
//! Kept apart from `settings.toml` because it is written often.

use std::collections::HashSet;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

use serde::{Deserialize, Serialize};

use crate::profile;
use crate::project::ProjectId;
use crate::saved_tabs::{SavedTab, SavedTabs};
use crate::sidebar::folders::FolderKind;
use crate::window_state::SavedWindow;

pub const VERSION: u32 = 2;
/// A state file this big is not something IVE wrote.
const MAX_STATE_BYTES: u64 = 512 * 1024;
pub const MAX_PROJECTS: usize = 100;
pub const MAX_RECENT: usize = 10;
/// shown tabs the hub remembers
pub const MAX_HISTORY: usize = 500;
/// checkouts whose last use is remembered
pub const MAX_WORKTREE_USE: usize = 200;
/// terminals whose checkout is remembered
pub const MAX_TERMINALS: usize = 1000;
/// open commits, change folders and expanded folders, each, that a checkout's folders keep
pub const MAX_SAVED_OPEN: usize = 1000;
pub const MAX_COMMITS_SHOWN: usize = 10_000;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SavedState {
    pub version: u32,
    /// the folders every worktree shows; `None`: Processes and Files
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shown_folders: Option<Vec<FolderKind>>,
    /// the ids of the open projects the tree and the hub leave out, sorted
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hidden_projects: Vec<String>,
    /// in tree order
    #[serde(default)]
    pub projects: Vec<SavedProject>,
    /// closed projects, newest first
    #[serde(default)]
    pub recent: Vec<RecentProject>,
    /// `None` before any tabs were saved in this profile
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tabs: Option<SavedTabs>,
    /// `None` before the window was saved; unusable bounds are ignored when opening, not set aside
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window: Option<SavedWindow>,
    /// the hub's history of shown tabs, oldest first
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub history: Vec<SavedShown>,
    /// the last input or run start in each checkout
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub worktree_use: Vec<SavedUse>,
    /// where each hosted terminal is listed, which its session's labels cannot follow
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub terminals: Vec<SavedTerminal>,
}

impl Default for SavedState {
    fn default() -> Self {
        Self { version: VERSION, shown_folders: None, hidden_projects: Vec::new(), projects: Vec::new(), recent: Vec::new(), tabs: None, window: None, history: Vec::new(), worktree_use: Vec::new(), terminals: Vec::new() }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedProject {
    /// main checkout or plain folder, canonical
    pub path: PathBuf,
    pub id: String,
    #[serde(default)]
    pub expanded: bool,
    #[serde(default)]
    pub expanded_checkouts: Vec<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_checkout: Option<PathBuf>,
    /// the checkouts whose worktree folders differ from the default
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub folders: Vec<SavedFolders>,
    /// worktrees left out of the tree and the hub until shown again, sorted
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hidden_worktrees: Vec<PathBuf>,
}

/// Which of a checkout's folders its badges switched on, which are folded, and what is open inside them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedFolders {
    pub checkout: PathBuf,
    /// the folders but Files
    pub shown: Vec<FolderKind>,
    /// Files was switched off; not part of `shown`, where state saved before Files had a badge never lists it
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub files_hidden: bool,
    /// folded by the user; the folders in neither list open as `config.open_folders` says
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub folded: Vec<FolderKind>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unfolded: Vec<FolderKind>,
    /// full shas
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub open_commits: Vec<String>,
    /// `None`: the first page
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commits_shown: Option<usize>,
    /// expanded folders of Files
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<PathBuf>,
    /// folders of Files whose hidden group is expanded
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hidden_groups: Vec<PathBuf>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub change_folders: Vec<SavedChangeFolder>,
}

impl SavedFolders {
    fn is_over_limits(&self) -> bool {
        let kinds = [self.open_commits.len(), self.files.len(), self.hidden_groups.len(), self.change_folders.len()];
        kinds.into_iter().any(|count| count > MAX_SAVED_OPEN) || self.commits_shown.is_some_and(|shown| shown > MAX_COMMITS_SHOWN)
    }
}

/// A change folder the user opened or closed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedChangeFolder {
    /// `uncommitted`, `since-base` or a commit's full sha
    pub list: String,
    pub path: String,
    pub open: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecentProject {
    pub path: PathBuf,
    pub id: String,
}

/// A tab the hub's Recent lists, when it was last shown (seconds since the epoch).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SavedShown {
    pub shown_at: i64,
    /// the checkout the tab belonged to when shown, for worktree recency before its project is read again
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checkout: Option<PathBuf>,
    pub tab: SavedTab,
}

/// A hosted terminal or Claude Code session's place: moved by hand or following its directory, it can be listed
/// under another checkout, even another project, than its session's labels say.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedTerminal {
    pub session: String,
    pub project: String,
    pub checkout: PathBuf,
    /// the directory it was moved from, which it does not follow
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pinned: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedUse {
    pub checkout: PathBuf,
    pub used_at: i64,
}

impl SavedState {
    fn validate(&self) -> Result<(), String> {
        if self.version != VERSION {
            return Err(format!("version {} is not supported, expected {VERSION}", self.version));
        }
        if self.history.len() > MAX_HISTORY || self.worktree_use.len() > MAX_WORKTREE_USE {
            return Err(format!(
                "{} shown tabs and {} worktree uses, the limits are {MAX_HISTORY} and {MAX_WORKTREE_USE}",
                self.history.len(),
                self.worktree_use.len()
            ));
        }
        if self.terminals.len() > MAX_TERMINALS {
            return Err(format!("{} terminals, the limit is {MAX_TERMINALS}", self.terminals.len()));
        }
        for terminal in &self.terminals {
            ProjectId::parse(&terminal.project)?;
        }
        if self.projects.len() > MAX_PROJECTS || self.recent.len() > MAX_RECENT {
            return Err(format!(
                "{} projects and {} recent ones, the limits are {MAX_PROJECTS} and {MAX_RECENT}",
                self.projects.len(),
                self.recent.len()
            ));
        }
        let ids = self.projects.iter().map(|project| &project.id).chain(self.recent.iter().map(|recent| &recent.id));
        let mut seen = HashSet::new();
        for id in ids {
            ProjectId::parse(id)?;
            if seen.insert(id) {
                continue;
            }
            return Err(format!("project id {id:?} is used twice"));
        }
        if let Some(tabs) = &self.tabs {
            tabs.validate()?;
        }
        for hidden in &self.hidden_projects {
            ProjectId::parse(hidden)?;
        }
        if let Some(over) = self.projects.iter().flat_map(|project| &project.folders).find(|folders| folders.is_over_limits()) {
            return Err(format!(
                "the folders of {} keep over {MAX_SAVED_OPEN} open items of a kind or list over {MAX_COMMITS_SHOWN} commits",
                over.checkout.display()
            ));
        }
        let checkouts = self.projects.iter().flat_map(|project| {
            let folders = project.folders.iter().flat_map(|folders| std::iter::once(&folders.checkout).chain(&folders.files).chain(&folders.hidden_groups));
            project.expanded_checkouts.iter().chain(&project.last_checkout).chain(&project.hidden_worktrees).chain(folders)
        });
        let paths = self.projects.iter().map(|project| &project.path).chain(self.recent.iter().map(|recent| &recent.path));
        let shown = self.history.iter().flat_map(|shown| shown.tab.paths());
        let shown = shown.chain(self.history.iter().flat_map(|shown| &shown.checkout));
        let used = self.worktree_use.iter().map(|used| &used.checkout);
        let terminals = self.terminals.iter().flat_map(|terminal| std::iter::once(&terminal.checkout).chain(&terminal.pinned));
        match paths.chain(checkouts).chain(shown).chain(used).chain(terminals).find(|path| path.is_relative()) {
            Some(relative) => Err(format!("path {} is not absolute", relative.display())),
            None => Ok(()),
        }
    }
}

/// Ids that still own data in the profile's folder (part 3's `scratches/<id>/`), so a new project never takes one
/// over. Reads the disk; run it on the background executor.
pub fn ids_with_data() -> Vec<String> {
    profile::current().scratches_dir().map(|dir| folder_names(&dir)).unwrap_or_default()
}

fn folder_names(dir: &Path) -> Vec<String> {
    let entries = fs::read_dir(dir).into_iter().flatten().flatten();
    entries.filter(|entry| entry.path().is_dir()).filter_map(|entry| entry.file_name().into_string().ok()).collect()
}

/// The saved state, and why the file was set aside when it could not be used.
pub struct Loaded {
    pub state: SavedState,
    pub problem: Option<String>,
}

/// Reads `path`; a missing file is an empty state. A file that is too large, unparsable or of another version is
/// renamed to `state.toml.bad` so the next save does not overwrite it, and IVE starts with no projects.
pub fn load(path: &Path) -> Loaded {
    match read(path) {
        Ok(state) => Loaded { state, problem: None },
        Err(reason) => Loaded { state: SavedState::default(), problem: Some(set_aside(path, &reason)) },
    }
}

fn read(path: &Path) -> Result<SavedState, String> {
    let bytes = match fs::metadata(path) {
        Ok(metadata) => metadata.len(),
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(SavedState::default()),
        Err(error) => return Err(error.to_string()),
    };
    if bytes > MAX_STATE_BYTES {
        return Err(format!("{bytes} bytes, the limit is {MAX_STATE_BYTES}"));
    }
    let text = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let state: SavedState = toml::from_str(&text).map_err(|error| error.message().to_owned())?;
    state.validate()?;
    Ok(state)
}

/// Renames the unusable file at `path` to `<name>.bad` so the next save keeps it; answers what happened, for the log.
pub fn set_aside(path: &Path, reason: &str) -> String {
    let bad = path.with_added_extension("bad");
    let bad_name = bad.file_name().map_or_else(String::new, |name| name.to_string_lossy().into_owned());
    match fs::rename(path, &bad) {
        Ok(()) => format!("cannot read {}: {reason}; moved it to {bad_name}", path.display()),
        Err(error) => format!("cannot read {}: {reason}; cannot move it to {bad_name}: {error}", path.display()),
    }
}

/// Numbers the staging files of this process, so two saves never write the same one.
static STAGING: AtomicU64 = AtomicU64::new(0);

/// Writes `state` to `path`, creating its folder; replaces the file atomically. Each save stages in a file of
/// its own (`state.toml.<pid>-<n>.tmp`), so saves of two IVEs, or of one IVE twice, cannot mix.
pub fn save(state: &SavedState, path: &Path) -> Result<(), StateError> {
    let failed = |reason: String| StateError { path: path.to_path_buf(), reason };
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|error| failed(error.to_string()))?;
    }
    let text = toml::to_string(state).map_err(|error| failed(error.to_string()))?;
    let staging = path.with_extension(format!("toml.{}-{}.tmp", std::process::id(), STAGING.fetch_add(1, Ordering::Relaxed)));
    fs::write(&staging, text).map_err(|error| failed(error.to_string()))?;
    fs::rename(&staging, path).map_err(|error| {
        // a staging file left behind would never be cleaned up
        let _ = fs::remove_file(&staging);
        failed(error.to_string())
    })
}

/// Writes one state file for the writes one IVE takes in turn (the coalesced background write, the flush at
/// quit): one at a time, and never one taken before the write already done.
#[derive(Debug, Clone, Default)]
pub struct Writer {
    /// number of the last write done; held while writing
    done: Arc<Mutex<u64>>,
}

impl Writer {
    /// Saves `state` as write `number` (numbers count up as writes are taken); skipped when a later-taken
    /// write is already on disk.
    pub fn write(&self, number: u64, state: &SavedState, path: &Path) -> Result<(), StateError> {
        let mut done = self.done.lock().unwrap_or_else(PoisonError::into_inner);
        if number <= *done {
            return Ok(());
        }
        save(state, path)?;
        *done = number;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateError {
    pub path: PathBuf,
    pub reason: String,
}

impl fmt::Display for StateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "cannot save {}: {}", self.path.display(), self.reason)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::window_state::WindowMode;

    fn project(path: &str) -> SavedProject {
        SavedProject {
            path: PathBuf::from(path),
            id: "p".to_owned(),
            expanded: false,
            expanded_checkouts: Vec::new(),
            last_checkout: None,
            folders: Vec::new(),
            hidden_worktrees: Vec::new(),
        }
    }

    fn sample() -> SavedState {
        SavedState {
            version: VERSION,
            shown_folders: None,
            hidden_projects: Vec::new(),
            projects: vec![SavedProject {
                path: PathBuf::from("/mnt/data/Projects/atlassian"),
                id: "atlassian".to_owned(),
                expanded: true,
                expanded_checkouts: vec![PathBuf::from("/mnt/data/Projects/atlassian/.claude/worktrees/GEN-817-ive")],
                last_checkout: Some(PathBuf::from("/mnt/data/Projects/atlassian/.claude/worktrees/GEN-817-ive")),
                folders: vec![SavedFolders {
                    checkout: PathBuf::from("/mnt/data/Projects/atlassian/.claude/worktrees/GEN-817-ive"),
                    shown: vec![FolderKind::Processes, FolderKind::Commits],
                    files_hidden: true,
                    folded: vec![FolderKind::Files],
                    unfolded: vec![FolderKind::Processes],
                    open_commits: vec!["ab".repeat(20)],
                    commits_shown: Some(30),
                    files: vec![PathBuf::from("/mnt/data/Projects/atlassian/.claude/worktrees/GEN-817-ive/src")],
                    hidden_groups: vec![PathBuf::from("/mnt/data/Projects/atlassian/.claude/worktrees/GEN-817-ive")],
                    change_folders: vec![SavedChangeFolder { list: "since-base".to_owned(), path: "src/ui".to_owned(), open: false }],
                }],
                hidden_worktrees: vec![PathBuf::from("/mnt/data/Projects/atlassian/.claude/worktrees/GEN-811")],
            }],
            recent: vec![RecentProject { path: PathBuf::from("/home/kb/Documents/notes"), id: "notes".to_owned() }],
            tabs: None,
            window: Some(SavedWindow { x: 1950., y: 40., width: 1000., height: 700., mode: WindowMode::Maximized }),
            history: Vec::new(),
            worktree_use: Vec::new(),
            terminals: Vec::new(),
        }
    }

    #[test]
    fn should_save_and_load_projects_and_recent() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ive").join("state.toml");

        // when
        save(&sample(), &path).unwrap();
        let loaded = load(&path);

        // then
        assert_eq!((loaded.state, loaded.problem), (sample(), None));
        assert!(fs::read_to_string(&path).unwrap().starts_with("version = 2\n"));
    }

    #[test]
    fn should_round_trip_saved_terminals() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.toml");
        let terminal = SavedTerminal { session: "s1".to_owned(), project: "ive".to_owned(), checkout: PathBuf::from("/r/wt"), pinned: Some(PathBuf::from("/r/src")) };
        let state = SavedState { terminals: vec![terminal], ..sample() };

        // when
        save(&state, &path).unwrap();
        let loaded = load(&path);

        // then
        assert_eq!((loaded.state, loaded.problem), (state, None));
    }

    #[test]
    fn should_set_aside_relative_terminal_checkout() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.toml");
        let terminal = SavedTerminal { session: "s1".to_owned(), project: "ive".to_owned(), checkout: PathBuf::from("r/wt"), pinned: None };
        save(&SavedState { terminals: vec![terminal], ..sample() }, &path).unwrap();

        // when
        let loaded = load(&path);

        // then
        assert!(loaded.problem.is_some_and(|problem| problem.contains("r/wt")));
    }

    #[test]
    fn should_set_aside_a_version_1_file() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.toml");
        fs::write(&path, "version = 1\n").unwrap();

        // when
        let loaded = load(&path);

        // then
        assert_eq!(loaded.state, SavedState::default());
        assert!(loaded.problem.is_some_and(|problem| problem.contains("version 1")));
    }

    #[test]
    fn should_start_empty_without_a_file() {
        // given
        let dir = tempfile::tempdir().unwrap();

        // when
        let loaded = load(&dir.path().join("state.toml"));

        // then
        assert_eq!((loaded.state, loaded.problem), (SavedState::default(), None));
    }

    #[test]
    fn should_move_unparsable_file_aside_and_say_why() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.toml");
        fs::write(&path, "version = [").unwrap();

        // when
        let loaded = load(&path);

        // then
        let problem = loaded.problem.unwrap();
        assert!(problem.starts_with(&format!("cannot read {}: ", path.display())), "{problem}");
        assert!(problem.ends_with("; moved it to state.toml.bad"), "{problem}");
        assert_eq!((path.exists(), dir.path().join("state.toml.bad").exists()), (false, true));
        assert_eq!(loaded.state, SavedState::default());
    }

    #[test]
    fn should_set_aside_other_versions_oversized_files_and_unsafe_ids() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let files = [
            "version = 3\n".to_owned(),
            format!("version = 2\n# {}\n", "x".repeat(600 * 1024)),
            "version = 2\n[[projects]]\npath = \"/a\"\nid = \"../../etc\"\n".to_owned(),
            "version = 2\n[[projects]]\npath = \"/a\"\nid = \"a\"\n[[recent]]\npath = \"/b\"\nid = \"a\"\n".to_owned(),
        ];

        // when
        let problems: Vec<_> = files
            .iter()
            .enumerate()
            .map(|(index, text)| {
                let path = dir.path().join(format!("{index}.toml"));
                fs::write(&path, text).unwrap();
                load(&path).problem.unwrap_or_default()
            })
            .collect();

        // then
        assert!(problems[0].contains("version 3 is not supported"), "{}", problems[0]);
        assert!(problems[1].contains("the limit is 524288"), "{}", problems[1]);
        assert!(problems[2].contains("invalid project id"), "{}", problems[2]);
        assert!(problems[3].contains("used twice"), "{}", problems[3]);
    }

    #[test]
    fn should_set_aside_relative_checkout_paths() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.toml");
        let mut state = sample();
        state.projects[0].last_checkout = Some(PathBuf::from("worktrees/x"));
        save(&state, &path).unwrap();

        // when
        let loaded = load(&path);

        // then
        assert_eq!(loaded.problem.map(|problem| problem.contains("path worktrees/x is not absolute")), Some(true));
    }

    #[test]
    fn should_set_aside_relative_folder_checkout_paths() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.toml");
        let mut state = sample();
        state.projects[0].folders[0].checkout = PathBuf::from("worktrees/y");
        save(&state, &path).unwrap();

        // when
        let loaded = load(&path);

        // then
        assert_eq!(loaded.problem.map(|problem| problem.contains("path worktrees/y is not absolute")), Some(true));
    }

    #[test]
    fn should_read_a_project_saved_without_folders() {
        // given
        let text = "version = 2\n\n[[projects]]\npath = \"/p\"\nid = \"p\"\n";

        // when
        let state: SavedState = toml::from_str(text).unwrap();

        // then
        assert!(state.projects[0].folders.is_empty());
    }

    #[test]
    fn should_never_replace_a_later_write_with_an_earlier_one() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.toml");
        let writer = Writer::default();
        let later = SavedState { recent: Vec::new(), ..sample() };

        // when
        writer.write(2, &later, &path).unwrap();
        writer.write(1, &sample(), &path).unwrap();

        // then
        assert_eq!(load(&path).state, later);
    }

    #[test]
    fn should_keep_the_file_whole_when_two_writers_save_at_once() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.toml");
        let states = [sample(), SavedState { recent: Vec::new(), ..sample() }];

        // when
        std::thread::scope(|scope| {
            for state in &states {
                scope.spawn(|| (0..20).for_each(|_| save(state, &path).unwrap()));
            }
        });

        // then
        let loaded = load(&path);
        assert_eq!(loaded.problem, None);
        assert!(states.contains(&loaded.state));
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1, "no staging file left");
    }

    #[test]
    fn should_name_the_folders_that_hold_data_of_an_id() {
        // given
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("notes")).unwrap();
        fs::write(dir.path().join("stray.txt"), "").unwrap();

        // when
        let names = folder_names(dir.path());

        // then
        assert_eq!((names, folder_names(&dir.path().join("missing"))), (vec!["notes".to_owned()], Vec::new()));
    }

    fn shown(number: usize) -> SavedShown {
        SavedShown { shown_at: number as i64, checkout: Some(PathBuf::from("/w")), tab: SavedTab::Settings { page: "appearance".to_owned() } }
    }

    #[test]
    fn should_round_trip_history_and_worktree_use() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.toml");
        let state = SavedState {
            history: vec![shown(1), shown(2)],
            worktree_use: vec![SavedUse { checkout: PathBuf::from("/w"), used_at: 5 }],
            ..sample()
        };

        // when
        save(&state, &path).unwrap();
        let loaded = load(&path);

        // then
        assert_eq!((loaded.state, loaded.problem), (state, None));
    }

    #[test]
    fn should_round_trip_shown_folders_and_hidden_projects() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.toml");
        let state = SavedState { shown_folders: Some(vec![FolderKind::Processes, FolderKind::Commits]), hidden_projects: vec!["notes".to_owned()], ..sample() };

        // when
        save(&state, &path).unwrap();
        let loaded = load(&path);

        // then
        assert_eq!((loaded.state, loaded.problem), (state, None));
    }

    #[test]
    fn should_load_a_state_without_shown_folders_or_hidden_projects() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.toml");
        fs::write(&path, "version = 2\n").unwrap();

        // when
        let loaded = load(&path);

        // then
        assert_eq!((loaded.state.shown_folders, loaded.state.hidden_projects, loaded.problem), (None, Vec::<String>::new(), None));
    }

    #[test]
    fn should_load_a_state_without_history() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.toml");
        fs::write(&path, "version = 2\n").unwrap();

        // when
        let loaded = load(&path);

        // then
        assert_eq!((loaded.state.history, loaded.state.worktree_use, loaded.problem), (Vec::new(), Vec::new(), None));
    }

    #[test]
    fn should_reject_too_many_history_entries() {
        // given
        let state = SavedState { history: (0..=MAX_HISTORY).map(shown).collect(), ..SavedState::default() };

        // when
        let error = state.validate().unwrap_err();

        // then
        assert!(error.contains("501") && error.contains("500"), "{error}");
    }

    #[test]
    fn should_reject_a_relative_worktree_use_path() {
        // given
        let state = SavedState { worktree_use: vec![SavedUse { checkout: PathBuf::from("w"), used_at: 1 }], ..SavedState::default() };

        // when
        let error = state.validate().unwrap_err();

        // then
        assert!(error.contains("path w is not absolute"), "{error}");
    }

    #[test]
    fn should_reject_a_relative_hidden_worktree() {
        // given
        let project = SavedProject { hidden_worktrees: vec![PathBuf::from("wt")], ..project("/p") };
        let state = SavedState { projects: vec![project], ..SavedState::default() };

        // when
        let error = state.validate().unwrap_err();

        // then
        assert!(error.contains("path wt is not absolute"), "{error}");
    }

    #[test]
    fn should_set_aside_more_projects_than_the_limit() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.toml");
        let projects = (0..=MAX_PROJECTS).map(|n| SavedProject { id: format!("p{n}"), ..project(&format!("/p/{n}")) });
        save(&SavedState { projects: projects.collect(), ..SavedState::default() }, &path).unwrap();

        // when
        let loaded = load(&path);

        // then
        assert!(loaded.problem.unwrap().contains("the limits are 100 and 10"));
    }
}
