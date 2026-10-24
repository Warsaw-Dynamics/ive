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

    fn holds(&self, id: &str) -> bool {
        self.id == id
    }
}

/// The window's one text selection and the focus it takes, so the user's editor keys reach it.
pub struct TextSelection {
    pub selection: Selection,
    /// where the drag began, as the span's other end
    anchor: usize,
    pub focus: FocusHandle,
    /// what had the keyboard before the selection took it, to give it back: keys only reach views on the focused
    /// element's path, so a focus nothing draws would leave every key unheard
    previous: Option<FocusHandle>,
}

impl Global for TextSelection {}

impl TextSelection {
    pub fn new(cx: &mut App) -> Self {
        Self { selection: Selection::default(), anchor: 0, focus: cx.focus_handle(), previous: None }
    }
}

/// Clears the selection and gives the keyboard back to what had it before, when the selection still has it; for
/// when its text is about to go (another tab shown).
pub fn let_go(window: &mut Window, cx: &mut App) {
    let Some(state) = cx.try_global::<TextSelection>() else {
        return;
    };
    let had_keys = state.focus.is_focused(window);
    let state = cx.global_mut::<TextSelection>();
    state.selection.clear();
    let previous = state.previous.take();
    if had_keys {
        match previous {
            Some(previous) => window.focus(&previous, cx),
            None => window.blur(cx),
        }
    }
    window.refresh();
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
    // the block holding the selection holds the focus too, so keys pass through the views around it (the user's Copy)
    let focus = cx.try_global::<TextSelection>().filter(|state| state.selection.holds(&id)).map(|state| state.focus.clone());
    div()
        .id(id)
        .when_some(focus, |block, focus| block.track_focus(&focus))
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
            let holds = cx.try_global::<TextSelection>().is_some_and(|state| state.selection.holds(&out_id));
            if holds {
                let_go(window, cx);
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
    let focused = window.focused(cx);
    let state = cx.global_mut::<TextSelection>();
    if focused.as_ref() != Some(&state.focus) {
        state.previous = focused;
    }
    let extends = event.modifiers.shift && state.selection.holds(id);
    let (anchor, span) = match event.click_count {
        2 => {
            let word = word_at(text, index);
            (word.start, word)
        }
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

/// Extends block `id`'s selection to `index`; a drag that began in another block leaves it alone.
fn drag(id: &SharedString, index: usize, window: &mut Window, cx: &mut App) {
    let state = cx.global_mut::<TextSelection>();
    if state.selection.holds(id) {
        let span = spanned(state.anchor, index);
        state.selection.span = span;
        window.refresh();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use gpui::{Modifiers, MouseUpEvent, Point, TestAppContext, VisualTestContext, point, px};

    use crate::theme::Appearance;

    const SHOWN: &str = "hello world, read /a/b.rs";

    /// `SHOWN` as selectable block `t` above an empty area, inside a panel that counts the keys reaching it, as the
    /// workspace's key handler sits inside the window's frame.
    struct Shown {
        keys: std::rc::Rc<std::cell::Cell<usize>>,
    }

    impl Render for Shown {
        fn render(&mut self, _window: &mut Window, cx: &mut gpui::Context<Self>) -> impl IntoElement {
            let keys = self.keys.clone();
            let panel = div().size_full().on_key_down(move |_event, _window, _cx| keys.set(keys.get() + 1)).child(selectable("t", SHOWN, cx)).child(div().id("below").h(px(100.)));
            div().size_full().child(panel)
        }
    }

    /// A window showing [`Shown`], with the theme and selection set up, and the count of keys its panel saw.
    fn window(cx: &mut TestAppContext) -> (&mut VisualTestContext, std::rc::Rc<std::cell::Cell<usize>>) {
        cx.update(|cx| {
            cx.set_global(Theme::of(Appearance::Dark));
            let selection = TextSelection::new(cx);
            cx.set_global(selection);
        });
        let keys = std::rc::Rc::new(std::cell::Cell::new(0));
        let seen = keys.clone();
        let (_, cx) = cx.add_window_view(|_window, _cx| Shown { keys: seen });
        redraw(cx);
        (cx, keys)
    }

    #[gpui::test]
    fn should_give_focus_back_when_selection_clears(cx: &mut TestAppContext) {
        // given
        let (cx, keys) = window(cx);
        let before = cx.update(|window, cx| {
            let handle = cx.focus_handle();
            window.focus(&handle, cx);
            handle
        });
        press(point(px(5.), px(5.)), 2, cx);
        redraw(cx);

        // when
        press(point(px(5.), px(150.)), 1, cx);
        redraw(cx);

        // then
        assert!(cx.update(|window, _cx| before.is_focused(window)));
        assert_eq!(keys.get(), 0);
    }

    #[gpui::test]
    fn should_give_focus_back_on_release(cx: &mut TestAppContext) {
        // given
        let (cx, _) = window(cx);
        let before = cx.update(|window, cx| {
            let handle = cx.focus_handle();
            window.focus(&handle, cx);
            handle
        });
        press(point(px(5.), px(5.)), 2, cx);
        redraw(cx);

        // when
        cx.update(let_go);

        // then
        assert!(cx.update(|window, _cx| before.is_focused(window)));
        assert_eq!(copied(cx), None);
    }

    #[gpui::test]
    fn should_pass_keys_to_the_panel_around_selected_text(cx: &mut TestAppContext) {
        // given
        let (cx, keys) = window(cx);
        press(point(px(5.), px(5.)), 2, cx);
        redraw(cx);

        // when
        cx.simulate_keystrokes("ctrl-c");

        // then
        assert_eq!(keys.get(), 1);
    }

    fn redraw(cx: &mut VisualTestContext) {
        cx.update(|window, _cx| window.refresh());
        cx.run_until_parked();
    }

    fn press(at: Point<gpui::Pixels>, click_count: usize, cx: &mut VisualTestContext) {
        cx.simulate_event(MouseDownEvent { button: MouseButton::Left, position: at, modifiers: Modifiers::default(), click_count, first_mouse: false });
    }

    fn release(at: Point<gpui::Pixels>, cx: &mut VisualTestContext) {
        cx.simulate_event(MouseUpEvent { button: MouseButton::Left, position: at, modifiers: Modifiers::default(), click_count: 1 });
    }

    fn copied(cx: &mut VisualTestContext) -> Option<String> {
        cx.read(|cx| cx.global::<TextSelection>().selection.copy_text().map(str::to_owned))
    }

    #[gpui::test]
    fn should_select_text_dragged_over(cx: &mut TestAppContext) {
        // given
        let (cx, _) = window(cx);

        // when
        press(point(px(1.), px(5.)), 1, cx);
        cx.simulate_event(MouseMoveEvent { position: point(px(60.), px(5.)), pressed_button: Some(MouseButton::Left), modifiers: Modifiers::default() });
        release(point(px(60.), px(5.)), cx);

        // then
        let text = copied(cx).expect("a selection");
        assert!(SHOWN.starts_with(&text) && text.len() > 3, "{text:?}");
    }

    #[gpui::test]
    fn should_select_word_on_double_click(cx: &mut TestAppContext) {
        // given
        let (cx, _) = window(cx);

        // when
        press(point(px(5.), px(5.)), 2, cx);

        // then
        assert_eq!(copied(cx).as_deref(), Some("hello"));
    }

    #[gpui::test]
    fn should_extend_from_the_word_start_after_double_click(cx: &mut TestAppContext) {
        // given
        let (cx, _) = window(cx);
        press(point(px(20.), px(5.)), 2, cx);
        redraw(cx);

        // when
        let shift = Modifiers { shift: true, ..Modifiers::default() };
        cx.simulate_event(MouseDownEvent { button: MouseButton::Left, position: point(px(120.), px(5.)), modifiers: shift, click_count: 1, first_mouse: false });

        // then
        let text = copied(cx).expect("a selection");
        assert!(text.starts_with("hello ") && text.len() > "hello ".len(), "{text:?}");
    }

    #[gpui::test]
    fn should_clear_selection_on_press_elsewhere(cx: &mut TestAppContext) {
        // given
        let (cx, _) = window(cx);
        press(point(px(5.), px(5.)), 3, cx);
        redraw(cx);

        // when
        press(point(px(5.), px(150.)), 1, cx);

        // then
        assert_eq!(copied(cx), None);
    }

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
