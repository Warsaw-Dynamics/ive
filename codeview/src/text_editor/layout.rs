//! What part of the text a frame shows, and where text positions are on screen; free of gpui.

use std::ops::Range;

use super::model::{EditorModel, Position, content, shown, shown_char_starts};
use crate::colors::ColorKey;
use crate::syntax::Highlight;

/// One visible line: its index, the visible slice of its text, its selected columns (one past the end when the
/// selection goes on past the line break), and the colours of `text`'s bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisibleLine {
    pub line: usize,
    pub first_column: usize,
    pub text: String,
    pub selected: Option<Range<usize>>,
    pub colours: Vec<(Range<usize>, ColorKey)>,
}

/// The lines and columns a frame shows.
pub struct Area {
    pub rows: Range<usize>,
    pub columns: Range<usize>,
}

/// The lines of `area`, coloured by `highlights` (byte offsets of the text, sorted and disjoint).
pub fn visible_lines(model: &EditorModel, area: Area, highlights: &[Highlight]) -> Vec<VisibleLine> {
    let Area { rows, columns } = area;
    let selection = model.selection();
    let rows = rows.start.min(model.line_count())..rows.end.min(model.line_count());
    rows.map(|line| {
        let (start, stored) = model.stored_line(line);
        let all = shown(&stored);
        let visible = all.chars().skip(columns.start).take(columns.len());
        let text: String = visible.map(|c| if c.is_control() { ' ' } else { c }).collect();
        let starts = shown_char_starts(content(&stored)).skip(columns.start).map(|at| start + at as u64);
        VisibleLine {
            line,
            first_column: columns.start,
            colours: colours(&text, starts, highlights),
            text,
            selected: selection.as_ref().and_then(|selection| selected_columns(selection, line, all.chars().count())),
        }
    })
    .collect()
}

/// Runs of `text` in one colour key; `starts` has the text offset each of its characters is stored at.
fn colours(text: &str, starts: impl Iterator<Item = u64>, highlights: &[Highlight]) -> Vec<(Range<usize>, ColorKey)> {
    let mut runs: Vec<(Range<usize>, ColorKey)> = Vec::new();
    for ((at, c), stored) in text.char_indices().zip(starts) {
        let Some(key) = key_at(highlights, stored as usize) else {
            continue;
        };
        let bytes = at..at + c.len_utf8();
        match runs.last_mut() {
            Some((run, last)) if *last == key && run.end == at => run.end = bytes.end,
            _ => runs.push((bytes, key)),
        }
    }
    runs
}

fn key_at(highlights: &[Highlight], offset: usize) -> Option<ColorKey> {
    let at = highlights.partition_point(|highlight| highlight.bytes.end <= offset);
    highlights.get(at).filter(|highlight| highlight.bytes.start <= offset).map(|highlight| highlight.key)
}

fn selected_columns(selection: &Range<Position>, line: usize, line_chars: usize) -> Option<Range<usize>> {
    if line < selection.start.line || line > selection.end.line {
        return None;
    }
    let start = if line == selection.start.line { selection.start.column } else { 0 };
    let end = if line == selection.end.line { selection.end.column } else { line_chars + 1 };
    (start < end).then_some(start..end)
}

/// Where the text was painted, in window pixels: the top-left of the text area (right of the gutter), the cell
/// size, and the scroll position.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Geometry {
    pub origin_x: f32,
    pub origin_y: f32,
    pub line_height: f32,
    pub char_width: f32,
    pub top: f64,
    pub left: f64,
}

impl Geometry {
    /// The line under the point and the column boundary nearest to it, clamped to the text.
    pub fn position_at(&self, x: f32, y: f32, model: &EditorModel) -> Position {
        let line = f64::from((y - self.origin_y) / self.line_height) + self.top;
        let column = (f64::from(x - self.origin_x) + self.left) / f64::from(self.char_width);
        model.clamp(Position::new(line.max(0.).floor() as usize, column.max(0.).round() as usize))
    }

    pub fn point_of(&self, position: Position) -> (f32, f32) {
        let x = f64::from(self.origin_x) + position.column as f64 * f64::from(self.char_width) - self.left;
        let y = f64::from(self.origin_y) + (position.line as f64 - self.top) * f64::from(self.line_height);
        (x as f32, y as f32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::piece_table::PieceTable;
    use crate::text_editor::model::Selecting;

    fn model(text: &str) -> EditorModel {
        EditorModel::new(PieceTable::from_text(text), None)
    }

    #[test]
    fn should_slice_visible_columns_and_mark_selected_columns_per_line() {
        // given
        let mut model = model("abcdef\nxy\n\nlast");
        model.set_caret(Position::new(0, 4), Selecting::Move);
        model.set_caret(Position::new(2, 0), Selecting::Extend);

        // when
        let lines = visible_lines(&model, Area { rows: 0..3, columns: 2..5 }, &[]);

        // then
        assert_eq!(
            lines,
            [
                VisibleLine { line: 0, first_column: 2, text: "cde".to_owned(), selected: Some(4..7), colours: Vec::new() },
                VisibleLine { line: 1, first_column: 2, text: String::new(), selected: Some(0..3), colours: Vec::new() },
                VisibleLine { line: 2, first_column: 2, text: String::new(), selected: None, colours: Vec::new() },
            ]
        );
    }

    #[test]
    fn should_draw_tabs_and_control_characters_as_spaces() {
        // given
        let model = model("a\tb\u{7}c");

        // when
        let lines = visible_lines(&model, Area { rows: 0..1, columns: 0..10 }, &[]);

        // then
        assert_eq!(lines[0].text, "a b c");
    }

    #[test]
    fn should_colour_the_visible_characters_by_their_stored_bytes() {
        // given
        let model = model("x\nab\t\u{e9}\u{1}cd");
        let (one, two) = (ColorKey::TEXT, ColorKey::by_id("DEFAULT_KEYWORD").unwrap());
        // "ab" in one, the tab and the e-acute in two, the control character plain, "c" in two
        let highlights = [
            Highlight { bytes: 2..4, key: one },
            Highlight { bytes: 4..7, key: two },
            Highlight { bytes: 8..9, key: two },
        ];

        // when
        let lines = visible_lines(&model, Area { rows: 1..2, columns: 1..10 }, &highlights);

        // then
        assert_eq!(lines[0].text, "b \u{e9} cd");
        assert_eq!(lines[0].colours, [(0..1, one), (1..4, two), (5..6, two)]);
    }

    #[test]
    fn should_map_points_to_the_nearest_column_and_clamp_to_the_text() {
        // given
        let model = model("abcd\nxy");
        // the first row shows the bottom half of line 0; the text is scrolled 15 px to the left
        let geometry = Geometry { origin_x: 100., origin_y: 50., line_height: 20., char_width: 10., top: 0.5, left: 15. };

        // when
        let points = [(100., 50.), (126., 55.), (300., 75.), (0., 500.)].map(|(x, y)| geometry.position_at(x, y, &model));

        // then
        assert_eq!(points, [Position::new(0, 2), Position::new(0, 4), Position::new(1, 2), Position::new(1, 0)]);
    }

    #[test]
    fn should_place_a_position_on_screen() {
        // given
        let geometry = Geometry { origin_x: 100., origin_y: 50., line_height: 20., char_width: 10., top: 0.5, left: 15. };

        // when
        let point = geometry.point_of(Position::new(1, 3));

        // then
        assert_eq!(point, (115., 60.));
    }
}
