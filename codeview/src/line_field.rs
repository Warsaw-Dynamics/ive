//! A one-line text field: a change popup's commit message, the find box's query. It has the keyboard while focused:
//! the view around it types nothing then.

use std::cell::Cell;
use std::ops::Range;
use std::rc::Rc;

use gpui::{
    App, Bounds, Context, CursorStyle, ElementInputHandler, EntityInputHandler, EventEmitter, FocusHandle, Focusable,
    KeyDownEvent, Keystroke, MouseButton, Pixels, Point, SharedString, UTF16Selection, Window, canvas, div, prelude::*, px, rgb,
};

use crate::changes::Toward;
use crate::find::FindOption;
use crate::text_editor::Motion;
use crate::text_editor::keys::command;
use crate::host::{ClipboardTarget, Command, Host, KeyOutcome, KeyScope};
use crate::style::ViewStyle;

const HEIGHT: f32 = 22.;
const MIN_WIDTH: f32 = 120.;

pub(crate) enum LineFieldEvent {
    /// Enter, or Shift+Enter (previous)
    Submitted(Toward),
    /// the text changed
    Edited,
    /// a find option's key (Alt+C, Alt+W, Alt+R; on macOS with Cmd)
    Toggled(FindOption),
}

/// What a key means to the field before the host's bindings: Enter and Shift+Enter, the find options' keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FieldKey {
    Submit(Toward),
    Toggle(FindOption),
}

/// Which keys a field takes besides typing and moving: the find options' only in the find box, so a commit message
/// leaves them (macOS Cmd+Alt+W: Stop Tab) to the window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Keys {
    Plain,
    WithFindOptions,
}

pub(crate) fn field_key(keystroke: &Keystroke, keys: Keys) -> Option<FieldKey> {
    let modifiers = &keystroke.modifiers;
    match keystroke.key.as_str() {
        "enter" if modifiers.control || modifiers.alt || modifiers.platform => None,
        "enter" if modifiers.shift => Some(FieldKey::Submit(Toward::Previous)),
        "enter" => Some(FieldKey::Submit(Toward::Next)),
        key if keys == Keys::WithFindOptions && is_option_chord(modifiers) => option_of(key).map(FieldKey::Toggle),
        _ => None,
    }
}

/// Alt, and on macOS Cmd+Alt, since Alt alone types characters there.
fn is_option_chord(modifiers: &gpui::Modifiers) -> bool {
    let platform = cfg!(target_os = "macos");
    modifiers.alt && modifiers.platform == platform && !modifiers.control && !modifiers.shift
}

fn option_of(key: &str) -> Option<FindOption> {
    match key {
        "c" => Some(FindOption::MatchCase),
        "w" => Some(FindOption::WholeWord),
        "r" => Some(FindOption::Regex),
        _ => None,
    }
}

/// How the field's border looks: the find box's turns red when nothing matches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Tone {
    Plain,
    Error,
}

/// Whether the field takes typing; it does not while its message is being committed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FieldState {
    Editable,
    Busy,
}

pub(crate) struct LineField {
    text: String,
    /// where the caret is, in bytes of `text`, on a character boundary
    caret: usize,
    /// the text the input method is composing, in bytes of `text`
    marked: Option<Range<usize>>,
    /// what typing replaces, in bytes of `text`: all of it after a select-all, else nothing
    selected: Option<Range<usize>>,
    tone: Tone,
    state: FieldState,
    focus_handle: FocusHandle,
    /// where the field was last painted, for the input method
    bounds: Rc<Cell<Option<Bounds<Pixels>>>>,
    /// shown while the text is empty
    placeholder: &'static str,
    /// the element's id
    id: &'static str,
    keys: Keys,
}

impl EventEmitter<LineFieldEvent> for LineField {}

impl Focusable for LineField {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl LineField {
    pub(crate) fn new(placeholder: &'static str, id: &'static str, cx: &mut Context<Self>) -> Self {
        Self {
            text: String::new(),
            caret: 0,
            marked: None,
            selected: None,
            tone: Tone::Plain,
            state: FieldState::Editable,
            focus_handle: cx.focus_handle(),
            bounds: Rc::default(),
            placeholder,
            id,
            keys: Keys::Plain,
        }
    }

    /// This field taking the find options' keys too.
    pub(crate) fn with_find_options(self) -> Self {
        Self { keys: Keys::WithFindOptions, ..self }
    }

    pub(crate) fn text(&self) -> &str {
        &self.text
    }

    /// Replaces the text with `text`, all of it selected.
    pub(crate) fn set_text(&mut self, text: &str, cx: &mut Context<Self>) {
        self.text = text.replace(['\r', '\n'], " ");
        self.marked = None;
        self.select_all(cx);
    }

    pub(crate) fn select_all(&mut self, cx: &mut Context<Self>) {
        self.caret = self.text.len();
        self.selected = Some(0..self.text.len()).filter(|all| !all.is_empty());
        cx.notify();
    }

    /// Types `text` as the keyboard would: over the selection, else at the caret.
    pub(crate) fn type_over(&mut self, text: &str, cx: &mut Context<Self>) {
        self.replace(self.target(None), text);
        self.edited(cx);
    }

    pub(crate) fn set_tone(&mut self, tone: Tone, cx: &mut Context<Self>) {
        self.tone = tone;
        cx.notify();
    }

    /// Tells the field's owner the text changed.
    fn edited(&mut self, cx: &mut Context<Self>) {
        cx.emit(LineFieldEvent::Edited);
        cx.notify();
    }

    pub(crate) fn clear(&mut self, cx: &mut Context<Self>) {
        self.text.clear();
        self.caret = 0;
        self.marked = None;
        self.selected = None;
        cx.notify();
    }

    pub(crate) fn set_state(&mut self, state: FieldState, cx: &mut Context<Self>) {
        self.state = state;
        cx.notify();
    }

    /// Replaces bytes `range` of the text with `text` (line breaks made spaces); the caret goes after it.
    fn replace(&mut self, range: Range<usize>, text: &str) -> Range<usize> {
        let text = text.replace("\r\n", " ").replace(['\r', '\n'], " ");
        self.text.replace_range(range.clone(), &text);
        self.caret = range.start + text.len();
        self.marked = None;
        self.selected = None;
        range.start..self.caret
    }

    fn on_key_down(&mut self, event: &KeyDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if let Some(key) = field_key(&event.keystroke, self.keys) {
            cx.stop_propagation();
            return cx.emit(match key {
                FieldKey::Submit(toward) => LineFieldEvent::Submitted(toward),
                FieldKey::Toggle(option) => LineFieldEvent::Toggled(option),
            });
        }
        match Host::key_outcome(event, KeyScope::Editor, cx) {
            // the find commands are the panel's, around the field
            KeyOutcome::Command(command) if command.runs_in_panel() => {}
            KeyOutcome::Command(command) => {
                cx.stop_propagation();
                self.run(command, cx);
            }
            KeyOutcome::Handled => cx.stop_propagation(),
            // the window's to run
            KeyOutcome::Foreign => {}
            // typed characters arrive through the input handler; Escape goes on to the editor, which closes the popup
            KeyOutcome::Unbound => {
                let Some(command) = command(&event.keystroke, 1) else {
                    return;
                };
                cx.stop_propagation();
                self.run(command, cx);
            }
        }
    }

    /// Runs `command` as a one-line field does; moving between lines and undoing do nothing here.
    fn run(&mut self, command: Command, cx: &mut Context<Self>) {
        match command {
            Command::Move(motion, _) => self.move_caret(motion),
            Command::Copy(target) => Host::write_clipboard(target, self.text.clone(), cx),
            Command::SelectAll => self.select_all(cx),
            _ if self.state == FieldState::Busy => {}
            Command::Newline => cx.emit(LineFieldEvent::Submitted(Toward::Next)),
            Command::Backspace(_) => self.delete(self.before(self.caret)..self.caret, cx),
            Command::Delete(_) => self.delete(self.caret..self.after(self.caret), cx),
            Command::Paste(source) => self.paste(source, cx),
            Command::Cut => {
                Host::write_clipboard(ClipboardTarget::Clipboard, std::mem::take(&mut self.text), cx);
                self.caret = 0;
                self.selected = None;
                self.edited(cx);
            }
            Command::Undo | Command::Redo | Command::Indent | Command::Flush | Command::ToggleLayout => {}
            Command::Find | Command::FindNext | Command::FindPrevious | Command::ToggleMarkdownView => {}
        }
        cx.notify();
    }

    /// Without a selection Left and Right step a character; with one, they go to its start or end.
    fn move_caret(&mut self, motion: Motion) {
        self.caret = match (self.selected.take(), motion) {
            (Some(selected), Motion::Left | Motion::WordLeft) => selected.start,
            (Some(selected), Motion::Right | Motion::WordRight) => selected.end,
            _ => self.moved(motion),
        };
    }

    /// Deletes the selection if there is one, else `range`.
    fn delete(&mut self, range: Range<usize>, cx: &mut Context<Self>) {
        let range = self.selected.clone().unwrap_or(range);
        if range.is_empty() {
            return;
        }
        self.replace(range, "");
        self.edited(cx);
    }

    fn paste(&mut self, source: ClipboardTarget, cx: &mut Context<Self>) {
        if let Some(text) = Host::read_clipboard(source, cx) {
            self.replace(self.target(None), &text);
            self.edited(cx);
        }
    }

    fn moved(&self, motion: Motion) -> usize {
        match motion {
            Motion::Left | Motion::WordLeft => self.before(self.caret),
            Motion::Right | Motion::WordRight => self.after(self.caret),
            Motion::LineStart | Motion::DocumentStart => 0,
            Motion::LineEnd | Motion::DocumentEnd => self.text.len(),
            Motion::Up | Motion::Down | Motion::PageUp(_) | Motion::PageDown(_) => self.caret,
        }
    }

    /// The character boundary before `at`, or the start.
    fn before(&self, at: usize) -> usize {
        self.text[..at].char_indices().next_back().map_or(0, |(index, _)| index)
    }

    /// The character boundary after `at`, or the end.
    fn after(&self, at: usize) -> usize {
        self.text[at..].chars().next().map_or(at, |next| at + next.len_utf8())
    }

    fn utf16_of(&self, byte: usize) -> usize {
        self.text[..byte].encode_utf16().count()
    }

    fn byte_of(&self, utf16: usize) -> usize {
        let mut counted = 0;
        for (index, character) in self.text.char_indices() {
            if counted >= utf16 {
                return index;
            }
            counted += character.len_utf16();
        }
        self.text.len()
    }

    fn bytes_of(&self, utf16: Range<usize>) -> Range<usize> {
        self.byte_of(utf16.start)..self.byte_of(utf16.end)
    }

    /// What an input method's edit replaces: `range`, else the text being composed, else the caret.
    fn target(&self, range: Option<Range<usize>>) -> Range<usize> {
        let typed_over = self.marked.clone().or(self.selected.clone());
        range.map(|range| self.bytes_of(range)).or(typed_over).unwrap_or(self.caret..self.caret)
    }
}

impl EntityInputHandler for LineField {
    fn text_for_range(&mut self, range: Range<usize>, adjusted: &mut Option<Range<usize>>, _window: &mut Window, _cx: &mut Context<Self>) -> Option<String> {
        let bytes = self.bytes_of(range);
        adjusted.replace(self.utf16_of(bytes.start)..self.utf16_of(bytes.end));
        Some(self.text[bytes].to_owned())
    }

    fn selected_text_range(&mut self, _ignore_disabled_input: bool, _window: &mut Window, _cx: &mut Context<Self>) -> Option<UTF16Selection> {
        let selected = self.selected.clone().unwrap_or(self.caret..self.caret);
        Some(UTF16Selection { range: self.utf16_of(selected.start)..self.utf16_of(selected.end), reversed: false })
    }

    fn marked_text_range(&self, _window: &mut Window, _cx: &mut Context<Self>) -> Option<Range<usize>> {
        self.marked.as_ref().map(|marked| self.utf16_of(marked.start)..self.utf16_of(marked.end))
    }

    fn unmark_text(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {
        self.marked = None;
    }

    fn replace_text_in_range(&mut self, range: Option<Range<usize>>, text: &str, _window: &mut Window, cx: &mut Context<Self>) {
        if self.state == FieldState::Busy {
            return;
        }
        self.replace(self.target(range), text);
        self.edited(cx);
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        _selected: Option<Range<usize>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.state == FieldState::Busy {
            return;
        }
        let replaced = self.replace(self.target(range), text);
        self.marked = Some(replaced).filter(|marked| !marked.is_empty());
        self.edited(cx);
    }

    fn bounds_for_range(&mut self, _range: Range<usize>, element_bounds: Bounds<Pixels>, _window: &mut Window, _cx: &mut Context<Self>) -> Option<Bounds<Pixels>> {
        Some(self.bounds.get().unwrap_or(element_bounds))
    }

    fn character_index_for_point(&mut self, _point: Point<Pixels>, _window: &mut Window, _cx: &mut Context<Self>) -> Option<usize> {
        None
    }
}

impl Render for LineField {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let style = ViewStyle::drawing(cx).clone();
        let focused = self.focus_handle.is_focused(window);
        let (before, after) = self.text.split_at(self.caret);
        let caret = div().w(px(1.)).h(px(HEIGHT - 8.)).bg(rgb(style.text));
        let words = |text: &str| div().whitespace_nowrap().child(SharedString::from(text.to_owned()));
        let (focus, bounds, field) = (self.focus_handle.clone(), self.bounds.clone(), cx.entity());
        div()
            .id(self.id)
            .track_focus(&self.focus_handle)
            .relative()
            .flex()
            .flex_row()
            .items_center()
            .flex_1()
            .min_w(px(MIN_WIDTH))
            .h(px(HEIGHT))
            .px(px(4.))
            .overflow_hidden()
            .rounded(px(3.))
            .border(px(1.))
            .border_color(rgb(match (self.tone, focused) {
                (Tone::Error, _) => style.error,
                (Tone::Plain, true) => style.accent,
                (Tone::Plain, false) => style.border,
            }))
            .text_size(px(style.editor_notice_text_size))
            .text_color(rgb(if self.state == FieldState::Busy { style.muted } else { style.text }))
            .cursor(CursorStyle::IBeam)
            .on_key_down(cx.listener(Self::on_key_down))
            .on_mouse_down(MouseButton::Left, cx.listener(|field, _, window, cx| {
                window.focus(&field.focus_handle, cx);
                cx.stop_propagation();
            }))
            // a select-all puts the caret at the end, so all of it is before the caret
            .child(words(before).when(self.selected.is_some(), |selected| selected.bg(rgb(style.selected))))
            .when(focused, |field| field.child(caret))
            .when(self.text.is_empty(), |field| field.child(div().whitespace_nowrap().text_color(rgb(style.muted)).child(self.placeholder)))
            .child(words(after))
            .child(
                canvas(|_, _, _| {}, move |painted, (), window, cx| {
                    bounds.set(Some(painted));
                    window.handle_input(&focus, ElementInputHandler::new(painted, field.clone()), cx);
                })
                .absolute()
                .size_full(),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[gpui::test]
    fn should_edit_one_line_by_characters(cx: &mut gpui::TestAppContext) {
        // given
        let field = cx.new(|cx| LineField::new("Find", "find-query", cx));

        // when
        let text = field.update(cx, |field, cx| {
            field.replace(0..0, "fixé\nb");
            field.run(Command::Backspace(crate::text_editor::Unit::Char), cx);
            field.run(Command::Move(Motion::Left, crate::text_editor::Selecting::Move), cx);
            field.run(Command::Delete(crate::text_editor::Unit::Char), cx);
            (field.text().to_owned(), field.utf16_of(field.caret))
        });

        // then
        assert_eq!(text, ("fixé".to_owned(), 4), "the line break typed as a space, then removed");
    }

    /// The events `field` emits from now on.
    fn events_of(field: &gpui::Entity<LineField>, cx: &mut gpui::TestAppContext) -> Rc<std::cell::RefCell<Vec<String>>> {
        let events: Rc<std::cell::RefCell<Vec<String>>> = Rc::default();
        let seen = events.clone();
        cx.update(|cx| {
            cx.subscribe(field, move |_, event: &LineFieldEvent, _| {
                let name = match event {
                    LineFieldEvent::Submitted(toward) => format!("submitted {toward:?}"),
                    LineFieldEvent::Edited => "edited".to_owned(),
                    LineFieldEvent::Toggled(option) => format!("toggled {option:?}"),
                };
                seen.borrow_mut().push(name);
            })
            .detach();
        });
        events
    }

    #[test]
    fn should_leave_the_find_options_keys_to_others_in_a_plain_field() {
        // given
        let toggle = if cfg!(target_os = "macos") { "cmd-alt-w" } else { "alt-w" };

        // when
        let read = ["enter", toggle].map(|key| field_key(&Keystroke::parse(key).unwrap(), Keys::Plain));

        // then
        assert_eq!(read, [Some(FieldKey::Submit(Toward::Next)), None]);
    }

    #[gpui::test]
    fn should_replace_the_whole_text_typed_over_a_select_all(cx: &mut gpui::TestAppContext) {
        // given
        let field = cx.new(|cx| LineField::new("Find", "find-query", cx));
        field.update(cx, |field, cx| field.set_text("old", cx));
        let events = events_of(&field, cx);

        // when
        let text = field.update(cx, |field, cx| {
            let target = field.target(None);
            field.replace(target, "n");
            field.edited(cx);
            field.text().to_owned()
        });
        cx.run_until_parked();

        // then
        assert_eq!((text.as_str(), events.borrow().clone()), ("n", vec!["edited".to_owned()]));
    }

    #[gpui::test]
    fn should_drop_the_select_all_when_the_caret_moves(cx: &mut gpui::TestAppContext) {
        // given
        let field = cx.new(|cx| LineField::new("Find", "find-query", cx));
        field.update(cx, |field, cx| field.set_text("ab", cx));

        // when
        let text = field.update(cx, |field, cx| {
            field.run(Command::Move(Motion::Left, crate::text_editor::Selecting::Move), cx);
            let target = field.target(None);
            field.replace(target, "x");
            field.text().to_owned()
        });

        // then
        assert_eq!(text, "xab", "Left goes to the selection's start");
    }

    #[gpui::test]
    fn should_delete_a_selected_text_with_backspace(cx: &mut gpui::TestAppContext) {
        // given
        let field = cx.new(|cx| LineField::new("Find", "find-query", cx));
        field.update(cx, |field, cx| field.set_text("abc", cx));

        // when
        let text = field.update(cx, |field, cx| {
            field.run(Command::Backspace(crate::text_editor::Unit::Char), cx);
            field.text().to_owned()
        });

        // then
        assert_eq!(text, "");
    }

    #[test]
    fn should_read_enter_shift_enter_and_the_find_options_keys() {
        // given
        let toggle = |key: &str| if cfg!(target_os = "macos") { format!("cmd-alt-{key}") } else { format!("alt-{key}") };
        let keys = ["enter".to_owned(), "shift-enter".to_owned(), toggle("c"), toggle("w"), toggle("r"), "a".to_owned(), "ctrl-enter".to_owned()];

        // when
        let read: Vec<Option<FieldKey>> = keys.iter().map(|key| field_key(&Keystroke::parse(key).unwrap(), Keys::WithFindOptions)).collect();

        // then
        assert_eq!(read, [
            Some(FieldKey::Submit(Toward::Next)),
            Some(FieldKey::Submit(Toward::Previous)),
            Some(FieldKey::Toggle(FindOption::MatchCase)),
            Some(FieldKey::Toggle(FindOption::WholeWord)),
            Some(FieldKey::Toggle(FindOption::Regex)),
            None,
            None,
        ]);
    }
}
