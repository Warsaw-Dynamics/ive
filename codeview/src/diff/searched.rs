//! The lines a diff's find box searches, joined by line breaks into one text, and where each came from. Free of
//! gpui.

use std::collections::HashMap;
use std::sync::Arc;

use super::data::{DiffBody, FileDiff, LineKind};
use super::side_by_side::{DiffLineRef, Side};
use crate::find::FindText;
use crate::panel::DiffLayout;

/// Inline, every code line of every file; side by side, the lines of one side.
pub(crate) struct Searched {
    text: Arc<Vec<u8>>,
    /// each line's file and place, in order
    lines: Vec<(usize, DiffLineRef)>,
    /// where each line starts in `text`
    starts: Vec<usize>,
    /// where each hunk's first searched line starts: a match stays within one hunk
    parts: Arc<Vec<usize>>,
    index: HashMap<(usize, DiffLineRef), usize>,
}

impl Searched {
    pub(crate) fn of(files: &[FileDiff], layout: DiffLayout, side: Side) -> Self {
        let searched = |kind: LineKind| match (layout, side, kind) {
            (_, _, LineKind::NoNewline) => false,
            (DiffLayout::Inline, _, _) | (DiffLayout::SideBySide, _, LineKind::Context) => true,
            (DiffLayout::SideBySide, Side::Old, kind) => kind == LineKind::Removed,
            (DiffLayout::SideBySide, Side::New, kind) => kind == LineKind::Added,
        };
        let (mut text, mut lines, mut starts, mut parts) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
        for (file, diff) in files.iter().enumerate() {
            let DiffBody::Text { hunks, .. } = &diff.body else {
                continue;
            };
            for (hunk, content) in hunks.iter().enumerate() {
                for (at, (line, shown)) in content.lines.iter().enumerate().filter(|(_, shown)| searched(shown.kind)).enumerate() {
                    if !starts.is_empty() {
                        text.push(b'\n');
                    }
                    if at == 0 {
                        parts.push(text.len());
                    }
                    starts.push(text.len());
                    text.extend_from_slice(shown.text.as_bytes());
                    lines.push((file, DiffLineRef { hunk, line }));
                }
            }
        }
        let index = lines.iter().enumerate().map(|(index, place)| (*place, index)).collect();
        Self { text: Arc::new(text), lines, starts, parts: Arc::new(parts), index }
    }

    #[cfg(test)]
    pub(crate) fn text(&self) -> Arc<Vec<u8>> {
        self.text.clone()
    }

    /// The text to search, each hunk on its own.
    pub(crate) fn find_text(&self) -> FindText {
        FindText::Parts { bytes: self.text.clone(), starts: self.parts.clone() }
    }

    /// The searched line byte `byte` of the text is on.
    pub(crate) fn line_of(&self, byte: usize) -> usize {
        self.starts.partition_point(|start| *start <= byte).saturating_sub(1)
    }

    pub(crate) fn line(&self, index: usize) -> (usize, DiffLineRef) {
        self.lines[index]
    }

    /// Where line `index` starts in the text.
    pub(crate) fn start(&self, index: usize) -> usize {
        self.starts[index]
    }

    pub(crate) fn len(&self) -> usize {
        self.lines.len()
    }

    /// The first searched line of file `file`.
    pub(crate) fn first_of(&self, file: usize) -> Option<usize> {
        self.lines.iter().position(|(of, _)| *of == file)
    }

    /// The searched line showing `at` of file `file`; `None` for a line not searched.
    pub(crate) fn index_of(&self, file: usize, at: DiffLineRef) -> Option<usize> {
        self.index.get(&(file, at)).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::data::{DiffLine, GitPath, Hunk, Mark};

    fn line(kind: LineKind, old: Option<u32>, new: Option<u32>, text: &str) -> DiffLine {
        DiffLine { kind, old, new, text: text.to_owned() }
    }

    fn file() -> FileDiff {
        let lines = vec![
            line(LineKind::Removed, Some(1), None, "old x"),
            line(LineKind::Added, None, Some(1), "new x"),
            line(LineKind::Context, Some(2), Some(2), "same"),
            line(LineKind::NoNewline, None, None, "\\ No newline at end of file"),
        ];
        FileDiff { path: GitPath::from_bytes(b"a.txt"), old_path: None, mark: Mark::Modified, body: DiffBody::Text { hunks: vec![Hunk { header: "@@".to_owned(), lines }], cut: None } }
    }

    fn text_of(searched: &Searched) -> String {
        String::from_utf8(searched.text().to_vec()).unwrap()
    }

    #[test]
    fn should_search_every_line_inline_and_one_side_side_by_side() {
        // given
        let files = [file()];

        // when
        let [inline, old, new] = [(DiffLayout::Inline, Side::New), (DiffLayout::SideBySide, Side::Old), (DiffLayout::SideBySide, Side::New)]
            .map(|(layout, side)| text_of(&Searched::of(&files, layout, side)));

        // then
        assert_eq!([inline.as_str(), old.as_str(), new.as_str()], ["old x\nnew x\nsame", "old x\nsame", "new x\nsame"]);
    }

    #[test]
    fn should_find_the_line_of_a_byte_and_a_line_by_its_place() {
        // given
        let searched = Searched::of(&[file()], DiffLayout::Inline, Side::New);

        // when
        let lines = [0, 5, 6, 12].map(|byte| searched.line_of(byte));

        // then
        assert_eq!(lines, [0, 0, 1, 2]);
        assert_eq!(searched.index_of(0, DiffLineRef { hunk: 0, line: 2 }), Some(2));
        assert_eq!((searched.line(1), searched.start(2)), ((0, DiffLineRef { hunk: 0, line: 1 }), 12));
    }
}
