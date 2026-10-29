//! A project group's menu, opened from its head's name: the group's tabs under a head per worktree (a rail in its
//! colour, the main checkout's without), the project's own tabs last; a tab drags within its worktree, a worktree's
//! head among the others.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use gpui::{
    AnyElement, Context, Div, DragMoveEvent, Modifiers, MouseButton, Pixels, Point, Render, SharedString, Stateful, Window, anchored, deferred, div, img, prelude::*, px, rgb,
};

use crate::colors::checkout_colour;
use crate::favicon_badge::{Badge, badged};
use crate::icons;
use crate::project::{CheckoutRef, ProjectId};
use crate::script::keymap::Builtin;
use crate::tab_groups::{self, Side, TabFacts};
use crate::tabs::TabId;
use crate::theme::Theme;
use crate::click_targets::target;
use crate::workspace::{Dropped, MenuDrop, Workspace};

/// A worktree's section of the menu, or the project's own tabs (`checkout` none).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    pub checkout: Option<PathBuf>,
    pub tabs: Vec<TabId>,
}

/// The open menu: whose group, where (under the head's name), its picked rows, the row under the pointer, and the
/// rows as they were when one closed while it was open.
#[derive(Debug, Clone, PartialEq)]
pub struct GroupMenu {
    pub project: ProjectId,
    pub at: Point<Pixels>,
    pub picks: Picks,
    pub hovered: Option<TabId>,
    pub frozen: Option<Frozen>,
}

impl GroupMenu {
    pub fn new(project: ProjectId, at: Point<Pixels>) -> Self {
        Self { project, at, picks: Picks::default(), hovered: None, frozen: None }
    }
}

/// The menu's rows when one first closed while it was open: they stay put, a closed one struck through with the title
/// it had, until the pointer leaves the menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frozen {
    pub sections: Vec<Section>,
    pub titles: HashMap<TabId, SharedString>,
}

/// What X, a middle click or a key does to tabs: Close as Ctrl+Shift+W (a process goes on in Processes), Stop and
/// Kill as Ctrl+Shift+Q (a process tab's process ends for good; another tab just closes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ending {
    Close,
    Stop,
    Kill,
}

impl Ending {
    /// Stop while Shift is held, else Close.
    pub fn held(modifiers: &Modifiers) -> Self {
        if modifiers.shift {
            return Self::Stop;
        }
        Self::Close
    }

    /// The tab action a key bound in the `"group-menu"` scope runs.
    pub fn bound(builtin: Builtin) -> Option<Self> {
        match builtin {
            Builtin::CloseTab => Some(Self::Close),
            Builtin::StopProgram => Some(Self::Stop),
            Builtin::KillProgram => Some(Self::Kill),
            _ => None,
        }
    }
}

/// The menu's picked rows (Ctrl+click, Shift+click) and the row a Shift+click ranges from.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Picks {
    picked: Vec<TabId>,
    anchor: Option<TabId>,
}

impl Picks {
    pub fn picked(&self) -> &[TabId] {
        &self.picked
    }

    pub fn contains(&self, id: TabId) -> bool {
        self.picked.contains(&id)
    }

    pub fn is_empty(&self) -> bool {
        self.picked.is_empty()
    }

    /// Ctrl+click on row `id`: picked, or dropped when it was; it becomes the anchor.
    pub fn toggled(&self, id: TabId) -> Self {
        let picked = match self.contains(id) {
            true => self.picked.iter().copied().filter(|picked| *picked != id).collect(),
            false => self.picked.iter().copied().chain([id]).collect(),
        };
        Self { picked, anchor: Some(id) }
    }

    /// Shift+click on row `id`: the rows of `order` from the anchor to it; with no anchor in `order`, `id` alone, which
    /// becomes the anchor.
    pub fn extended(&self, order: &[TabId], id: TabId) -> Self {
        let from = self.anchor.and_then(|anchor| order.iter().position(|row| *row == anchor));
        let to = order.iter().position(|row| *row == id);
        match from.zip(to) {
            Some((from, to)) => Self { picked: order[from.min(to)..=from.max(to)].to_vec(), anchor: self.anchor },
            None => Self { picked: vec![id], anchor: Some(id) },
        }
    }

    /// Ctrl+click on a worktree's head with rows `ids`: all picked, or all dropped when all were.
    pub fn toggled_all(&self, ids: &[TabId]) -> Self {
        let rest = self.picked.iter().copied().filter(|picked| !ids.contains(picked));
        let picked = match ids.iter().all(|id| self.contains(*id)) {
            true => rest.collect(),
            false => rest.chain(ids.iter().copied()).collect(),
        };
        Self { picked, anchor: self.anchor }
    }

    /// What X or a middle click on row `on` ends: every picked row when `on` is one, else `on` alone.
    pub fn targets(&self, on: TabId) -> Vec<TabId> {
        match self.contains(on) {
            true => self.picked.clone(),
            false => vec![on],
        }
    }

    /// The picks still open.
    pub fn retained(&self, open: impl Fn(TabId) -> bool) -> Self {
        Self { picked: self.picked.iter().copied().filter(|id| open(*id)).collect(), anchor: self.anchor.filter(|anchor| open(*anchor)) }
    }
}

/// The menu's sections: the group's worktree runs, in the same order as the tab row.
pub fn sections(tabs: &[TabFacts], order: &[PathBuf]) -> Vec<Section> {
    tab_groups::worktree_runs(tabs, order).into_iter().map(|(checkout, tabs)| Section { checkout, tabs }).collect()
}

/// The open group menu over a layer that closes it on a press outside.
pub fn render(workspace: &Workspace, window: &Window, cx: &mut Context<Workspace>) -> Option<impl IntoElement> {
    let menu = workspace.group_menu()?.clone();
    let facts = workspace.tab_group_facts();
    // a group whose last tab closed has nothing to list: its menu is gone with it
    facts.iter().find(|fact| fact.project.as_ref() == Some(&menu.project))?;
    let size = window.viewport_size();
    let held = Ending::held(&window.modifiers());
    let panel = anchored().position(menu.at).snap_to_window().child(panel(&menu, (&facts, held), workspace, cx));
    let layer = div()
        .id("group-menu-layer")
        .w(size.width)
        .h(size.height)
        .occlude()
        .on_any_mouse_down(cx.listener(|workspace, _event, window, cx| workspace.dismiss_group_menu(window, cx)))
        .child(panel);
    Some(deferred(anchored().position(Point::default()).child(layer)).with_priority(1))
}

/// The menu's panel; `held` is what a row's X does (Stop while Shift is down). The pointer leaving it lets the rows
/// closed meanwhile go.
fn panel(menu: &GroupMenu, (all, held): (&[TabFacts], Ending), workspace: &Workspace, cx: &mut Context<Workspace>) -> Stateful<Div> {
    let theme = crate::theme::tabs(cx);
    let sizes = &theme.sizes.menu;
    let project = &menu.project;
    let current = workspace.tab_groups(all).into_iter().find(|group| group.project.as_ref() == Some(project)).map(|group| group.current);
    let rows: Vec<AnyElement> = workspace
        .group_menu_sections(project)
        .into_iter()
        .flat_map(|section| {
            let head = section_head(project, section.checkout.as_deref(), workspace, cx);
            let shown = Shown { section: section.checkout.as_deref(), current, picks: &menu.picks, held };
            let rows = section.tabs.iter().filter_map(|id| tab_row(*id, shown, workspace, cx)).collect::<Vec<_>>();
            std::iter::once(head).chain(rows)
        })
        .collect();
    let bar = (!menu.picks.is_empty()).then(|| picks_bar(menu.picks.picked().len(), cx));
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
        .on_hover(cx.listener(|workspace, hovering: &bool, _window, cx| {
            if !*hovering {
                workspace.leave_group_menu(cx);
            }
        }))
        .children(rows)
        .children(bar)
}

/// The bar under the rows while some are picked: how many, Close and Stop (as the keys).
fn picks_bar(picked: usize, cx: &mut Context<Workspace>) -> Div {
    let theme = crate::theme::tabs(cx);
    let sizes = &theme.sizes.menu;
    let action = |ending: Ending, cx: &mut Context<Workspace>| {
        let (icon, label, name) = match ending {
            Ending::Close => (icons::CLOSE, "Close", "group-menu-close-picked"),
            Ending::Stop | Ending::Kill => (icons::STOP, "Stop", "group-menu-stop-picked"),
        };
        div()
            .id(name)
            .flex()
            .items_center()
            .gap(px(4.))
            .h(px(24.))
            .px(px(8.))
            .rounded(sizes.item_radius.px())
            .text_color(rgb(theme.menu.text.0))
            .cursor_pointer()
            .hover(|style| style.bg(rgb(theme.menu.highlight.0)).text_color(rgb(theme.menu.highlight_text.0)))
            .child(img(icons::embedded(theme.appearance, icon)).size(px(12.)))
            .child(label)
            .on_click(target(name.to_owned(), cx.listener(move |workspace, _event, window, cx| workspace.end_picked(ending, window, cx)), cx))
    };
    div()
        .flex()
        .items_center()
        .gap(px(4.))
        .mt(px(4.))
        .pt(px(4.))
        .pl(px(8.) + sizes.item_margin_x.px())
        .pr(sizes.item_margin_x.px())
        .border_t(px(1.))
        .border_color(rgb(theme.menu.border.0))
        .text_size(px(12.))
        .text_color(rgb(theme.muted))
        .child(div().flex_1().child(format!("{picked} selected")))
        .child(action(Ending::Close, cx))
        .child(action(Ending::Stop, cx))
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
        .id(SharedString::from(format!("group-menu-head-{}", checkout.map_or_else(|| "whole-project".to_owned(), |path| path.display().to_string()))))
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
    // Ctrl+click picks the section's rows
    let picked = checkout.map(Path::to_path_buf);
    let pick = cx.listener(move |workspace, _event: &gpui::ClickEvent, _window, cx| workspace.pick_menu_worktree(picked.as_deref(), cx));
    drop(target(format!("group-menu-pick-worktree:{}", checkout.map_or_else(|| "whole-project".to_owned(), |path| path.display().to_string())), pick, cx));
    let picked = checkout.map(Path::to_path_buf);
    let head = head.on_click(cx.listener(move |workspace, event: &gpui::ClickEvent, _window, cx| {
        if event.modifiers().secondary() {
            workspace.pick_menu_worktree(picked.as_deref(), cx);
        }
    }));
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

/// Which section a row is in, the group's current tab, the picked rows, and what a row's X does.
#[derive(Clone, Copy)]
struct Shown<'a> {
    section: Option<&'a Path>,
    current: Option<TabId>,
    picks: &'a Picks,
    held: Ending,
}

/// A row closed while the menu was open: its title as it was, struck through; nothing on it reacts.
fn gone_row(id: TabId, workspace: &Workspace, cx: &mut Context<Workspace>) -> Option<AnyElement> {
    let theme = crate::theme::tabs(cx);
    let sizes = &theme.sizes.menu;
    let title = workspace.group_menu_title(id)?;
    let row = div()
        .id(("group-menu-gone", id.value()))
        .flex()
        .items_center()
        .gap(px(8.))
        .h(sizes.item_height.px())
        .mx(sizes.item_margin_x.px())
        .px(sizes.item_padding_x.px())
        .whitespace_nowrap()
        .opacity(0.4)
        .child(div().w(px(12.)))
        .child(div().size(theme.sizes.icon.px()))
        .child(div().line_through().child(title));
    Some(row.into_any_element())
}

/// A tab's row: a tick on the group's current tab, its icon with its module's badge, its title, its X (faint until the
/// row is hovered); picked rows filled. A click shows it, Ctrl+click picks it, Shift+click picks the range to it; X or
/// a middle click ends it (with the picked rows when it is one), Shift stopping.
fn tab_row(id: TabId, shown: Shown<'_>, workspace: &Workspace, cx: &mut Context<Workspace>) -> Option<AnyElement> {
    let theme = crate::theme::tabs(cx);
    let sizes = &theme.sizes.menu;
    let Some(tab) = workspace.tabs().get(id) else {
        return gone_row(id, workspace, cx);
    };
    let title: SharedString = tab.title(workspace).into();
    let badge = workspace.tab_module_badge(tab).map(|path| Badge { path, ring: theme.menu.background.0 });
    let tick = div().w(px(12.)).text_size(px(12.)).child(if shown.current == Some(id) { "✓" } else { "" });
    let section = shown.section.map(Path::to_path_buf);
    let dragged = DraggedMenuTab { id, checkout: section.clone(), title: title.clone() };
    let row_group = SharedString::from(format!("group-menu-tab-{}", id.value()));
    let row = div()
        .id(("group-menu-tab", id.value()))
        .group(row_group.clone())
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
        .when(shown.picks.contains(id), |row| row.bg(rgb(theme.menu.highlight.0)).text_color(rgb(theme.menu.highlight_text.0)))
        .hover(|style| style.bg(rgb(theme.menu.highlight.0)).text_color(rgb(theme.menu.highlight_text.0)))
        .child(tick)
        .child(badged(tab.icon(workspace, theme), badge, theme))
        .child(div().flex_1().child(title))
        .child(row_end(id, (row_group, shown.held), cx))
        .children(drop_line(&MenuTarget::Tab(id), workspace, cx))
        .on_hover(cx.listener(move |workspace, hovering: &bool, _window, cx| workspace.hover_menu_tab(id, *hovering, cx)))
        .on_mouse_down(MouseButton::Middle, |_event, _window, cx| cx.stop_propagation())
        .on_aux_click(cx.listener(move |workspace, event: &gpui::ClickEvent, window, cx| {
            if event.is_middle_click() {
                workspace.end_from_menu(id, Ending::held(&event.modifiers()), window, cx);
            }
        }))
        .on_click(target(format!("group-menu-tab:{}", id.value()), cx.listener(move |workspace, event: &gpui::ClickEvent, window, cx| {
            workspace.click_menu_tab(id, &event.modifiers(), window, cx);
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

/// A row's X: ends it (with the picked rows when it is one), closing, with Shift (`held`) stopping; faint until the row
/// (`row_group`) is hovered. Registers the row's MCP targets for what needs a modifier.
fn row_end(id: TabId, (row_group, held): (SharedString, Ending), cx: &mut Context<Workspace>) -> Stateful<Div> {
    let theme = crate::theme::tabs(cx);
    let (icon, verb) = match held {
        Ending::Close => (icons::CLOSE, "Close"),
        Ending::Stop | Ending::Kill => (icons::STOP, "Stop"),
    };
    // for MCP's click, which cannot hold keys
    let modified = |name: &str, modifiers: Modifiers, cx: &mut Context<Workspace>| {
        let listener = cx.listener(move |workspace, _event, window, cx| workspace.click_menu_tab(id, &modifiers, window, cx));
        drop(target(format!("{name}:{}", id.value()), listener, cx));
    };
    modified("group-menu-pick", Modifiers::secondary_key(), cx);
    modified("group-menu-extend", Modifiers::shift(), cx);
    for (name, ending) in [("group-menu-close", Ending::Close), ("group-menu-stop", Ending::Stop)] {
        drop(target(format!("{name}:{}", id.value()), cx.listener(move |workspace, _event, window, cx| workspace.end_from_menu(id, ending, window, cx)), cx));
    }
    div()
        .id(("group-menu-end", id.value()))
        .flex_none()
        .ml(px(8.))
        .p(px(3.))
        .rounded(px(4.))
        .opacity(0.5)
        .group_hover(row_group, |style| style.opacity(1.))
        .hover(|style| style.bg(rgb(theme.hover)))
        .child(img(icons::embedded(theme.appearance, icon)).size(px(12.)))
        .tooltip(crate::tooltip::text_tooltip(format!("{verb} (middle click too; Shift: Stop)")))
        // a press on the X starts no drag of the row
        .on_mouse_down(MouseButton::Left, |_event, _window, cx| cx.stop_propagation())
        .on_click(cx.listener(move |workspace, event: &gpui::ClickEvent, window, cx| {
            cx.stop_propagation();
            workspace.end_from_menu(id, Ending::held(&event.modifiers()), window, cx);
        }))
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
        TabFacts { id: TabId::for_tests(id), project: Some(ProjectId::new("a")), checkout: checkout.map(PathBuf::from), main, owner: None, worktrees: true, icon: None }
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

    fn ids(values: &[u64]) -> Vec<TabId> {
        values.iter().map(|value| TabId::for_tests(*value)).collect()
    }

    #[test]
    fn should_pick_a_row_on_ctrl_click_and_drop_it_on_the_next() {
        // given
        let id = TabId::for_tests(2);

        // when
        let picked = Picks::default().toggled(id);
        let dropped = picked.toggled(id);

        // then
        assert_eq!((picked.picked(), dropped.picked()), (&ids(&[2])[..], &[][..]));
    }

    #[test]
    fn should_pick_the_shift_clicked_row_alone_without_an_anchor() {
        // given
        let order = ids(&[1, 2, 3, 4]);

        // when
        let first = Picks::default().extended(&order, TabId::for_tests(2));
        let range = first.extended(&order, TabId::for_tests(4));

        // then
        assert_eq!((first.picked(), range.picked()), (&ids(&[2])[..], &ids(&[2, 3, 4])[..]));
    }

    #[test]
    fn should_pick_the_range_upwards_from_the_ctrl_clicked_row() {
        // given
        let order = ids(&[1, 2, 3, 4]);
        let picks = Picks::default().toggled(TabId::for_tests(3));

        // when
        let range = picks.extended(&order, TabId::for_tests(1));

        // then
        assert_eq!(range.picked(), &ids(&[1, 2, 3])[..]);
    }

    #[test]
    fn should_pick_a_worktrees_rows_and_drop_them_when_all_were_picked() {
        // given
        let worktree = ids(&[2, 3]);
        let picks = Picks::default().toggled(TabId::for_tests(1)).toggled(TabId::for_tests(2));

        // when
        let all = picks.toggled_all(&worktree);
        let none = all.toggled_all(&worktree);

        // then
        assert_eq!((all.picked(), none.picked()), (&ids(&[1, 2, 3])[..], &ids(&[1])[..]));
    }

    #[test]
    fn should_end_every_picked_row_from_a_picked_one_and_only_the_row_from_another() {
        // given
        let picks = Picks::default().toggled(TabId::for_tests(1)).toggled(TabId::for_tests(3));

        // when
        let targets = (picks.targets(TabId::for_tests(3)), picks.targets(TabId::for_tests(2)));

        // then
        assert_eq!(targets, (ids(&[1, 3]), ids(&[2])));
    }

    #[test]
    fn should_keep_only_the_open_picks_and_anchor() {
        // given
        let picks = Picks::default().toggled(TabId::for_tests(1)).toggled(TabId::for_tests(3));

        // when
        let kept = picks.retained(|id| id == TabId::for_tests(1));

        // then: the anchor (3) is gone, so a Shift+click picks its row alone
        assert_eq!((kept.picked(), kept.extended(&ids(&[1, 2]), TabId::for_tests(2)).picked()), (&ids(&[1])[..], &ids(&[2])[..]));
    }

    #[test]
    fn should_stop_with_shift_and_close_without() {
        // given
        let shift = Modifiers { shift: true, ..Modifiers::default() };

        // when
        let endings = (Ending::held(&shift), Ending::held(&Modifiers::default()));

        // then
        assert_eq!(endings, (Ending::Stop, Ending::Close));
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
