//! The tab row's project groups, as the workspace keeps them: each project's folded flag and worktree order (saved
//! with the project), and its tab shown last.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use gpui::{App, Context, KeyDownEvent, Modifiers, Pixels, Point, SharedString, Window};

use super::{ProgramEnding, Workspace};
use crate::editor::Tab;
use crate::open_project::TabsFolded;
use crate::project::ProjectId;
use crate::saved_tabs::SavedTab;
use crate::script::keymap::{Action, Place};
use crate::tab_group_menu::{Ending, Frozen, GroupMenu, Picks, Section};
use crate::tab_groups::{self, GroupState, Side, TabFacts, TabGroup};
use crate::tabs::TabId;

/// Where a drag in a project group's menu would land: a tab beside another of its worktree, a worktree beside
/// another of the project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuDrop {
    Tab { moved: TabId, beside: TabId, side: Side },
    Worktree { moved: PathBuf, beside: PathBuf, side: Side },
}

/// What a drag in a project group's menu dropped: a tab, or a worktree's head (its checkout path).
pub enum Dropped {
    Tab(TabId),
    Worktree(PathBuf),
}

/// The mark a drop applies: the one made for the very item dropped; none for a mark left by another item.
pub fn accepted_drop(mark: Option<&MenuDrop>, dropped: &Dropped) -> Option<MenuDrop> {
    let accepted = match (mark?, dropped) {
        (MenuDrop::Tab { moved, .. }, Dropped::Tab(id)) => moved == id,
        (MenuDrop::Worktree { moved, .. }, Dropped::Worktree(path)) => moved == path,
        _ => false,
    };
    mark.filter(|_| accepted).cloned()
}

/// A worktree dragged in a project group's menu, to `beside`'s side of another (both checkout paths).
pub struct WorktreeMove<'a> {
    pub project: &'a ProjectId,
    pub moved: &'a Path,
    pub beside: (&'a Path, Side),
}

impl Workspace {
    /// What grouping needs of every tab of the row, in tab order.
    pub fn tab_group_facts(&self) -> Vec<TabFacts> {
        self.tabs.row().map(|(id, tab)| self.tab_facts(id, tab)).collect()
    }

    fn tab_facts(&self, id: TabId, tab: &Tab) -> TabFacts {
        // resolved once: where it runs, which a project's own tab has too (its last focused checkout)
        let resolved = tab.checkout(self);
        let project = resolved.as_ref().map(|checkout| checkout.project.clone()).or_else(|| tab.project(self));
        let checkout = resolved.clone().filter(|_| !is_project_own(tab));
        let main = checkout.as_ref().and_then(|checkout| self.checkout(checkout)).is_some_and(|found| found.is_main);
        // a project's own tab has no path in a module: its owner is the project
        let owner = resolved.as_ref().map(|at| self.owner_of(at, checkout.as_ref().and(tab.owner_path(self)).as_deref()));
        let key = match (&owner, &project) {
            (Some(owner), _) => Some(owner_key(owner.module.as_deref(), &owner.project)),
            (None, Some(project)) => Some(owner_key(None, project)),
            (None, None) => None,
        };
        let icon = match owner {
            Some(owner) => owner.icon,
            None => project.as_ref().and_then(|project| self.project_icon(project)),
        };
        let worktrees = project.as_ref().and_then(|project| self.project(project)).is_some_and(|found| found.checkouts.len() > 1);
        TabFacts { id, project, checkout: checkout.map(|checkout| checkout.path), main, owner: key, worktrees, icon }
    }

    /// The row's groups, from the tabs and each project's folded flag, worktree order and tab shown last.
    pub fn tab_groups(&self, facts: &[TabFacts]) -> Vec<TabGroup> {
        let folded: HashSet<ProjectId> =
            self.projects.iter().filter(|open| open.tabs_folded() == TabsFolded::Folded).map(|open| open.project.id.clone()).collect();
        let order: HashMap<ProjectId, Vec<PathBuf>> =
            self.projects.iter().map(|open| (open.project.id.clone(), open.worktree_order().to_vec())).collect();
        tab_groups::groups(facts, &GroupState { folded: &folded, order: &order, current: &self.group_current })
    }

    /// The open group menu.
    pub fn group_menu(&self) -> Option<&GroupMenu> {
        self.group_menu.as_ref()
    }

    /// Opens project `project`'s group menu at `at`, under its head; it has the keyboard while open (as the context
    /// menu).
    pub fn open_group_menu(&mut self, project: ProjectId, at: Point<Pixels>, window: &mut Window, cx: &mut Context<Self>) {
        self.group_menu = Some(GroupMenu::new(project, at));
        self.group_menu_drop = None;
        window.focus(&self.focus_handle, cx);
        cx.notify();
    }

    pub fn close_group_menu(&mut self, cx: &mut Context<Self>) {
        if self.group_menu.take().is_some() {
            self.group_menu_drop = None;
            cx.notify();
        }
    }

    /// Closes the group menu (Escape, a press outside it), giving the keyboard back to the shown tab.
    pub fn dismiss_group_menu(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.group_menu.is_some() {
            self.close_group_menu(cx);
            self.focus_active_tab(window, cx);
        }
    }

    /// The pointer on project `project`'s group head (`hovering`), or off it.
    pub fn hover_group_head(&mut self, project: &ProjectId, hovering: bool, cx: &mut Context<Self>) {
        let now = match hovering {
            true => Some(project.clone()),
            false => self.hovered_group_head.clone().filter(|was| was != project),
        };
        if self.hovered_group_head != now {
            self.hovered_group_head = now;
            cx.notify();
        }
    }

    /// Shift went down or up: a hovered head's X and the open menu's show Stop or Close.
    pub(super) fn redraw_held_ending(&mut self, cx: &mut Context<Self>) {
        if self.hovered_group_head.is_some() || self.group_menu.is_some() {
            cx.notify();
        }
    }

    /// Ends tabs `ids` as `ending` says.
    pub fn end_tabs(&mut self, ids: Vec<TabId>, ending: Ending, window: &mut Window, cx: &mut Context<Self>) {
        let program = match ending {
            Ending::Close => {
                self.close_tabs(ids, window, cx);
                return;
            }
            Ending::Stop => ProgramEnding::Stop,
            Ending::Kill => ProgramEnding::Kill,
        };
        for id in ids {
            self.stop_tab(id, program, window, cx);
        }
    }

    /// The head's X, or a middle click on the head: every tab of project `project`'s group.
    pub fn end_group(&mut self, project: &ProjectId, ending: Ending, window: &mut Window, cx: &mut Context<Self>) {
        let ids = self.project_facts(project).into_iter().map(|fact| fact.id).collect();
        self.end_tabs(ids, ending, window, cx);
    }

    /// A row's X, or a middle click on it: every picked row when it is one, else that row. The menu stays open, its
    /// rows put until the pointer leaves it.
    pub fn end_from_menu(&mut self, on: TabId, ending: Ending, window: &mut Window, cx: &mut Context<Self>) {
        let Some(menu) = &self.group_menu else { return };
        let targets = menu.picks.targets(on);
        self.end_menu_tabs(targets, ending, window, cx);
    }

    /// The picked rows, from the bar's Close and Stop.
    pub fn end_picked(&mut self, ending: Ending, window: &mut Window, cx: &mut Context<Self>) {
        let Some(menu) = &self.group_menu else { return };
        let targets = menu.picks.picked().to_vec();
        self.end_menu_tabs(targets, ending, window, cx);
    }

    fn end_menu_tabs(&mut self, ids: Vec<TabId>, ending: Ending, window: &mut Window, cx: &mut Context<Self>) {
        let frozen = self.group_menu.as_ref().map(|menu| menu.frozen.clone().unwrap_or_else(|| self.menu_rows_now(&menu.project)));
        self.end_tabs(ids, ending, window, cx);
        let picks = self.group_menu.as_ref().map(|menu| menu.picks.retained(|id| self.tabs.get(id).is_some()));
        // ending the tabs closed the menu with the project's last tab
        if let Some((menu, (frozen, picks))) = self.group_menu.as_mut().zip(frozen.zip(picks)) {
            menu.frozen = Some(frozen);
            menu.picks = picks;
        }
        cx.notify();
    }

    fn project_facts(&self, project: &ProjectId) -> Vec<TabFacts> {
        self.tab_group_facts().into_iter().filter(|fact| fact.project.as_ref() == Some(project)).collect()
    }

    /// Project `project`'s menu rows as they are, with their titles.
    fn menu_rows_now(&self, project: &ProjectId) -> Frozen {
        let facts = self.project_facts(project);
        let titles = facts.iter().filter_map(|fact| self.tabs.get(fact.id).map(|tab| (fact.id, SharedString::from(tab.title(self))))).collect();
        Frozen { sections: crate::tab_group_menu::sections(&facts, &self.worktree_order(project)), titles }
    }

    /// The menu's sections as drawn: frozen while closed rows stay put, else the group's.
    pub fn group_menu_sections(&self, project: &ProjectId) -> Vec<Section> {
        match self.group_menu.as_ref().and_then(|menu| menu.frozen.as_ref()) {
            Some(frozen) => frozen.sections.clone(),
            None => crate::tab_group_menu::sections(&self.project_facts(project), &self.worktree_order(project)),
        }
    }

    /// A closed row's title, as it was when the menu froze.
    pub fn group_menu_title(&self, id: TabId) -> Option<SharedString> {
        self.group_menu.as_ref()?.frozen.as_ref()?.titles.get(&id).cloned()
    }

    /// The menu's open rows in menu order, which a Shift+click ranges over.
    fn open_menu_rows(&self, project: &ProjectId) -> Vec<TabId> {
        self.group_menu_sections(project).into_iter().flat_map(|section| section.tabs).filter(|id| self.tabs.get(*id).is_some()).collect()
    }

    /// A click on a row: Ctrl (Cmd) picks it or drops it, Shift picks the range to it; a plain click shows the tab.
    pub fn click_menu_tab(&mut self, id: TabId, modifiers: &Modifiers, window: &mut Window, cx: &mut Context<Self>) {
        let Some(menu) = &self.group_menu else { return };
        let picks = match (modifiers.secondary(), modifiers.shift) {
            (true, _) => menu.picks.toggled(id),
            (false, true) => menu.picks.extended(&self.open_menu_rows(&menu.project), id),
            (false, false) => {
                self.close_group_menu(cx);
                return self.activate_tab(id, window, cx);
            }
        };
        self.set_menu_picks(picks, cx);
    }

    /// Ctrl+click on worktree `checkout`'s head (none: the project's own tabs): its open rows picked, or dropped.
    pub fn pick_menu_worktree(&mut self, checkout: Option<&Path>, cx: &mut Context<Self>) {
        let Some(menu) = &self.group_menu else { return };
        let section = self.group_menu_sections(&menu.project).into_iter().find(|section| section.checkout.as_deref() == checkout);
        let ids: Vec<TabId> = section.map(|section| section.tabs).unwrap_or_default().into_iter().filter(|id| self.tabs.get(*id).is_some()).collect();
        let picks = menu.picks.toggled_all(&ids);
        self.set_menu_picks(picks, cx);
    }

    fn set_menu_picks(&mut self, picks: Picks, cx: &mut Context<Self>) {
        if let Some(menu) = self.group_menu.as_mut() {
            menu.picks = picks;
            cx.notify();
        }
    }

    /// The pointer on menu row `id` (`hovered`), or off it.
    pub fn hover_menu_tab(&mut self, id: TabId, hovered: bool, cx: &mut Context<Self>) {
        let Some(menu) = self.group_menu.as_mut() else { return };
        let now = match hovered {
            true => Some(id),
            false => menu.hovered.filter(|was| *was != id),
        };
        if menu.hovered != now {
            menu.hovered = now;
            cx.notify();
        }
    }

    /// The pointer left the menu: the rows closed meanwhile go.
    pub fn leave_group_menu(&mut self, cx: &mut Context<Self>) {
        let Some(menu) = self.group_menu.as_mut() else { return };
        let (frozen, hovered) = (menu.frozen.take(), menu.hovered.take());
        if frozen.is_some() || hovered.is_some() {
            cx.notify();
        }
    }

    /// The open group menu's keys: Escape drops the picks, else closes it; the `"group-menu"` scope's Close, Stop and
    /// Kill end the picked rows, else the row under the pointer. Whether the key was the menu's.
    pub(super) fn group_menu_key(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let Some(menu) = &self.group_menu else { return false };
        if event.keystroke.key == "escape" {
            match menu.picks.is_empty() {
                true => self.dismiss_group_menu(window, cx),
                false => self.set_menu_picks(Picks::default(), cx),
            }
            return true;
        }
        let Some(ending) = bound_ending(event, cx) else { return false };
        // a held key ends nothing more
        if event.is_held {
            return true;
        }
        let targets = match menu.picks.is_empty() {
            true => menu.hovered.into_iter().collect(),
            false => menu.picks.picked().to_vec(),
        };
        self.end_menu_tabs(targets, ending, window, cx);
        true
    }

    /// Where a drag in the group menu would land, for the menu to mark.
    pub fn group_menu_drop(&self) -> Option<&MenuDrop> {
        self.group_menu_drop.as_ref()
    }

    pub fn hover_group_menu_drop(&mut self, drop: Option<MenuDrop>, cx: &mut Context<Self>) {
        if self.group_menu_drop != drop {
            self.group_menu_drop = drop;
            cx.notify();
        }
    }

    /// A drag in the group menu ended on a row with `dropped`: it moves where its own mark says, nothing else does;
    /// the menu stays open.
    pub fn drop_in_group_menu(&mut self, dropped: &Dropped, cx: &mut Context<Self>) {
        let project = self.group_menu.as_ref().map(|menu| menu.project.clone());
        let mark = self.group_menu_drop.take();
        match (accepted_drop(mark.as_ref(), dropped), project) {
            (Some(MenuDrop::Tab { moved, beside, side }), _) => self.move_tab_beside(moved, (beside, side), cx),
            (Some(MenuDrop::Worktree { moved, beside, side }), Some(project)) => {
                self.move_worktree_in_group(WorktreeMove { project: &project, moved: &moved, beside: (&beside, side) }, cx);
            }
            _ => cx.notify(),
        }
    }

    /// Project `id`'s worktree order in its group, as dragged.
    pub fn worktree_order(&self, id: &ProjectId) -> Vec<PathBuf> {
        self.open_project(id).map(|open| open.worktree_order().to_vec()).unwrap_or_default()
    }

    /// A tab's module's favicon, for the group menu (whose head names the project); none outside a module.
    pub fn tab_module_badge(&self, tab: &Tab) -> Option<PathBuf> {
        let checkout = tab.checkout(self)?;
        let owner = self.owner_of(&checkout, tab.owner_path(self).as_deref());
        owner.module.and(owner.icon)
    }

    /// The shown tab becomes its project's current tab, the one its group keeps out when folded.
    pub(super) fn record_group_current(&mut self) {
        let shown = self.tabs.active_id().zip(self.tabs.active()).and_then(|(id, tab)| tab.project(self).map(|project| (project, id)));
        if let Some((project, id)) = shown {
            self.group_current.insert(project, id);
        }
    }

    /// Folds project `id`'s group of tabs, or unfolds it.
    pub fn toggle_tab_group(&mut self, id: &ProjectId, cx: &mut Context<Self>) {
        let Some(open) = self.open_project_mut(id) else {
            return;
        };
        let folded = match open.tabs_folded() {
            TabsFolded::Folded => TabsFolded::Open,
            TabsFolded::Open => TabsFolded::Folded,
        };
        open.set_tabs_folded(folded);
        self.schedule_save(cx);
        cx.notify();
    }

    /// Moves tab `moved` to `beside`'s side of another tab in the row.
    pub fn move_tab_beside(&mut self, moved: TabId, beside: (TabId, Side), cx: &mut Context<Self>) {
        let order: Vec<TabId> = self.tabs.row().map(|(id, _)| id).collect();
        if let Some(gap) = tab_groups::move_gap(&order, moved, beside) {
            self.move_tab_in_group(moved, gap, cx);
        }
    }

    /// Moves tab `moved` into row gap `gap` (as `Tabs::move_tab` counts them), keeping the groups' order: a drag
    /// reorders tabs within their project's group.
    pub fn move_tab_in_group(&mut self, moved: TabId, gap: usize, cx: &mut Context<Self>) {
        let before = self.tab_group_facts();
        self.tabs.move_tab(moved, gap);
        let after: Vec<TabId> = self.tabs.row().map(|(id, _)| id).collect();
        for (place, id) in tab_groups::keep_group_order(&before, &after).into_iter().enumerate() {
            // every tab before `place` is in its place already, so gap `place` is right before it
            self.tabs.move_tab(id, place);
        }
        cx.notify();
    }

    /// Moves a worktree among its project's in the group's order; the order then lists every worktree the group
    /// shows, so the dragged place holds.
    pub fn move_worktree_in_group(&mut self, moved: WorktreeMove<'_>, cx: &mut Context<Self>) {
        let facts = self.tab_group_facts();
        let saved = self.open_project(moved.project).map(|open| open.worktree_order().to_vec()).unwrap_or_default();
        let shown = tab_groups::worktree_runs(facts.iter().filter(|fact| fact.project.as_ref() == Some(moved.project)), &saved);
        let order: Vec<PathBuf> = shown.into_iter().filter_map(|(checkout, _)| checkout).collect();
        let order = tab_groups::placed(&order, &moved.moved.to_path_buf(), (&moved.beside.0.to_path_buf(), moved.beside.1));
        if let Some(open) = self.open_project_mut(moved.project) {
            open.set_worktree_order(tab_groups::merged_order(&saved, &order));
            self.schedule_save(cx);
            cx.notify();
        }
    }
}

/// What a key bound in the `"group-menu"` scope does to tabs, if anything.
fn bound_ending(event: &KeyDownEvent, cx: &App) -> Option<Ending> {
    match crate::script::state::bound_action(event, Place::GroupMenu, cx)? {
        Action::Builtin(builtin) => Ending::bound(builtin),
        _ => None,
    }
}

/// Tabs of the whole project, in no checkout: scratches, the Git log, commits, the project overview.
fn is_project_own(tab: &Tab) -> bool {
    matches!(
        tab,
        Tab::Scratch(_)
            | Tab::GitLog(_)
            | Tab::Commit(_)
            | Tab::ProjectOverview(_)
            | Tab::Unloaded(SavedTab::Scratch { .. } | SavedTab::GitLog { .. } | SavedTab::Commit { .. } | SavedTab::ProjectOverview { .. })
    )
}

/// An owner's key: the project, and its module folder when in one.
fn owner_key(module: Option<&str>, project: &ProjectId) -> String {
    match module {
        Some(module) => format!("{}/{module}", project.as_str()),
        None => project.as_str().to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_apply_the_mark_made_for_the_tab_dropped() {
        // given
        let (moved, beside) = (TabId::for_tests(1), TabId::for_tests(2));
        let mark = MenuDrop::Tab { moved, beside, side: Side::After };

        // when
        let accepted = (accepted_drop(Some(&mark), &Dropped::Tab(moved)), accepted_drop(Some(&mark), &Dropped::Tab(beside)));

        // then
        assert_eq!(accepted, (Some(mark.clone()), None));
    }

    #[test]
    fn should_not_apply_a_worktree_mark_to_a_dropped_tab() {
        // given
        let mark = MenuDrop::Worktree { moved: PathBuf::from("/x"), beside: PathBuf::from("/a"), side: Side::Before };

        // when
        let accepted = (accepted_drop(Some(&mark), &Dropped::Tab(TabId::for_tests(1))), accepted_drop(Some(&mark), &Dropped::Worktree(PathBuf::from("/x"))));

        // then
        assert_eq!(accepted, (None, Some(mark.clone())));
    }
}
