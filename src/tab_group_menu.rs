//! A project group's menu, opened from its head's name: the group's tabs under a head per worktree (a rail in its
//! colour, the main checkout's without), the project's own tabs last; a tab drags within its worktree, a worktree's
//! head among the others.

use std::path::{Path, PathBuf};

use gpui::{
    AnyElement, Context, Div, DragMoveEvent, Pixels, Point, Render, SharedString, Stateful, Window, anchored, deferred, div, img, prelude::*, px, rgb,
};

use crate::colors::checkout_colour;
use crate::favicon_badge::{Badge, badged};
use crate::icons;
use crate::project::{CheckoutRef, ProjectId};
use crate::tab_groups::{self, Side, TabFacts};
use crate::tabs::TabId;
use crate::theme::Theme;
use crate::workspace::{Dropped, MenuDrop, Workspace};

/// A worktree's section of the menu, or the project's own tabs (`checkout` none).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    pub checkout: Option<PathBuf>,
    pub tabs: Vec<TabId>,
}

/// The open menu: whose group, and where (under the head's name).
#[derive(Debug, Clone, PartialEq)]
pub struct GroupMenu {
    pub project: ProjectId,
    pub at: Point<Pixels>,
}

/// The menu's sections: the group's worktree runs, in the same order as the tab row.
pub fn sections(tabs: &[TabFacts], order: &[PathBuf]) -> Vec<Section> {
    tab_groups::worktree_runs(tabs, order).into_iter().map(|(checkout, tabs)| Section { checkout, tabs }).collect()
}

/// The open group menu over a layer that closes it on a press outside.
pub fn render(workspace: &Workspace, theme: Theme, window: &Window, cx: &mut Context<Workspace>) -> Option<impl IntoElement> {
    let menu = workspace.group_menu()?.clone();
    let size = window.viewport_size();
    let panel = anchored().position(menu.at).snap_to_window().child(panel(&menu.project, workspace, theme, cx));
    let layer = div()
        .id("group-menu-layer")
        .w(size.width)
        .h(size.height)
        .occlude()
        .on_any_mouse_down(cx.listener(|workspace, _event, _window, cx| workspace.close_group_menu(cx)))
        .child(panel);
    Some(deferred(anchored().position(Point::default()).child(layer)).with_priority(1))
}

fn panel(project: &ProjectId, workspace: &Workspace, theme: Theme, cx: &mut Context<Workspace>) -> Stateful<Div> {
    let sizes = &theme.sizes.menu;
    let facts: Vec<TabFacts> = workspace.tab_group_facts().into_iter().filter(|fact| fact.project.as_ref() == Some(project)).collect();
    let order = workspace.worktree_order(project);
    let current = workspace.tab_groups(&workspace.tab_group_facts()).into_iter().find(|group| group.project.as_ref() == Some(project)).map(|group| group.current);
    let rows: Vec<AnyElement> = sections(&facts, &order)
        .into_iter()
        .flat_map(|section| {
            let head = section_head(project, section.checkout.as_deref(), workspace, cx);
            let rows = section.tabs.iter().filter_map(|id| tab_row(*id, Shown { section: section.checkout.as_deref(), current }, workspace, cx)).collect::<Vec<_>>();
            std::iter::once(head).chain(rows)
        })
        .collect();
    div()
        .id("group-menu")
        .min_w(sizes.min_width.px())
        .py(sizes.padding_y.px())
        .rounded(sizes.radius.px())
        .border(sizes.border.px())
        .border_color(rgb(theme.menu.border.0))
        .bg(rgb(theme.menu.background.0))
        .shadow_lg()
        .occlude()
        // a press inside the menu is not a press outside it
        .on_any_mouse_down(|_event, _window, cx| cx.stop_propagation())
        .text_size(sizes.text_size.px())
        .text_color(rgb(theme.menu.text.0))
        .children(rows)
}

/// A worktree's head (branch icon, its name, a rail in its colour), or the project's own tabs' (**Whole project**).
fn section_head(project: &ProjectId, checkout: Option<&Path>, workspace: &Workspace, cx: &mut Context<Workspace>) -> AnyElement {
    let theme = crate::theme::tabs(cx);
    let sizes = &theme.sizes;
    let (icon, label) = match checkout {
        Some(path) => (icons::BRANCH, workspace.checkout_label(&CheckoutRef { project: project.clone(), path: path.to_path_buf() })),
        None => (icons::PROJECT_FOLDER, "Whole project".to_owned()),
    };
    let colour = checkout
        .and_then(|path| workspace.checkout(&CheckoutRef { project: project.clone(), path: path.to_path_buf() }))
        .filter(|found| !found.is_main)
        .map(|found| checkout_colour(found, theme));
    let rail = colour.map(|colour| {
        div().absolute().left(px(10.)).top(sizes.tabs.rail_inset.px()).bottom(sizes.tabs.rail_inset.px()).w(sizes.tabs.rail_width.px()).rounded(px(1.)).bg(rgb(colour))
    });
    let head = div()
        .id(SharedString::from(format!("group-menu-head-{}", label)))
        .relative()
        .flex()
        .items_center()
        .gap(px(6.))
        .h(sizes.menu.item_height.px())
        .pl(px(20.))
        .pr(sizes.menu.item_padding_x.px())
        .text_size(px(12.))
        .text_color(rgb(theme.muted))
        .children(rail)
        .child(img(icons::embedded(theme.appearance, icon)).size(px(14.)))
        .child(label.clone())
        .children(drop_line(&MenuTarget::Worktree(checkout.map(Path::to_path_buf)), workspace, cx));
    let Some(path) = checkout else {
        return head.into_any_element();
    };
    let dragged = DraggedWorktree { checkout: path.to_path_buf(), label: label.into() };
    let target = path.to_path_buf();
    head.cursor_grab()
        .on_drag(dragged, |dragged, _offset, _window, cx| cx.new(|_| dragged.clone()))
        .on_drag_move(cx.listener(move |workspace, event: &DragMoveEvent<DraggedWorktree>, _window, cx| {
            if !event.bounds.contains(&event.event.position) {
                return;
            }
            // over its own head the worktree lands nowhere: the mark goes
            let moved = event.drag(cx).checkout.clone();
            let side = if event.event.position.y < event.bounds.center().y { Side::Before } else { Side::After };
            let drop = (moved != target).then(|| MenuDrop::Worktree { moved, beside: target.clone(), side });
            workspace.hover_group_menu_drop(drop, cx);
        }))
        .on_drop(cx.listener(|workspace, dragged: &DraggedWorktree, _window, cx| workspace.drop_in_group_menu(&Dropped::Worktree(dragged.checkout.clone()), cx)))
        .into_any_element()
}

/// Which section a row is in and the group's current tab, for a tab's row.
#[derive(Clone, Copy)]
struct Shown<'a> {
    section: Option<&'a Path>,
    current: Option<TabId>,
}

/// A tab's row: a tick on the group's current tab, its icon with its module's badge, its title; a click shows it.
fn tab_row(id: TabId, shown: Shown<'_>, workspace: &Workspace, cx: &mut Context<Workspace>) -> Option<AnyElement> {
    let theme = crate::theme::tabs(cx);
    let sizes = &theme.sizes.menu;
    let tab = workspace.tabs().get(id)?;
    let title: SharedString = tab.title(workspace).into();
    let badge = workspace.tab_module_badge(tab).map(|path| Badge { path, ring: theme.menu.background.0 });
    let tick = div().w(px(12.)).text_size(px(12.)).child(if shown.current == Some(id) { "✓" } else { "" });
    let section = shown.section.map(Path::to_path_buf);
    let dragged = DraggedMenuTab { id, checkout: section.clone(), title: title.clone() };
    let row = div()
        .id(("group-menu-tab", id.value()))
        .relative()
        .flex()
        .items_center()
        .gap(px(8.))
        .h(sizes.item_height.px())
        .mx(sizes.item_margin_x.px())
        .px(sizes.item_padding_x.px())
        .rounded(sizes.item_radius.px())
        .whitespace_nowrap()
        .cursor_pointer()
        .hover(|style| style.bg(rgb(theme.menu.highlight.0)).text_color(rgb(theme.menu.highlight_text.0)))
        .child(tick)
        .child(badged(tab.icon(workspace, theme), badge, theme))
        .child(title)
        .children(drop_line(&MenuTarget::Tab(id), workspace, cx))
        .on_click(crate::click_targets::target(format!("group-menu-tab:{}", id.value()), cx.listener(move |workspace, _event, window, cx| {
            workspace.close_group_menu(cx);
            workspace.activate_tab(id, window, cx);
        }), cx))
        .on_drag(dragged, |dragged, _offset, _window, cx| cx.new(|_| dragged.clone()))
        .on_drag_move(cx.listener(move |workspace, event: &DragMoveEvent<DraggedMenuTab>, _window, cx| {
            if !event.bounds.contains(&event.event.position) {
                return;
            }
            // over itself, or a row of another worktree, the tab lands nowhere: the mark goes
            let moved = event.drag(cx);
            let side = if event.event.position.y < event.bounds.center().y { Side::Before } else { Side::After };
            let drop = (moved.id != id && moved.checkout == section).then_some(MenuDrop::Tab { moved: moved.id, beside: id, side });
            workspace.hover_group_menu_drop(drop, cx);
        }))
        .on_drop(cx.listener(|workspace, dragged: &DraggedMenuTab, _window, cx| workspace.drop_in_group_menu(&Dropped::Tab(dragged.id), cx)));
    Some(row.into_any_element())
}

/// A row a drag may land beside.
enum MenuTarget {
    Tab(TabId),
    Worktree(Option<PathBuf>),
}

/// The accent line above or below the row a drag would land beside, while a drag goes on.
fn drop_line(target: &MenuTarget, workspace: &Workspace, cx: &Context<Workspace>) -> Option<Div> {
    let theme = crate::theme::tabs(cx);
    let mark = workspace.group_menu_drop().filter(|_| cx.has_active_drag())?;
    let side = match (mark, target) {
        (MenuDrop::Tab { beside, side, .. }, MenuTarget::Tab(id)) if beside == id => *side,
        (MenuDrop::Worktree { beside, side, .. }, MenuTarget::Worktree(Some(path))) if beside == path => *side,
        _ => return None,
    };
    let line = div().absolute().left(px(4.)).right(px(4.)).h(px(2.)).rounded(px(1.)).bg(rgb(theme.accent));
    Some(match side {
        Side::Before => line.top(px(-1.)),
        Side::After => line.bottom(px(-1.)),
    })
}

/// A tab dragged in the menu, within its worktree's section; it draws its title under the pointer.
#[derive(Clone)]
struct DraggedMenuTab {
    id: TabId,
    checkout: Option<PathBuf>,
    title: SharedString,
}

/// A worktree's head dragged among the project's; it draws its name under the pointer.
#[derive(Clone)]
struct DraggedWorktree {
    checkout: PathBuf,
    label: SharedString,
}

impl Render for DraggedMenuTab {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        drag_chip(self.title.clone(), cx)
    }
}

impl Render for DraggedWorktree {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        drag_chip(self.label.clone(), cx)
    }
}

fn drag_chip<V>(text: SharedString, cx: &Context<V>) -> Div {
    let theme = *cx.global::<Theme>();
    let sizes = &theme.sizes.menu;
    div()
        .px(sizes.item_padding_x.px())
        .h(sizes.item_height.px())
        .flex()
        .items_center()
        .rounded(sizes.item_radius.px())
        .bg(rgb(theme.menu.highlight.0))
        .text_color(rgb(theme.menu.highlight_text.0))
        .text_size(sizes.text_size.px())
        .opacity(theme.sizes.tabs.drag_chip_opacity.0)
        .child(text)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::project::ProjectId;
    use crate::tab_groups::TabFacts;
    use crate::tabs::TabId;

    fn fact(id: u64, checkout: Option<&str>, main: bool) -> TabFacts {
        TabFacts { id: TabId::for_tests(id), project: Some(ProjectId::new("a")), checkout: checkout.map(PathBuf::from), main, owner: None, worktrees: true }
    }

    #[test]
    fn should_list_sections_main_first_and_the_project_last() {
        // given
        let tabs = [fact(1, None, false), fact(2, Some("/x"), false), fact(3, Some("/a"), true), fact(4, Some("/x"), false)];

        // when
        let sections = sections(&tabs, &[]);

        // then
        let shown: Vec<(Option<PathBuf>, Vec<TabId>)> = sections.into_iter().map(|section| (section.checkout, section.tabs)).collect();
        assert_eq!(
            shown,
            [
                (Some(PathBuf::from("/a")), vec![TabId::for_tests(3)]),
                (Some(PathBuf::from("/x")), vec![TabId::for_tests(2), TabId::for_tests(4)]),
                (None, vec![TabId::for_tests(1)]),
            ]
        );
    }

    #[test]
    fn should_follow_the_worktree_order() {
        // given
        let tabs = [fact(1, Some("/a"), true), fact(2, Some("/x"), false)];

        // when
        let sections = sections(&tabs, &[PathBuf::from("/x")]);

        // then
        let checkouts: Vec<Option<PathBuf>> = sections.into_iter().map(|section| section.checkout).collect();
        assert_eq!(checkouts, [Some(PathBuf::from("/x")), Some(PathBuf::from("/a"))]);
    }
}
