//! All git access, through libgit2 (`git2`); IVE runs the `git` command only to commit one change
//! ([`commit_block`]), so the user's hooks and signing apply.

pub mod base;
mod blob_filter;
pub mod commit_block;
pub mod diff;
pub mod files;
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
