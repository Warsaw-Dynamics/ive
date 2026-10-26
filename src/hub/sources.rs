//! What the hub lists, as plain data the workspace gathers each time the list is drawn: no gpui, so the model is
//! tested alone.

use std::ops::Range;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::output::OutputHit;
use super::files::Class;
use crate::git::types::Mark;
use crate::process::ProcessId;
use crate::project::{CheckoutRef, ProjectId};
use crate::saved_tabs::SavedTab;
use crate::scratch::ScratchRef;
use crate::settings::ThemePreference;
use crate::tabs::TabId;

#[derive(Debug, Clone, PartialEq)]
pub struct WorktreeInfo {
    pub checkout: CheckoutRef,
    pub label: String,
    pub branch: Option<String>,
    /// the worktree card's tags: changes, ahead/behind, running
    pub tags: Vec<String>,
    /// seconds since the epoch; `None` when never used
    pub last_used: Option<i64>,
    pub running: usize,
}

/// An open project; its worktrees in the project's order, which is also their bit in the corpus' masks.
#[derive(Debug, Clone, PartialEq)]
pub struct ProjectInfo {
    pub id: ProjectId,
    pub name: String,
    /// false for a plain folder: its one worktree is the folder
    pub git: bool,
    pub worktrees: Vec<WorktreeInfo>,
}

/// A branch without a worktree, as New worktree would check it out.
#[derive(Debug, Clone, PartialEq)]
pub struct FreeBranch {
    pub project: ProjectId,
    /// the local name a worktree of it checks out: `feature/x`
    pub local: String,
    /// as git prints it: `feature/x`, `origin/feature/x`
    pub shown: String,
    /// the remote of a branch only a remote has
    pub remote: Option<String>,
    /// its last commit, seconds since the epoch
    pub when: i64,
}

/// What a row opens.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Target {
    /// an open tab
    Tab(TabId),
    /// a closed tab of the history
    Reopen(SavedTab),
    Process(ProcessId),
    /// a project file by its project-relative path
    File { project: ProjectId, path: String },
    Scratch(ScratchRef),
    /// an uncommitted change in one checkout
    Change { checkout: CheckoutRef, path: String },
    /// a configuration of the `.run.toml` in project-relative `folder`
    RunConfig { project: ProjectId, folder: String, key: String },
    Action(HubAction),
    Worktree(CheckoutRef),
    /// a branch without a worktree
    Branch { project: ProjectId, name: String },
    /// new terminal mode: a terminal in this checkout
    NewTerminal(CheckoutRef),
    /// new terminal mode: a worktree of the branch (`remote`: only that remote has it), then a terminal in it
    NewWorktreeTerminal { project: ProjectId, branch: String, remote: Option<String> },
    Scope(HubScope),
    /// a line of the tab shown before the hub
    Line(u32),
    /// a contents search's line: an absolute path, its line from 1, and its first match's bytes in that line
    Hit { path: PathBuf, line: u32, bytes: Range<usize> },
    /// a contents search's line of a process's output
    Output(Box<OutputHit>),
}

/// The `>` list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HubAction {
    NewTerminal,
    NewScratch,
    GitLog,
    ProjectOverview,
    CloseOtherTabs,
    Settings,
    Theme(ThemePreference),
}

/// What the hub covers.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum HubScope {
    #[default]
    All,
    Project(ProjectId),
    Worktree(CheckoutRef),
}

/// Where a row's thing is.
#[derive(Debug, Clone, PartialEq)]
pub enum Place {
    /// no worktree: a scratch, Settings, an action
    Nowhere,
    One(CheckoutRef),
    /// in several of a project's checkouts: Enter asks which
    Several { project: ProjectId, checkouts: Vec<PathBuf> },
}

/// One row before matching.
#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    pub target: Target,
    pub title: String,
    pub detail: String,
    pub icon: &'static str,
    /// `● open`, `running`, a run's status
    pub state: Option<String>,
    pub place: Place,
    /// the project it belongs to, also for rows of no worktree (scratches, commits)
    pub project: Option<ProjectId>,
    /// seconds since the epoch, for Recent and worktree rows
    pub age: Option<i64>,
    /// a closed Recent row
    pub greyed: bool,
    /// its icon drawn faded: a branch only a remote has
    pub faded: bool,
    pub mark: Option<Mark>,
    pub class: Class,
    /// project-relative, for Ctrl+Y and Alt+Enter
    pub path: Option<String>,
}

impl Item {
    pub fn new(target: Target, title: impl Into<String>, icon: &'static str) -> Item {
        Item {
            target,
            title: title.into(),
            detail: String::new(),
            icon,
            state: None,
            place: Place::Nowhere,
            project: None,
            age: None,
            greyed: false,
            faded: false,
            mark: None,
            class: Class::Searched,
            path: None,
        }
    }
}

/// The tab shown before the hub.
#[derive(Debug, Clone, PartialEq)]
pub struct BackTab {
    pub tab: TabId,
    pub project: Option<ProjectId>,
    pub checkout: Option<CheckoutRef>,
    /// a file or a scratch, which `:42` can scroll
    pub is_text: bool,
}

/// Everything but the files, which the search finds.
#[derive(Debug, Clone, Default)]
pub struct Sources {
    /// in the corpus' order
    pub projects: Vec<ProjectInfo>,
    /// newest first
    pub recent: Vec<Item>,
    pub changed: Vec<Item>,
    /// processes that called for the user, longest waiting first
    pub waiting: Vec<Item>,
    pub running: Vec<Item>,
    pub processes: Vec<Item>,
    pub configs: Vec<Item>,
    /// newest commit first
    pub branches: Vec<FreeBranch>,
    pub actions: Vec<Item>,
    pub back: Option<BackTab>,
    /// a project's file list is still being read
    pub loading: bool,
    /// the open projects the tree header's picker left out; never all of them
    pub hidden: Vec<ProjectId>,
}

impl Sources {
    /// Whether rows of `project` are listed: those of no project always are.
    pub fn shows(&self, project: Option<&ProjectId>) -> bool {
        project.is_none_or(|project| !self.hidden.contains(project))
    }

    /// These sources without the rows of the projects left out.
    pub fn without_hidden(mut self) -> Self {
        let hidden = std::mem::take(&mut self.hidden);
        let shows = |item: &Item| item.project.as_ref().is_none_or(|project| !hidden.contains(project));
        for items in [&mut self.recent, &mut self.changed, &mut self.waiting, &mut self.running, &mut self.processes, &mut self.configs] {
            items.retain(shows);
        }
        self.branches.retain(|branch| !hidden.contains(&branch.project));
        self.hidden = hidden;
        self
    }

    /// The projects listed, in the corpus' order.
    pub fn shown_projects(&self) -> impl Iterator<Item = &ProjectInfo> {
        self.projects.iter().filter(|project| self.shows(Some(&project.id)))
    }

    /// The one project listed, when the others are left out.
    pub fn only_shown(&self) -> Option<&ProjectInfo> {
        let mut shown = self.shown_projects();
        match (self.hidden.is_empty(), shown.next(), shown.next()) {
            (false, Some(one), None) => Some(one),
            _ => None,
        }
    }

    /// What the All scope covers: `All projects`, the one project listed, or `2 of 4 projects`.
    pub fn all_label(&self) -> String {
        if self.hidden.is_empty() {
            return "All projects".to_owned();
        }
        match self.only_shown() {
            Some(one) => one.name.clone(),
            None => format!("{} of {} projects", self.shown_projects().count(), self.projects.len()),
        }
    }

    pub fn project_index(&self, id: &ProjectId) -> Option<usize> {
        self.projects.iter().position(|project| &project.id == id)
    }

    pub fn worktree(&self, checkout: &CheckoutRef) -> Option<&WorktreeInfo> {
        let project = self.projects.iter().find(|project| project.id == checkout.project)?;
        project.worktrees.iter().find(|worktree| worktree.checkout.path == checkout.path)
    }

    /// The project's worktrees most recently used first, never-used ones last in the project's order; only those
    /// in `only` when given.
    pub fn by_recency(&self, project: &ProjectId, only: Option<&[PathBuf]>) -> Vec<&WorktreeInfo> {
        let Some(project) = self.projects.iter().find(|info| &info.id == project) else {
            return Vec::new();
        };
        let mut worktrees: Vec<&WorktreeInfo> =
            project.worktrees.iter().filter(|worktree| only.is_none_or(|only| only.contains(&worktree.checkout.path))).collect();
        worktrees.sort_by_key(|worktree| std::cmp::Reverse(worktree.last_used));
        worktrees
    }
}

/// A row's state in one worktree, for the worktree question: `● open` for a file open there, a run's status.
pub trait StateIn {
    fn state_in(&self, target: &Target, checkout: &Path) -> Option<String>;
}

/// No states: for tests.
#[cfg(test)]
pub struct NoStates;

#[cfg(test)]
impl StateIn for NoStates {
    fn state_in(&self, _target: &Target, _checkout: &Path) -> Option<String> {
        None
    }
}
