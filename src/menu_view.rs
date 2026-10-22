//! Drawing the open menu of a tab or a tree row, its open submenus beside their lines, and its pending lines faded;
//! what it offers is in `workspace::menu`.

use gpui::{AnyElement, Context, Div, Stateful, anchored, deferred, div, prelude::*, px, rgb};

use crate::click_targets::target;
use crate::menu::resolve::{ItemId, Line, LineAction, LineState};
use crate::script::menus::ItemAction;
use crate::theme::Theme;
use crate::workspace::{OpenMenu, Workspace, line_target, tab_action_target};

/// How faded a pending line is.
const PENDING_OPACITY: f32 = 0.5;

/// The open menu at the pointer, kept inside the window, over a layer that closes it on a press outside.
pub fn render_menu(workspace: &Workspace, theme: Theme, window: &gpui::Window, cx: &mut Context<Workspace>) -> Option<impl IntoElement> {
    let menu = workspace.menu()?;
    let size = window.viewport_size();
    let main = anchored().position(menu.position).snap_to_window().child(panel(menu, &menu.lines, &[], theme, cx));
    let layer = div()
        .id("menu-layer")
        .w(size.width)
        .h(size.height)
        .occlude()
        .on_any_mouse_down(cx.listener(|workspace, _event, window, cx| workspace.close_context_menu(window, cx)))
        .child(main);
    Some(deferred(anchored().position(gpui::Point::default()).child(layer)).with_priority(1))
}

fn panel(menu: &OpenMenu, lines: &[Line], labels: &[String], theme: Theme, cx: &mut Context<Workspace>) -> Stateful<Div> {
    let sizes = &theme.sizes.tree;
    let name = format!("menu-{}", labels.join("/"));
    let rows: Vec<AnyElement> = lines.iter().map(|line| row(menu, line, labels, theme, cx)).collect();
    div()
        .id(gpui::SharedString::from(name))
        .p(sizes.menu_padding.px())
        .rounded(sizes.menu_radius.px())
        .border(sizes.border.px())
        .border_color(rgb(theme.border))
        .bg(rgb(theme.sidebar))
        .shadow_lg()
        .occlude()
        // a press inside a panel is not a press outside the menu
        .on_any_mouse_down(|_event, _window, cx| cx.stop_propagation())
        .text_size(sizes.menu_text_size.px())
        .font_weight(sizes.menu_weight.font_weight())
        .text_color(rgb(theme.text))
        .children(rows)
}

fn row(menu: &OpenMenu, line: &Line, labels: &[String], theme: Theme, cx: &mut Context<Workspace>) -> AnyElement {
    let sizes = &theme.sizes.tree;
    match line {
        Line::Separator => div().my(sizes.menu_padding.px()).h(px(sizes.border.0)).bg(rgb(theme.border)).into_any_element(),
        Line::Item { id, label, action, state, enabled } => {
            let path = [labels, &[target_label(label, action)]].concat();
            item(menu, Shown { id, path: &path, label, state: *state, enabled: *enabled, submenu: None }, theme, cx).into_any_element()
        }
        Line::Submenu { id, label, lines, state } => {
            let path = [labels, std::slice::from_ref(label)].concat();
            let open = menu.nav.open().contains(id).then_some(lines.as_slice());
            item(menu, Shown { id, path: &path, label, state: *state, enabled: true, submenu: Some(open) }, theme, cx).into_any_element()
        }
    }
}

/// A line as drawn: `submenu` is `Some` for a submenu's line, holding its lines while it is open.
struct Shown<'a> {
    id: &'a ItemId,
    path: &'a [String],
    label: &'a str,
    state: LineState,
    enabled: bool,
    submenu: Option<Option<&'a [Line]>>,
}

fn item(menu: &OpenMenu, shown: Shown, theme: Theme, cx: &mut Context<Workspace>) -> Stateful<Div> {
    let sizes = &theme.sizes.tree;
    let id = shown.id.clone();
    let highlighted = menu.nav.highlighted() == Some(shown.id);
    let text = match shown.state {
        LineState::Pending => format!("{} …", shown.label),
        LineState::Ready => shown.label.to_owned(),
    };
    let arrow = shown.submenu.is_some().then(|| div().pl(sizes.menu_item_padding_x.px()).text_color(rgb(theme.muted)).child("▸"));
    let line = div()
        .id(gpui::SharedString::from(format!("menu-line-{:?}", shown.id)))
        .relative()
        .flex()
        .items_center()
        .justify_between()
        .h(sizes.menu_item_height.px())
        .px(sizes.menu_item_padding_x.px())
        .rounded(sizes.menu_item_radius.px())
        .whitespace_nowrap()
        .when(highlighted, |line| line.bg(rgb(theme.hover)))
        .child(text)
        .children(arrow);
    let name = line_target(menu.kind, shown.path);
    if shown.state == LineState::Pending {
        drop(target(format!("{name} (pending)"), |_, _, _| {}, cx));
        return line.opacity(PENDING_OPACITY);
    }
    if !shown.enabled {
        return line.text_color(rgb(theme.muted));
    }
    let hovered = id.clone();
    let line = line.cursor_pointer().on_hover(cx.listener(move |workspace, hovering: &bool, _window, cx| {
        if *hovering {
            workspace.hover_menu_line(hovered.clone(), cx);
        }
    }));
    // a line that just moved under a still pointer is not the one the press meant
    let choose = move |workspace: &mut Workspace, window: &mut gpui::Window, cx: &mut Context<Workspace>| {
        if workspace.menu().is_some_and(OpenMenu::is_settled) {
            workspace.choose_menu_line(&id, window, cx);
        }
    };
    let line = line.on_click(target(name, cx.listener(move |workspace, _event, window, cx| choose(workspace, window, cx)), cx));
    let line = match tab_alias(shown.path, menu) {
        Some(alias) => {
            let id = shown.id.clone();
            drop(target(alias.to_owned(), cx.listener(move |workspace, _event, window, cx| workspace.choose_menu_line(&id, window, cx)), cx));
            line
        }
        None => line,
    };
    match shown.submenu {
        Some(Some(lines)) => {
            let beside = div().absolute().top(-sizes.menu_padding.px()).left_full().child(anchored().snap_to_window().child(panel(menu, lines, shown.path, theme, cx)));
            line.child(beside)
        }
        _ => line,
    }
}

/// A line's part of its target name: its label, or a built-in choice's key.
fn target_label(label: &str, action: &LineAction) -> String {
    match action {
        LineAction::Choice { choice, .. } => choice.clone(),
        LineAction::Script(_) => label.to_owned(),
    }
}

/// Today's `tab-menu:keep` … names of the tab actions on a tab menu's top level.
fn tab_alias(path: &[String], menu: &OpenMenu) -> Option<&'static str> {
    let [label] = path else {
        return None;
    };
    let line = menu.lines.iter().find(|line| matches!(line, Line::Item { label: shown, .. } if shown == label))?;
    match line {
        Line::Item { action: LineAction::Script(ItemAction::Builtin(builtin)), .. } if menu.kind == crate::script::menus::MenuKind::Tab => tab_action_target(*builtin),
        _ => None,
    }
}
