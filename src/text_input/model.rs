//! The text of a single-line input, its selection and IME composition; offsets are UTF-8 bytes.

use std::ops::Range;

use unicode_segmentation::UnicodeSegmentation;

pub const MAX_CHARS: usize = 1_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    Char,
    Word,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Movement {
    Move,
    /// extends the selection, as with Shift held
    Select,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EditModel {
    text: String,
    selection: Range<usize>,
    /// the caret is at the selection's start
    reversed: bool,
    /// text an IME is still composing
    marked: Option<Range<usize>>,
}

impl EditModel {
    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn selection(&self) -> Range<usize> {
        self.selection.clone()
    }

    pub fn marked(&self) -> Option<Range<usize>> {
        self.marked.clone()
    }

    pub fn cursor(&self) -> usize {
        if self.reversed { self.selection.start } else { self.selection.end }
    }

    pub fn selected_text(&self) -> Option<&str> {
        if self.selection.is_empty() {
            return None;
        }
        Some(&self.text[self.selection.clone()])
    }

    /// Replaces everything, caret at the end.
    pub fn set_text(&mut self, text: &str) {
        self.text = single_line(text).chars().take(MAX_CHARS).collect();
        self.marked = None;
        self.move_to(self.text.len());
    }

    pub fn select_all(&mut self) {
        self.selection = 0..self.text.len();
        self.reversed = false;
    }

    /// Selects `range`, clamped to the text and to character boundaries.
    pub fn select_range(&mut self, range: Range<usize>) {
        let start = self.text.floor_char_boundary(range.start);
        let end = self.text.floor_char_boundary(range.end.max(range.start));
        self.selection = start..end;
        self.reversed = false;
    }

    pub fn move_to(&mut self, offset: usize) {
        let offset = self.text.floor_char_boundary(offset);
        self.selection = offset..offset;
        self.reversed = false;
    }

    /// Moves the caret end of the selection to `offset`, keeping the other end.
    pub fn select_to(&mut self, offset: usize) {
        let offset = self.text.floor_char_boundary(offset);
        if self.reversed {
            self.selection.start = offset;
        } else {
            self.selection.end = offset;
        }
        if self.selection.end < self.selection.start {
            self.reversed = !self.reversed;
            self.selection = self.selection.end..self.selection.start;
        }
    }

    pub fn left(&mut self, movement: Movement) {
        match movement {
            Movement::Select => self.select_to(self.previous_boundary(self.cursor())),
            Movement::Move if self.selection.is_empty() => self.move_to(self.previous_boundary(self.cursor())),
            Movement::Move => self.move_to(self.selection.start),
        }
    }

    pub fn right(&mut self, movement: Movement) {
        match movement {
            Movement::Select => self.select_to(self.next_boundary(self.cursor())),
            Movement::Move if self.selection.is_empty() => self.move_to(self.next_boundary(self.cursor())),
            Movement::Move => self.move_to(self.selection.end),
        }
    }

    pub fn home(&mut self, movement: Movement) {
        match movement {
            Movement::Move => self.move_to(0),
            Movement::Select => self.select_to(0),
        }
    }

    pub fn end(&mut self, movement: Movement) {
        match movement {
            Movement::Move => self.move_to(self.text.len()),
            Movement::Select => self.select_to(self.text.len()),
        }
    }

    /// Replaces `range` (else the composed text, else the selection) with `text`, flattened to one line and cut so
    /// the whole stays within [`MAX_CHARS`].
    pub fn replace(&mut self, range: Option<Range<usize>>, text: &str) -> bool {
        let range = self.target(range);
        let inserted = self.fitting(&range, text);
        let changed = !range.is_empty() || !inserted.is_empty();
        self.text.replace_range(range.clone(), &inserted);
        self.marked = None;
        self.move_to(range.start + inserted.len());
        changed
    }

    /// Like [`Self::replace`], keeping the inserted text marked as IME composition; `selected` is relative to it.
    pub fn replace_and_mark(&mut self, range: Option<Range<usize>>, text: &str, selected: Option<Range<usize>>) {
        let range = self.target(range);
        let inserted = self.fitting(&range, text);
        self.text.replace_range(range.clone(), &inserted);
        let end = range.start + inserted.len();
        self.marked = (!inserted.is_empty()).then_some(range.start..end);
        match selected {
            Some(selected) => self.select_range((range.start + selected.start).min(end)..(range.start + selected.end).min(end)),
            None => self.move_to(end),
        }
    }

    pub fn unmark(&mut self) {
        self.marked = None;
    }

    /// Backspace: the selection, else the grapheme or word before the caret.
    pub fn delete_backward(&mut self, unit: Unit) -> bool {
        if self.selection.is_empty() {
            let cursor = self.cursor();
            let start = match unit {
                Unit::Char => self.previous_boundary(cursor),
                Unit::Word => self.word_start(cursor),
            };
            self.selection = start..cursor;
        }
        self.replace(None, "")
    }

    /// Delete: the selection, else the grapheme or word after the caret.
    pub fn delete_forward(&mut self, unit: Unit) -> bool {
        if self.selection.is_empty() {
            let cursor = self.cursor();
            let end = match unit {
                Unit::Char => self.next_boundary(cursor),
                Unit::Word => self.word_end(cursor),
            };
            self.selection = cursor..end;
        }
        self.replace(None, "")
    }

    pub fn utf16_of(&self, offset: usize) -> usize {
        self.text[..self.text.floor_char_boundary(offset)].encode_utf16().count()
    }

    pub fn offset_of_utf16(&self, offset: usize) -> usize {
        let mut units = 0;
        for (index, character) in self.text.char_indices() {
            if units >= offset {
                return index;
            }
            units += character.len_utf16();
        }
        self.text.len()
    }

    pub fn utf16_range(&self, range: &Range<usize>) -> Range<usize> {
        self.utf16_of(range.start)..self.utf16_of(range.end)
    }

    /// Clamped to the text, start never past end: the platform input handler may ask for any range.
    pub fn range_of_utf16(&self, range: &Range<usize>) -> Range<usize> {
        let start = self.offset_of_utf16(range.start);
        start..self.offset_of_utf16(range.end).max(start)
    }

    fn target(&self, range: Option<Range<usize>>) -> Range<usize> {
        let range = range.or_else(|| self.marked.clone()).unwrap_or_else(|| self.selection.clone());
        let start = self.text.floor_char_boundary(range.start);
        start..self.text.floor_char_boundary(range.end.max(start))
    }

    /// `text` on one line, cut to the room left once `range` is replaced.
    fn fitting(&self, range: &Range<usize>, text: &str) -> String {
        let kept = self.text.chars().count() - self.text[range.clone()].chars().count();
        single_line(text).chars().take(MAX_CHARS.saturating_sub(kept)).collect()
    }

    fn previous_boundary(&self, offset: usize) -> usize {
        self.text.grapheme_indices(true).rev().find_map(|(index, _)| (index < offset).then_some(index)).unwrap_or(0)
    }

    fn next_boundary(&self, offset: usize) -> usize {
        self.text.grapheme_indices(true).find_map(|(index, _)| (index > offset).then_some(index)).unwrap_or(self.text.len())
    }

    /// Skips separators, then the word before them.
    fn word_start(&self, offset: usize) -> usize {
        let before = self.text[..offset].trim_end_matches(|character: char| !is_word(character));
        before.trim_end_matches(is_word).len()
    }

    fn word_end(&self, offset: usize) -> usize {
        let after = self.text[offset..].trim_start_matches(|character: char| !is_word(character));
        self.text.len() - after.trim_start_matches(is_word).len()
    }
}

fn is_word(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
}

fn single_line(text: &str) -> String {
    text.replace("\r\n", " ").replace(['\n', '\r'], " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model(text: &str) -> EditModel {
        let mut model = EditModel::default();
        model.set_text(text);
        model
    }

    #[test]
    fn should_insert_at_the_caret() {
        // given
        let mut input = model("helo");
        input.move_to(3);

        // when
        let changed = input.replace(None, "l");

        // then
        assert_eq!((changed, input.text(), input.cursor()), (true, "hello", 4));
    }

    #[test]
    fn should_replace_the_selection_and_put_the_caret_after_it() {
        // given
        let mut input = model("scratch.md");
        input.select_range(0..7);

        // when
        input.replace(None, "notes");

        // then
        assert_eq!((input.text(), input.selection()), ("notes.md", 5..5));
    }

    #[test]
    fn should_delete_whole_words_with_ctrl() {
        // given
        let mut input = model("git log --all");

        // when
        input.delete_backward(Unit::Word);
        let after_backward = input.text().to_owned();
        input.move_to(0);
        input.delete_forward(Unit::Word);

        // then
        assert_eq!((after_backward.as_str(), input.text()), ("git log --", " log --"));
    }

    #[test]
    fn should_remove_a_whole_grapheme_with_backspace() {
        // given
        let mut input = model("ok 👍🏽");

        // when
        input.delete_backward(Unit::Char);

        // then
        assert_eq!(input.text(), "ok ");
    }

    #[test]
    fn should_cut_pasted_text_at_the_character_limit_and_flatten_lines() {
        // given
        let mut input = model(&"ż".repeat(MAX_CHARS - 3));

        // when
        input.replace(None, "a\nb\r\ncdef");

        // then
        assert_eq!((input.text().chars().count(), input.text().ends_with("a b")), (MAX_CHARS, true));
    }

    #[test]
    fn should_extend_the_selection_with_shift_moves() {
        // given
        let mut input = model("abcd");
        input.move_to(2);

        // when
        input.left(Movement::Select);
        input.left(Movement::Select);
        let selected = input.selected_text().map(str::to_owned);
        input.end(Movement::Select);

        // then
        assert_eq!((selected.as_deref(), input.selection()), (Some("ab"), 2..4));
    }

    #[test]
    fn should_convert_offsets_to_and_from_utf16() {
        // given
        let input = model("a😀b");

        // when
        let there = input.utf16_of(5);
        let back = input.offset_of_utf16(there);

        // then
        assert_eq!((there, back), (3, 5));
    }

    #[test]
    fn should_clamp_utf16_ranges_past_the_end_or_reversed() {
        // given
        let input = model("a😀b");
        let reversed = Range { start: 4, end: 1 };

        // when
        let ranges = [input.range_of_utf16(&(1..99)), input.range_of_utf16(&reversed)];

        // then
        assert_eq!(ranges, [1..6, 6..6]);
    }

    #[test]
    fn should_report_no_change_when_deleting_at_the_start() {
        // given
        let mut input = model("abc");
        input.move_to(0);

        // when
        let changed = input.delete_backward(Unit::Char);

        // then
        assert_eq!((changed, input.text()), (false, "abc"));
    }
}
