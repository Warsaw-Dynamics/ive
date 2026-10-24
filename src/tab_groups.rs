//! The tab row's project groups: which tabs go together, in which order, which are folded and which tab a folded
//! group keeps out. A view over the flat tab order; nothing here owns tabs.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use crate::project::ProjectId;
use crate::tabs::TabId;

/// What grouping needs of a tab: its project (none for Settings, the hub), its checkout (none for the project's own
/// tabs: scratches, Git log, commits), whether that is the main checkout, and its owner (project or module) key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabFacts {
    pub id: TabId,
    pub project: Option<ProjectId>,
    pub checkout: Option<PathBuf>,
    pub main: bool,
    pub owner: Option<String>,
}

/// The groups' state: folded projects, each project's worktree order, each project's tab shown last.
pub struct GroupState<'a> {
    pub folded: &'a HashSet<ProjectId>,
    pub order: &'a HashMap<ProjectId, Vec<PathBuf>>,
    pub current: &'a HashMap<ProjectId, TabId>,
}

/// A group of the row: `tabs` in the order shown; folded, the row shows only `current`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabGroup {
    pub project: Option<ProjectId>,
    pub head: bool,
    pub folded: bool,
    pub tabs: Vec<TabId>,
    pub current: TabId,
}

/// Where a dragged item lands beside another.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Before,
    After,
}

/// The row's groups: one per project in the order its first tab shows, the tabs of no project in one group, each
/// project's tabs in worktree runs. With fewer than two projects, one group of every tab in tab order, no head.
pub fn groups(tabs: &[TabFacts], state: &GroupState) -> Vec<TabGroup> {
    let mut projects: Vec<Option<&ProjectId>> = Vec::new();
    for tab in tabs {
        if !projects.contains(&tab.project.as_ref()) {
            projects.push(tab.project.as_ref());
        }
    }
    if projects.iter().flatten().count() < 2 {
        let project = projects.into_iter().flatten().next().cloned();
        return ungrouped(tabs, project).into_iter().collect();
    }
    projects.into_iter().map(|project| project_group(tabs, project, state)).collect()
}

/// Every tab in tab order, under no head; nothing for no tabs.
fn ungrouped(tabs: &[TabFacts], project: Option<ProjectId>) -> Option<TabGroup> {
    let ids: Vec<TabId> = tabs.iter().map(|tab| tab.id).collect();
    let current = *ids.first()?;
    Some(TabGroup { project, head: false, folded: false, tabs: ids, current })
}

fn project_group(tabs: &[TabFacts], project: Option<&ProjectId>, state: &GroupState) -> TabGroup {
    let own = tabs.iter().filter(|tab| tab.project.as_ref() == project);
    let Some(project) = project else {
        let ids: Vec<TabId> = own.map(|tab| tab.id).collect();
        return TabGroup { project: None, head: false, folded: false, current: ids[0], tabs: ids };
    };
    let order = state.order.get(project).map_or(&[][..], Vec::as_slice);
    let ids: Vec<TabId> = worktree_runs(own, order).into_iter().flat_map(|(_, run)| run).collect();
    let current = state.current.get(project).filter(|current| ids.contains(current)).copied().unwrap_or(ids[0]);
    TabGroup { project: Some(project.clone()), head: true, folded: state.folded.contains(project), tabs: ids, current }
}

/// A project's tabs in runs of one checkout each: those in `order` first, in its order, then the main checkout's,
/// then the other worktrees' in the order each first shows, the project's own tabs (no checkout) last.
pub fn worktree_runs<'a>(tabs: impl IntoIterator<Item = &'a TabFacts>, order: &[PathBuf]) -> Vec<(Option<PathBuf>, Vec<TabId>)> {
    let mut runs: Vec<(Option<PathBuf>, bool, Vec<TabId>)> = Vec::new();
    for tab in tabs {
        match runs.iter_mut().find(|(checkout, _, _)| *checkout == tab.checkout) {
            Some((_, _, run)) => run.push(tab.id),
            None => runs.push((tab.checkout.clone(), tab.main, vec![tab.id])),
        }
    }
    let rank = |checkout: &Option<PathBuf>, main: bool| match checkout {
        Some(path) => order.iter().position(|ordered| ordered == path).map_or(if main { (1, 0) } else { (1, 1) }, |at| (0, at)),
        None => (2, 0),
    };
    runs.sort_by_key(|(checkout, main, _)| rank(checkout, *main));
    runs.into_iter().map(|(checkout, _, run)| (checkout, run)).collect()
}

/// Whether the tabs come from more than one checkout: only then are worktree tabs tinted.
pub fn spread_worktrees(tabs: &[TabFacts]) -> bool {
    distinct(tabs.iter().filter_map(|tab| tab.checkout.as_ref())) > 1
}

/// Whether the tabs have more than one owner: only then do tabs carry their owner's badge.
pub fn spread_owners(tabs: &[TabFacts]) -> bool {
    distinct(tabs.iter().filter_map(|tab| tab.owner.as_ref())) > 1
}

fn distinct<T: Eq + std::hash::Hash>(items: impl Iterator<Item = T>) -> usize {
    items.collect::<HashSet<T>>().len()
}

/// `items` with `moved` taken out and put on `beside`'s side of `beside`'s item.
pub fn placed<T: PartialEq + Clone>(items: &[T], moved: &T, beside: (&T, Side)) -> Vec<T> {
    let mut rest: Vec<T> = items.iter().filter(|item| *item != moved).cloned().collect();
    let (target, side) = beside;
    let at = rest.iter().position(|item| item == target).map_or(rest.len(), |at| if side == Side::After { at + 1 } else { at });
    rest.insert(at, moved.clone());
    rest
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};
    use std::path::PathBuf;

    use super::*;
    use crate::project::ProjectId;
    use crate::tabs::TabId;

    fn fact(id: u64, project: Option<&str>, checkout: Option<&str>, main: bool) -> TabFacts {
        TabFacts {
            id: TabId::for_tests(id),
            project: project.map(ProjectId::new),
            checkout: checkout.map(PathBuf::from),
            main,
            owner: project.map(str::to_owned),
        }
    }

    fn ids(values: &[u64]) -> Vec<TabId> {
        values.iter().map(|value| TabId::for_tests(*value)).collect()
    }

    struct Given {
        folded: HashSet<ProjectId>,
        order: HashMap<ProjectId, Vec<PathBuf>>,
        current: HashMap<ProjectId, TabId>,
    }

    impl Given {
        fn new() -> Self {
            Self { folded: HashSet::new(), order: HashMap::new(), current: HashMap::new() }
        }

        fn state(&self) -> GroupState<'_> {
            GroupState { folded: &self.folded, order: &self.order, current: &self.current }
        }
    }

    #[test]
    fn should_not_group_tabs_of_one_project() {
        // given
        let tabs = [fact(1, Some("a"), Some("/a"), true), fact(2, Some("a"), Some("/a-wt"), false)];
        let given = Given::new();

        // when
        let groups = groups(&tabs, &given.state());

        // then
        assert_eq!(groups.len(), 1);
        assert!(!groups[0].head);
        assert_eq!(groups[0].tabs, ids(&[1, 2]));
    }

    #[test]
    fn should_group_by_project_in_order_of_first_tab() {
        // given
        let tabs = [fact(1, Some("a"), Some("/a"), true), fact(2, Some("b"), Some("/b"), true), fact(3, Some("a"), Some("/a"), true)];
        let given = Given::new();

        // when
        let groups = groups(&tabs, &given.state());

        // then
        let shown: Vec<(Option<ProjectId>, bool, Vec<TabId>)> = groups.into_iter().map(|group| (group.project, group.head, group.tabs)).collect();
        assert_eq!(shown, [(Some(ProjectId::new("a")), true, ids(&[1, 3])), (Some(ProjectId::new("b")), true, ids(&[2]))]);
    }

    #[test]
    fn should_put_main_first_then_worktrees_then_the_projects_own() {
        // given
        let tabs = [
            fact(1, Some("a"), Some("/x"), false),
            fact(2, Some("a"), Some("/a"), true),
            fact(3, Some("a"), None, false),
            fact(4, Some("a"), Some("/y"), false),
            fact(5, Some("a"), Some("/x"), false),
        ];

        // when
        let runs = worktree_runs(&tabs, &[]);

        // then
        let expected = [(Some("/a"), vec![2]), (Some("/x"), vec![1, 5]), (Some("/y"), vec![4]), (None, vec![3])];
        assert_eq!(runs, expected.map(|(checkout, tabs)| (checkout.map(PathBuf::from), ids(&tabs))));
    }

    #[test]
    fn should_follow_the_dragged_worktree_order() {
        // given
        let tabs = [fact(1, Some("a"), Some("/x"), false), fact(2, Some("a"), Some("/a"), true), fact(3, Some("a"), Some("/y"), false)];
        let order = [PathBuf::from("/y"), PathBuf::from("/a")];

        // when
        let runs = worktree_runs(&tabs, &order);

        // then
        let checkouts: Vec<Option<PathBuf>> = runs.into_iter().map(|(checkout, _)| checkout).collect();
        assert_eq!(checkouts, [Some(PathBuf::from("/y")), Some(PathBuf::from("/a")), Some(PathBuf::from("/x"))]);
    }

    #[test]
    fn should_keep_the_current_tab_out_of_a_folded_group() {
        // given
        let tabs = [fact(1, Some("a"), Some("/a"), true), fact(2, Some("a"), Some("/a"), true), fact(3, Some("b"), Some("/b"), true)];
        let mut given = Given::new();
        given.folded.insert(ProjectId::new("a"));
        given.current.insert(ProjectId::new("a"), TabId::for_tests(2));

        // when
        let groups = groups(&tabs, &given.state());

        // then
        assert!(groups[0].folded);
        assert_eq!(groups[0].current, TabId::for_tests(2));
        assert!(!groups[1].folded);
    }

    #[test]
    fn should_fall_back_to_the_first_tab_when_the_current_one_closed() {
        // given
        let tabs = [fact(1, Some("a"), Some("/a"), true), fact(3, Some("b"), Some("/b"), true)];
        let mut given = Given::new();
        given.current.insert(ProjectId::new("a"), TabId::for_tests(2));

        // when
        let groups = groups(&tabs, &given.state());

        // then
        assert_eq!(groups[0].current, TabId::for_tests(1));
    }

    #[test]
    fn should_collect_tabs_without_a_project_in_a_headless_group() {
        // given
        let tabs = [fact(1, None, None, false), fact(2, Some("a"), Some("/a"), true), fact(3, Some("b"), Some("/b"), true), fact(4, None, None, false)];
        let given = Given::new();

        // when
        let groups = groups(&tabs, &given.state());

        // then
        let headless: Vec<&TabGroup> = groups.iter().filter(|group| group.project.is_none()).collect();
        assert_eq!(headless.len(), 1);
        assert!(!headless[0].head);
        assert_eq!(headless[0].tabs, ids(&[1, 4]));
    }

    #[test]
    fn should_spread_only_over_more_than_one_worktree() {
        // given
        let one = [fact(1, Some("a"), Some("/a"), true), fact(2, Some("a"), None, false)];
        let two = [fact(1, Some("a"), Some("/a"), true), fact(2, Some("a"), Some("/x"), false)];

        // when
        let spread = (spread_worktrees(&one), spread_worktrees(&two));

        // then
        assert_eq!(spread, (false, true));
    }

    #[test]
    fn should_spread_only_over_more_than_one_owner() {
        // given
        let mut module = fact(2, Some("a"), Some("/a"), true);
        module.owner = Some("a/idp".to_owned());
        let one = [fact(1, Some("a"), Some("/a"), true), fact(3, Some("a"), Some("/x"), false)];
        let two = [fact(1, Some("a"), Some("/a"), true), module];

        // when
        let spread = (spread_owners(&one), spread_owners(&two));

        // then
        assert_eq!(spread, (false, true));
    }

    #[test]
    fn should_place_an_item_after_another() {
        // given
        let items = ["a", "b", "c"];

        // when
        let placed = (placed(&items, &"a", (&"c", Side::After)), placed(&items, &"c", (&"a", Side::Before)));

        // then
        assert_eq!(placed, (vec!["b", "c", "a"], vec!["c", "a", "b"]));
    }
}
