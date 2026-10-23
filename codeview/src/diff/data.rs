//! What a diff view shows: files, hunks and lines, as the host read them. Free of any git library.

pub const FILE_LINE_LIMIT: usize = 20_000;
pub const VIEW_LINE_LIMIT: usize = 200_000;
/// Larger files are not loaded: libgit2 marks them binary, IVE says "file too large".
pub const MAX_FILE_BYTES: u64 = 10 * 1024 * 1024;

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

fn lossy(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
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
    /// The file is on one side only, added or deleted: side by side would leave a panel empty.
    pub fn is_one_sided(self) -> bool {
        matches!(self, Self::Added | Self::Untracked | Self::Deleted)
    }

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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileDiff {
    pub path: GitPath,
    pub old_path: Option<GitPath>,
    pub mark: Mark,
    pub body: DiffBody,
}

impl FileDiff {
    pub fn line_count(&self) -> usize {
        match &self.body {
            DiffBody::Text { hunks, .. } => hunks.iter().map(|hunk| hunk.lines.len()).sum(),
            DiffBody::Binary | DiffBody::TooLarge => 0,
        }
    }

    /// The hunk showing new-side line `line` (from 1), else the last one starting before it.
    pub fn hunk_at_new_line(&self, line: usize) -> Option<usize> {
        let DiffBody::Text { hunks, .. } = &self.body else {
            return None;
        };
        let first_new = |hunk: &Hunk| hunk.lines.iter().find_map(|diff_line| diff_line.new).map_or(usize::MAX, |new| new as usize);
        let holding = hunks.iter().position(|hunk| hunk.lines.iter().any(|diff_line| diff_line.new == Some(line as u32)));
        holding.or_else(|| hunks.iter().rposition(|hunk| first_new(hunk) <= line))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffBody {
    Text { hunks: Vec<Hunk>, cut: Option<Cut> },
    Binary,
    TooLarge,
}

/// Why a text diff stops early, with the limit that was reached.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Cut {
    FileLimit(usize),
    ViewLimit(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hunk {
    pub header: String,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffLine {
    pub kind: LineKind,
    pub old: Option<u32>,
    pub new: Option<u32>,
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineKind {
    Context,
    Added,
    Removed,
    NoNewline,
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn should_find_the_hunk_showing_a_new_line() {
        // given
        let line = |old, new| DiffLine { kind: LineKind::Context, old, new, text: String::new() };
        let hunk = |lines| Hunk { header: "@@ @@".to_owned(), lines };
        let hunks = vec![hunk(vec![line(Some(1), Some(1)), line(Some(2), None)]), hunk(vec![line(Some(9), Some(8)), line(None, Some(9))])];
        let file = FileDiff { path: GitPath::from_bytes(b"a"), old_path: None, mark: Mark::Modified, body: DiffBody::Text { hunks, cut: None } };

        // when
        let found = [1, 5, 9, 0].map(|new| file.hunk_at_new_line(new));

        // then
        assert_eq!(found, [Some(0), Some(0), Some(1), None]);
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
}
