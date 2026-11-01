//! Soft wrap: where each line breaks into rows no wider than the viewport, and places counted in those rows; free
//! of gpui.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use serde::{Deserialize, Serialize};

use super::model::{EditorModel, Position};

/// Lines whose rows are cached at most; past it the cache starts over.
const MAX_CACHED_LINES: usize = 10_000;

/// Whether lines longer than the viewport is wide break into rows.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SoftWrap {
    #[default]
    Off,
    On,
}

impl SoftWrap {
    pub fn toggled(self) -> Self {
        match self {
            Self::Off => Self::On,
            Self::On => Self::Off,
        }
    }
}

/// The rows of one line: the column each starts at (the first 0), and the line's length in characters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineRows {
    pub starts: Vec<usize>,
    pub chars: usize,
}

impl LineRows {
    pub fn count(&self) -> usize {
        self.starts.len()
    }

    /// The row holding column `column`: a column where a row starts belongs to that row, not the one before.
    pub fn row_of(&self, column: usize) -> usize {
        self.starts.partition_point(|start| *start <= column) - 1
    }

    /// The columns row `row` shows, without the next row's first.
    pub fn columns(&self, row: usize) -> std::ops::Range<usize> {
        self.starts[row]..self.starts.get(row + 1).copied().unwrap_or(self.chars)
    }

    /// The last column the caret may take in row `row` and still be drawn on it: before the next row's first
    /// character, or the line's end on its last row.
    pub fn last_column(&self, row: usize) -> usize {
        match self.starts.get(row + 1) {
            Some(next) => next.saturating_sub(1).max(self.starts[row]),
            None => self.chars,
        }
    }
}

/// Breaks `line` into rows of at most `width` characters: after the last whitespace that fits, else after `width`
/// characters. One whitespace character may hang past `width` instead of starting a row.
pub fn line_rows(line: &str, width: usize) -> LineRows {
    let width = width.max(1);
    let (mut starts, mut start, mut space) = (vec![0], 0, None);
    let mut chars = 0;
    for (column, c) in line.chars().enumerate() {
        let used = column - start;
        let hangs = c.is_whitespace() && used == width;
        if used >= width && !hangs {
            start = match space {
                Some(space) if space >= start => space + 1,
                _ => column,
            };
            starts.push(start);
        }
        if c.is_whitespace() {
            space = Some(column);
        }
        chars = column + 1;
    }
    LineRows { starts, chars }
}

/// Rows already broken, for one text revision and width.
#[derive(Default)]
pub struct RowCache {
    lines: RefCell<HashMap<usize, Rc<LineRows>>>,
    key: RefCell<Option<(u64, usize)>>,
}

impl RowCache {
    fn rows(&self, model: &EditorModel, width: usize, line: usize) -> Rc<LineRows> {
        let key = (model.revision(), width);
        if *self.key.borrow() != Some(key) || self.lines.borrow().len() >= MAX_CACHED_LINES {
            self.lines.borrow_mut().clear();
            *self.key.borrow_mut() = Some(key);
        }
        let cached = self.lines.borrow().get(&line).cloned();
        cached.unwrap_or_else(|| {
            let rows = Rc::new(line_rows(&model.line(line), width));
            self.lines.borrow_mut().insert(line, rows.clone());
            rows
        })
    }

    /// Forgets every line: the text was replaced, and its revisions count from the start again.
    pub fn clear(&self) {
        self.lines.borrow_mut().clear();
        *self.key.borrow_mut() = None;
    }
}

/// A place counted in rows: a line, and the rows of it above the place, a fraction while scrolled into a row.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct RowPlace {
    pub line: usize,
    pub row: f64,
}

impl RowPlace {
    pub fn new(line: usize, row: f64) -> Self {
        Self { line, row }
    }
}

/// One row on screen: line `line`'s row `row`, showing its columns `columns`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub line: usize,
    pub row: usize,
    pub columns: std::ops::Range<usize>,
    /// the last column the caret may take on it, see [`LineRows::last_column`]
    pub last_column: usize,
    /// the line's last row, which also shows its line break
    pub is_last: bool,
}

/// The text of `model` broken into rows `width` characters wide.
pub struct Wrapped<'a> {
    pub model: &'a EditorModel,
    pub width: usize,
    pub cache: &'a RowCache,
}

impl Wrapped<'_> {
    pub fn rows(&self, line: usize) -> Rc<LineRows> {
        self.cache.rows(self.model, self.width, line)
    }

    fn last_line(&self) -> usize {
        self.model.line_count() - 1
    }

    /// A scroll position, a line and the fraction of its rows scrolled past, as a place.
    pub fn place_of_top(&self, top: f64) -> RowPlace {
        let line = (top.max(0.).floor() as usize).min(self.last_line());
        let fraction = (top - line as f64).clamp(0., 1.);
        RowPlace::new(line, fraction * self.rows(line).count() as f64)
    }

    /// The scroll position of `place`, as [`Wrapped::place_of_top`] reads it.
    pub fn top_of(&self, place: RowPlace) -> f64 {
        place.line as f64 + place.row / self.rows(place.line).count() as f64
    }

    /// `rows` rows below `place` (above for a negative count), kept between the text's start and its end.
    pub fn advance(&self, place: RowPlace, rows: f64) -> RowPlace {
        let RowPlace { mut line, mut row } = place;
        row += rows;
        while row < 0. {
            if line == 0 {
                return RowPlace::new(0, 0.);
            }
            line -= 1;
            row += self.rows(line).count() as f64;
        }
        loop {
            let count = self.rows(line).count() as f64;
            if row < count {
                return RowPlace::new(line, row);
            }
            if line == self.last_line() {
                return RowPlace::new(line, count);
            }
            row -= count;
            line += 1;
        }
    }

    /// The lowest scroll position: the last row at the bottom of a viewport `viewport` rows high.
    pub fn max_top(&self, viewport: f64) -> f64 {
        let end = RowPlace::new(self.last_line(), self.rows(self.last_line()).count() as f64);
        self.top_of(self.advance(end, -viewport))
    }

    /// Up to `count` rows from row `place` down.
    pub fn rows_from(&self, place: RowPlace, count: usize) -> Vec<Row> {
        let (mut line, mut row) = (place.line, place.row.floor() as usize);
        let mut shown = Vec::with_capacity(count);
        while shown.len() < count && line <= self.last_line() {
            let rows = self.rows(line);
            if row >= rows.count() {
                (line, row) = (line + 1, 0);
                continue;
            }
            shown.push(Row { line, row, columns: rows.columns(row), last_column: rows.last_column(row), is_last: row + 1 == rows.count() });
            row += 1;
        }
        shown
    }

    /// The place of the row holding `position`.
    pub fn place_of(&self, position: Position) -> RowPlace {
        RowPlace::new(position.line, self.rows(position.line).row_of(position.column) as f64)
    }

    /// The column of `position` in its row.
    pub fn column_in_row(&self, position: Position) -> usize {
        let rows = self.rows(position.line);
        position.column - rows.starts[rows.row_of(position.column)]
    }

    /// The position `rows` rows below `from` (above when negative), at column `goal` of its row or that row's last;
    /// past the first row the text's start, past the last its end.
    pub fn step(&self, from: Position, rows: isize, goal: usize) -> Position {
        let start = self.place_of(from);
        let target = self.advance(start, rows as f64);
        if rows < 0 && start.line == 0 && start.row < -rows as f64 {
            return Position::default();
        }
        let line_rows = self.rows(target.line);
        if target.row >= line_rows.count() as f64 {
            return Position::new(target.line, line_rows.chars);
        }
        let row = target.row as usize;
        Position::new(target.line, (line_rows.starts[row] + goal).min(line_rows.last_column(row)))
    }

    /// The scroll position that shows the row of `position` whole in a viewport `viewport` rows high, moving the
    /// least from `top`; `None` when it shows already.
    pub fn reveal(&self, top: f64, position: Position, viewport: f64) -> Option<f64> {
        let (top, caret) = (self.place_of_top(top), self.place_of(position));
        if caret < top {
            return Some(self.top_of(caret));
        }
        let lowest = self.advance(RowPlace::new(caret.line, caret.row + 1.), -viewport.max(1.));
        (lowest > top).then(|| self.top_of(lowest))
    }

    /// Whether any row of lines `lines` (for no lines, the top of the line they would start) is in a viewport
    /// `viewport` rows high scrolled to `top`.
    pub fn shows_lines(&self, top: f64, lines: &std::ops::Range<usize>, viewport: f64) -> bool {
        let top = self.place_of_top(top);
        let bottom = self.advance(top, viewport);
        let end = RowPlace::new(lines.end.max(lines.start + 1), 0.);
        end > top && RowPlace::new(lines.start, 0.) < bottom
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::piece_table::PieceTable;

    fn model(text: &str) -> EditorModel {
        EditorModel::new(PieceTable::from_text(text), None)
    }

    #[test]
    fn should_keep_a_line_that_fits_in_one_row() {
        // given
        let line = "short";

        // when
        let rows = line_rows(line, 10);

        // then
        assert_eq!(rows, LineRows { starts: vec![0], chars: 5 });
    }

    #[test]
    fn should_break_after_the_last_space_that_fits() {
        // given
        let line = "one two three four";

        // when
        let rows = line_rows(line, 9);

        // then
        assert_eq!(rows.starts, [0, 8, 14]);
    }

    #[test]
    fn should_break_a_word_longer_than_the_row() {
        // given
        let line = "abcdefghij";

        // when
        let rows = line_rows(line, 4);

        // then
        assert_eq!(rows.starts, [0, 4, 8]);
    }

    #[test]
    fn should_let_one_space_hang_past_the_row_instead_of_starting_the_next() {
        // given
        let line = "abcd efgh";

        // when
        let rows = line_rows(line, 4);

        // then
        assert_eq!(rows.starts, [0, 5]);
    }

    #[test]
    fn should_place_a_column_at_a_row_start_on_that_row() {
        // given
        let rows = line_rows("abcdefghij", 4);

        // when
        let placed = [0, 3, 4, 9, 10].map(|column| rows.row_of(column));

        // then
        assert_eq!(placed, [0, 0, 1, 2, 2]);
    }

    #[test]
    fn should_list_the_rows_from_a_place_across_lines() {
        // given
        let model = model("abcdefgh\nxy\n1234567");
        let cache = RowCache::default();
        let wrapped = Wrapped { model: &model, width: 3, cache: &cache };

        // when
        let rows = wrapped.rows_from(RowPlace::new(0, 2.), 4);

        // then
        let shown: Vec<(usize, usize, std::ops::Range<usize>, bool)> = rows.into_iter().map(|row| (row.line, row.row, row.columns, row.is_last)).collect();
        assert_eq!(shown, [(0, 2, 6..8, true), (1, 0, 0..2, true), (2, 0, 0..3, false), (2, 1, 3..6, false)]);
    }

    #[test]
    fn should_advance_by_rows_across_lines_and_stop_at_the_ends() {
        // given
        let model = model("abcdefgh\nxy\n1234567");
        let cache = RowCache::default();
        let wrapped = Wrapped { model: &model, width: 3, cache: &cache };

        // when
        let down = wrapped.advance(RowPlace::new(0, 1.5), 3.);
        let up = wrapped.advance(down, -2.);
        let past_start = wrapped.advance(up, -10.);
        let past_end = wrapped.advance(up, 10.);

        // then
        assert_eq!([down, up, past_start, past_end], [RowPlace::new(2, 0.5), RowPlace::new(0, 2.5), RowPlace::new(0, 0.), RowPlace::new(2, 3.)]);
    }

    #[test]
    fn should_scroll_no_further_than_the_last_row_at_the_bottom() {
        // given
        let model = model("abcdefgh\nxy\n1234567");
        let cache = RowCache::default();
        let wrapped = Wrapped { model: &model, width: 3, cache: &cache };

        // when
        let max_top = wrapped.max_top(4.);

        // then: three rows of line 0, one of line 1, three of line 2; the last four start at line 1
        assert_eq!(max_top, 1.);
    }

    #[test]
    fn should_step_by_rows_keeping_the_column_in_the_row() {
        // given
        let model = model("abcdefgh\nxy\n1234567");
        let cache = RowCache::default();
        let wrapped = Wrapped { model: &model, width: 3, cache: &cache };

        // when
        let down = wrapped.step(Position::new(0, 4), 1, 1);
        let to_short = wrapped.step(Position::new(0, 7), 1, 1);
        let up_from_top = wrapped.step(Position::new(0, 1), -1, 1);
        let down_from_bottom = wrapped.step(Position::new(2, 6), 1, 0);

        // then
        assert_eq!([down, to_short, up_from_top, down_from_bottom], [Position::new(0, 7), Position::new(1, 1), Position::new(0, 0), Position::new(2, 7)]);
    }

    #[test]
    fn should_reveal_a_row_below_the_viewport_at_its_bottom() {
        // given
        let model = model("abcdefgh\nxy\n1234567");
        let cache = RowCache::default();
        let wrapped = Wrapped { model: &model, width: 3, cache: &cache };

        // when
        let below = wrapped.reveal(0., Position::new(2, 4), 3.);
        let shown = wrapped.reveal(0., Position::new(0, 4), 3.);
        let above = wrapped.reveal(2., Position::new(0, 7), 3.);

        // then: line 2's row 1 at the bottom puts line 1 on top
        assert_eq!([below, shown, above], [Some(1.), None, Some(2. / 3.)]);
    }

    #[test]
    fn should_tell_whether_lines_show_in_the_viewport() {
        // given
        let model = model("abcdefgh\nxy\n1234567");
        let cache = RowCache::default();
        let wrapped = Wrapped { model: &model, width: 3, cache: &cache };

        // when
        let shown = [0..1, 1..2, 2..3].map(|lines| wrapped.shows_lines(0., &lines, 3.));

        // then
        assert_eq!(shown, [true, false, false]);
    }
}
