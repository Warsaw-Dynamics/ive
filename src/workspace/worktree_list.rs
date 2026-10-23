//! The project page's Worktrees list, as the workspace drives it: its groups from the project's checkouts and
//! branches, its keys (taken before the filter field sees them, as the hub's), and what a row does.

use gpui::{App, Context, Div, Entity, Keystroke, Window};

use super::Workspace;
use crate::editor::Tab;
use crate::open_project::OpenProject;
use crate::overview::worktree_list::WorktreeList;
use crate::overview::worktree_list_view::{Drawn, DrawnGroup, DrawnRow, render_worktree_list};
use crate::overview::worktree_rows::{Candidate, Group, GroupKind, KeyEffect, ListInput, ListKey, RowId, groups, key_effect, kept_place, line_of, lines};
use crate::overview::{ProjectSection, project_card_tags};
use crate::age::Clock;
use crate::colors::checkout_colour;
use crate::icons;
use crate::paths::{card_folder, tilde_home};
use crate::project::{Checkout, CheckoutRef, ProjectId, ProjectKind};
use crate::theme::Theme;

impl Workspace {
    /// The Worktrees list of `project`, or the main checkout's row alone for a plain folder (`list` is `None`).
    pub(crate) fn render_worktree_list(&self, project: &ProjectId, list: Option<&Entity<WorktreeList>>, theme: Theme, cx: &mut Context<Self>) -> Div {
        let drawn = self.drawn_list(project, list.map(|list| list.read(cx)), cx);
        render_worktree_list(drawn, list, theme, cx)
    }

    /// The list's groups as they are now.
    fn list_groups(&self, project: &ProjectId, list: Option<&WorktreeList>, cx: &App) -> Vec<Group> {
        let Some(open) = self.open_project(project) else {
            return Vec::new();
        };
        let query = list.map(|list| list.query(cx)).unwrap_or_default();
        let text = |checkout: &Checkout| format!("{} {} {}", checkout.label, checkout.branch.as_deref().unwrap_or_default(), checkout.path.display());
        let candidate = |checkout: &Checkout| Candidate::new(RowId::Checkout(checkout.path.clone()), &text(checkout));
        let used = open.listed_checkouts().map(|checkout| (checkout, self.worktree_use.last_used(&checkout.path, &self.history))).collect();
        let shown: Vec<Candidate> = by_last_use(used).into_iter().map(candidate).collect();
        let hidden: Vec<Candidate> = open.hidden_checkouts().map(candidate).collect();
        let input = ListInput {
            shown: &shown,
            hidden: &hidden,
            branches: list.map_or(&[], WorktreeList::branch_candidates),
            query: &query,
            branches_visible: list.map_or(0, WorktreeList::branches_visible),
            git: list.is_some(),
        };
        groups(&input)
    }

    fn drawn_list(&self, project: &ProjectId, list: Option<&WorktreeList>, cx: &App) -> Drawn {
        let groups = self.list_groups(project, list, cx);
        let selected = list.and_then(|list| list.selected(&groups));
        let hidden_open = list.is_none_or(WorktreeList::hidden_open);
        let headers = groups.iter().map(|group| drawn_group(group, list)).collect();
        let error = list.and_then(|list| list.error().or(list.load_error()).map(str::to_owned));
        let escape = if selected.is_some() { "back to typing" } else { "clear" };
        let strip = match list {
            Some(_) => vec![("↑↓", "move"), ("Enter", "show in the tree · add the branch's worktree"), ("Space", "hide or show · add the worktree, staying put"), ("type", "filter"), ("Esc", escape)],
            None => Vec::new(),
        };
        Drawn { project: project.clone(), lines: lines(&groups, hidden_open), groups, headers, selected, hidden_open, error, strip }
    }

    /// The rows of `ids` as drawn, `None` for one no longer there: only the rows in view are drawn.
    pub(crate) fn drawn_rows(&self, project: &ProjectId, list: Option<&WorktreeList>, ids: &[&RowId], theme: Theme) -> Vec<Option<DrawnRow>> {
        let Some(open) = self.open_project(project) else {
            return ids.iter().map(|_| None).collect();
        };
        ids.iter().map(|row| self.drawn_row(row, open, list, theme)).collect()
    }

    fn drawn_row(&self, row: &RowId, open: &OpenProject, list: Option<&WorktreeList>, theme: Theme) -> Option<DrawnRow> {
        let clock = Clock::system();
        match row {
            RowId::Checkout(path) => {
                let checkout = open.project.checkouts.iter().find(|checkout| &checkout.path == path)?;
                let hidden = open.is_hidden(path);
                let running = self.processes().by_checkout(path).len();
                let plain = open.project.kind == ProjectKind::Plain;
                let detail = match (checkout.is_main, checkout.branch.as_ref().filter(|branch| **branch != checkout.label)) {
                    (true, _) if plain => tilde_home(path),
                    (true, _) => format!("main checkout · {}", tilde_home(path)),
                    (false, Some(branch)) => format!("{branch} · {}", card_folder(path, &open.project.root, dirs::home_dir().as_deref())),
                    (false, None) => card_folder(path, &open.project.root, dirs::home_dir().as_deref()),
                };
                let action = match (checkout.is_main, hidden) {
                    (true, _) => None,
                    (false, true) => Some("Show"),
                    (false, false) => Some("Hide"),
                };
                let used = self.worktree_use.last_used(path, &self.history);
                Some(DrawnRow {
                    id: row.clone(),
                    colour: Some(checkout_colour(checkout, theme)),
                    icon: if plain { icons::PROJECT_FOLDER } else { icons::BRANCH },
                    title: if plain { open.project.name.clone() } else { checkout.label.clone() },
                    detail,
                    tags: project_card_tags(checkout, open.project.kind, 0),
                    running: (running > 0).then(|| format!("● {running} running")),
                    age: used.map_or_else(|| "never".to_owned(), |used| clock.age(used)),
                    faded_icon: false,
                    hidden,
                    action,
                })
            }
            RowId::Branch(name) => {
                let list = list?;
                let branch = list.branch(name)?;
                let (detail, remote) = match &branch.name {
                    crate::git::refs::BranchName::Local(_) => ("local".to_owned(), false),
                    crate::git::refs::BranchName::Remote { remote, .. } => (format!("only on {remote}"), true),
                };
                let adding = list.adding() == Some(name.as_str());
                Some(DrawnRow {
                    id: row.clone(),
                    colour: None,
                    icon: icons::BRANCH,
                    title: name.clone(),
                    detail,
                    tags: Vec::new(),
                    running: None,
                    age: if adding { "Creating…".to_owned() } else { clock.age(branch.when.seconds) },
                    faded_icon: remote,
                    hidden: false,
                    action: Some("+ Worktree"),
                })
            }
        }
    }

    /// The shown tab's Worktrees list, when it has the keyboard's tab.
    fn shown_list(&self) -> Option<(ProjectId, Entity<WorktreeList>)> {
        let Tab::ProjectOverview(overview) = self.tabs.get(self.tabs.active_id()?)? else {
            return None;
        };
        let views = overview.git.as_ref().filter(|_| overview.section == ProjectSection::Worktrees)?;
        Some((overview.project.clone(), views.list.clone()))
    }

    /// A key while the Worktrees filter has the keyboard, before the field sees it; true when the list took it.
    pub(super) fn worktree_list_keystroke(&mut self, keystroke: &Keystroke, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let (Some(key), Some((project, list))) = (list_key(keystroke), self.shown_list()) else {
            return false;
        };
        let groups = self.list_groups(&project, Some(list.read(cx)), cx);
        let (selected, hidden_open, query_empty) = {
            let read = list.read(cx);
            (read.selected(&groups), read.hidden_open(), read.query(cx).is_empty())
        };
        match key_effect(key, &groups, hidden_open, selected.as_ref(), query_empty) {
            KeyEffect::Select(row) => list.update(cx, |list, cx| {
                let line = row.as_ref().and_then(|row| line_of(&lines(&groups, hidden_open), &groups, row));
                if let Some(line) = line {
                    list.scroll_to_line(line);
                }
                list.select(row.as_ref(), &groups, cx);
            }),
            KeyEffect::Open(row) => self.open_list_row(&project, &list, &row, window, cx),
            KeyEffect::Toggle(row) => self.toggle_list_row(&project, &list, &row, cx),
            KeyEffect::ClearFilter => list.update(cx, |list, cx| list.clear_filter(cx)),
            KeyEffect::Pass => return false,
        }
        true
    }

    /// A click on a row: selects it and does what Enter does.
    pub(crate) fn worktree_row_clicked(&mut self, project: &ProjectId, row: &RowId, window: &mut Window, cx: &mut Context<Self>) {
        let Some(list) = self.worktree_list_of(project) else {
            return self.open_list_row_plain(project, row, cx);
        };
        let groups = self.list_groups(project, Some(list.read(cx)), cx);
        list.update(cx, |list, cx| list.select(Some(row), &groups, cx));
        self.open_list_row(project, &list, row, window, cx);
    }

    /// Hide, Show or + Worktree: what Space does on the row.
    pub(crate) fn worktree_row_toggled(&mut self, project: &ProjectId, row: &RowId, cx: &mut Context<Self>) {
        if let Some(list) = self.worktree_list_of(project) {
            self.toggle_list_row(project, &list, row, cx);
        }
    }

    fn open_list_row(&mut self, project: &ProjectId, list: &Entity<WorktreeList>, row: &RowId, _window: &mut Window, cx: &mut Context<Self>) {
        match row {
            RowId::Checkout(_) => self.open_list_row_plain(project, row, cx),
            RowId::Branch(name) => {
                let branch = list.read(cx).branch(name).cloned();
                if let Some(branch) = branch {
                    list.update(cx, |list, cx| list.add(branch, false, cx));
                }
            }
        }
    }

    /// A checkout's row: shown expanded and selected in the tree; a hidden one does nothing.
    fn open_list_row_plain(&mut self, project: &ProjectId, row: &RowId, cx: &mut Context<Self>) {
        let RowId::Checkout(path) = row else {
            return;
        };
        if self.open_project(project).is_some_and(|open| !open.is_hidden(path)) {
            self.reveal_checkout(&CheckoutRef { project: project.clone(), path: path.clone() }, cx);
        }
    }

    /// Hides or shows a worktree, or adds a branch's worktree; the selection stays in place.
    fn toggle_list_row(&mut self, project: &ProjectId, list: &Entity<WorktreeList>, row: &RowId, cx: &mut Context<Self>) {
        let before = self.list_groups(project, Some(list.read(cx)), cx);
        list.update(cx, |list, cx| list.select(Some(row), &before, cx));
        match row {
            RowId::Checkout(path) => {
                let checkout = CheckoutRef { project: project.clone(), path: path.clone() };
                match self.open_project(project).map(|open| open.is_hidden(path)) {
                    Some(true) => self.show_worktree(&checkout, cx),
                    Some(false) => self.hide_worktree(&checkout, cx),
                    None => {}
                }
                let after = self.list_groups(project, Some(list.read(cx)), cx);
                let hidden_open = list.read(cx).hidden_open();
                let kept = kept_place(&before, row, &after, hidden_open);
                list.update(cx, |list, cx| list.select(kept.as_ref(), &after, cx));
            }
            // the branch's row stays selected until its worktree is there; then the selection keeps its place
            RowId::Branch(name) => {
                let branch = list.read(cx).branch(name).cloned();
                if let Some(branch) = branch {
                    list.update(cx, |list, cx| list.add(branch, true, cx));
                }
            }
        }
    }
}

fn drawn_group(group: &Group, list: Option<&WorktreeList>) -> DrawnGroup {
    let folder = list.and_then(WorktreeList::worktrees_folder).unwrap_or(crate::ive_toml::DEFAULT_WORKTREES).to_owned();
    let (title, hint, empty) = match group.kind {
        GroupKind::Worktrees => ("Worktrees".to_owned(), "the main checkout, then by last use".to_owned(), Some("No worktree matches")),
        GroupKind::Hidden => (format!("Hidden {}", group.total), "not in the tree, the hub or New terminal; their tabs stay open".to_owned(), None),
        GroupKind::Branches => (format!("Branches without a worktree {}", group.total), format!("newest first; Enter adds its worktree in {folder}"), Some("No branch matches")),
    };
    DrawnGroup { kind: group.kind, title, hint, total: group.total, shown: group.rows.len(), empty }
}

/// The keys the list takes: plain arrows, Enter, Space and Escape.
fn list_key(keystroke: &Keystroke) -> Option<ListKey> {
    let modifiers = &keystroke.modifiers;
    if modifiers.control || modifiers.alt || modifiers.shift || modifiers.platform || modifiers.function {
        return None;
    }
    match keystroke.key.as_str() {
        "up" => Some(ListKey::Up),
        "down" => Some(ListKey::Down),
        "enter" => Some(ListKey::Enter),
        "space" => Some(ListKey::Space),
        "escape" => Some(ListKey::Escape),
        _ => None,
    }
}

/// The main checkout first, then the others by last use, most recent first; never used ones last, in project order.
fn by_last_use(checkouts: Vec<(&Checkout, Option<i64>)>) -> Vec<&Checkout> {
    let (main, mut others): (Vec<_>, Vec<_>) = checkouts.into_iter().partition(|(checkout, _)| checkout.is_main);
    // stable: never used ones, and ties, keep the project's order
    others.sort_by_key(|(_, used)| std::cmp::Reverse(*used));
    main.into_iter().chain(others).map(|(checkout, _)| checkout).collect()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn checkout(name: &str, is_main: bool) -> Checkout {
        Checkout { path: PathBuf::from(format!("/p/{name}")), label: name.to_owned(), branch: None, head: None, is_main, status: None }
    }

    #[test]
    fn should_order_worktrees_main_first_then_by_last_use() {
        // given
        let (main, old, new, never, unused) = (checkout("dev", true), checkout("old", false), checkout("new", false), checkout("never", false), checkout("unused", false));
        let checkouts = vec![(&main, Some(1)), (&never, None), (&old, Some(5)), (&new, Some(9)), (&unused, None)];

        // when
        let labels: Vec<&str> = by_last_use(checkouts).iter().map(|checkout| checkout.label.as_str()).collect();

        // then
        assert_eq!(labels, ["dev", "new", "old", "never", "unused"]);
    }

    #[test]
    fn should_map_plain_keys_only() {
        // given
        let keys = ["up", "down", "enter", "space", "escape", "ctrl-down", "a", "shift-space"];

        // when
        let mapped: Vec<Option<ListKey>> = keys.iter().map(|key| list_key(&Keystroke::parse(key).unwrap())).collect();

        // then
        assert_eq!(mapped, [Some(ListKey::Up), Some(ListKey::Down), Some(ListKey::Enter), Some(ListKey::Space), Some(ListKey::Escape), None, None, None]);
    }
}
