//! Rename and Delete in the tree: what they apply to, and the flow around the file operation: every tab's edits
//! saved first (one whose edits cannot be saved stops it), the operation off the UI thread, then the tabs of what
//! went closed or moved to the new path.

use std::os::unix::ffi::OsStrExt as _;
use std::path::{Path, PathBuf};

use gpui::{Context, PromptLevel, Window};

use super::Workspace;
use super::transfers::NoteKind;
use crate::editor::Tab;
use crate::file_ops;
use crate::menu::Subject;
use crate::menu::resolve::Applies;
use crate::script::delete_mode::DeleteMode;
use crate::sidebar::rows::TreeKey;
use crate::sidebar::selection::without_nested;
use crate::tabs::TabId;

/// The tabs of `tabs` (with the file each shows) whose file is one of `paths` or inside one.
pub fn affected(tabs: impl IntoIterator<Item = (TabId, PathBuf)>, paths: &[PathBuf]) -> Vec<TabId> {
    tabs.into_iter().filter(|(_, shown)| paths.iter().any(|path| shown.starts_with(path))).map(|(id, _)| id).collect()
}

/// What Delete takes of `subject`: its paths but the protected ones (checkout roots, folders holding one, `.git`),
/// without those inside another; and why each protected one stays.
pub fn deletable(subject: &Subject, checkouts: &[PathBuf]) -> (Vec<PathBuf>, Vec<String>) {
    let (refused, kept): (Vec<_>, Vec<_>) = subject.paths.iter().map(|path| (path.path.clone(), file_ops::protected(&path.path, checkouts))).partition(|(_, why)| why.is_some());
    let kept = without_nested(kept.into_iter().map(|(path, _)| path).collect());
    (kept, refused.into_iter().filter_map(|(_, why)| why).collect())
}

/// Delete in a menu: shown while anything of `subject` can be deleted.
pub fn delete_applies(subject: &Subject, checkouts: &[PathBuf]) -> Applies {
    match deletable(subject, checkouts).0.is_empty() {
        true => Applies::Hidden,
        false => Applies::Shown { enabled: true },
    }
}

/// Rename in a menu: shown for one path that is not protected.
pub fn rename_applies(subject: &Subject, checkouts: &[PathBuf]) -> Applies {
    match &subject.paths[..] {
        [only] if file_ops::protected(&only.path, checkouts).is_none() => Applies::Shown { enabled: true },
        _ => Applies::Hidden,
    }
}

/// Where `path` is once `from` was renamed to `to`: `path` itself, or inside it.
pub fn moved_path(path: &Path, (from, to): (&Path, &Path)) -> PathBuf {
    path.strip_prefix(from).map_or_else(|_| path.to_path_buf(), |inside| to.join(inside))
}

/// The question a permanent delete asks: `Delete main.rs permanently?`, `Delete 3 items permanently?`.
fn permanent_question(paths: &[PathBuf]) -> String {
    match paths {
        [only] => format!("Delete {} permanently?", only.file_name().map_or_else(|| only.display().to_string(), |name| name.to_string_lossy().into_owned())),
        many => format!("Delete {} items permanently?", many.len()),
    }
}

impl Workspace {
    /// The file each file and diff tab shows, with the tab.
    pub(super) fn tab_files(&self) -> Vec<(TabId, PathBuf)> {
        let file = |tab: &Tab| match tab {
            Tab::Document(document) => Some(document.path().to_path_buf()),
            Tab::Diff(diff) => Some(diff.checkout.path.join(std::ffi::OsStr::from_bytes(diff.request.file.path.as_bytes()))),
            _ => None,
        };
        self.tabs.iter().filter_map(|(id, tab)| file(tab).map(|path| (id, path))).collect()
    }

    /// Delete on the row of `pressed` (and the selection with it): the protected paths stay, named under the tree;
    /// a permanent delete asks first.
    pub(super) fn delete_files(&mut self, pressed: &TreeKey, window: &mut Window, cx: &mut Context<Self>) {
        let (paths, refused) = deletable(&self.tree_subject(pressed), &self.all_checkouts());
        if let Some(why) = refused.first() {
            self.note(format!("Delete refused: {why}"), NoteKind::Staying, cx);
        }
        if paths.is_empty() {
            return;
        }
        let mode = cx.try_global::<DeleteMode>().copied().unwrap_or_default();
        if mode == DeleteMode::Trash {
            return self.delete_when_saved(paths, mode, window, cx);
        }
        let answer = window.prompt(PromptLevel::Warning, &permanent_question(&paths), None, &["Delete", "Cancel"], cx);
        cx.spawn_in(window, async move |this, cx| {
            if answer.await == Ok(0) {
                this.update_in(cx, |workspace, window, cx| workspace.delete_when_saved(paths, mode, window, cx)).ok();
            }
        })
        .detach();
    }

    /// Deletes `paths` once every tab's edits are on disk; a tab of one of them whose edits cannot be saved stops it.
    fn delete_when_saved(&mut self, paths: Vec<PathBuf>, mode: DeleteMode, window: &mut Window, cx: &mut Context<Self>) {
        self.flush_edits(cx);
        cx.spawn_in(window, async move |this, cx| {
            super::saves::saves_landed(&this, cx).await;
            let Ok(Some(paths)) = this.update_in(cx, |workspace, window, cx| workspace.held_by_edits(&paths, "Delete", window, cx).then_some(paths)) else {
                return;
            };
            let failures = cx.background_executor().spawn(async move { deleted(&paths, mode) }).await;
            this.update_in(cx, |workspace, window, cx| workspace.finish_delete(failures, window, cx)).ok();
        })
        .detach();
    }

    /// False, showing the tab and saying why under the tree, when a tab of one of `paths` holds edits that could not
    /// be saved; true when `action` can go on.
    pub(super) fn held_by_edits(&mut self, paths: &[PathBuf], action: &str, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let unsaved = affected(self.tab_files(), paths).into_iter().find(|id| self.is_unsettled(*id, cx));
        let Some(tab) = unsaved else {
            return true;
        };
        let name = self.tab_files().into_iter().find(|(id, _)| *id == tab).and_then(|(_, path)| path.file_name().map(|name| name.to_string_lossy().into_owned())).unwrap_or_default();
        self.activate_tab(tab, window, cx);
        self.note(format!("{action} stopped: {name} has edits that could not be saved"), NoteKind::Staying, cx);
        false
    }

    /// After a delete: the tabs of what went close, and the first failure is said under the tree.
    fn finish_delete(&mut self, outcome: Deleted, window: &mut Window, cx: &mut Context<Self>) {
        let gone = affected(self.tab_files(), &outcome.gone);
        self.close_tabs(gone, window, cx);
        self.sync_watches();
        if let Some(failure) = outcome.failures.first() {
            self.note(failure.clone(), NoteKind::Staying, cx);
        }
        cx.notify();
    }
}

/// What a delete did: the paths that went, and why the others stayed.
struct Deleted {
    gone: Vec<PathBuf>,
    failures: Vec<String>,
}

fn deleted(paths: &[PathBuf], mode: DeleteMode) -> Deleted {
    let (gone, failures): (Vec<_>, Vec<_>) = paths.iter().map(|path| (path, file_ops::delete(path, mode))).partition(|(_, result)| result.is_ok());
    Deleted { gone: gone.into_iter().map(|(path, _)| path.clone()).collect(), failures: failures.into_iter().filter_map(|(_, result)| result.err()).collect() }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::menu::PathKind;
    use crate::workspace::menu::subject_of;

    fn checkouts() -> Vec<PathBuf> {
        vec![PathBuf::from("/r"), PathBuf::from("/r/.worktree/b")]
    }

    #[test]
    fn should_find_tabs_inside_a_renamed_folder() {
        // given
        let tabs = [(TabId::for_tests(1), PathBuf::from("/r/src/a.rs")), (TabId::for_tests(2), PathBuf::from("/r/srcs/b.rs")), (TabId::for_tests(3), PathBuf::from("/r/src"))];

        // when
        let found = affected(tabs, &[PathBuf::from("/r/src")]);

        // then
        assert_eq!(found, [TabId::for_tests(1), TabId::for_tests(3)]);
    }

    #[test]
    fn should_leave_roots_out_of_delete() {
        // given
        let subject = subject_of(vec![(PathBuf::from("/r/a.rs"), PathKind::File), (PathBuf::from("/r/.worktree/b"), PathKind::Folder)], &checkouts());

        // when
        let (kept, refused) = deletable(&subject, &checkouts());

        // then
        assert_eq!((kept, refused), (vec![PathBuf::from("/r/a.rs")], vec!["/r/.worktree/b is a checkout's root".to_owned()]));
    }

    #[test]
    fn should_delete_a_folder_without_what_is_inside_it() {
        // given
        let subject = subject_of(vec![(PathBuf::from("/r/d"), PathKind::Folder), (PathBuf::from("/r/d/x"), PathKind::File)], &checkouts());

        // when
        let (kept, _) = deletable(&subject, &checkouts());

        // then
        assert_eq!(kept, [PathBuf::from("/r/d")]);
    }

    #[test]
    fn should_hide_delete_without_deletable_paths() {
        // given
        let subject = subject_of(vec![(PathBuf::from("/r"), PathKind::Folder)], &checkouts());

        // when
        let applies = delete_applies(&subject, &checkouts());

        // then
        assert_eq!(applies, Applies::Hidden);
    }

    #[test]
    fn should_offer_rename_for_one_path() {
        // given
        let one = subject_of(vec![(PathBuf::from("/r/a.rs"), PathKind::File)], &checkouts());
        let two = subject_of(vec![(PathBuf::from("/r/a.rs"), PathKind::File), (PathBuf::from("/r/b.rs"), PathKind::File)], &checkouts());

        // when
        let applies = [rename_applies(&one, &checkouts()), rename_applies(&two, &checkouts())];

        // then
        assert_eq!(applies, [Applies::Shown { enabled: true }, Applies::Hidden]);
    }

    #[test]
    fn should_hide_rename_for_a_root() {
        // given
        let subject = subject_of(vec![(PathBuf::from("/r/.worktree/b"), PathKind::Folder)], &checkouts());

        // when
        let applies = rename_applies(&subject, &checkouts());

        // then
        assert_eq!(applies, Applies::Hidden);
    }

    #[test]
    fn should_place_the_renamed_path_inside_a_renamed_folder() {
        // when
        let moved = [moved_path(Path::new("/r/src/a.rs"), (Path::new("/r/src"), Path::new("/r/lib"))), moved_path(Path::new("/r/src"), (Path::new("/r/src"), Path::new("/r/lib")))];

        // then
        assert_eq!(moved, [PathBuf::from("/r/lib/a.rs"), PathBuf::from("/r/lib")]);
    }
}
