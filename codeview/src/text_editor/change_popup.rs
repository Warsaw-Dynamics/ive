//! The popup a change marker opens: the change's text in the base (HEAD), and what to do with the change.

use std::rc::Rc;

use gpui::{AnyElement, App, MouseButton, Pixels, SharedString, Window, div, img, prelude::*, px, rgb, rgba};
use tree_sitter::InputEdit;

use crate::changes::{ChangeHunk, ChangeMarkers, Marker, shifted};
use crate::host::{ClickListener, Host};
use crate::icons;
use crate::style::ViewStyle;

/// Lines of the previous text shown before it scrolls.
const MAX_SHOWN_LINES: f32 = 12.;
/// The toolbar row's height and its buttons' size.
const TOOLBAR_HEIGHT: f32 = 30.;
const BUTTON_SIZE: f32 = 24.;
/// Padding around the previous text, and the popup's border.
const TEXT_PADDING: f32 = 4.;
const BORDER: f32 = 1.;
/// The widest the popup gets, and its distance from the editor's right edge.
const MAX_WIDTH: f32 = 720.;
const RIGHT_MARGIN: f32 = 16.;

/// Whether a change's popup is open.
pub(crate) enum Popup {
    Closed,
    Open(ChangePopup),
}

/// An open popup: the change it is for, that change's lines in the base, and the action waiting for the markers of
/// the current text.
pub(crate) struct ChangePopup {
    /// the change, in the current text's lines
    block: ChangeHunk,
    /// the change's lines in the base as shown, with `\n` line breaks; empty for added lines
    previous: String,
    waiting: Waiting,
    commit: Committing,
}

/// Where committing the change shown is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Committing {
    Idle,
    /// sent to the host, which has not answered yet
    Sent,
    /// refused or failed, for the reason shown
    Failed(String),
}

/// What a popup's button does with its change, which needs the markers to be the diff of the current text: markers
/// moved by edits may hold lines that are not the change's (a line joined to its last one).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ChangeAction {
    Rollback,
    Copy,
    Commit,
}

#[derive(Clone, Copy)]
enum Waiting {
    Nothing,
    /// to run on the change as the diff of the current text finds it
    Action(ChangeAction),
}

impl ChangePopup {
    pub(crate) fn of(hunk: &ChangeHunk, markers: &ChangeMarkers) -> Self {
        let previous = String::from_utf8_lossy(&markers.base_text(hunk)).replace("\r\n", "\n");
        Self { block: hunk.clone(), previous, waiting: Waiting::Nothing, commit: Committing::Idle }
    }

    /// Runs `action` once the markers are the diff of the current text, instead of an action waiting before.
    pub(crate) fn wait_to(&mut self, action: ChangeAction) {
        self.waiting = Waiting::Action(action);
    }

    /// The action waiting, which no longer waits.
    pub(crate) fn take_waiting(&mut self) -> Option<ChangeAction> {
        match std::mem::replace(&mut self.waiting, Waiting::Nothing) {
            Waiting::Action(action) => Some(action),
            Waiting::Nothing => None,
        }
    }

    pub(crate) fn is_waiting(&self) -> bool {
        matches!(self.waiting, Waiting::Action(_))
    }

    pub(crate) fn committing(&self) -> &Committing {
        &self.commit
    }

    pub(crate) fn set_committing(&mut self, commit: Committing) {
        self.commit = commit;
    }

    pub(crate) fn block(&self) -> &ChangeHunk {
        &self.block
    }

    pub(crate) fn previous(&self) -> &str {
        &self.previous
    }

    /// The previous text's lines as shown, without their line breaks.
    fn previous_lines(&self) -> Vec<&str> {
        let text = self.previous.strip_suffix('\n').unwrap_or(&self.previous);
        if self.block.base.is_empty() {
            return Vec::new();
        }
        text.split('\n').collect()
    }
}

impl Popup {
    /// After `edits`: closed by one that touched text outside the change, else open on where the change went.
    pub(crate) fn edited(self, edits: &[InputEdit], markers: &ChangeMarkers) -> Self {
        let Self::Open(popup) = self else {
            return self;
        };
        let mut block = popup.block.clone();
        for edit in edits {
            if !is_inside(&block, edit) {
                return Self::Closed;
            }
            block = shifted(&block, edit);
        }
        Self::Open(ChangePopup { block, ..popup }).followed(markers)
    }

    /// Open on the change of `markers` that meets the one shown, which may have grown or moved; closed when it is gone.
    pub(crate) fn followed(self, markers: &ChangeMarkers) -> Self {
        let Self::Open(popup) = self else {
            return self;
        };
        match markers.meeting(&popup.block.current) {
            Some(hunk) if *hunk == popup.block => Self::Open(popup),
            Some(hunk) => Self::Open(ChangePopup { waiting: popup.waiting, commit: popup.commit, ..ChangePopup::of(hunk, markers) }),
            None => Self::Closed,
        }
    }
}

/// Whether `edit` changed only the lines of `block`, or inserted text at the start of the line after them (for
/// deleted lines, where they were). Removing the last line's break joins the next line to the block: outside.
fn is_inside(block: &ChangeHunk, edit: &InputEdit) -> bool {
    let (start, end) = (edit.start_position, edit.old_end_position);
    if start.row < block.current.start {
        return false;
    }
    if end.row < block.current.end {
        return true;
    }
    start == end && (end.row, end.column) == (block.current.end, 0)
}

/// Closes the popup.
pub(crate) type CloseListener = Rc<dyn Fn(&mut Window, &mut App)>;

/// What the popup's buttons do, and the commit message's field.
pub(crate) struct PopupActions {
    pub previous: ClickListener,
    pub next: ClickListener,
    pub rollback: ClickListener,
    pub diff: ClickListener,
    pub copy: ClickListener,
    pub commit: ClickListener,
    /// gives the message field the keyboard
    pub focus_message: ClickListener,
    pub message: AnyElement,
    /// a press outside the popup
    pub close: CloseListener,
}

/// Where the popup goes in the editor, in the editor's pixels.
#[derive(Clone, Copy)]
pub(crate) struct Anchor {
    /// the change's lines: top and bottom
    pub top: Pixels,
    pub bottom: Pixels,
    /// the left of the text and its width
    pub left: Pixels,
    pub width: Pixels,
    /// the height of the editor
    pub height: Pixels,
    pub line_height: Pixels,
    pub font_size: Pixels,
}

/// The popup under the change, or above it when there is no room below, inside the editor.
pub(crate) fn popup_element(popup: &ChangePopup, anchor: Anchor, actions: PopupActions, cx: &App) -> AnyElement {
    let style = ViewStyle::drawing(cx).clone();
    let lines = popup.previous_lines();
    let text_height = anchor.line_height * (lines.len().max(1) as f32).min(MAX_SHOWN_LINES) + px(2. * TEXT_PADDING);
    let height = px(TOOLBAR_HEIGHT + 2. * BORDER) + text_height;
    let below = anchor.bottom + height <= anchor.height || anchor.top - height < px(0.);
    let top = if below { anchor.bottom } else { anchor.top - height };
    let width = (anchor.width - px(RIGHT_MARGIN)).min(px(MAX_WIDTH)).max(px(0.));
    let close = actions.close.clone();
    let state = if popup.is_waiting() { ButtonState::Busy } else { ButtonState::Ready };
    let sending = if popup.is_waiting() || popup.commit == Committing::Sent { ButtonState::Busy } else { ButtonState::Ready };
    div()
        .id("change-popup")
        .absolute()
        .left(anchor.left)
        .top(top.max(px(0.)))
        .w(width)
        .flex()
        .flex_col()
        .overflow_hidden()
        .bg(rgb(style.background))
        .border(px(BORDER))
        .border_color(rgb(style.border))
        .rounded(px(4.))
        .shadow_md()
        .occlude()
        .cursor_default()
        // the editor under it neither moves its caret nor starts a selection
        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
        .on_mouse_down_out(move |_, window, cx| close(window, cx))
        .child(toolbar(actions, [state, sending], &style, cx))
        .children(failure(&popup.commit, &style))
        .child(previous_text(popup, &lines, anchor, &style))
        .into_any_element()
}

/// The buttons, in a row, those acting on the change shown as `state` says, then the commit message's field and its
/// send button, as `sending` says.
fn toolbar(actions: PopupActions, [state, sending]: [ButtonState; 2], style: &ViewStyle, cx: &App) -> impl IntoElement {
    let ready = ButtonState::Ready;
    let focus_message = actions.focus_message;
    // MCP's `click` gives the field the keyboard; a press on it does that itself
    drop(Host::click_target("change-popup:message".to_owned(), move |event, window, cx| focus_message(event, window, cx), cx));
    let send = Button { name: "commit", icon: "commit.svg", tip: "Commit this change (Enter)", listener: actions.commit, state: sending };
    let buttons = [
        ("prev", "arrow-up.svg", "Previous change", actions.previous, ready),
        ("next", "arrow-down.svg", "Next change", actions.next, ready),
        ("rollback", "rollback.svg", "Rollback", actions.rollback, state),
        ("diff", "changes.svg", "Show diff", actions.diff, ready),
        ("copy", "copy.svg", "Copy the previous text", actions.copy, state),
    ];
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(2.))
        .h(px(TOOLBAR_HEIGHT))
        .px(px(4.))
        .border_b(px(BORDER))
        .border_color(rgb(style.border))
        .children(buttons.map(|(name, icon, tip, listener, state)| button(Button { name, icon, tip, listener, state }, style, cx)))
        .child(div().w(px(8.)))
        .child(actions.message)
        .child(button(send, style, cx))
}

/// Why the last commit of the change failed, in a row under the buttons.
fn failure(commit: &Committing, style: &ViewStyle) -> Option<AnyElement> {
    let Committing::Failed(reason) = commit else {
        return None;
    };
    let row = div().px(px(8.)).py(px(3.)).text_size(px(style.editor_notice_text_size)).text_color(rgb(style.error));
    Some(row.border_b(px(BORDER)).border_color(rgb(style.border)).child(SharedString::from(reason.clone())).into_any_element())
}

struct Button {
    /// the click target's name after `change-popup:`
    name: &'static str,
    icon: &'static str,
    tip: &'static str,
    listener: ClickListener,
    state: ButtonState,
}

/// A busy button waits for the markers of the current text to run what it was clicked for.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ButtonState {
    Ready,
    Busy,
}

/// How faint a busy button's icon is.
const BUSY_OPACITY: f32 = 0.4;

fn button(button: Button, style: &ViewStyle, cx: &App) -> AnyElement {
    let listener = button.listener;
    let click = Host::click_target(format!("change-popup:{}", button.name), move |event, window, cx| listener(event, window, cx), cx);
    let element = div()
        .id(SharedString::from(button.name))
        .flex()
        .items_center()
        .justify_center()
        .size(px(BUTTON_SIZE))
        .rounded(px(3.))
        .cursor_pointer()
        .hover(|hovered| hovered.bg(rgb(style.hover)))
        .on_click(click)
        .when(button.state == ButtonState::Busy, |busy| busy.opacity(BUSY_OPACITY).cursor_default())
        .child(img(icons::embedded(style.appearance, button.icon)).size(px(style.icon_size)));
    match Host::tooltip(button.tip, cx) {
        Some(tooltip) => element.tooltip(tooltip).into_any_element(),
        None => element.into_any_element(),
    }
}

/// The change's lines in the base, in the removed lines' tint; a note for added lines, which have none.
fn previous_text(popup: &ChangePopup, lines: &[&str], anchor: Anchor, style: &ViewStyle) -> AnyElement {
    let text = div()
        .id("change-popup-text")
        .max_h(anchor.line_height * MAX_SHOWN_LINES + px(2. * TEXT_PADDING))
        .overflow_y_scroll()
        .py(px(TEXT_PADDING))
        .font_family(style.mono_font.clone())
        .text_size(anchor.font_size)
        .line_height(anchor.line_height);
    if popup.block.marker() == Marker::Added {
        return text.px(px(8.)).text_color(rgb(style.muted)).child("added lines").into_any_element();
    }
    let line = |line: &&str| div().h(anchor.line_height).px(px(8.)).whitespace_nowrap().child(SharedString::from(line.to_string()));
    text.bg(rgba(style.diff_removed)).text_color(rgb(style.text)).children(lines.iter().map(line)).into_any_element()
}

#[cfg(test)]
mod tests {
    use tree_sitter::Point;

    use super::*;

    fn block(current: std::ops::Range<usize>) -> ChangeHunk {
        ChangeHunk { base: 0..1, current }
    }

    /// An edit replacing from `start` to `end` (row, column), leaving the end where it was.
    fn edit(start: (usize, usize), end: (usize, usize)) -> InputEdit {
        let point = |(row, column)| Point { row, column };
        InputEdit { start_byte: 0, old_end_byte: 0, new_end_byte: 0, start_position: point(start), old_end_position: point(end), new_end_position: point(end) }
    }

    #[test]
    fn should_take_only_edits_within_the_change_as_inside() {
        // given
        let changed = block(2..4);
        let deleted = block(2..2);

        // when
        let inside = [edit((2, 0), (2, 3)), edit((3, 5), (3, 5)), edit((4, 0), (4, 0))].map(|edit| is_inside(&changed, &edit));
        let joining_the_next_line = edit((3, 2), (4, 0));
        let outside = [edit((1, 9), (2, 0)), edit((4, 0), (4, 1)), edit((3, 0), (4, 1)), joining_the_next_line]
            .map(|edit| is_inside(&changed, &edit));
        let at_deleted = [edit((2, 0), (2, 0)), edit((2, 1), (2, 1))].map(|edit| is_inside(&deleted, &edit));

        // then
        assert_eq!((inside, outside, at_deleted), ([true; 3], [false; 4], [true, false]));
    }
}
