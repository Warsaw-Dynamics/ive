//! Rename and Delete in the tree: what they apply to, and the flow around the file operation: every tab's edits
//! saved first (one whose edits cannot be saved stops it), the operation off the UI thread, then the tabs of what
//! went closed or moved to the new path.

use std::path::{Path, PathBuf};

use gpui::{AppContext as _, Context, Focusable as _, PromptLevel, Window};

use super::Workspace;
use super::transfers::NoteKind;
use crate::document::Document;
use crate::editor::Tab;
use crate::file_ops;
use crate::file_tab::FileTab;
use crate::git::types::GitPath;
use crate::menu::Subject;
use crate::menu::resolve::Applies;
use crate::saved_tabs::SavedTab;
use crate::script::delete_mode::DeleteMode;
use crate::sidebar::rows::TreeKey;
use crate::sidebar::selection::without_nested;
use crate::sidebar::view::{RenameStage, Renaming};
use crate::tabs::TabId;
use crate::text_input::{TextInput, TextInputEvent};

/// What the line under the tree says before the paths Delete leaves alone.
const REFUSED: &str = "Delete refused";
/// The rename field's click target, for MCP.
const RENAME_TARGET: &str = "tree-rename";

/// The tabs of `tabs` (with the file each shows) whose file is one of `paths` or inside one.
pub fn affected(tabs: impl IntoIterator<Item = (TabId, PathBuf)>, paths: &[PathBuf]) -> Vec<TabId> {
    tabs.into_iter().filter(|(_, shown)| paths.iter().any(|path| shown.starts_with(path))).map(|(id, _)| id).collect()
}

/// `subject` without the paths that are not in the work tree now: a commit's file deleted since, say.
pub fn in_work_tree(subject: &Subject) -> Subject {
    let paths = subject.paths.iter().filter(|path| std::fs::symlink_metadata(&path.path).is_ok()).cloned().collect();
    Subject { paths, home: subject.home.clone() }
}

/// What Delete takes of `subject`: its paths but the protected ones (checkout roots, folders holding one, `.git`),
/// without those inside another; and why each protected one stays.
pub fn deletable(subject: &Subject, checkouts: &[PathBuf]) -> (Vec<PathBuf>, Vec<String>) {
    let checked = subject.paths.iter().map(|path| (path.path.clone(), file_ops::protected(&path.path, checkouts)));
    let (refused, kept): (Vec<_>, Vec<_>) = checked.partition(|(_, why)| why.is_some());
    let kept = without_nested(kept.into_iter().map(|(path, _)| path).collect());
    // a protected path inside a folder that goes (a nested repository's `.git`) goes with it, so it is not refused
    let inside_kept = |path: &PathBuf| kept.iter().any(|folder| path.starts_with(folder));
    let refused = refused.into_iter().filter(|(path, _)| !inside_kept(path)).filter_map(|(_, why)| why).collect();
    (kept, refused)
}

/// The line under the tree naming every one of `items` after `what`: `Delete refused: a; b`; `None` without items.
pub fn listed_note(what: &str, items: &[String]) -> Option<String> {
    match items.is_empty() {
        true => None,
        false => Some(format!("{what}: {}", items.join("; "))),
    }
}

/// The row of `key` once its file was renamed to `to`: the same kind of row, at the new path.
pub fn renamed_key(key: &TreeKey, to: &Path) -> Option<TreeKey> {
    match key {
        TreeKey::Entry(_) => Some(TreeKey::Entry(to.to_path_buf())),
        TreeKey::Change(checkout, source, _) => {
            let inside = to.strip_prefix(checkout).ok()?;
            Some(TreeKey::Change(checkout.clone(), source.clone(), GitPath::of_path(inside)))
        }
        _ => None,
    }
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
    match path.strip_prefix(from) {
        // `to.join("")` would end in `/`
        Ok(inside) if inside.as_os_str().is_empty() => to.to_path_buf(),
        Ok(inside) => to.join(inside),
        Err(_) => path.to_path_buf(),
    }
}

/// The file restored tab `saved` (not shown since IVE started) shows: a file's, a diff's.
pub fn saved_file(saved: &SavedTab) -> Option<PathBuf> {
    match saved {
        SavedTab::File { path, .. } => Some(path.clone()),
        SavedTab::Diff { checkout, request, .. } => Some(checkout.join(GitPath::from_bytes(&request.path).to_path())),
        _ => None,
    }
}

/// Restored file tab `saved` once `from` was renamed to `to`; `None` for a restored diff, which closes.
pub fn moved_saved(saved: &SavedTab, renamed: (&Path, &Path)) -> Option<SavedTab> {
    match saved {
        SavedTab::File { path, panel } => Some(SavedTab::File { path: moved_path(path, renamed), panel: panel.clone() }),
        _ => None,
    }
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
            Tab::Diff(diff) => Some(diff.checkout.path.join(diff.request.file.path.to_path())),
            Tab::Unloaded(saved) => saved_file(saved),
            _ => None,
        };
        self.tabs.iter().filter_map(|(id, tab)| file(tab).map(|path| (id, path))).collect()
    }

    /// Delete on the row of `pressed` (and the selection with it): the protected paths stay, named under the tree;
    /// a permanent delete asks first.
    pub(super) fn delete_files(&mut self, pressed: &TreeKey, window: &mut Window, cx: &mut Context<Self>) {
        let (paths, refused) = deletable(&in_work_tree(&self.tree_subject(pressed)), &self.all_checkouts());
        if let Some(note) = listed_note(REFUSED, &refused) {
            self.note(note, NoteKind::Staying, cx);
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
            let settled = this.update_in(cx, |workspace, window, cx| workspace.edits_settled(&paths, "Delete", window, cx)).ok();
            if settled != Some(Settled::Saved) {
                return;
            }
            let failures = cx.background_executor().spawn(async move { deleted(&paths, mode) }).await;
            this.update_in(cx, |workspace, window, cx| workspace.finish_delete(failures, window, cx)).ok();
        })
        .detach();
    }

    /// Whether the tabs of `paths` have their edits on disk, so `action` can go on; when one holds edits that could
    /// not be saved, that tab is shown and the line under the tree says why.
    pub(super) fn edits_settled(&mut self, paths: &[PathBuf], action: &str, window: &mut Window, cx: &mut Context<Self>) -> Settled {
        let unsaved = affected(self.tab_files(), paths).into_iter().find(|id| self.is_unsettled(*id, cx));
        let Some(tab) = unsaved else {
            return Settled::Saved;
        };
        let path = self.tab_files().into_iter().find(|(id, _)| *id == tab).map(|(_, path)| path).unwrap_or_default();
        let name = path.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default();
        self.activate_tab(tab, window, cx);
        self.note(format!("{action} stopped: {name} has edits that could not be saved"), NoteKind::Staying, cx);
        Settled::Held
    }

    /// After a delete: the tabs of what went close, and the first failure is said under the tree.
    fn finish_delete(&mut self, outcome: Deleted, window: &mut Window, cx: &mut Context<Self>) {
        // a tab edited while the delete ran stays with its edits (its save writes the file again), never closed on them
        let (edited, gone): (Vec<TabId>, Vec<TabId>) = affected(self.tab_files(), &outcome.gone).into_iter().partition(|id| self.is_unsettled(*id, cx));
        self.close_tabs(gone, window, cx);
        self.sync_watches();
        let edited_note = (!edited.is_empty()).then(|| format!("{} edited during the delete: kept open with the edits", plan_count(edited.len())));
        let said: Vec<String> = outcome.failures.into_iter().chain(edited_note).collect();
        let after = (!said.is_empty()).then(|| said.join("; "));
        // the refusals said when the delete started stay in front of what went wrong since
        let refused = self.tree.note.as_ref().map(|note| note.text.clone()).filter(|text| text.starts_with(REFUSED));
        if let Some(after) = after {
            self.note(refused.map_or(after.clone(), |refused| format!("{refused} · {after}")), NoteKind::Staying, cx);
        }
        cx.notify();
    }
}

impl Workspace {
    /// Rename on the row of `pressed`: its name becomes a field, the part before the extension selected. Nothing for
    /// a row Rename does not apply to.
    pub(super) fn start_rename(&mut self, pressed: &TreeKey, window: &mut Window, cx: &mut Context<Self>) {
        let subject = in_work_tree(&self.tree_subject(pressed));
        let ([only], Applies::Shown { .. }) = (&subject.paths[..], rename_applies(&subject, &self.all_checkouts())) else {
            return;
        };
        let name = only.path.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default();
        let selected = file_ops::stem_end(&name, only.kind);
        let input = cx.new(TextInput::new);
        input.update(cx, |input, cx| {
            input.set_text(name, cx);
            input.select_range(0..selected);
            input.set_target(RENAME_TARGET);
        });
        let focus = input.focus_handle(cx);
        window.focus(&focus, cx);
        let subscriptions = vec![
            cx.subscribe_in(&input, window, |workspace, _input, event: &TextInputEvent, window, cx| match event {
                TextInputEvent::Changed(_) => workspace.dismiss_note(cx),
                TextInputEvent::Submitted => workspace.submit_rename(window, cx),
            }),
            // a click elsewhere cancels, as Escape does
            cx.on_blur(&focus, window, |workspace, _window, cx| workspace.cancel_rename(cx)),
        ];
        self.tree.renaming = Some(Renaming { key: pressed.clone(), path: only.path.clone(), input, stage: RenameStage::Typing, _subscriptions: subscriptions });
        cx.notify();
    }

    /// Escape or a click elsewhere: the field goes, unless a rename it asked for is under way.
    pub(crate) fn cancel_rename(&mut self, cx: &mut Context<Self>) {
        if self.tree.renaming.as_ref().is_some_and(|renaming| renaming.stage == RenameStage::Typing) {
            self.tree.renaming = None;
            cx.notify();
        }
    }

    /// Escape in the field: it goes, and the tree has the keyboard again.
    pub(crate) fn escape_rename(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.tree.renaming.as_ref().is_some_and(|renaming| renaming.stage == RenameStage::Typing) {
            self.give_keys_to_tree(window, cx);
        }
        self.cancel_rename(cx);
    }

    /// The field of a rename on the row of `key` goes; a field opened since on another row stays. The tree gets the
    /// keyboard back while the field had it.
    fn end_rename(&mut self, key: &TreeKey, window: &mut Window, cx: &mut Context<Self>) {
        if self.tree.renaming.as_ref().is_some_and(|renaming| renaming.key == *key) {
            self.give_keys_to_tree(window, cx);
            self.tree.renaming = None;
            cx.notify();
        }
    }

    /// The sidebar takes the keyboard when the rename field has it, so F2 and Delete work again without a click.
    fn give_keys_to_tree(&self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(renaming) = &self.tree.renaming else {
            return;
        };
        if renaming.input.focus_handle(cx).is_focused(window) {
            let sidebar = cx.global::<crate::theme::SidebarFocus>().0.clone();
            window.focus(&sidebar, cx);
        }
    }

    /// Enter in the field: a refused name says why under the tree, the field staying open; otherwise the tabs' edits
    /// are saved, then the file or folder is renamed off the UI thread.
    fn submit_rename(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(renaming) = self.tree.renaming.as_mut().filter(|renaming| renaming.stage == RenameStage::Typing) else {
            return;
        };
        let (from, key) = (renaming.path.clone(), renaming.key.clone());
        let to = match file_ops::checked_name(&from, renaming.input.read(cx).text()) {
            Ok(to) => to,
            Err(refused) => return self.note(refused, NoteKind::Staying, cx),
        };
        // a second Enter, Escape or a click elsewhere waits for this rename's answer
        renaming.stage = RenameStage::Submitted;
        self.flush_edits(cx);
        cx.spawn_in(window, async move |this, cx| {
            super::saves::saves_landed(&this, cx).await;
            let settled = this.update_in(cx, |workspace, window, cx| workspace.edits_settled(std::slice::from_ref(&from), "Rename", window, cx)).ok();
            if settled == Some(Settled::Held) {
                this.update_in(cx, |workspace, window, cx| workspace.end_rename(&key, window, cx)).ok();
            }
            if settled == Some(Settled::Saved) {
                let (source, target) = (from.clone(), to.clone());
                let renamed = cx.background_executor().spawn(async move { file_ops::rename(&source, &target) }).await;
                this.update_in(cx, |workspace, window, cx| workspace.finish_rename(Renamed { key, from, to, result: renamed }, window, cx)).ok();
            }
        })
        .detach();
    }

    /// After a rename: the file tabs of what moved reopen at the new path in their place, its diff tabs close; a
    /// failure is said under the tree, the field staying open to try another name.
    fn finish_rename(&mut self, renamed: Renamed, window: &mut Window, cx: &mut Context<Self>) {
        if let Err(failure) = renamed.result {
            if let Some(renaming) = self.tree.renaming.as_mut().filter(|renaming| renaming.key == renamed.key) {
                renaming.stage = RenameStage::Typing;
            }
            return self.note(failure, NoteKind::Staying, cx);
        }
        self.end_rename(&renamed.key, window, cx);
        self.dismiss_note(cx);
        let moved = affected(self.tab_files(), std::slice::from_ref(&renamed.from));
        // a tab edited while the rename ran keeps its edits (its save writes the old path again), never replaced on them
        let moved: Vec<TabId> = moved.into_iter().filter(|id| !self.is_unsettled(*id, cx)).collect();
        let is_file_tab = |id: &TabId| matches!(self.tabs.get(*id), Some(Tab::Document(_) | Tab::Unloaded(SavedTab::File { .. })));
        let (documents, diffs): (Vec<TabId>, Vec<TabId>) = moved.into_iter().partition(is_file_tab);
        for id in documents {
            self.reopen_moved(id, (&renamed.from, &renamed.to), window, cx);
        }
        self.close_tabs(diffs, window, cx);
        match renamed_key(&renamed.key, &renamed.to) {
            Some(key) => self.tree.selection.select_only(key),
            None => self.tree.selection.clear(),
        }
        self.sync_watches();
        self.tree_changed(cx);
        cx.notify();
    }

    /// File tab `id`, its file moved by renaming `from` to `to`, replaced in its place by a tab of the file there.
    fn reopen_moved(&mut self, id: TabId, (from, to): (&Path, &Path), window: &mut Window, cx: &mut Context<Self>) {
        let document = match self.tabs.get(id) {
            Some(Tab::Document(document)) => document,
            Some(Tab::Unloaded(saved)) => {
                // a restored tab not shown yet loads from its new path when it is shown
                if let Some(moved) = moved_saved(saved, (from, to)) {
                    self.tabs.replace(id, Tab::Unloaded(moved));
                }
                return;
            }
            _ => return,
        };
        let path = moved_path(document.path(), (from, to));
        let tab = match Document::open_stamped(&path) {
            Ok(opened) => Tab::Document(FileTab::new(opened, cx)),
            Err(error) => Tab::Failed { title: crate::project::folder_name(&path), failure: crate::failure::Failure::new(error.to_string()) },
        };
        self.tabs.replace(id, tab);
        self.save_on_state_change(id, cx);
        self.tab_shown(window, cx);
    }
}

/// A rename that ran: the row it was started on, the paths, and how it went.
struct Renamed {
    key: TreeKey,
    from: PathBuf,
    to: PathBuf,
    result: Result<(), String>,
}

/// `1 tab`, `3 tabs`.
fn plan_count(tabs: usize) -> String {
    crate::transfer::plan::counted(tabs, "tab", "tabs")
}

/// Whether the tabs of what an action touches have their edits on disk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Settled {
    Saved,
    /// a tab holds edits that could not be saved: the action stops
    Held,
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
    fn should_leave_out_paths_gone_from_the_work_tree() {
        // given
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("here.rs"), "").unwrap();
        let subject = subject_of(vec![(dir.path().join("here.rs"), PathKind::File), (dir.path().join("gone.rs"), PathKind::File)], &[dir.path().to_path_buf()]);

        // when
        let kept = in_work_tree(&subject);

        // then
        assert_eq!(kept.paths.iter().map(|path| path.path.clone()).collect::<Vec<_>>(), [dir.path().join("here.rs")]);
    }

    #[test]
    fn should_move_a_restored_file_tab_inside_a_renamed_folder() {
        // given
        let saved = SavedTab::File { path: PathBuf::from("/r/src/b.rs"), panel: Default::default() };

        // when
        let moved = moved_saved(&saved, (Path::new("/r/src"), Path::new("/r/lib")));

        // then
        assert_eq!(moved.as_ref().and_then(saved_file), Some(PathBuf::from("/r/lib/b.rs")));
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
    fn should_not_refuse_a_git_folder_that_goes_with_its_folder() {
        // given
        let subject = subject_of(vec![(PathBuf::from("/r/d"), PathKind::Folder), (PathBuf::from("/r/d/.git"), PathKind::Folder)], &checkouts());

        // when
        let (kept, refused) = deletable(&subject, &checkouts());

        // then
        assert_eq!((kept, refused), (vec![PathBuf::from("/r/d")], Vec::<String>::new()));
    }

    #[test]
    fn should_name_every_refusal() {
        // when
        let text = listed_note("Delete refused", &["/r is a checkout's root".to_owned(), "/r/.git is git's own".to_owned()]);

        // then
        assert_eq!(text, Some("Delete refused: /r is a checkout's root; /r/.git is git's own".to_owned()));
    }

    #[test]
    fn should_say_nothing_without_refusals() {
        // when
        let text = listed_note("Delete refused", &[]);

        // then
        assert_eq!(text, None);
    }

    #[test]
    fn should_key_a_renamed_change_by_its_new_path() {
        // given
        let key = TreeKey::Change(PathBuf::from("/r"), crate::sidebar::folders::ChangeSource::Uncommitted, crate::git::types::GitPath::from_bytes(b"src/a.rs"));

        // when
        let renamed = [renamed_key(&key, Path::new("/r/src/b.rs")), renamed_key(&TreeKey::Entry(PathBuf::from("/r/a")), Path::new("/r/b"))];

        // then
        let expected = TreeKey::Change(PathBuf::from("/r"), crate::sidebar::folders::ChangeSource::Uncommitted, crate::git::types::GitPath::from_bytes(b"src/b.rs"));
        assert_eq!(renamed, [Some(expected), Some(TreeKey::Entry(PathBuf::from("/r/b")))]);
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
        // as text: `PathBuf` equality ignores a trailing `/`, which a file's path must not have
        let file = Path::new("/r/lib").join("a.rs");
        assert_eq!(moved.map(|path| path.display().to_string()), [file.display().to_string(), "/r/lib".to_owned()]);
    }
}
