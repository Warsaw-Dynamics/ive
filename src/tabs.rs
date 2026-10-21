/// Identity of a tab; indexes shift as tabs close, ids do not.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct TabId(u64);

impl TabId {
    /// For element ids, which must stay the same across frames.
    pub fn value(self) -> u64 {
        self.0
    }
}

/// Which of the row's tabs, seen from one of them, Close Other Tabs and Close Tabs to the Left / Right close.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Beside {
    Left,
    Right,
    Others,
}

/// Open tabs in display order, and which one is shown.
pub struct Tabs<T> {
    /// the tab row's
    entries: Vec<(TabId, T)>,
    /// each in a window of its own: in no row, and never the active one
    detached: Vec<(TabId, T)>,
    active: Option<TabId>,
    /// the tab that goes away once another is shown; only ever the active one
    temporary: Option<TabId>,
    /// the last tab opened behind and the tab shown then: the next one opened behind goes right of it while that
    /// tab is still the shown one
    behind: Option<(TabId, TabId)>,
    next_id: u64,
}

impl<T> Default for Tabs<T> {
    fn default() -> Self {
        Self { entries: Vec::new(), detached: Vec::new(), active: None, temporary: None, behind: None, next_id: 0 }
    }
}

impl<T> Tabs<T> {
    /// Opens `make(id)` right of the active tab, as IntelliJ does, and shows it; in place of a temporary tab, which
    /// goes.
    pub fn open_with(&mut self, make: impl FnOnce(TabId) -> T) -> TabId {
        let id = TabId(self.next_id);
        self.next_id += 1;
        let at = self.insertion_index();
        self.entries.insert(at, (id, make(id)));
        self.active = Some(id);
        id
    }

    pub fn open(&mut self, item: T) -> TabId {
        self.open_with(|_| item)
    }

    /// Opens `item` as `open` does, to go away once another tab is shown.
    pub fn open_temporary(&mut self, item: T) -> TabId {
        let id = self.open(item);
        self.temporary = Some(id);
        id
    }

    #[cfg(test)]
    /// Opens `item` right of the shown tab without showing it, permanent: the shown tab, temporary or not, stays.
    /// Tabs opened behind one after another keep their order.
    pub fn open_behind(&mut self, item: T) -> TabId {
        let id = TabId(self.next_id);
        self.next_id += 1;
        self.entries.push((id, item));
        self.place_behind(id);
        id
    }

    /// Moves row tab `id` right of the shown tab, after the tabs placed there before while that tab stays shown;
    /// the shown tab itself stays where it is.
    pub fn place_behind(&mut self, id: TabId) {
        if self.active == Some(id) {
            return;
        }
        let Some(index) = self.index_of(id) else {
            return;
        };
        let entry = self.entries.remove(index);
        let after = match (self.behind, self.active) {
            (Some((shown, last)), Some(active)) if shown == active => self.index_of(last).or_else(|| self.index_of(active)),
            (_, active) => active.and_then(|active| self.index_of(active)),
        };
        let at = after.map_or(self.entries.len(), |index| index + 1);
        self.entries.insert(at, entry);
        self.behind = self.active.map(|active| (active, id));
    }

    /// Makes the shown tab `id` temporary (again); nothing when another tab is shown or one is temporary.
    pub fn make_temporary(&mut self, id: TabId) {
        if self.active == Some(id) && self.temporary.is_none() {
            self.temporary = Some(id);
        }
    }

    /// Makes tab `id` stay when another is shown; a permanent tab is left as it is.
    pub fn keep(&mut self, id: TabId) {
        if self.temporary == Some(id) {
            self.temporary = None;
        }
    }

    pub fn is_temporary(&self, id: TabId) -> bool {
        self.temporary == Some(id)
    }

    /// The shown tab, or, when it is temporary, the one closing it would show.
    pub fn settled_active(&self) -> Option<TabId> {
        let Some(temporary) = self.temporary else {
            return self.active;
        };
        let index = self.index_of(temporary)?;
        let neighbour = self.entries.get(index + 1).or_else(|| index.checked_sub(1).and_then(|left| self.entries.get(left)));
        neighbour.map(|(id, _)| *id)
    }

    pub fn find(&self, matches: impl Fn(&T) -> bool) -> Option<TabId> {
        self.iter().find(|(_, item)| matches(item)).map(|(id, _)| id)
    }

    /// Shows tab `id`, closing a temporary tab shown before; an id that is no longer open, or detached, is ignored.
    pub fn activate(&mut self, id: TabId) {
        if self.index_of(id).is_some() && self.active != Some(id) {
            self.take_temporary();
            self.active = Some(id);
        }
    }

    pub fn active_id(&self) -> Option<TabId> {
        self.active
    }

    /// The shown tab when `matches` it; a detached tab is never the shown one.
    pub fn active_if(&self, matches: impl Fn(&T) -> bool) -> Option<TabId> {
        self.active.filter(|id| self.get(*id).is_some_and(&matches))
    }

    pub fn active(&self) -> Option<&T> {
        self.active.and_then(|id| self.get(id))
    }

    pub fn get(&self, id: TabId) -> Option<&T> {
        self.iter().find(|(entry, _)| *entry == id).map(|(_, item)| item)
    }

    pub fn get_mut(&mut self, id: TabId) -> Option<&mut T> {
        self.entries.iter_mut().chain(self.detached.iter_mut()).find(|(entry, _)| *entry == id).map(|(_, item)| item)
    }

    /// Swaps what tab `id` holds, keeping its place; returns the old content.
    pub fn replace(&mut self, id: TabId, item: T) -> Option<T> {
        self.get_mut(id).map(|slot| std::mem::replace(slot, item))
    }

    /// Closes tab `id`; when it was shown, its right neighbour is shown instead, or the left one at the end.
    pub fn close(&mut self, id: TabId) -> Option<T> {
        if let Some(item) = self.take_detached(id) {
            return Some(item);
        }
        let index = self.index_of(id)?;
        self.keep(id);
        let (_, item) = self.entries.remove(index);
        if self.active == Some(id) {
            let neighbour = self.entries.get(index).or_else(|| index.checked_sub(1).and_then(|left| self.entries.get(left)));
            self.active = neighbour.map(|(id, _)| *id);
        }
        Some(item)
    }

    /// Closes the row's tabs `which` of tab `id`, detached ones staying; when the shown tab was among them, `id` is
    /// shown instead.
    pub fn close_beside(&mut self, id: TabId, which: Beside) -> Vec<T> {
        let Some(index) = self.index_of(id) else {
            return Vec::new();
        };
        let range = match which {
            Beside::Left => 0..index,
            Beside::Right => index + 1..self.entries.len(),
            Beside::Others => 0..self.entries.len(),
        };
        let (closed, kept): (Vec<_>, Vec<_>) =
            std::mem::take(&mut self.entries).into_iter().enumerate().partition(|(place, (entry, _))| range.contains(place) && *entry != id);
        self.entries = kept.into_iter().map(|(_, entry)| entry).collect();
        if self.temporary.is_some_and(|temporary| self.index_of(temporary).is_none()) {
            self.temporary = None;
        }
        if self.active.is_some_and(|active| self.index_of(active).is_none()) {
            self.active = Some(id);
        }
        closed.into_iter().map(|(_, (_, item))| item).collect()
    }

    /// Whether [`close_beside`](Self::close_beside) would close any tab.
    pub fn has_beside(&self, id: TabId, which: Beside) -> bool {
        self.index_of(id).is_some_and(|index| match which {
            Beside::Left => index > 0,
            Beside::Right => index + 1 < self.entries.len(),
            Beside::Others => self.entries.len() > 1,
        })
    }

    /// Closes tab `id` that turned out to show nothing (a restored tab whose target is gone); when it was shown,
    /// its left neighbour is shown instead, as the saved shown tab falls back at start, or the right one at the
    /// start.
    pub fn drop_tab(&mut self, id: TabId) -> Option<T> {
        if let Some(item) = self.take_detached(id) {
            return Some(item);
        }
        let index = self.index_of(id)?;
        self.keep(id);
        if self.active == Some(id) {
            let neighbour = index.checked_sub(1).and_then(|left| self.entries.get(left)).or_else(|| self.entries.get(index + 1));
            self.active = neighbour.map(|(id, _)| *id);
        }
        Some(self.entries.remove(index).1)
    }

    /// Moves tab `id` into gap `gap` between tabs (0 before the first, the tab count after the last), keeping which
    /// one is shown.
    pub fn move_tab(&mut self, id: TabId, gap: usize) {
        if let Some(from) = self.index_of(id) {
            let entry = self.entries.remove(from);
            let to = if gap > from { gap - 1 } else { gap };
            self.entries.insert(to.min(self.entries.len()), entry);
        }
    }

    /// Whether moving tab `id` into gap `gap` changes the order: the gaps on either side of it do not.
    pub fn moves(&self, id: TabId, gap: usize) -> bool {
        self.index_of(id).is_some_and(|from| gap != from && gap != from + 1 && gap <= self.entries.len())
    }

    /// Takes tab `id` out of the row into a window of its own, to stay; when it was shown, the neighbour that
    /// closing it would show is shown instead. False when it is not in the row.
    pub fn detach(&mut self, id: TabId) -> bool {
        let Some(item) = self.index_of(id).and_then(|_| self.close(id)) else {
            return false;
        };
        self.detached.push((id, item));
        true
    }

    /// Puts detached tab `id` back into the row, right of the shown tab or in place of a temporary one, and shows
    /// it; false when it is not detached.
    pub fn attach(&mut self, id: TabId) -> bool {
        let Some(item) = self.take_detached(id) else {
            return false;
        };
        let at = self.insertion_index();
        self.entries.insert(at, (id, item));
        self.active = Some(id);
        true
    }

    pub fn is_detached(&self, id: TabId) -> bool {
        self.detached.iter().any(|(entry, _)| *entry == id)
    }

    /// The tabs on screen: the shown one, then the detached ones.
    pub fn on_screen(&self) -> impl Iterator<Item = (TabId, &T)> {
        let active = self.active.and_then(|id| Some((id, self.get(id)?)));
        active.into_iter().chain(self.detached.iter().map(|(id, item)| (*id, item)))
    }

    /// The tab row's tabs, in order.
    pub fn row(&self) -> impl Iterator<Item = (TabId, &T)> {
        self.entries.iter().map(|(id, item)| (*id, item))
    }

    pub fn row_len(&self) -> usize {
        self.entries.len()
    }

    /// Every tab: the row's in order, then the detached ones.
    pub fn iter(&self) -> impl Iterator<Item = (TabId, &T)> {
        self.row().chain(self.detached.iter().map(|(id, item)| (*id, item)))
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty() && self.detached.is_empty()
    }

    fn take_detached(&mut self, id: TabId) -> Option<T> {
        let index = self.detached.iter().position(|(entry, _)| *entry == id)?;
        Some(self.detached.remove(index).1)
    }

    /// Where a tab being shown goes: the temporary tab's place, which it leaves, or right of the shown tab.
    fn insertion_index(&mut self) -> usize {
        if let Some(index) = self.take_temporary() {
            return index;
        }
        self.active.and_then(|active| self.index_of(active)).map_or(self.entries.len(), |index| index + 1)
    }

    /// Removes the temporary tab, if any, leaving `active` to the caller; the index it had.
    fn take_temporary(&mut self) -> Option<usize> {
        let temporary = self.temporary.take()?;
        let index = self.index_of(temporary)?;
        self.entries.remove(index);
        Some(index)
    }

    fn index_of(&self, id: TabId) -> Option<usize> {
        self.entries.iter().position(|(entry, _)| *entry == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The row's titles: the shown one in brackets, a temporary one starred.
    fn titles(tabs: &Tabs<&str>) -> Vec<String> {
        tabs.row()
            .map(|(id, item)| if tabs.is_temporary(id) { (id, format!("{item}*")) } else { (id, item.to_string()) })
            .map(|(id, title)| if tabs.active_id() == Some(id) { format!("[{title}]") } else { title })
            .collect()
    }

    #[test]
    fn should_replace_the_temporary_tab_with_a_new_one_in_its_place() {
        // given
        let mut tabs = Tabs::default();
        let a = tabs.open("a");
        tabs.open("b");
        tabs.activate(a);
        tabs.open_temporary("t");

        // when
        tabs.open("n");

        // then
        assert_eq!(titles(&tabs), ["a", "[n]", "b"]);
    }

    #[test]
    fn should_remove_the_temporary_tab_when_another_is_activated() {
        // given
        let mut tabs = Tabs::default();
        let a = tabs.open("a");
        tabs.open("b");
        tabs.open_temporary("t");

        // when
        tabs.activate(a);

        // then
        assert_eq!(titles(&tabs), ["[a]", "b"]);
    }

    #[test]
    fn should_keep_the_temporary_tab_when_another_tab_closes() {
        // given
        let mut tabs = Tabs::default();
        let a = tabs.open("a");
        tabs.open_temporary("t");

        // when
        tabs.close(a);

        // then
        assert_eq!(titles(&tabs), ["[t*]"]);
    }

    #[test]
    fn should_keep_a_tab_made_permanent() {
        // given
        let mut tabs = Tabs::default();
        let a = tabs.open("a");
        let t = tabs.open_temporary("t");

        // when
        tabs.keep(t);
        tabs.activate(a);

        // then
        assert_eq!(titles(&tabs), ["[a]", "t"]);
    }

    #[test]
    fn should_open_into_an_empty_row_when_the_temporary_tab_was_alone() {
        // given
        let mut tabs = Tabs::default();
        tabs.open_temporary("t");

        // when
        tabs.open("n");

        // then
        assert_eq!(titles(&tabs), ["[n]"]);
    }

    #[test]
    fn should_replace_the_temporary_tab_with_an_attached_one() {
        // given
        let mut tabs = Tabs::default();
        tabs.open("a");
        let d = tabs.open("d");
        tabs.detach(d);
        tabs.open_temporary("t");

        // when
        tabs.attach(d);

        // then
        assert_eq!(titles(&tabs), ["a", "[d]"]);
    }

    #[test]
    fn should_keep_a_detached_temporary_tab() {
        // given
        let mut tabs = Tabs::default();
        let a = tabs.open("a");
        let t = tabs.open_temporary("t");
        tabs.detach(t);
        tabs.attach(t);

        // when
        tabs.activate(a);

        // then
        assert_eq!(titles(&tabs), ["[a]", "t"]);
    }

    #[test]
    fn should_settle_on_the_neighbour_of_a_shown_temporary_tab() {
        // given
        let mut tabs = Tabs::default();
        let b = tabs.open("b");
        let t = tabs.open_temporary("t");
        tabs.move_tab(t, 0);

        // when
        let settled = tabs.settled_active();

        // then
        assert_eq!(titles(&tabs), ["[t*]", "b"]);
        assert_eq!(settled, Some(b));
    }

    #[test]
    fn should_not_find_the_shown_tab_for_a_row_whose_tab_is_detached() {
        // given
        let mut tabs = Tabs::default();
        let worktree = tabs.open("worktree");
        tabs.detach(worktree);
        tabs.open_temporary("settings");

        // when
        let found = tabs.active_if(|tab| *tab == "worktree");

        // then
        assert_eq!(found, None);
    }

    #[test]
    fn should_find_the_shown_tab_when_it_matches() {
        // given
        let mut tabs = Tabs::default();
        let project = tabs.open_temporary("project");

        // when
        let found = tabs.active_if(|tab| *tab == "project");

        // then
        assert_eq!(found, Some(project));
    }

    #[test]
    fn should_settle_on_a_shown_permanent_tab() {
        // given
        let mut tabs = Tabs::default();
        let a = tabs.open("a");

        // when
        let settled = tabs.settled_active();

        // then
        assert_eq!(settled, Some(a));
    }

    #[test]
    fn should_close_the_tabs_left_of_a_tab_keeping_the_shown_one() {
        // given
        let mut tabs = Tabs::default();
        tabs.open("a");
        let b = tabs.open("b");
        tabs.open("c");

        // when
        let closed = tabs.close_beside(b, Beside::Left);

        // then
        assert_eq!((closed, titles(&tabs)), (vec!["a"], vec!["b".to_owned(), "[c]".to_owned()]));
    }

    #[test]
    fn should_show_the_tab_whose_right_tabs_closed_with_the_shown_one() {
        // given
        let mut tabs = Tabs::default();
        let a = tabs.open("a");
        tabs.open("b");
        tabs.open("c");

        // when
        tabs.close_beside(a, Beside::Right);

        // then
        assert_eq!(titles(&tabs), ["[a]"]);
    }

    #[test]
    fn should_close_the_other_tabs_leaving_detached_ones() {
        // given
        let mut tabs = Tabs::default();
        let d = tabs.open("d");
        tabs.detach(d);
        tabs.open("a");
        let b = tabs.open("b");
        tabs.open_temporary("t");

        // when
        tabs.close_beside(b, Beside::Others);

        // then
        assert_eq!((titles(&tabs), tabs.is_detached(d), tabs.is_temporary(b)), (vec!["[b]".to_owned()], true, false));
    }

    #[test]
    fn should_tell_whether_there_are_tabs_beside_a_tab() {
        // given
        let mut tabs = Tabs::default();
        let a = tabs.open("a");
        let b = tabs.open("b");
        let d = tabs.open("d");
        tabs.detach(d);

        // when
        let beside = [Beside::Left, Beside::Right, Beside::Others].map(|which| (tabs.has_beside(a, which), tabs.has_beside(b, which)));

        // then
        assert_eq!(beside, [(false, true), (true, false), (true, true)]);
    }

    #[test]
    fn should_show_the_left_neighbour_when_a_shown_tab_is_dropped() {
        // given
        let mut tabs = Tabs::default();
        tabs.open("a");
        let dropped = tabs.open("b");
        tabs.open("c");
        tabs.activate(dropped);

        // when
        tabs.drop_tab(dropped);

        // then
        assert_eq!(titles(&tabs), ["[a]", "c"]);
    }

    #[test]
    fn should_show_the_right_neighbour_when_the_dropped_tab_was_first() {
        // given
        let mut tabs = Tabs::default();
        let dropped = tabs.open("a");
        tabs.open("b");
        tabs.activate(dropped);

        // when
        tabs.drop_tab(dropped);

        // then
        assert_eq!(titles(&tabs), ["[b]"]);
    }

    #[test]
    fn should_open_new_tab_right_of_active_one_and_show_it() {
        // given
        let mut tabs = Tabs::default();
        let first = tabs.open("a");
        tabs.open("b");
        tabs.activate(first);

        // when
        tabs.open("c");

        // then
        assert_eq!(titles(&tabs), ["a", "[c]", "b"]);
    }

    #[test]
    fn should_show_right_neighbour_when_active_tab_closes() {
        // given
        let mut tabs = Tabs::default();
        tabs.open("a");
        let middle = tabs.open("b");
        tabs.open("c");
        tabs.activate(middle);

        // when
        let closed = tabs.close(middle);

        // then
        assert_eq!((closed, titles(&tabs)), (Some("b"), vec!["a".to_owned(), "[c]".to_owned()]));
    }

    #[test]
    fn should_show_left_neighbour_when_last_active_tab_closes() {
        // given
        let mut tabs = Tabs::default();
        tabs.open("a");
        let last = tabs.open("b");

        // when
        tabs.close(last);

        // then
        assert_eq!(titles(&tabs), ["[a]"]);
    }

    #[test]
    fn should_keep_shown_tab_when_background_tab_closes() {
        // given
        let mut tabs = Tabs::default();
        let background = tabs.open("a");
        tabs.open("b");

        // when
        tabs.close(background);

        // then
        assert_eq!(titles(&tabs), ["[b]"]);
    }

    #[test]
    fn should_have_nothing_shown_after_closing_last_tab() {
        // given
        let mut tabs = Tabs::default();
        let only = tabs.open("a");

        // when
        tabs.close(only);

        // then
        assert_eq!((tabs.is_empty(), tabs.active_id()), (true, None));
    }

    #[test]
    fn should_keep_ids_stable_and_unique_after_closing() {
        // given
        let mut tabs = Tabs::default();
        let first = tabs.open("a");
        let second = tabs.open("b");
        tabs.close(first);

        // when
        let third = tabs.open("c");
        tabs.replace(second, "b2");

        // then
        assert_eq!((third == first, tabs.get(second), tabs.find(|item| *item == "c")), (false, Some(&"b2"), Some(third)));
    }

    #[test]
    fn should_move_tab_after_target_when_dragged_right() {
        // given
        let mut tabs = Tabs::default();
        let moved = tabs.open("a");
        tabs.open("b");
        tabs.open("c");

        // when
        tabs.move_tab(moved, 2);

        // then
        assert_eq!(titles(&tabs), ["b", "a", "[c]"]);
    }

    #[test]
    fn should_move_tab_before_target_when_dragged_left() {
        // given
        let mut tabs = Tabs::default();
        tabs.open("a");
        tabs.open("b");
        let moved = tabs.open("c");

        // when
        tabs.move_tab(moved, 0);

        // then
        assert_eq!(titles(&tabs), ["[c]", "a", "b"]);
    }

    #[test]
    fn should_move_tab_last_into_gap_after_last_tab() {
        // given
        let mut tabs = Tabs::default();
        let moved = tabs.open("a");
        tabs.open("b");

        // when
        tabs.move_tab(moved, 2);

        // then
        assert_eq!(titles(&tabs), ["[b]", "a"]);
    }

    #[test]
    fn should_not_count_gaps_beside_the_tab_as_moves() {
        // given
        let mut tabs = Tabs::default();
        tabs.open("a");
        let moved = tabs.open("b");
        tabs.open("c");

        // when
        let moves: Vec<bool> = (0..=4).map(|gap| tabs.moves(moved, gap)).collect();

        // then
        assert_eq!(moves, [true, false, false, true, false]);
    }

    #[test]
    fn should_ignore_moving_closed_tab() {
        // given
        let mut tabs = Tabs::default();
        let gone = tabs.open("a");
        tabs.open("b");
        tabs.close(gone);

        // when
        tabs.move_tab(gone, 0);

        // then
        assert_eq!(titles(&tabs), ["[b]"]);
    }

    #[test]
    fn should_ignore_activating_closed_tab() {
        // given
        let mut tabs = Tabs::default();
        let gone = tabs.open("a");
        tabs.open("b");
        tabs.close(gone);

        // when
        tabs.activate(gone);

        // then
        assert_eq!(titles(&tabs), ["[b]"]);
    }

    #[test]
    fn should_show_the_neighbour_when_the_shown_tab_is_detached() {
        // given
        let mut tabs = Tabs::default();
        tabs.open("a");
        let detached = tabs.open("b");
        tabs.open("c");
        tabs.activate(detached);

        // when
        let moved = tabs.detach(detached);

        // then
        assert_eq!((moved, titles(&tabs), tabs.is_detached(detached)), (true, vec!["a".to_owned(), "[c]".to_owned()], true));
    }

    #[test]
    fn should_keep_finding_a_detached_tab() {
        // given
        let mut tabs = Tabs::default();
        let detached = tabs.open("a");
        tabs.open("b");
        tabs.detach(detached);

        // when
        let found = (tabs.find(|item| *item == "a"), tabs.get(detached).copied(), tabs.iter().count());

        // then
        assert_eq!(found, (Some(detached), Some("a"), 2));
    }

    #[test]
    fn should_not_show_a_detached_tab_in_the_main_window() {
        // given
        let mut tabs = Tabs::default();
        let detached = tabs.open("a");
        tabs.open("b");
        tabs.detach(detached);

        // when
        tabs.activate(detached);

        // then
        assert_eq!(titles(&tabs), ["[b]"]);
    }

    #[test]
    fn should_attach_right_of_the_shown_tab_and_show_it() {
        // given
        let mut tabs = Tabs::default();
        let detached = tabs.open("a");
        let shown = tabs.open("b");
        tabs.open("c");
        tabs.detach(detached);
        tabs.activate(shown);

        // when
        let attached = tabs.attach(detached);

        // then
        assert_eq!((attached, titles(&tabs), tabs.is_detached(detached)), (true, vec!["b".to_owned(), "[a]".to_owned(), "c".to_owned()], false));
    }

    #[test]
    fn should_close_a_detached_tab_keeping_the_shown_one() {
        // given
        let mut tabs = Tabs::default();
        let detached = tabs.open("a");
        tabs.open("b");
        tabs.detach(detached);

        // when
        let closed = tabs.close(detached);

        // then
        assert_eq!((closed, titles(&tabs), tabs.iter().count()), (Some("a"), vec!["[b]".to_owned()], 1));
    }

    #[test]
    fn should_count_the_shown_and_the_detached_tabs_as_on_screen() {
        // given
        let mut tabs = Tabs::default();
        let detached = tabs.open("a");
        tabs.open("b");
        tabs.open("c");
        tabs.detach(detached);

        // when
        let on_screen: Vec<&str> = tabs.on_screen().map(|(_, item)| *item).collect();

        // then
        assert_eq!(on_screen, ["c", "a"]);
    }

    #[test]
    fn should_open_a_tab_behind_the_shown_one() {
        // given
        let mut tabs = Tabs::default();
        let a = tabs.open("a");
        tabs.open("b");
        tabs.activate(a);

        // when
        tabs.open_behind("x");

        // then
        assert_eq!(titles(&tabs), ["[a]", "x", "b"]);
    }

    #[test]
    fn should_keep_the_temporary_tab_shown_when_opening_behind() {
        // given
        let mut tabs = Tabs::default();
        tabs.open("a");
        tabs.open_temporary("h");

        // when
        tabs.open_behind("x");

        // then
        assert_eq!(titles(&tabs), ["a", "[h*]", "x"]);
    }

    #[test]
    fn should_keep_the_order_of_tabs_opened_behind() {
        // given
        let mut tabs = Tabs::default();
        tabs.open_temporary("h");

        // when
        tabs.open_behind("x");
        tabs.open_behind("y");

        // then
        assert_eq!(titles(&tabs), ["[h*]", "x", "y"]);
    }

    #[test]
    fn should_open_behind_the_newly_shown_tab_after_a_switch() {
        // given
        let mut tabs = Tabs::default();
        let a = tabs.open("a");
        let b = tabs.open("b");
        tabs.activate(a);
        tabs.open_behind("x");
        tabs.activate(b);

        // when
        tabs.open_behind("y");

        // then
        assert_eq!(titles(&tabs), ["a", "x", "[b]", "y"]);
    }
}
