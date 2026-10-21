//! The bar over a running static server's output: its link, which opens the browser, and a button copying it.

use gpui::{App, Div, div, prelude::*, px, rgb};

use crate::click_targets::target;
use crate::clipboard;
use crate::icons;
use crate::process::ProcessId;
use crate::script::keymap::ClipboardTarget;
use crate::theme::Theme;
use crate::tooltip;
use crate::workspace::icon;

/// `place` tells the bars of one process apart (its tab, the folder overview), for ids and click targets.
pub fn link_bar(process: ProcessId, place: &'static str, url: &str, theme: Theme, cx: &App) -> Div {
    let opened = url.to_owned();
    let open = target(format!("open-link:{place}:{}", process.value()), move |_event, _window, cx| cx.open_url(&opened), cx);
    let copied = url.to_owned();
    let copy = target(
        format!("copy-link:{place}:{}", process.value()),
        move |_event, _window, cx| clipboard::write(ClipboardTarget::Clipboard, copied.clone(), cx),
        cx,
    );
    let link = div()
        .id(("server-link", process.value() as usize))
        .min_w_0()
        .overflow_hidden()
        .text_ellipsis()
        .whitespace_nowrap()
        .text_color(rgb(theme.accent))
        .cursor_pointer()
        .hover(|style| style.underline())
        .tooltip(tooltip::text_tooltip("Open in the browser"))
        .child(url.to_owned())
        .on_click(open);
    let copy_button = div()
        .id(("server-link-copy", process.value() as usize))
        .flex_none()
        .flex()
        .items_center()
        .gap_1()
        .px(px(6.))
        .py(px(2.))
        .rounded_sm()
        .cursor_pointer()
        .hover(|style| style.bg(rgb(theme.border)))
        .child(icon(theme, icons::COPY))
        .child("Copy")
        .tooltip(tooltip::text_tooltip("Copy the link"))
        .on_click(copy);
    div()
        .flex()
        .flex_none()
        .items_center()
        .justify_between()
        .gap_2()
        .px(px(10.))
        .py(px(4.))
        .border_b_1()
        .border_color(rgb(theme.border))
        .bg(rgb(theme.sidebar))
        .text_sm()
        .child(link)
        .child(copy_button)
}
