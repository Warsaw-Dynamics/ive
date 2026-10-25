//! Files and folders dropped on tree rows, and Copy and Paste in the tree: what a drop would do, the layout menu, the
//! copy or move off the UI thread, and the line under the tree saying how it went.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use gpui::{ClipboardItem, Context, Modifiers, Window};

use super::Workspace;
use super::menu::MenuOn;
use crate::menu::resolve::{self, Applies, Choice};
use crate::project::CheckoutRef;
use crate::script::keymap::Builtin;
use crate::sidebar::drop_place::{DropHint, DropPlace, files_place};
use crate::sidebar::rows::TreeKey;
use crate::sidebar::view::TreeNote;
use crate::conflicts_tab::{ConflictsTab, Details};
use crate::editor::Tab;
use crate::tabs::TabId;
use crate::transfer::plan::{self, Action, Layout, LayoutChoice, Operation, Plan, Source, SourceKind, layout_choice, place_name, refusal, verb_for};
use crate::transfer::scan::{self, classify, existing};
use crate::transfer::write::{self, Report};
use crate::transfer::{DropModifiers, Verb};

/// How long the line under the tree stays after a copy or move that went well.
const NOTE_FOR: Duration = Duration::from_secs(5);
/// The layout menu's choices.
const KEEP_FOLDERS: &str = "keep-folders";
const FLAT: &str = "flat";
const CANCEL: &str = "cancel";

/// One copy or move at a time; the drop the layout menu asks about; whether the clipboard held files when the last
/// tree menu opened.
#[derive(Debug, Default)]
pub(crate) struct Transfers {
    running: bool,
    pending: Option<Operation>,
    clipboard_files: bool,
}

/// How a copy or move ended, off the UI thread.
enum Outcome {
    Done { report: Report, operation: Operation },
    Conflicts(Plan, Details),
    Refused(String),
}

/// The paths Copy put on the clipboard: every line not blank, each an absolute path that exists; `None` for anything
/// else on it.
pub fn clipboard_paths(text: &str) -> Option<Vec<PathBuf>> {
    let paths: Vec<PathBuf> = text.lines().map(str::trim).filter(|line| !line.is_empty()).map(PathBuf::from).collect();
    let usable = !paths.is_empty() && paths.iter().all(|path| path.is_absolute() && std::fs::symlink_metadata(path).is_ok());
    usable.then_some(paths)
}

/// Paste in a row's menu: hidden on a row that takes no files, greyed while the clipboard holds none.
pub fn paste_applies(takes_files: bool, clipboard_has_files: bool) -> Applies {
    match takes_files {
        true => Applies::Shown { enabled: clipboard_has_files },
        false => Applies::Hidden,
    }
}

/// What a drop's hint needs besides the files: the verb the modifiers force, the copy modifier's name, and the
/// target as the tree names it (`dev · lib`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hint {
    pub forced: Option<Verb>,
    pub copy_name: String,
    pub place: String,
}

/// What dropping `sources` on `place` does (`None`: refused) and the hint saying so:
/// `Copy 2 items to dev · lib · flat: from 2 worktrees`.
pub fn files_hint(sources: &[Source], place: &DropPlace, hint: &Hint) -> (Option<Verb>, String) {
    let verb = verb_for(sources, place, hint.forced);
    if let Some(refused) = refusal(sources, place, verb) {
        return (None, refused.text(&hint.copy_name));
    }
    let what = match sources {
        [only] => only.path.file_name().map_or_else(|| only.path.display().to_string(), |name| name.to_string_lossy().into_owned()),
        many => format!("{} items", many.len()),
    };
    let layout = match layout_choice(sources) {
        LayoutChoice::Straight => String::new(),
        LayoutChoice::Ask { .. } => " · asks: keep folders or flat".to_owned(),
        LayoutChoice::Flat => format!(" · flat: from {} worktrees", distinct_checkouts(sources)),
    };
    let said = if verb == Verb::Move { "Move" } else { "Copy" };
    (Some(verb), format!("{said} {what} to {}{layout}", hint.place))
}

fn distinct_checkouts(sources: &[Source]) -> usize {
    let mut seen: Vec<&Path> = sources.iter().map(|source| source.checkout.path.as_path()).collect();
    seen.sort();
    seen.dedup();
    seen.len()
}

impl Workspace {
    /// What dropping the files of a drag of `pressed` on the row of `row` would do, for the drag's chip.
    pub(crate) fn files_drop_hint(&self, pressed: &TreeKey, row: &TreeKey, modifiers: &Modifiers, cx: &Context<Self>) -> DropHint {
        match self.files_drop(pressed, row, modifiers, cx) {
            Some((verb, text, ..)) => DropHint { row: row.clone(), text, allowed: verb.is_some() },
            None => DropHint { row: row.clone(), text: "Drop on a folder, a worktree or a project".to_owned(), allowed: false },
        }
    }

    /// The verb (`None`: refused), hint, sources and place of a drop on the row of `row`; `None` when the row takes
    /// no files.
    fn files_drop(&self, pressed: &TreeKey, row: &TreeKey, modifiers: &Modifiers, cx: &Context<Self>) -> Option<(Option<Verb>, String, Vec<Source>, DropPlace)> {
        let place = self.files_target(row)?;
        let sources = self.sources_of(&self.dragged_paths(pressed), &place.checkout);
        let configured = cx.try_global::<DropModifiers>().copied().unwrap_or_default();
        let hint = Hint { forced: configured.forced(modifiers), copy_name: configured.copy_name(), place: place_name(&place, |checkout| self.checkout_label(checkout)) };
        let (verb, text) = files_hint(&sources, &place, &hint);
        Some((verb, text, sources, place))
    }

    /// The folder files dropped on the row of `row` go into, in the checkout that holds it (a worktree inside the
    /// main checkout's folder owns its files).
    fn files_target(&self, row: &TreeKey) -> Option<DropPlace> {
        let item = &self.tree.rows.iter().find(|listed| listed.key() == *row)?.item;
        let place = files_place(item, |project| self.main_of(project))?;
        let checkout = self.checkout_of_path(&place.folder).unwrap_or(place.checkout);
        Some(DropPlace { checkout, folder: place.folder })
    }

    /// `paths` as sources: each in the deepest checkout holding it; one outside every checkout stands in a
    /// "checkout" of its own folder, in `fallback`'s project, so it is never laid out with others.
    fn sources_of(&self, paths: &[PathBuf], fallback: &CheckoutRef) -> Vec<Source> {
        let source = |path: &PathBuf| {
            let kind = if std::fs::symlink_metadata(path).is_ok_and(|metadata| metadata.is_dir()) { SourceKind::Folder } else { SourceKind::File };
            let own_folder = || CheckoutRef { project: fallback.project.clone(), path: path.parent().unwrap_or(path).to_path_buf() };
            Source { path: path.clone(), checkout: self.checkout_of_path(path).unwrap_or_else(own_folder), kind }
        };
        paths.iter().map(source).collect()
    }

    /// Files dropped on the row of `onto`: refused as the hint said, asked about with the layout menu, or copied or
    /// moved.
    pub(crate) fn drop_files(&mut self, (pressed, onto): (&TreeKey, TreeKey), modifiers: &Modifiers, window: &mut Window, cx: &mut Context<Self>) {
        self.tree.drop_hint = None;
        let Some((Some(verb), _, sources, target)) = self.files_drop(pressed, &onto, modifiers, cx) else {
            return cx.notify();
        };
        let operation = Operation { verb, sources, target, layout: Layout::Flat };
        match layout_choice(&operation.sources) {
            LayoutChoice::Ask { common } => self.ask_layout(operation, &common, window, cx),
            LayoutChoice::Straight | LayoutChoice::Flat => self.start_transfer(operation, window, cx),
        }
    }

    /// The menu at the pointer asking how files from several folders of one checkout go in.
    fn ask_layout(&mut self, operation: Operation, common: &Path, window: &mut Window, cx: &mut Context<Self>) {
        let below = common.strip_prefix(&operation.sources[0].checkout.path).ok().filter(|inside| !inside.as_os_str().is_empty());
        let from = below.map_or_else(|| "/".to_owned(), |inside| format!("{}/", inside.display()));
        let target = &operation.target;
        let into = match target.folder == target.checkout.path {
            true => "/".to_owned(),
            false => format!("{}/", target.folder.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default()),
        };
        let item = |label: String, choice: &str| Choice::Item { label, choice: choice.to_owned(), enabled: true };
        let choices = vec![item(format!("Keep folders, from {from}"), KEEP_FOLDERS), item(format!("Flat, all into {into}"), FLAT), Choice::Separator, item("Cancel".to_owned(), CANCEL)];
        let lines = resolve::choice_lines(Builtin::PasteFiles, choices, &[]);
        let on = MenuOn::Tree { row: "drop".to_owned(), pressed: TreeKey::Entry(target.folder.clone()) };
        self.transfers.pending = Some(operation);
        self.open_lines(on, crate::menu::Subject::empty(), lines, window, cx);
    }

    /// A line of the layout menu chosen.
    pub(super) fn choose_layout(&mut self, choice: &str, window: &mut Window, cx: &mut Context<Self>) {
        let Some(operation) = self.transfers.pending.take() else { return };
        let layout = match choice {
            KEEP_FOLDERS => Layout::KeepFolders,
            FLAT => Layout::Flat,
            _ => return,
        };
        self.start_transfer(Operation { layout, ..operation }, window, cx);
    }

    /// Copy: the picked files and folders onto the clipboard, as absolute paths one per line.
    pub(super) fn copy_files(&mut self, pressed: &TreeKey, cx: &mut Context<Self>) {
        let subject = self.tree_subject(pressed);
        if subject.paths.is_empty() {
            return;
        }
        let paths = crate::sidebar::selection::without_nested(subject.paths.into_iter().map(|path| path.path).collect());
        let lines: Vec<String> = paths.iter().map(|path| path.display().to_string()).collect();
        cx.write_to_clipboard(ClipboardItem::new_string(lines.join("\n")));
    }

    /// Paste: the files and folders on the clipboard copied into the folder of the row of `pressed`, keeping their
    /// folders when they come from one checkout, flat otherwise.
    pub(super) fn paste_files(&mut self, pressed: &TreeKey, window: &mut Window, cx: &mut Context<Self>) {
        let Some(paths) = cx.read_from_clipboard().and_then(|item| item.text()).and_then(|text| clipboard_paths(&text)) else {
            return self.note("The clipboard holds no files to paste".to_owned(), NoteKind::Fading, cx);
        };
        let Some(target) = self.files_target(pressed) else { return };
        let sources = self.sources_of(&paths, &target.checkout);
        let one_checkout = sources.iter().all(|source| self.checkout_of_path(&source.path).is_some()) && distinct_checkouts(&sources) == 1;
        let layout = if one_checkout { Layout::KeepFolders } else { Layout::Flat };
        let operation = Operation { verb: Verb::Copy, sources, target, layout };
        if let Some(refused) = refusal(&operation.sources, &operation.target, Verb::Copy) {
            return self.note(refused.text(""), NoteKind::Fading, cx);
        }
        self.start_transfer(operation, window, cx);
    }

    /// Whether the clipboard holds files, read when a tree menu opens: Paste is greyed without them.
    pub(super) fn read_clipboard_files(&mut self, cx: &Context<Self>) {
        self.transfers.clipboard_files = cx.read_from_clipboard().and_then(|item| item.text()).and_then(|text| clipboard_paths(&text)).is_some();
    }

    /// Copy and Paste in a tree row's menu.
    pub(super) fn clip_applies(&self, builtin: Builtin, pressed: &TreeKey) -> Applies {
        match builtin {
            Builtin::CopyFiles if matches!(pressed, TreeKey::Entry(_) | TreeKey::Change(..)) => Applies::Shown { enabled: true },
            Builtin::PasteFiles => paste_applies(self.files_target(pressed).is_some(), self.transfers.clipboard_files),
            _ => Applies::Hidden,
        }
    }

    /// Plans `operation` off the UI thread, then writes it, or opens the Conflicts tab when anything is in the way;
    /// refused while another copy or move runs.
    fn start_transfer(&mut self, operation: Operation, window: &mut Window, cx: &mut Context<Self>) {
        if self.transfers.running {
            return self.note("A copy is running".to_owned(), NoteKind::Fading, cx);
        }
        self.transfers.running = true;
        let doing = if operation.verb == Verb::Move { "Moving" } else { "Copying" };
        let what = plan::counted(operation.sources.len(), "item", "items");
        let place = place_name(&operation.target, |checkout| self.checkout_label(checkout));
        self.note(format!("{doing} {what} to {place}…"), NoteKind::Staying, cx);
        let others = self.all_checkouts();
        cx.spawn_in(window, async move |this, cx| {
            let outcome = cx.background_executor().spawn(async move { transferred(operation, &others) }).await;
            let _ = this.update_in(cx, |workspace, window, cx| workspace.finish_transfer(outcome, window, cx));
        })
        .detach();
    }

    fn finish_transfer(&mut self, outcome: Outcome, window: &mut Window, cx: &mut Context<Self>) {
        self.transfers.running = false;
        match outcome {
            Outcome::Refused(why) => self.note(why, NoteKind::Staying, cx),
            Outcome::Conflicts(plan, details) => {
                self.tree.note = None;
                self.open_conflicts(plan, details, window, cx);
            }
            Outcome::Done { report, operation } => self.reported(&report, &operation, cx),
        }
    }

    /// A Conflicts tab asking about `plan`; it opens permanent, after the shown tab.
    fn open_conflicts(&mut self, plan: Plan, details: Details, window: &mut Window, cx: &mut Context<Self>) {
        self.open_tab(Tab::Conflicts(Box::new(ConflictsTab::new(plan, details))), window, cx);
    }

    fn conflicts_tab_mut(&mut self, id: TabId) -> Option<&mut ConflictsTab> {
        match self.tabs.get_mut(id) {
            Some(Tab::Conflicts(tab)) => Some(tab),
            _ => None,
        }
    }

    /// Row `index` of Conflicts tab `id` gets `action`.
    pub(crate) fn set_conflict(&mut self, id: TabId, (index, action): (usize, Action), cx: &mut Context<Self>) {
        if let Some(tab) = self.conflicts_tab_mut(id) {
            tab.set_action(index, action);
            cx.notify();
        }
    }

    pub(crate) fn set_all_conflicts(&mut self, id: TabId, action: Action, cx: &mut Context<Self>) {
        if let Some(tab) = self.conflicts_tab_mut(id) {
            tab.set_all(action);
            cx.notify();
        }
    }

    /// Unfolds or folds both texts of row `index`.
    pub(crate) fn toggle_conflict_row(&mut self, id: TabId, index: usize, cx: &mut Context<Self>) {
        if let Some(tab) = self.conflicts_tab_mut(id) {
            tab.read_preview(index);
            tab.toggle_row(index);
            cx.notify();
        }
    }

    pub(crate) fn toggle_conflicts_clean(&mut self, id: TabId, cx: &mut Context<Self>) {
        if let Some(tab) = self.conflicts_tab_mut(id) {
            tab.clean_unfolded = !tab.clean_unfolded;
            cx.notify();
        }
    }

    /// Copy (or Move) in Conflicts tab `id`: once every conflict is decided, the tab closes and the plan is written.
    pub(crate) fn apply_conflicts(&mut self, id: TabId, window: &mut Window, cx: &mut Context<Self>) {
        let Some(plan) = self.conflicts_tab_mut(id).filter(|tab| tab.can_apply()).map(|tab| tab.plan.clone()) else { return };
        if self.transfers.running {
            return self.note("A copy is running".to_owned(), NoteKind::Fading, cx);
        }
        self.close_tab(id, window, cx);
        self.transfers.running = true;
        cx.spawn_in(window, async move |this, cx| {
            let outcome = cx.background_executor().spawn(async move { written(plan) }).await;
            let _ = this.update_in(cx, |workspace, window, cx| workspace.finish_transfer(outcome, window, cx));
        })
        .detach();
    }

    /// The line saying how it went, and what was written picked in the tree.
    pub(super) fn reported(&mut self, report: &Report, operation: &Operation, cx: &mut Context<Self>) {
        let line = report.line(operation, |checkout| self.checkout_label(checkout));
        let kind = if report.failed.is_some() { NoteKind::Staying } else { NoteKind::Fading };
        self.note(line, kind, cx);
        self.reveal_written(&top_level(&report.written, &operation.target.folder), cx);
    }

    /// Unfolds the tree down to the first of `items` and, once it is there, picks them all.
    fn reveal_written(&mut self, items: &[PathBuf], cx: &mut Context<Self>) {
        let Some(first) = items.first() else { return };
        self.locating = Some(TreeKey::Entry(first.clone()));
        self.locating_picks = items.iter().cloned().map(TreeKey::Entry).collect();
        self.continue_locating(cx);
        cx.notify();
    }

    pub(super) fn note(&mut self, text: String, kind: NoteKind, cx: &mut Context<Self>) {
        let until = (kind == NoteKind::Fading).then(|| Instant::now() + NOTE_FOR);
        self.tree.note = Some(TreeNote { text, until });
        cx.notify();
    }

    /// A click on the line under the tree.
    pub(crate) fn dismiss_note(&mut self, cx: &mut Context<Self>) {
        self.tree.note = None;
        cx.notify();
    }

    /// Once a second: the line of a copy that went well goes after a while.
    pub(super) fn expire_note(&mut self, cx: &mut Context<Self>) {
        if self.tree.note.as_ref().and_then(|note| note.until).is_some_and(|until| until <= Instant::now()) {
            self.dismiss_note(cx);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum NoteKind {
    /// goes after [`NOTE_FOR`]
    Fading,
    /// stays until clicked, or until the next one
    Staying,
}

/// What `written` put directly in `folder`: the files there, and the folders holding the rest, each once.
fn top_level(written: &[PathBuf], folder: &Path) -> Vec<PathBuf> {
    let mut items: Vec<PathBuf> = Vec::new();
    for path in written {
        let first = path.strip_prefix(folder).ok().and_then(|inside| inside.components().next()).map(|part| folder.join(part));
        if let Some(item) = first.filter(|item| !items.contains(item)) {
            items.push(item);
        }
    }
    items
}

/// The disk work of a copy or move: a whole move by rename where it can, else walk, plan and classify; a plan with
/// conflicts goes back to ask, one without is written.
fn transferred(operation: Operation, other_checkouts: &[PathBuf]) -> Outcome {
    if let Some(report) = write::move_whole(&operation, other_checkouts) {
        return Outcome::Done { report, operation };
    }
    let walked = match scan::walk(&operation.sources, operation.verb, other_checkouts) {
        Ok(walked) => walked,
        Err(why) => return Outcome::Refused(why),
    };
    let mut planned = plan::plan(operation, walked.incoming, existing);
    classify(&mut planned);
    planned.left_out = walked.left_out;
    if planned.conflicts().next().is_some() {
        let details = described(&planned);
        return Outcome::Conflicts(planned, details);
    }
    written(planned)
}

/// Writes `planned` and says how it went.
fn written(planned: Plan) -> Outcome {
    let report = Report { left_out: planned.left_out, ..write::execute(&planned) };
    Outcome::Done { report, operation: planned.operation }
}

/// Both files of each conflicting row, as the Conflicts tab lists them.
fn described(planned: &Plan) -> Details {
    let rows = planned.rows.iter().enumerate().filter(|(_, row)| row.conflict.is_some());
    rows.map(|(index, row)| (index, (scan::describe(&row.incoming.path), row.there.map_or_else(|| "—".to_owned(), |_| scan::describe(&row.destination))))).collect()
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use super::*;
    use crate::project::{CheckoutRef, ProjectId};
    use crate::sidebar::drop_place::DropPlace;
    use crate::transfer::plan::{Source, SourceKind};

    fn checkout(path: &str) -> CheckoutRef {
        CheckoutRef { project: ProjectId::new("p"), path: PathBuf::from(path) }
    }

    fn file(at: &str, relative: &str) -> Source {
        Source { path: Path::new(at).join(relative), checkout: checkout(at), kind: SourceKind::File }
    }

    fn lib() -> DropPlace {
        DropPlace { checkout: checkout("/dev"), folder: PathBuf::from("/dev/lib") }
    }

    #[test]
    fn should_read_existing_absolute_paths_from_clipboard() {
        // given
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a.rs"), "a").unwrap();
        fs::create_dir(dir.path().join("src")).unwrap();
        let text = format!("{}\n{}\n", dir.path().join("a.rs").display(), dir.path().join("src").display());

        // when
        let paths = clipboard_paths(&text);

        // then
        assert_eq!(paths, Some(vec![dir.path().join("a.rs"), dir.path().join("src")]));
    }

    #[test]
    fn should_refuse_clipboard_with_relative_or_missing_path() {
        // given
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a.rs"), "a").unwrap();
        let texts = [format!("{}\nsrc/main.rs", dir.path().join("a.rs").display()), dir.path().join("gone.rs").display().to_string(), "hello".to_owned()];

        // when
        let read = texts.map(|text| clipboard_paths(&text));

        // then
        assert_eq!(read, [None, None, None]);
    }

    #[test]
    fn should_ignore_blank_clipboard_lines() {
        // given
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a.rs"), "a").unwrap();
        let text = format!("\n  \n{}\n\n", dir.path().join("a.rs").display());

        // when
        let paths = clipboard_paths(&text);

        // then
        assert_eq!(paths, Some(vec![dir.path().join("a.rs")]));
    }

    #[test]
    fn should_grey_paste_without_files_on_clipboard() {
        // when
        let applies = [paste_applies(true, false), paste_applies(true, true), paste_applies(false, true)];

        // then
        assert_eq!(applies, [Applies::Shown { enabled: false }, Applies::Shown { enabled: true }, Applies::Hidden]);
    }

    #[test]
    fn should_hint_copy_across_worktrees_flat() {
        // given
        let sources = [file("/wt-a", "a.rs"), file("/wt-b", "b.rs")];

        // when
        let hint = files_hint(&sources, &lib(), &Hint { forced: None, copy_name: "Ctrl".to_owned(), place: "dev · lib".to_owned() });

        // then
        assert_eq!(hint, (Some(Verb::Copy), "Copy 2 items to dev · lib · flat: from 2 worktrees".to_owned()));
    }

    #[test]
    fn should_hint_asking_for_layout_from_folders_of_one_checkout() {
        // given
        let sources = [file("/dev", "src/a.rs"), file("/dev", "doc/b.md")];

        // when
        let hint = files_hint(&sources, &lib(), &Hint { forced: None, copy_name: "Ctrl".to_owned(), place: "dev · lib".to_owned() });

        // then
        assert_eq!(hint, (Some(Verb::Move), "Move 2 items to dev · lib · asks: keep folders or flat".to_owned()));
    }

    #[test]
    fn should_hint_refusal_with_copy_key() {
        // given
        let sources = [file("/dev", "lib/a.rs")];

        // when
        let hint = files_hint(&sources, &lib(), &Hint { forced: None, copy_name: "Alt".to_owned(), place: "dev · lib".to_owned() });

        // then
        assert_eq!(hint, (None, "Already here; Alt+drop copies".to_owned()));
    }

    #[test]
    fn should_name_one_dragged_file() {
        // when
        let hint = files_hint(&[file("/wt-a", "a.rs")], &lib(), &Hint { forced: None, copy_name: "Ctrl".to_owned(), place: "dev · lib".to_owned() });

        // then
        assert_eq!(hint.1, "Copy a.rs to dev · lib");
    }
}
