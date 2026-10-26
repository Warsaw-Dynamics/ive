//! Text | Rendered: a pill floating at the top right of a Markdown text, where the find box floats and in its frame;
//! the find box takes its place while open.

use std::rc::Rc;

use gpui::{AnyElement, App, MouseButton, SharedString, Window, div, prelude::*, px, rgb, rgba};

use crate::find::bar::{BORDER, MARGIN};
use crate::host::Host;
use crate::panel::{MarkdownView, Offer};
use crate::style::ViewStyle;

const SEGMENT_HEIGHT: f32 = 22.;

/// Switches the panel to a view.
pub(crate) type SwitchListener = Rc<dyn Fn(MarkdownView, &mut Window, &mut App)>;

#[cfg(target_os = "macos")]
mod tips {
    pub const TEXT: &str = "Edit the Markdown source (Cmd+Shift+M)";
    pub const RENDERED: &str = "Read it formatted (Cmd+Shift+M)";
}

#[cfg(not(target_os = "macos"))]
mod tips {
    pub const TEXT: &str = "Edit the Markdown source (Ctrl+Shift+M)";
    pub const RENDERED: &str = "Read it formatted (Ctrl+Shift+M)";
}

const TOO_LARGE: &str = "Over 8 MiB: shown as text only";

/// The pill with `shown` picked; with `offer` [`Offer::TooLarge`], Rendered is greyed and takes no click.
pub(crate) fn pill(shown: MarkdownView, offer: Offer, switch: SwitchListener, cx: &App) -> AnyElement {
    let style = ViewStyle::drawing(cx).clone();
    let segments = [(MarkdownView::Text, "text", "Text"), (MarkdownView::Rendered, "rendered", "Rendered")].map(|(view, name, label)| {
        let face = Face { view, name, label, picked: shown == view, greyed: view == MarkdownView::Rendered && offer == Offer::TooLarge };
        segment(face, switch.clone(), &style, cx)
    });
    div()
        .id("markdown-pill")
        .debug_selector(|| "markdown-pill".to_owned())
        .absolute()
        .top(px(MARGIN))
        .right(px(style.scrollbar_thickness + MARGIN))
        .flex()
        .flex_row()
        .gap(px(2.))
        .p(px(3.))
        .bg(rgb(style.background))
        .border(px(BORDER))
        .border_color(rgb(style.border))
        .rounded(px(4.))
        .shadow_md()
        .occlude()
        .cursor_default()
        .text_size(px(style.editor_notice_text_size))
        .text_color(rgb(style.text))
        // the text under it neither moves its caret nor starts a selection
        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
        .children(segments)
        .into_any_element()
}

/// One segment: its view, the click target's name after `markdown-view:`, and how it shows.
struct Face {
    view: MarkdownView,
    name: &'static str,
    label: &'static str,
    picked: bool,
    greyed: bool,
}

fn segment(face: Face, switch: SwitchListener, style: &ViewStyle, cx: &App) -> AnyElement {
    let tip = match (face.view, face.greyed) {
        (_, true) => TOO_LARGE,
        (MarkdownView::Text, false) => tips::TEXT,
        (MarkdownView::Rendered, false) => tips::RENDERED,
    };
    let element = div()
        .id(SharedString::from(format!("markdown-view-{}", face.name)))
        .flex()
        .items_center()
        .h(px(SEGMENT_HEIGHT))
        .px(px(10.))
        .rounded(px(3.))
        .border(px(BORDER))
        .border_color(rgba(0))
        .child(face.label);
    let element = match Host::tooltip(tip, cx) {
        Some(tooltip) => element.tooltip(tooltip),
        None => element,
    };
    if face.greyed {
        return element.text_color(rgb(style.muted)).into_any_element();
    }
    let view = face.view;
    let click = Host::click_target(format!("markdown-view:{}", face.name), move |_, window, cx| switch(view, window, cx), cx);
    let element = element.on_click(click);
    if face.picked {
        // as the find box's options look when on
        return element.border_color(rgb(style.accent)).bg(rgba((style.accent << 8) | 0x33)).into_any_element();
    }
    let hover = rgb(style.hover);
    element.cursor_pointer().hover(move |hovered| hovered.bg(hover)).into_any_element()
}
