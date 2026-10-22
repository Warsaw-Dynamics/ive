//! All git access, through libgit2 (`git2`); IVE never runs the `git` command.

pub mod base;
pub mod diff;
pub mod files;
pub mod ignore_file;
pub mod log;
pub mod refs;
pub mod repo;
pub mod status;
#[cfg(test)]
pub mod testing;
pub mod types;
pub mod worktree_add;
pub mod worktree_data;
pub mod worktrees;

pub use repo::GitError;
