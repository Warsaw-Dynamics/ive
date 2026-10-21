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

#[derive(Debug, PartialEq, Eq)]
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
    let resolver = ColorResolver { palette, overrides: content.colors };
    let offset = content.display_offset as i32;
    let cursor_line = usize::try_from(content.cursor.point.line.0 + offset).ok();
    let cursor_shape = match content.cursor.shape {
        CursorShape::Block if !focused => CursorShape::HollowBlock,
        shape => shape,
    };
    let mut frame = Frame::default();
    for indexed in content.display_iter {
        let Ok(line) = usize::try_from(indexed.point.line.0 + offset) else {
            continue;
        };
        if indexed.cell.flags.intersects(Flags::WIDE_CHAR_SPACER | Flags::LEADING_WIDE_CHAR_SPACER) {
            continue;
        }
        let column = indexed.point.column.0;
        let selected = content.selection.is_some_and(|selection| selection.contains(indexed.point));
        let on_cursor = cursor_line == Some(line) && content.cursor.point.column.0 == column;
        let mut look = CellLook::of(indexed.cell, &resolver);
        if selected {
            look.background = palette.selection;
        }
        if on_cursor && cursor_shape == CursorShape::Block {
            look = look.under_block_cursor(palette.cursor);
        }
        if on_cursor && !matches!(cursor_shape, CursorShape::Block | CursorShape::Hidden) {
            frame.cursor = Some(CursorMark { line, column, cells: look.cells, shape: cursor_shape });
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
    use alacritty_terminal::Term;
    use alacritty_terminal::event::VoidListener;
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
}
