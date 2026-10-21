//! Plain data shared by the git readers; no git2 type leaves the `git` module.

use std::path::Path;

use git2::{Delta, ErrorCode, Oid, Repository, Tree};

use super::repo::GitError;

/// As part 1's `short_sha`.
const SHORT_SHA_LEN: usize = 7;

/// A path inside a repository as libgit2 stores it: `/`-separated bytes, not necessarily UTF-8.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GitPath(Vec<u8>);

impl GitPath {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self(bytes.to_vec())
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// For display; bytes that are not UTF-8 show as U+FFFD.
    pub fn display(&self) -> String {
        lossy(&self.0)
    }

    /// `view.rs` of `src/view.rs`; `nested` of an untracked folder `nested/`.
    pub fn file_name(&self) -> String {
        let path = self.trimmed();
        let start = path.iter().rposition(|byte| *byte == b'/').map_or(0, |slash| slash + 1);
        lossy(&path[start..])
    }

    /// `src` of `src/view.rs`; empty at the top of the repository.
    pub fn folder(&self) -> String {
        let path = self.trimmed();
        path.iter().rposition(|byte| *byte == b'/').map_or_else(String::new, |slash| lossy(&path[..slash]))
    }

    /// An untracked folder libgit2 did not descend into: a nested repository or checkout (`nested/`).
    pub fn is_folder(&self) -> bool {
        self.0.ends_with(b"/")
    }

    fn trimmed(&self) -> &[u8] {
        self.0.strip_suffix(b"/").unwrap_or(&self.0)
    }
}

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

/// How a path changed, as the one-letter marks of `git status --short`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mark {
    Modified,
    Added,
    Deleted,
    Renamed,
    TypeChange,
    Untracked,
    Conflicted,
}

impl Mark {
    pub fn letter(self) -> &'static str {
        match self {
            Self::Modified => "M",
            Self::Added => "A",
            Self::Deleted => "D",
            Self::Renamed => "R",
            Self::TypeChange => "T",
            Self::Untracked => "?",
            Self::Conflicted => "U",
        }
    }

    pub(crate) fn of_delta(delta: Delta) -> Self {
        match delta {
            Delta::Added | Delta::Copied => Self::Added,
            Delta::Deleted => Self::Deleted,
            Delta::Renamed => Self::Renamed,
            Delta::Typechange => Self::TypeChange,
            Delta::Untracked => Self::Untracked,
            Delta::Conflicted => Self::Conflicted,
            Delta::Modified | Delta::Unmodified | Delta::Ignored | Delta::Unreadable => Self::Modified,
        }
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
    fn should_split_paths_into_name_and_folder() {
        // given
        let paths = [GitPath::from_bytes(b"src/view.rs"), GitPath::from_bytes(b"README.md"), GitPath::from_bytes(b".claude/worktrees/nested/")];

        // when
        let parts: Vec<_> = paths.iter().map(|path| (path.file_name(), path.folder(), path.is_folder())).collect();

        // then
        assert_eq!(parts, [
            ("view.rs".to_owned(), "src".to_owned(), false),
            ("README.md".to_owned(), String::new(), false),
            ("nested".to_owned(), ".claude/worktrees".to_owned(), true),
        ]);
    }

    #[test]
    fn should_keep_bytes_and_display_invalid_utf8_lossily() {
        // given
        let path = GitPath::from_bytes(b"caf\xe9.txt");

        // when
        let shown = path.display();

        // then
        assert_eq!((shown.as_str(), path.as_bytes()), ("caf\u{fffd}.txt", &b"caf\xe9.txt"[..]));
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
