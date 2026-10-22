//! The text being edited: lines, a caret and a selection anchor, and the edits on them.

use std::fmt;
use std::ops::Range;

use super::history::{Change, Cursor, Edit, History};

/// What Tab inserts.
pub const INDENT: &str = "    ";

/// A place in the text: a line, and a column counted in characters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Position {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

/// The largest text accepted, counted in bytes as written to disk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SizeLimit {
    pub max_bytes: usize,
    /// 2 for files written back with CRLF
    pub newline_bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditError {
    TooLarge { limit: usize },
}

impl fmt::Display for EditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooLarge { limit } => {
                write!(f, "refused: the scratch would pass its {:.1} MiB limit", *limit as f64 / (1024. * 1024.))
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Motion {
    Left,
    Right,
    WordLeft,
    WordRight,
    Up,
    Down,
    LineStart,
    LineEnd,
    DocumentStart,
    DocumentEnd,
    PageUp(usize),
    PageDown(usize),
}

/// How far Backspace and Delete reach: a character, or with Ctrl a word.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    Char,
    Word,
}

/// Whether a caret move drops the selection or, with Shift, extends it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Selecting {
    Move,
    Extend,
}

/// How an edit joins the undo history.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditKind {
    Typing,
    Other,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CharClass {
    Space,
    Word,
    Punctuation,
}

fn class(c: char) -> CharClass {
    if c.is_whitespace() {
        CharClass::Space
    } else if c.is_alphanumeric() || c == '_' {
        CharClass::Word
    } else {
        CharClass::Punctuation
    }
}

pub struct EditorModel {
    /// never empty; LF separates them
    lines: Vec<String>,
    caret: Position,
    anchor: Option<Position>,
    /// column Up and Down aim for, kept across short lines
    goal_column: Option<usize>,
    /// text being composed by an input method
    marked: Option<(Position, Position)>,
    history: History,
    limit: SizeLimit,
    revision: u64,
}

impl EditorModel {
    /// `text` has LF line breaks.
    pub fn new(text: &str, limit: SizeLimit) -> Self {
        Self {
            lines: text.split('\n').map(str::to_owned).collect(),
            caret: Position::default(),
            anchor: None,
            goal_column: None,
            marked: None,
            history: History::default(),
            limit,
            revision: 0,
        }
    }

    pub fn text(&self) -> String {
        self.lines.join("\n")
    }

    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn caret(&self) -> Position {
        self.caret
    }

    /// Changes with every edit, undo and redo; the tab compares it with the saved one.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Ordered; `None` when nothing is selected.
    pub fn selection(&self) -> Option<Range<Position>> {
        let anchor = self.anchor.filter(|anchor| *anchor != self.caret)?;
        Some(anchor.min(self.caret)..anchor.max(self.caret))
    }

    pub fn selected_text(&self) -> Option<String> {
        self.selection().map(|range| self.text_in(range.start, range.end))
    }

    /// Size as written to disk.
    pub fn byte_len(&self) -> usize {
        self.lines.iter().map(String::len).sum::<usize>() + (self.lines.len() - 1) * self.limit.newline_bytes
    }

    pub fn clamp(&self, position: Position) -> Position {
        let line = position.line.min(self.lines.len() - 1);
        Position::new(line, position.column.min(self.line_chars(line)))
    }

    /// Replaces the selection (or the text being composed) with `text`; CRLF and CR become line breaks.
    pub fn insert(&mut self, text: &str, kind: EditKind) -> Result<(), EditError> {
        let range = self.edit_range(None);
        self.replace(range, &normalize(text), kind)
    }

    pub fn backspace(&mut self, unit: Unit) -> Result<(), EditError> {
        if let Some(selection) = self.selection() {
            return self.replace(selection, "", EditKind::Other);
        }
        let from = match unit {
            Unit::Char => self.target(Motion::Left),
            Unit::Word => self.word_left(),
        };
        self.replace(from..self.caret, "", EditKind::Other)
    }

    pub fn delete_forward(&mut self, unit: Unit) -> Result<(), EditError> {
        if let Some(selection) = self.selection() {
            return self.replace(selection, "", EditKind::Other);
        }
        let to = match unit {
            Unit::Char => self.target(Motion::Right),
            Unit::Word => self.word_right(),
        };
        self.replace(self.caret..to, "", EditKind::Other)
    }

    pub fn delete_selection(&mut self) -> Result<(), EditError> {
        match self.selection() {
            Some(selection) => self.replace(selection, "", EditKind::Other),
            None => Ok(()),
        }
    }

    pub fn move_caret(&mut self, motion: Motion, selecting: Selecting) {
        let target = match (motion, self.selection(), selecting) {
            // without Shift, Left and Right first collapse a selection to its edge
            (Motion::Left, Some(selection), Selecting::Move) => selection.start,
            (Motion::Right, Some(selection), Selecting::Move) => selection.end,
            _ => self.target(motion),
        };
        let vertical = matches!(motion, Motion::Up | Motion::Down | Motion::PageUp(_) | Motion::PageDown(_));
        let goal = vertical.then(|| self.goal_column.unwrap_or(self.caret.column));
        self.place_caret(target, selecting);
        self.goal_column = goal;
    }

    /// For the mouse: places the caret, or with `Selecting::Extend` moves the selection's end there.
    pub fn set_caret(&mut self, position: Position, selecting: Selecting) {
        let position = self.clamp(position);
        self.place_caret(position, selecting);
    }

    pub fn select_all(&mut self) {
        self.place_caret(Position::default(), Selecting::Move);
        self.place_caret(self.end(), Selecting::Extend);
    }

    /// The run of word, space or punctuation characters at `position`.
    pub fn select_word(&mut self, position: Position) {
        let position = self.clamp(position);
        let chars: Vec<char> = self.lines[position.line].chars().collect();
        // the character under the click, or the one before it at the end of a line
        let Some(probe) = (position.column < chars.len()).then_some(position.column).or(position.column.checked_sub(1)) else {
            return self.place_caret(position, Selecting::Move);
        };
        let kind = class(chars[probe]);
        let start = (0..probe).rev().take_while(|&at| class(chars[at]) == kind).last().unwrap_or(probe);
        let end = (probe + 1..chars.len()).take_while(|&at| class(chars[at]) == kind).last().map_or(probe + 1, |at| at + 1);
        self.place_caret(Position::new(position.line, start), Selecting::Move);
        self.place_caret(Position::new(position.line, end), Selecting::Extend);
    }

    /// The whole line with its line break.
    pub fn select_line(&mut self, line: usize) {
        let line = line.min(self.lines.len() - 1);
        let end = if line + 1 < self.lines.len() { Position::new(line + 1, 0) } else { self.end() };
        self.place_caret(Position::new(line, 0), Selecting::Move);
        self.place_caret(end, Selecting::Extend);
    }

    pub fn utf16_len(&self) -> usize {
        self.lines.iter().map(|line| line.encode_utf16().count()).sum::<usize>() + self.lines.len() - 1
    }

    pub fn offset_utf16(&self, position: Position) -> usize {
        let before: usize = self.lines[..position.line].iter().map(|line| line.encode_utf16().count() + 1).sum();
        before + self.lines[position.line].chars().take(position.column).map(char::len_utf16).sum::<usize>()
    }

    /// An offset inside a surrogate pair maps to the character's start.
    pub fn position_utf16(&self, offset: usize) -> Position {
        let mut remaining = offset;
        for (index, line) in self.lines.iter().enumerate() {
            let units = line.encode_utf16().count();
            if remaining <= units {
                return Position::new(index, column_at_utf16(line, remaining));
            }
            remaining -= units + 1;
        }
        self.end()
    }

    pub fn text_utf16(&self, range: Range<usize>) -> String {
        let (start, end) = (range.start.min(range.end), range.start.max(range.end));
        self.text_in(self.position_utf16(start), self.position_utf16(end))
    }

    /// The selection, or the caret as an empty range; and whether the caret is at its start.
    pub fn selection_utf16(&self) -> (Range<usize>, bool) {
        let anchor = self.anchor.unwrap_or(self.caret);
        let (start, end) = (anchor.min(self.caret), anchor.max(self.caret));
        (self.offset_utf16(start)..self.offset_utf16(end), self.caret < anchor)
    }

    pub fn marked_utf16(&self) -> Option<Range<usize>> {
        self.marked.map(|(start, end)| self.offset_utf16(start)..self.offset_utf16(end))
    }

    /// Text from the keyboard or an input method; replaces `range`, else the text being composed, else the
    /// selection.
    pub fn replace_utf16(&mut self, range: Option<Range<usize>>, text: &str) -> Result<(), EditError> {
        let range = self.edit_range(range);
        self.replace(range, &normalize(text), EditKind::Typing)
    }

    /// Text an input method is composing; `selected` is relative to the composed text.
    pub fn mark_utf16(&mut self, range: Option<Range<usize>>, text: &str, selected: Option<Range<usize>>) -> Result<(), EditError> {
        let range = self.edit_range(range);
        let text = normalize(text);
        self.replace(range.clone(), &text, EditKind::Typing)?;
        self.marked = Some((range.start, end_of(range.start, &text))).filter(|(start, end)| start != end);
        if let Some(selected) = selected {
            let base = self.offset_utf16(range.start);
            self.anchor = Some(self.position_utf16(base + selected.start));
            self.caret = self.position_utf16(base + selected.end);
        }
        Ok(())
    }

    pub fn unmark(&mut self) {
        self.marked = None;
    }

    pub fn undo(&mut self) -> bool {
        let Some(group) = self.history.pop_undo() else {
            return false;
        };
        for change in group.changes.iter().rev() {
            self.splice(change.start, end_of(change.start, &change.inserted), &change.removed);
        }
        self.restore(group.before);
        self.history.push_redo(group);
        true
    }

    pub fn redo(&mut self) -> bool {
        let Some(group) = self.history.pop_redo() else {
            return false;
        };
        for change in &group.changes {
            self.splice(change.start, end_of(change.start, &change.removed), &change.inserted);
        }
        self.restore(group.after);
        self.history.push_undone(group);
        true
    }

    fn restore(&mut self, cursor: Cursor) {
        self.caret = cursor.caret;
        self.anchor = cursor.anchor;
        self.goal_column = None;
        self.marked = None;
        self.revision += 1;
    }

    fn edit_range(&self, range: Option<Range<usize>>) -> Range<Position> {
        match (range, self.marked) {
            (Some(range), _) => {
                self.position_utf16(range.start.min(range.end))..self.position_utf16(range.start.max(range.end))
            }
            (None, Some((start, end))) => start..end,
            (None, None) => self.selection().unwrap_or(self.caret..self.caret),
        }
    }

    /// Every edit goes through here: size limit, history, caret.
    fn replace(&mut self, range: Range<Position>, text: &str, kind: EditKind) -> Result<(), EditError> {
        if range.is_empty() && text.is_empty() {
            return Ok(());
        }
        let removed = self.text_in(range.start, range.end);
        let (added, dropped) = (self.written_len(text), self.written_len(&removed));
        if added > dropped && self.byte_len() - dropped + added > self.limit.max_bytes {
            return Err(EditError::TooLarge { limit: self.limit.max_bytes });
        }
        let before = self.cursor();
        self.caret = self.splice(range.start, range.end, text);
        self.anchor = None;
        self.goal_column = None;
        self.marked = None;
        self.revision += 1;
        let change = Change { start: range.start, removed, inserted: text.to_owned() };
        self.history.record(Edit { change, kind, before, after: self.cursor() });
        Ok(())
    }

    /// Replaces `start..end` with `text` (LF only); returns where the inserted text ends.
    fn splice(&mut self, start: Position, end: Position, text: &str) -> Position {
        let head = &self.lines[start.line][..byte_at(&self.lines[start.line], start.column)];
        let tail = &self.lines[end.line][byte_at(&self.lines[end.line], end.column)..];
        let joined = format!("{head}{text}{tail}");
        self.lines.splice(start.line..=end.line, joined.split('\n').map(str::to_owned));
        end_of(start, text)
    }

    fn text_in(&self, start: Position, end: Position) -> String {
        let (first, last) = (&self.lines[start.line], &self.lines[end.line]);
        if start.line == end.line {
            return first[byte_at(first, start.column)..byte_at(first, end.column)].to_owned();
        }
        let mut parts = vec![&first[byte_at(first, start.column)..]];
        parts.extend(self.lines[start.line + 1..end.line].iter().map(String::as_str));
        parts.push(&last[..byte_at(last, end.column)]);
        parts.join("\n")
    }

    fn written_len(&self, text: &str) -> usize {
        text.len() + text.matches('\n').count() * (self.limit.newline_bytes - 1)
    }

    fn place_caret(&mut self, target: Position, selecting: Selecting) {
        self.anchor = match selecting {
            Selecting::Move => None,
            Selecting::Extend => self.anchor.or(Some(self.caret)),
        };
        self.caret = target;
        self.goal_column = None;
        self.marked = None;
        self.history.seal();
    }

    fn cursor(&self) -> Cursor {
        Cursor { caret: self.caret, anchor: self.anchor }
    }

    fn target(&self, motion: Motion) -> Position {
        let Position { line, column } = self.caret;
        let goal = self.goal_column.unwrap_or(column);
        let last = self.lines.len() - 1;
        match motion {
            Motion::Left if column > 0 => Position::new(line, column - 1),
            Motion::Left if line > 0 => Position::new(line - 1, self.line_chars(line - 1)),
            Motion::Right if column < self.line_chars(line) => Position::new(line, column + 1),
            Motion::Right if line < last => Position::new(line + 1, 0),
            Motion::Left | Motion::Right => self.caret,
            Motion::WordLeft => self.word_left(),
            Motion::WordRight => self.word_right(),
            Motion::Up if line == 0 => Position::default(),
            Motion::Up => self.at_line(line - 1, goal),
            Motion::Down if line == last => self.end(),
            Motion::Down => self.at_line(line + 1, goal),
            Motion::LineStart => Position::new(line, 0),
            Motion::LineEnd => Position::new(line, self.line_chars(line)),
            Motion::DocumentStart => Position::default(),
            Motion::DocumentEnd => self.end(),
            Motion::PageUp(rows) => self.at_line(line.saturating_sub(rows), goal),
            Motion::PageDown(rows) => self.at_line((line + rows).min(last), goal),
        }
    }

    /// Skips spaces, then one run of the same class; at a line start, goes to the previous line's end.
    fn word_left(&self) -> Position {
        let Position { line, column } = self.caret;
        if column == 0 {
            return self.target(Motion::Left);
        }
        let chars: Vec<char> = self.lines[line].chars().take(column).collect();
        let spaces = chars.iter().rev().take_while(|c| class(**c) == CharClass::Space).count();
        let at = column - spaces;
        let run = chars[..at].last().map_or(0, |last| chars[..at].iter().rev().take_while(|c| class(**c) == class(*last)).count());
        Position::new(line, at - run)
    }

    /// Skips one run of the same class, then spaces; at a line end, goes to the next line's start.
    fn word_right(&self) -> Position {
        let Position { line, column } = self.caret;
        let chars: Vec<char> = self.lines[line].chars().collect();
        if column >= chars.len() {
            return self.target(Motion::Right);
        }
        let kind = class(chars[column]);
        let run = chars[column..].iter().take_while(|c| class(**c) == kind).count();
        let spaces = chars[column + run..].iter().take_while(|c| class(**c) == CharClass::Space).count();
        Position::new(line, column + run + spaces)
    }

    fn at_line(&self, line: usize, goal: usize) -> Position {
        Position::new(line, goal.min(self.line_chars(line)))
    }

    fn end(&self) -> Position {
        let line = self.lines.len() - 1;
        Position::new(line, self.line_chars(line))
    }

    fn line_chars(&self, line: usize) -> usize {
        self.lines[line].chars().count()
    }
}

fn normalize(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

/// Byte index of character `column`, or the line's end.
fn byte_at(line: &str, column: usize) -> usize {
    line.char_indices().nth(column).map_or(line.len(), |(index, _)| index)
}

/// Where `text` ends when inserted at `start`.
fn end_of(start: Position, text: &str) -> Position {
    match text.rfind('\n') {
        None => Position::new(start.line, start.column + text.chars().count()),
        Some(last) => Position::new(start.line + text.matches('\n').count(), text[last + 1..].chars().count()),
    }
}

fn column_at_utf16(line: &str, units: usize) -> usize {
    let mut seen = 0;
    line.chars()
        .take_while(|c| {
            seen += c.len_utf16();
            seen <= units
        })
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model(text: &str) -> EditorModel {
        EditorModel::new(text, SizeLimit { max_bytes: 1024 * 1024, newline_bytes: 1 })
    }

    #[test]
    fn should_insert_typed_text_at_the_caret() {
        // given
        let mut model = model("ac");
        model.set_caret(Position::new(0, 1), Selecting::Move);

        // when
        model.insert("b", EditKind::Typing).unwrap();

        // then
        assert_eq!((model.text().as_str(), model.caret()), ("abc", Position::new(0, 2)));
    }

    #[test]
    fn should_split_and_join_lines_with_newline_backspace_and_delete() {
        // given
        let mut model = model("abcd");
        model.set_caret(Position::new(0, 2), Selecting::Move);

        // when
        model.insert("\n", EditKind::Other).unwrap();
        let split = model.text();
        model.backspace(Unit::Char).unwrap();
        let joined = model.text();
        model.move_caret(Motion::DocumentEnd, Selecting::Move);
        model.delete_forward(Unit::Char).unwrap();

        // then
        assert_eq!((split.as_str(), joined.as_str(), model.text().as_str()), ("ab\ncd", "abcd", "abcd"));
    }

    #[test]
    fn should_insert_four_spaces_for_tab_and_turn_crlf_and_cr_into_line_breaks() {
        // given
        let mut model = model("");

        // when
        model.insert(INDENT, EditKind::Other).unwrap();
        model.insert("a\r\nb\rc", EditKind::Other).unwrap();

        // then
        assert_eq!((model.text().as_str(), model.caret()), ("    a\nb\nc", Position::new(2, 1)));
    }

    #[test]
    fn should_replace_the_selection_with_typed_text() {
        // given
        let mut model = model("hello world");
        model.set_caret(Position::new(0, 6), Selecting::Move);
        model.move_caret(Motion::LineEnd, Selecting::Extend);

        // when
        model.insert("there", EditKind::Typing).unwrap();

        // then
        assert_eq!((model.text().as_str(), model.selection()), ("hello there", None));
    }

    #[test]
    fn should_extend_the_selection_with_shift_and_collapse_it_without() {
        // given
        let mut model = model("hello world");

        // when
        model.move_caret(Motion::Right, Selecting::Extend);
        model.move_caret(Motion::Right, Selecting::Extend);
        let extended = model.selected_text();
        model.move_caret(Motion::Right, Selecting::Move);
        let collapsed = (model.selection(), model.caret());
        model.move_caret(Motion::LineEnd, Selecting::Extend);
        model.move_caret(Motion::WordLeft, Selecting::Extend);

        // then
        assert_eq!(extended.as_deref(), Some("he"));
        assert_eq!(collapsed, (None, Position::new(0, 2)));
        assert_eq!(model.selected_text().as_deref(), Some("llo "));
    }

    #[test]
    fn should_move_and_delete_by_words() {
        // given
        let mut model = model("foo bar_baz qux");
        model.move_caret(Motion::LineEnd, Selecting::Move);

        // when
        model.move_caret(Motion::WordLeft, Selecting::Move);
        let first = model.caret().column;
        model.move_caret(Motion::WordLeft, Selecting::Move);
        let second = model.caret().column;
        model.move_caret(Motion::WordRight, Selecting::Move);
        let back = model.caret().column;
        model.backspace(Unit::Word).unwrap();
        let deleted_left = model.text();
        model.move_caret(Motion::DocumentStart, Selecting::Move);
        model.delete_forward(Unit::Word).unwrap();

        // then
        assert_eq!((first, second, back), (12, 4, 12));
        assert_eq!((deleted_left.as_str(), model.text().as_str()), ("foo qux", "qux"));
    }

    #[test]
    fn should_cross_line_ends_with_word_moves() {
        // given
        let mut model = model("ab\ncd");
        model.set_caret(Position::new(1, 0), Selecting::Move);

        // when
        model.move_caret(Motion::WordLeft, Selecting::Move);
        let left = model.caret();
        model.move_caret(Motion::WordRight, Selecting::Move);

        // then
        assert_eq!((left, model.caret()), (Position::new(0, 2), Position::new(1, 0)));
    }

    #[test]
    fn should_keep_the_goal_column_across_short_lines() {
        // given
        let mut model = model("long line here\nab\nanother long one");
        model.set_caret(Position::new(0, 10), Selecting::Move);

        // when
        model.move_caret(Motion::Down, Selecting::Move);
        let short = model.caret();
        model.move_caret(Motion::Down, Selecting::Move);
        let long = model.caret();
        model.move_caret(Motion::Down, Selecting::Move);

        // then
        assert_eq!((short, long), (Position::new(1, 2), Position::new(2, 10)));
        assert_eq!(model.caret(), Position::new(2, 16));
    }

    #[test]
    fn should_move_by_pages_and_to_the_document_ends() {
        // given
        let text = (0..100).map(|n| n.to_string()).collect::<Vec<_>>().join("\n");
        let mut model = model(&text);

        // when
        model.move_caret(Motion::PageDown(30), Selecting::Move);
        let down = model.caret();
        model.move_caret(Motion::DocumentEnd, Selecting::Extend);
        let selected_lines = model.selected_text().unwrap().lines().count();
        model.move_caret(Motion::PageUp(1000), Selecting::Move);

        // then
        assert_eq!((down, selected_lines, model.caret()), (Position::new(30, 0), 70, Position::new(0, 1)));
    }

    #[test]
    fn should_select_a_word_and_a_line_for_double_and_triple_clicks() {
        // given
        let mut model = model("foo bar_baz\nnext");

        // when
        model.select_word(Position::new(0, 6));
        let word = model.selected_text();
        model.select_line(0);
        let line = model.selected_text();
        model.select_all();

        // then
        assert_eq!((word.as_deref(), line.as_deref()), (Some("bar_baz"), Some("foo bar_baz\n")));
        assert_eq!(model.selected_text().as_deref(), Some("foo bar_baz\nnext"));
    }

    #[test]
    fn should_delete_characters_and_words_of_non_ascii_text() {
        // given
        let mut model = model("zażółć gęślą jaźń");
        model.move_caret(Motion::LineEnd, Selecting::Move);

        // when
        model.backspace(Unit::Char).unwrap();
        let one_char = model.text();
        model.backspace(Unit::Word).unwrap();
        let one_word = model.text();
        model.move_caret(Motion::DocumentStart, Selecting::Move);
        model.delete_forward(Unit::Word).unwrap();

        // then
        assert_eq!((one_char.as_str(), one_word.as_str()), ("zażółć gęślą jaź", "zażółć gęślą "));
        assert_eq!((model.text().as_str(), model.caret()), ("gęślą ", Position::new(0, 0)));
    }

    #[test]
    fn should_select_a_word_of_cjk_text_by_characters() {
        // given
        let mut model = model("说 你好世界, ok");

        // when
        model.select_word(Position::new(0, 3));

        // then
        assert_eq!((model.selected_text().as_deref(), model.caret()), (Some("你好世界"), Position::new(0, 6)));
    }

    #[test]
    fn should_refuse_an_edit_that_passes_the_size_limit_counting_crlf() {
        // given
        let mut model = EditorModel::new("ab\ncd", SizeLimit { max_bytes: 8, newline_bytes: 2 });

        // when
        let fits = model.insert("\n", EditKind::Other);
        let passes = model.insert("x", EditKind::Typing);
        let shrinks = model.backspace(Unit::Char);

        // then
        assert_eq!((fits, passes.clone()), (Ok(()), Err(EditError::TooLarge { limit: 8 })));
        assert_eq!((shrinks, model.byte_len()), (Ok(()), 6));
        assert!(passes.unwrap_err().to_string().starts_with("refused:"));
    }

    #[test]
    fn should_convert_positions_to_utf16_offsets_and_back() {
        // given
        let model = model("a😀b\nc");
        let positions = [Position::new(0, 1), Position::new(0, 2), Position::new(1, 0), Position::new(1, 1)];

        // when
        let offsets = positions.map(|position| model.offset_utf16(position));
        let back = offsets.map(|offset| model.position_utf16(offset));

        // then
        assert_eq!((offsets, back, model.utf16_len()), ([1, 3, 5, 6], positions, 6));
    }

    #[test]
    fn should_mark_composed_text_and_replace_it_when_committed() {
        // given
        let mut model = model("ab");
        model.set_caret(Position::new(0, 1), Selecting::Move);

        // when
        model.mark_utf16(None, "ka", None).unwrap();
        let composing = (model.text(), model.marked_utf16());
        model.replace_utf16(None, "か").unwrap();

        // then
        assert_eq!(composing, ("akab".to_owned(), Some(1..3)));
        assert_eq!((model.text().as_str(), model.marked_utf16(), model.caret()), ("aかb", None, Position::new(0, 2)));
    }

    #[test]
    fn should_clamp_positions_beyond_the_text() {
        // given
        let mut model = model("ab\nc");

        // when
        model.set_caret(Position::new(9, 9), Selecting::Move);

        // then
        assert_eq!(model.caret(), Position::new(1, 1));
    }

    #[test]
    fn should_count_each_edit_as_a_new_revision() {
        // given
        let mut model = model("");

        // when
        model.insert("a", EditKind::Typing).unwrap();
        model.move_caret(Motion::Left, Selecting::Move);
        model.backspace(Unit::Char).unwrap();

        // then
        assert_eq!(model.revision(), 1);
    }

    #[test]
    fn should_undo_a_run_of_typed_characters_as_one_group() {
        // given
        let mut model = model("");
        for typed in ["a", "b", "c"] {
            model.insert(typed, EditKind::Typing).unwrap();
        }

        // when
        model.undo();

        // then
        assert_eq!((model.text().as_str(), model.caret()), ("", Position::new(0, 0)));
    }

    #[test]
    fn should_start_a_new_group_after_a_caret_move_or_another_kind_of_edit() {
        // given
        let mut model = model("");
        model.insert("a", EditKind::Typing).unwrap();
        model.insert("b", EditKind::Typing).unwrap();
        model.insert("\n", EditKind::Other).unwrap();
        model.insert("c", EditKind::Typing).unwrap();
        model.move_caret(Motion::Left, Selecting::Move);
        model.move_caret(Motion::Right, Selecting::Move);
        model.insert("d", EditKind::Typing).unwrap();

        // when
        let undone: Vec<String> = (0..4)
            .map(|_| {
                model.undo();
                model.text()
            })
            .collect();

        // then
        assert_eq!(undone, ["ab\nc", "ab\n", "ab", ""]);
    }

    #[test]
    fn should_redo_undone_groups_until_a_new_edit_clears_them() {
        // given
        let mut model = model("x");
        model.move_caret(Motion::LineEnd, Selecting::Move);
        model.insert("1", EditKind::Other).unwrap();
        model.insert("2", EditKind::Other).unwrap();
        model.undo();
        model.undo();

        // when
        model.redo();
        let redone = model.text();
        model.insert("3", EditKind::Other).unwrap();
        let redo_after_edit = model.redo();

        // then
        assert_eq!((redone.as_str(), redo_after_edit, model.text().as_str()), ("x1", false, "x13"));
    }

    #[test]
    fn should_restore_the_selection_an_edit_replaced() {
        // given
        let mut model = model("hello world");
        model.set_caret(Position::new(0, 6), Selecting::Move);
        model.move_caret(Motion::LineEnd, Selecting::Extend);
        model.insert("there", EditKind::Typing).unwrap();

        // when
        model.undo();

        // then
        assert_eq!((model.text().as_str(), model.selected_text().as_deref()), ("hello world", Some("world")));
    }

    #[test]
    fn should_keep_only_the_last_200_groups() {
        // given
        let mut model = model("");
        for _ in 0..250 {
            model.insert("x", EditKind::Other).unwrap();
        }

        // when
        let undone = (0..300).take_while(|_| model.undo()).count();

        // then
        assert_eq!((undone, model.text().len()), (200, 50));
    }
}
