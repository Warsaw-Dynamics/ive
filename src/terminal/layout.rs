use std::ops::Range;

use alacritty_terminal::Term;
use alacritty_terminal::grid::{Dimensions, Indexed};
use alacritty_terminal::index::{Line, Point};
use alacritty_terminal::selection::SelectionRange;
use alacritty_terminal::term::RenderableContent;
use alacritty_terminal::term::cell::{Cell, Flags};
use alacritty_terminal::vte::ansi::{Color, CursorShape, NamedColor};

use super::colors::{ColorResolver, dim};
use crate::theme::TerminalPalette;

/// One screen of the terminal, ready to paint: per line, background fills and text segments, by column.
#[derive(Debug, Default)]
pub struct Frame {
    pub lines: Vec<FrameLine>,
    /// cursors other than a focused block; that one is drawn by inverting its cell
    pub cursor: Option<CursorMark>,
}

#[derive(Debug, Default)]
pub struct FrameLine {
    /// only where the background differs from the terminal's own
    pub fills: Vec<Fill>,
    pub segments: Vec<Segment>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fill {
    pub column: usize,
    pub width: usize,
    pub color: u32,
}

/// Text drawn from `column`, one cell per character; a double-width character gets a segment of its own.
#[derive(Debug, PartialEq, Eq)]
pub struct Segment {
    pub column: usize,
    pub cells: usize,
    pub text: String,
    pub runs: Vec<Run>,
    pub wide: bool,
}

/// `len` bytes of a segment's text drawn in one style.
#[derive(Debug, PartialEq, Eq)]
pub struct Run {
    pub len: usize,
    pub style: Style,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Style {
    pub color: u32,
    pub bold: bool,
    pub italic: bool,
    pub underline: Underline,
    pub strikeout: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Underline {
    None,
    Straight,
    Curly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CursorMark {
    pub line: usize,
    pub column: usize,
    pub cells: usize,
    pub shape: CursorShape,
}

/// Lays out the visible screen; an unfocused terminal shows a hollow cursor.
pub fn layout(content: RenderableContent<'_>, palette: &TerminalPalette, focused: bool) -> Frame {
    let shape = match content.cursor.shape {
        CursorShape::Block if !focused => CursorShape::HollowBlock,
        shape => shape,
    };
    let painting = Painting {
        resolver: ColorResolver { palette, overrides: content.colors },
        top: -(content.display_offset as i32),
        cursor: Some((content.cursor.point, shape)),
        selection: content.selection,
    };
    lay_out(content.display_iter, &painting)
}

/// Which grid lines a frame shows: `rows` from line `top`, with `mark` (first and last cell) drawn as selected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Window {
    pub top: i32,
    pub rows: usize,
    pub mark: Option<(Point, Point)>,
}

/// Lays out `window` of the terminal's lines without a cursor, whatever it shows on screen.
pub fn layout_window<T>(term: &Term<T>, window: &Window, palette: &TerminalPalette) -> Frame {
    let grid = term.grid();
    let bottom = window.top + window.rows as i32;
    // the iterator yields the cells after its start: the first is the window's top left
    let cells = grid.iter_from(Point::new(Line(window.top - 1), grid.last_column())).take_while(|indexed| indexed.point.line.0 < bottom);
    let painting = Painting {
        resolver: ColorResolver { palette, overrides: term.colors() },
        top: window.top,
        cursor: None,
        selection: window.mark.map(|(start, end)| SelectionRange::new(start, end, false)),
    };
    lay_out(cells, &painting)
}

/// The first line of a window of `rows` that shows grid line `line` a third of the way down, kept inside the
/// `history` lines above the screen and the `screen` lines of it.
pub fn top_showing(line: i32, rows: usize, history: usize, screen: usize) -> i32 {
    let top = line - (rows / 3) as i32;
    top.min(screen as i32 - rows as i32).max(-(history as i32))
}

/// Cells kept between a match and the edge of a view it is scrolled into.
const SHIFT_MARGIN: usize = 2;

/// Columns to scroll a view `visible` columns wide to the right so `columns` are in it; a match wider than the view
/// shows its start.
pub fn columns_shift(columns: &Range<usize>, visible: usize) -> usize {
    (columns.end + SHIFT_MARGIN).saturating_sub(visible).min(columns.start.saturating_sub(SHIFT_MARGIN))
}

/// How the cells of a frame are drawn: colours, its first grid line, the cursor and the selection.
struct Painting<'a> {
    resolver: ColorResolver<'a>,
    top: i32,
    cursor: Option<(Point, CursorShape)>,
    selection: Option<SelectionRange>,
}

fn lay_out<'a>(cells: impl Iterator<Item = Indexed<&'a Cell>>, painting: &Painting) -> Frame {
    let palette = painting.resolver.palette;
    let mut frame = Frame::default();
    for indexed in cells {
        let Ok(line) = usize::try_from(indexed.point.line.0 - painting.top) else {
            continue;
        };
        if indexed.cell.flags.intersects(Flags::WIDE_CHAR_SPACER | Flags::LEADING_WIDE_CHAR_SPACER) {
            continue;
        }
        let column = indexed.point.column.0;
        let selected = painting.selection.is_some_and(|selection| selection.contains(indexed.point));
        let cursor = painting.cursor.filter(|(point, _)| *point == indexed.point).map(|(_, shape)| shape);
        let mut look = CellLook::of(indexed.cell, &painting.resolver);
        if selected {
            look.background = palette.selection;
        }
        if cursor == Some(CursorShape::Block) {
            look = look.under_block_cursor(palette.cursor);
        }
        if let Some(shape) = cursor.filter(|shape| !matches!(shape, CursorShape::Block | CursorShape::Hidden)) {
            frame.cursor = Some(CursorMark { line, column, cells: look.cells, shape });
        }
        if frame.lines.len() <= line {
            frame.lines.resize_with(line + 1, FrameLine::default);
        }
        frame.lines[line].push(column, look, palette.background);
    }
    frame
}

/// How one cell is drawn, after its attributes are applied.
struct CellLook {
    character: char,
    cells: usize,
    style: Style,
    background: u32,
}

impl CellLook {
    fn of(cell: &Cell, resolver: &ColorResolver<'_>) -> Self {
        let flags = cell.flags;
        let mut foreground = resolver.resolve(bright_when_bold(cell.fg, flags));
        let mut background = resolver.resolve(cell.bg);
        if flags.contains(Flags::DIM) {
            foreground = dim(foreground);
        }
        if flags.contains(Flags::INVERSE) {
            (foreground, background) = (background, foreground);
        }
        let underline = match flags {
            f if f.contains(Flags::UNDERCURL) => Underline::Curly,
            f if f.intersects(Flags::ALL_UNDERLINES) => Underline::Straight,
            _ => Underline::None,
        };
        let style = Style {
            color: if flags.contains(Flags::HIDDEN) { background } else { foreground },
            bold: flags.contains(Flags::BOLD),
            italic: flags.contains(Flags::ITALIC),
            underline,
            strikeout: flags.contains(Flags::STRIKEOUT),
        };
        let cells = if flags.contains(Flags::WIDE_CHAR) { 2 } else { 1 };
        Self { character: cell.c, cells, style, background }
    }

    fn under_block_cursor(self, cursor: u32) -> Self {
        Self { style: Style { color: self.background, ..self.style }, background: cursor, ..self }
    }
}

/// Bold text in one of the eight basic colours is drawn in its bright variant, as xterm does.
fn bright_when_bold(color: Color, flags: Flags) -> Color {
    match color {
        Color::Named(name) if flags.contains(Flags::BOLD) && (name as usize) < 8 => Color::Indexed(name as u8 + 8),
        Color::Named(NamedColor::Foreground) if flags.contains(Flags::BOLD) => Color::Named(NamedColor::BrightForeground),
        color => color,
    }
}

impl FrameLine {
    fn push(&mut self, column: usize, look: CellLook, default_background: u32) {
        if look.background != default_background {
            self.push_fill(column, look.cells, look.background);
        }
        self.push_text(column, &look);
    }

    fn push_fill(&mut self, column: usize, width: usize, color: u32) {
        match self.fills.last_mut() {
            Some(last) if last.color == color && last.column + last.width == column => last.width += width,
            _ => self.fills.push(Fill { column, width, color }),
        }
    }

    fn push_text(&mut self, column: usize, look: &CellLook) {
        let wide = look.cells > 1;
        let continues = self
            .segments
            .last()
            .is_some_and(|last| !last.wide && !wide && last.column + last.cells == column);
        if !continues {
            self.segments.push(Segment { column, cells: 0, text: String::new(), runs: Vec::new(), wide });
        }
        let Some(segment) = self.segments.last_mut() else {
            return;
        };
        segment.cells += look.cells;
        segment.text.push(look.character);
        let len = look.character.len_utf8();
        match segment.runs.last_mut() {
            Some(run) if run.style == look.style => run.len += len,
            _ => segment.runs.push(Run { len, style: look.style }),
        }
    }
}

#[cfg(test)]
mod tests {
    use alacritty_terminal::event::VoidListener;
    use alacritty_terminal::index::Column;
    use alacritty_terminal::term::Config;
    use alacritty_terminal::vte::ansi::Processor;

    use super::*;
    use crate::terminal::session::GridSize;
    use crate::theme::{Appearance, Theme};

    fn render(bytes: &[u8], focused: bool) -> (Frame, TerminalPalette) {
        let size = GridSize { columns: 20, lines: 3, cell_width: 8, cell_height: 16 };
        let mut term = Term::new(Config::default(), &size, VoidListener);
        Processor::<alacritty_terminal::vte::ansi::StdSyncHandler>::new().advance(&mut term, bytes);
        let palette = Theme::of(Appearance::Dark).terminal;
        (layout(term.renderable_content(), &palette, focused), palette)
    }

    fn line_text(frame: &Frame, line: usize) -> String {
        frame.lines[line].segments.iter().map(|segment| segment.text.as_str()).collect::<String>().trim_end().to_owned()
    }

    #[test]
    fn should_split_line_into_runs_by_color_and_weight() {
        // given
        let output = b"ok \x1b[31mred\x1b[1mbold\x1b[0m";

        // when
        let (frame, palette) = render(output, true);

        // then
        let runs: Vec<_> = frame.lines[0].segments[0].runs.iter().take(3).map(|run| (run.len, run.style.color, run.style.bold)).collect();
        assert_eq!(runs, [(3, palette.foreground, false), (3, palette.ansi[1], false), (4, palette.ansi[9], true)]);
    }

    #[test]
    fn should_fill_only_backgrounds_that_differ_from_terminal_background() {
        // given
        let output = b"ab\x1b[44mcd\x1b[0mef";

        // when
        // unfocused, so the cursor is hollow and paints no fill of its own
        let (frame, palette) = render(output, false);

        // then
        assert_eq!(frame.lines[0].fills, [Fill { column: 2, width: 2, color: palette.ansi[4] }]);
    }

    #[test]
    fn should_give_wide_characters_their_own_two_cell_segment() {
        // given
        let output = "a日b".as_bytes();

        // when
        let (frame, _) = render(output, true);

        // then
        let segments: Vec<_> =
            frame.lines[0].segments.iter().map(|s| (s.column, s.text.trim_end(), s.wide)).collect();
        assert_eq!(segments, [(0, "a", false), (1, "日", true), (3, "b", false)]);
    }

    #[test]
    fn should_invert_cell_under_focused_block_cursor() {
        // given
        let output = b"$ ";

        // when
        let (frame, palette) = render(output, true);

        // then
        let cursor_fill = Fill { column: 2, width: 1, color: palette.cursor };
        assert_eq!((frame.lines[0].fills.as_slice(), frame.cursor), (&[cursor_fill][..], None));
    }

    #[test]
    fn should_mark_hollow_cursor_when_unfocused() {
        // given
        let output = b"$ ";

        // when
        let (frame, _) = render(output, false);

        // then
        let expected = CursorMark { line: 0, column: 2, cells: 1, shape: CursorShape::HollowBlock };
        assert_eq!((frame.lines[0].fills.is_empty(), frame.cursor), (true, Some(expected)));
    }

    #[test]
    fn should_lay_out_every_screen_line() {
        // given
        let output = b"one\r\ntwo\r\nthree";

        // when
        let (frame, _) = render(output, true);

        // then
        let lines: Vec<_> = (0..frame.lines.len()).map(|line| line_text(&frame, line)).collect();
        assert_eq!(lines, ["one", "two", "three"]);
    }

    fn scrolled_term() -> Term<VoidListener> {
        let size = GridSize { columns: 20, lines: 3, cell_width: 8, cell_height: 16 };
        let mut term = Term::new(Config { scrolling_history: 100, ..Config::default() }, &size, VoidListener);
        let printed: Vec<String> = (0..10).map(|line| format!("l{line}")).collect();
        Processor::<alacritty_terminal::vte::ansi::StdSyncHandler>::new().advance(&mut term, printed.join("\r\n").as_bytes());
        term
    }

    #[test]
    fn should_lay_out_only_the_window_rows() {
        // given
        let term = scrolled_term();
        let palette = Theme::of(Appearance::Dark).terminal;

        // when
        let frame = layout_window(&term, &Window { top: -5, rows: 2, mark: None }, &palette);

        // then
        let lines: Vec<_> = (0..frame.lines.len()).map(|line| line_text(&frame, line)).collect();
        assert_eq!(lines, ["l2", "l3"]);
    }

    #[test]
    fn should_mark_the_window_selection() {
        // given
        let term = scrolled_term();
        let palette = Theme::of(Appearance::Dark).terminal;
        let mark = (Point::new(Line(-4), Column(0)), Point::new(Line(-4), Column(1)));

        // when
        let frame = layout_window(&term, &Window { top: -5, rows: 2, mark: Some(mark) }, &palette);

        // then
        assert_eq!((frame.lines[0].fills.len(), frame.lines[1].fills.clone()), (0, vec![Fill { column: 0, width: 2, color: palette.selection }]));
    }

    #[test]
    fn should_not_move_the_terminal_display_offset() {
        // given
        let term = scrolled_term();
        let palette = Theme::of(Appearance::Dark).terminal;

        // when
        layout_window(&term, &Window { top: -5, rows: 2, mark: None }, &palette);

        // then
        assert_eq!(term.grid().display_offset(), 0);
    }

    #[test]
    fn should_put_the_line_a_third_down() {
        // given
        let (line, rows, history, screen) = (-40, 30, 100, 24);

        // when
        let top = top_showing(line, rows, history, screen);

        // then
        assert_eq!(top, -50);
    }

    #[test]
    fn should_clamp_to_the_top_of_history() {
        // given
        let (line, rows, history, screen) = (-98, 30, 100, 24);

        // when
        let top = top_showing(line, rows, history, screen);

        // then
        assert_eq!(top, -100);
    }

    #[test]
    fn should_stay_at_the_bottom_for_a_screen_line() {
        // given
        let (line, rows, history, screen) = (20, 24, 100, 24);

        // when
        let top = top_showing(line, rows, history, screen);

        // then
        assert_eq!(top, 0);
    }

    #[test]
    fn should_shift_a_match_past_the_view_into_it() {
        // given
        let (columns, visible) = (100..106, 80);

        // when
        let shifted = [columns_shift(&columns, visible), columns_shift(&(10..16), visible), columns_shift(&(0..300), visible)];

        // then
        assert_eq!(shifted, [28, 0, 0]);
    }
}
