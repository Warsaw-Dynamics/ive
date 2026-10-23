//! The rows a diff view draws, built without gpui.

use std::collections::HashSet;

use super::data::{Cut, DiffBody, FileDiff, Hunk};
use super::side_by_side::{DiffLineRef, SideRow, hunk_rows};
use crate::panel::DiffLayout;

/// Files with more diff lines start collapsed in a commit tab.
pub const COLLAPSE_LINES: usize = 2_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Headers {
    /// one file: the pane and the diff tab show its name above the view
    Hidden,
    /// several files: a commit tab, each file collapsible by its header row
    Shown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffRow {
    File(usize),
    Hunk { file: usize, hunk: usize },
    Line { file: usize, hunk: usize, line: usize },
    /// a side-by-side row: an old and a new line of one hunk
    Pair { file: usize, row: SideRow },
    Note { file: usize, note: Note },
}

impl DiffRow {
    /// Whether this row shows a line `other` shows, or is `other`: where a row of one layout is in the other.
    pub fn meets(&self, other: &DiffRow) -> bool {
        match (self.lines(), other.lines()) {
            (Some((file, mine)), Some((other_file, theirs))) => file == other_file && mine.iter().flatten().any(|line| theirs.contains(&Some(*line))),
            _ => self == other,
        }
    }

    /// The file and the lines of a code row.
    fn lines(&self) -> Option<(usize, [Option<DiffLineRef>; 2])> {
        match self {
            Self::Line { file, hunk, line } => Some((*file, [Some(DiffLineRef { hunk: *hunk, line: *line }), None])),
            Self::Pair { file, row } => Some((*file, [row.old, row.new])),
            Self::File(_) | Self::Hunk { .. } | Self::Note { .. } => None,
        }
    }
}

/// How a diff view lays out its files.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shape {
    pub headers: Headers,
    pub layout: DiffLayout,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Note {
    Binary,
    TooLarge,
    /// a rename or mode change without content changes
    NoTextChanges,
    /// an untracked folder with a repository of its own: a nested checkout, whose changes are its own
    Nested,
    Cut(Cut),
}

impl Note {
    pub fn text(&self) -> String {
        match self {
            Self::Binary => "binary file".to_owned(),
            Self::TooLarge => "file too large".to_owned(),
            Self::NoTextChanges => "no content changes".to_owned(),
            Self::Nested => "a nested repository or checkout; its overview shows its changes".to_owned(),
            Self::Cut(Cut::FileLimit(lines)) => format!("diff cut at {} lines", grouped(*lines)),
            Self::Cut(Cut::ViewLimit(lines)) => format!("diff cut at {} lines in this view", grouped(*lines)),
        }
    }
}

pub fn diff_rows(files: &[FileDiff], shape: Shape, collapsed: &HashSet<usize>) -> Vec<DiffRow> {
    let mut rows = Vec::new();
    for (file, diff) in files.iter().enumerate() {
        if shape.headers == Headers::Shown {
            rows.push(DiffRow::File(file));
            if collapsed.contains(&file) {
                continue;
            }
        }
        rows.extend(file_rows(file, diff, shape.layout));
    }
    rows
}

fn file_rows(file: usize, diff: &FileDiff, layout: DiffLayout) -> Vec<DiffRow> {
    let (hunks, cut) = match &diff.body {
        DiffBody::Binary => return vec![DiffRow::Note { file, note: Note::Binary }],
        DiffBody::TooLarge => return vec![DiffRow::Note { file, note: Note::TooLarge }],
        DiffBody::Text { hunks, cut } => (hunks, cut),
    };
    if hunks.is_empty() && cut.is_none() {
        let note = if diff.path.is_folder() { Note::Nested } else { Note::NoTextChanges };
        return vec![DiffRow::Note { file, note }];
    }
    let mut rows = Vec::new();
    for (hunk, content) in hunks.iter().enumerate() {
        rows.push(DiffRow::Hunk { file, hunk });
        rows.extend(line_rows(file, (hunk, content), layout));
    }
    if let Some(cut) = cut {
        rows.push(DiffRow::Note { file, note: Note::Cut(cut.clone()) });
    }
    rows
}

/// The rows of the lines of hunk `hunk` of file `file`.
fn line_rows(file: usize, (hunk, content): (usize, &Hunk), layout: DiffLayout) -> Vec<DiffRow> {
    match layout {
        DiffLayout::Inline => (0..content.lines.len()).map(|line| DiffRow::Line { file, hunk, line }).collect(),
        DiffLayout::SideBySide => hunk_rows(hunk, &content.lines).into_iter().map(|row| DiffRow::Pair { file, row }).collect(),
    }
}

/// `20,000`.
pub(crate) fn grouped(number: usize) -> String {
    let digits = number.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            out.push(',');
        }
        out.push(digit);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::data::{DiffLine, GitPath, LineKind, Mark};

    const INLINE: Shape = Shape { headers: Headers::Hidden, layout: DiffLayout::Inline };

    fn text_file(name: &str, lines: usize, cut: Option<Cut>) -> FileDiff {
        let lines = (0..lines).map(|n| DiffLine { kind: LineKind::Added, old: None, new: Some(n as u32 + 1), text: format!("{n}") }).collect();
        FileDiff { path: GitPath::from_bytes(name.as_bytes()), old_path: None, mark: Mark::Added, body: DiffBody::Text { hunks: vec![Hunk { header: "@@ -0,0 +1 @@".to_owned(), lines }], cut } }
    }

    #[test]
    fn should_list_hunk_header_and_lines_then_the_cut_marker() {
        // given
        let files = [text_file("a.txt", 2, Some(Cut::FileLimit(20_000)))];

        // when
        let rows = diff_rows(&files, INLINE, &HashSet::new());

        // then
        assert_eq!(rows, [
            DiffRow::Hunk { file: 0, hunk: 0 },
            DiffRow::Line { file: 0, hunk: 0, line: 0 },
            DiffRow::Line { file: 0, hunk: 0, line: 1 },
            DiffRow::Note { file: 0, note: Note::Cut(Cut::FileLimit(20_000)) },
        ]);
        assert_eq!(Note::Cut(Cut::FileLimit(20_000)).text(), "diff cut at 20,000 lines");
    }

    #[test]
    fn should_find_where_an_inline_line_is_side_by_side() {
        // given
        let mut file = text_file("a.txt", 2, None);
        let DiffBody::Text { hunks, .. } = &mut file.body else { unreachable!() };
        hunks[0].lines.insert(0, DiffLine { kind: LineKind::Removed, old: Some(1), new: None, text: "gone".to_owned() });
        let files = [file];
        let side_by_side = diff_rows(&files, Shape { headers: Headers::Hidden, layout: DiffLayout::SideBySide }, &HashSet::new());

        // when
        let found: Vec<Option<usize>> = (1..=2).map(|line| side_by_side.iter().position(|row| row.meets(&DiffRow::Line { file: 0, hunk: 0, line }))).collect();

        // then
        let pair = |old, new| DiffRow::Pair { file: 0, row: SideRow { old, new } };
        let at = |line| Some(DiffLineRef { hunk: 0, line });
        assert_eq!(side_by_side, [DiffRow::Hunk { file: 0, hunk: 0 }, pair(at(0), at(1)), pair(None, at(2))]);
        assert_eq!(found, [Some(1), Some(2)]);
    }

    #[test]
    fn should_show_only_the_header_of_a_collapsed_file() {
        // given
        let files = [text_file("a.txt", 1, None), text_file("b.txt", 1, None)];

        // when
        let rows = diff_rows(&files, Shape { headers: Headers::Shown, layout: DiffLayout::Inline }, &HashSet::from([0]));

        // then
        assert_eq!(rows, [
            DiffRow::File(0),
            DiffRow::File(1),
            DiffRow::Hunk { file: 1, hunk: 0 },
            DiffRow::Line { file: 1, hunk: 0, line: 0 },
        ]);
    }

    #[test]
    fn should_say_binary_too_large_nested_or_no_text_changes_instead_of_lines() {
        // given
        let mut binary = text_file("a.bin", 0, None);
        binary.body = DiffBody::Binary;
        let mut large = text_file("big.txt", 0, None);
        large.body = DiffBody::TooLarge;
        let mut renamed = text_file("moved.txt", 0, None);
        renamed.body = DiffBody::Text { hunks: Vec::new(), cut: None };
        let mut nested = text_file(".claude/worktrees/topic/", 0, None);
        nested.body = DiffBody::Text { hunks: Vec::new(), cut: None };

        // when
        let rows = diff_rows(&[binary, large, renamed, nested], INLINE, &HashSet::new());

        // then
        assert_eq!(rows, [
            DiffRow::Note { file: 0, note: Note::Binary },
            DiffRow::Note { file: 1, note: Note::TooLarge },
            DiffRow::Note { file: 2, note: Note::NoTextChanges },
            DiffRow::Note { file: 3, note: Note::Nested },
        ]);
    }
}
