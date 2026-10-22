//! A selection in a read-only document, as file offsets: what it marks on each shown line and what it copies.

use std::ops::Range;

use crate::document::DisplayLine;

/// Beyond this a copy would hold the UI and flood the clipboard; refuse instead.
pub const MAX_COPY_BYTES: usize = 64 * 1024 * 1024;

/// Where a selection began and where it reaches now; either may come first in the file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextSelection {
    anchor: usize,
    head: usize,
}

impl TextSelection {
    /// Nothing selected yet, with the next extension starting from `offset`.
    pub fn at(offset: usize) -> Self {
        Self { anchor: offset, head: offset }
    }

    pub fn of(range: Range<usize>) -> Self {
        Self { anchor: range.start, head: range.end }
    }

    pub fn extended_to(self, head: usize) -> Self {
        Self { head, ..self }
    }

    pub fn range(&self) -> Range<usize> {
        self.anchor.min(self.head)..self.anchor.max(self.head)
    }

    /// The display columns it marks on `line`; one past the text when it goes on past the line break.
    pub fn columns_on(&self, line: &DisplayLine) -> Option<Range<usize>> {
        let range = self.range();
        let first = line.offset_at(0);
        if range.end <= first || range.start > line.end() {
            return None;
        }
        let start = if range.start <= first { 0 } else { line.column_of(range.start) };
        let end = if range.end > line.end() { line.text.chars().count() + 1 } else { line.column_of(range.end) };
        (start < end).then_some(start..end)
    }

    /// The selected bytes as text; invalid UTF-8 shows as U+FFFD, as on screen.
    pub fn text(&self, bytes: &[u8]) -> Result<String, String> {
        let range = self.range();
        let range = range.start.min(bytes.len())..range.end.min(bytes.len());
        if range.len() > MAX_COPY_BYTES {
            return Err(format!("selection of {} bytes is too large to copy (at most {MAX_COPY_BYTES})", range.len()));
        }
        Ok(String::from_utf8_lossy(&bytes[range]).into_owned())
    }
}

/// The word around `offset` on `line`: letters, digits, `_` and non-ASCII characters; a single other character
/// otherwise; nothing at the end of the line.
pub fn word_around(bytes: &[u8], line: &DisplayLine, offset: usize) -> Range<usize> {
    let shown = line.file_bytes();
    let offset = offset.clamp(shown.start, shown.end);
    let is_word = |byte: &u8| byte.is_ascii_alphanumeric() || *byte == b'_' || !byte.is_ascii();
    let Some(byte) = bytes.get(offset).filter(|_| offset < shown.end) else {
        return offset..offset;
    };
    if !is_word(byte) {
        return offset..offset + 1;
    }
    let start = bytes[shown.start..offset].iter().rposition(|byte| !is_word(byte)).map_or(shown.start, |at| shown.start + at + 1);
    let end = bytes[offset..shown.end].iter().position(|byte| !is_word(byte)).map_or(shown.end, |at| offset + at);
    start..end
}

/// All of `line` with its line break.
pub fn whole_line(bytes: &[u8], line: &DisplayLine) -> Range<usize> {
    line.offset_at(0)..(line.end() + 1).min(bytes.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEXT: &[u8] = b"let x_1 = a;\n\tb\nlast";

    fn line(start: usize) -> DisplayLine {
        let end = memchr::memchr(b'\n', &TEXT[start..]).map_or(TEXT.len(), |at| start + at);
        DisplayLine::of(&TEXT[start..end], start)
    }

    #[test]
    fn should_mark_columns_of_each_line_with_the_break_after_a_line_it_passes() {
        // given
        let selection = TextSelection::at(15).extended_to(4);

        // when
        let marked = [0, 13, 16].map(|start| selection.columns_on(&line(start)));

        // then
        assert_eq!(marked, [Some(4..13), Some(0..5), None]);
    }

    #[test]
    fn should_mark_only_the_line_break_when_a_selection_starts_at_the_end_of_a_line() {
        // given
        let selection = TextSelection::of(12..13);

        // when
        let marked = [0, 13].map(|start| selection.columns_on(&line(start)));

        // then
        assert_eq!(marked, [Some(12..13), None]);
    }

    #[test]
    fn should_copy_the_file_bytes_with_tabs_and_line_breaks() {
        // given
        let selection = TextSelection::of(10..15);

        // when
        let text = selection.text(TEXT);

        // then
        assert_eq!(text, Ok("a;\n\tb".to_owned()));
    }

    #[test]
    fn should_refuse_to_copy_more_than_the_limit() {
        // given
        let bytes = vec![b'a'; MAX_COPY_BYTES + 1];

        // when
        let text = TextSelection::of(0..bytes.len()).text(&bytes);

        // then
        assert_eq!(text, Err(format!("selection of {} bytes is too large to copy (at most {MAX_COPY_BYTES})", MAX_COPY_BYTES + 1)));
    }

    #[test]
    fn should_select_the_word_or_the_single_character_around_an_offset() {
        // given
        let first = line(0);

        // when
        let words = [5, 4, 8, 12].map(|offset| word_around(TEXT, &first, offset));

        // then
        assert_eq!(words, [4..7, 4..7, 8..9, 12..12]);
    }

    #[test]
    fn should_select_a_whole_line_with_its_break_and_the_last_without() {
        // when
        let lines = [whole_line(TEXT, &line(0)), whole_line(TEXT, &line(16))];

        // then
        assert_eq!(lines, [0..13, 16..20]);
    }
}
