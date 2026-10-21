//! Drawing the menu a right click on a tab opens; what it offers is in `workspace::tab_menu`.

use gpui::{Context, Div, Stateful, anchored, deferred, div, prelude::*, px, rgb};

use crate::click_targets::target;
use crate::theme::Theme;
use crate::script::keymap::Builtin;
use crate::workspace::{MenuLine, Workspace, tab_command_label, tab_command_target};

/// The open menu at the pointer, kept inside the window; a press outside closes it.
pub fn render_tab_menu(workspace: &Workspace, theme: Theme, cx: &mut Context<Workspace>) -> Option<impl IntoElement> {
    let menu = workspace.tab_menu()?;
    let sizes = &theme.sizes.tree;
    let lines = workspace.tab_menu_lines(menu).into_iter().enumerate().map(|(index, line)| match line {
        MenuLine::Item { command, enabled } => item(index, command, enabled, theme, cx).into_any_element(),
        MenuLine::Separator => div().my(sizes.menu_padding.px()).h(px(sizes.border.0)).bg(rgb(theme.border)).into_any_element(),
    });
    let body = div()
        .id("tab-menu")
        .p(sizes.menu_padding.px())
        .rounded(sizes.menu_radius.px())
        .border(sizes.border.px())
        .border_color(rgb(theme.border))
        .bg(rgb(theme.sidebar))
        .shadow_lg()
        .occlude()
        .text_size(sizes.menu_text_size.px())
        .font_weight(sizes.menu_weight.font_weight())
        .text_color(rgb(theme.text))
        .children(lines)
        .on_mouse_down_out(cx.listener(|workspace, _event, window, cx| workspace.close_tab_menu(window, cx)));
    Some(deferred(anchored().position(menu.position).snap_to_window().child(body)).with_priority(1))
}

fn item(index: usize, command: Builtin, enabled: bool, theme: Theme, cx: &mut Context<Workspace>) -> Stateful<Div> {
    let sizes = &theme.sizes.tree;
    let line = div()
        .id(("tab-menu-item", index))
        .flex()
        .items_center()
        .h(sizes.menu_item_height.px())
        .px(sizes.menu_item_padding_x.px())
        .rounded(sizes.menu_item_radius.px())
        .whitespace_nowrap()
        .child(tab_command_label(command).unwrap_or_default());
    if enabled {
        let name = tab_command_target(command).unwrap_or_default().to_owned();
        let run = target(name, cx.listener(move |workspace, _event, window, cx| workspace.run_tab_menu_item(command, window, cx)), cx);
        return line.cursor_pointer().hover(|style| style.bg(rgb(theme.hover))).on_click(run);
    }
    line.text_color(rgb(theme.muted))
}
