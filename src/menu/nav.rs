//! Moving through an open menu: the highlighted line and the open submenus, by item id, so they stay on the same
//! items while pending lines are answered and lines come and go.

use super::resolve::{ItemId, Line, LineState};

/// The open submenus, outermost first, and the highlighted line, which is in the innermost one (or the menu itself).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Nav {
    open: Vec<ItemId>,
    highlighted: Vec<ItemId>,
}

impl Nav {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn highlighted(&self) -> Option<&ItemId> {
        self.highlighted.first()
    }

    pub fn open(&self) -> &[ItemId] {
        &self.open
    }

    /// Whether no submenu is open: Left or Escape then closes the menu.
    pub fn is_at_top(&self) -> bool {
        self.open.is_empty()
    }

    /// The next line that can be chosen in the innermost open level, wrapping.
    pub fn down(&self, lines: &[Line]) -> Self {
        self.step(lines, Direction::Down)
    }

    pub fn up(&self, lines: &[Line]) -> Self {
        self.step(lines, Direction::Up)
    }

    /// Opens the highlighted submenu, highlighting its first line; anything else stays.
    pub fn right(&self, lines: &[Line]) -> Self {
        let Some(Line::Submenu { id, lines: inner, state: LineState::Ready, .. }) = self.chosen(lines) else {
            return self.clone();
        };
        let open = [self.open.clone(), vec![id.clone()]].concat();
        let first = choosable(inner).next().cloned();
        Self { open, highlighted: first.into_iter().collect() }
    }

    /// Closes the innermost submenu, highlighting its line.
    pub fn left(&self) -> Self {
        let mut open = self.open.clone();
        let closed = open.pop();
        Self { open, highlighted: closed.into_iter().collect() }
    }

    /// Highlights `id` and opens the submenus it is in, closing others.
    pub fn hover(&self, id: ItemId) -> Self {
        let open = (1..id.len()).map(|length| id[..length].to_vec()).collect();
        Self { open, highlighted: vec![id] }
    }

    /// The highlighted line, wherever it is now.
    pub fn chosen<'a>(&self, lines: &'a [Line]) -> Option<&'a Line> {
        self.highlighted().and_then(|id| find(lines, id))
    }

    /// After `before` became `after`: submenus that went are closed; a highlighted line that went passes the highlight
    /// to the next line that came after it, else the one before.
    pub fn settle(&self, before: &[Line], after: &[Line]) -> Self {
        let open: Vec<ItemId> = self.open.iter().take_while(|id| find(after, id).is_some()).cloned().collect();
        let Some(highlighted) = self.highlighted() else {
            return Self { open, highlighted: Vec::new() };
        };
        if find(after, highlighted).is_some() && open.len() == self.open.len() {
            return Self { open, highlighted: self.highlighted.clone() };
        }
        let old = level(before, &self.open);
        let new: Vec<&ItemId> = choosable(level(after, &open)).collect();
        let position = old.iter().position(|line| line.id() == Some(highlighted)).unwrap_or(0);
        let later = old[position..].iter().filter_map(Line::id).find(|id| new.contains(id));
        let earlier = old[..position].iter().rev().filter_map(Line::id).find(|id| new.contains(id));
        Self { open, highlighted: later.or(earlier).cloned().into_iter().collect() }
    }

    fn step(&self, lines: &[Line], direction: Direction) -> Self {
        let candidates: Vec<&ItemId> = choosable(level(lines, &self.open)).collect();
        if candidates.is_empty() {
            return self.clone();
        }
        let at = self.highlighted().and_then(|id| candidates.iter().position(|candidate| *candidate == id));
        let next = match (at, direction) {
            (None, Direction::Down) => 0,
            (None, Direction::Up) => candidates.len() - 1,
            (Some(at), Direction::Down) => (at + 1) % candidates.len(),
            (Some(at), Direction::Up) => (at + candidates.len() - 1) % candidates.len(),
        };
        Self { open: self.open.clone(), highlighted: vec![candidates[next].clone()] }
    }
}

#[derive(Clone, Copy)]
enum Direction {
    Down,
    Up,
}

/// The lines of the innermost of `open`, or `lines` when none is.
fn level<'a>(lines: &'a [Line], open: &[ItemId]) -> &'a [Line] {
    match open.last().and_then(|id| find(lines, id)) {
        Some(Line::Submenu { lines, .. }) => lines,
        _ => lines,
    }
}

/// Lines that can be highlighted: ready items that are enabled, and ready submenus.
fn choosable(lines: &[Line]) -> impl Iterator<Item = &ItemId> {
    lines.iter().filter_map(|line| match line {
        Line::Item { id, state: LineState::Ready, enabled: true, .. } | Line::Submenu { id, state: LineState::Ready, .. } => Some(id),
        _ => None,
    })
}

/// The line of `id`, at any depth.
fn find<'a>(lines: &'a [Line], id: &ItemId) -> Option<&'a Line> {
    lines.iter().find_map(|line| match line {
        _ if line.id() == Some(id) => Some(line),
        Line::Submenu { id: submenu, lines, .. } if id.starts_with(submenu) => find(lines, id),
        _ => None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu::resolve::LineAction;
    use crate::script::keymap::Builtin;
    use crate::script::menus::ItemAction;

    fn item(id: &[usize]) -> Line {
        Line::Item { id: id.to_vec(), label: format!("{id:?}"), action: LineAction::Script(ItemAction::Builtin(Builtin::CloseTab)), state: LineState::Ready, enabled: true }
    }

    fn pending(id: &[usize]) -> Line {
        Line::Item { id: id.to_vec(), label: format!("{id:?}"), action: LineAction::Script(ItemAction::Builtin(Builtin::CloseTab)), state: LineState::Pending, enabled: true }
    }

    fn submenu(id: &[usize], lines: Vec<Line>) -> Line {
        Line::Submenu { id: id.to_vec(), label: "sub".to_owned(), lines, state: LineState::Ready }
    }

    fn lines() -> Vec<Line> {
        vec![item(&[0]), Line::Separator, pending(&[2]), submenu(&[3], vec![item(&[3, 0]), item(&[3, 1])]), item(&[4])]
    }

    #[test]
    fn should_skip_separators_and_pending_lines_moving_down() {
        // given
        let nav = Nav::new().down(&lines());

        // when
        let next = nav.down(&lines());

        // then
        assert_eq!((nav.highlighted(), next.highlighted()), (Some(&vec![0]), Some(&vec![3])));
    }

    #[test]
    fn should_wrap_moving_up_from_the_first_line() {
        // given
        let nav = Nav::new().down(&lines());

        // when
        let wrapped = nav.up(&lines());

        // then
        assert_eq!(wrapped.highlighted(), Some(&vec![4]));
    }

    #[test]
    fn should_open_a_submenu_on_right_and_close_on_left() {
        // given
        let on_submenu = Nav::new().down(&lines()).down(&lines());

        // when
        let opened = on_submenu.right(&lines());
        let closed = opened.left();

        // then
        assert_eq!((opened.highlighted(), opened.open()), (Some(&vec![3, 0]), &[vec![3]][..]));
        assert_eq!((closed.highlighted(), closed.open()), (Some(&vec![3]), &[][..]));
    }

    #[test]
    fn should_tell_left_at_the_top_level_closes_the_menu() {
        // when
        let at_top = Nav::new().down(&lines());

        // then
        assert!(at_top.is_at_top());
    }

    #[test]
    fn should_keep_the_highlight_on_the_same_item_when_a_line_above_goes() {
        // given
        let nav = Nav::new().down(&lines()).down(&lines());
        let after: Vec<Line> = lines().into_iter().filter(|line| line.id() != Some(&vec![0])).collect();

        // when
        let settled = nav.settle(&lines(), &after);

        // then
        assert_eq!(settled.highlighted(), Some(&vec![3]));
    }

    #[test]
    fn should_move_the_highlight_when_its_line_goes() {
        // given
        let nav = Nav::new().down(&lines()).down(&lines());
        let after: Vec<Line> = lines().into_iter().filter(|line| line.id() != Some(&vec![3])).collect();

        // when
        let settled = nav.settle(&lines(), &after);

        // then
        assert_eq!(settled.highlighted(), Some(&vec![4]));
    }

    #[test]
    fn should_open_submenus_along_a_hovered_path() {
        // when
        let nav = Nav::new().hover(vec![3, 1]);

        // then
        assert_eq!((nav.highlighted(), nav.open()), (Some(&vec![3, 1]), &[vec![3]][..]));
    }

    #[test]
    fn should_choose_the_highlighted_line() {
        // given
        let nav = Nav::new().hover(vec![3, 1]);

        let lines = lines();

        // when
        let chosen = nav.chosen(&lines);

        // then
        assert_eq!(chosen, Some(&item(&[3, 1])));
    }
}
