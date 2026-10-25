//! A tree row's project or module favicon, and a worktree row's rail: worked out once per row from the workspace,
//! drawn by the row.

use std::path::PathBuf;

use super::rows::{TreeItem, TreeRow};
use crate::colors::checkout_colour;
use crate::theme::Theme;
use crate::workspace::Workspace;

/// A folder row's icon: its module's favicon, else the one it has without (folder, run badge, excluded).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FolderIcon {
    Favicon(PathBuf),
    Embedded(&'static str),
}

pub fn folder_icon(embedded: &'static str, module: Option<PathBuf>) -> FolderIcon {
    match module {
        Some(path) => FolderIcon::Favicon(path),
        None => FolderIcon::Embedded(embedded),
    }
}

/// What a row shows of its project or worktree: a project's favicon (a project row), a module's (a folder row),
/// a worktree's rail colour (a worktree row but the main checkout's), the main checkout's star.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RowMark {
    pub favicon: Option<PathBuf>,
    pub rail: Option<u32>,
    pub star: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckoutKind {
    Main,
    Linked,
}

/// A worktree row's mark: the main checkout's star, another worktree's rail in the colour `colour` gives.
pub fn checkout_mark(kind: CheckoutKind, colour: impl FnOnce() -> Option<u32>) -> RowMark {
    match kind {
        CheckoutKind::Main => RowMark { star: true, ..RowMark::default() },
        CheckoutKind::Linked => RowMark { rail: colour(), ..RowMark::default() },
    }
}

/// Row `row`'s mark, from the workspace's favicons and checkouts.
pub fn row_mark(row: &TreeRow, workspace: &Workspace, theme: Theme) -> RowMark {
    match &row.item {
        TreeItem::Project(project) => RowMark { favicon: workspace.project_icon(&project.id), ..RowMark::default() },
        TreeItem::Entry(entry) if entry.entry.expands() => RowMark { favicon: workspace.module_icon(&entry.checkout.path, &entry.entry.path), ..RowMark::default() },
        TreeItem::ChangeFolder(folder) => RowMark { favicon: workspace.module_icon(&folder.checkout.path, folder.path.trim_end_matches('/').as_ref()), ..RowMark::default() },
        TreeItem::Checkout(checkout) => {
            let kind = if checkout.is_main { CheckoutKind::Main } else { CheckoutKind::Linked };
            checkout_mark(kind, || workspace.checkout(&checkout.checkout).map(|found| checkout_colour(found, theme)))
        }
        _ => RowMark::default(),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::icons;

    #[test]
    fn should_star_the_main_checkout() {
        // when
        let mark = checkout_mark(CheckoutKind::Main, || Some(0x123456));

        // then
        assert_eq!(mark, RowMark { favicon: None, rail: None, star: true });
    }

    #[test]
    fn should_rail_another_worktree_without_a_star() {
        // when
        let mark = checkout_mark(CheckoutKind::Linked, || Some(0x123456));

        // then
        assert_eq!(mark, RowMark { favicon: None, rail: Some(0x123456), star: false });
    }

    #[test]
    fn should_show_the_module_favicon_over_the_run_badge() {
        // given
        let module = Some(PathBuf::from("/r/svc/.favicon.svg"));

        // when
        let icon = folder_icon(icons::FOLDER_RUN, module);

        // then
        assert_eq!(icon, FolderIcon::Favicon(PathBuf::from("/r/svc/.favicon.svg")));
    }

    #[test]
    fn should_keep_the_run_badge_without_a_favicon() {
        // given
        let module = None;

        // when
        let icon = folder_icon(icons::FOLDER_RUN, module);

        // then
        assert_eq!(icon, FolderIcon::Embedded(icons::FOLDER_RUN));
    }
}
