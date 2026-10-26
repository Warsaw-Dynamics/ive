//! The find box over a panel's code: the query, the count, previous and next, the options and close.

use gpui::{AnyElement, App, Entity, IntoElement, MouseButton, SharedString, Window, div, img, prelude::*, px, rgb, rgba};

use super::{FindOption, FindOptions};
use crate::host::{ClickListener, Host};
use crate::icons;
use crate::line_field::LineField;
use crate::style::ViewStyle;

const BUTTON_SIZE: f32 = 24.;
pub(crate) const BORDER: f32 = 1.;
/// Between the box and the panel's top edge, and its scrollbar.
pub(crate) const MARGIN: f32 = 6.;
const WIDTH: f32 = 440.;
const COUNT_WIDTH: f32 = 96.;

/// What the box shows.
pub(crate) struct BarView {
    pub field: Entity<LineField>,
    /// `3 of 12`, `No results`, `Bad regex`
    pub count: String,
    /// why the count says what it does, as its tooltip: a regex's error, the side of a diff searched
    pub detail: Option<SharedString>,
    pub options: FindOptions,
}

/// Turns an option on or off.
pub(crate) type ToggleListener = std::rc::Rc<dyn Fn(FindOption, &mut Window, &mut App)>;

/// What its buttons run.
pub(crate) struct BarActions {
    pub focus_query: ClickListener,
    pub previous: ClickListener,
    pub next: ClickListener,
    pub toggle: ToggleListener,
    pub close: ClickListener,
}

/// The box, floating at the top right of the panel it is a child of, left of the scrollbar.
pub(crate) fn bar(view: BarView, actions: BarActions, cx: &App) -> AnyElement {
    let style = ViewStyle::drawing(cx).clone();
    let focus_query = actions.focus_query.clone();
    drop(Host::click_target("find:query".to_owned(), move |event, window, cx| focus_query(event, window, cx), cx));
    let option = |OptionButton { option, label, tip, name }: OptionButton| {
        let toggle = actions.toggle.clone();
        let listener: ClickListener = std::rc::Rc::new(move |_, window, cx| toggle(option, window, cx));
        let on = match option {
            FindOption::MatchCase => view.options.match_case,
            FindOption::WholeWord => view.options.whole_word,
            FindOption::Regex => view.options.regex,
        };
        let pressed = if on { Pressed::On } else { Pressed::Off };
        toggle_button(Button { name, face: Face::Text(label), tip, listener }, pressed, &style, cx)
    };
    let count = div()
        .id("find-count")
        .flex_none()
        .w(px(COUNT_WIDTH))
        .text_center()
        .whitespace_nowrap()
        .text_color(rgb(style.muted))
        .child(view.count);
    let count = match (view.detail, cx.try_global::<Host>().map(|host| host.tooltip.clone())) {
        (Some(detail), Some(tooltip)) => count.tooltip(move |window, cx| tooltip(detail.clone(), window, cx)).into_any_element(),
        _ => count.into_any_element(),
    };
    div()
        .id("find-box")
        .absolute()
        .top(px(MARGIN))
        .right(px(style.scrollbar_thickness + MARGIN))
        .w(px(WIDTH))
        .max_w(gpui::relative(1.))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(2.))
        .p(px(4.))
        .bg(rgb(style.background))
        .border(px(BORDER))
        .border_color(rgb(style.border))
        .rounded(px(4.))
        .shadow_md()
        .occlude()
        .cursor_default()
        .text_size(px(style.editor_notice_text_size))
        .text_color(rgb(style.text))
        // the code under it neither moves its caret nor starts a selection
        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
        .child(div().flex_1().min_w_0().flex().child(view.field))
        .child(count)
        .child(button(Button { name: "previous", face: Face::Icon("arrow-up.svg"), tip: "Previous match (Shift+Enter, Shift+F3)", listener: actions.previous }, &style, cx))
        .child(button(Button { name: "next", face: Face::Icon("arrow-down.svg"), tip: "Next match (Enter, F3)", listener: actions.next }, &style, cx))
        .child(option(OptionButton { option: FindOption::MatchCase, label: "Aa", tip: tips::MATCH_CASE, name: "match-case" }))
        .child(option(OptionButton { option: FindOption::WholeWord, label: "W", tip: tips::WHOLE_WORD, name: "whole-word" }))
        .child(option(OptionButton { option: FindOption::Regex, label: ".*", tip: tips::REGEX, name: "regex" }))
        .child(button(Button { name: "close", face: Face::Icon("close.svg"), tip: "Close (Esc)", listener: actions.close }, &style, cx))
        .into_any_element()
}

/// An option's button: its option, face, tooltip and click target's name after `find:`.
struct OptionButton {
    option: FindOption,
    label: &'static str,
    tip: &'static str,
    name: &'static str,
}

/// The options' tooltips, with their keys: Cmd+Alt on macOS, where Alt alone types characters.
#[cfg(target_os = "macos")]
mod tips {
    pub const MATCH_CASE: &str = "Match case (Cmd+Alt+C)";
    pub const WHOLE_WORD: &str = "Whole words (Cmd+Alt+W)";
    pub const REGEX: &str = "Regex (Cmd+Alt+R)";
}

#[cfg(not(target_os = "macos"))]
mod tips {
    pub const MATCH_CASE: &str = "Match case (Alt+C)";
    pub const WHOLE_WORD: &str = "Whole words (Alt+W)";
    pub const REGEX: &str = "Regex (Alt+R)";
}

enum Face {
    Icon(&'static str),
    Text(&'static str),
}

struct Button {
    /// the click target's name after `find:`
    name: &'static str,
    face: Face,
    tip: &'static str,
    listener: ClickListener,
}

fn button(button: Button, style: &ViewStyle, cx: &App) -> AnyElement {
    shaped(button, style, cx).into_any_element()
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Pressed {
    On,
    Off,
}

/// An option's button: pressed while the option is on.
fn toggle_button(button: Button, pressed: Pressed, style: &ViewStyle, cx: &App) -> AnyElement {
    let element = shaped(button, style, cx).border(px(BORDER));
    match pressed {
        Pressed::On => element.border_color(rgb(style.accent)).bg(rgba((style.accent << 8) | 0x33)).into_any_element(),
        Pressed::Off => element.border_color(rgba(0)).into_any_element(),
    }
}

fn shaped(button: Button, style: &ViewStyle, cx: &App) -> gpui::Stateful<gpui::Div> {
    let listener = button.listener;
    let click = Host::click_target(format!("find:{}", button.name), move |event, window, cx| listener(event, window, cx), cx);
    let face = match button.face {
        Face::Icon(icon) => img(icons::embedded(style.appearance, icon)).size(px(style.icon_size)).into_any_element(),
        Face::Text(text) => div().font_family(style.mono_font.clone()).child(text).into_any_element(),
    };
    let element = div()
        .id(SharedString::from(format!("find-{}", button.name)))
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .size(px(BUTTON_SIZE))
        .rounded(px(3.))
        .cursor_pointer()
        .hover(|hovered| hovered.bg(rgb(style.hover)))
        .on_click(click)
        .child(face);
    match Host::tooltip(button.tip, cx) {
        Some(tooltip) => element.tooltip(tooltip),
        None => element,
    }
}
