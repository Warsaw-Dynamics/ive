//! The text editor entity: draws the visible lines on a canvas, takes keys, mouse and input-method text.

use std::cell::Cell;
use std::ops::Range;
use std::rc::Rc;
use std::sync::Arc;

use gpui::{
    AnyElement, App, Bounds, ClickEvent, ContentMask, Context, CursorStyle, DispatchPhase, ElementInputHandler, Entity,
    EntityInputHandler, EventEmitter, FocusHandle, Focusable, FontStyle, FontWeight, KeyDownEvent, MouseButton,
    MouseDownEvent, MouseMoveEvent, MouseUpEvent, PathBuilder, Pixels, Point, ScrollWheelEvent, Subscription, TextRun, UTF16Selection,
    WeakEntity, Window, canvas,
    div, fill, point, prelude::*, px, rgb, size,
};

use super::change_popup::{Anchor, ChangeAction, ChangePopup, CommitShown, Committing, Popup, PopupActions, SentChange, popup_element};
use super::message_field::{FieldState, MessageField, MessageFieldEvent};
use super::colours::Colours;
use super::keys::command;
use super::layout::{Area, Geometry, VisibleLine, visible_lines};
use super::model::{EditError, EditKind, EditorModel, INDENT, Position, Selecting, SizeLimit};
use crate::changes::{BaseChange, ChangeBase, ChangeDiffJob, ChangeHunk, ChangeHunks, ChangeMarkers, Marker, Toward};
use crate::colors::{ResolvedScheme, TextAttributes};
use crate::line_view::{self, LineScroll, Metrics};
use crate::panel::{Edit, LineSource, ParseJob};
use crate::piece_table::{PieceTable, WrongBase};
use crate::host::{ClickListener, ClipboardTarget, Command, Host, KeyOutcome, KeyScope};
use crate::position::{CaretPosition, ScrollPosition};
use crate::scrollbar::{ScrollState, horizontal_scrollbar, vertical_scrollbar};
use crate::style::ViewStyle;
use crate::syntax::{Highlight, Language, Syntax};

/// Columns assumed visible before the first layout reports the viewport width.
const FIRST_FRAME_COLUMNS: usize = 400;
/// Columns kept between the caret and the viewport's edge when scrolling sideways to it.
const CARET_MARGIN_CHARS: f64 = 2.;
/// The width of a change marker's bar, and the height of a deleted marker's triangle, in pixels.
const CHANGE_BAR_WIDTH: f32 = 3.;
const DELETED_MARKER_HEIGHT: f32 = 8.;

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
    message: Entity<MessageField>,
    committing: Committing,
    /// closes the popup when the keyboard leaves the editor (and what it holds), from the first popup on
    focus_out: FocusWatch,
    focus_handle: FocusHandle,
    scroll: Rc<LineScroll>,
    scrollbars: ScrollState,
    /// where the last frame painted the text, for the mouse and the input method
    geometry: Rc<Cell<Option<Geometry>>>,
    /// a left-button drag that started in the editor goes on
    dragging: bool,
    reveal_caret: bool,
    _message_submits: Subscription,
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
        let message = cx.new(MessageField::new);
        let submits = cx.subscribe(&message, |editor, _field, MessageFieldEvent::Submitted, cx| editor.commit_change(cx));
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
            dragging: false,
            reveal_caret: false,
        }
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

    /// Puts the caret at the start of line `line` (from 1, clamped) and scrolls it into view.
    pub fn reveal_line(&mut self, line: usize, cx: &mut Context<Self>) {
        self.model.set_caret(Position::new(line.saturating_sub(1), 0), Selecting::Move);
        self.reveal_caret = true;
        cx.notify();
    }

    /// Replaces the text after a reload: the history, the colours and the change markers are dropped (the markers'
    /// base stays), the caret stays where it still fits.
    pub fn reset(&mut self, table: PieceTable, setup: Setup, cx: &mut Context<Self>) {
        let caret = self.model.caret();
        self.model = EditorModel::new(table, setup.limit);
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
            Command::Move(motion, selecting) => self.navigate(|model| model.move_caret(motion, selecting), cx),
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
            // saving is the panel's, and a text has no layout to toggle
            Command::Flush | Command::ToggleLayout => Ok(()),
        };
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
        let geometry = self.geometry.get()?;
        Some(geometry.position_at(f32::from(position.x), f32::from(position.y), &self.model))
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
        self.scroll.scroll_by_wheel(event, line_height, window);
        let Some(block) = self.change_in_popup() else {
            return;
        };
        if self.shows_lines(&block.current) {
            return;
        }
        self.close_change_popup(cx);
    }

    /// Whether any of lines `lines` is in view; for no lines, the top of the line they would start.
    fn shows_lines(&self, lines: &Range<usize>) -> bool {
        let (top, bottom) = (self.scroll.top(), self.scroll.top() + self.scroll.viewport_lines());
        let end = lines.end.max(lines.start + 1);
        end as f64 > top && (lines.start as f64) < bottom
    }

    /// Scrolls to the caret when it moved, then gathers what this frame paints.
    fn frame(&mut self, metrics: Metrics, style: ViewStyle, window: &Window) -> Frame {
        let char_width = f64::from(f32::from(metrics.char_width));
        self.scroll.set_line_count(self.model.line_count());
        if self.reveal_caret {
            self.scroll.reveal_line(self.model.caret().line);
        }
        let first = self.scroll.top().floor() as usize;
        let rows = first..(first + self.scroll.viewport_lines().ceil() as usize + 1).min(self.model.line_count());
        let widest = (rows.start.min(rows.end)..rows.end).map(|line| self.model.line(line).chars().count()).max().unwrap_or(0);
        self.scroll.set_content_width(metrics.text_width(widest));
        if std::mem::take(&mut self.reveal_caret) {
            self.scroll.reveal_x(char_width * self.model.caret().column as f64, char_width * CARET_MARGIN_CHARS);
        }
        let first_column = (self.scroll.left() / char_width).floor() as usize;
        let columns = first_column..first_column + visible_columns(self.scroll.viewport_width(), char_width) + 1;
        let highlights = self.highlights_of(rows.clone());
        Frame {
            changes: self.markers.shown(rows.clone()).cloned().collect(),
            lines: visible_lines(&self.model, Area { rows, columns }, &highlights),
            caret: self.focus_handle.is_focused(window).then(|| self.model.caret()),
            top: self.scroll.top(),
            left: self.scroll.left(),
            metrics,
            style,
            focus: self.focus_handle.clone(),
            geometry: self.geometry.clone(),
        }
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
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            // a drag that ends outside the editor still ends the selection
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_scroll_wheel(cx.listener(move |editor, event, window, cx| editor.on_scroll_wheel(event, line_height, window, cx)))
            .child(line_view::measure_viewport(self.scroll.clone(), metrics))
            .children(change_targets(&frame, cx.weak_entity(), cx))
            .child(text_canvas(frame, cx.entity()))
            .child(vertical_scrollbar(&self.scrollbars, &style))
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
fn popup_actions(editor: WeakEntity<TextEditor>, message: &Entity<MessageField>, commit_shown: CommitShown) -> PopupActions {
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
    lines: Vec<VisibleLine>,
    /// the change markers on the lines shown
    changes: Vec<ChangeHunk>,
    /// `None` while the editor is not focused
    caret: Option<Position>,
    top: f64,
    left: f64,
    metrics: Metrics,
    style: ViewStyle,
    focus: FocusHandle,
    geometry: Rc<Cell<Option<Geometry>>>,
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
            top: self.top,
            left: self.left,
        }
    }

    /// Where change markers stand: the last column of the gutter, right of the line numbers.
    fn marker_x(&self, origin_x: Pixels) -> Pixels {
        origin_x + self.metrics.gutter_width() - self.metrics.char_width / 2. - px(CHANGE_BAR_WIDTH / 2.)
    }

    fn line_top(&self, origin_y: Pixels, line: usize) -> Pixels {
        origin_y + self.metrics.line_height * (line as f64 - self.top) as f32
    }

    fn column_x(&self, origin_x: Pixels, column: usize) -> Pixels {
        origin_x + self.metrics.char_width * column as f32 - px(self.left as f32)
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
            frame.geometry.set(Some(frame.geometry(frame.text_area(bounds))));
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
    for line in &frame.lines {
        let top = frame.line_top(bounds.origin.y, line.line);
        let number = format!("{:>width$}", line.line + 1, width = frame.metrics.number_chars);
        let origin = point(bounds.origin.x + frame.metrics.char_width, top);
        let runs = vec![frame.run(number.len(), TextAttributes { foreground: Some(frame.style.muted), ..TextAttributes::default() })];
        paint_text(frame, Glyphs { text: number, origin, runs }, window, cx);
        window.with_content_mask(mask, |window| paint_line(frame, line, point(text_area.origin.x, top), window, cx));
    }
    let gutter = Bounds::new(bounds.origin, size(frame.metrics.gutter_width(), bounds.size.height));
    window.with_content_mask(Some(ContentMask { bounds: gutter }), |window| paint_changes(frame, bounds.origin, window));
    if let Some(caret) = frame.caret {
        let at = point(frame.column_x(text_area.origin.x, caret.column), frame.line_top(text_area.origin.y, caret.line));
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

fn paint_line(frame: &Frame, line: &VisibleLine, origin: Point<Pixels>, window: &mut Window, cx: &mut App) {
    if let Some(selected) = &line.selected {
        let from = point(frame.column_x(origin.x, selected.start), origin.y);
        let to = point(frame.column_x(origin.x, selected.end), origin.y + frame.metrics.line_height);
        window.paint_quad(fill(Bounds::from_corners(from, to), rgb(frame.style.selected)));
    }
    if line.text.is_empty() {
        return;
    }
    let origin = point(frame.column_x(origin.x, line.first_column), origin.y);
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
        let geometry = self.geometry.get()?;
        let (x, y) = geometry.point_of(self.model.position_utf16(range.start));
        Some(Bounds::new(point(px(x), px(y)), size(px(geometry.char_width), px(geometry.line_height))))
    }

    fn character_index_for_point(&mut self, point: Point<Pixels>, _window: &mut Window, _cx: &mut Context<Self>) -> Option<usize> {
        let position = self.position_at(point)?;
        Some(self.model.offset_utf16(position))
    }
}
