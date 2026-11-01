//! The text editor entity: draws the visible lines on a canvas, takes keys, mouse and input-method text.

use std::cell::RefCell;
use std::ops::Range;
use std::rc::Rc;
use std::sync::Arc;

use gpui::{
    AnyElement, App, Bounds, ClickEvent, ContentMask, Context, CursorStyle, DispatchPhase, ElementInputHandler, Entity,
    EntityInputHandler, EventEmitter, FocusHandle, Focusable, FontStyle, FontWeight, KeyDownEvent, MouseButton,
    MouseDownEvent, MouseMoveEvent, MouseUpEvent, PathBuilder, Pixels, Point, ScrollWheelEvent, Subscription, TextRun, UTF16Selection,
    WeakEntity, Window, canvas,
    div, fill, point, prelude::*, px, rgb, rgba, size,
};

use super::change_popup::{Anchor, ChangeAction, ChangePopup, CommitShown, Committing, Popup, PopupActions, SentChange, popup_element};
use crate::line_field::{FieldState, LineField, LineFieldEvent};
use super::colours::Colours;
use super::keys::command;
use super::layout::{Area, Geometry, Placement, RowSpan, VisibleLine, line_spans, mark_columns, visible_lines};
use super::model::{EditError, EditKind, EditorModel, INDENT, Motion, Position, Selecting, SizeLimit};
use super::wrap::{Row, RowCache, RowPlace, SoftWrap, Wrapped};
use crate::changes::{BaseChange, ChangeBase, ChangeDiffJob, ChangeHunk, ChangeHunks, ChangeMarkers, Marker, Toward};
use crate::colors::{ResolvedScheme, TextAttributes};
use crate::find::FindText;
use crate::find::marks::{FindMarks, Mark};
use crate::line_view::{self, LineScroll, Metrics};
use crate::panel::{Edit, LineSource, ParseJob};
use crate::piece_table::{PieceTable, WrongBase};
use crate::host::{ClickListener, ClipboardTarget, Command, Host, KeyOutcome, KeyScope};
use crate::position::{CaretPosition, ScrollPosition};
use crate::scrollbar::{Axis, ScrollState, Scrollable, Tick, horizontal_scrollbar, vertical_scrollbar_with_ticks, wheel_axes};
use crate::style::ViewStyle;
use crate::syntax::{Highlight, Language, Syntax};

/// Columns assumed visible before the first layout reports the viewport width.
const FIRST_FRAME_COLUMNS: usize = 400;
/// Columns kept between the caret and the viewport's edge when scrolling sideways to it.
const CARET_MARGIN_CHARS: f64 = 2.;
/// The width of a change marker's bar, and the height of a deleted marker's triangle, in pixels.
const CHANGE_BAR_WIDTH: f32 = 3.;
const DELETED_MARKER_HEIGHT: f32 = 8.;
const COMMIT_PLACEHOLDER: &str = "Commit this change";
/// Columns left free right of wrapped rows: for the caret after a full row, a space hanging past it, the scrollbar.
const WRAP_MARGIN_CHARS: usize = 2;

pub enum TextEditorEvent {
    /// the text changed
    Edited,
    /// an edit was refused; the reason is for the user
    Refused(String),
    /// the user asked for the file's diff at line `line` (from 1), from a change's popup
    OpenDiff { line: usize },
    /// a change's popup waits for the markers of the current text: diff it now, without the pause
    NeedChanges,
    /// the user asked to commit the change shown, alone: see [`crate::panel::Event::CommitChange`]
    CommitChange { head_id: String, block: Range<usize>, text: Vec<u8>, message: String },
    /// a right click on the line numbers: the host's menu for them opens at the pointer
    GutterMenu,
}

/// What an editor is opened with: a scratch's size limit (files have none), and the language colouring the text.
pub struct Setup {
    pub limit: Option<SizeLimit>,
    pub language: Option<Language>,
}

pub struct TextEditor {
    model: EditorModel,
    colours: Colours,
    markers: ChangeMarkers,
    popup: Popup,
    /// the popup's commit message, kept while it is closed; it takes no typing while its commit is in flight
    message: Entity<LineField>,
    committing: Committing,
    /// closes the popup when the keyboard leaves the editor (and what it holds), from the first popup on
    focus_out: FocusWatch,
    focus_handle: FocusHandle,
    scroll: Rc<LineScroll>,
    scrollbars: ScrollState,
    /// where the last frame painted the text, for the mouse and the input method
    geometry: Rc<RefCell<Option<Geometry>>>,
    soft_wrap: SoftWrap,
    /// the columns of the last frame's wrapped rows; `None` while lines are not wrapped
    wrap_width: Option<usize>,
    /// the rows of wrapped lines, kept while the text and the width stay
    rows: RowCache,
    /// a left-button drag that started in the editor goes on
    dragging: bool,
    reveal_caret: bool,
    _message_submits: Subscription,
    /// the find box's matches, while it is open
    find_marks: Option<FindMarks>,
    /// the text the last search copied, and its revision: typing in the box searches it again without a copy
    find_snapshot: std::cell::RefCell<Option<(u64, Arc<Vec<u8>>)>>,
}

enum FocusWatch {
    Unwatched,
    Watched { _subscription: Subscription },
}

impl EventEmitter<TextEditorEvent> for TextEditor {}

impl TextEditor {
    /// Edits `table`, plain until the first [`TextEditor::parse_job`] lands.
    pub fn new(table: PieceTable, setup: Setup, cx: &mut Context<Self>) -> Self {
        let model = EditorModel::new(table, setup.limit);
        let scroll = Rc::new(LineScroll::new(model.line_count()));
        let message = cx.new(|cx| LineField::new(COMMIT_PLACEHOLDER, "change-popup-message", cx));
        let submits = cx.subscribe(&message, |editor, _field, event: &LineFieldEvent, cx| {
            if let LineFieldEvent::Submitted(_) = event {
                editor.commit_change(cx);
            }
        });
        Self {
            message,
            _message_submits: submits,
            scrollbars: ScrollState::new(scroll.clone()),
            scroll,
            model,
            colours: Colours::new(setup.language),
            markers: ChangeMarkers::new(),
            popup: Popup::Closed,
            committing: Committing::Idle,
            focus_out: FocusWatch::Unwatched,
            focus_handle: cx.focus_handle(),
            geometry: Rc::default(),
            soft_wrap: SoftWrap::Off,
            wrap_width: None,
            rows: RowCache::default(),
            dragging: false,
            reveal_caret: false,
            find_marks: None,
            find_snapshot: std::cell::RefCell::default(),
        }
    }

    pub fn soft_wrap(&self) -> SoftWrap {
        self.soft_wrap
    }

    /// Breaks lines longer than the viewport into rows, or stops; the caret stays in view.
    pub fn set_soft_wrap(&mut self, soft_wrap: SoftWrap, cx: &mut Context<Self>) {
        self.soft_wrap = soft_wrap;
        self.reveal_caret = true;
        cx.notify();
    }

    /// The text as of its revision, copied out for a search off the UI thread.
    pub fn find_text(&self) -> (u64, FindText) {
        let revision = self.model.revision();
        let mut snapshot = self.find_snapshot.borrow_mut();
        if let Some((copied, bytes)) = snapshot.as_ref()
            && *copied == revision
        {
            return (revision, FindText::Owned(bytes.clone()));
        }
        let table = self.model.table();
        let bytes = Arc::new(table.bytes(0..table.byte_len()));
        *snapshot = Some((revision, bytes.clone()));
        (revision, FindText::Owned(bytes))
    }

    /// The selection when on one line, else the word at the caret, and the byte it starts at.
    pub fn find_prefill(&self) -> Option<(String, usize)> {
        let picked = match self.model.selection() {
            Some(selection) if selection.start.line == selection.end.line => selection,
            Some(_) => return None,
            None => self.model.word_at(self.model.caret())?,
        };
        Some((self.model.text_in(picked.start, picked.end), self.model.byte_of(picked.start)))
    }

    /// The selection's bytes, else the caret's.
    pub fn find_anchor(&self) -> Range<usize> {
        let caret = self.model.caret();
        let selection = self.model.selection().unwrap_or(caret..caret);
        self.model.byte_of(selection.start)..self.model.byte_of(selection.end)
    }

    /// Selects bytes `bytes`, the caret at their end, and scrolls them into view.
    pub fn show_match(&mut self, bytes: Range<usize>, cx: &mut Context<Self>) {
        self.model.set_caret(self.model.position_of(bytes.start), Selecting::Move);
        self.model.set_caret(self.model.position_of(bytes.end), Selecting::Extend);
        self.reveal_caret = true;
        cx.notify();
    }

    pub fn set_find_marks(&mut self, marks: Option<FindMarks>, cx: &mut Context<Self>) {
        self.find_marks = marks;
        cx.notify();
    }

    #[cfg(test)]
    pub(crate) fn has_find_marks(&self) -> bool {
        self.find_marks.is_some()
    }

    pub fn selected_text(&self) -> Option<String> {
        self.model.selected_text()
    }

    fn find_ticks(&self) -> Rc<[Tick]> {
        let lines = self.model.line_count() as f32;
        self.find_marks.as_ref().map_or_else(|| Rc::from([]), |marks| marks.ticks(|line| line as f32 / lines))
    }

    /// The changes since the base: the text as opened or last saved.
    pub fn edits_since_base(&self) -> Vec<Edit> {
        self.model.edits_since_base()
    }

    /// `saved` was written: `base` holds the text they made, and becomes the base; see [`PieceTable::rebase`].
    pub fn rebase(&mut self, base: Arc<dyn LineSource>, saved: &[Edit]) -> Result<(), WrongBase> {
        self.model.rebase(base, saved)
    }

    pub fn revision(&self) -> u64 {
        self.model.revision()
    }

    pub fn caret_position(&self) -> CaretPosition {
        let caret = self.model.caret();
        CaretPosition { line: caret.line, column: caret.column }
    }

    #[cfg(test)]
    pub(crate) fn text(&self) -> String {
        let table = self.model.table();
        String::from_utf8_lossy(&table.bytes(0..table.byte_len())).into_owned()
    }

    pub fn scroll_position(&self) -> ScrollPosition {
        self.scroll.position()
    }

    /// Puts the caret and the scroll back where a saved tab had them; the caret is kept inside the text and the
    /// scroll is not moved to show it.
    pub fn restore_view(&mut self, caret: CaretPosition, scroll: ScrollPosition, cx: &mut Context<Self>) {
        self.model.set_caret(Position::new(caret.line, caret.column), Selecting::Move);
        self.scroll.set_position(scroll);
        self.reveal_caret = false;
        cx.notify();
    }

    /// Scrolls line `line` (from 0) to the top; the caret stays when it is among the lines then shown, else it goes to
    /// the start of that line, nothing selected. Not an edit: the undo history does not hear of it.
    pub fn land_at(&mut self, line: usize, cx: &mut Context<Self>) {
        let scroll = ScrollPosition { top_line: line as f64, ..self.scroll_position() };
        let shown = line..line + self.scroll.viewport_lines().ceil().max(1.) as usize;
        let caret = self.caret_position();
        if shown.contains(&caret.line) {
            self.scroll.set_position(scroll);
            return cx.notify();
        }
        self.restore_view(CaretPosition { line, column: 0 }, scroll, cx);
    }

    /// Puts the caret at the start of line `line` (from 1, clamped) and scrolls it into view.
    pub fn reveal_line(&mut self, line: usize, cx: &mut Context<Self>) {
        self.model.set_caret(Position::new(line.saturating_sub(1), 0), Selecting::Move);
        self.reveal_caret = true;
        cx.notify();
    }

    /// Selects `bytes` of line `line` (from 1, clamped) and scrolls them into view.
    pub fn reveal_range(&mut self, line: usize, bytes: Range<usize>, cx: &mut Context<Self>) {
        self.model.select_in_line(line.saturating_sub(1), bytes);
        self.reveal_caret = true;
        cx.notify();
    }

    /// Replaces the text after a reload: the history, the colours and the change markers are dropped (the markers'
    /// base stays), the caret stays where it still fits.
    pub fn reset(&mut self, table: PieceTable, setup: Setup, cx: &mut Context<Self>) {
        let caret = self.model.caret();
        self.model = EditorModel::new(table, setup.limit);
        // the new text counts its revisions from the start again: a copy kept for find, and rows, are of the old one
        *self.find_snapshot.borrow_mut() = None;
        self.rows.clear();
        self.model.set_caret(caret, Selecting::Move);
        self.colours = Colours::new(setup.language);
        self.markers.reset();
        self.popup = Popup::Closed;
        cx.notify();
    }

    /// What the change markers compare the text with from now on; see [`ChangeMarkers::set_base`].
    pub fn set_change_base(&mut self, base: ChangeBase, cx: &mut Context<Self>) -> BaseChange {
        cx.notify();
        let change = self.markers.set_base(base);
        self.follow_changes();
        change
    }

    /// The diff of the current text with the markers' base; see [`ChangeMarkers::job`].
    pub fn change_job(&mut self) -> Option<ChangeDiffJob> {
        let job = self.markers.job(self.model.revision(), self.model.table());
        // a text grown too large to diff loses its markers, and the popup its change
        self.follow_changes();
        job
    }

    /// A diff of the text at `revision` landed; see [`ChangeMarkers::landed`]. The popup's action waiting for it runs
    /// once it is of the current text.
    pub fn change_diffed(&mut self, revision: u64, hunks: ChangeHunks, cx: &mut Context<Self>) {
        self.markers.landed(revision, hunks);
        self.follow_changes();
        cx.notify();
        if self.has_current_changes() {
            self.run_waiting(cx);
        }
    }

    fn run_waiting(&mut self, cx: &mut Context<Self>) {
        let Popup::Open(popup) = &mut self.popup else {
            return;
        };
        if let Some(action) = popup.take_waiting() {
            self.run_action(action, cx);
        }
    }

    /// Whether the popup's action waits for the markers of the current text.
    pub fn waits_for_changes(&self) -> bool {
        matches!(&self.popup, Popup::Open(popup) if popup.is_waiting())
    }

    /// The popup stays on its change, wherever the markers put it, and closes when it is gone.
    fn follow_changes(&mut self) {
        self.popup = std::mem::replace(&mut self.popup, Popup::Closed).followed(&self.markers);
    }

    /// Opens the popup of the change whose marker starts on line `line` (from 0) and takes the keyboard.
    pub fn open_change_popup(&mut self, line: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(hunk) = self.markers.starting_at(line) else {
            return;
        };
        self.popup = Popup::Open(ChangePopup::of(hunk, &self.markers));
        if let FocusWatch::Unwatched = self.focus_out {
            let watch = cx.on_focus_out(&self.focus_handle, window, |editor, _event, _window, cx| editor.close_change_popup(cx));
            self.focus_out = FocusWatch::Watched { _subscription: watch };
        }
        window.focus(&self.focus_handle, cx);
        cx.notify();
    }

    pub fn close_change_popup(&mut self, cx: &mut Context<Self>) {
        self.popup = Popup::Closed;
        cx.notify();
    }

    /// The change the popup shows, in the current text's lines.
    pub fn change_in_popup(&self) -> Option<&ChangeHunk> {
        match &self.popup {
            Popup::Open(popup) => Some(popup.block()),
            Popup::Closed => None,
        }
    }

    /// Moves the caret and the popup to the first line of the change before or after the one shown, going round
    /// from the last change to the first.
    pub fn step_change(&mut self, toward: Toward, cx: &mut Context<Self>) {
        let Some(hunk) = self.change_in_popup().and_then(|shown| self.markers.beside(shown.current.start, toward)) else {
            return;
        };
        let line = hunk.current.start;
        self.popup = Popup::Open(ChangePopup::of(hunk, &self.markers));
        self.model.set_caret(Position::new(line, 0), Selecting::Move);
        self.reveal_caret = true;
        cx.notify();
    }

    /// Replaces the lines of the change shown with the base's, as one edit (one undo step), and closes the popup;
    /// once the markers are the diff of the current text.
    pub fn roll_back_change(&mut self, cx: &mut Context<Self>) {
        self.act(ChangeAction::Rollback, cx);
    }

    /// Puts the previous text of the change shown on the clipboard (added lines have none), once the markers are the
    /// diff of the current text.
    pub fn copy_previous_text(&mut self, cx: &mut Context<Self>) {
        self.act(ChangeAction::Copy, cx);
    }

    /// Commits the change shown, alone, with the message typed, once the markers are the diff of the current text;
    /// the host answers with [`TextEditor::change_committed`] or [`TextEditor::change_commit_failed`]. Nothing while
    /// a commit is in flight.
    pub fn commit_change(&mut self, cx: &mut Context<Self>) {
        if self.committing.is_in_flight() {
            return;
        }
        self.act(ChangeAction::Commit, cx);
    }

    /// The change sent to be committed is: its popup closes if open, and the message goes.
    pub fn change_committed(&mut self, cx: &mut Context<Self>) {
        let Committing::Sent(sent) = &self.committing else {
            return;
        };
        if matches!(&self.popup, Popup::Open(popup) if sent.is_of(popup.block(), self.markers.head_id())) {
            self.close_change_popup(cx);
        }
        self.message.update(cx, |field, cx| field.clear(cx));
        self.set_committing(Committing::Idle, cx);
    }

    /// The change sent to be committed is not, for `reason`: its popup says why, now or when it next opens, until
    /// the next commit is sent; the message stays.
    pub fn change_commit_failed(&mut self, reason: String, cx: &mut Context<Self>) {
        let Committing::Sent(sent) = &self.committing else {
            return;
        };
        self.set_committing(Committing::Failed(sent.clone(), reason), cx);
    }

    /// The change sent to be committed is not, and the host shows why elsewhere: the popup stays as it is, with the
    /// message, ready to send again.
    pub fn change_not_committed(&mut self, cx: &mut Context<Self>) {
        if self.committing.is_in_flight() {
            self.set_committing(Committing::Idle, cx);
        }
    }

    /// The field takes typing unless the commit is in flight.
    fn set_committing(&mut self, committing: Committing, cx: &mut Context<Self>) {
        let state = if committing.is_in_flight() { FieldState::Busy } else { FieldState::Editable };
        self.committing = committing;
        self.message.update(cx, |field, cx| field.set_state(state, cx));
        cx.notify();
    }

    #[cfg(test)]
    pub(crate) fn commit_message(&self, cx: &App) -> String {
        self.message.read(cx).text().to_owned()
    }

    /// Why the last commit of the change shown failed.
    #[cfg(test)]
    pub(crate) fn commit_failure(&self) -> Option<String> {
        let Popup::Open(popup) = &self.popup else {
            return None;
        };
        match self.committing.shown(popup.block(), self.markers.head_id()) {
            CommitShown::Failed(reason) => Some(reason.to_string()),
            CommitShown::Ready | CommitShown::Sending => None,
        }
    }

    /// Runs `action` on the change shown now when the markers are the diff of the current text; else asks for that
    /// diff, and it runs on the change as the diff finds it when it lands.
    fn act(&mut self, action: ChangeAction, cx: &mut Context<Self>) {
        if self.has_current_changes() {
            return self.run_action(action, cx);
        }
        let Popup::Open(popup) = &mut self.popup else {
            return;
        };
        popup.wait_to(action);
        cx.emit(TextEditorEvent::NeedChanges);
        cx.notify();
    }

    fn run_action(&mut self, action: ChangeAction, cx: &mut Context<Self>) {
        match action {
            ChangeAction::Rollback => self.roll_back_now(cx),
            ChangeAction::Copy => self.copy_now(cx),
            ChangeAction::Commit => self.commit_now(cx),
        }
    }

    /// Sends the change shown to be committed, unless a commit is in flight: HEAD's text with only its lines replaced.
    fn commit_now(&mut self, cx: &mut Context<Self>) {
        let Popup::Open(popup) = &self.popup else {
            return;
        };
        if self.committing.is_in_flight() {
            return;
        }
        let block = popup.block().clone();
        let lines = self.model.table().bytes(self.bytes_of(block.current.clone()));
        let Some((head_id, text)) = self.markers.committed_text(&block, &lines) else {
            return;
        };
        let message = self.message.read(cx).text().to_owned();
        self.set_committing(Committing::Sent(SentChange { base: block.base.clone(), head_id: head_id.clone() }), cx);
        cx.emit(TextEditorEvent::CommitChange { head_id, block: block.base, text, message });
    }

    fn roll_back_now(&mut self, cx: &mut Context<Self>) {
        let Popup::Open(popup) = std::mem::replace(&mut self.popup, Popup::Closed) else {
            return;
        };
        let block = popup.block().clone();
        let text = self.markers.base_text(&block);
        // the marker goes at once instead of moving onto the restored lines until the next diff lands
        self.markers.drop_starting_at(block.current.start);
        // a refused edit was already reported through TextEditorEvent::Refused
        let _ = self.edit(|model| model.replace_lines(block.current.clone(), &text), cx);
    }

    fn copy_now(&mut self, cx: &mut Context<Self>) {
        let Popup::Open(popup) = &self.popup else {
            return;
        };
        if popup.previous().is_empty() {
            return;
        }
        Host::write_clipboard(ClipboardTarget::Clipboard, popup.previous().to_owned(), cx);
    }

    /// Asks for the file's diff at the change shown and closes the popup.
    pub fn show_change_diff(&mut self, cx: &mut Context<Self>) {
        let Popup::Open(popup) = std::mem::replace(&mut self.popup, Popup::Closed) else {
            return;
        };
        cx.emit(TextEditorEvent::OpenDiff { line: popup.block().current.start + 1 });
        cx.notify();
    }

    /// Whether the markers shown are the diff of the current text.
    pub fn has_current_changes(&self) -> bool {
        self.markers.is_current(self.model.revision())
    }

    /// The parse of the current text; see [`Colours::job`].
    pub fn parse_job(&mut self) -> Option<ParseJob> {
        self.colours.job(self.model.revision(), self.model.table())
    }

    /// A parse of the text at `revision` landed; see [`Colours::parsed`].
    pub fn parsed(&mut self, revision: u64, syntax: Syntax, cx: &mut Context<Self>) {
        self.colours.parsed(revision, syntax);
        cx.notify();
    }

    /// Types `text` at the caret as the keyboard would; `\r` is a line break.
    pub fn type_text(&mut self, text: &str, cx: &mut Context<Self>) -> Result<(), EditError> {
        self.edit(|model| model.insert(text, EditKind::Typing), cx)
    }

    fn edit(&mut self, edit: impl FnOnce(&mut EditorModel) -> Result<(), EditError>, cx: &mut Context<Self>) -> Result<(), EditError> {
        let before = self.model.revision();
        let result = edit(&mut self.model);
        let replacements = self.model.take_replacements();
        self.markers.edited(&replacements);
        self.popup = std::mem::replace(&mut self.popup, Popup::Closed).edited(&replacements, &self.markers);
        self.colours.edited(self.model.revision(), replacements);
        match &result {
            Err(error) => cx.emit(TextEditorEvent::Refused(error.to_string())),
            Ok(()) if self.model.revision() != before => cx.emit(TextEditorEvent::Edited),
            Ok(()) => {}
        }
        self.reveal_caret = true;
        cx.notify();
        result
    }

    fn navigate(&mut self, navigate: impl FnOnce(&mut EditorModel), cx: &mut Context<Self>) -> Result<(), EditError> {
        navigate(&mut self.model);
        self.reveal_caret = true;
        cx.notify();
        Ok(())
    }

    /// Runs `command` as its key would.
    pub fn run(&mut self, command: Command, cx: &mut Context<Self>) {
        // a refused edit was already reported through TextEditorEvent::Refused
        let _ = match command {
            Command::Move(motion, selecting) => match (self.wrap_width, rows_moved(motion)) {
                (Some(width), Some(rows)) => self.move_by_rows(rows, width, selecting, cx),
                _ => self.navigate(|model| model.move_caret(motion, selecting), cx),
            },
            Command::SelectAll => self.navigate(EditorModel::select_all, cx),
            Command::Copy(target) => self.copy(target, cx),
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
            // saving, finding and the Markdown view are the panel's, a text has no layout to toggle, and only a reading
            // view scrolls by a command
            Command::Flush
            | Command::ToggleLayout
            | Command::Find
            | Command::FindNext
            | Command::FindPrevious
            | Command::ToggleMarkdownView
            | Command::ToggleSoftWrap
            | Command::Scroll(_) => Ok(()),
        };
    }

    /// Up, Down, Page Up and Page Down on wrapped lines: `rows` rows, keeping the caret's column in its row.
    fn move_by_rows(&mut self, rows: isize, width: usize, selecting: Selecting, cx: &mut Context<Self>) -> Result<(), EditError> {
        let wrapped = Wrapped { model: &self.model, width, cache: &self.rows };
        let caret = self.model.caret();
        let goal = self.model.goal_column().unwrap_or_else(|| wrapped.column_in_row(caret));
        let target = wrapped.step(caret, rows, goal);
        self.navigate(|model| model.move_vertically(target, goal, selecting), cx)
    }

    fn copy(&mut self, target: ClipboardTarget, cx: &mut Context<Self>) -> Result<(), EditError> {
        if let Some(text) = self.model.selected_text() {
            Host::write_clipboard(target, text, cx);
        }
        Ok(())
    }

    fn cut(&mut self, cx: &mut Context<Self>) -> Result<(), EditError> {
        self.copy(ClipboardTarget::Clipboard, cx)?;
        self.edit(EditorModel::delete_selection, cx)
    }

    fn paste(&mut self, source: ClipboardTarget, cx: &mut Context<Self>) -> Result<(), EditError> {
        let Some(text) = Host::read_clipboard(source, cx) else {
            return Ok(());
        };
        self.edit(|model| model.insert(&text, EditKind::Other), cx)
    }

    fn page_rows(&self) -> usize {
        (self.scroll.viewport_lines() as usize).saturating_sub(1).max(1)
    }

    fn position_at(&self, position: Point<Pixels>) -> Option<Position> {
        let geometry = self.geometry.borrow();
        Some(geometry.as_ref()?.position_at(f32::from(position.x), f32::from(position.y), &self.model))
    }

    /// A right click on the line numbers asks for their menu.
    fn on_right_mouse_down(&mut self, event: &MouseDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let on_gutter = self.geometry.borrow().as_ref().is_some_and(|geometry| f32::from(event.position.x) < geometry.origin_x);
        if on_gutter {
            cx.stop_propagation();
            cx.emit(TextEditorEvent::GutterMenu);
        }
    }

    fn on_key_down(&mut self, event: &KeyDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if let Popup::Open(_) = self.popup
            && event.keystroke.key == "escape"
            && !event.keystroke.modifiers.modified()
        {
            cx.stop_propagation();
            return self.close_change_popup(cx);
        }
        match Host::key_outcome(event, KeyScope::Editor, cx) {
            // the panel around the editor runs the find box
            KeyOutcome::Command(command) if command.runs_in_panel() => {}
            KeyOutcome::Command(command) => {
                cx.stop_propagation();
                self.run(command, cx);
            }
            KeyOutcome::Handled => cx.stop_propagation(),
            // a window action (Settings, a new terminal) is the workspace's to run
            KeyOutcome::Foreign => {}
            // typed characters arrive through the input handler; other keys belong to the host
            KeyOutcome::Unbound => {
                let Some(command) = command(&event.keystroke, self.page_rows()) else {
                    return;
                };
                cx.stop_propagation();
                self.run(command, cx);
            }
        }
    }

    fn on_mouse_down(&mut self, event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.focus_handle, cx);
        let Some(position) = self.position_at(event.position) else {
            return;
        };
        match event.click_count {
            2 => self.model.select_word(position),
            3.. => self.model.select_line(position.line),
            _ if event.modifiers.shift => self.model.set_caret(position, Selecting::Extend),
            _ => self.model.set_caret(position, Selecting::Move),
        }
        self.dragging = true;
        cx.notify();
    }

    /// A mouse move anywhere in the window while a drag that started here goes on: past the editor's edges the
    /// caret follows the pointer to lines and columns out of view, which scrolls them in.
    fn on_window_mouse_move(&mut self, event: &MouseMoveEvent, cx: &mut Context<Self>) {
        let held = self.dragging && event.pressed_button == Some(MouseButton::Left);
        if let Some(position) = self.position_at(event.position).filter(|_| held) {
            self.model.set_caret(position, Selecting::Extend);
            self.reveal_caret = true;
            cx.notify();
        }
    }

    fn on_mouse_up(&mut self, _event: &MouseUpEvent, _window: &mut Window, _cx: &mut Context<Self>) {
        self.dragging = false;
    }

    /// Scrolls; the popup closes once its change is out of view.
    fn on_scroll_wheel(&mut self, event: &ScrollWheelEvent, line_height: Pixels, window: &mut Window, cx: &mut Context<Self>) {
        match self.wrap_width {
            Some(width) => self.scroll_rows_by_wheel(event, line_height, width, window),
            None => self.scroll.scroll_by_wheel(event, line_height, window),
        }
        let Some(block) = self.change_in_popup() else {
            return;
        };
        if self.shows_lines(&block.current) {
            return;
        }
        self.close_change_popup(cx);
    }

    /// Wrapped lines scroll by rows, which may be fewer or more than a line; there is no sideways scrolling.
    fn scroll_rows_by_wheel(&self, event: &ScrollWheelEvent, line_height: Pixels, width: usize, window: &mut Window) {
        let delta = event.delta.pixel_delta(line_height);
        let (_, dy) = wheel_axes(f32::from(delta.x), f32::from(delta.y), event.modifiers.shift);
        let wrapped = Wrapped { model: &self.model, width, cache: &self.rows };
        let top = wrapped.advance(wrapped.place_of_top(self.scroll.top()), -f64::from(dy / f32::from(line_height)));
        self.scroll.scroll_to(Axis::Vertical, wrapped.top_of(top));
        window.refresh();
    }

    /// Whether any of lines `lines` is in view; for no lines, the top of the line they would start.
    fn shows_lines(&self, lines: &Range<usize>) -> bool {
        if let Some(width) = self.wrap_width {
            let wrapped = Wrapped { model: &self.model, width, cache: &self.rows };
            return wrapped.shows_lines(self.scroll.top(), lines, self.scroll.viewport_lines());
        }
        let (top, bottom) = (self.scroll.top(), self.scroll.top() + self.scroll.viewport_lines());
        let end = lines.end.max(lines.start + 1);
        end as f64 > top && (lines.start as f64) < bottom
    }

    /// Scrolls to the caret when it moved, then gathers what this frame paints.
    fn frame(&mut self, metrics: Metrics, style: ViewStyle, window: &Window) -> Frame {
        let char_width = f64::from(f32::from(metrics.char_width));
        self.scroll.set_line_count(self.model.line_count());
        self.wrap_width = self.wrap_columns(char_width);
        let shown = match self.wrap_width {
            Some(width) => self.wrapped_rows(width, char_width),
            None => self.line_rows(metrics),
        };
        Frame {
            changes: self.markers.shown(shown.lines.clone()).cloned().collect(),
            rows: shown.rows,
            caret: shown.caret.filter(|_| self.focus_handle.is_focused(window)),
            tops: shown.tops,
            placement: shown.placement,
            metrics,
            style,
            focus: self.focus_handle.clone(),
            geometry: self.geometry.clone(),
        }
    }

    /// The columns of wrapped rows; `None` while lines are not wrapped, or before the first layout gave a width.
    fn wrap_columns(&self, char_width: f64) -> Option<usize> {
        let width = self.scroll.viewport_width();
        if self.soft_wrap == SoftWrap::Off || width <= 0. {
            return None;
        }
        Some(((width / char_width).floor() as usize).saturating_sub(WRAP_MARGIN_CHARS).max(1))
    }

    /// One row per line, scrolled sideways as a whole.
    fn line_rows(&mut self, metrics: Metrics) -> Shown {
        let char_width = f64::from(f32::from(metrics.char_width));
        self.scroll.set_wrapped_max_top(None);
        if self.reveal_caret {
            self.scroll.reveal_line(self.model.caret().line);
        }
        let first = self.scroll.top().floor() as usize;
        let rows = first..(first + self.scroll.viewport_lines().ceil() as usize + 1).min(self.model.line_count());
        let widest = (rows.start.min(rows.end)..rows.end).map(|line| self.model.line(line).chars().count()).max().unwrap_or(0);
        self.scroll.set_content_width(metrics.text_width(widest));
        // a tab just opened has no width before its first layout: the caret is revealed in the next frame
        if self.reveal_caret && self.scroll.viewport_width() > 0. {
            self.reveal_caret = false;
            self.scroll.reveal_x(char_width * self.model.caret().column as f64, char_width * CARET_MARGIN_CHARS);
        }
        let (top, left) = (self.scroll.top(), self.scroll.left());
        let first_column = (left / char_width).floor() as usize;
        let columns = first_column..first_column + visible_columns(self.scroll.viewport_width(), char_width) + 1;
        let highlights = self.highlights_of(rows.clone());
        let mut lines = visible_lines(&self.model, Area { rows: rows.clone(), columns }, &highlights);
        if let Some(marks) = &self.find_marks {
            lines.iter_mut().for_each(|line| line.marks = mark_columns(&self.model, line, marks));
        }
        let caret = self.model.caret();
        Shown {
            rows: lines.into_iter().map(|text| FrameRow { y: text.line as f64 - top, left, numbered: true, text }).collect(),
            lines: rows,
            caret: Some(CaretAt { x: char_width * caret.column as f64 - left, y: caret.line as f64 - top }),
            tops: LineTops::Scrolled { top },
            placement: Placement::Lines { top, left },
        }
    }

    /// Lines broken into rows `width` columns wide, scrolled by rows, never sideways; a row more above and below
    /// the viewport is laid out for a drag past its edges.
    fn wrapped_rows(&mut self, width: usize, char_width: f64) -> Shown {
        self.scroll.set_content_width(0.);
        let viewport = self.scroll.viewport_lines();
        let reveal = std::mem::take(&mut self.reveal_caret);
        let wrapped = Wrapped { model: &self.model, width, cache: &self.rows };
        self.scroll.set_wrapped_max_top(Some(wrapped.max_top(viewport)));
        if let Some(top) = wrapped.reveal(self.scroll.top(), self.model.caret(), viewport).filter(|_| reveal) {
            self.scroll.scroll_to(Axis::Vertical, top);
        }
        let top = wrapped.place_of_top(self.scroll.top());
        let first = RowPlace::new(top.line, top.row.floor());
        let above = wrapped.advance(first, -1.);
        let first_y = first.row - top.row - if above == first { 0. } else { 1. };
        let rows = wrapped.rows_from(above, viewport.ceil() as usize + 3);
        let lines = rows.first().map_or(0, |row| row.line)..rows.last().map_or(0, |row| row.line + 1);
        let highlights = self.highlights_of(lines.clone());
        let painted = self.painted_rows(&rows, first_y, char_width, &highlights);
        let tops = LineTops::of_rows(&rows, first_y, |line| wrapped.rows(line).count());
        let caret = self.model.caret();
        let caret_y = tops.line_y(caret.line) + wrapped.place_of(caret).row;
        let spans = rows.iter().enumerate().map(|(at, row)| RowSpan { line: row.line, start: row.columns.start, last_column: row.last_column, y: first_y + at as f64 });
        Shown {
            rows: painted,
            lines,
            caret: Some(CaretAt { x: char_width * wrapped.column_in_row(caret) as f64, y: caret_y }),
            tops,
            placement: Placement::Rows(spans.collect()),
        }
    }

    /// `rows` as painted, the first `first_y` rows from the viewport's top, each line read once.
    fn painted_rows(&self, rows: &[Row], first_y: f64, char_width: f64, highlights: &[Highlight]) -> Vec<FrameRow> {
        let mut painted = Vec::with_capacity(rows.len());
        for line_rows in rows.chunk_by(|one, other| one.line == other.line) {
            let spans: Vec<Range<usize>> = line_rows.iter().map(|row| row.columns.clone()).collect();
            for (row, mut text) in line_rows.iter().zip(line_spans(&self.model, line_rows[0].line, &spans, highlights)) {
                if let Some(marks) = &self.find_marks {
                    text.marks = mark_columns(&self.model, &text, marks);
                }
                let y = first_y + painted.len() as f64;
                painted.push(FrameRow { y, left: char_width * row.columns.start as f64, numbered: row.row == 0, text: within_row(text, row) });
            }
        }
        painted
    }
}

impl TextEditor {
    /// The bytes lines `rows` take, line breaks included.
    fn bytes_of(&self, rows: Range<usize>) -> Range<u64> {
        let table = self.model.table();
        let end = if rows.end < table.line_count() { table.line_start(rows.end) } else { table.byte_len() };
        table.line_start(rows.start)..end
    }

    /// Highlights of lines `rows`, queried per line once while the text and its colours stay.
    fn highlights_of(&self, rows: Range<usize>) -> Vec<Highlight> {
        self.colours.start_frame();
        let mut highlights = Vec::new();
        for line in rows {
            highlights.extend_from_slice(&self.colours.line_highlights(self.bytes_of(line..line + 1)));
        }
        highlights
    }
}

fn visible_columns(viewport_width: f64, char_width: f64) -> usize {
    if viewport_width <= 0. { FIRST_FRAME_COLUMNS } else { (viewport_width / char_width).ceil() as usize }
}

impl Focusable for TextEditor {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for TextEditor {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let style = ViewStyle::drawing(cx).clone();
        let metrics = Metrics::measure(window, cx, self.model.line_count());
        let frame = self.frame(metrics, style.clone(), window);
        // the message field had the keyboard and went with the popup: the text takes it back
        if let Popup::Closed = self.popup
            && self.message.focus_handle(cx).is_focused(window)
        {
            window.focus(&self.focus_handle, cx);
        }
        let popup = self.popup_element(&frame, cx);
        let line_height = metrics.line_height;
        // MCP's way to the gutter's menu, which a right click on the line numbers opens
        let editor = cx.weak_entity();
        let gutter_menu = move |_: &ClickEvent, _: &mut Window, cx: &mut App| drop(editor.update(cx, |_, cx| cx.emit(TextEditorEvent::GutterMenu)));
        drop(Host::click_target("gutter-menu".to_owned(), gutter_menu, cx));
        div()
            .id("text-editor")
            .track_focus(&self.focus_handle)
            .relative()
            .flex_1()
            .min_h_0()
            .overflow_hidden()
            .cursor(CursorStyle::IBeam)
            .on_key_down(cx.listener(Self::on_key_down))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_down(MouseButton::Right, cx.listener(Self::on_right_mouse_down))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            // a drag that ends outside the editor still ends the selection
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_scroll_wheel(cx.listener(move |editor, event, window, cx| editor.on_scroll_wheel(event, line_height, window, cx)))
            .child(line_view::measure_viewport(self.scroll.clone(), metrics))
            .children(change_targets(&frame, cx.weak_entity(), cx))
            .child(text_canvas(frame, cx.entity()))
            .child(vertical_scrollbar_with_ticks(&self.scrollbars, &style, self.find_ticks()))
            .child(horizontal_scrollbar(&self.scrollbars, &style))
            .children(popup)
    }
}

impl TextEditor {
    /// The open popup under (or over) its change; `None` while closed or while its change is out of view (scrolled
    /// away by the caret; the wheel closes it).
    fn popup_element(&self, frame: &Frame, cx: &mut Context<Self>) -> Option<AnyElement> {
        let Popup::Open(popup) = &self.popup else {
            return None;
        };
        if self.shows_lines(&popup.block().current) {
            return Some(self.shown_popup(popup, frame, cx));
        }
        None
    }

    fn shown_popup(&self, popup: &ChangePopup, frame: &Frame, cx: &mut Context<Self>) -> AnyElement {
        let (lines, metrics) = (&popup.block().current, frame.metrics);
        let anchor = Anchor {
            top: frame.line_top(px(0.), lines.start),
            bottom: frame.line_top(px(0.), lines.end),
            left: metrics.gutter_width(),
            width: px(self.scroll.viewport_width() as f32),
            height: metrics.line_height * self.scroll.viewport_lines() as f32,
            line_height: metrics.line_height,
            font_size: metrics.font_size,
        };
        let commit_shown = self.committing.shown(popup.block(), self.markers.head_id());
        popup_element(popup, anchor, popup_actions(cx.weak_entity(), &self.message, commit_shown), cx)
    }
}

/// The popup's buttons, each running an editor method, the commit message's field, and how committing shows.
fn popup_actions(editor: WeakEntity<TextEditor>, message: &Entity<LineField>, commit_shown: CommitShown) -> PopupActions {
    let on = |run: fn(&mut TextEditor, &mut Context<TextEditor>)| -> ClickListener {
        let editor = editor.clone();
        Rc::new(move |_: &ClickEvent, _window: &mut Window, cx: &mut App| drop(editor.update(cx, run)))
    };
    let closing = editor.clone();
    PopupActions {
        previous: on(|editor, cx| editor.step_change(Toward::Previous, cx)),
        next: on(|editor, cx| editor.step_change(Toward::Next, cx)),
        rollback: on(TextEditor::roll_back_change),
        diff: on(TextEditor::show_change_diff),
        copy: on(TextEditor::copy_previous_text),
        commit: on(TextEditor::commit_change),
        focus_message: {
            let field = message.downgrade();
            Rc::new(move |_: &ClickEvent, window: &mut Window, cx: &mut App| {
                if let Some(field) = field.upgrade() {
                    window.focus(&field.focus_handle(cx), cx);
                }
            })
        },
        message: message.clone().into_any_element(),
        commit_shown,
        close: Rc::new(move |_window: &mut Window, cx: &mut App| drop(closing.update(cx, TextEditor::close_change_popup))),
    }
}

/// Everything one frame paints, gathered in render so painting reads no editor state, and where painting reports
/// the text's place for the mouse and the input method.
struct Frame {
    rows: Vec<FrameRow>,
    /// the change markers on the lines shown
    changes: Vec<ChangeHunk>,
    /// `None` while the editor is not focused
    caret: Option<CaretAt>,
    tops: LineTops,
    placement: Placement,
    metrics: Metrics,
    style: ViewStyle,
    focus: FocusHandle,
    geometry: Rc<RefCell<Option<Geometry>>>,
}

/// What a frame shows, before it is painted.
struct Shown {
    rows: Vec<FrameRow>,
    /// the lines the rows are of
    lines: Range<usize>,
    caret: Option<CaretAt>,
    tops: LineTops,
    placement: Placement,
}

/// A row on screen: `y` rows below the viewport's top, its column 0 `left` pixels left of the text area; its line's
/// number shows on its line's first row.
struct FrameRow {
    text: VisibleLine,
    y: f64,
    left: f64,
    numbered: bool,
}

/// Where the caret is drawn: pixels right of the text area's left edge, rows below the viewport's top.
#[derive(Clone, Copy)]
struct CaretAt {
    x: f64,
    y: f64,
}

/// Where each line's first row is, in rows below the viewport's top.
enum LineTops {
    /// a line per row, line `top` (with its fraction) at the top
    Scrolled { top: f64 },
    /// the lines shown with the top of their first row and their row count, a first row above the viewport for a
    /// line scrolled into
    Wrapped(Vec<(usize, f64, usize)>),
}

impl LineTops {
    fn of_rows(rows: &[Row], first_y: f64, count: impl Fn(usize) -> usize) -> Self {
        let tops = rows.iter().enumerate().filter(|(at, row)| *at == 0 || row.row == 0).map(|(at, row)| (row.line, first_y + at as f64 - row.row as f64, count(row.line)));
        Self::Wrapped(tops.collect())
    }

    /// The top of line `line`'s first row; above or below the lines shown, as if each had one row.
    fn line_y(&self, line: usize) -> f64 {
        let tops = match self {
            Self::Scrolled { top } => return line as f64 - top,
            Self::Wrapped(tops) => tops,
        };
        let at = tops.partition_point(|(shown, _, _)| *shown < line);
        match (tops.get(at), tops.first(), tops.last()) {
            (Some((shown, y, _)), _, _) if *shown == line => *y,
            (Some(_), Some((first, y, _)), _) if at == 0 => y - (first - line) as f64,
            (_, _, Some((last, y, rows))) if line > *last => y + *rows as f64 + (line - last - 1) as f64,
            _ => 0.,
        }
    }
}

/// `text`, one row of a wrapped line, with its selection and matches cut to the row's columns: a row before the
/// line's last ends before the next row's first column, the last one goes on past the line's end for its break.
fn within_row(text: VisibleLine, row: &Row) -> VisibleLine {
    let end = if row.is_last { usize::MAX } else { row.columns.end };
    let cut = |columns: &Range<usize>| Some(columns.start.max(row.columns.start)..columns.end.min(end)).filter(|cut| !cut.is_empty());
    let marks = text.marks.iter().filter_map(|(columns, mark)| Some((cut(columns)?, *mark))).collect();
    let selected = text.selected.as_ref().and_then(cut);
    VisibleLine { selected, marks, ..text }
}

/// How many rows Up, Down, Page Up and Page Down move; `None` for the other motions.
fn rows_moved(motion: Motion) -> Option<isize> {
    match motion {
        Motion::Up => Some(-1),
        Motion::Down => Some(1),
        Motion::PageUp(rows) => Some(-(rows as isize)),
        Motion::PageDown(rows) => Some(rows as isize),
        _ => None,
    }
}

impl Frame {
    /// Right of the line-number gutter.
    fn text_area(&self, bounds: Bounds<Pixels>) -> Bounds<Pixels> {
        let gutter = self.metrics.gutter_width();
        Bounds::new(point(bounds.origin.x + gutter, bounds.origin.y), size(bounds.size.width - gutter, bounds.size.height))
    }

    fn geometry(&self, text_area: Bounds<Pixels>) -> Geometry {
        Geometry {
            origin_x: f32::from(text_area.origin.x),
            origin_y: f32::from(text_area.origin.y),
            line_height: f32::from(self.metrics.line_height),
            char_width: f32::from(self.metrics.char_width),
            placement: self.placement.clone(),
        }
    }

    /// Where change markers stand: the last column of the gutter, right of the line numbers.
    fn marker_x(&self, origin_x: Pixels) -> Pixels {
        origin_x + self.metrics.gutter_width() - self.metrics.char_width / 2. - px(CHANGE_BAR_WIDTH / 2.)
    }

    fn line_top(&self, origin_y: Pixels, line: usize) -> Pixels {
        self.row_top(origin_y, self.tops.line_y(line))
    }

    fn row_top(&self, origin_y: Pixels, y: f64) -> Pixels {
        origin_y + self.metrics.line_height * y as f32
    }

    /// Where column `column` of a row whose column 0 is `left` pixels left of `origin_x` is.
    fn column_x(&self, origin_x: Pixels, column: usize, left: f64) -> Pixels {
        origin_x + self.metrics.char_width * column as f32 - px(left as f32)
    }

    /// `len` bytes in the editor's font, looking as `attributes` say; in the text colour where they set none.
    fn run(&self, len: usize, attributes: TextAttributes) -> TextRun {
        let mut font = gpui::font(self.style.mono_font.clone());
        if attributes.bold {
            font.weight = FontWeight::BOLD;
        }
        if attributes.italic {
            font.style = FontStyle::Italic;
        }
        TextRun {
            len,
            font,
            color: rgb(attributes.foreground.unwrap_or(self.style.text)).into(),
            background_color: attributes.background.map(|background| rgb(background).into()),
            underline: None,
            strikethrough: None,
        }
    }
}

/// Paints the frame; while focused, takes the platform's text input (IME included); follows a drag past the edges.
fn text_canvas(frame: Frame, editor: Entity<TextEditor>) -> impl IntoElement {
    canvas(
        |_bounds, _window, _cx| {},
        move |bounds, (), window, cx| {
            *frame.geometry.borrow_mut() = Some(frame.geometry(frame.text_area(bounds)));
            window.handle_input(&frame.focus, ElementInputHandler::new(bounds, editor.clone()), cx);
            // an element's own mouse-move listener hears only moves over it
            window.on_mouse_event(move |event: &MouseMoveEvent, phase, _window, cx| {
                if phase == DispatchPhase::Bubble {
                    editor.update(cx, |editor, cx| editor.on_window_mouse_move(event, cx));
                }
            });
            paint_frame(&frame, bounds, window, cx);
        },
    )
    .absolute()
    .size_full()
}

fn paint_frame(frame: &Frame, bounds: Bounds<Pixels>, window: &mut Window, cx: &mut App) {
    let text_area = frame.text_area(bounds);
    let mask = Some(ContentMask { bounds: text_area });
    // the row laid out above the viewport, for a drag, is not painted
    for row in frame.rows.iter().filter(|row| row.y > -1.) {
        let top = frame.row_top(bounds.origin.y, row.y);
        if row.numbered {
            let number = format!("{:>width$}", row.text.line + 1, width = frame.metrics.number_chars);
            let origin = point(bounds.origin.x + frame.metrics.char_width, top);
            let runs = vec![frame.run(number.len(), TextAttributes { foreground: Some(frame.style.muted), ..TextAttributes::default() })];
            paint_text(frame, Glyphs { text: number, origin, runs }, window, cx);
        }
        window.with_content_mask(mask, |window| paint_line(frame, row, point(text_area.origin.x, top), window, cx));
    }
    let gutter = Bounds::new(bounds.origin, size(frame.metrics.gutter_width(), bounds.size.height));
    window.with_content_mask(Some(ContentMask { bounds: gutter }), |window| paint_changes(frame, bounds.origin, window));
    if let Some(caret) = frame.caret {
        let at = point(text_area.origin.x + px(caret.x as f32), frame.row_top(text_area.origin.y, caret.y));
        let quad = fill(Bounds::new(at, size(px(frame.style.editor_caret_width), frame.metrics.line_height)), rgb(frame.style.text));
        window.with_content_mask(mask, |window| window.paint_quad(quad));
    }
}

/// Bars on added and changed lines, a triangle pointing at the text where lines were deleted.
fn paint_changes(frame: &Frame, origin: Point<Pixels>, window: &mut Window) {
    let x = frame.marker_x(origin.x);
    for hunk in &frame.changes {
        let (top, bottom) = (frame.line_top(origin.y, hunk.current.start), frame.line_top(origin.y, hunk.current.end));
        let bar = Bounds::from_corners(point(x, top), point(x + px(CHANGE_BAR_WIDTH), bottom));
        match hunk.marker() {
            Marker::Added => window.paint_quad(fill(bar, rgb(frame.style.change_added))),
            Marker::Changed => window.paint_quad(fill(bar, rgb(frame.style.change_modified))),
            Marker::Deleted => paint_triangle(point(x, top), rgb(frame.style.change_deleted), window),
        }
    }
}

/// A triangle pointing right, its left side centred on `at`.
fn paint_triangle(at: Point<Pixels>, colour: gpui::Rgba, window: &mut Window) {
    let half = px(DELETED_MARKER_HEIGHT / 2.);
    let mut path = PathBuilder::fill();
    path.move_to(point(at.x, at.y - half));
    path.line_to(point(at.x + half, at.y));
    path.line_to(point(at.x, at.y + half));
    path.close();
    // fails only for a path lyon cannot tessellate, which three distinct points are not
    if let Ok(path) = path.build() {
        window.paint_path(path, colour);
    }
}

/// An element over each change marker shown, registered as click target `change-marker:<line>` (its first line,
/// from 1), that opens the change's popup; the caret stays where it is.
fn change_targets(frame: &Frame, editor: WeakEntity<TextEditor>, cx: &App) -> Vec<AnyElement> {
    let (line_height, x) = (frame.metrics.line_height, frame.marker_x(px(0.)) - frame.metrics.char_width / 4.);
    let width = px(CHANGE_BAR_WIDTH) + frame.metrics.char_width / 2.;
    let element = |hunk: &ChangeHunk| {
        let line = hunk.current.start;
        let (top, lines) = match hunk.marker() {
            Marker::Deleted => (frame.line_top(px(0.), line) - px(DELETED_MARKER_HEIGHT / 2.), px(DELETED_MARKER_HEIGHT)),
            Marker::Added | Marker::Changed => (frame.line_top(px(0.), line), line_height * hunk.current.len() as f32),
        };
        let editor = editor.clone();
        let open = move |_: &ClickEvent, window: &mut Window, cx: &mut App| drop(editor.update(cx, |editor, cx| editor.open_change_popup(line, window, cx)));
        let click = Host::click_target(format!("change-marker:{}", line + 1), open, cx);
        div()
            .id(("change-marker", line))
            .absolute()
            .left(x)
            .top(top)
            .w(width)
            .h(lines)
            .cursor_pointer()
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .on_click(click)
            .into_any_element()
    };
    frame.changes.iter().map(element).collect()
}

fn paint_line(frame: &Frame, row: &FrameRow, origin: Point<Pixels>, window: &mut Window, cx: &mut App) {
    let (line, left) = (&row.text, row.left);
    let columns_box = |columns: &Range<usize>| {
        let from = point(frame.column_x(origin.x, columns.start, left), origin.y);
        Bounds::from_corners(from, point(frame.column_x(origin.x, columns.end, left), origin.y + frame.metrics.line_height))
    };
    let marked = |mark: Mark| line.marks.iter().filter(move |(_, kind)| *kind == mark).map(|(columns, _)| columns);
    // the matches under the selection, the current one over it
    for columns in marked(Mark::Match) {
        window.paint_quad(fill(columns_box(columns), rgba(frame.style.find_match)));
    }
    if let Some(selected) = &line.selected {
        window.paint_quad(fill(columns_box(selected), rgb(frame.style.selected)));
    }
    for columns in marked(Mark::Current) {
        window.paint_quad(fill(columns_box(columns), rgba(frame.style.find_current)));
    }
    if line.text.is_empty() {
        return;
    }
    let origin = point(frame.column_x(origin.x, line.first_column, left), origin.y);
    let runs = coloured_runs(frame, line, cx.global::<ResolvedScheme>());
    paint_text(frame, Glyphs { text: line.text.clone(), origin, runs }, window, cx);
}

/// `line`'s text in runs: its colours from the scheme, plain text between them.
fn coloured_runs(frame: &Frame, line: &VisibleLine, scheme: &ResolvedScheme) -> Vec<TextRun> {
    let plain = |len| frame.run(len, TextAttributes::default());
    let mut runs = Vec::new();
    let mut at = 0;
    for (bytes, key) in &line.colours {
        if bytes.start > at {
            runs.push(plain(bytes.start - at));
        }
        runs.push(frame.run(bytes.len(), scheme.get(*key)));
        at = bytes.end;
    }
    if line.text.len() > at {
        runs.push(plain(line.text.len() - at));
    }
    runs
}

/// Text to paint at a place, in runs covering it.
struct Glyphs {
    text: String,
    origin: Point<Pixels>,
    runs: Vec<TextRun>,
}

/// Glyphs are pinned to the character grid, so a column's x position is exact.
fn paint_text(frame: &Frame, glyphs: Glyphs, window: &mut Window, cx: &mut App) {
    let shaped = window.text_system().shape_line(glyphs.text.into(), frame.metrics.font_size, &glyphs.runs, Some(frame.metrics.char_width));
    // fails only when a glyph cannot be rasterized; the rest is still worth drawing
    let _ = shaped.paint(glyphs.origin, frame.metrics.line_height, gpui::TextAlign::Left, None, window, cx);
}

impl EntityInputHandler for TextEditor {
    fn text_for_range(&mut self, range: Range<usize>, adjusted: &mut Option<Range<usize>>, _window: &mut Window, _cx: &mut Context<Self>) -> Option<String> {
        let length = self.model.utf16_len();
        let range = range.start.min(length)..range.end.min(length);
        adjusted.replace(range.clone());
        Some(self.model.text_utf16(range))
    }

    fn selected_text_range(&mut self, _ignore_disabled_input: bool, _window: &mut Window, _cx: &mut Context<Self>) -> Option<UTF16Selection> {
        let (range, reversed) = self.model.selection_utf16();
        Some(UTF16Selection { range, reversed })
    }

    fn marked_text_range(&self, _window: &mut Window, _cx: &mut Context<Self>) -> Option<Range<usize>> {
        self.model.marked_utf16()
    }

    fn unmark_text(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {
        self.model.unmark();
    }

    fn replace_text_in_range(&mut self, range: Option<Range<usize>>, text: &str, _window: &mut Window, cx: &mut Context<Self>) {
        let _ = self.edit(|model| model.replace_utf16(range, text), cx);
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        selected: Option<Range<usize>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let _ = self.edit(|model| model.mark_utf16(range, text, selected), cx);
    }

    fn bounds_for_range(&mut self, range: Range<usize>, _element_bounds: Bounds<Pixels>, _window: &mut Window, _cx: &mut Context<Self>) -> Option<Bounds<Pixels>> {
        let geometry = self.geometry.borrow().clone()?;
        let (x, y) = geometry.point_of(self.model.position_utf16(range.start));
        Some(Bounds::new(point(px(x), px(y)), size(px(geometry.char_width), px(geometry.line_height))))
    }

    fn character_index_for_point(&mut self, point: Point<Pixels>, _window: &mut Window, _cx: &mut Context<Self>) -> Option<usize> {
        let position = self.position_at(point)?;
        Some(self.model.offset_utf16(position))
    }
}
