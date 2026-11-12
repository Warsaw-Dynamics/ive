use std::path::Path;

use git2::StatusOptions;

use super::repo::{self, GitError};
use super::status::Change;
use super::types::{Mark, failed};

/// gitlinks (submodules) are folders of another repository, not files
const GITLINK_MODE: u32 = 0o160000;

/// The checkout's files as git sees them, as git path bytes: the index's entries and, of `changes` (its status),
/// the untracked files git does not ignore. A nested repository is one untracked folder, left out.
pub fn listed(dir: &Path, changes: &[Change]) -> Result<Vec<Vec<u8>>, GitError> {
    let repo = repo::open(dir)?;
    let index = repo.index().map_err(failed("read the git index", dir))?;
    let mut paths: Vec<Vec<u8>> = index.iter().filter(|entry| entry.mode != GITLINK_MODE).map(|entry| entry.path).collect();
    // an untracked file the status pairs with a deleted one shows as the new name of a rename
    let unstaged = changes.iter().filter(|change| matches!(change.mark, Mark::Untracked | Mark::Renamed));
    paths.extend(unstaged.map(|change| change.path.as_bytes().to_vec()));
    Ok(files_only(paths))
}

/// The files git ignores in the checkout, ignored folders walked.
pub fn ignored(dir: &Path) -> Result<Vec<Vec<u8>>, GitError> {
    let repo = repo::open(dir)?;
    let mut options = StatusOptions::new();
    options.include_ignored(true).recurse_ignored_dirs(true).exclude_submodules(true);
    let statuses = repo.statuses(Some(&mut options)).map_err(failed("list the ignored files", dir))?;
    Ok(files_only(statuses.iter().filter(|entry| entry.status().is_ignored()).map(|entry| entry.path_bytes().to_vec()).collect()))
}

fn files_only(mut paths: Vec<Vec<u8>>) -> Vec<Vec<u8>> {
    paths.retain(|path| path.last() != Some(&b'/'));
    paths.sort_unstable();
    paths.dedup();
    paths
}
