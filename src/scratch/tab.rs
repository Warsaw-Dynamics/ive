//! The scratch tab: where the scratch lives and runs, its editor, and saving.

use std::path::{Path, PathBuf};
use std::time::Duration;

use gpui::{
    AnyElement, App, ClickEvent, Context, Div, Entity, FocusHandle, Focusable, SharedString, Stateful, Subscription, Task,
    WeakEntity, Window, div, img, prelude::*, rgb,
};

use super::store::{FileStamp, LineEnding, MAX_SCRATCH_BYTES, ScratchContent, ScratchError, ScratchStore};
use super::{ScratchRef, log_error, run};
use crate::click_targets::target;
use crate::document::Document;
use crate::editor::{self, EditorView, Reread};
use crate::icons;
use crate::paths::tilde_home;
use crate::project::ProjectId;
use crate::saved_tabs::{CaretPosition, ScrollPosition};
use crate::text_editor::{SizeLimit, TextEditor, TextEditorEvent};
use crate::theme::Theme;
use crate::workspace::Workspace;

const SAVE_DELAY: Duration = Duration::from_millis(500);
const TOO_LARGE: &str = "too large to edit here";
const NOT_UTF8: &str = "not UTF-8, cannot edit";

/// How saving treats a file that changed on disk since IVE last read or wrote it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveMode {
    /// the 500 ms save: a change on disk or a removed file is the user's call
    Ask,
    /// Overwrite, and closing: unsaved edits win over the disk
    Overwrite,
    /// Save again: writes the text even without unsaved edits, bringing back a removed file
    Always,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveAction {
    Nothing,
    Write,
    Conflict,
    Deleted,
}

/// What a change on disk means for the tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiskAction {
    /// the file is as IVE last read or wrote it: its own save
    Unchanged,
    Reload,
    Conflict,
    Deleted,
}

/// The saved revision of the editor's text and the file as IVE last saw it.
pub struct SaveState {
    saved_revision: u64,
    stamp: FileStamp,
    /// the scratch was deleted; nothing may write it back
    forgotten: bool,
}

impl SaveState {
    pub fn new(saved_revision: u64, stamp: FileStamp) -> Self {
        Self { saved_revision, stamp, forgotten: false }
    }

    /// `on_disk` is the file's stamp now, `None` once it is gone.
    pub fn action(&self, revision: u64, on_disk: Option<FileStamp>, mode: SaveMode) -> SaveAction {
        let saved = revision == self.saved_revision;
        match (on_disk, mode) {
            _ if self.forgotten => SaveAction::Nothing,
            (_, SaveMode::Always) => SaveAction::Write,
            _ if saved => SaveAction::Nothing,
            (None, SaveMode::Ask) => SaveAction::Deleted,
            (Some(stamp), SaveMode::Ask) if stamp != self.stamp => SaveAction::Conflict,
            _ => SaveAction::Write,
        }
    }

    /// After the watched file changed: its own saves leave the stamp IVE wrote.
    pub fn on_disk_change(&self, revision: u64, on_disk: Option<FileStamp>) -> DiskAction {
        match on_disk {
            None => DiskAction::Deleted,
            Some(stamp) if stamp == self.stamp => DiskAction::Unchanged,
            Some(_) if revision == self.saved_revision => DiskAction::Reload,
            Some(_) => DiskAction::Conflict,
        }
    }

    pub fn saved(&mut self, revision: u64, stamp: FileStamp) {
        self.saved_revision = revision;
        self.stamp = stamp;
    }

    pub fn forget(&mut self) {
        self.forgotten = true;
    }
}

/// What a scratch tab needs to know about its scratch.
pub struct ScratchTarget {
    pub scratch: ScratchRef,
    pub project_name: String,
    pub store: ScratchStore,
    pub workspace: WeakEntity<Workspace>,
}

/// The `Tab::Scratch` content. Name and project are kept here so the tab row needs no entity read.
pub struct ScratchTab {
    scratch: ScratchRef,
    project_name: String,
    /// registered in `OpenPaths` while the tab is open
    path: PathBuf,
    view: Entity<ScratchView>,
}

impl ScratchTab {
    /// Editable, or read-only in the document view when too large or not UTF-8.
    pub fn open(target: ScratchTarget, cx: &mut Context<Workspace>) -> Result<Self, ScratchError> {
        let content = target.store.read(&target.scratch.name)?;
        let (scratch, project_name) = (target.scratch.clone(), target.project_name.clone());
        let path = target.store.path(&scratch.name);
        let view = cx.new(|cx| ScratchView::new(target, content, cx));
        Ok(Self { scratch, project_name, path, view })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn name(&self) -> &str {
        &self.scratch.name
    }

    pub fn project(&self) -> &ProjectId {
        &self.scratch.project
    }

    pub fn project_name(&self) -> &str {
        &self.project_name
    }

    pub fn shows(&self, scratch: &ScratchRef) -> bool {
        self.scratch == *scratch
    }

    pub fn view(&self) -> &Entity<ScratchView> {
        &self.view
    }

    pub fn view_state(&self, cx: &App) -> (CaretPosition, ScrollPosition) {
        self.view.read(cx).view_state(cx)
    }

    pub fn restore_view(&self, caret: CaretPosition, scroll: ScrollPosition, cx: &mut App) {
        self.view.update(cx, |view, cx| view.restore_view(caret, scroll, cx));
    }

    pub fn focus_handle(&self, cx: &App) -> Option<FocusHandle> {
        self.view.read(cx).editor().map(|editor| editor.focus_handle(cx))
    }

    /// Saves now instead of after the delay.
    pub fn save(&self, cx: &mut App) {
        self.view.update(cx, |view, cx| view.save_with(SaveMode::Ask, cx));
    }

    /// Before the file is deleted: the tab must not write it back when it closes.
    pub fn forget(&self, cx: &mut App) {
        self.view.update(cx, |view, _cx| view.forget());
    }

    pub fn rename(&mut self, name: &str, cx: &mut App) {
        self.scratch.name = name.to_owned();
        self.path.set_file_name(name);
        self.view.update(cx, |view, cx| view.renamed(name, cx));
    }

    /// For MCP `send_text`: the text goes to the editor as typed.
    pub fn type_text(&self, text: &str, cx: &mut App) -> Result<(), String> {
        let editor = self.view.read(cx).editor().ok_or_else(|| format!("scratch {} is read-only", self.scratch.name))?;
        editor.update(cx, |editor, cx| editor.type_text(text, cx)).map_err(|error| error.to_string())
    }
}

enum Body {
    Editing { editor: Entity<TextEditor>, ending: LineEnding, save: SaveState },
    /// boxed: a document view is many times the size of the editing variant
    ReadOnly { document: Result<Box<EditorView>, String>, note: &'static str },
}

/// A red bar under the header.
enum Bar {
    Failed(String),
    ChangedOnDisk,
    DeletedOnDisk,
    Refused(String),
    /// Run met a change on disk the tab had not shown yet
    Reloaded,
}

impl Bar {
    /// Says something about the last action only; the next edit or save takes it away.
    fn is_passing(&self) -> bool {
        matches!(self, Self::Refused(_) | Self::Reloaded)
    }
}

/// What a save did; the 500 ms save often has nothing to write.
enum Saved {
    Nothing,
    Written,
}

pub struct ScratchView {
    target: ScratchTarget,
    body: Body,
    bar: Option<Bar>,
    save_task: Option<Task<()>>,
    /// a read-only scratch's re-read after a change on disk
    read_task: Option<Task<()>>,
    _subscriptions: Vec<Subscription>,
}

impl ScratchView {
    fn new(target: ScratchTarget, content: ScratchContent, cx: &mut Context<Self>) -> Self {
        // closing the tab, closing the window and quitting all release the view: the last chance to save
        let mut subscriptions = vec![cx.on_release(|view: &mut Self, cx| view.flush(cx))];
        let body = match content {
            ScratchContent::Editable { text, ending, stamp } => {
                let editor = cx.new(|cx| TextEditor::new(&text, limit(ending), cx));
                subscriptions.push(cx.subscribe(&editor, |view, _editor, event: &TextEditorEvent, cx| view.on_editor_event(event, cx)));
                // caret and scroll are saved with the open tabs
                subscriptions.push(cx.observe(&editor, |view, _editor, cx| view.schedule_workspace_save(cx)));
                Body::Editing { editor, ending, save: SaveState::new(0, stamp) }
            }
            ScratchContent::TooLarge => read_only(&target, TOO_LARGE),
            ScratchContent::NotUtf8 => read_only(&target, NOT_UTF8),
        };
        Self { target, body, bar: None, save_task: None, read_task: None, _subscriptions: subscriptions }
    }

    fn restore_view(&self, caret: CaretPosition, scroll: ScrollPosition, cx: &mut Context<Self>) {
        match &self.body {
            Body::Editing { editor, .. } => editor.update(cx, |editor, cx| editor.restore_view(caret, scroll, cx)),
            Body::ReadOnly { document: Ok(document), .. } => document.restore_scroll(scroll),
            Body::ReadOnly { document: Err(_), .. } => {}
        }
    }

    fn schedule_workspace_save(&self, cx: &mut Context<Self>) {
        self.target.workspace.update(cx, |workspace, cx| workspace.schedule_save(cx)).ok();
    }

    /// Caret and scroll to save; a read-only scratch has no caret, so it saves the start.
    fn view_state(&self, cx: &App) -> (CaretPosition, ScrollPosition) {
        match &self.body {
            Body::Editing { editor, .. } => (editor.read(cx).caret_position(), editor.read(cx).scroll_position()),
            Body::ReadOnly { document: Ok(document), .. } => (CaretPosition::default(), document.scroll_position()),
            Body::ReadOnly { document: Err(_), .. } => (CaretPosition::default(), ScrollPosition::default()),
        }
    }

    fn editor(&self) -> Option<Entity<TextEditor>> {
        match &self.body {
            Body::Editing { editor, .. } => Some(editor.clone()),
            Body::ReadOnly { .. } => None,
        }
    }

    fn on_editor_event(&mut self, event: &TextEditorEvent, cx: &mut Context<Self>) {
        match event {
            TextEditorEvent::Edited => {
                self.bar.take_if(|bar| bar.is_passing());
                self.schedule_save(cx);
            }
            TextEditorEvent::Refused(reason) => self.bar = Some(Bar::Refused(reason.clone())),
        }
        cx.notify();
    }

    fn schedule_save(&mut self, cx: &mut Context<Self>) {
        // replacing the task cancels the previous wait, so the save comes SAVE_DELAY after the last edit
        self.save_task = Some(cx.spawn(async move |view, cx| {
            cx.background_executor().timer(SAVE_DELAY).await;
            view.update(cx, |view, cx| view.save_with(SaveMode::Ask, cx)).ok();
        }));
    }

    fn save_with(&mut self, mode: SaveMode, cx: &mut Context<Self>) {
        self.save_task = None;
        match self.write_out(mode, cx) {
            // a bar about the file stays until a write settles it
            Ok(Saved::Nothing) => {}
            Ok(Saved::Written) => self.bar = self.bar.take().filter(|bar| matches!(bar, Bar::Refused(_))),
            Err(bar) => self.bar = Some(bar),
        }
        cx.notify();
    }

    /// Writes unsaved edits; the error is the bar to show.
    fn write_out(&mut self, mode: SaveMode, cx: &App) -> Result<Saved, Bar> {
        let Body::Editing { editor, ending, save } = &mut self.body else {
            return Ok(Saved::Nothing);
        };
        let (store, name) = (&self.target.store, &self.target.scratch.name);
        let revision = editor.read(cx).revision();
        match save.action(revision, store.stamp(name).ok(), mode) {
            SaveAction::Nothing => Ok(Saved::Nothing),
            SaveAction::Conflict => Err(Bar::ChangedOnDisk),
            SaveAction::Deleted => Err(Bar::DeletedOnDisk),
            SaveAction::Write => {
                let written = store.write(name, &editor.read(cx).text(), *ending).inspect_err(log_error);
                save.saved(revision, written.map_err(|error| Bar::Failed(error.to_string()))?);
                Ok(Saved::Written)
            }
        }
    }

    /// On release there is no bar left to show: write errors go to stderr, and the user's edits win over a
    /// change on disk.
    fn flush(&mut self, cx: &App) {
        let _ = self.write_out(SaveMode::Overwrite, cx);
    }

    /// Drops unsaved edits and shows the file as it is on disk.
    fn reload(&mut self, cx: &mut Context<Self>) {
        let read = self.target.store.read(&self.target.scratch.name);
        self.bar = match (read, &mut self.body) {
            (Ok(ScratchContent::Editable { text, ending, stamp }), Body::Editing { editor, ending: current, save }) => {
                editor.update(cx, |editor, cx| editor.reset(&text, limit(ending), cx));
                *current = ending;
                *save = SaveState::new(editor.read(cx).revision(), stamp);
                None
            }
            (Ok(_), _) => Some(Bar::Failed("the file is now too large or not UTF-8; reopen the tab to view it".to_owned())),
            (Err(error), _) => {
                log_error(&error);
                Some(Bar::Failed(error.to_string()))
            }
        };
        cx.notify();
    }

    /// The watched file changed or went away (`Change::File`, `Change::FileRemoved`): reload, or ask.
    pub fn disk_changed(&mut self, cx: &mut Context<Self>) {
        match self.disk_action(cx) {
            // its own save, a `touch`, or the file back as IVE knows it
            Some(DiskAction::Unchanged) => {
                self.bar.take_if(|bar| matches!(bar, Bar::ChangedOnDisk | Bar::DeletedOnDisk));
            }
            Some(DiskAction::Reload) => self.reload(cx),
            Some(DiskAction::Conflict) => self.bar = Some(Bar::ChangedOnDisk),
            Some(DiskAction::Deleted) => self.bar = Some(Bar::DeletedOnDisk),
            None => self.reread_document(cx),
        }
        cx.notify();
    }

    /// What the file on disk means for an editable scratch; `None` for a read-only one.
    fn disk_action(&self, cx: &App) -> Option<DiskAction> {
        let Body::Editing { editor, save, .. } = &self.body else {
            return None;
        };
        let on_disk = self.target.store.stamp(&self.target.scratch.name).ok();
        Some(save.on_disk_change(editor.read(cx).revision(), on_disk))
    }

    /// A read-only scratch re-reads its document view on the background executor, as part 1's document tabs do.
    fn reread_document(&mut self, cx: &mut Context<Self>) {
        let Body::ReadOnly { document: Ok(document), .. } = &self.body else {
            return;
        };
        let (path, known) = (document.path().to_path_buf(), document.stamp());
        self.read_task = Some(cx.spawn(async move |view, cx| {
            let reread = cx.background_executor().spawn(async move { editor::reread(&path, known) }).await;
            view.update(cx, |view, cx| view.take_reread(reread, cx)).ok();
        }));
    }

    fn take_reread(&mut self, reread: Reread, cx: &mut Context<Self>) {
        if let Body::ReadOnly { document: Ok(document), .. } = &mut self.body {
            document.take_reread(reread);
            cx.notify();
        }
    }

    /// Close on the "deleted on disk" bar: the tab goes without writing the file back.
    fn close_unsaved(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.forget();
        let scratch = self.target.scratch.clone();
        self.target.workspace.update(cx, |workspace, cx| workspace.close_scratch(&scratch, window, cx)).ok();
    }

    fn forget(&mut self) {
        self.save_task = None;
        if let Body::Editing { save, .. } = &mut self.body {
            save.forget();
        }
    }

    fn renamed(&mut self, name: &str, cx: &mut Context<Self>) {
        self.target.scratch.name = name.to_owned();
        cx.notify();
    }

    /// Saves, then runs in a new terminal; a failed save stops it, and so does a change on disk, with or without
    /// unsaved edits: what runs is what the editor shows.
    fn run(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.save_task = None;
        if let Err(bar) = self.settle_for_run(cx) {
            self.bar = Some(bar);
            cx.notify();
            return;
        }
        self.bar.take_if(|bar| bar.is_passing());
        let (workspace, scratch, view) = (self.target.workspace.clone(), self.target.scratch.clone(), cx.weak_entity());
        // after this update: the workspace opens and shows a terminal tab, which may read or update this view
        window.defer(cx, move |window, cx| {
            let ran = workspace.update(cx, |workspace, cx| workspace.run_scratch(&scratch, window, cx));
            if let Ok(Err(error)) = ran {
                view.update(cx, |view, cx| view.show_failure(error, cx)).ok();
            }
        });
    }

    /// The file as the editor shows it and saved, or the bar that says why not. A change the tab has not shown
    /// yet is loaded when there are no unsaved edits, and Run waits for another click.
    fn settle_for_run(&mut self, cx: &mut Context<Self>) -> Result<(), Bar> {
        match self.disk_action(cx) {
            Some(DiskAction::Reload) => {
                self.reload(cx);
                return Err(self.bar.take().unwrap_or(Bar::Reloaded));
            }
            Some(DiskAction::Conflict) => return Err(Bar::ChangedOnDisk),
            Some(DiskAction::Deleted) => return Err(Bar::DeletedOnDisk),
            Some(DiskAction::Unchanged) | None => {}
        }
        self.write_out(SaveMode::Ask, cx).map(|_| ())
    }

    fn show_failure(&mut self, reason: String, cx: &mut Context<Self>) {
        self.bar = Some(Bar::Failed(reason));
        cx.notify();
    }

    fn render_header(&self, runs_in: Option<String>, theme: Theme, cx: &mut Context<Self>) -> Div {
        let runs_in = runs_in.map(|label| format!(" · runs in ⎇ {label}")).unwrap_or_default();
        let path = tilde_home(&self.target.store.path(&self.target.scratch.name));
        let text = format!("scratch of {} · {path}{runs_in}", self.target.project_name);
        let sizes = &theme.sizes.editor;
        div()
            .flex()
            .flex_none()
            .items_center()
            .gap(sizes.scratch_header_gap.px())
            .px(sizes.scratch_strip_padding_x.px())
            .py(sizes.scratch_strip_padding_y.px())
            .text_size(sizes.scratch_strip_text_size.px())
            .text_color(rgb(theme.muted))
            .border_b(sizes.scratch_header_border.px())
            .border_color(rgb(theme.border))
            .child(div().flex_1().min_w_0().overflow_hidden().text_ellipsis().whitespace_nowrap().child(text))
            .when(run::shell_for(&self.target.scratch.name).is_some(), |header| header.child(run_button(theme, cx)))
    }

    fn render_bar(&self, theme: Theme, cx: &mut Context<Self>) -> Option<Div> {
        let bar = self.bar.as_ref()?;
        let sizes = &theme.sizes.editor;
        let row = div()
            .flex()
            .flex_none()
            .items_center()
            .gap(sizes.scratch_bar_gap.px())
            .px(sizes.scratch_strip_padding_x.px())
            .py(sizes.scratch_strip_padding_y.px())
            .text_size(sizes.scratch_strip_text_size.px())
            .bg(rgb(theme.error))
            .text_color(rgb(theme.on_accent));
        Some(match bar {
            Bar::Failed(reason) | Bar::Refused(reason) => row.child(reason.clone()),
            Bar::Reloaded => row.child("changed on disk and reloaded; Run again to run it"),
            Bar::ChangedOnDisk => row
                .child("changed on disk")
                .child(bar_button("scratch-reload", "Reload", cx.listener(|view, _: &ClickEvent, _window, cx| view.reload(cx)), cx))
                .child(bar_button(
                    "scratch-overwrite",
                    "Overwrite",
                    cx.listener(|view, _: &ClickEvent, _window, cx| view.save_with(SaveMode::Overwrite, cx)),
                    cx,
                )),
            Bar::DeletedOnDisk => row
                .child("deleted on disk")
                .child(bar_button(
                    "scratch-save-again",
                    "Save again",
                    cx.listener(|view, _: &ClickEvent, _window, cx| view.save_with(SaveMode::Always, cx)),
                    cx,
                ))
                .child(bar_button("scratch-close", "Close", cx.listener(|view, _: &ClickEvent, window, cx| view.close_unsaved(window, cx)), cx)),
        })
    }

    fn render_body(&self, theme: Theme, window: &mut Window, cx: &App) -> AnyElement {
        let sizes = &theme.sizes.editor;
        let note = |note: &'static str| {
            div()
                .px(sizes.scratch_strip_padding_x.px())
                .py(sizes.scratch_strip_padding_y.px())
                .text_size(sizes.scratch_strip_text_size.px())
                .text_color(rgb(theme.muted))
                .child(note)
        };
        match &self.body {
            Body::Editing { editor, .. } => editor.clone().into_any_element(),
            Body::ReadOnly { document: Ok(document), note: text } => {
                div().flex().flex_col().flex_1().min_h_0().child(note(text)).child(document.render(theme, window, cx)).into_any_element()
            }
            Body::ReadOnly { document: Err(error), note: text } => div()
                .flex()
                .flex_col()
                .child(note(text))
                .child(div().p(sizes.scratch_error_padding.px()).text_color(rgb(theme.error)).child(error.clone()))
                .into_any_element(),
        }
    }
}

impl Render for ScratchView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = crate::theme::tabs(cx);
        // child views render after the workspace's own render returned, so reading it here is safe
        let runs_in = self.target.workspace.upgrade().and_then(|workspace| workspace.read(cx).scratch_run_label(&self.target.scratch.project));
        div()
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .child(self.render_header(runs_in, theme, cx))
            .children(self.render_bar(theme, cx))
            .child(self.render_body(theme, window, cx))
    }
}

fn limit(ending: LineEnding) -> SizeLimit {
    SizeLimit { max_bytes: MAX_SCRATCH_BYTES as usize, newline_bytes: ending.newline_bytes() }
}

fn read_only(target: &ScratchTarget, note: &'static str) -> Body {
    let document = Document::open(&target.store.path(&target.scratch.name)).map(|document| Box::new(EditorView::new(document))).map_err(|error| error.to_string());
    Body::ReadOnly { document, note }
}

/// Click target `run-scratch`.
fn run_button(theme: Theme, cx: &mut Context<ScratchView>) -> Stateful<Div> {
    let run = cx.listener(|view, _: &ClickEvent, window, cx| view.run(window, cx));
    let sizes = &theme.sizes.editor;
    div()
        .id("run-scratch")
        .flex()
        .flex_none()
        .items_center()
        .gap(sizes.scratch_button_gap.px())
        .px(sizes.scratch_button_padding_x.px())
        .rounded(sizes.scratch_button_radius.px())
        .cursor_pointer()
        .text_color(rgb(theme.text))
        .hover(|style| style.bg(rgb(theme.hover)))
        .child(img(icons::embedded(theme.appearance, icons::RUN)).size(theme.sizes.icon.px()))
        .child("Run")
        .on_click(target("run-scratch".to_owned(), run, cx))
}

/// A button in the bar; `name` is its element id and its click target.
fn bar_button(
    name: &'static str,
    label: &'static str,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    cx: &App,
) -> Stateful<Div> {
    let theme = cx.global::<Theme>();
    let sizes = &theme.sizes.editor;
    div()
        .id(SharedString::from(name))
        .px(sizes.scratch_button_padding_x.px())
        .rounded(sizes.scratch_button_radius.px())
        .border(sizes.scratch_button_border.px())
        .border_color(rgb(theme.on_accent))
        .cursor_pointer()
        .child(label)
        .on_click(target(name.to_owned(), on_click, cx))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stamp(content: &str) -> FileStamp {
        FileStamp::of_bytes(content.as_bytes())
    }

    #[test]
    fn should_write_only_when_there_are_unsaved_edits() {
        // given
        let state = SaveState::new(3, stamp("saved"));

        // when
        let actions = [3, 4].map(|revision| state.action(revision, Some(stamp("saved")), SaveMode::Ask));

        // then
        assert_eq!(actions, [SaveAction::Nothing, SaveAction::Write]);
    }

    #[test]
    fn should_ask_before_writing_over_a_change_or_a_removal_unless_overwriting() {
        // given
        let state = SaveState::new(3, stamp("saved"));

        // when
        let actions = [
            state.action(4, Some(stamp("theirs")), SaveMode::Ask),
            state.action(4, Some(stamp("theirs")), SaveMode::Overwrite),
            state.action(4, None, SaveMode::Ask),
            state.action(4, None, SaveMode::Overwrite),
        ];

        // then
        assert_eq!(actions, [SaveAction::Conflict, SaveAction::Write, SaveAction::Deleted, SaveAction::Write]);
    }

    #[test]
    fn should_write_a_removed_scratch_again_only_when_asked_to() {
        // given
        let state = SaveState::new(3, stamp("saved"));

        // when
        let actions = [state.action(3, None, SaveMode::Overwrite), state.action(3, None, SaveMode::Always)];

        // then
        assert_eq!(actions, [SaveAction::Nothing, SaveAction::Write]);
    }

    #[test]
    fn should_ignore_its_own_save_and_reload_or_ask_on_other_changes() {
        // given
        let state = SaveState::new(3, stamp("saved"));

        // when
        let actions = [
            state.on_disk_change(4, Some(stamp("saved"))),
            state.on_disk_change(3, Some(stamp("theirs"))),
            state.on_disk_change(4, Some(stamp("theirs"))),
            state.on_disk_change(3, None),
        ];

        // then
        assert_eq!(actions, [DiskAction::Unchanged, DiskAction::Reload, DiskAction::Conflict, DiskAction::Deleted]);
    }

    #[test]
    fn should_never_write_a_forgotten_scratch() {
        // given
        let mut state = SaveState::new(3, stamp("saved"));

        // when
        state.forget();

        // then
        assert_eq!(state.action(9, None, SaveMode::Always), SaveAction::Nothing);
    }

    #[test]
    fn should_count_a_save_as_the_new_baseline() {
        // given
        let mut state = SaveState::new(3, stamp("saved"));

        // when
        state.saved(5, stamp("newer"));

        // then
        assert_eq!(state.action(5, Some(stamp("newer")), SaveMode::Ask), SaveAction::Nothing);
    }
}
