//! The project page's Worktrees list as data: its groups (worktrees, hidden worktrees, branches without a worktree),
//! the filter over them, and where the keys move the selection.

use std::path::PathBuf;

/// A row of the list: a checkout by path, or a branch without a worktree by its shown name.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RowId {
    Checkout(PathBuf),
    Branch(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupKind {
    Worktrees,
    Hidden,
    Branches,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Group {
    pub kind: GroupKind,
    pub rows: Vec<RowId>,
    /// the rows the filter matched, before paging
    pub total: usize,
}

/// A row that may be listed, with the text the filter matches, lowercase.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    pub id: RowId,
    pub text: String,
}

impl Candidate {
    pub fn new(id: RowId, text: &str) -> Self {
        Self { id, text: text.to_lowercase() }
    }
}

/// What the groups are made of: the shown checkouts in list order (the main one, then by last use), the hidden
/// ones, the branches newest first.
pub struct ListInput<'a> {
    pub shown: &'a [Candidate],
    pub hidden: &'a [Candidate],
    pub branches: &'a [Candidate],
    pub query: &'a str,
    /// how many matching branches the paging shows
    pub branches_visible: usize,
    /// a plain folder has no branches
    pub git: bool,
}

/// The keys the list takes while its filter has the keyboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListKey {
    Up,
    Down,
    Enter,
    Space,
    Escape,
}

/// What a key does to the list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyEffect {
    Select(Option<RowId>),
    Open(RowId),
    /// hide or show a worktree; add a branch's worktree
    Toggle(RowId),
    ClearFilter,
    /// the field takes it: Space types, Enter and Escape do nothing
    Pass,
}

/// Worktrees always; Hidden only when it has rows; Branches for a git project.
pub fn groups(input: &ListInput) -> Vec<Group> {
    let words = query_words(input.query);
    let matching = |candidates: &[Candidate]| -> Vec<RowId> {
        candidates.iter().filter(|candidate| matches(&candidate.text, &words)).map(|candidate| candidate.id.clone()).collect()
    };
    let whole = |kind: GroupKind, rows: Vec<RowId>| Group { kind, total: rows.len(), rows };
    let mut groups = vec![whole(GroupKind::Worktrees, matching(input.shown))];
    let hidden = matching(input.hidden);
    if !hidden.is_empty() {
        groups.push(whole(GroupKind::Hidden, hidden));
    }
    if input.git {
        let branches = matching(input.branches);
        let total = branches.len();
        groups.push(Group { kind: GroupKind::Branches, rows: branches.into_iter().take(input.branches_visible).collect(), total });
    }
    groups
}

/// The words of `query`, lowercase, as [`matches`] takes them.
pub fn query_words(query: &str) -> Vec<String> {
    query.to_lowercase().split_whitespace().map(str::to_owned).collect()
}

/// Whether the lowercase `text` holds every one of `words`.
pub fn matches(text: &str, words: &[String]) -> bool {
    words.iter().all(|word| text.contains(word.as_str()))
}

/// A line of the list as drawn, every one a row high; groups and rows by their index in the groups.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Line {
    Header(usize),
    /// "No worktree matches", under a group the filter emptied
    Empty(usize),
    Row { group: usize, at: usize },
    /// Show 10 more, Show all: under the branches while some are not shown
    More(usize),
}

/// The lines of `groups`, top to bottom: each group's header, then its rows unless it is the closed Hidden group.
pub fn lines(groups: &[Group], hidden_open: bool) -> Vec<Line> {
    let mut lines = Vec::new();
    for (index, group) in groups.iter().enumerate() {
        lines.push(Line::Header(index));
        if group.kind == GroupKind::Hidden && !hidden_open {
            continue;
        }
        if group.rows.is_empty() && group.kind != GroupKind::Hidden {
            lines.push(Line::Empty(index));
        }
        lines.extend((0..group.rows.len()).map(|at| Line::Row { group: index, at }));
        if group.kind == GroupKind::Branches && group.total > group.rows.len() {
            lines.push(Line::More(index));
        }
    }
    lines
}

/// The line of `row` among `lines`, for scrolling it into view.
pub fn line_of(lines: &[Line], groups: &[Group], row: &RowId) -> Option<usize> {
    lines.iter().position(|line| matches!(line, Line::Row { group, at } if &groups[*group].rows[*at] == row))
}

/// The rows ↑↓ walk, top to bottom: a closed Hidden group's are left out.
pub fn walkable(groups: &[Group], hidden_open: bool) -> Vec<&RowId> {
    open_groups(groups, hidden_open).flat_map(|group| &group.rows).collect()
}

fn open_groups(groups: &[Group], hidden_open: bool) -> impl Iterator<Item = &Group> {
    groups.iter().filter(move |group| hidden_open || group.kind != GroupKind::Hidden)
}

/// The row after (`down`) or before `selected`, stopping at the ends; the first row when nothing is selected.
pub fn moved(groups: &[Group], hidden_open: bool, selected: Option<&RowId>, down: bool) -> Option<RowId> {
    let rows = walkable(groups, hidden_open);
    let at = selected.and_then(|selected| rows.iter().position(|row| *row == selected));
    let next = match (at, down) {
        (None, _) => 0,
        (Some(at), true) => (at + 1).min(rows.len().saturating_sub(1)),
        (Some(at), false) => at.saturating_sub(1),
    };
    rows.get(next).map(|row| (*row).clone())
}

/// Where the selection goes after `acted` was toggled and moved elsewhere: the row now at its position in its group,
/// else the first row after that group, else the last row.
pub fn kept_place(before: &[Group], acted: &RowId, after: &[Group], hidden_open: bool) -> Option<RowId> {
    let selection = selection_of(before, acted)?;
    place(&selection, after, hidden_open)
}

/// The row at `selection`'s place in `groups`, else the first row after its group, else the last row.
fn place(selection: &Selection, groups: &[Group], hidden_open: bool) -> Option<RowId> {
    let same = groups.iter().find(|group| group.kind == selection.kind).and_then(|group| group.rows.get(selection.at));
    let following = || {
        let later = open_groups(groups, hidden_open).skip_while(|group| rank(group.kind) <= rank(selection.kind));
        later.flat_map(|group| &group.rows).next()
    };
    let last = || walkable(groups, hidden_open).last().copied();
    same.or_else(following).or_else(last).cloned()
}

/// A selected row and where it was: a row that leaves its group (hidden, shown, given a worktree) leaves the
/// selection at its place.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection {
    pub row: RowId,
    pub kind: GroupKind,
    pub at: usize,
}

pub fn selection_of(groups: &[Group], row: &RowId) -> Option<Selection> {
    groups.iter().find_map(|group| group.rows.iter().position(|listed| listed == row).map(|at| Selection { row: row.clone(), kind: group.kind, at }))
}

/// The row `selection` stands for in `groups`: its row while its group has it, else the row at its place; nothing
/// while it is in the closed Hidden group, so Space never acts on a row out of sight.
pub fn resolved(selection: &Selection, groups: &[Group], hidden_open: bool) -> Option<RowId> {
    if selection.kind == GroupKind::Hidden && !hidden_open {
        return None;
    }
    let kept = groups.iter().any(|group| group.kind == selection.kind && group.rows.contains(&selection.row));
    if kept {
        return Some(selection.row.clone());
    }
    place(selection, groups, hidden_open)
}

/// Groups in list order.
fn rank(kind: GroupKind) -> u8 {
    match kind {
        GroupKind::Worktrees => 0,
        GroupKind::Hidden => 1,
        GroupKind::Branches => 2,
    }
}

pub fn key_effect(key: ListKey, groups: &[Group], hidden_open: bool, selected: Option<&RowId>, query_empty: bool) -> KeyEffect {
    match (key, selected) {
        (ListKey::Up | ListKey::Down, _) => KeyEffect::Select(moved(groups, hidden_open, selected, key == ListKey::Down)),
        (ListKey::Enter, Some(row)) => KeyEffect::Open(row.clone()),
        (ListKey::Space, Some(row)) => KeyEffect::Toggle(row.clone()),
        (ListKey::Escape, Some(_)) => KeyEffect::Select(None),
        (ListKey::Escape, None) if !query_empty => KeyEffect::ClearFilter,
        (ListKey::Enter | ListKey::Space | ListKey::Escape, _) => KeyEffect::Pass,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn checkout(name: &str) -> RowId {
        RowId::Checkout(PathBuf::from(format!("/p/{name}")))
    }

    fn branch(name: &str) -> RowId {
        RowId::Branch(name.to_owned())
    }

    fn candidates(ids: &[RowId]) -> Vec<Candidate> {
        let text = |id: &RowId| match id {
            RowId::Checkout(path) => path.display().to_string(),
            RowId::Branch(name) => name.clone(),
        };
        ids.iter().map(|id| Candidate::new(id.clone(), &text(id))).collect()
    }

    /// The candidates of the shown, hidden and branch rows.
    struct Rows([Vec<Candidate>; 3]);

    impl Rows {
        fn of(shown: &[RowId], hidden: &[RowId], branches: &[RowId]) -> Self {
            Self([candidates(shown), candidates(hidden), candidates(branches)])
        }

        fn input<'a>(&'a self, query: &'a str) -> ListInput<'a> {
            let [shown, hidden, branches] = &self.0;
            ListInput { shown, hidden, branches, query, branches_visible: 10, git: true }
        }
    }

    fn group(kind: GroupKind, rows: &[RowId]) -> Group {
        Group { kind, rows: rows.to_vec(), total: rows.len() }
    }

    fn list(worktrees: &[RowId], hidden: &[RowId], branches: &[RowId]) -> Vec<Group> {
        let hidden = (!hidden.is_empty()).then(|| group(GroupKind::Hidden, hidden));
        std::iter::once(group(GroupKind::Worktrees, worktrees)).chain(hidden).chain([group(GroupKind::Branches, branches)]).collect()
    }

    #[test]
    fn should_group_worktrees_hidden_and_branches() {
        // given
        let rows = Rows::of(&[checkout("dev"), checkout("a")], &[checkout("h")], &[branch("x"), branch("y")]);

        // when
        let found = groups(&rows.input(""));

        // then
        assert_eq!(found, list(&[checkout("dev"), checkout("a")], &[checkout("h")], &[branch("x"), branch("y")]));
    }

    #[test]
    fn should_leave_out_an_empty_hidden_group_and_the_branches_of_a_plain_folder() {
        // given
        let rows = Rows::of(&[checkout("notes")], &[], &[]);
        let given = ListInput { git: false, ..rows.input("") };

        // when
        let kinds: Vec<GroupKind> = groups(&given).iter().map(|group| group.kind).collect();

        // then
        assert_eq!(kinds, [GroupKind::Worktrees]);
    }

    #[test]
    fn should_filter_every_group_by_all_words() {
        // given
        let rows = Rows::of(&[checkout("dev"), checkout("GEN-817-ive")], &[checkout("GEN-811")], &[branch("gen-813-fix"), branch("main")]);

        // when
        let found = groups(&rows.input("gen 81"));

        // then
        assert_eq!(found, list(&[checkout("GEN-817-ive")], &[checkout("GEN-811")], &[branch("gen-813-fix")]));
        assert!(matches("gen-817-ive", &query_words("IVE  817")));
    }

    #[test]
    fn should_page_the_branches_but_count_them_all() {
        // given
        let branches: Vec<RowId> = (0..25).map(|n| branch(&format!("b{n}"))).collect();
        let rows = Rows::of(&[checkout("dev")], &[], &branches);

        // when
        let found = groups(&rows.input(""));

        // then
        assert_eq!((found[1].rows.len(), found[1].total), (10, 25));
    }

    #[test]
    fn should_draw_headers_rows_empty_notes_and_the_paging_line() {
        // given
        let groups = vec![group(GroupKind::Worktrees, &[]), group(GroupKind::Hidden, &[checkout("h")]), Group { kind: GroupKind::Branches, rows: vec![branch("x")], total: 3 }];

        // when
        let (open, closed) = (lines(&groups, true), lines(&groups, false));

        // then
        assert_eq!(open, [Line::Header(0), Line::Empty(0), Line::Header(1), Line::Row { group: 1, at: 0 }, Line::Header(2), Line::Row { group: 2, at: 0 }, Line::More(2)]);
        assert_eq!(closed, [Line::Header(0), Line::Empty(0), Line::Header(1), Line::Header(2), Line::Row { group: 2, at: 0 }, Line::More(2)]);
        assert_eq!(line_of(&open, &groups, &branch("x")), Some(5));
    }

    #[test]
    fn should_move_over_all_groups_and_stop_at_the_ends() {
        // given
        let groups = list(&[checkout("dev"), checkout("a")], &[checkout("h")], &[branch("x")]);
        let step = |selected: Option<RowId>, down: bool, open: bool| moved(&groups, open, selected.as_ref(), down);

        // when
        let steps = [
            step(None, true, true),
            step(Some(checkout("a")), true, true),
            step(Some(checkout("a")), true, false),
            step(Some(branch("x")), true, true),
            step(Some(checkout("dev")), false, true),
        ];

        // then
        assert_eq!(steps, [Some(checkout("dev")), Some(checkout("h")), Some(branch("x")), Some(branch("x")), Some(checkout("dev"))]);
    }

    #[test]
    fn should_keep_the_place_after_a_worktree_moves_to_hidden() {
        // given
        let before = list(&[checkout("dev"), checkout("a"), checkout("b")], &[checkout("h")], &[]);
        let after = list(&[checkout("dev"), checkout("b")], &[checkout("a"), checkout("h")], &[]);

        // when
        let kept = kept_place(&before, &checkout("a"), &after, true);

        // then
        assert_eq!(kept, Some(checkout("b")));
    }

    #[test]
    fn should_go_past_the_group_when_it_got_shorter() {
        // given
        let before = list(&[checkout("dev"), checkout("a")], &[], &[branch("x")]);
        let after = list(&[checkout("dev")], &[checkout("a")], &[branch("x")]);

        // when
        let kept = kept_place(&before, &checkout("a"), &after, true);

        // then
        assert_eq!(kept, Some(checkout("a")), "the first row after Worktrees");
    }

    #[test]
    fn should_keep_the_place_after_a_branch_gets_its_worktree() {
        // given
        let before = list(&[checkout("dev")], &[], &[branch("x"), branch("y"), branch("z")]);
        let after = list(&[checkout("dev"), checkout("y")], &[], &[branch("x"), branch("z")]);
        let only = list(&[checkout("dev")], &[], &[branch("x")]);
        let none_left = list(&[checkout("dev"), checkout("x")], &[], &[]);

        // when
        let kept = [kept_place(&before, &branch("y"), &after, true), kept_place(&only, &branch("x"), &none_left, true)];

        // then
        assert_eq!(kept, [Some(branch("z")), Some(checkout("x"))]);
    }

    #[test]
    fn should_resolve_a_selection_to_its_row_while_its_group_has_it_else_to_its_place() {
        // given
        let before = list(&[checkout("dev")], &[], &[branch("x"), branch("y"), branch("z")]);
        let selection = selection_of(&before, &branch("y")).unwrap();
        let adding = list(&[checkout("dev"), checkout("a")], &[], &[branch("x"), branch("y"), branch("z")]);
        let added = list(&[checkout("dev"), checkout("y")], &[], &[branch("x"), branch("z")]);

        // when
        let resolved = [resolved(&selection, &adding, true), resolved(&selection, &added, true)];

        // then
        assert_eq!(selection, Selection { row: branch("y"), kind: GroupKind::Branches, at: 1 });
        assert_eq!(resolved, [Some(branch("y")), Some(branch("z"))]);
    }

    #[test]
    fn should_select_nothing_in_a_closed_hidden_group() {
        // given
        let groups = list(&[checkout("dev")], &[checkout("h")], &[branch("x")]);
        let selection = selection_of(&groups, &checkout("h")).unwrap();

        // when
        let found = [resolved(&selection, &groups, false), resolved(&selection, &groups, true)];

        // then
        assert_eq!(found, [None, Some(checkout("h"))]);
    }

    #[test]
    fn should_turn_keys_into_effects() {
        // given
        let groups = list(&[checkout("dev"), checkout("a")], &[], &[branch("x")]);
        let effect = |key: ListKey, selected: Option<RowId>, query_empty: bool| key_effect(key, &groups, true, selected.as_ref(), query_empty);

        // when
        let effects = [
            effect(ListKey::Down, None, true),
            effect(ListKey::Space, None, false),
            effect(ListKey::Space, Some(checkout("a")), false),
            effect(ListKey::Enter, Some(branch("x")), false),
            effect(ListKey::Enter, None, false),
            effect(ListKey::Escape, Some(checkout("a")), false),
            effect(ListKey::Escape, None, false),
            effect(ListKey::Escape, None, true),
        ];

        // then
        assert_eq!(effects, [
            KeyEffect::Select(Some(checkout("dev"))),
            KeyEffect::Pass,
            KeyEffect::Toggle(checkout("a")),
            KeyEffect::Open(branch("x")),
            KeyEffect::Pass,
            KeyEffect::Select(None),
            KeyEffect::ClearFilter,
            KeyEffect::Pass,
        ]);
    }
}
