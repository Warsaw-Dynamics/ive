use alacritty_terminal::event::EventListener;
use alacritty_terminal::grid::Dimensions;
use alacritty_terminal::index::{Column, Line, Point};
use alacritty_terminal::term::Term;
use alacritty_terminal::term::cell::Flags;
use serde::Serialize;

/// The visible screen as text, for tools that read the terminal instead of looking at it.
#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct ScreenText {
    pub columns: usize,
    pub rows: usize,
    /// one entry per screen row, trailing blanks trimmed
    pub lines: Vec<String>,
    /// zero-based row and column; `None` when the cursor is hidden or scrolled out of view
    pub cursor: Option<(usize, usize)>,
}

pub fn screen_text<T: EventListener>(term: &Term<T>) -> ScreenText {
    let content = term.renderable_content();
    let offset = content.display_offset as i32;
    let mut lines = vec![String::new(); term.screen_lines()];
    for indexed in content.display_iter {
        let Ok(row) = usize::try_from(indexed.point.line.0 + offset) else {
            continue;
        };
        if indexed.cell.flags.intersects(Flags::WIDE_CHAR_SPACER | Flags::LEADING_WIDE_CHAR_SPACER) {
            continue;
        }
        if let Some(line) = lines.get_mut(row) {
            line.push(indexed.cell.c);
        }
    }
    for line in &mut lines {
        line.truncate(line.trim_end().len());
    }
    let cursor_row = usize::try_from(content.cursor.point.line.0 + offset).ok();
    let visible = content.cursor.shape != alacritty_terminal::vte::ansi::CursorShape::Hidden;
    let cursor = cursor_row.filter(|_| visible).map(|row| (row, content.cursor.point.column.0));
    ScreenText { columns: term.columns(), rows: term.screen_lines(), lines, cursor }
}

/// The whole output, scrollback included, as text lines: rows a long line wrapped onto are joined again, and the
/// blank rows below the output are left out. `last` keeps only that many lines from the end.
pub fn output_text<T: EventListener>(term: &Term<T>, last: Option<usize>) -> Vec<String> {
    let mut lines: Vec<String> = output_rows(term).into_iter().map(|row| row.text).collect();
    let keep = last.map_or(lines.len(), |last| last.min(lines.len()));
    lines.split_off(lines.len() - keep)
}

/// A line of the output, as [`output_text`] gives it, and the grid line its first row is on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputRow {
    pub first: i32,
    pub text: String,
}

impl AsRef<str> for OutputRow {
    fn as_ref(&self) -> &str {
        &self.text
    }
}

/// The lines of [`output_text`], each with the grid line it starts on.
pub fn output_rows<T: EventListener>(term: &Term<T>) -> Vec<OutputRow> {
    let grid = term.grid();
    let mut rows = Vec::new();
    let mut current = String::new();
    let top = -(grid.history_size() as i32);
    let mut first = top;
    for line in top..term.screen_lines() as i32 {
        if push_row(term, Line(line), &mut current) {
            continue;
        }
        current.truncate(current.trim_end().len());
        rows.push(OutputRow { first, text: std::mem::take(&mut current) });
        first = line + 1;
    }
    while rows.last().is_some_and(|row| row.text.is_empty()) {
        rows.pop();
    }
    rows
}

/// The last grid line with anything on it; the first screen line when there is none.
pub fn last_output_line<T: EventListener>(term: &Term<T>) -> i32 {
    let grid = term.grid();
    let blank = |line: i32| (0..term.columns()).all(|column| grid[Line(line)][Column(column)].c == ' ');
    (-(grid.history_size() as i32)..term.screen_lines() as i32).rev().find(|line| !blank(*line)).unwrap_or(0)
}

/// Adds the characters of grid line `line` to `text`; whether the line wraps onto the next.
fn push_row<T: EventListener>(term: &Term<T>, line: Line, text: &mut String) -> bool {
    let row = &term.grid()[line];
    let mut wrapped = false;
    for column in 0..term.columns() {
        let cell = &row[Column(column)];
        wrapped = cell.flags.contains(Flags::WRAPLINE);
        if cell.flags.intersects(Flags::WIDE_CHAR_SPACER | Flags::LEADING_WIDE_CHAR_SPACER) {
            continue;
        }
        text.push(cell.c);
    }
    wrapped
}

/// Where byte `byte` of `row`'s text is on the grid: past its end, the cell after its last character.
pub fn output_point<T: EventListener>(term: &Term<T>, row: &OutputRow, byte: usize) -> Point {
    let grid = term.grid();
    let mut at = 0;
    for line in row.first..term.screen_lines() as i32 {
        for column in 0..term.columns() {
            let cell = &grid[Line(line)][Column(column)];
            if cell.flags.intersects(Flags::WIDE_CHAR_SPACER | Flags::LEADING_WIDE_CHAR_SPACER) {
                continue;
            }
            if at >= byte {
                return Point::new(Line(line), Column(column));
            }
            at += cell.c.len_utf8();
        }
    }
    Point::new(Line(term.screen_lines() as i32 - 1), term.last_column())
}

#[cfg(test)]
mod tests {
    use alacritty_terminal::event::VoidListener;
    use alacritty_terminal::term::Config;
    use alacritty_terminal::vte::ansi::{Processor, StdSyncHandler};

    use super::*;
    use crate::terminal::session::GridSize;

    fn screen(bytes: &[u8]) -> ScreenText {
        let size = GridSize { columns: 12, lines: 3, cell_width: 8, cell_height: 16 };
        let mut term = Term::new(Config::default(), &size, VoidListener);
        Processor::<StdSyncHandler>::new().advance(&mut term, bytes);
        screen_text(&term)
    }

    #[test]
    fn should_return_plain_text_without_colors_and_trailing_blanks() {
        // given
        let output = b"\x1b[31mred\x1b[0m ok   \r\n$ ";

        // when
        let text = screen(output);

        // then
        assert_eq!((text.lines, text.cursor), (vec!["red ok".to_owned(), "$".to_owned(), String::new()], Some((1, 2))));
    }

    #[test]
    fn should_keep_wide_characters_once() {
        // given
        let output = "日本語".as_bytes();

        // when
        let text = screen(output);

        // then
        assert_eq!(text.lines[0], "日本語");
    }

    fn output(bytes: &[u8], last: Option<usize>) -> Vec<String> {
        let size = GridSize { columns: 12, lines: 3, cell_width: 8, cell_height: 16 };
        let mut term = Term::new(Config { scrolling_history: 100, ..Config::default() }, &size, VoidListener);
        Processor::<StdSyncHandler>::new().advance(&mut term, bytes);
        output_text(&term, last)
    }

    fn term_of(bytes: &[u8]) -> Term<VoidListener> {
        let size = GridSize { columns: 12, lines: 3, cell_width: 8, cell_height: 16 };
        let mut term = Term::new(Config { scrolling_history: 100, ..Config::default() }, &size, VoidListener);
        Processor::<StdSyncHandler>::new().advance(&mut term, bytes);
        term
    }

    #[test]
    fn should_give_each_output_line_its_first_row() {
        // given
        let term = term_of(b"abcdefghijklmnopq\r\nnext");

        // when
        let rows = output_rows(&term);

        // then
        let found: Vec<(i32, &str)> = rows.iter().map(|row| (row.first, row.text.as_str())).collect();
        assert_eq!(found, [(0, "abcdefghijklmnopq"), (2, "next")]);
    }

    #[test]
    fn should_map_a_match_on_a_wrapped_line_to_its_row() {
        // given
        let term = term_of(b"abcdefghijklmnopq\r\nnext");
        let rows = output_rows(&term);

        // when
        let point = output_point(&term, &rows[0], 14);

        // then
        assert_eq!(point, Point::new(Line(1), Column(2)));
    }

    #[test]
    fn should_count_a_wide_character_as_two_columns() {
        // given
        let term = term_of("日本x".as_bytes());
        let rows = output_rows(&term);

        // when
        let point = output_point(&term, &rows[0], 6);

        // then
        assert_eq!(point, Point::new(Line(0), Column(4)));
    }

    #[test]
    fn should_find_the_last_row_of_a_wrapped_last_line() {
        // given
        let term = term_of(b"one\r\nabcdefghijklmnopq");

        // when
        let last = last_output_line(&term);

        // then
        assert_eq!(last, 2);
    }

    #[test]
    fn should_return_whole_output_including_scrollback() {
        // given
        let printed = b"one\r\ntwo\r\nthree\r\nfour\r\nfive";

        // when
        let lines = output(printed, None);

        // then
        assert_eq!(lines, ["one", "two", "three", "four", "five"]);
    }

    #[test]
    fn should_return_only_the_last_lines_asked_for() {
        // given
        let printed = b"one\r\ntwo\r\nthree\r\nfour\r\nfive";

        // when
        let lines = output(printed, Some(2));

        // then
        assert_eq!(lines, ["four", "five"]);
    }

    #[test]
    fn should_join_rows_a_long_line_wrapped_onto() {
        // given
        let printed = b"0123456789abcdefghij\r\nend";

        // when
        let lines = output(printed, None);

        // then
        assert_eq!(lines, ["0123456789abcdefghij", "end"]);
    }

    #[test]
    fn should_leave_out_blank_rows_below_the_output() {
        // given
        let printed = b"only\r\n";

        // when
        let lines = output(printed, None);

        // then
        assert_eq!(lines, ["only"]);
    }

    #[test]
    fn should_report_no_cursor_when_program_hides_it() {
        // given
        let output = b"\x1b[?25lbusy";

        // when
        let text = screen(output);

        // then
        assert_eq!((text.columns, text.rows, text.cursor), (12, 3, None));
    }
}
