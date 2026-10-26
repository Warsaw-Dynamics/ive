//! The scratch tab: where the scratch lives and runs, its editor, and saving.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use codeview::document::{self, Reread, TextSource};
use codeview::panel::{Cast, Content, Edit, EditPolicy, Event, Panel, PanelState, Resolve, SaveFailure, SaveKind, SaveRequest};
use codeview::text_editor::SizeLimit;
use gpui::{
    AnyElement, App, ClickEvent, Context, Div, Entity, FocusHandle, Focusable, SharedString, Stateful, Subscription, Task,
    WeakEntity, Window, div, img, prelude::*, rgb,
};

use super::store::{FileStamp, LineEnding, MAX_SCRATCH_BYTES, ScratchContent, ScratchError, ScratchStore};
use super::{ScratchRef, log_error, run};
use crate::click_targets::target;
use crate::selectable::selectable;
use crate::codeview_host;
use crate::disk_text::DiskText;
use crate::document::Document;
use crate::icons;
use crate::paths::tilde_home;
use crate::project::ProjectId;
use crate::syntax::Language;
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

    pub fn panel_state(&self, cx: &App) -> PanelState {
        self.view.read(cx).panel_state(cx)
    }

    /// Puts the scroll and caret back where a saved tab had them.
    pub fn restore(&self, state: PanelState, cx: &mut App) {
        self.view.update(cx, |view, cx| view.restore(state, cx));
    }

    /// The panel editing the scratch; `None` while it is read-only.
    pub fn editing_panel(&self, cx: &App) -> Option<Entity<Panel>> {
        self.view.read(cx).editor()
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

    /// For MCP `send_text`: the text goes to the editor as typed; one over the size limit is refused in the tab.
    pub fn type_text(&self, text: &str, cx: &mut App) -> Result<(), String> {
        let editor = self.view.read(cx).editor().ok_or_else(|| format!("scratch {} is read-only", self.scratch.name))?;
        editor.update(cx, |editor, cx| editor.handle(Cast::Type(text.to_owned()), cx));
        Ok(())
    }
}

enum Body {
    /// `text` is what the panel's base holds: the scratch as last read or saved, with LF line breaks
    Editing { editor: Entity<Panel>, ending: LineEnding, save: SaveState, text: String },
    ReadOnly { panel: Entity<Panel>, disk: DiskText, note: &'static str },
    /// read-only, and the file could not be read either
    Unreadable { error: String, note: &'static str },
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

pub struct ScratchView {
    target: ScratchTarget,
    body: Body,
    bar: Option<Bar>,
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
                let editor = cx.new(Panel::new);
                codeview_host::serve(&editor, target.store.path(&target.scratch.name).display().to_string(), cx);
                subscriptions.push(cx.subscribe(&editor, |view, _editor, event: &Event, cx| view.on_panel_event(event, cx)));
                editor.update(cx, |editor, cx| editor.handle(Cast::Open(edited(&target.scratch.name, &text, ending)), cx));
                let save = SaveState::new(editor.read(cx).revision(), stamp);
                Body::Editing { editor, ending, save, text }
            }
            ScratchContent::TooLarge => read_only(&target, TOO_LARGE, cx),
            ScratchContent::NotUtf8 => read_only(&target, NOT_UTF8, cx),
        };
        Self { target, body, bar: None, read_task: None, _subscriptions: subscriptions }
    }

    fn restore(&self, state: PanelState, cx: &mut Context<Self>) {
        match &self.body {
            Body::Editing { editor: panel, .. } | Body::ReadOnly { panel, .. } => panel.update(cx, |panel, cx| panel.handle(Cast::Restore(state), cx)),
            Body::Unreadable { .. } => {}
        }
    }

    fn schedule_workspace_save(&self, cx: &mut Context<Self>) {
        self.target.workspace.update(cx, |workspace, cx| workspace.schedule_save(cx)).ok();
    }

    /// Scroll, and the caret while editing, to save.
    fn panel_state(&self, cx: &App) -> PanelState {
        match &self.body {
            Body::Editing { editor: panel, .. } | Body::ReadOnly { panel, .. } => panel.read(cx).state(),
            Body::Unreadable { .. } => PanelState::default(),
        }
    }

    fn editor(&self) -> Option<Entity<Panel>> {
        match &self.body {
            Body::Editing { editor, .. } => Some(editor.clone()),
            Body::ReadOnly { .. } | Body::Unreadable { .. } => None,
        }
    }

    fn on_panel_event(&mut self, event: &Event, cx: &mut Context<Self>) {
        match event {
            Event::Edited { .. } => {
                self.bar.take_if(|bar| bar.is_passing());
            }
            Event::Refused(reason) => self.bar = Some(Bar::Refused(reason.clone())),
            // caret and scroll are saved with the open tabs
            Event::StateChanged => return self.schedule_workspace_save(cx),
            Event::SaveRequested(request) => self.write_request(request.clone(), mode_of(request.kind), cx),
            // the tab stays open after closing without saving: the edits given up are neither shown nor written
            Event::ReloadRequested | Event::DiscardRequested => self.reload(cx),
            // a scratch has no change markers: it gets no base to compare with
            Event::NeedChangeBase | Event::NeedChangeDiff(_) | Event::CommitChange { .. } => return,
            // parsing is `codeview_host::serve`'s
            // the Markdown view is remembered by `codeview_host`
            Event::NeedParse(_) | Event::LayoutToggled(_) | Event::MarkdownViewToggled(_) | Event::NeedTexts(_) | Event::OpenAt { .. } | Event::OpenDiff { .. } => {
                return;
            }
        }
        cx.notify();
    }

    /// A bar's Overwrite or Save again.
    fn save_shown(&mut self, mode: SaveMode, cx: &mut Context<Self>) {
        self.save_with(mode, cx);
        cx.notify();
    }

    /// Writes what the panel has unsaved now, as `mode` allows; Save again also writes a scratch without edits.
    fn save_with(&mut self, mode: SaveMode, cx: &mut App) {
        match self.take_request(mode, cx) {
            Some(request) => self.write_request(request, mode, cx),
            None if mode == SaveMode::Always => drop(self.write_out(&self.unchanged(cx), mode)),
            None => {}
        }
    }

    /// The panel's unsaved edits, now instead of after its pause.
    fn take_request(&self, mode: SaveMode, cx: &mut App) -> Option<SaveRequest> {
        let Body::Editing { editor, .. } = &self.body else {
            return None;
        };
        let kind = match mode {
            SaveMode::Ask => SaveKind::Checked,
            SaveMode::Overwrite | SaveMode::Always => SaveKind::Overwrite,
        };
        editor.update(cx, |editor, cx| editor.take_save(kind, cx))
    }

    /// A save of the text as it is on disk already: for Save again without edits.
    fn unchanged(&self, cx: &App) -> SaveRequest {
        let revision = self.editor().map_or(0, |editor| editor.read(cx).revision());
        SaveRequest { base: revision, revision, edits: Vec::new(), kind: SaveKind::Overwrite }
    }

    /// Writes the edits of `request` and answers the panel.
    fn write_request(&mut self, request: SaveRequest, mode: SaveMode, cx: &mut App) {
        let answer = self.write_out(&request, mode);
        self.answer(answer, cx);
    }

    fn answer(&self, answer: Cast, cx: &mut App) {
        if let Body::Editing { editor, .. } = &self.body {
            editor.update(cx, |editor, cx| editor.handle(answer, cx));
        }
    }

    /// Writes the scratch's text with `request`'s edits; the answer to the panel, whose bar shows why a save
    /// failed. A scratch deleted on disk gets this tab's own bar too, which can write it again or close the tab.
    fn write_out(&mut self, request: &SaveRequest, mode: SaveMode) -> Cast {
        let failed = |reason: &str| Cast::SaveFailed { revision: request.revision, reason: SaveFailure::Io(reason.to_owned()) };
        let Body::Editing { save, .. } = &self.body else {
            return failed("the scratch is read-only");
        };
        let (store, name) = (&self.target.store, &self.target.scratch.name);
        match save.action(request.revision, store.stamp(name).ok(), mode) {
            SaveAction::Nothing => failed("the scratch is being deleted"),
            SaveAction::Conflict => {
                // the panel's bar asks now
                self.bar.take_if(|bar| matches!(bar, Bar::ChangedOnDisk));
                Cast::SaveFailed { revision: request.revision, reason: SaveFailure::Conflict(Resolve::ReloadOrOverwrite) }
            }
            SaveAction::Deleted => {
                self.bar = Some(Bar::DeletedOnDisk);
                failed("deleted on disk")
            }
            SaveAction::Write => self.write_edits(request),
        }
    }

    fn write_edits(&mut self, request: &SaveRequest) -> Cast {
        let Body::Editing { ending, save, text, .. } = &mut self.body else {
            return Cast::SaveFailed { revision: request.revision, reason: SaveFailure::Io("the scratch is read-only".to_owned()) };
        };
        let new_text = spliced(text, &request.edits);
        match self.target.store.write(&self.target.scratch.name, &new_text, *ending).inspect_err(log_error) {
            Ok(stamp) => {
                save.saved(request.revision, stamp);
                *text = new_text;
                // a bar about the file stays until a write settles it
                self.bar.take_if(|bar| !matches!(bar, Bar::Refused(_)));
                Cast::Saved { revision: request.revision, source: Arc::new(TextSource::new(text)) }
            }
            Err(error) => Cast::SaveFailed { revision: request.revision, reason: SaveFailure::Io(error.to_string()) },
        }
    }

    /// On release there is no bar left to show: write errors go to stderr, and the user's edits win over a
    /// change on disk.
    fn flush(&mut self, cx: &mut App) {
        self.save_with(SaveMode::Overwrite, cx);
    }

    /// Drops unsaved edits and shows the file as it is on disk.
    fn reload(&mut self, cx: &mut Context<Self>) {
        let read = self.target.store.read(&self.target.scratch.name);
        self.bar = match (read, &mut self.body) {
            (Ok(ScratchContent::Editable { text, ending, stamp }), Body::Editing { editor, ending: current, save, text: base }) => {
                let content = edited(&self.target.scratch.name, &text, ending);
                editor.update(cx, |editor, cx| editor.handle(Cast::Open(content), cx));
                *current = ending;
                *save = SaveState::new(editor.read(cx).revision(), stamp);
                *base = text;
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
        let Body::ReadOnly { disk, .. } = &self.body else {
            return;
        };
        let (path, known) = (self.target.store.path(&self.target.scratch.name), disk.stamp());
        self.read_task = Some(cx.spawn(async move |view, cx| {
            let reread = cx.background_executor().spawn(async move { document::reread(&path, known) }).await;
            view.update(cx, |view, cx| view.take_reread(reread, cx)).ok();
        }));
    }

    fn take_reread(&mut self, reread: Reread, cx: &mut Context<Self>) {
        if let Body::ReadOnly { panel, disk, .. } = &mut self.body {
            disk.take(reread, panel, cx);
        }
        cx.notify();
    }

    /// Close on the "deleted on disk" bar: the tab goes without writing the file back.
    fn close_unsaved(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.forget();
        let scratch = self.target.scratch.clone();
        self.target.workspace.update(cx, |workspace, cx| workspace.close_scratch(&scratch, window, cx)).ok();
    }

    fn forget(&mut self) {
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
        let Some(request) = self.take_request(SaveMode::Ask, cx) else {
            return Ok(());
        };
        let answer = self.write_out(&request, SaveMode::Ask);
        let saved = matches!(answer, Cast::Saved { .. });
        self.answer(answer, cx);
        if saved {
            return Ok(());
        }
        Err(self.bar.take().unwrap_or(Bar::Failed("not run: the scratch could not be saved".to_owned())))
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
            .child(div().flex_1().min_w_0().overflow_hidden().text_ellipsis().whitespace_nowrap().child(selectable(format!("scratch-header-{}", self.target.scratch.name), text, cx)))
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
                    cx.listener(|view, _: &ClickEvent, _window, cx| view.save_shown(SaveMode::Overwrite, cx)),
                    cx,
                )),
            Bar::DeletedOnDisk => row
                .child("deleted on disk")
                .child(bar_button(
                    "scratch-save-again",
                    "Save again",
                    cx.listener(|view, _: &ClickEvent, _window, cx| view.save_shown(SaveMode::Always, cx)),
                    cx,
                ))
                .child(bar_button("scratch-close", "Close", cx.listener(|view, _: &ClickEvent, window, cx| view.close_unsaved(window, cx)), cx)),
        })
    }

    fn render_body(&self, theme: Theme) -> AnyElement {
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
            Body::ReadOnly { panel, disk, note: text } => {
                div().flex().flex_col().flex_1().min_h_0().child(note(text)).child(disk.render(panel, theme)).into_any_element()
            }
            Body::Unreadable { error, note: text } => div()
                .flex()
                .flex_col()
                .child(note(text))
                .child(div().p(sizes.scratch_error_padding.px()).text_color(rgb(theme.error)).child(error.clone()))
                .into_any_element(),
        }
    }
}

impl Render for ScratchView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
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
            .child(self.render_body(theme))
    }
}

/// How a save the panel asked for treats a change on disk.
fn mode_of(kind: SaveKind) -> SaveMode {
    match kind {
        SaveKind::Checked => SaveMode::Ask,
        SaveKind::Overwrite => SaveMode::Overwrite,
    }
}

/// `text` with `edits` (sorted, in its bytes) applied: a scratch is small enough to splice in memory.
fn spliced(text: &str, edits: &[Edit]) -> String {
    let mut bytes = Vec::with_capacity(text.len());
    let mut kept_from = 0;
    for edit in edits {
        bytes.extend_from_slice(&text.as_bytes()[kept_from..edit.bytes.start as usize]);
        bytes.extend_from_slice(&edit.text);
        kept_from = edit.bytes.end as usize;
    }
    bytes.extend_from_slice(&text.as_bytes()[kept_from..]);
    // edits typed into UTF-8 text are UTF-8
    String::from_utf8(bytes).unwrap_or_else(|error| String::from_utf8_lossy(error.as_bytes()).into_owned())
}

fn limit(ending: LineEnding) -> SizeLimit {
    SizeLimit { max_bytes: MAX_SCRATCH_BYTES as usize, newline_bytes: ending.newline_bytes() }
}

/// The text to edit, coloured as the scratch's name says.
fn edited(name: &str, text: &str, ending: LineEnding) -> Content {
    let policy = EditPolicy { save_delay: SAVE_DELAY, limit: Some(limit(ending)) };
    let language = Language::for_path(Path::new(name), text.as_bytes());
    Content::Lines { source: Arc::new(TextSource::new(text)), language, edit: Some(policy) }
}

/// The file in a view that only shows it: the mouse selects no text in a read-only scratch.
fn read_only(target: &ScratchTarget, note: &'static str, cx: &mut App) -> Body {
    let path = target.store.path(&target.scratch.name);
    match Document::open(&path) {
        Ok(document) => {
            let panel = cx.new(|cx| Panel::new(cx).unselectable());
            codeview_host::serve(&panel, path.display().to_string(), cx);
            let disk = DiskText::show(document, &panel, cx);
            Body::ReadOnly { panel, disk, note }
        }
        Err(error) => Body::Unreadable { error: error.to_string(), note },
    }
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
    use codeview::host::Command;

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
    fn should_splice_edits_into_the_scratch_text() {
        // given
        let text = "one\ntwo\nthree\n";
        let edits = [Edit { bytes: 0..3, text: b"ONE".to_vec() }, Edit { bytes: 8..8, text: "é\n".as_bytes().to_vec() }];

        // when
        let new_text = spliced(text, &edits);

        // then
        assert_eq!(new_text, "ONE\ntwo\né\nthree\n");
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

    /// An editing scratch view of scratch `name` holding `text`, in a store under `dir`.
    fn scratch_view(dir: &Path, text: &str, cx: &mut gpui::TestAppContext) -> (ScratchStore, Entity<ScratchView>) {
        let store = ScratchStore::new(dir, "p").unwrap();
        store.ensure_dir().unwrap();
        store.write("notes", text, LineEnding::Lf).unwrap();
        let content = store.read("notes").unwrap();
        let scratch = ScratchRef { project: ProjectId::new("p"), name: "notes".to_owned() };
        let target = ScratchTarget { scratch, project_name: "p".to_owned(), store: store.clone(), workspace: WeakEntity::new_invalid() };
        let view = cx.new(|cx| ScratchView::new(target, content, cx));
        (store, view)
    }

    #[gpui::test]
    fn should_reload_a_tab_that_stays_open_after_closing_without_saving(cx: &mut gpui::TestAppContext) {
        // given
        let dir = tempfile::tempdir().unwrap();
        let (store, view) = scratch_view(dir.path(), "mine\n", cx);
        let panel = view.read_with(cx, |view, _| view.editor().unwrap());
        let cast = |cast: Cast, cx: &mut gpui::TestAppContext| panel.update(cx, |panel, cx| panel.handle(cast, cx));
        cast(Cast::Type("lost ".to_owned()), cx);
        // another program's change: the save meets a conflict, and a quit waits for the edits
        std::fs::write(store.path("notes"), "theirs\n").unwrap();
        cast(Cast::Command(Command::Flush), cx);
        cast(Cast::CloseAsked, cx);

        // when
        // as the panel does on the bar's "Close without saving"
        panel.update(cx, |_, cx| cx.emit(Event::DiscardRequested));
        let unsaved = panel.read_with(cx, |panel, cx| panel.has_unsaved(cx));
        cast(Cast::Type("kept ".to_owned()), cx);
        cast(Cast::Command(Command::Flush), cx);

        // then
        let written = std::fs::read_to_string(store.path("notes")).unwrap();
        // the caret stays where it was, in the text as reloaded
        assert_eq!((unsaved, written.contains("kept "), written.replace("kept ", "")), (false, true, "theirs\n".to_owned()));
    }
}
