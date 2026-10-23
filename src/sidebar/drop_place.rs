//! Where something dropped on a tree row goes: files and folders into a folder, processes into a checkout.

use std::path::PathBuf;

use super::folders::FolderKind;
use super::rows::{TreeItem, TreeKey};
use crate::file_tree::EntryKind;
use crate::project::{CheckoutRef, ProjectId};

/// A folder of a checkout that files are dropped into.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DropPlace {
    pub checkout: CheckoutRef,
    /// absolute
    pub folder: PathBuf,
}

/// What a drop on the row under the pointer would do, as the drag's chip says it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DropHint {
    pub row: TreeKey,
    pub text: String,
    /// a drop there does something: the row is drawn as the drop row
    pub allowed: bool,
}

/// Where files dropped on `item` go: a folder row's folder, a file's folder, the root of a worktree (its row or its
/// Files folder) or of a project's main checkout (`main_of`); `None` for any other row.
pub fn files_place(item: &TreeItem, main_of: impl Fn(&ProjectId) -> Option<CheckoutRef>) -> Option<DropPlace> {
    let root = |checkout: &CheckoutRef| DropPlace { checkout: checkout.clone(), folder: checkout.path.clone() };
    match item {
        TreeItem::Entry(row) => match row.entry.kind {
            EntryKind::Folder => Some(DropPlace { checkout: row.checkout.clone(), folder: row.entry.path.clone() }),
            EntryKind::File => Some(DropPlace { checkout: row.checkout.clone(), folder: row.entry.path.parent()?.to_path_buf() }),
            _ => None,
        },
        TreeItem::Checkout(row) => Some(root(&row.checkout)),
        TreeItem::Folder(row) if row.kind == FolderKind::Files => Some(root(&row.checkout)),
        TreeItem::Project(row) => main_of(&row.id).map(|main| root(&main)),
        _ => None,
    }
}

/// The checkout processes dropped on `item` go to: a worktree's row or its Processes folder, or a project's main
/// checkout (`main_of`); `None` for any other row.
pub fn processes_place(item: &TreeItem, main_of: impl Fn(&ProjectId) -> Option<CheckoutRef>) -> Option<CheckoutRef> {
    match item {
        TreeItem::Checkout(row) => Some(row.checkout.clone()),
        TreeItem::Folder(row) if row.kind == FolderKind::Processes => Some(row.checkout.clone()),
        TreeItem::Project(row) => main_of(&row.id),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::file_tree::{Entry, EntryKind, RunState};
    use crate::project::ProjectId;
    use crate::sidebar::folders::{FolderKind, FolderRow};
    use crate::sidebar::rows::{CheckoutRow, EntryRow, ProjectRow};
    use crate::sidebar::states::EntryState;

    fn checkout(path: &str) -> CheckoutRef {
        CheckoutRef { project: ProjectId::new("p"), path: PathBuf::from(path) }
    }

    fn main_of(_project: &ProjectId) -> Option<CheckoutRef> {
        Some(checkout("/p"))
    }

    fn entry(path: &str, kind: EntryKind) -> TreeItem {
        let entry = Entry { name: path.rsplit('/').next().unwrap().to_owned(), path: PathBuf::from(path), kind, run: RunState::None };
        TreeItem::Entry(EntryRow { checkout: checkout("/p/wt"), entry, expanded: false, warning: None, run_file: false, state: EntryState::Plain, mark: None })
    }

    fn folder(kind: FolderKind) -> TreeItem {
        TreeItem::Folder(FolderRow { checkout: checkout("/p/wt"), kind, count: None, expanded: true, empty: false, nothing: None })
    }

    fn project() -> TreeItem {
        TreeItem::Project(ProjectRow { id: ProjectId::new("p"), name: "p".to_owned(), initials: "P".to_owned(), expanded: true, tag: None, running: 0, error: None })
    }

    fn place(checkout_path: &str, folder: &str) -> Option<DropPlace> {
        Some(DropPlace { checkout: checkout(checkout_path), folder: PathBuf::from(folder) })
    }

    #[test]
    fn should_drop_files_into_folder_row() {
        // when
        let placed = files_place(&entry("/p/wt/src", EntryKind::Folder), main_of);

        // then
        assert_eq!(placed, place("/p/wt", "/p/wt/src"));
    }

    #[test]
    fn should_drop_files_on_file_into_its_folder() {
        // when
        let placed = files_place(&entry("/p/wt/src/main.rs", EntryKind::File), main_of);

        // then
        assert_eq!(placed, place("/p/wt", "/p/wt/src"));
    }

    #[test]
    fn should_drop_files_on_worktree_and_files_folder_into_its_root() {
        // given
        let worktree = TreeItem::Checkout(CheckoutRow { checkout: checkout("/p/wt"), label: "wt".to_owned(), expanded: true, is_main: false });

        // when
        let placed = [files_place(&worktree, main_of), files_place(&folder(FolderKind::Files), main_of)];

        // then
        assert_eq!(placed, [place("/p/wt", "/p/wt"), place("/p/wt", "/p/wt")]);
    }

    #[test]
    fn should_drop_files_on_project_into_main_checkout_root() {
        // when
        let placed = files_place(&project(), main_of);

        // then
        assert_eq!(placed, place("/p", "/p"));
    }

    #[test]
    fn should_refuse_files_on_processes_and_git_folders() {
        // when
        let placed = [FolderKind::Processes, FolderKind::Uncommitted, FolderKind::Commits].map(|kind| files_place(&folder(kind), main_of));

        // then
        assert_eq!(placed, [None, None, None]);
    }

    #[test]
    fn should_drop_processes_on_processes_folder_into_its_checkout() {
        // when
        let placed = processes_place(&folder(FolderKind::Processes), main_of);

        // then
        assert_eq!(placed, Some(checkout("/p/wt")));
    }

    #[test]
    fn should_drop_processes_on_project_into_main_checkout() {
        // when
        let placed = processes_place(&project(), main_of);

        // then
        assert_eq!(placed, Some(checkout("/p")));
    }

    #[test]
    fn should_refuse_processes_on_entry_and_files_folder() {
        // when
        let placed = [processes_place(&entry("/p/wt/src", EntryKind::Folder), main_of), processes_place(&folder(FolderKind::Files), main_of)];

        // then
        assert_eq!(placed, [None, None]);
    }
}
