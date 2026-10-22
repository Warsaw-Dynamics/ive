//! A file's tab: the file edited in a panel, its edits written by `file_write` as the panel asks, and what IVE
//! knows of the file on disk.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use codeview::document::{self, Reread, Stamp};
use codeview::panel::{Cast, Content, EditPolicy, Edit, Event, Panel, SaveFailure, SaveKind, SaveRequest};
use gpui::{AnyElement, App, AppContext, Context, Entity, Subscription, div, prelude::*};

use crate::codeview_host;
use crate::disk_text::{Disk, failure_note};
use crate::document::Document;
use crate::file_write::{self, Expected, Target, WriteError};
use crate::syntax::Language;
use crate::theme::Theme;

/// A file's tab: the file's path, which stays as the tab opened, its panel, and the file behind it.
pub struct FileTab {
    path: PathBuf,
    pub panel: Entity<Panel>,
    file: Entity<EditedFile>,
}

impl FileTab {
    /// Opens `document` for editing.
    pub fn new(document: Document, cx: &mut App) -> Self {
        let path = document.path().to_path_buf();
        let panel = cx.new(Panel::new);
        codeview_host::serve(&panel, path.display().to_string(), cx);
        let file = cx.new(|cx| EditedFile::new(document, panel.clone(), cx));
        Self { path, panel, file }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// What a re-read compares the file with.
    pub fn stamp(&self, cx: &App) -> Option<Stamp> {
        self.file.read(cx).disk.stamp()
    }

    pub fn take_reread(&mut self, reread: Reread, cx: &mut App) {
        self.file.update(cx, |file, cx| file.take_reread(reread, cx));
    }

    pub fn cast(&self, cast: Cast, cx: &mut App) {
        self.panel.update(cx, |panel, cx| panel.handle(cast, cx));
    }

    pub fn file_name(&self) -> String {
        self.path.file_name().map_or_else(|| self.path.display().to_string(), |name| name.to_string_lossy().into_owned())
    }

    /// The panel under the note about a failed re-read, if there is one.
    pub fn render(&self, theme: Theme, cx: &App) -> AnyElement {
        let Some(failure) = self.file.read(cx).disk.failure().cloned() else {
            return self.panel.clone().into_any_element();
        };
        div().flex().flex_col().flex_1().min_h_0().child(failure_note(failure, theme)).child(self.panel.clone()).into_any_element()
    }
}

/// The file behind a file tab's panel: the text the panel's edits apply to, and its saves.
struct EditedFile {
    path: PathBuf,
    panel: Entity<Panel>,
    /// the panel's base: the file as last read or written
    base: Arc<Document>,
    disk: Disk,
    writing: Writing,
    _subscriptions: Vec<Subscription>,
}

/// Whether IVE is writing the file: its own change on disk is no reason to reload.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Writing {
    Idle,
    Busy,
}

/// A save to run off the UI thread.
struct WriteJob {
    path: PathBuf,
    base: Arc<Document>,
    expected: Expected,
    edits: Vec<Edit>,
}

/// What a save found.
enum Written {
    /// the file holds the edits, read again as `document`
    Saved { stamp: Stamp, document: Document },
    /// the file holds the edits but cannot be read again
    Unreadable { stamp: Stamp, reason: String },
    Failed(SaveFailure),
}

impl EditedFile {
    fn new(document: Document, panel: Entity<Panel>, cx: &mut Context<Self>) -> Self {
        let disk = document::stamp(document.path()).map_or(Disk::Unstamped, Disk::Read);
        let base = Arc::new(document);
        panel.update(cx, |panel, cx| panel.handle(Cast::Open(edited(&base)), cx));
        let subscriptions = vec![
            cx.subscribe(&panel, |file, _panel, event: &Event, cx| file.on_panel_event(event, cx)),
            // closing the tab and quitting release the file: the last chance to save
            cx.on_release(|file, cx| file.flush(cx)),
        ];
        Self { path: base.path().to_path_buf(), panel, base, disk, writing: Writing::Idle, _subscriptions: subscriptions }
    }

    fn on_panel_event(&mut self, event: &Event, cx: &mut Context<Self>) {
        match event {
            Event::SaveRequested(request) => self.save(request.clone(), cx),
            Event::ReloadRequested => self.reload(cx),
            // parsing is `codeview_host::serve`'s; scroll and caret are saved with the tabs on redraw
            Event::Edited { .. }
            | Event::Refused(_)
            | Event::StateChanged
            | Event::LayoutToggled(_)
            | Event::NeedParse(_)
            | Event::NeedTexts { .. }
            | Event::OpenAt { .. } => {}
        }
    }

    /// What the file must be for a save of `kind`; `None` when IVE cannot tell what it is.
    fn expected(&self, kind: SaveKind) -> Option<Expected> {
        match kind {
            SaveKind::Checked => self.disk.stamp().map(Expected::Stamp),
            SaveKind::Overwrite => Some(Expected::Anything),
        }
    }

    fn job(&self, request: SaveRequest) -> Option<WriteJob> {
        let expected = self.expected(request.kind)?;
        Some(WriteJob { path: self.path.clone(), base: self.base.clone(), expected, edits: request.edits })
    }

    /// Writes the edits on the background executor and answers the panel.
    fn save(&mut self, request: SaveRequest, cx: &mut Context<Self>) {
        let revision = request.revision;
        let Some(job) = self.job(request) else {
            return self.answer(Cast::SaveFailed { revision, reason: SaveFailure::Conflict }, cx);
        };
        self.writing = Writing::Busy;
        let written = cx.background_executor().spawn(async move { job.run() });
        cx.spawn(async move |file, cx| {
            let written = written.await;
            file.update(cx, |file, cx| file.written(revision, written, cx)).ok();
        })
        .detach();
    }

    fn written(&mut self, revision: u64, written: Written, cx: &mut Context<Self>) {
        self.writing = Writing::Idle;
        let answer = match written {
            Written::Saved { stamp, document } => {
                self.base = Arc::new(document);
                self.disk = Disk::Read(stamp);
                self.answer(Cast::SetPolicy(policy(&self.base)), cx);
                Cast::Saved { revision, source: self.base.clone() }
            }
            Written::Unreadable { stamp, reason } => {
                self.disk = Disk::Read(stamp);
                Cast::SaveFailed { revision, reason: SaveFailure::Io(format!("saved, but cannot read it again: {reason}")) }
            }
            Written::Failed(reason) => Cast::SaveFailed { revision, reason },
        };
        self.answer(answer, cx);
    }

    fn answer(&self, cast: Cast, cx: &mut App) {
        self.panel.update(cx, |panel, cx| panel.handle(cast, cx));
    }

    /// When the tab goes: writes what the panel has not saved yet, unless the file changed on disk. Edits typed
    /// while a save was in flight are lost here; the window's flushes make that rare.
    fn flush(&mut self, cx: &mut App) {
        let Some(request) = self.panel.update(cx, |panel, cx| panel.take_save(SaveKind::Checked, cx)) else {
            return;
        };
        let Some(job) = self.job(request) else {
            return eprintln!("ive: not saved, {} is not as IVE last read it", self.path.display());
        };
        cx.background_executor()
            .spawn(async move {
                if let Written::Failed(reason) = job.run() {
                    eprintln!("ive: not saved, {}: {reason:?}", job.path.display());
                }
            })
            .detach();
    }

    /// Reload on a conflict: the file as it is now replaces the edits.
    fn reload(&mut self, cx: &mut Context<Self>) {
        let path = self.path.clone();
        let reread = cx.background_executor().spawn(async move { document::reread(&path, None) });
        cx.spawn(async move |file, cx| {
            let reread = reread.await;
            file.update(cx, |file, cx| file.show(reread, cx)).ok();
        })
        .detach();
    }

    /// A change on disk: the panel shows it unless it holds edits, which the next save checks against the file.
    /// IVE's own writes, seen late, change nothing.
    fn take_reread(&mut self, reread: Reread, cx: &mut Context<Self>) {
        let own = matches!(&reread, Reread::Changed(_, Some(stamp)) if Some(*stamp) == self.disk.stamp());
        let editing = self.writing == Writing::Busy || self.panel.read(cx).has_unsaved(cx);
        if own || editing {
            return;
        }
        self.show(reread, cx);
    }

    /// Shows what a re-read found: new text to edit, a file gone (shown as last read, read-only), or why it could
    /// not be read.
    fn show(&mut self, reread: Reread, cx: &mut Context<Self>) {
        let Some(disk) = Disk::after(&reread) else {
            return;
        };
        self.disk = disk;
        match reread {
            Reread::Changed(document, _) => {
                self.base = Arc::new(document);
                self.answer(Cast::Open(edited(&self.base)), cx);
            }
            Reread::Gone => {
                self.answer(Cast::Open(viewed(&self.base)), cx);
                self.answer(Cast::MarkDeleted, cx);
            }
            Reread::Unchanged | Reread::Failed(_) => {}
        }
        cx.notify();
    }
}

impl WriteJob {
    /// Writes the edits, then reads the file again: its line index may have moved even after an in-place write.
    fn run(&self) -> Written {
        let target = Target { path: &self.path, base: self.base.bytes(), expected: self.expected };
        let written = match file_write::apply_edits(&target, &self.edits) {
            Ok(written) => written,
            Err(WriteError::Conflict) => return Written::Failed(SaveFailure::Conflict),
            Err(error) => return Written::Failed(SaveFailure::Io(error.to_string())),
        };
        match Document::open(&self.path) {
            Ok(document) => Written::Saved { stamp: written.stamp, document },
            Err(error) => Written::Unreadable { stamp: written.stamp, reason: error.to_string() },
        }
    }
}

/// How the panel saves `document`: after a pause that grows with its size.
fn policy(document: &Document) -> EditPolicy {
    EditPolicy { save_delay: file_write::save_delay(document.byte_len() as u64), limit: None }
}

/// `document` to edit, coloured as its name says.
fn edited(document: &Arc<Document>) -> Content {
    let language = Language::for_path(document.path(), document.bytes());
    Content::Lines { source: document.clone(), language, edit: Some(policy(document)) }
}

/// `document` to view only: a file gone from disk.
fn viewed(document: &Arc<Document>) -> Content {
    let language = Language::for_path(document.path(), document.bytes());
    Content::Lines { source: document.clone(), language, edit: None }
}
