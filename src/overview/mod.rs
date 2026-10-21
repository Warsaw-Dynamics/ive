//! The project overview tab (Worktrees, Git, Scratches) and the git tabs it opens. A worktree's git and processes,
//! and a folder's runs, are in the tree.

pub mod commit_tab;
pub mod diff_tab;
pub mod file_list;
pub mod git_event;
pub mod git_log_tab;
pub mod git_ui;
mod project;
pub use project::card_tags as project_card_tags;
pub mod project_git;
pub mod scratches;
mod section_row;
pub mod worktree_list;
pub mod worktree_list_view;
pub mod worktree_rows;

pub use project::render_project_overview;
pub use section_row::{Listener, Location, section_row};

use gpui::{Context, Entity};

use crate::project::ProjectId;
use scratches::ScratchesSection;
use crate::workspace::Workspace;
use worktree_list::WorktreeList;
use project_git::ProjectGit;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ProjectSection {
    #[default]
    Worktrees,
    /// git projects only
    Git,
    /// shown for every project, git or plain
    Scratches,
}

impl ProjectSection {
    /// In section-row order.
    pub const ALL: &'static [Self] = &[Self::Worktrees, Self::Git, Self::Scratches];

    pub fn from_label(label: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|section| section.label() == label)
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Worktrees => "Worktrees",
            Self::Git => "Git",
            Self::Scratches => "Scratches",
        }
    }
}

/// A project overview tab; it remembers its selected section.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectOverview {
    pub project: ProjectId,
    pub section: ProjectSection,
    /// the Git section and the Worktrees list; `None` for a plain folder, which has neither
    pub git: Option<ProjectGitViews>,
    /// made when the Scratches section is first shown
    pub scratches: Option<Entity<ScratchesSection>>,
}

/// A git project overview's parts of part 2.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectGitViews {
    pub section: Entity<ProjectGit>,
    pub list: Entity<WorktreeList>,
}

impl ProjectOverview {
    pub fn new(project: ProjectId, git: Option<ProjectGitViews>) -> Self {
        Self { project, section: ProjectSection::default(), git, scratches: None }
    }

    pub fn select(&mut self, section: ProjectSection, cx: &mut Context<Workspace>) {
        self.section = section;
        cx.notify();
    }
}

/// `↑ahead` and `↓behind`, each only when non-zero.
pub fn distance_tags(ahead: u32, behind: u32) -> Vec<String> {
    let ahead = (ahead > 0).then(|| format!("↑{ahead}"));
    let behind = (behind > 0).then(|| format!("↓{behind}"));
    ahead.into_iter().chain(behind).collect()
}
