//! The tab row's project groups, as the workspace keeps them: each project's folded flag and worktree order (saved
//! with the project), and its tab shown last.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use gpui::Context;

use super::Workspace;
use crate::editor::Tab;
use crate::open_project::TabsFolded;
use crate::project::ProjectId;
use crate::saved_tabs::SavedTab;
use crate::tab_groups::{self, GroupState, Side, TabFacts, TabGroup};
use crate::tabs::TabId;

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
        TabFacts { id, project, checkout: checkout.map(|checkout| checkout.path), main, owner }
    }

    /// The row's groups, from the tabs and each project's folded flag, worktree order and tab shown last.
    pub fn tab_groups(&self, facts: &[TabFacts]) -> Vec<TabGroup> {
        let folded: HashSet<ProjectId> =
            self.projects.iter().filter(|open| open.tabs_folded() == TabsFolded::Folded).map(|open| open.project.id.clone()).collect();
        let order: HashMap<ProjectId, Vec<PathBuf>> =
            self.projects.iter().map(|open| (open.project.id.clone(), open.worktree_order().to_vec())).collect();
        tab_groups::groups(facts, &GroupState { folded: &folded, order: &order, current: &self.group_current })
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
        let placed = tab_groups::placed(&order, &moved, (&beside.0, beside.1));
        if let Some(gap) = placed.iter().position(|id| *id == moved) {
            // move_tab counts gaps in the row before the move: past the tab's own place, one more
            let from = order.iter().position(|id| *id == moved).unwrap_or(gap);
            self.tabs.move_tab(moved, if gap > from { gap + 1 } else { gap });
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
