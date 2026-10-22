//! The one-line field of a change's popup where the commit message is typed. It has the keyboard while focused: the
//! editor around it types nothing then.

use std::cell::Cell;
use std::ops::Range;
use std::rc::Rc;

use gpui::{
    App, Bounds, Context, CursorStyle, ElementInputHandler, EntityInputHandler, EventEmitter, FocusHandle, Focusable,
    KeyDownEvent, MouseButton, Pixels, Point, SharedString, UTF16Selection, Window, canvas, div, prelude::*, px, rgb,
};

use super::keys::command;
use super::model::Motion;
use crate::host::{ClipboardTarget, Command, Host, KeyOutcome, KeyScope};
use crate::style::ViewStyle;

const PLACEHOLDER: &str = "Commit this change";
const HEIGHT: f32 = 22.;
const MIN_WIDTH: f32 = 120.;

pub(crate) enum MessageFieldEvent {
    /// Enter: commit with the message
    Submitted,
}

/// Whether the field takes typing; it does not while its message is being committed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FieldState {
    Editable,
    Busy,
}

pub(crate) struct MessageField {
    text: String,
    /// where the caret is, in bytes of `text`, on a character boundary
    caret: usize,
    /// the text the input method is composing, in bytes of `text`
    marked: Option<Range<usize>>,
    state: FieldState,
    focus_handle: FocusHandle,
    /// where the field was last painted, for the input method
    bounds: Rc<Cell<Option<Bounds<Pixels>>>>,
}

impl EventEmitter<MessageFieldEvent> for MessageField {}

impl Focusable for MessageField {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl MessageField {
    pub(crate) fn new(cx: &mut Context<Self>) -> Self {
        Self { text: String::new(), caret: 0, marked: None, state: FieldState::Editable, focus_handle: cx.focus_handle(), bounds: Rc::default() }
    }

    pub(crate) fn text(&self) -> &str {
        &self.text
    }

    pub(crate) fn clear(&mut self, cx: &mut Context<Self>) {
        self.text.clear();
        self.caret = 0;
        self.marked = None;
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
        range.start..self.caret
    }

    fn on_key_down(&mut self, event: &KeyDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        match Host::key_outcome(event, KeyScope::Editor, cx) {
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
            Command::Newline => cx.emit(MessageFieldEvent::Submitted),
            Command::Move(motion, _) => self.caret = self.moved(motion),
            Command::Copy(target) => Host::write_clipboard(target, self.text.clone(), cx),
            _ if self.state == FieldState::Busy => {}
            Command::Backspace(_) => drop(self.replace(self.before(self.caret)..self.caret, "")),
            Command::Delete(_) => drop(self.replace(self.caret..self.after(self.caret), "")),
            Command::Paste(source) => self.paste(source, cx),
            Command::Cut => {
                Host::write_clipboard(ClipboardTarget::Clipboard, std::mem::take(&mut self.text), cx);
                self.caret = 0;
            }
            Command::SelectAll | Command::Undo | Command::Redo | Command::Indent | Command::Flush | Command::ToggleLayout => {}
        }
        cx.notify();
    }

    fn paste(&mut self, source: ClipboardTarget, cx: &mut Context<Self>) {
        if let Some(text) = Host::read_clipboard(source, cx) {
            self.replace(self.caret..self.caret, &text);
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
        range.map(|range| self.bytes_of(range)).or(self.marked.clone()).unwrap_or(self.caret..self.caret)
    }
}

impl EntityInputHandler for MessageField {
    fn text_for_range(&mut self, range: Range<usize>, adjusted: &mut Option<Range<usize>>, _window: &mut Window, _cx: &mut Context<Self>) -> Option<String> {
        let bytes = self.bytes_of(range);
        adjusted.replace(self.utf16_of(bytes.start)..self.utf16_of(bytes.end));
        Some(self.text[bytes].to_owned())
    }

    fn selected_text_range(&mut self, _ignore_disabled_input: bool, _window: &mut Window, _cx: &mut Context<Self>) -> Option<UTF16Selection> {
        let caret = self.utf16_of(self.caret);
        Some(UTF16Selection { range: caret..caret, reversed: false })
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
        cx.notify();
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
        cx.notify();
    }

    fn bounds_for_range(&mut self, _range: Range<usize>, element_bounds: Bounds<Pixels>, _window: &mut Window, _cx: &mut Context<Self>) -> Option<Bounds<Pixels>> {
        Some(self.bounds.get().unwrap_or(element_bounds))
    }

    fn character_index_for_point(&mut self, _point: Point<Pixels>, _window: &mut Window, _cx: &mut Context<Self>) -> Option<usize> {
        None
    }
}

impl Render for MessageField {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let style = ViewStyle::drawing(cx).clone();
        let focused = self.focus_handle.is_focused(window);
        let (before, after) = self.text.split_at(self.caret);
        let caret = div().w(px(1.)).h(px(HEIGHT - 8.)).bg(rgb(style.text));
        let words = |text: &str| div().whitespace_nowrap().child(SharedString::from(text.to_owned()));
        let (focus, bounds, field) = (self.focus_handle.clone(), self.bounds.clone(), cx.entity());
        div()
            .id("change-popup-message")
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
            .border_color(rgb(if focused { style.accent } else { style.border }))
            .text_size(px(style.editor_notice_text_size))
            .text_color(rgb(if self.state == FieldState::Busy { style.muted } else { style.text }))
            .cursor(CursorStyle::IBeam)
            .on_key_down(cx.listener(Self::on_key_down))
            .on_mouse_down(MouseButton::Left, cx.listener(|field, _, window, cx| {
                window.focus(&field.focus_handle, cx);
                cx.stop_propagation();
            }))
            .child(words(before))
            .when(focused, |field| field.child(caret))
            .when(self.text.is_empty(), |field| field.child(div().whitespace_nowrap().text_color(rgb(style.muted)).child(PLACEHOLDER)))
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
        let field = cx.new(MessageField::new);

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
}
