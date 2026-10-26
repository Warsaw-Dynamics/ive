//! The tab row's bell: there while processes wait for the user; its list shows them, longest waiting first, and a
//! line shows its process.

use gpui::{Context, Div, ParentElement, Stateful, Styled, div, deferred, img, prelude::*, px, rgb};

use crate::age::{Clock, unix_seconds};
use crate::attention_dot::with_dot;
use crate::click_targets::target;
use crate::icons;
use crate::process::Process;
use crate::sidebar::view::{HeaderMenu, closing_outside, header_button_press, menu_item, menu_panel, menu_section};
use crate::theme::Theme;
use crate::tooltip;
use crate::workspace::{Workspace, icon};

/// The list's width, in pixels: a process title, its worktree and what it said fit on two lines.
const LIST_WIDTH: f32 = 340.0;

/// The bell, and its list while open; `None` while no process waits.
pub fn waiting_button(workspace: &Workspace, theme: Theme, cx: &mut Context<Workspace>) -> Option<Div> {
    let waiting = workspace.processes().waiting();
    let count = waiting.len();
    if count == 0 {
        return None;
    }
    let sizes = &theme.sizes.tabs;
    let bell = img(icons::embedded(theme.appearance, icons::NOTIFICATIONS_20)).size(sizes.toolbar_icon_size.px());
    let button = div()
        .id("waiting")
        .p(sizes.toolbar_button_padding.px())
        .rounded(sizes.toolbar_button_radius.px())
        .cursor_pointer()
        .hover(|style| style.bg(rgb(theme.hover)))
        .child(with_dot(bell, theme, theme.sidebar))
        .tooltip(tooltip::text_tooltip(format!("{count} waiting")));
    let toggle = cx.listener(|workspace, _event, window, cx| workspace.toggle_menu(HeaderMenu::Waiting, window, cx));
    let button = header_button_press(button, cx).on_click(target("waiting".to_owned(), toggle, cx));
    // lines only while open: each names a click target
    let list = (workspace.header_menu() == Some(HeaderMenu::Waiting)).then(|| {
        let lines: Vec<Stateful<Div>> = waiting.iter().map(|process| line(workspace, process, theme, cx)).collect();
        deferred(list(lines, theme, cx)).with_priority(1)
    });
    Some(div().relative().child(button).children(list))
}

fn list(lines: Vec<Stateful<Div>>, theme: Theme, cx: &mut Context<Workspace>) -> Stateful<Div> {
    let menu = menu_panel("waiting-menu", theme)
        .top(px(theme.sizes.tabs.row_height.0))
        .right(px(0.0))
        .w(px(LIST_WIDTH))
        .child(menu_section("Waiting, longest first", theme))
        .children(lines);
    closing_outside(menu, cx)
}

/// One waiting process: its icon, `title · worktree`, what it said, and how long ago it first called.
fn line(workspace: &Workspace, process: &Process, theme: Theme, cx: &mut Context<Workspace>) -> Stateful<Div> {
    let id = process.id;
    let worktree = process.checkout.as_ref().map(|checkout| format!(" · {}", workspace.checkout_label(checkout))).unwrap_or_default();
    let (said, age) = process.attention().map_or_else(Default::default, |attention| {
        (attention.text.clone(), Clock::system().age(unix_seconds(attention.since)))
    });
    let small = theme.sizes.tree.tag_text_size.px();
    let words = div()
        .flex_1()
        .min_w_0()
        .flex()
        .flex_col()
        .child(div().overflow_hidden().text_ellipsis().child(format!("{}{worktree}", process.title())))
        .child(div().overflow_hidden().text_ellipsis().text_size(small).text_color(rgb(theme.attention)).child(said));
    let show = cx.listener(move |workspace, _event, window, cx| workspace.show_waiting(id, window, cx));
    menu_item(("waiting-line", id.value()), theme)
        .h_auto()
        .py(px(4.0))
        .items_start()
        .child(icon(theme, process.icon()))
        .child(words)
        .child(div().flex_none().text_size(small).text_color(rgb(theme.muted)).child(age))
        .on_click(target(format!("waiting:{}", id.value()), show, cx))
}
