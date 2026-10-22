//! Drawing the open menu of a tab or a tree row, its open submenus beside their lines, and its pending lines faded;
//! what it offers is in `workspace::menu`.

mod beside;

use gpui::{AnyElement, Context, Div, Stateful, anchored, deferred, div, prelude::*, rgb};

use crate::click_targets::target;
use crate::icons;
use crate::menu::keys::shortcut;
use crate::menu::resolve::{ItemId, Line, LineAction, LineState};
use crate::script::menus::ItemAction;
use crate::theme::Theme;
use crate::workspace::{OpenMenu, Workspace, line_target, mask_icon, tab_action_target};

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
    let sizes = &theme.sizes.menu;
    let name = format!("menu-{}", labels.join("/"));
    let rows: Vec<AnyElement> = lines.iter().map(|line| row(menu, line, labels, theme, cx)).collect();
    div()
        .id(gpui::SharedString::from(name))
        .min_w(sizes.min_width.px())
        .py(sizes.padding_y.px())
        .rounded(sizes.radius.px())
        .border(sizes.border.px())
        .border_color(rgb(theme.menu.border.0))
        .bg(rgb(theme.menu.background.0))
        .shadow_lg()
        .occlude()
        // a press inside a panel is not a press outside the menu
        .on_any_mouse_down(|_event, _window, cx| cx.stop_propagation())
        .text_size(sizes.text_size.px())
        .font_weight(sizes.weight.font_weight())
        .text_color(rgb(theme.menu.text.0))
        .children(rows)
}

fn row(menu: &OpenMenu, line: &Line, labels: &[String], theme: Theme, cx: &mut Context<Workspace>) -> AnyElement {
    let sizes = &theme.sizes.menu;
    match line {
        Line::Separator => {
            div().my(sizes.separator_margin_y.px()).mx(sizes.separator_inset_x.px()).h(sizes.border.px()).bg(rgb(theme.menu.separator.0)).into_any_element()
        }
        Line::Item { id, label, action, state, enabled } => {
            let path = [labels, &[target_label(label, action)]].concat();
            let key = crate::script::state::keymap(cx).and_then(|keymap| shortcut(keymap, menu.kind, action));
            item(menu, Shown { id, path: &path, label, key, state: *state, enabled: *enabled, submenu: None }, theme, cx).into_any_element()
        }
        Line::Submenu { id, label, lines, state } => {
            let path = [labels, std::slice::from_ref(label)].concat();
            let open = menu.nav.open().contains(id).then_some(lines.as_slice());
            item(menu, Shown { id, path: &path, label, key: None, state: *state, enabled: true, submenu: Some(open) }, theme, cx).into_any_element()
        }
    }
}

/// A line as drawn: `submenu` is `Some` for a submenu's line, holding its lines while it is open.
struct Shown<'a> {
    id: &'a ItemId,
    path: &'a [String],
    label: &'a str,
    /// the key bound to its action, drawn on its right
    key: Option<String>,
    state: LineState,
    enabled: bool,
    submenu: Option<Option<&'a [Line]>>,
}

fn item(menu: &OpenMenu, shown: Shown, theme: Theme, cx: &mut Context<Workspace>) -> Stateful<Div> {
    let sizes = &theme.sizes.menu;
    let id = shown.id.clone();
    // as IntelliJ, a submenu's line stays highlighted while the submenu is open
    let highlighted = menu.nav.highlighted() == Some(shown.id) || matches!(shown.submenu, Some(Some(_)));
    let text = match shown.state {
        LineState::Pending => format!("{} …", shown.label),
        LineState::Ready => shown.label.to_owned(),
    };
    let key_colour = if shown.enabled { theme.menu.shortcut } else { theme.menu.disabled };
    let key = shown.key.map(|key| div().flex_none().ml(sizes.shortcut_gap.px()).text_color(rgb(key_colour.0)).child(key));
    let arrow = shown.submenu.is_some().then(|| div().flex_none().ml(sizes.shortcut_gap.px()).child(mask_icon(theme, icons::CHEVRON_RIGHT)));
    let line = div()
        .id(gpui::SharedString::from(format!("menu-line-{:?}", shown.id)))
        .relative()
        .flex()
        .items_center()
        .justify_between()
        .h(sizes.item_height.px())
        .mx(sizes.item_margin_x.px())
        .px(sizes.item_padding_x.px())
        .rounded(sizes.item_radius.px())
        .whitespace_nowrap()
        .when(highlighted, |line| line.bg(rgb(theme.menu.highlight.0)).text_color(rgb(theme.menu.highlight_text.0)))
        .child(text)
        .children(key)
        .children(arrow);
    let name = line_target(menu.kind, shown.path);
    if shown.state == LineState::Pending {
        drop(target(format!("{name} (pending)"), |_, _, _| {}, cx));
        return line.opacity(sizes.pending_opacity.0);
    }
    if !shown.enabled {
        return line.text_color(rgb(theme.menu.disabled.0));
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
            // drawn over the parent, so no line of the parent crosses the submenu
            let over = gpui::point(sizes.item_margin_x.px() + sizes.border.px(), sizes.padding_y.px() + sizes.border.px());
            let submenu = beside::beside(panel(menu, lines, shown.path, theme, cx).flex_none().into_any_element(), over);
            let beside = deferred(submenu).with_priority(shown.path.len() + 1);
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
