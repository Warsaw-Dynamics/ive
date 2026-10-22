//! A selection in a diff view, by row and character column; only code lines hold text to select. Free of gpui.

use std::ops::Range;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DiffPosition {
    pub row: usize,
    pub column: usize,
}

impl DiffPosition {
    pub fn new(row: usize, column: usize) -> Self {
        Self { row, column }
    }
}

/// Where a selection began and where it reaches now; either may come first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiffSelection {
    anchor: DiffPosition,
    head: DiffPosition,
}

impl DiffSelection {
    /// Nothing selected yet, with the next extension starting from `position`.
    pub fn at(position: DiffPosition) -> Self {
        Self { anchor: position, head: position }
    }

    pub fn of(range: Range<DiffPosition>) -> Self {
        Self { anchor: range.start, head: range.end }
    }

    pub fn extended_to(self, head: DiffPosition) -> Self {
        Self { head, ..self }
    }

    pub fn range(&self) -> Range<DiffPosition> {
        self.anchor.min(self.head)..self.anchor.max(self.head)
    }

    /// The columns it marks on row `row` of `chars` characters; one past the text when it goes on past the row.
    pub fn columns_on(&self, row: usize, chars: usize) -> Option<Range<usize>> {
        let range = self.range();
        if row < range.start.row || row > range.end.row {
            return None;
        }
        let start = if row == range.start.row { range.start.column.min(chars) } else { 0 };
        let end = if row == range.end.row { range.end.column.min(chars) } else { chars + 1 };
        (start < end).then_some(start..end)
    }

    /// The selected text of the rows `line_text` gives text for (code lines), one per line.
    pub fn text<'a>(&self, line_text: impl Fn(usize) -> Option<&'a str>) -> String {
        let range = self.range();
        let pieces: Vec<String> = (range.start.row..=range.end.row)
            .filter_map(|row| line_text(row).map(|text| (row, text)))
            .map(|(row, text)| {
                let start = if row == range.start.row { range.start.column } else { 0 };
                let end = if row == range.end.row { range.end.column } else { usize::MAX };
                text.chars().take(end).skip(start).collect()
            })
            .collect();
        pieces.join("\n")
    }
}

/// The word around `column` in `text`: letters, digits and `_`; a single other character otherwise; nothing at
/// the end.
pub fn word_around(text: &str, column: usize) -> Range<usize> {
    let chars: Vec<char> = text.chars().collect();
    let is_word = |ch: &char| ch.is_alphanumeric() || *ch == '_';
    let Some(ch) = chars.get(column) else {
        return column.min(chars.len())..column.min(chars.len());
    };
    if !is_word(ch) {
        return column..column + 1;
    }
    let start = chars[..column].iter().rposition(|ch| !is_word(ch)).map_or(0, |at| at + 1);
    let end = chars[column..].iter().position(|ch| !is_word(ch)).map_or(chars.len(), |at| column + at);
    start..end
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROWS: [Option<&str>; 4] = [Some("fn main() {"), None, Some("    let x = 1;"), Some("}")];

    fn line_text(row: usize) -> Option<&'static str> {
        ROWS.get(row).copied().flatten()
    }

    #[test]
    fn should_mark_columns_of_each_row_with_the_break_after_a_row_it_passes() {
        // given
        let selection = DiffSelection::at(DiffPosition::new(2, 4)).extended_to(DiffPosition::new(0, 3));

        // when
        let marked = [0, 2, 3].map(|row| selection.columns_on(row, ROWS[row].map_or(0, |text| text.chars().count())));

        // then
        assert_eq!(marked, [Some(3..12), Some(0..4), None]);
    }

    #[test]
    fn should_copy_code_lines_only_and_join_them_by_line_breaks() {
        // given
        let selection = DiffSelection::of(DiffPosition::new(0, 3)..DiffPosition::new(3, 1));

        // when
        let text = selection.text(line_text);

        // then
        assert_eq!(text, "main() {\n    let x = 1;\n}");
    }

    #[test]
    fn should_copy_part_of_one_line() {
        // given
        let selection = DiffSelection::of(DiffPosition::new(2, 8)..DiffPosition::new(2, 4));

        // when
        let text = selection.text(line_text);

        // then
        assert_eq!(text, "let ");
    }

    #[test]
    fn should_select_the_word_or_the_single_character_around_a_column() {
        // given
        let text = "let x_1 = a;";

        // when
        let words = [5, 8, 12, 40].map(|column| word_around(text, column));

        // then
        assert_eq!(words, [4..7, 8..9, 12..12, 12..12]);
    }
}
