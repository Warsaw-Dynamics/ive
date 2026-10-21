//! The rows a diff view draws, built without gpui.

use std::collections::HashSet;

use crate::git::diff::{Cut, DiffBody, FileDiff};

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
    Note { file: usize, note: Note },
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

pub fn diff_rows(files: &[FileDiff], headers: Headers, collapsed: &HashSet<usize>) -> Vec<DiffRow> {
    let mut rows = Vec::new();
    for (file, diff) in files.iter().enumerate() {
        if headers == Headers::Shown {
            rows.push(DiffRow::File(file));
            if collapsed.contains(&file) {
                continue;
            }
        }
        file_rows(file, diff, &mut rows);
    }
    rows
}

fn file_rows(file: usize, diff: &FileDiff, rows: &mut Vec<DiffRow>) {
    let (hunks, cut) = match &diff.body {
        DiffBody::Binary => return rows.push(DiffRow::Note { file, note: Note::Binary }),
        DiffBody::TooLarge => return rows.push(DiffRow::Note { file, note: Note::TooLarge }),
        DiffBody::Text { hunks, cut } => (hunks, cut),
    };
    if hunks.is_empty() && cut.is_none() {
        let note = if diff.path.is_folder() { Note::Nested } else { Note::NoTextChanges };
        return rows.push(DiffRow::Note { file, note });
    }
    for (hunk, content) in hunks.iter().enumerate() {
        rows.push(DiffRow::Hunk { file, hunk });
        rows.extend((0..content.lines.len()).map(|line| DiffRow::Line { file, hunk, line }));
    }
    if let Some(cut) = cut {
        rows.push(DiffRow::Note { file, note: Note::Cut(cut.clone()) });
    }
}

/// `20,000`.
fn grouped(number: usize) -> String {
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
    use crate::git::diff::{DiffLine, Hunk, LineKind};
    use crate::git::types::{GitPath, Mark};

    fn text_file(name: &str, lines: usize, cut: Option<Cut>) -> FileDiff {
        let lines = (0..lines).map(|n| DiffLine { kind: LineKind::Added, old: None, new: Some(n as u32 + 1), text: format!("{n}") }).collect();
        FileDiff { path: GitPath::from_bytes(name.as_bytes()), old_path: None, mark: Mark::Added, body: DiffBody::Text { hunks: vec![Hunk { header: "@@ -0,0 +1 @@".to_owned(), lines }], cut } }
    }

    #[test]
    fn should_list_hunk_header_and_lines_then_the_cut_marker() {
        // given
        let files = [text_file("a.txt", 2, Some(Cut::FileLimit(20_000)))];

        // when
        let rows = diff_rows(&files, Headers::Hidden, &HashSet::new());

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
    fn should_show_only_the_header_of_a_collapsed_file() {
        // given
        let files = [text_file("a.txt", 1, None), text_file("b.txt", 1, None)];

        // when
        let rows = diff_rows(&files, Headers::Shown, &HashSet::from([0]));

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
        let rows = diff_rows(&[binary, large, renamed, nested], Headers::Hidden, &HashSet::new());

        // then
        assert_eq!(rows, [
            DiffRow::Note { file: 0, note: Note::Binary },
            DiffRow::Note { file: 1, note: Note::TooLarge },
            DiffRow::Note { file: 2, note: Note::NoTextChanges },
            DiffRow::Note { file: 3, note: Note::Nested },
        ]);
    }
}
