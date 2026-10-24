//! The tab row's project groups, as the workspace keeps them: each project's folded flag and worktree order (saved
//! with the project), and its tab shown last.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use gpui::{Context, Pixels, Point};

use super::Workspace;
use crate::editor::Tab;
use crate::open_project::TabsFolded;
use crate::project::ProjectId;
use crate::saved_tabs::SavedTab;
use crate::tab_group_menu::GroupMenu;
use crate::tab_groups::{self, GroupState, Side, TabFacts, TabGroup};
use crate::tabs::TabId;

/// Where a drag in a project group's menu would land: a tab beside another of its worktree, a worktree beside
/// another of the project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuDrop {
    Tab { moved: TabId, beside: TabId, side: Side },
    Worktree { moved: PathBuf, beside: PathBuf, side: Side },
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
        let project = tab.project(self);
        let checkout = (!is_project_own(tab)).then(|| tab.checkout(self)).flatten();
        let main = checkout.as_ref().and_then(|checkout| self.checkout(checkout)).is_some_and(|found| found.is_main);
        let owner = match (&project, &checkout) {
            (_, Some(checkout)) => Some(owner_key(self.owner_of(checkout, tab.owner_path(self).as_deref()).module.as_deref(), &checkout.project)),
            (Some(project), None) => Some(owner_key(None, project)),
            (None, None) => None,
        };
        let worktrees = project.as_ref().and_then(|project| self.project(project)).is_some_and(|found| found.checkouts.len() > 1);
        TabFacts { id, project, checkout: checkout.map(|checkout| checkout.path), main, owner, worktrees }
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

    /// Opens project `project`'s group menu at `at`, under its head.
    pub fn open_group_menu(&mut self, project: ProjectId, at: Point<Pixels>, cx: &mut Context<Self>) {
        self.group_menu = Some(GroupMenu { project, at });
        self.group_menu_drop = None;
        cx.notify();
    }

    pub fn close_group_menu(&mut self, cx: &mut Context<Self>) {
        if self.group_menu.take().is_some() {
            self.group_menu_drop = None;
            cx.notify();
        }
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

    /// A drag in the group menu ended on a row: what it marked moves; the menu stays open.
    pub fn drop_in_group_menu(&mut self, cx: &mut Context<Self>) {
        let project = self.group_menu.as_ref().map(|menu| menu.project.clone());
        match (self.group_menu_drop.take(), project) {
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

    /// The favicon a tab's badge shows: its module's, else its project's.
    pub fn tab_badge(&self, tab: &Tab) -> Option<PathBuf> {
        match tab.checkout(self) {
            Some(checkout) => self.owner_of(&checkout, tab.owner_path(self).as_deref()).icon,
            None => tab.project(self).and_then(|project| self.project_icon(&project)),
        }
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
            self.tabs.move_tab(moved, gap);
            cx.notify();
        }
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
            open.set_worktree_order(order);
            self.schedule_save(cx);
            cx.notify();
        }
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
