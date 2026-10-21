//! A worktree's folders in the tree (Processes, Uncommitted, Since base, Commits, Files), their badges, and the rows of
//! the three git folders; pure, so it is tested without gpui.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::LazyLock;

use serde::{Deserialize, Serialize};

use super::commit_label::{commit_label, shared_prefix};
use super::rows::{TreeItem, TreeRow};
use crate::age::Clock;
use crate::git::log::CommitInfo;
use crate::git::status::Change;
use crate::git::types::Sha;
use crate::git::worktrees::BaseBranch;
use crate::icons;
use crate::overview::file_list::{self, FileEntry, FileLayout, FileOrder, FileSort, Modified};
use crate::git::worktree_data::WorktreeData;
use crate::project::CheckoutRef;

/// The Commits folder lists this many at first, and this many more on each Load more.
pub const COMMITS_SHOWN: usize = 10;
const COMMITS_MORE: usize = 20;
/// The tree layout of a list of changed files opens the folders the user left alone while it shows at most this
/// many rows.
const CHANGE_ROWS: usize = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FolderKind {
    Processes,
    Uncommitted,
    SinceBase,
    Commits,
    Files,
}

impl FolderKind {
    /// Its part of click target names: `folder:atlassian/dev/since-base`.
    pub fn name(self) -> &'static str {
        match self {
            Self::Processes => "processes",
            Self::Uncommitted => "uncommitted",
            Self::SinceBase => "since-base",
            Self::Commits => "commits",
            Self::Files => "files",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::Processes => "Processes",
            Self::Uncommitted => "Uncommitted",
            Self::SinceBase => "Since base",
            Self::Commits => "Commits",
            Self::Files => "Files",
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            Self::Processes => icons::TERMINAL,
            Self::Uncommitted => icons::CHANGES,
            Self::SinceBase => icons::HISTORY,
            Self::Commits => icons::COMMIT,
            Self::Files => icons::PROJECT_FOLDER,
        }
    }
}

/// What the tree shows of a checkout's folders: which are switched on by their badges, which are folded, and inside
/// the git folders the change folders the user opened or closed and the open commits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckoutFolders {
    pub shown: BTreeSet<FolderKind>,
    pub folded: BTreeSet<FolderKind>,
    /// `true`: open; the others open while [`CHANGE_ROWS`] allows
    pub change_folders: HashMap<(ChangeSource, String), bool>,
    pub open_commits: HashSet<Sha>,
    /// how many commits Commits lists; Load more adds [`COMMITS_MORE`]
    pub commits_shown: usize,
}

impl Default for CheckoutFolders {
    fn default() -> Self {
        Self {
            shown: BTreeSet::from([FolderKind::Processes, FolderKind::Files]),
            folded: BTreeSet::new(),
            change_folders: HashMap::new(),
            open_commits: HashSet::new(),
            commits_shown: COMMITS_SHOWN,
        }
    }
}

/// The folders of a checkout nobody changed.
pub static DEFAULT_FOLDERS: LazyLock<CheckoutFolders> = LazyLock::new(CheckoutFolders::default);

impl CheckoutFolders {
    /// Whether `state.toml` has nothing to keep for it.
    pub fn is_default_saved(&self) -> bool {
        let git_default = self.change_folders.is_empty() && self.open_commits.is_empty() && self.commits_shown == COMMITS_SHOWN;
        self.shown == DEFAULT_FOLDERS.shown && self.folded.is_empty() && git_default
    }

    pub fn is_shown(&self, kind: FolderKind) -> bool {
        self.shown.contains(&kind)
    }

    pub fn is_expanded(&self, kind: FolderKind) -> bool {
        !self.folded.contains(&kind)
    }

    pub fn toggle_shown(&mut self, kind: FolderKind) {
        if self.shown.remove(&kind) {
            return;
        }
        self.shown.insert(kind);
    }

    pub fn toggle_folded(&mut self, kind: FolderKind) {
        if self.folded.remove(&kind) {
            return;
        }
        self.folded.insert(kind);
    }

    /// Keeps change folder `path` of `source` open or closed, whatever room the list has.
    pub fn set_change_folder(&mut self, source: ChangeSource, path: String, open: bool) {
        self.change_folders.insert((source, path), open);
    }

    pub fn show_more_commits(&mut self) {
        self.commits_shown += COMMITS_MORE;
    }

    /// Whether the commit is open now.
    pub fn toggle_commit(&mut self, sha: Sha) -> bool {
        if self.open_commits.remove(&sha) {
            return false;
        }
        self.open_commits.insert(sha);
        true
    }

    /// Whether a git folder is switched on, empty or not: its data is kept fresh while it shows.
    pub fn shows_git(&self) -> bool {
        [FolderKind::Uncommitted, FolderKind::SinceBase, FolderKind::Commits].iter().any(|kind| self.shown.contains(kind))
    }

    /// Forgets the open commits and change folders `git` no longer lists, so `state.toml` does not keep them; whether
    /// any were.
    pub fn forget_unlisted(&mut self, git: &CheckoutGit) -> bool {
        let listed: HashSet<&Sha> = git.since.iter().flat_map(|since| since.commits.iter().map(|commit| &commit.sha)).collect();
        let before = (self.open_commits.len(), self.change_folders.len());
        self.open_commits.retain(|sha| listed.contains(sha));
        self.change_folders.retain(|(source, path), _| {
            let files = match source {
                ChangeSource::Uncommitted => Some(&git.changes),
                ChangeSource::SinceBase => git.since.as_ref().map(|since| &since.files),
                // files not read yet keep their folders
                ChangeSource::Commit(sha) if listed.contains(sha) => match git.commit_files.get(sha) {
                    Some(Ok(files)) => Some(files),
                    _ => return true,
                },
                ChangeSource::Commit(_) => None,
            };
            files.is_some_and(|files| files.iter().any(|change| change.path.display().starts_with(&format!("{path}/"))))
        });
        before != (self.open_commits.len(), self.change_folders.len())
    }

    fn toggled_in(&self, source: &ChangeSource) -> HashMap<String, bool> {
        self.change_folders.iter().filter(|((at, _), _)| at == source).map(|((_, path), open)| (path.clone(), *open)).collect()
    }
}

/// What a checkout's git folders list, once read: a snapshot of its Git section's data.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CheckoutGit {
    pub changes: Vec<Change>,
    /// `None` without a base branch or a fork point
    pub since: Option<SinceGit>,
    /// of the commits opened in the tree; kept while listed
    pub commit_files: HashMap<Sha, Result<Vec<Change>, String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SinceGit {
    pub base: BaseBranch,
    pub fork: Sha,
    /// committed and uncommitted
    pub files: Vec<Change>,
    /// newest first
    pub commits: Vec<CommitInfo>,
    /// the branch has more commits than were read
    pub more: bool,
}

impl CheckoutGit {
    pub fn of(data: &WorktreeData) -> Self {
        let since = data.since.as_ref().ok().map(|since| SinceGit {
            base: since.base.clone(),
            fork: since.fork.clone(),
            files: since.with_uncommitted.clone(),
            commits: since.history.commits.clone(),
            more: since.history.more,
        });
        Self { changes: data.changes.clone(), since, commit_files: HashMap::new() }
    }

    /// This snapshot with the files of `previous`'s commits that are still listed.
    pub fn keeping_commit_files(mut self, previous: Option<&CheckoutGit>) -> Self {
        let listed: HashSet<&Sha> = self.since.iter().flat_map(|since| since.commits.iter().map(|commit| &commit.sha)).collect();
        if let Some(previous) = previous {
            self.commit_files = previous.commit_files.iter().filter(|(sha, _)| listed.contains(sha)).map(|(sha, files)| (sha.clone(), files.clone())).collect();
        }
        self
    }
}

/// Where a changed file's diff comes from.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ChangeSource {
    Uncommitted,
    SinceBase,
    Commit(Sha),
}

impl ChangeSource {
    /// Its part of click target names: `uncommitted`, `since-base`, a short sha.
    pub fn name(&self) -> String {
        match self {
            Self::Uncommitted => "uncommitted".to_owned(),
            Self::SinceBase => "since-base".to_owned(),
            Self::Commit(sha) => sha.short().to_owned(),
        }
    }

    /// How `state.toml` names it: `uncommitted`, `since-base`, a full sha.
    pub fn key(&self) -> &str {
        match self {
            Self::Uncommitted => "uncommitted",
            Self::SinceBase => "since-base",
            Self::Commit(sha) => sha.as_str(),
        }
    }

    pub fn parse(key: &str) -> Option<Self> {
        match key {
            "uncommitted" => Some(Self::Uncommitted),
            "since-base" => Some(Self::SinceBase),
            sha => Sha::parse(sha).map(Self::Commit),
        }
    }
}

/// A badge at a checkout row's end: switches its folder on and off.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderBadge {
    pub kind: FolderKind,
    /// `None` until the git data is read
    pub count: Option<usize>,
    pub shown: bool,
    /// `<checkout target>/<kind>`
    pub target: String,
}

/// A worktree folder's row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderRow {
    pub checkout: CheckoutRef,
    pub kind: FolderKind,
    /// `None` on Files, and on a git folder until its data is read
    pub count: Option<usize>,
    pub expanded: bool,
    /// an empty git folder: no chevron
    pub empty: bool,
    /// why a git folder lists nothing, once read
    pub nothing: Option<Nothing>,
}

/// An empty git folder's row: muted, `· 0` or `· no base` after its name, and why in its tooltip.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nothing {
    pub mark: &'static str,
    pub tip: String,
}

/// A changed file under Uncommitted, Since base or a commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeRow {
    pub checkout: CheckoutRef,
    pub source: ChangeSource,
    pub change: Change,
    /// in the flat layout, its folder, shown muted after the name
    pub folder: Option<String>,
}

/// A folder of changed files in the tree layout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeFolderRow {
    pub checkout: CheckoutRef,
    pub source: ChangeSource,
    pub path: String,
    pub label: String,
    pub files: usize,
    pub expanded: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitRow {
    pub checkout: CheckoutRef,
    pub sha: Sha,
    pub label: String,
    /// `<short sha> · <age>`
    pub meta: String,
    /// `<short sha> <subject>`
    pub hint: String,
    pub expanded: bool,
}

/// `Load 20 more · 10 of 32` under the listed commits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitsMoreRow {
    pub checkout: CheckoutRef,
    pub text: String,
}

/// A muted line under an open commit whose files could not be read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteRow {
    pub checkout: CheckoutRef,
    pub sha: Sha,
    pub text: String,
}

/// What the git folders of one checkout are built from.
pub struct GitFolders<'a> {
    pub checkout: &'a CheckoutRef,
    /// the checkout's click target name
    pub target: &'a str,
    pub folders: &'a CheckoutFolders,
    /// `None` until read
    pub git: Option<&'a CheckoutGit>,
    pub layout: FileLayout,
    pub clock: Clock,
}

impl GitFolders<'_> {
    /// A badge per folder, in order, empty or not, so each can be switched on; a git folder's count is `None` until
    /// read and without a base, Files' always. `processes` counts Processes' rows.
    pub fn badges(&self, processes: usize) -> Vec<FolderBadge> {
        let badge = |kind: FolderKind, count: Option<usize>| FolderBadge {
            kind,
            count,
            shown: self.folders.is_shown(kind),
            target: format!("{}/{}", self.target, kind.name()),
        };
        let git = [FolderKind::Uncommitted, FolderKind::SinceBase, FolderKind::Commits].map(|kind| badge(kind, self.git.and_then(|git| count_of(git, kind))));
        std::iter::once(badge(FolderKind::Processes, Some(processes))).chain(git).chain([badge(FolderKind::Files, None)]).collect()
    }

    /// Why git folder `kind` lists nothing; `None` while it lists something or is not read.
    pub fn nothing(&self, kind: FolderKind) -> Option<Nothing> {
        let git = self.git?;
        let base = git.since.as_ref().map(|since| since.base.name.as_str());
        let (count, base) = match (kind, base) {
            (FolderKind::Uncommitted, _) => return (git.changes.is_empty()).then(|| Nothing { mark: "0", tip: "Nothing uncommitted".to_owned() }),
            (FolderKind::SinceBase | FolderKind::Commits, None) => return Some(Nothing { mark: "no base", tip: "No base branch to compare with".to_owned() }),
            (FolderKind::SinceBase | FolderKind::Commits, Some(base)) => (count_of(git, kind), base),
            (FolderKind::Processes | FolderKind::Files, _) => return None,
        };
        let tip = match kind {
            FolderKind::SinceBase => format!("No changes since {base}"),
            _ => format!("No commits since {base}"),
        };
        (count == Some(0)).then_some(Nothing { mark: "0", tip })
    }

    /// The rows in git folder `kind`, one level below it at `depth`.
    pub fn rows(&self, kind: FolderKind, depth: usize) -> Vec<TreeRow> {
        let Some(git) = self.git else {
            return Vec::new();
        };
        match (kind, &git.since) {
            (FolderKind::Uncommitted, _) => self.change_rows(&git.changes, &ChangeSource::Uncommitted, depth),
            (FolderKind::SinceBase, Some(since)) => self.change_rows(&since.files, &ChangeSource::SinceBase, depth),
            (FolderKind::Commits, Some(since)) => self.commit_rows(git, since, depth),
            _ => Vec::new(),
        }
    }

    fn change_rows(&self, files: &[Change], source: &ChangeSource, depth: usize) -> Vec<TreeRow> {
        let order_by = FileOrder { sort: FileSort::Name, layout: self.layout };
        let order = file_list::sorted(files, &Modified::new(), FileSort::Name);
        let collapsed = file_list::collapsed_within(files, &order, order_by, &self.folders.toggled_in(source), CHANGE_ROWS);
        let entries = file_list::entries(files, &order, order_by, &collapsed);
        let target = |path: String| format!("{}/{}:{path}", self.target, source.name());
        entries
            .into_iter()
            .map(|entry| match entry {
                FileEntry::File { index, depth: level } => {
                    let change = files[index].clone();
                    let folder = Some(change.path.folder()).filter(|folder| level.is_none() && !folder.is_empty());
                    let row_target = target(change.path.display());
                    let item = TreeItem::Change(ChangeRow { checkout: self.checkout.clone(), source: source.clone(), change, folder });
                    TreeRow { depth: depth + level.unwrap_or(0), item, target: row_target }
                }
                FileEntry::Folder(folder) => {
                    let row_target = target(format!("{}/", folder.path));
                    let item = TreeItem::ChangeFolder(ChangeFolderRow {
                        checkout: self.checkout.clone(),
                        source: source.clone(),
                        path: folder.path,
                        label: folder.label,
                        files: folder.files,
                        expanded: folder.open,
                    });
                    TreeRow { depth: depth + folder.depth, item, target: row_target }
                }
            })
            .collect()
    }

    fn commit_rows(&self, git: &CheckoutGit, since: &SinceGit, depth: usize) -> Vec<TreeRow> {
        let subjects: Vec<&str> = since.commits.iter().map(|commit| commit.subject.as_str()).collect();
        let shared = shared_prefix(&subjects);
        let mut rows = Vec::new();
        let shown = self.folders.commits_shown.min(since.commits.len());
        for commit in &since.commits[..shown] {
            let expanded = self.folders.open_commits.contains(&commit.sha);
            let short = commit.sha.short();
            let item = TreeItem::Commit(CommitRow {
                checkout: self.checkout.clone(),
                sha: commit.sha.clone(),
                label: commit_label(&commit.subject, shared).to_owned(),
                meta: format!("{short} · {}", self.clock.age(commit.when.seconds)),
                hint: format!("{short} {}", commit.subject),
                expanded,
            });
            rows.push(TreeRow { depth, item, target: format!("{}/{short}", self.target) });
            if expanded {
                rows.extend(self.commit_file_rows(git, &commit.sha, depth + 1));
            }
        }
        let rest = since.commits.len() - shown;
        if rest > 0 {
            let plus = if since.more { "+" } else { "" };
            let text = format!("Load {} more · {shown} of {}{plus}", rest.min(COMMITS_MORE), since.commits.len());
            let item = TreeItem::CommitsMore(CommitsMoreRow { checkout: self.checkout.clone(), text });
            rows.push(TreeRow { depth, item, target: self.target.to_owned() });
        }
        rows
    }

    /// An open commit's files, nothing while they are read, or why they could not be.
    fn commit_file_rows(&self, git: &CheckoutGit, sha: &Sha, depth: usize) -> Vec<TreeRow> {
        match git.commit_files.get(sha) {
            None => Vec::new(),
            Some(Ok(files)) => self.change_rows(files, &ChangeSource::Commit(sha.clone()), depth),
            Some(Err(message)) => {
                let item = TreeItem::Note(NoteRow { checkout: self.checkout.clone(), sha: sha.clone(), text: message.clone() });
                vec![TreeRow { depth, item, target: String::new() }]
            }
        }
    }
}

/// How many rows git folder `kind` lists; `None` without a base.
fn count_of(git: &CheckoutGit, kind: FolderKind) -> Option<usize> {
    match (kind, &git.since) {
        (FolderKind::Uncommitted, _) => Some(git.changes.len()),
        (FolderKind::SinceBase, Some(since)) => Some(since.files.len()),
        (FolderKind::Commits, Some(since)) => Some(since.commits.len()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_show_git_while_a_git_folder_is_switched_on() {
        // given
        let mut folders = CheckoutFolders::default();
        folders.toggle_shown(FolderKind::SinceBase);

        // when
        let shows = [folders.shows_git(), CheckoutFolders::default().shows_git()];

        // then
        assert_eq!(shows, [true, false]);
    }

    #[test]
    fn should_forget_open_commits_and_change_folders_no_longer_listed() {
        // given
        let (kept, gone) = (Sha::parse(&"ab".repeat(20)).unwrap(), Sha::parse(&"cd".repeat(20)).unwrap());
        let mut folders = CheckoutFolders::default();
        folders.toggle_commit(kept.clone());
        folders.toggle_commit(gone.clone());
        folders.set_change_folder(ChangeSource::Uncommitted, "src".to_owned(), false);
        folders.set_change_folder(ChangeSource::Uncommitted, "old".to_owned(), false);
        folders.set_change_folder(ChangeSource::Commit(kept.clone()), "docs".to_owned(), true);
        folders.set_change_folder(ChangeSource::Commit(gone), "docs".to_owned(), true);
        let file = |path: &str| Change { mark: crate::git::types::Mark::Modified, path: crate::git::types::GitPath::from_bytes(path.as_bytes()), old_path: None };
        let commit = CommitInfo { sha: kept.clone(), subject: String::new(), author: String::new(), when: crate::age::When { seconds: 0, offset_minutes: 0 }, parents: Vec::new() };
        let base = BaseBranch { name: "dev".to_owned(), reference: "refs/heads/dev".to_owned() };
        let since = SinceGit { base, fork: kept.clone(), files: Vec::new(), commits: vec![commit], more: false };
        let git = CheckoutGit { changes: vec![file("src/a.rs")], since: Some(since), commit_files: HashMap::new() };

        // when
        let forgot = folders.forget_unlisted(&git);

        // then
        assert!(forgot);
        assert_eq!(folders.open_commits, HashSet::from([kept.clone()]));
        let mut left: Vec<String> = folders.change_folders.keys().map(|(source, path)| format!("{}:{path}", source.name())).collect();
        left.sort();
        assert_eq!(left, [format!("{}:docs", kept.short()), "uncommitted:src".to_owned()]);
    }
}
