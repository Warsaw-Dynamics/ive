use std::path::PathBuf;

use crate::git::log::CommitInfo;
use crate::project::ProjectId;

/// What a git view asks the workspace for; views emit it, the workspace subscribes.
#[derive(Debug, Clone)]
pub enum GitEvent {
    OpenCommit(CommitLink),
    OpenGitLog(ProjectId),
    /// New worktree added `path`; the workspace refreshes the project and shows the new checkout
    WorktreeAdded { project: ProjectId, path: PathBuf },
}

/// A commit to show in its tab.
#[derive(Debug, Clone)]
pub struct CommitLink {
    pub project: ProjectId,
    /// the checkout the commit was read from
    pub dir: PathBuf,
    pub commit: CommitInfo,
}
