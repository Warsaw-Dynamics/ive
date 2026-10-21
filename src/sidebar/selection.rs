//! The tree's selection: the rows a plain click, Ctrl+click and Shift+click picked, and the anchor a Shift+click
//! ranges from. Only files and folders can be selected together.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use super::rows::{TreeItem, TreeKey, TreeRow};
use crate::file_tree::EntryKind;

/// A row as the selection sees it: its key, and whether it can be selected with others.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Listed {
    pub key: TreeKey,
    pub multi: bool,
}

/// `rows` as the selection sees them; files and folders can be selected with others, nothing else can.
pub fn listed(rows: &[TreeRow]) -> Vec<Listed> {
    rows.iter().map(|row| Listed { key: row.key(), multi: is_multi_selectable(row) }).collect()
}

pub fn is_multi_selectable(row: &TreeRow) -> bool {
    matches!(&row.item, TreeItem::Entry(entry) if matches!(entry.entry.kind, EntryKind::File | EntryKind::Folder))
}

/// Keys whose rows are not listed (a collapsed folder's) stay selected, unseen, until the selection changes.
#[derive(Debug, Default)]
pub struct Selection {
    keys: HashSet<TreeKey>,
    /// the row last clicked or Ctrl+clicked: where a Shift+click ranges from, and the row Alt+F12 and the
    /// crosshair go by
    anchor: Option<TreeKey>,
}

impl Selection {
    pub fn select_only(&mut self, key: TreeKey) {
        self.keys = HashSet::from([key.clone()]);
        self.anchor = Some(key);
    }

    pub fn toggle(&mut self, key: TreeKey) {
        if !self.keys.remove(&key) {
            self.keys.insert(key.clone());
        }
        self.anchor = Some(key);
    }

    /// Selects the files and folders listed from the anchor to `to`; false, changing nothing, when the anchor is
    /// not listed or either end is not a file or folder.
    pub fn extend(&mut self, to: &TreeKey, listed: &[Listed]) -> bool {
        let position = |key: &TreeKey| listed.iter().position(|row| row.key == *key && row.multi);
        let Some((from, to)) = self.anchor.as_ref().and_then(position).zip(position(to)) else {
            return false;
        };
        let range = from.min(to)..=from.max(to);
        self.keys = listed[range].iter().filter(|row| row.multi).map(|row| row.key.clone()).collect();
        true
    }

    pub fn contains(&self, key: &TreeKey) -> bool {
        self.keys.contains(key)
    }

    pub fn anchor(&self) -> Option<&TreeKey> {
        self.anchor.as_ref()
    }

    /// What dragging the row of `pressed` drags: the whole selection when it is selected, else that row alone.
    pub fn dragged_paths(&self, pressed: &Path, listed: &[Listed]) -> Vec<PathBuf> {
        if self.contains(&TreeKey::Entry(pressed.to_path_buf())) {
            return self.listed_paths(listed);
        }
        vec![pressed.to_path_buf()]
    }

    /// The paths of the selected files and folders that are listed, in the tree's order.
    pub fn listed_paths(&self, listed: &[Listed]) -> Vec<PathBuf> {
        listed
            .iter()
            .filter(|row| row.multi && self.contains(&row.key))
            .filter_map(|row| match &row.key {
                TreeKey::Entry(path) => Some(path.clone()),
                _ => None,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(name: &str) -> Listed {
        Listed { key: TreeKey::Entry(PathBuf::from(format!("/p/{name}"))), multi: true }
    }

    fn other(name: &str) -> Listed {
        Listed { key: TreeKey::CommitsMore(PathBuf::from(format!("/p/{name}"))), multi: false }
    }

    fn keys(selection: &Selection, listed: &[Listed]) -> Vec<TreeKey> {
        listed.iter().map(|row| row.key.clone()).filter(|key| selection.contains(key)).collect()
    }

    #[test]
    fn should_select_only_the_clicked_row() {
        // given
        let rows = [file("a"), file("b")];
        let mut selection = Selection::default();
        selection.toggle(rows[0].key.clone());

        // when
        selection.select_only(rows[1].key.clone());

        // then
        assert_eq!((keys(&selection, &rows), selection.anchor()), (vec![rows[1].key.clone()], Some(&rows[1].key)));
    }

    #[test]
    fn should_add_and_remove_row_on_toggle() {
        // given
        let rows = [file("a"), file("b")];
        let mut selection = Selection::default();

        // when
        selection.toggle(rows[0].key.clone());
        selection.toggle(rows[1].key.clone());
        selection.toggle(rows[0].key.clone());

        // then
        assert_eq!((keys(&selection, &rows), selection.anchor()), (vec![rows[1].key.clone()], Some(&rows[0].key)));
    }

    #[test]
    fn should_empty_selection_when_only_row_toggled_off() {
        // given
        let rows = [file("a")];
        let mut selection = Selection::default();
        selection.select_only(rows[0].key.clone());

        // when
        selection.toggle(rows[0].key.clone());

        // then
        assert_eq!((keys(&selection, &rows), selection.anchor()), (vec![], Some(&rows[0].key)));
    }

    #[test]
    fn should_select_range_skipping_rows_that_are_not_files() {
        // given
        let rows = [file("f1"), other("run"), file("f2"), file("f3"), file("f4")];
        let mut selection = Selection::default();
        selection.select_only(rows[0].key.clone());

        // when
        let extended = selection.extend(&rows[3].key, &rows);

        // then
        assert!(extended);
        assert_eq!(keys(&selection, &rows), vec![rows[0].key.clone(), rows[2].key.clone(), rows[3].key.clone()]);
        assert_eq!(selection.anchor(), Some(&rows[0].key));
    }

    #[test]
    fn should_select_range_upwards() {
        // given
        let rows = [file("a"), file("b"), file("c")];
        let mut selection = Selection::default();
        selection.select_only(rows[2].key.clone());

        // when
        selection.extend(&rows[1].key, &rows);

        // then
        assert_eq!(keys(&selection, &rows), vec![rows[1].key.clone(), rows[2].key.clone()]);
    }

    #[test]
    fn should_refuse_range_without_listed_anchor() {
        // given
        let rows = [file("a"), file("b")];
        let mut selection = Selection::default();
        selection.select_only(file("collapsed-away").key);

        // when
        let extended = selection.extend(&rows[1].key, &rows);

        // then
        assert!(!extended);
    }

    #[test]
    fn should_refuse_range_to_a_row_that_is_not_a_file() {
        // given
        let rows = [file("a"), other("run")];
        let mut selection = Selection::default();
        selection.select_only(rows[0].key.clone());

        // when
        let extended = selection.extend(&rows[1].key, &rows);

        // then
        assert!(!extended);
    }

    #[test]
    fn should_drag_only_the_pressed_row_when_it_is_not_selected() {
        // given
        let rows = [file("a"), file("b")];
        let mut selection = Selection::default();
        selection.select_only(rows[0].key.clone());

        // when
        let dragged = selection.dragged_paths(&PathBuf::from("/p/b"), &rows);

        // then
        assert_eq!(dragged, [PathBuf::from("/p/b")]);
    }

    #[test]
    fn should_drag_the_whole_selection_when_the_pressed_row_is_selected() {
        // given
        let rows = [file("a"), file("b"), file("c")];
        let mut selection = Selection::default();
        selection.toggle(rows[2].key.clone());
        selection.toggle(rows[0].key.clone());

        // when
        let dragged = selection.dragged_paths(&PathBuf::from("/p/c"), &rows);

        // then
        assert_eq!(dragged, ["/p/a", "/p/c"].map(PathBuf::from));
    }

    #[test]
    fn should_list_selected_paths_in_tree_order() {
        // given
        let rows = [file("a"), file("b"), file("c")];
        let mut selection = Selection::default();
        selection.toggle(rows[2].key.clone());
        selection.toggle(rows[0].key.clone());

        // when
        let paths = selection.listed_paths(&rows);

        // then
        assert_eq!(paths, ["/p/a", "/p/c"].map(PathBuf::from));
    }

    #[test]
    fn should_leave_out_selected_rows_not_listed() {
        // given
        let rows = [file("a")];
        let mut selection = Selection::default();
        selection.toggle(rows[0].key.clone());
        selection.toggle(file("collapsed-away").key);

        // when
        let paths = selection.listed_paths(&rows);

        // then
        assert_eq!(paths, [PathBuf::from("/p/a")]);
    }
}
