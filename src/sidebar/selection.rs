//! The tree's selection: the rows a plain click, Ctrl+click and Shift+click picked, and the anchor a Shift+click
//! ranges from. Files, folders and changed files are picked together, processes and runs together; never both.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use super::folders::change_path;
use super::rows::{TreeItem, TreeKey, TreeRow};
use crate::file_tree::EntryKind;
use crate::git::types::Mark;

/// What a row can be picked with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickGroup {
    Files,
    Processes,
}

/// A row as the selection sees it: its key, what it can be picked with (`None`: alone), and the path in the work tree
/// it stands for (a file or folder, a changed file that is there).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Listed {
    pub key: TreeKey,
    pub pick: Option<PickGroup>,
    pub path: Option<PathBuf>,
}

/// `rows` as the selection sees them.
pub fn listed(rows: &[TreeRow]) -> Vec<Listed> {
    rows.iter().map(|row| Listed { key: row.key(), pick: pick_group(row), path: work_path(row) }).collect()
}

/// Files, folders and changed files still in the work tree pick together, terminals, Claude Code and runs together;
/// nothing else picks with others.
pub fn pick_group(row: &TreeRow) -> Option<PickGroup> {
    match &row.item {
        TreeItem::Process(_) | TreeItem::LiveRun(_) => Some(PickGroup::Processes),
        _ => work_path(row).map(|_| PickGroup::Files),
    }
}

/// The path in the work tree that row `row` stands for: a file or folder's, a changed file's unless it was deleted.
pub fn work_path(row: &TreeRow) -> Option<PathBuf> {
    match &row.item {
        TreeItem::Entry(entry) if matches!(entry.entry.kind, EntryKind::File | EntryKind::Folder) => Some(entry.entry.path.clone()),
        TreeItem::Change(change) if change.change.mark != Mark::Deleted => Some(change_path(change)),
        _ => None,
    }
}

/// `paths` without repeats, the first of each kept.
pub fn distinct(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut seen = HashSet::new();
    paths.into_iter().filter(|path| seen.insert(path.clone())).collect()
}

/// `paths` without those inside another one of them, which goes with it.
pub fn without_nested(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let inside = |path: &PathBuf| paths.iter().any(|other| other != path && path.starts_with(other));
    paths.iter().filter(|path| !inside(path)).cloned().collect()
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

    /// Adds or removes `key`; a row of another group than the picked ones starts a new selection (a row not listed,
    /// in a collapsed folder, has no known group and differs from none).
    pub fn toggle(&mut self, key: TreeKey, listed: &[Listed]) {
        let group = |key: &TreeKey| listed.iter().find(|row| row.key == *key).and_then(|row| row.pick);
        let differs = |picked: &TreeKey| matches!((group(picked), group(&key)), (Some(a), Some(b)) if a != b);
        if self.keys.iter().any(differs) {
            return self.select_only(key);
        }
        if !self.keys.remove(&key) {
            self.keys.insert(key.clone());
        }
        self.anchor = Some(key);
    }

    /// Picks `keys` alone, the first as the anchor.
    pub fn pick_all(&mut self, keys: Vec<TreeKey>) {
        self.anchor = keys.first().cloned().or(self.anchor.take());
        self.keys = keys.into_iter().collect();
    }

    /// Drops the picks; the anchor stays, so a Shift+click still ranges from it.
    pub fn clear(&mut self) {
        self.keys.clear();
    }

    /// Selects the rows listed from the anchor to `to` of the anchor's group; false, changing nothing, when the anchor
    /// is not listed, either end cannot be picked with others, or the ends are of different groups.
    pub fn extend(&mut self, to: &TreeKey, listed: &[Listed]) -> bool {
        let position = |key: &TreeKey| listed.iter().position(|row| row.key == *key && row.pick.is_some());
        let Some((from, to)) = self.anchor.as_ref().and_then(position).zip(position(to)) else {
            return false;
        };
        let group = listed[from].pick;
        if listed[to].pick != group {
            return false;
        }
        let range = from.min(to)..=from.max(to);
        self.keys = listed[range].iter().filter(|row| row.pick == group).map(|row| row.key.clone()).collect();
        true
    }

    pub fn contains(&self, key: &TreeKey) -> bool {
        self.keys.contains(key)
    }

    pub fn anchor(&self) -> Option<&TreeKey> {
        self.anchor.as_ref()
    }

    /// What dragging the row of `pressed` drags: the listed picks of its group when it is picked, else that row alone.
    pub fn dragged_keys(&self, pressed: &TreeKey, listed: &[Listed]) -> Vec<TreeKey> {
        if !self.contains(pressed) {
            return vec![pressed.clone()];
        }
        let group = listed.iter().find(|row| row.key == *pressed).and_then(|row| row.pick);
        let picked: Vec<&Listed> = listed.iter().filter(|row| row.pick.is_some() && row.pick == group && self.contains(&row.key)).collect();
        let paths: Vec<&Path> = picked.iter().filter_map(|row| row.path.as_deref()).collect();
        // a file inside a picked folder goes with its folder
        let inside = |row: &&Listed| row.path.as_deref().is_some_and(|path| paths.iter().any(|folder| path != *folder && path.starts_with(folder)));
        picked.into_iter().filter(|row| !inside(row)).map(|row| row.key.clone()).collect()
    }

    /// The paths dragging the row of `pressed` drags: the whole selection's when it is selected, else that row's; a path
    /// picked twice (a file and its change) counts once.
    pub fn dragged_paths(&self, pressed: &TreeKey, listed: &[Listed]) -> Vec<PathBuf> {
        let path_of = |key: &TreeKey| match key {
            TreeKey::Entry(path) => Some(path.clone()),
            _ => listed.iter().find(|row| row.key == *key).and_then(|row| row.path.clone()),
        };
        distinct(self.dragged_keys(pressed, listed).iter().filter_map(path_of).collect())
    }

    /// The paths of the selected files, folders and changed files that are listed, in the tree's order, each once.
    pub fn listed_paths(&self, listed: &[Listed]) -> Vec<PathBuf> {
        let picked = listed.iter().filter(|row| row.pick == Some(PickGroup::Files) && self.contains(&row.key));
        distinct(picked.filter_map(|row| row.path.clone()).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::types::GitPath;
    use crate::process::ProcessId;
    use crate::sidebar::folders::ChangeSource;

    fn file(name: &str) -> Listed {
        let path = PathBuf::from(format!("/p/{name}"));
        Listed { key: TreeKey::Entry(path.clone()), pick: Some(PickGroup::Files), path: Some(path) }
    }

    /// The uncommitted change of `name`, a file of checkout `/p`.
    fn change(name: &str) -> Listed {
        let key = TreeKey::Change(PathBuf::from("/p"), ChangeSource::Uncommitted, GitPath::from_bytes(name.as_bytes()));
        Listed { key, pick: Some(PickGroup::Files), path: Some(PathBuf::from(format!("/p/{name}"))) }
    }

    fn other(name: &str) -> Listed {
        Listed { key: TreeKey::CommitsMore(PathBuf::from(format!("/p/{name}"))), pick: None, path: None }
    }

    fn process(number: u64) -> Listed {
        Listed { key: TreeKey::Process(ProcessId::for_tests(number)), pick: Some(PickGroup::Processes), path: None }
    }

    fn keys(selection: &Selection, listed: &[Listed]) -> Vec<TreeKey> {
        listed.iter().map(|row| row.key.clone()).filter(|key| selection.contains(key)).collect()
    }

    #[test]
    fn should_select_only_the_clicked_row() {
        // given
        let rows = [file("a"), file("b")];
        let mut selection = Selection::default();
        selection.toggle(rows[0].key.clone(), &rows);

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
        selection.toggle(rows[0].key.clone(), &rows);
        selection.toggle(rows[1].key.clone(), &rows);
        selection.toggle(rows[0].key.clone(), &rows);

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
        selection.toggle(rows[0].key.clone(), &rows);

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
        let dragged = selection.dragged_paths(&TreeKey::Entry(PathBuf::from("/p/b")), &rows);

        // then
        assert_eq!(dragged, [PathBuf::from("/p/b")]);
    }

    #[test]
    fn should_drag_the_whole_selection_when_the_pressed_row_is_selected() {
        // given
        let rows = [file("a"), file("b"), file("c")];
        let mut selection = Selection::default();
        selection.toggle(rows[2].key.clone(), &rows);
        selection.toggle(rows[0].key.clone(), &rows);

        // when
        let dragged = selection.dragged_paths(&TreeKey::Entry(PathBuf::from("/p/c")), &rows);

        // then
        assert_eq!(dragged, ["/p/a", "/p/c"].map(PathBuf::from));
    }

    #[test]
    fn should_list_selected_paths_in_tree_order() {
        // given
        let rows = [file("a"), file("b"), file("c")];
        let mut selection = Selection::default();
        selection.toggle(rows[2].key.clone(), &rows);
        selection.toggle(rows[0].key.clone(), &rows);

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
        selection.toggle(rows[0].key.clone(), &rows);
        selection.toggle(file("collapsed-away").key, &rows);

        // when
        let paths = selection.listed_paths(&rows);

        // then
        assert_eq!(paths, [PathBuf::from("/p/a")]);
    }

    #[test]
    fn should_start_new_selection_when_toggling_other_group() {
        // given
        let rows = [file("a"), process(1)];
        let mut selection = Selection::default();
        selection.select_only(rows[0].key.clone());

        // when
        selection.toggle(rows[1].key.clone(), &rows);

        // then
        assert_eq!(keys(&selection, &rows), vec![rows[1].key.clone()]);
    }

    #[test]
    fn should_range_only_over_rows_of_anchor_group() {
        // given
        let rows = [file("a"), process(1), file("b")];
        let mut selection = Selection::default();
        selection.select_only(rows[0].key.clone());

        // when
        let extended = selection.extend(&rows[2].key, &rows);

        // then
        assert!(extended);
        assert_eq!(keys(&selection, &rows), vec![rows[0].key.clone(), rows[2].key.clone()]);
    }

    #[test]
    fn should_refuse_range_to_row_of_other_group() {
        // given
        let rows = [file("a"), process(1)];
        let mut selection = Selection::default();
        selection.select_only(rows[0].key.clone());

        // when
        let extended = selection.extend(&rows[1].key, &rows);

        // then
        assert!(!extended);
        assert_eq!(keys(&selection, &rows), vec![rows[0].key.clone()]);
    }

    #[test]
    fn should_clear_picks_and_keep_anchor() {
        // given
        let rows = [file("a")];
        let mut selection = Selection::default();
        selection.select_only(rows[0].key.clone());

        // when
        selection.clear();

        // then
        assert_eq!((keys(&selection, &rows), selection.anchor()), (vec![], Some(&rows[0].key)));
    }

    #[test]
    fn should_drag_picked_processes_when_pressed_one_is_picked() {
        // given
        let rows = [process(1), file("a"), process(2)];
        let mut selection = Selection::default();
        selection.toggle(rows[0].key.clone(), &rows);
        selection.toggle(rows[2].key.clone(), &rows);

        // when
        let dragged = selection.dragged_keys(&rows[2].key, &rows);

        // then
        assert_eq!(dragged, vec![rows[0].key.clone(), rows[2].key.clone()]);
    }

    #[test]
    fn should_drag_only_pressed_process_when_it_is_not_picked() {
        // given
        let rows = [process(1), process(2)];
        let mut selection = Selection::default();
        selection.select_only(rows[0].key.clone());

        // when
        let dragged = selection.dragged_keys(&rows[1].key, &rows);

        // then
        assert_eq!(dragged, vec![rows[1].key.clone()]);
    }

    #[test]
    fn should_leave_out_picks_inside_a_picked_folder() {
        // given
        let rows = [file("src"), file("src/a.rs"), file("b.rs")];
        let mut selection = Selection::default();
        for row in &rows {
            selection.toggle(row.key.clone(), &rows);
        }

        // when
        let dragged = selection.dragged_paths(&TreeKey::Entry(PathBuf::from("/p/src/a.rs")), &rows);

        // then
        assert_eq!(dragged, ["/p/src", "/p/b.rs"].map(PathBuf::from));
    }

    #[test]
    fn should_pick_given_keys_at_once() {
        // given
        let rows = [file("a"), file("b"), file("c")];
        let mut selection = Selection::default();
        selection.select_only(rows[2].key.clone());

        // when
        selection.pick_all(vec![rows[0].key.clone(), rows[1].key.clone()]);

        // then
        assert_eq!((keys(&selection, &rows), selection.anchor()), (vec![rows[0].key.clone(), rows[1].key.clone()], Some(&rows[0].key)));
    }

    #[test]
    fn should_pick_a_change_with_files() {
        // given
        let rows = [file("a"), change("b")];
        let mut selection = Selection::default();

        // when
        selection.toggle(rows[0].key.clone(), &rows);
        selection.toggle(rows[1].key.clone(), &rows);

        // then
        assert_eq!(keys(&selection, &rows), [rows[0].key.clone(), rows[1].key.clone()]);
    }

    #[test]
    fn should_count_a_path_picked_twice_once() {
        // given
        let rows = [change("a"), file("a")];
        let mut selection = Selection::default();
        for row in &rows {
            selection.toggle(row.key.clone(), &rows);
        }

        // when
        let (dragged, listed) = (selection.dragged_paths(&rows[0].key, &rows), selection.listed_paths(&rows));

        // then
        assert_eq!((dragged, listed), (vec![PathBuf::from("/p/a")], vec![PathBuf::from("/p/a")]));
    }

    #[test]
    fn should_drag_a_changed_file_inside_a_picked_folder_with_its_folder() {
        // given
        let rows = [change("d/x"), file("d")];
        let mut selection = Selection::default();
        for row in &rows {
            selection.toggle(row.key.clone(), &rows);
        }

        // when
        let dragged = selection.dragged_paths(&rows[0].key, &rows);

        // then
        assert_eq!(dragged, [PathBuf::from("/p/d")]);
    }
}
