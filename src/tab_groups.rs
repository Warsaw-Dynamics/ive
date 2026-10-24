//! The tab row's project groups: which tabs go together, in which order, which are folded and which tab a folded
//! group keeps out. A view over the flat tab order; nothing here owns tabs.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use crate::project::ProjectId;
use crate::tabs::TabId;

/// What grouping needs of a tab: its project (none for Settings, the hub), its checkout (none for the project's own
/// tabs: scratches, Git log, commits), whether that is the main checkout, its owner (project or module) key, and
/// whether its project has worktrees.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabFacts {
    pub id: TabId,
    pub project: Option<ProjectId>,
    pub checkout: Option<PathBuf>,
    pub main: bool,
    pub owner: Option<String>,
    /// its project has worktrees besides the main checkout
    pub worktrees: bool,
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

/// What the tab row draws, in order: a group's head (by the group's index) or a tab shown (with its group's index).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowItem {
    Head(usize),
    Tab(usize, TabId),
}

/// The row's items: each group's head when it has one, then its tabs, a folded group's current one alone.
pub fn row_items(groups: &[TabGroup]) -> Vec<RowItem> {
    let mut items = Vec::new();
    for (index, group) in groups.iter().enumerate() {
        if group.head {
            items.push(RowItem::Head(index));
        }
        let shown: &[TabId] = if group.folded { std::slice::from_ref(&group.current) } else { &group.tabs };
        items.extend(shown.iter().map(|id| RowItem::Tab(index, *id)));
    }
    items
}

/// Whether the row's tabs come from more than one checkout, and have more than one owner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Spread {
    pub worktrees: bool,
    pub owners: bool,
}

/// How a tab shows its worktree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CueKind {
    Plain,
    /// its background in its worktree's colour
    Tint,
    /// its worktree's name after its title: the tab out of a folded group, which nothing beside it names
    Label,
}

/// A tab's worktree cue in its group: a label out of a folded group, a tint for a worktree other than the main
/// checkout while the row spans worktrees; none for the project's own tabs.
pub fn chip_cue(fact: &TabFacts, group: &TabGroup, spread: Spread) -> CueKind {
    match (&fact.checkout, group.folded) {
        (None, _) => CueKind::Plain,
        (Some(_), true) if fact.worktrees => CueKind::Label,
        (Some(_), true) => CueKind::Plain,
        (Some(_), false) if spread.worktrees && !fact.main => CueKind::Tint,
        (Some(_), false) => CueKind::Plain,
    }
}

/// Whether a tab carries its owner's badge: only while the row has several owners, and under a head only a
/// module's, the head naming the project.
pub fn shows_badge(fact: &TabFacts, group: &TabGroup, spread: Spread) -> bool {
    let Some(owner) = &fact.owner else {
        return false;
    };
    let is_module = fact.project.as_ref().is_none_or(|project| project.as_str() != owner);
    spread.owners && (is_module || !group.head)
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

/// The tab order `after` a move, with the projects in the order their first tabs had `before` it: a move reorders
/// tabs within their project's group, never the groups (a tab dropped among another project's goes back to its own).
pub fn keep_group_order(before: &[TabFacts], after: &[TabId]) -> Vec<TabId> {
    let mut projects: Vec<Option<&ProjectId>> = Vec::new();
    for tab in before {
        if !projects.contains(&tab.project.as_ref()) {
            projects.push(tab.project.as_ref());
        }
    }
    let rank = |id: &TabId| {
        let project = before.iter().find(|tab| tab.id == *id).map(|tab| tab.project.as_ref());
        project.and_then(|project| projects.iter().position(|known| *known == project)).unwrap_or(projects.len())
    };
    let mut order = after.to_vec();
    order.sort_by_key(rank);
    order
}

/// The gap of the row `order` (0 before the first tab, as `Tabs::move_tab` counts them before the move) that puts
/// `moved` on `beside`'s side of `beside`'s tab; `None` when either is not in the row.
pub fn move_gap(order: &[TabId], moved: TabId, beside: (TabId, Side)) -> Option<usize> {
    order.iter().position(|id| *id == moved)?;
    let at = order.iter().position(|id| *id == beside.0)?;
    Some(if beside.1 == Side::After { at + 1 } else { at })
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
            worktrees: true,
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

    fn group_of(facts: &[TabFacts], head: bool, folded: bool) -> TabGroup {
        TabGroup { project: facts[0].project.clone(), head, folded, tabs: facts.iter().map(|fact| fact.id).collect(), current: facts[0].id }
    }

    const SPREAD: Spread = Spread { worktrees: true, owners: true };

    #[test]
    fn should_tint_a_worktree_tab_when_tabs_span_worktrees() {
        // given
        let tab = fact(1, Some("a"), Some("/a-wt"), false);
        let group = group_of(std::slice::from_ref(&tab), true, false);

        // when
        let cue = chip_cue(&tab, &group, SPREAD);

        // then
        assert_eq!(cue, CueKind::Tint);
    }

    #[test]
    fn should_not_tint_the_main_checkout() {
        // given
        let tab = fact(1, Some("a"), Some("/a"), true);
        let group = group_of(std::slice::from_ref(&tab), true, false);

        // when
        let cue = chip_cue(&tab, &group, SPREAD);

        // then
        assert_eq!(cue, CueKind::Plain);
    }

    #[test]
    fn should_not_tint_when_one_worktree_is_open() {
        // given
        let tab = fact(1, Some("a"), Some("/a-wt"), false);
        let group = group_of(std::slice::from_ref(&tab), false, false);

        // when
        let cue = chip_cue(&tab, &group, Spread { worktrees: false, owners: true });

        // then
        assert_eq!(cue, CueKind::Plain);
    }

    #[test]
    fn should_label_the_tab_out_of_a_folded_group() {
        // given
        let worktree = fact(1, Some("a"), Some("/a-wt"), false);
        let main = fact(2, Some("a"), Some("/a"), true);
        let own = fact(3, Some("a"), None, false);
        let group = group_of(&[worktree.clone(), main.clone(), own.clone()], true, true);

        // when
        let cues = [&worktree, &main, &own].map(|tab| chip_cue(tab, &group, SPREAD));

        // then
        assert_eq!(cues, [CueKind::Label, CueKind::Label, CueKind::Plain]);
    }

    #[test]
    fn should_not_label_a_tab_of_a_project_without_worktrees() {
        // given
        let mut tab = fact(1, Some("a"), Some("/a"), true);
        tab.worktrees = false;
        let group = group_of(std::slice::from_ref(&tab), true, true);

        // when
        let cue = chip_cue(&tab, &group, SPREAD);

        // then
        assert_eq!(cue, CueKind::Plain);
    }

    #[test]
    fn should_badge_a_module_tab_when_owners_spread() {
        // given
        let mut tab = fact(1, Some("a"), Some("/a"), true);
        tab.owner = Some("a/idp".to_owned());
        let group = group_of(std::slice::from_ref(&tab), true, false);

        // when
        let shown = (shows_badge(&tab, &group, SPREAD), shows_badge(&tab, &group, Spread { worktrees: true, owners: false }));

        // then
        assert_eq!(shown, (true, false));
    }

    #[test]
    fn should_not_badge_the_project_itself_under_a_head() {
        // given
        let tab = fact(1, Some("a"), Some("/a"), true);
        let group = group_of(std::slice::from_ref(&tab), true, false);

        // when
        let shown = shows_badge(&tab, &group, SPREAD);

        // then
        assert!(!shown);
    }

    #[test]
    fn should_badge_the_project_without_heads_when_owners_spread() {
        // given
        let tab = fact(1, Some("a"), Some("/a"), true);
        let group = group_of(std::slice::from_ref(&tab), false, false);

        // when
        let shown = shows_badge(&tab, &group, SPREAD);

        // then
        assert!(shown);
    }

    #[test]
    fn should_list_heads_and_shown_tabs_in_row_order() {
        // given
        let tabs = [fact(1, Some("a"), Some("/a"), true), fact(2, Some("a"), Some("/a"), true), fact(3, Some("b"), Some("/b"), true)];
        let mut given = Given::new();
        given.folded.insert(ProjectId::new("a"));
        given.current.insert(ProjectId::new("a"), TabId::for_tests(2));
        let groups = groups(&tabs, &given.state());

        // when
        let items = row_items(&groups);

        // then
        assert_eq!(items, [RowItem::Head(0), RowItem::Tab(0, TabId::for_tests(2)), RowItem::Head(1), RowItem::Tab(1, TabId::for_tests(3))]);
    }

    #[test]
    fn should_give_the_gap_that_puts_a_tab_beside_another() {
        // given
        let moves = |moved: usize, beside: (usize, Side)| {
            let mut tabs = crate::tabs::Tabs::default();
            let ids = [tabs.open("a"), tabs.open("b"), tabs.open("c")];
            let order: Vec<TabId> = tabs.row().map(|(id, _)| id).collect();
            let gap = move_gap(&order, ids[moved], (ids[beside.0], beside.1)).unwrap();
            tabs.move_tab(ids[moved], gap);
            tabs.row().map(|(_, title)| *title).collect::<Vec<_>>()
        };

        // when
        let rows = [moves(0, (2, Side::After)), moves(2, (0, Side::Before)), moves(0, (1, Side::Before)), moves(2, (1, Side::After))];

        // then
        assert_eq!(rows, [["b", "c", "a"], ["c", "a", "b"], ["a", "b", "c"], ["a", "b", "c"]]);
    }

    #[test]
    fn should_keep_group_order_when_a_tab_moves_within_its_group() {
        // given: [A1, B1, A2] drawn as [A] A1 A2 [B] B1; A1 dropped after A2 gives [B1, A2, A1]
        let before = [fact(1, Some("a"), Some("/a"), true), fact(2, Some("b"), Some("/b"), true), fact(3, Some("a"), Some("/a"), true)];

        // when
        let order = keep_group_order(&before, &ids(&[2, 3, 1]));

        // then
        assert_eq!(order, ids(&[3, 1, 2]));
    }

    #[test]
    fn should_put_a_tab_dropped_among_another_projects_back_in_its_own() {
        // given: B1 dropped between A1 and A2
        let before = [fact(1, Some("a"), Some("/a"), true), fact(3, Some("a"), Some("/a"), true), fact(2, Some("b"), Some("/b"), true)];

        // when
        let order = keep_group_order(&before, &ids(&[1, 2, 3]));

        // then
        assert_eq!(order, ids(&[1, 3, 2]));
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
