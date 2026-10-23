//! Read-only text the mouse selects, like text on a web page: drag, double click a word, triple click the block,
//! Shift+click to extend. One selection in the window; the user's Copy and Select All act on it (`Place::Text`).

use gpui::{App, CursorStyle, FocusHandle, Global, HighlightStyle, MouseButton, MouseDownEvent, MouseMoveEvent, SharedString, Stateful, StyledText, Div, Window, div, prelude::*, rgb};

use crate::theme::Theme;

/// Byte offsets into a block's text, `start <= end`, both on char boundaries once [`clamped`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

/// The span from `anchor` to `head`, whichever comes first.
pub fn spanned(anchor: usize, head: usize) -> Span {
    Span { start: anchor.min(head), end: anchor.max(head) }
}

/// The run of word characters around `index` (paths and shas count as one word), or the char at it.
pub fn word_at(text: &str, index: usize) -> Span {
    let index = floor_boundary(text, index.min(text.len()));
    let start = text[..index].char_indices().rev().take_while(|(_, char)| is_word(*char)).last().map_or(index, |(at, _)| at);
    let end = text[index..].char_indices().find(|(_, char)| !is_word(*char)).map_or(text.len(), |(at, _)| index + at);
    if start < end {
        return Span { start, end };
    }
    let next = text[index..].chars().next().map_or(index, |char| index + char.len_utf8());
    Span { start: index, end: next }
}

/// The selected part of `text`; offsets inside a char move to its start.
pub fn selected_text(text: &str, span: Span) -> &str {
    let clamped = clamped(text, span);
    &text[clamped.start..clamped.end]
}

fn clamped(text: &str, span: Span) -> Span {
    Span { start: floor_boundary(text, span.start.min(text.len())), end: floor_boundary(text, span.end.min(text.len())) }
}

fn floor_boundary(text: &str, mut index: usize) -> usize {
    while !text.is_char_boundary(index) {
        index -= 1;
    }
    index
}

fn is_word(char: char) -> bool {
    char.is_alphanumeric() || matches!(char, '_' | '.' | '/' | '-')
}

/// Which block holds the selection, its whole text and the selected span.
#[derive(Debug, Clone, Default)]
pub struct Selection {
    id: SharedString,
    text: SharedString,
    span: Span,
}

impl Selection {
    pub fn select(&mut self, id: SharedString, text: SharedString, span: Span) {
        *self = Self { id, text, span };
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// What Copy takes; none when nothing is selected.
    pub fn copy_text(&self) -> Option<&str> {
        Some(selected_text(&self.text, self.span)).filter(|text| !text.is_empty())
    }

    pub fn select_all(&mut self) {
        self.span = Span { start: 0, end: self.text.len() };
    }

    /// The selected span when block `id` holds the selection.
    fn span_in(&self, id: &str) -> Option<Span> {
        Some(self.span).filter(|span| self.id == id && !span.is_empty())
    }

    fn anchor_in(&self, id: &str) -> Option<usize> {
        Some(self.span.start).filter(|_| self.id == id)
    }
}

/// The window's one text selection and the focus it takes, so the user's editor keys reach it.
pub struct TextSelection {
    pub selection: Selection,
    /// where the drag began, as the span's other end
    anchor: usize,
    pub focus: FocusHandle,
}

impl Global for TextSelection {}

impl TextSelection {
    pub fn new(cx: &mut App) -> Self {
        Self { selection: Selection::default(), anchor: 0, focus: cx.focus_handle() }
    }
}

/// Text in `id`'s block that the mouse selects; style it like any div (colour, size, font).
pub fn selectable(id: impl Into<SharedString>, text: impl Into<SharedString>, cx: &App) -> Stateful<Div> {
    let (id, text): (SharedString, SharedString) = (id.into(), text.into());
    let span = cx.try_global::<TextSelection>().and_then(|state| state.selection.span_in(&id));
    let background = rgb(cx.global::<Theme>().selected).into();
    let highlights = span.map(|span| clamped(&text, span)).map(|span| (span.start..span.end, HighlightStyle { background_color: Some(background), ..Default::default() }));
    let styled = StyledText::new(text.clone()).with_highlights(highlights);
    let layout = styled.layout().clone();
    let (down_id, down_text, down_layout) = (id.clone(), text.clone(), layout.clone());
    let move_id = id.clone();
    let out_id = id.clone();
    div()
        .id(id)
        .cursor(CursorStyle::IBeam)
        .child(styled)
        .on_mouse_down(MouseButton::Left, move |event: &MouseDownEvent, window: &mut Window, cx: &mut App| {
            let index = down_layout.index_for_position(event.position).unwrap_or_else(|index| index);
            press(Press { id: &down_id, text: &down_text, index, event }, window, cx);
        })
        .on_mouse_move(move |event: &MouseMoveEvent, window: &mut Window, cx: &mut App| {
            if event.pressed_button != Some(MouseButton::Left) {
                return;
            }
            let index = layout.index_for_position(event.position).unwrap_or_else(|index| index);
            drag(&move_id, index, window, cx);
        })
        .on_mouse_down_out(move |_event, window: &mut Window, cx: &mut App| {
            let holds = cx.try_global::<TextSelection>().is_some_and(|state| state.selection.id == out_id);
            if holds {
                cx.global_mut::<TextSelection>().selection.clear();
                window.refresh();
            }
        })
}

/// A left press in a block: where, and with which clicks and keys.
struct Press<'a> {
    id: &'a SharedString,
    text: &'a SharedString,
    index: usize,
    event: &'a MouseDownEvent,
}

fn press(press: Press, window: &mut Window, cx: &mut App) {
    let Press { id, text, index, event } = press;
    let state = cx.global_mut::<TextSelection>();
    let extends = event.modifiers.shift && state.selection.anchor_in(id).is_some();
    let (anchor, span) = match event.click_count {
        2 => (index, word_at(text, index)),
        count if count >= 3 => (0, Span { start: 0, end: text.len() }),
        _ if extends => (state.anchor, spanned(state.anchor, index)),
        _ => (index, spanned(index, index)),
    };
    state.anchor = anchor;
    state.selection.select(id.clone(), text.clone(), span);
    let focus = state.focus.clone();
    window.focus(&focus, cx);
    window.refresh();
}

fn drag(id: &SharedString, index: usize, window: &mut Window, cx: &mut App) {
    let state = cx.global_mut::<TextSelection>();
    if state.selection.anchor_in(id).is_none() {
        return;
    }
    let span = spanned(state.anchor, index);
    let (text, id) = (state.selection.text.clone(), state.selection.id.clone());
    state.selection.select(id, text, span);
    window.refresh();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_select_word_around_index() {
        // given
        let text = "cannot read /a/b.rs: gone";

        // when
        let span = word_at(text, 14);

        // then
        assert_eq!(selected_text(text, span), "/a/b.rs");
    }

    #[test]
    fn should_find_word_in_multibyte_text() {
        // given
        let text = "zażółć gęślą";

        // when
        let span = word_at(text, 3);

        // then
        assert_eq!(selected_text(text, span), "zażółć");
    }

    #[test]
    fn should_order_span_when_dragging_backwards() {
        // given
        let (anchor, head) = (9, 2);

        // when
        let span = spanned(anchor, head);

        // then
        assert_eq!((span.start, span.end), (2, 9));
    }

    #[test]
    fn should_not_select_on_click_without_drag() {
        // given
        let mut selection = Selection::default();

        // when
        selection.select("a".into(), "some text".into(), spanned(4, 4));

        // then
        assert_eq!(selection.copy_text(), None);
    }

    #[test]
    fn should_copy_selected_part() {
        // given
        let mut selection = Selection::default();

        // when
        selection.select("a".into(), "exit status: 1".into(), spanned(5, 11));

        // then
        assert_eq!(selection.copy_text(), Some("status"));
    }

    #[test]
    fn should_select_whole_block() {
        // given
        let mut selection = Selection::default();
        selection.select("a".into(), "exit status: 1".into(), spanned(5, 11));

        // when
        selection.select_all();

        // then
        assert_eq!(selection.copy_text(), Some("exit status: 1"));
    }

    #[test]
    fn should_clamp_index_to_char_boundary() {
        // given
        let text = "zażółć";

        // when
        let span = spanned(1, 3);

        // then
        assert_eq!(selected_text(text, span), "a");
    }
}
