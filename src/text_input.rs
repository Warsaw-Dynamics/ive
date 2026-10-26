//! A single-line text field; the editing rules live in [`model`], without gpui.
//! Built like gpui's `examples/input.rs`, so the platform input handler (IME, dead keys) reaches it.

pub mod model;

use std::ops::Range;

use gpui::{
    App, Bounds, ClickEvent, ClipboardItem, ContentMask, Context, CursorStyle, Element, ElementId, ElementInputHandler, Entity,
    EntityInputHandler, EventEmitter, FocusHandle, Focusable, GlobalElementId, KeyBinding, Keystroke, LayoutId, MouseButton,
    MouseDownEvent, MouseMoveEvent, MouseUpEvent, PaintQuad, Pixels, Point, ShapedLine, SharedString, Style,
    TextRun, UTF16Selection, UnderlineStyle, Window, actions, div, fill, point, prelude::*, relative, rgb, rgba,
    size,
};

use crate::click_targets::target;
use crate::theme::Theme;
use model::{EditModel, Movement, Unit};

actions!(
    text_input,
    [
        Backspace, Delete, DeleteWordLeft, DeleteWordRight, Left, Right, SelectLeft, SelectRight, Home, End,
        SelectHome, SelectEnd, SelectAll, Copy, Cut, Paste, Submit,
    ]
);

const CONTEXT: &str = "TextInput";
/// What a masked field draws for each character of a secret.
const MASK: char = '•';

/// Whether a text field has the keyboard in `window`.
pub fn holds_focus(window: &Window) -> bool {
    window.context_stack().iter().any(|context| context.contains(CONTEXT))
}

/// Whether a key pressed in a text field goes on to the window's bindings: it types no text (ctrl, alt, cmd or fn
/// held, or a function key) and the field does not take it. Typing a name never triggers a binding.
pub fn passes_to_window(keystroke: &Keystroke) -> bool {
    let modifiers = &keystroke.modifiers;
    let function_key = keystroke.key.strip_prefix('f').is_some_and(|number| number.parse::<u8>().is_ok());
    let types_text = !(modifiers.control || modifiers.alt || modifiers.platform || modifiers.function || function_key);
    !types_text && !field_bindings().iter().any(|binding| binding.match_keystrokes(std::slice::from_ref(keystroke)) == Some(false))
}

/// Whether a field keeps a key even where a binding of its own place (the hub's) names it: the keys that type text
/// and the field's editing keys. Enter is not kept: the place decides what it does.
pub fn keeps_key(keystroke: &Keystroke) -> bool {
    let modifiers = &keystroke.modifiers;
    let plain = !(modifiers.control || modifiers.alt || modifiers.platform || modifiers.function);
    let one_character = keystroke.key.chars().count() == 1 || keystroke.key == "space";
    let types_text = plain && (one_character || keystroke.key_char.is_some()) && !matches!(keystroke.key.as_str(), "enter" | "tab" | "escape");
    let edits = field_bindings().iter().any(|binding| binding.match_keystrokes(std::slice::from_ref(keystroke)) == Some(false) && !binding.action().partial_eq(&Submit));
    types_text || edits
}

/// Escape is not bound: it reaches the parent, so menus close.
pub fn bind_keys(cx: &mut App) {
    cx.bind_keys(field_bindings());
}

fn field_bindings() -> [KeyBinding; 17] {
    let context = Some(CONTEXT);
    [
        KeyBinding::new("backspace", Backspace, context),
        KeyBinding::new("delete", Delete, context),
        KeyBinding::new("ctrl-backspace", DeleteWordLeft, context),
        KeyBinding::new("ctrl-delete", DeleteWordRight, context),
        KeyBinding::new("left", Left, context),
        KeyBinding::new("right", Right, context),
        KeyBinding::new("shift-left", SelectLeft, context),
        KeyBinding::new("shift-right", SelectRight, context),
        KeyBinding::new("home", Home, context),
        KeyBinding::new("end", End, context),
        KeyBinding::new("shift-home", SelectHome, context),
        KeyBinding::new("shift-end", SelectEnd, context),
        KeyBinding::new("secondary-a", SelectAll, context),
        KeyBinding::new("secondary-c", Copy, context),
        KeyBinding::new("secondary-x", Cut, context),
        KeyBinding::new("secondary-v", Paste, context),
        KeyBinding::new("enter", Submit, context),
    ]
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextInputEvent {
    Changed(String),
    Submitted,
}

pub struct TextInput {
    focus_handle: FocusHandle,
    model: EditModel,
    placeholder: SharedString,
    disabled: bool,
    last_layout: Option<ShapedLine>,
    last_bounds: Option<Bounds<Pixels>>,
    selecting: bool,
    /// the click target that focuses the field, for MCP
    target: Option<String>,
    /// a secret: drawn as `•`, never copied
    masked: bool,
}

impl EventEmitter<TextInputEvent> for TextInput {}

impl Focusable for TextInput {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl TextInput {
    pub fn new(cx: &mut Context<TextInput>) -> TextInput {
        Self {
            focus_handle: cx.focus_handle(),
            model: EditModel::default(),
            placeholder: SharedString::default(),
            disabled: false,
            last_layout: None,
            last_bounds: None,
            selecting: false,
            target: None,
            masked: false,
        }
    }

    /// Draws the text as `•` and refuses Copy and Cut, for secrets.
    pub fn set_masked(&mut self, masked: bool) {
        self.masked = masked;
    }

    /// Byte offset `offset` of the text in what is drawn.
    fn shown_offset(&self, offset: usize) -> usize {
        if self.masked { masked_offset(self.model.text(), offset) } else { offset }
    }

    pub fn placeholder(&mut self, text: impl Into<SharedString>) {
        self.placeholder = text.into();
    }

    /// Replaces the text, caret at the end; no `Changed` event, since the program did it.
    pub fn set_text(&mut self, text: impl Into<String>, cx: &mut Context<Self>) {
        self.model.set_text(&text.into());
        cx.notify();
    }

    pub fn text(&self) -> &str {
        self.model.text()
    }

    /// Byte offsets into [`Self::text`], clamped to it.
    pub fn select_range(&mut self, range: Range<usize>) {
        self.model.select_range(range);
    }

    fn is_editable(&self) -> bool {
        !self.disabled
    }

    /// While drawn and enabled, the field is click target `name`, which focuses it (MCP `click`).
    pub fn set_target(&mut self, name: impl Into<String>) {
        self.target = Some(name.into());
    }

    fn edited(&mut self, changed: bool, cx: &mut Context<Self>) {
        if changed {
            cx.emit(TextInputEvent::Changed(self.model.text().to_owned()));
        }
        cx.notify();
    }

    fn moved(&mut self, cx: &mut Context<Self>) {
        cx.notify();
    }

    fn backspace(&mut self, _: &Backspace, _: &mut Window, cx: &mut Context<Self>) {
        let changed = self.model.delete_backward(Unit::Char);
        self.edited(changed, cx);
    }

    fn delete(&mut self, _: &Delete, _: &mut Window, cx: &mut Context<Self>) {
        let changed = self.model.delete_forward(Unit::Char);
        self.edited(changed, cx);
    }

    fn delete_word_left(&mut self, _: &DeleteWordLeft, _: &mut Window, cx: &mut Context<Self>) {
        let changed = self.model.delete_backward(Unit::Word);
        self.edited(changed, cx);
    }

    fn delete_word_right(&mut self, _: &DeleteWordRight, _: &mut Window, cx: &mut Context<Self>) {
        let changed = self.model.delete_forward(Unit::Word);
        self.edited(changed, cx);
    }

    fn left(&mut self, _: &Left, _: &mut Window, cx: &mut Context<Self>) {
        self.model.left(Movement::Move);
        self.moved(cx);
    }

    fn right(&mut self, _: &Right, _: &mut Window, cx: &mut Context<Self>) {
        self.model.right(Movement::Move);
        self.moved(cx);
    }

    fn select_left(&mut self, _: &SelectLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.model.left(Movement::Select);
        self.moved(cx);
    }

    fn select_right(&mut self, _: &SelectRight, _: &mut Window, cx: &mut Context<Self>) {
        self.model.right(Movement::Select);
        self.moved(cx);
    }

    fn home(&mut self, _: &Home, _: &mut Window, cx: &mut Context<Self>) {
        self.model.home(Movement::Move);
        self.moved(cx);
    }

    fn end(&mut self, _: &End, _: &mut Window, cx: &mut Context<Self>) {
        self.model.end(Movement::Move);
        self.moved(cx);
    }

    fn select_home(&mut self, _: &SelectHome, _: &mut Window, cx: &mut Context<Self>) {
        self.model.home(Movement::Select);
        self.moved(cx);
    }

    fn select_end(&mut self, _: &SelectEnd, _: &mut Window, cx: &mut Context<Self>) {
        self.model.end(Movement::Select);
        self.moved(cx);
    }

    fn select_every(&mut self, _: &SelectAll, _: &mut Window, cx: &mut Context<Self>) {
        self.model.select_all();
        self.moved(cx);
    }

    fn copy(&mut self, _: &Copy, _: &mut Window, cx: &mut Context<Self>) {
        if self.masked {
            return;
        }
        if let Some(selected) = self.model.selected_text() {
            cx.write_to_clipboard(ClipboardItem::new_string(selected.to_owned()));
        }
    }

    fn cut(&mut self, _: &Cut, _: &mut Window, cx: &mut Context<Self>) {
        if self.masked {
            return;
        }
        if let Some(selected) = self.model.selected_text() {
            cx.write_to_clipboard(ClipboardItem::new_string(selected.to_owned()));
            let changed = self.model.replace(None, "");
            self.edited(changed, cx);
        }
    }

    fn paste(&mut self, _: &Paste, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
            let changed = self.model.replace(None, &text);
            self.edited(changed, cx);
        }
    }

    fn submit(&mut self, _: &Submit, _: &mut Window, cx: &mut Context<Self>) {
        cx.emit(TextInputEvent::Submitted);
    }

    fn on_mouse_down(&mut self, event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.focus_handle, cx);
        self.selecting = true;
        let index = self.index_for_position(event.position);
        if event.modifiers.shift {
            self.model.select_to(index);
        } else {
            self.model.move_to(index);
        }
        self.moved(cx);
    }

    fn on_mouse_up(&mut self, _: &MouseUpEvent, _: &mut Window, _: &mut Context<Self>) {
        self.selecting = false;
    }

    fn on_mouse_move(&mut self, event: &MouseMoveEvent, _: &mut Window, cx: &mut Context<Self>) {
        if self.selecting {
            self.model.select_to(self.index_for_position(event.position));
            self.moved(cx);
        }
    }

    fn index_for_position(&self, position: Point<Pixels>) -> usize {
        let (Some(bounds), Some(line)) = (self.last_bounds.as_ref(), self.last_layout.as_ref()) else {
            return 0;
        };
        if self.model.text().is_empty() || position.y < bounds.top() {
            return 0;
        }
        if position.y > bounds.bottom() {
            return self.model.text().len();
        }
        let index = line.closest_index_for_x(position.x - bounds.left());
        if self.masked { unmasked_offset(self.model.text(), index) } else { index }
    }
}

impl EntityInputHandler for TextInput {
    fn text_for_range(&mut self, range_utf16: Range<usize>, actual_range: &mut Option<Range<usize>>, _: &mut Window, _: &mut Context<Self>) -> Option<String> {
        let range = self.model.range_of_utf16(&range_utf16);
        actual_range.replace(self.model.utf16_range(&range));
        Some(self.model.text()[range].to_owned())
    }

    fn selected_text_range(&mut self, _ignore_disabled_input: bool, _: &mut Window, _: &mut Context<Self>) -> Option<UTF16Selection> {
        let selection = self.model.selection();
        Some(UTF16Selection { range: self.model.utf16_range(&selection), reversed: self.model.cursor() == selection.start && !selection.is_empty() })
    }

    fn marked_text_range(&self, _: &mut Window, _: &mut Context<Self>) -> Option<Range<usize>> {
        self.model.marked().map(|range| self.model.utf16_range(&range))
    }

    fn unmark_text(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        self.model.unmark();
        cx.notify();
    }

    fn replace_text_in_range(&mut self, range_utf16: Option<Range<usize>>, text: &str, _: &mut Window, cx: &mut Context<Self>) {
        let range = range_utf16.map(|range| self.model.range_of_utf16(&range));
        let changed = self.model.replace(range, text);
        self.edited(changed, cx);
    }

    fn replace_and_mark_text_in_range(&mut self, range_utf16: Option<Range<usize>>, text: &str, selected_utf16: Option<Range<usize>>, _: &mut Window, cx: &mut Context<Self>) {
        let range = range_utf16.map(|range| self.model.range_of_utf16(&range));
        // the selection is relative to the inserted text, so it converts against that text
        let selected = selected_utf16.map(|selected| utf16_range_in(text, &selected));
        self.model.replace_and_mark(range, text, selected);
        self.edited(true, cx);
    }

    fn bounds_for_range(&mut self, range_utf16: Range<usize>, bounds: Bounds<Pixels>, _: &mut Window, _: &mut Context<Self>) -> Option<Bounds<Pixels>> {
        let layout = self.last_layout.as_ref()?;
        let range = self.model.range_of_utf16(&range_utf16);
        Some(Bounds::from_corners(
            point(bounds.left() + layout.x_for_index(self.shown_offset(range.start)), bounds.top()),
            point(bounds.left() + layout.x_for_index(self.shown_offset(range.end)), bounds.bottom()),
        ))
    }

    fn character_index_for_point(&mut self, point: Point<Pixels>, _: &mut Window, _: &mut Context<Self>) -> Option<usize> {
        let local = self.last_bounds?.localize(&point)?;
        let index = self.last_layout.as_ref()?.index_for_x(local.x)?;
        Some(self.model.utf16_of(index))
    }
}

/// What a masked field draws for `text`: one `•` per character.
fn masked(text: &str) -> String {
    MASK.to_string().repeat(text.chars().count())
}

/// Byte `offset` of `text` in its mask.
fn masked_offset(text: &str, offset: usize) -> usize {
    text[..offset.min(text.len())].chars().count() * MASK.len_utf8()
}

/// Byte `offset` of a mask in the `text` it hides.
fn unmasked_offset(text: &str, offset: usize) -> usize {
    let characters = offset / MASK.len_utf8();
    text.char_indices().nth(characters).map_or(text.len(), |(index, _)| index)
}

/// A UTF-16 range inside `text` as byte offsets.
fn utf16_range_in(text: &str, range: &Range<usize>) -> Range<usize> {
    let mut model = EditModel::default();
    model.set_text(text);
    model.range_of_utf16(range)
}

impl Render for TextInput {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let focused = self.focus_handle.is_focused(window);
        let field = div()
            .id(("text-input", cx.entity_id().as_u64()))
            .flex()
            .items_center()
            .w_full()
            .h(theme.sizes.controls.input_height.px())
            .px(theme.sizes.controls.input_padding_x.px())
            .rounded(theme.sizes.controls.input_radius.px())
            .border(theme.sizes.controls.input_border.px())
            .border_color(rgb(if focused { theme.accent } else { theme.border }))
            .bg(rgb(theme.background))
            .text_size(theme.sizes.controls.input_text_size.px())
            .child(TextElement { input: cx.entity() });
        if self.disabled {
            return field.opacity(theme.sizes.controls.input_disabled_opacity.0);
        }
        let focus = self.target.clone().map(|name| {
            target(name, cx.listener(|input: &mut Self, _: &ClickEvent, window, cx| {
                window.focus(&input.focus_handle, cx);
                cx.notify();
            }), cx)
        });
        field
            .when_some(focus, |field, focus| field.on_click(focus))
            .key_context(CONTEXT)
            .track_focus(&self.focus_handle)
            .cursor(CursorStyle::IBeam)
            .on_action(cx.listener(Self::backspace))
            .on_action(cx.listener(Self::delete))
            .on_action(cx.listener(Self::delete_word_left))
            .on_action(cx.listener(Self::delete_word_right))
            .on_action(cx.listener(Self::left))
            .on_action(cx.listener(Self::right))
            .on_action(cx.listener(Self::select_left))
            .on_action(cx.listener(Self::select_right))
            .on_action(cx.listener(Self::home))
            .on_action(cx.listener(Self::end))
            .on_action(cx.listener(Self::select_home))
            .on_action(cx.listener(Self::select_end))
            .on_action(cx.listener(Self::select_every))
            .on_action(cx.listener(Self::copy))
            .on_action(cx.listener(Self::cut))
            .on_action(cx.listener(Self::paste))
            .on_action(cx.listener(Self::submit))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_move(cx.listener(Self::on_mouse_move))
    }
}

/// Shapes and paints the line, its selection and caret, and hands the platform input handler to the entity.
struct TextElement {
    input: Entity<TextInput>,
}

struct PrepaintState {
    line: Option<ShapedLine>,
    caret: Option<PaintQuad>,
    selection: Option<PaintQuad>,
}

impl IntoElement for TextElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for TextElement {
    type RequestLayoutState = ();
    type PrepaintState = PrepaintState;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(&mut self, _: Option<&GlobalElementId>, _: Option<&gpui::InspectorElementId>, window: &mut Window, cx: &mut App) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = window.line_height().into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(&mut self, _: Option<&GlobalElementId>, _: Option<&gpui::InspectorElementId>, bounds: Bounds<Pixels>, _: &mut Self::RequestLayoutState, window: &mut Window, cx: &mut App) -> Self::PrepaintState {
        let theme = *cx.global::<Theme>();
        let input = self.input.read(cx);
        let style = window.text_style();
        let (shown, color) = match input.model.text().is_empty() {
            true => (input.placeholder.clone(), rgb(theme.muted).into()),
            false if input.masked => (SharedString::from(masked(input.model.text())), style.color),
            false => (SharedString::from(input.model.text().to_owned()), style.color),
        };
        let run = TextRun { len: shown.len(), font: style.font(), color, background_color: None, underline: None, strikethrough: None };
        let runs = match input.model.marked().filter(|_| !input.model.text().is_empty() && !input.masked) {
            Some(marked) => vec![
                TextRun { len: marked.start, ..run.clone() },
                TextRun { len: marked.end - marked.start, underline: Some(UnderlineStyle { color: Some(run.color), thickness: theme.sizes.controls.input_marked_underline.px(), wavy: false }), ..run.clone() },
                TextRun { len: shown.len() - marked.end, ..run },
            ]
            .into_iter()
            .filter(|run| run.len > 0)
            .collect(),
            None => vec![run],
        };
        let font_size = style.font_size.to_pixels(window.rem_size());
        let line = window.text_system().shape_line(shown, font_size, &runs, None);
        let selection = input.model.selection();
        let (selection, caret) = match selection.is_empty() {
            true => {
                let x = line.x_for_index(input.shown_offset(input.model.cursor()));
                (None, Some(fill(Bounds::new(point(bounds.left() + x, bounds.top()), size(theme.sizes.controls.input_caret_width.px(), bounds.size.height)), rgb(theme.accent))))
            }
            false => {
                let corners = Bounds::from_corners(
                    point(bounds.left() + line.x_for_index(input.shown_offset(selection.start)), bounds.top()),
                    point(bounds.left() + line.x_for_index(input.shown_offset(selection.end)), bounds.bottom()),
                );
                (Some(fill(corners, rgba(theme.selected << 8 | 0xff))), None)
            }
        };
        PrepaintState { line: Some(line), caret, selection }
    }

    fn paint(&mut self, _: Option<&GlobalElementId>, _: Option<&gpui::InspectorElementId>, bounds: Bounds<Pixels>, _: &mut Self::RequestLayoutState, prepaint: &mut Self::PrepaintState, window: &mut Window, cx: &mut App) {
        let (focus_handle, editable) = {
            let input = self.input.read(cx);
            (input.focus_handle.clone(), input.is_editable())
        };
        if editable {
            window.handle_input(&focus_handle, ElementInputHandler::new(bounds, self.input.clone()), cx);
        }
        if let Some(selection) = prepaint.selection.take() {
            window.paint_quad(selection);
        }
        let Some(line) = prepaint.line.take() else {
            return;
        };
        // a text or placeholder longer than the field stops at its edge, not over what sits beside it
        let painted = window.with_content_mask(Some(ContentMask { bounds }), |window| line.paint(bounds.origin, window.line_height(), gpui::TextAlign::Left, None, window, cx));
        if let Err(error) = painted {
            eprintln!("ive: cannot draw a text field: {error}");
        }
        if focus_handle.is_focused(window) && let Some(caret) = prepaint.caret.take() {
            window.paint_quad(caret);
        }
        self.input.update(cx, |input, _| {
            input.last_layout = Some(line);
            input.last_bounds = Some(bounds);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn passes(keys: &[&str]) -> Vec<bool> {
        keys.iter().map(|key| passes_to_window(&Keystroke::parse(key).unwrap())).collect()
    }

    #[test]
    fn should_show_a_mask_per_character() {
        // given
        let text = "zażółć";

        // when
        let shown = masked(text);

        // then
        assert_eq!(shown, "••••••");
    }

    #[test]
    fn should_map_offsets_between_text_and_mask() {
        // given
        let text = "aż1";

        // when
        let to_mask: Vec<usize> = [0, 1, 3, 4].iter().map(|offset| masked_offset(text, *offset)).collect();
        let to_text: Vec<usize> = [0, 3, 6, 9].iter().map(|offset| unmasked_offset(text, *offset)).collect();

        // then
        assert_eq!((to_mask, to_text), (vec![0, 3, 6, 9], vec![0, 1, 3, 4]));
    }

    #[test]
    fn should_pass_only_non_typing_keys_from_text_fields() {
        // when
        let passing = passes(&["alt-f12", "secondary-,", "f5"]);
        let kept = passes(&["a", "shift-a", "backspace", "secondary-c", "enter", "ctrl-backspace"]);

        // then
        assert_eq!(passing, [true, true, true]);
        assert_eq!(kept, [false; 6]);
    }

    #[test]
    fn should_keep_typing_keys_and_give_up_enter() {
        // given
        let kept = ["a", "shift-a", "backspace", "shift-left", "home", "secondary-v"];
        let given_up = ["enter", "ctrl-y", "tab", "down", "escape", "alt-1", "shift-enter"];

        // when
        let keeps = |keys: &[&str]| keys.iter().map(|key| keeps_key(&Keystroke::parse(key).unwrap())).collect::<Vec<_>>();

        // then
        assert_eq!(keeps(&kept), [true; 6]);
        assert_eq!(keeps(&given_up), [false; 7]);
    }
}
