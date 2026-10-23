use std::fmt;
use std::path::{Path, PathBuf};

use git2::{Oid, Repository};

/// A failed git read: what IVE tried, in which folder, and libgit2's reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitError {
    pub operation: String,
    pub dir: PathBuf,
    pub message: String,
}

impl GitError {
    /// `operation` completes "cannot … of <dir>", e.g. "read the status".
    pub fn new(operation: &str, dir: &Path, error: git2::Error) -> Self {
        let message = format!("{} (class {:?}, code {:?})", error.message(), error.class(), error.code());
        Self { operation: operation.to_owned(), dir: dir.to_path_buf(), message }
    }
}

impl fmt::Display for GitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "cannot {} of {}: {}", self.operation, self.dir.display(), self.message)
    }
}

impl std::error::Error for GitError {}

/// Opens the repository of checkout folder `dir`, without searching its parents.
/// A fresh `Repository` per background job: it is `Send` but not `Sync`, and opening is cheap.
pub fn open(dir: &Path) -> Result<Repository, GitError> {
    Repository::open(dir).map_err(|error| GitError::new("open the repository", dir, error))
}

/// The first 7 hex characters, as `git log --oneline` shows them.
pub fn short_sha(oid: Oid) -> String {
    let mut hex = oid.to_string();
    hex.truncate(7);
    hex
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_keep_git_code_and_class() {
        // given
        let error = git2::Error::new(git2::ErrorCode::NotFound, git2::ErrorClass::Reference, "reference 'refs/heads/x' not found");

        // when
        let shown = GitError::new("read the branch", Path::new("/r"), error).to_string();

        // then
        assert_eq!(shown, "cannot read the branch of /r: reference 'refs/heads/x' not found (class Reference, code NotFound)");
    }

    #[test]
    fn should_name_operation_folder_and_reason_when_folder_is_no_repository() {
        // given
        let dir = tempfile::tempdir().unwrap();

        // when
        let error = open(dir.path()).err().unwrap().to_string();

        // then
        let expected = format!("cannot open the repository of {}: ", dir.path().display());
        assert!(error.starts_with(&expected), "{error}");
        assert!(error.len() > expected.len(), "libgit2's reason is missing: {error}");
    }

    #[test]
    fn should_shorten_sha_to_seven_hex_characters() {
        // given
        let oid = Oid::from_str("0123456789abcdef0123456789abcdef01234567").unwrap();

        // when
        let short = short_sha(oid);

        // then
        assert_eq!(short, "0123456");
    }
}
