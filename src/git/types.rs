//! Plain data shared by the git readers; no git2 type leaves the `git` module.

use std::path::Path;

use git2::{Delta, ErrorCode, Oid, Repository, Tree};

pub use codeview::diff::{GitPath, Mark};

use super::repo::GitError;

/// As part 1's `short_sha`.
const SHORT_SHA_LEN: usize = 7;

/// A commit id as 40 hex characters.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Sha(String);

impl Sha {
    pub(crate) fn of(oid: Oid) -> Self {
        Self(oid.to_string())
    }

    /// A full sha read back from `state.toml`; `None` unless it is one.
    pub fn parse(text: &str) -> Option<Self> {
        Oid::from_str(text).ok().filter(|oid| oid.to_string() == text).map(Self::of)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn short(&self) -> &str {
        &self.0[..self.0.len().min(SHORT_SHA_LEN)]
    }

    pub(crate) fn oid(&self) -> Result<Oid, git2::Error> {
        Oid::from_str(&self.0)
    }
}

/// The mark of a libgit2 delta status.
pub(crate) fn mark_of_delta(delta: Delta) -> Mark {
    match delta {
        Delta::Added | Delta::Copied => Mark::Added,
        Delta::Deleted => Mark::Deleted,
        Delta::Renamed => Mark::Renamed,
        Delta::Typechange => Mark::TypeChange,
        Delta::Untracked => Mark::Untracked,
        Delta::Conflicted => Mark::Conflicted,
        Delta::Modified | Delta::Unmodified | Delta::Ignored | Delta::Unreadable => Mark::Modified,
    }
}

/// Turns a libgit2 error into a `GitError` naming `operation` and `dir`.
pub(crate) fn failed<'a>(operation: &'a str, dir: &'a Path) -> impl Fn(git2::Error) -> GitError + 'a {
    move |error| GitError::new(operation, dir, error)
}

/// HEAD's tree, or `None` while HEAD is unborn (a repository without commits).
pub(crate) fn head_tree(repo: &Repository) -> Result<Option<Tree<'_>>, git2::Error> {
    match repo.head() {
        Ok(head) => head.peel_to_tree().map(Some),
        Err(error) if matches!(error.code(), ErrorCode::UnbornBranch | ErrorCode::NotFound) => Ok(None),
        Err(error) => Err(error),
    }
}

pub(crate) fn lossy(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_read_back_only_full_shas() {
        // given
        let full = "0123456789abcdef0123456789abcdef01234567";

        // when
        let read = [Sha::parse(full).map(|sha| sha.as_str().to_owned()), Sha::parse("0123abc").map(|sha| sha.as_str().to_owned()), Sha::parse("xyz").map(|sha| sha.as_str().to_owned())];

        // then
        assert_eq!(read, [Some(full.to_owned()), None, None]);
    }

    #[test]
    fn should_shorten_sha_to_seven_characters() {
        // given
        let sha = Sha::of(Oid::from_str("4371d9d8e2c1a0b3f4e5d6c7b8a9f0e1d2c3b4a5").expect("valid oid"));

        // when
        let short = sha.short();

        // then
        assert_eq!(short, "4371d9d");
    }
}
