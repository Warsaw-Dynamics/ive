//! Editing the new side of a single file's diff: the file's full text in a piece table under an editor model, typed
//! into on the rows that show its lines (added and context rows; side by side, the right panel). Until the host
//! diffs the saved text again, the rows are its diff of the base patched with the edits since ([`super::patch`]).
//! Caret, selection and undo history belong to the text, so they outlive every new diff.

use std::ops::Range;
use std::sync::Arc;

use gpui::{
    App, Bounds, Context, EntityInputHandler, FocusHandle, KeyDownEvent, MouseDownEvent, MouseMoveEvent, Pixels, Point, ScrollStrategy,
    UTF16Selection, Window, point, px, size,
};

use super::data::{DiffBody, DiffLine, FileDiff, Hunk, LineKind};
use super::patch::{LineEdit, base_line, line_edits, patched};
use super::rows::DiffRow;
use super::side_by_side::Side;
use crate::code_view::HighlightCache;
use super::{DiffEvent, DiffView};
use crate::document::DisplayLine;
use crate::host::{ClipboardTarget, Command, Host, KeyOutcome, KeyScope};
use crate::panel::{DiffLayout, Edit, EditBase, Editable, LineSource, ParseJob, ParseTarget};
use crate::piece_table::{PieceTable, WrongBase};
use crate::scrollbar::{Axis, Scrollable};
use crate::text_editor::keys;
use crate::text_editor::model::{EditError, EditKind, EditorModel, INDENT, Position};
use crate::text_editor::{Motion, Selecting};

/// Columns kept between the caret and the edge of the text when scrolling sideways to it.
const CARET_MARGIN_CHARS: f64 = 2.;

/// Whether a diff edits its new side.
pub(crate) enum Editing {
    Off,
    /// editable once the new side's text comes, asked for with [`crate::panel::TextPurpose::Editing`]
    Waiting,
    Ready(Box<EditedSide>),
}

/// The edited new side of the diff's file.
pub(crate) struct EditedSide {
    pub(crate) model: EditorModel,
    /// the host's diff of the base: the file as last read or saved
    base_diff: FileDiff,
    /// the edits since the base, as lines
    edits: Vec<LineEdit>,
    /// the lines of the text that rows show, ascending
    shown: Vec<usize>,
}

/// The edited text's selection and caret on one row, in display columns.
pub(crate) struct EditMarks {
    pub(crate) selected: Option<Range<usize>>,
    pub(crate) caret: Option<usize>,
}

impl EditedSide {
    /// Edits `base`, the new side `base_diff` is the host's diff of, with the caret on its first added line.
    fn new(base: Arc<dyn LineSource>, base_diff: FileDiff) -> Self {
        let table = PieceTable::new(base);
        let mut side = Self { model: EditorModel::new(table, None), shown: shown_lines(&base_diff), base_diff, edits: Vec::new() };
        let first = first_added(&side.base_diff).or_else(|| side.shown.first().copied()).unwrap_or(0);
        side.model.set_caret(Position::new(first, 0), Selecting::Move);
        side
    }

    /// The diff as the text stands: the base's with the edits since; the rows show its lines from now on.
    fn refresh(&mut self) -> FileDiff {
        self.edits = line_edits(&**self.model.table().base(), &self.model.edits_since_base());
        let diff = with_hunks(&self.base_diff, |hunks| patched(hunks, &self.edits, &|line| self.line_text(line)));
        self.shown = shown_lines(&diff);
        diff
    }

    /// `saved` was written and `base` holds the text they made: it becomes the base, and the diff of the base takes
    /// the saved edits until the host diffs it again.
    fn rebase(&mut self, base: Arc<dyn LineSource>, saved: &[Edit]) -> Result<(), WrongBase> {
        let saved_lines = line_edits(&**self.model.table().base(), saved);
        self.model.rebase(base.clone(), saved)?;
        let text = |line: usize| base.lines(line..line + 1).pop().map_or_else(String::new, |shown| shown.text.to_string());
        self.base_diff = with_hunks(&self.base_diff, |hunks| patched(hunks, &saved_lines, &text));
        Ok(())
    }

    /// Line `line` of the text as a row shows it.
    fn line_text(&self, line: usize) -> String {
        self.display_line(line).text.to_string()
    }

    fn display_line(&self, line: usize) -> DisplayLine {
        let (start, stored) = self.model.stored_line(line);
        DisplayLine::of(&stored, start as usize)
    }

    /// The display column of `position` (tabs expanded, as rows show the line).
    pub(crate) fn display_column(&self, position: Position) -> usize {
        let (start, _) = self.model.stored_line(position.line);
        self.display_line(position.line).column_of(start as usize + self.model.byte_in_line(position))
    }

    /// The text position at display column `column` of line `line`.
    fn position_at(&self, line: usize, column: usize) -> Position {
        let (start, _) = self.model.stored_line(line);
        let byte = self.display_line(line).offset_at(column) - start as usize;
        Position::new(line, self.model.column_at_byte(line, byte))
    }

    /// The selection and the caret on line `line`, in display columns.
    pub(crate) fn marks(&self, line: usize) -> EditMarks {
        let caret = self.model.caret();
        EditMarks { selected: self.selected_columns(line), caret: (caret.line == line).then(|| self.display_column(caret)) }
    }

    /// The selected columns of line `line`, one past its end when the selection goes on past the line break.
    fn selected_columns(&self, line: usize) -> Option<Range<usize>> {
        let selection = self.model.selection().filter(|selection| (selection.start.line..=selection.end.line).contains(&line))?;
        let start = if line == selection.start.line { self.display_column(selection.start) } else { 0 };
        let end = match line == selection.end.line {
            true => self.display_column(selection.end),
            false => self.display_column(Position::new(line, usize::MAX)) + 1,
        };
        (start < end).then_some(start..end)
    }

    /// The base line that line `line` of the text still is, which the new side's colours are read by.
    pub(crate) fn base_line(&self, line: usize) -> Option<usize> {
        base_line(&self.edits, line)
    }

    fn is_shown(&self, line: usize) -> bool {
        self.shown.binary_search(&line).is_ok()
    }

    /// The `count`th shown line before `line`, or the first one.
    fn shown_before(&self, line: usize, count: usize) -> Option<usize> {
        let at = self.shown.partition_point(|shown| *shown < line);
        at.checked_sub(1).map(|last| self.shown[last.saturating_sub(count - 1)])
    }

    /// The `count`th shown line after `line`, or the last one.
    fn shown_after(&self, line: usize, count: usize) -> Option<usize> {
        let at = self.shown.partition_point(|shown| *shown <= line);
        (at < self.shown.len()).then(|| self.shown[(at + count - 1).min(self.shown.len() - 1)])
    }

    /// The shown line nearest to `line`; the one before it on a tie.
    fn nearest_shown(&self, line: usize) -> Option<usize> {
        match (self.shown_before(line + 1, 1), self.shown_after(line, 1)) {
            (Some(before), Some(after)) if after - line < line - before => Some(after),
            (Some(before), _) => Some(before),
            (None, after) => after,
        }
    }

    /// Moves the caret as `motion` says, over the lines rows show only: Up and Down go to the row above or below.
    fn move_caret(&mut self, motion: Motion, selecting: Selecting) {
        let line = self.model.caret().line;
        let vertical = match motion {
            Motion::Up => self.shown_before(line, 1),
            Motion::Down => self.shown_after(line, 1),
            Motion::PageUp(rows) => self.shown_before(line, rows),
            Motion::PageDown(rows) => self.shown_after(line, rows),
            Motion::DocumentStart => return self.jump(self.shown.first().map(|first| Position::new(*first, 0)), selecting),
            Motion::DocumentEnd => return self.jump(self.shown.last().map(|last| Position::new(*last, usize::MAX)), selecting),
            Motion::Left | Motion::Right | Motion::WordLeft | Motion::WordRight | Motion::LineStart | Motion::LineEnd => {
                self.model.move_caret(motion, selecting);
                return self.settle(motion, selecting);
            }
        };
        if let Some(target) = vertical {
            self.model.move_to_line(target, selecting);
        }
    }

    fn jump(&mut self, target: Option<Position>, selecting: Selecting) {
        if let Some(target) = target {
            self.model.set_caret(target, selecting);
        }
    }

    /// A move across a line break onto a line no row shows goes on to the next shown line that way.
    fn settle(&mut self, motion: Motion, selecting: Selecting) {
        let line = self.model.caret().line;
        if self.is_shown(line) {
            return;
        }
        let target = match motion {
            Motion::Left | Motion::WordLeft => self.shown_before(line, 1).map(|before| Position::new(before, usize::MAX)),
            _ => self.shown_after(line, 1).map(|after| Position::new(after, 0)),
        };
        self.jump(target, selecting);
    }

    /// After a new diff: a caret on a line no row shows any more (its hunk went) goes to the nearest one shown.
    fn keep_caret_shown(&mut self) {
        let caret = self.model.caret();
        if self.is_shown(caret.line) {
            return;
        }
        if let Some(line) = self.nearest_shown(caret.line) {
            self.model.set_caret(Position::new(line, caret.column), Selecting::Move);
        }
    }
}

/// The lines of the new side that rows of `diff` show, ascending.
fn shown_lines(diff: &FileDiff) -> Vec<usize> {
    let DiffBody::Text { hunks, .. } = &diff.body else {
        return Vec::new();
    };
    hunks.iter().flat_map(|hunk| &hunk.lines).filter_map(edited_line).collect()
}

/// The line of the new side `line` shows, from 0: none for a removed line or a note.
fn edited_line(line: &DiffLine) -> Option<usize> {
    match line.kind {
        LineKind::Added | LineKind::Context => line.new.map(|new| new as usize - 1),
        LineKind::Removed | LineKind::NoNewline => None,
    }
}

fn first_added(diff: &FileDiff) -> Option<usize> {
    let DiffBody::Text { hunks, .. } = &diff.body else {
        return None;
    };
    hunks.iter().flat_map(|hunk| &hunk.lines).find(|line| line.kind == LineKind::Added).and_then(edited_line)
}

/// `diff` with its hunks replaced by what `change` makes of them; a diff without text stays as it is.
fn with_hunks(diff: &FileDiff, change: impl FnOnce(&[Hunk]) -> Vec<Hunk>) -> FileDiff {
    let DiffBody::Text { hunks, cut } = &diff.body else {
        return diff.clone();
    };
    FileDiff { body: DiffBody::Text { hunks: change(hunks), cut: cut.clone() }, ..diff.clone() }
}

impl DiffView {
    /// [`Editable::NewSide`] waits for the new side's text, asking for it again; [`Editable::No`] drops the edited
    /// text.
    pub fn set_editable(&mut self, editable: Editable, cx: &mut Context<Self>) {
        self.editing = match editable {
            Editable::No => Editing::Off,
            Editable::NewSide { .. } => Editing::Waiting,
        };
        self.texts.forget(0);
        cx.notify();
    }

    /// The edited text; `None` while the diff does not edit, or waits for the text.
    pub fn edited(&self) -> Option<&EditorModel> {
        match &self.editing {
            Editing::Ready(side) => Some(&side.model),
            Editing::Off | Editing::Waiting => None,
        }
    }

    /// See [`PieceTable::rebase`]; the rows keep showing the text as it is.
    pub fn rebase(&mut self, base: Arc<dyn LineSource>, saved: &[Edit], cx: &mut Context<Self>) -> Result<(), WrongBase> {
        if let Editing::Ready(side) = &mut self.editing {
            side.rebase(base, saved)?;
        }
        self.show_edits(cx);
        Ok(())
    }

    /// The new side to edit, answering ask `generation` (an answer to another ask is dropped): its text becomes the
    /// edited one and its diff the rows; [`EditBase::Unavailable`] leaves the diff read-only. The parses that colour
    /// the two sides.
    pub fn set_edit_base(&mut self, generation: u64, base: EditBase, cx: &mut Context<Self>) -> Vec<ParseJob> {
        if self.waits_to_edit(generation) {
            return self.adopt(generation, base, cx);
        }
        Vec::new()
    }

    fn adopt(&mut self, generation: u64, base: EditBase, cx: &mut Context<Self>) -> Vec<ParseJob> {
        let EditBase::Text { old, new, diff } = base else {
            self.editing = Editing::Off;
            // asked for again, to colour the read-only diff
            self.texts.forget(0);
            cx.notify();
            return Vec::new();
        };
        let new_text = new.source();
        self.editing = Editing::Ready(Box::new(EditedSide::new(new, diff.clone())));
        self.show(vec![diff], cx);
        let sides = [(ParseTarget::DiffOld(0), old), (ParseTarget::DiffNew(0), new_text)];
        let file = &self.files[0];
        sides.into_iter().filter_map(|(target, text)| self.texts.set(target, generation, text?, file)).collect()
    }

    /// `diff`, the host's diff of `base`, becomes the edited text's while its edits apply to `base`: the rows show it
    /// patched with the edits since, and the caret stays on its line (or the nearest one shown) and on screen.
    pub fn set_base_diff(&mut self, base: &Arc<dyn LineSource>, diff: FileDiff, cx: &mut Context<Self>) {
        let caret_row = self.caret_row();
        let side = match &mut self.editing {
            Editing::Ready(side) if Arc::ptr_eq(side.model.table().base(), base) => side,
            Editing::Ready(_) | Editing::Off | Editing::Waiting => return,
        };
        side.base_diff = diff;
        let rows = side.refresh();
        // the base's text may have moved: its colours are asked for again, the old ones kept until they come
        self.texts.ask_again(0);
        self.show(vec![rows], cx);
        self.keep_caret(caret_row, cx);
    }

    /// Shows `files` as they are, keeping the scroll.
    fn show(&mut self, files: Vec<FileDiff>, cx: &mut Context<Self>) {
        self.files = files;
        self.colours = HighlightCache::default();
        self.measure_numbers();
        self.rebuild();
        cx.notify();
    }

    /// Whether the diff edits its new side, or waits for its text to.
    pub fn is_editable(&self) -> bool {
        match self.editing {
            Editing::Off => false,
            Editing::Waiting | Editing::Ready(_) => true,
        }
    }

    /// Whether the diff waits for the new side's text to edit it.
    pub(crate) fn waits_for_text(&self) -> bool {
        matches!(self.editing, Editing::Waiting)
    }

    /// Whether the new side's text answering ask `generation` becomes the edited text.
    pub fn waits_to_edit(&self, generation: u64) -> bool {
        self.waits_for_text() && self.texts.awaits(0, generation)
    }

    /// Whether the rows show the edited text, so only diffs of its base reach them.
    pub(crate) fn edits_rows(&self) -> bool {
        matches!(self.editing, Editing::Ready(_))
    }

    /// After a new diff: the caret goes to the nearest shown line if its own went, and its row stays where it was on
    /// screen (`before` is the row it was on).
    fn keep_caret(&mut self, before: Option<usize>, cx: &mut Context<Self>) {
        if let Editing::Ready(side) = &mut self.editing {
            side.keep_caret_shown();
        }
        let (Some(before), Some(after)) = (before, self.caret_row()) else {
            return;
        };
        let position = self.scroll_position();
        let top_line = position.top_line + after as f64 - before as f64;
        self.restore_scroll(crate::position::ScrollPosition { top_line, ..position }, cx);
    }

    /// The rows after an edit: the file's diff with the edits since its base.
    fn show_edits(&mut self, cx: &mut Context<Self>) {
        let Editing::Ready(side) = &mut self.editing else {
            return;
        };
        let diff = side.refresh();
        self.files = vec![diff];
        self.measure_numbers();
        self.rebuild();
        cx.notify();
    }

    /// The row showing the caret's line.
    pub(crate) fn caret_row(&self) -> Option<usize> {
        let line = self.edited()?.caret().line;
        (0..self.rows.len()).find(|row| self.edited_line_at(*row, Side::New) == Some(line))
    }

    /// The line of the edited text row `row` shows (side by side, in panel `side`), from 0.
    pub(crate) fn edited_line_at(&self, row: usize, side: Side) -> Option<usize> {
        let line = match (self.rows.get(row)?, self.shape.layout, side) {
            (DiffRow::Line { file: 0, hunk, line }, DiffLayout::Inline, _) => &self.hunk(0, *hunk).lines[*line],
            (DiffRow::Pair { file: 0, row }, DiffLayout::SideBySide, Side::New) => self.line(0, row.new?),
            _ => return None,
        };
        edited_line(line)
    }

    /// The edited text's marks on `line` of the diff; none while not editing, or on a removed line.
    pub(crate) fn edit_marks(&self, line: &DiffLine) -> EditMarks {
        let (Editing::Ready(side), Some(line)) = (&self.editing, edited_line(line)) else {
            return EditMarks { selected: None, caret: None };
        };
        let marks = side.marks(line);
        EditMarks { caret: marks.caret.filter(|_| self.focused), ..marks }
    }

    /// `line` as the new side's colours are read: by the base line it still is; `None` for a line an edit made.
    pub(crate) fn coloured_line(&self, line: &DiffLine) -> Option<DiffLine> {
        let Editing::Ready(side) = &self.editing else {
            return Some(line.clone());
        };
        match line.kind {
            LineKind::Removed | LineKind::NoNewline => Some(line.clone()),
            LineKind::Added | LineKind::Context => {
                let base = side.base_line(edited_line(line)?)?;
                Some(DiffLine { new: Some(base as u32 + 1), ..line.clone() })
            }
        }
    }

    /// Runs an editing command; nothing while not editing.
    pub(crate) fn run_editing(&mut self, command: Command, cx: &mut Context<Self>) {
        match command {
            Command::Move(motion, selecting) => self.move_caret(motion, selecting, cx),
            Command::Cut => self.cut(cx),
            Command::Paste(source) => self.paste(source, cx),
            Command::Newline => self.edit(|model| model.insert("\n", EditKind::Other), cx),
            Command::Indent => self.edit(|model| model.insert(INDENT, EditKind::Other), cx),
            Command::Backspace(unit) => self.edit(|model| model.backspace(unit), cx),
            Command::Delete(unit) => self.edit(|model| model.delete_forward(unit), cx),
            Command::Undo => self.edit(
                |model| {
                    model.undo();
                    Ok(())
                },
                cx,
            ),
            Command::Redo => self.edit(
                |model| {
                    model.redo();
                    Ok(())
                },
                cx,
            ),
            Command::Copy(_) | Command::SelectAll | Command::ToggleLayout | Command::Flush => {}
        }
    }

    /// Types `text` at the caret as the keyboard would; `\r` is a line break.
    pub fn type_text(&mut self, text: &str, cx: &mut Context<Self>) {
        self.edit(|model| model.insert(text, EditKind::Typing), cx);
    }

    /// Copies the edited text's selection; false when there is none.
    pub(crate) fn copy_edited(&self, target: ClipboardTarget, cx: &mut App) -> bool {
        let Some(text) = self.edited().and_then(EditorModel::selected_text) else {
            return false;
        };
        Host::write_clipboard(target, text, cx);
        true
    }

    fn cut(&mut self, cx: &mut Context<Self>) {
        if self.copy_edited(ClipboardTarget::Clipboard, cx) {
            self.edit(EditorModel::delete_selection, cx);
        }
    }

    fn paste(&mut self, source: ClipboardTarget, cx: &mut Context<Self>) {
        if let Some(text) = Host::read_clipboard(source, cx) {
            self.edit(|model| model.insert(&text, EditKind::Other), cx);
        }
    }

    /// Every change of the edited text goes through here: the rows follow it, and the panel hears of it.
    fn edit(&mut self, change: impl FnOnce(&mut EditorModel) -> Result<(), EditError>, cx: &mut Context<Self>) {
        let Editing::Ready(side) = &mut self.editing else {
            return;
        };
        let before = side.model.revision();
        // files have no size limit, so nothing is refused
        let _ = change(&mut side.model);
        // the diff's colours come from the host's texts, not from re-parsing the edited one
        side.model.take_replacements();
        if side.model.revision() != before {
            self.show_edits(cx);
            cx.emit(DiffEvent::Edited);
        }
        self.selection = None;
        self.reveal_caret = true;
        cx.notify();
    }

    fn move_caret(&mut self, motion: Motion, selecting: Selecting, cx: &mut Context<Self>) {
        if let Editing::Ready(side) = &mut self.editing {
            side.move_caret(motion, selecting);
            self.selection = None;
            self.reveal_caret = true;
            cx.notify();
        }
    }

    /// Keys while editing: the host's bindings as in an editor, then the editing and moving keys.
    pub(crate) fn on_editing_key(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) {
        let command = match Host::key_outcome(event, KeyScope::Editor, cx) {
            KeyOutcome::Command(command) => command,
            KeyOutcome::Handled => return cx.stop_propagation(),
            // a window action is the workspace's to run
            KeyOutcome::Foreign => return,
            // typed characters arrive through the input handler; other keys belong to the host
            KeyOutcome::Unbound => match keys::command(&event.keystroke, self.page_rows()) {
                Some(command) => command,
                None => return,
            },
        };
        cx.stop_propagation();
        self.run(command, cx);
    }

    fn page_rows(&self) -> usize {
        let viewport = self.list.extent(Axis::Vertical).viewport / f64::from(self.sizes.row_height);
        (viewport as usize).saturating_sub(1).max(1)
    }

    /// A press on a line of the edited text places the caret there (two clicks select a word, three the line);
    /// false when the press is elsewhere.
    pub(crate) fn press_text(&mut self, event: &MouseDownEvent) -> bool {
        let Some((line, column)) = self.edited_point(event.position) else {
            return false;
        };
        let Editing::Ready(side) = &mut self.editing else {
            return false;
        };
        let at = side.position_at(line, column);
        match event.click_count {
            2 => side.model.select_word(at),
            3.. => side.model.select_line(at.line),
            _ if event.modifiers.shift => side.model.set_caret(at, Selecting::Extend),
            _ => side.model.set_caret(at, Selecting::Move),
        }
        true
    }

    /// A drag that started on the edited text extends its selection over the edited lines it passes.
    pub(crate) fn drag_text(&mut self, event: &MouseMoveEvent) {
        let Some((line, column)) = self.edited_point(event.position) else {
            return;
        };
        if let Editing::Ready(side) = &mut self.editing {
            let at = side.position_at(line, column);
            side.model.set_caret(at, Selecting::Extend);
        }
    }

    /// The edited line and display column under window point `point`.
    fn edited_point(&self, point: Point<Pixels>) -> Option<(usize, usize)> {
        let position = self.position_at(point)?;
        Some((self.edited_line_at(position.row, self.side)?, position.column))
    }

    /// Drops the edited text's selection: a press elsewhere starts a selection of rows.
    pub(crate) fn deselect_text(&mut self) {
        if let Editing::Ready(side) = &mut self.editing {
            side.model.deselect();
        }
    }

    /// The focus the input method types into while editing.
    pub(crate) fn input_focus(&self) -> Option<FocusHandle> {
        self.edited().map(|_| self.focus_handle.clone())
    }

    /// Scrolls the caret's row and column into view, after it moved.
    pub(crate) fn reveal(&mut self) {
        let (true, Some(row), Some(model)) = (std::mem::take(&mut self.reveal_caret), self.caret_row(), self.edited()) else {
            return;
        };
        self.list.list_handle().scroll_to_item(row, ScrollStrategy::Top);
        let column = match &self.editing {
            Editing::Ready(side) => side.display_column(model.caret()),
            Editing::Off | Editing::Waiting => return,
        };
        let char_width = f64::from(f32::from(self.char_width));
        let (sideways, gutter) = self.edit_scroll();
        let x = f64::from(f32::from(gutter)) + char_width * column as f64;
        let extent = sideways.extent(Axis::Horizontal);
        let margin = char_width * CARET_MARGIN_CHARS;
        if x - margin < extent.scrolled {
            sideways.scroll_to(Axis::Horizontal, x - margin);
        } else if x + margin > extent.scrolled + extent.viewport {
            sideways.scroll_to(Axis::Horizontal, x + margin - extent.viewport);
        }
    }

    /// What scrolls the edited text sideways, and how far its text starts from that scroll's left edge.
    fn edit_scroll(&self) -> (&dyn Scrollable, Pixels) {
        match self.shape.layout {
            DiffLayout::Inline => (&self.list, self.text_start()),
            DiffLayout::SideBySide => (&self.panels.new.offset, px(0.)),
        }
    }

    /// Where the caret is drawn, in window pixels, for the input method's candidate window.
    fn caret_bounds(&self) -> Option<Bounds<Pixels>> {
        let (bounds, row, model) = (self.bounds.get()?, self.caret_row()?, self.edited()?);
        let Editing::Ready(side) = &self.editing else {
            return None;
        };
        let top = bounds.top() + px(self.sizes.row_height * row as f32 - self.list.extent(Axis::Vertical).scrolled as f32);
        let panel_left = match self.shape.layout {
            DiffLayout::Inline => bounds.left(),
            DiffLayout::SideBySide => self.split.span(Side::New, bounds).start,
        };
        let (sideways, _) = self.edit_scroll();
        let left = panel_left + self.text_start() - px(sideways.extent(Axis::Horizontal).scrolled as f32);
        let x = left + self.char_width * side.display_column(model.caret()) as f32;
        Some(Bounds::new(point(x, top), size(self.char_width, px(self.sizes.row_height))))
    }

    fn model_mut(&mut self) -> Option<&mut EditorModel> {
        match &mut self.editing {
            Editing::Ready(side) => Some(&mut side.model),
            Editing::Off | Editing::Waiting => None,
        }
    }
}

impl EntityInputHandler for DiffView {
    fn text_for_range(&mut self, range: Range<usize>, adjusted: &mut Option<Range<usize>>, _window: &mut Window, _cx: &mut Context<Self>) -> Option<String> {
        let model = self.edited()?;
        let length = model.utf16_len();
        let range = range.start.min(length)..range.end.min(length);
        adjusted.replace(range.clone());
        Some(model.text_utf16(range))
    }

    fn selected_text_range(&mut self, _ignore_disabled_input: bool, _window: &mut Window, _cx: &mut Context<Self>) -> Option<UTF16Selection> {
        let (range, reversed) = self.edited()?.selection_utf16();
        Some(UTF16Selection { range, reversed })
    }

    fn marked_text_range(&self, _window: &mut Window, _cx: &mut Context<Self>) -> Option<Range<usize>> {
        self.edited()?.marked_utf16()
    }

    fn unmark_text(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {
        if let Some(model) = self.model_mut() {
            model.unmark();
        }
    }

    fn replace_text_in_range(&mut self, range: Option<Range<usize>>, text: &str, _window: &mut Window, cx: &mut Context<Self>) {
        self.edit(|model| model.replace_utf16(range, text), cx);
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        selected: Option<Range<usize>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.edit(|model| model.mark_utf16(range, text, selected), cx);
    }

    fn bounds_for_range(&mut self, _range: Range<usize>, _element_bounds: Bounds<Pixels>, _window: &mut Window, _cx: &mut Context<Self>) -> Option<Bounds<Pixels>> {
        self.caret_bounds()
    }

    fn character_index_for_point(&mut self, point: Point<Pixels>, _window: &mut Window, _cx: &mut Context<Self>) -> Option<usize> {
        let (line, column) = self.edited_point(point)?;
        let Editing::Ready(side) = &self.editing else {
            return None;
        };
        Some(side.model.offset_utf16(side.position_at(line, column)))
    }
}
