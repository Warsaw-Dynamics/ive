//! A file's tab: the file edited in a panel, its edits written by `file_write` as the panel asks, and what IVE
//! knows of the file on disk. An uncommitted diff tab edits its file's new side through the same [`EditedFile`].
//!
//! Two panels may edit one file (its tab and a diff tab of it): while one holds unsaved edits, the others are
//! read-only and say why, and once its edits are saved they read the file again. So no panel ever writes the file
//! from a text another panel's save has changed.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use codeview::changes::ChangeBase;
use codeview::document::{self, DocumentError, Reread, Stamp};
use codeview::panel::{Cast, Content, EditPolicy, Edit, Editable, Event, Panel, Resolve, SaveFailure, SaveKind, SaveRequest};
use gpui::{
    AnyElement, App, AppContext, Context, Div, Entity, EntityId, EventEmitter, Global, SharedString, Subscription, WeakEntity, div, prelude::*,
    rgb,
};

use crate::codeview_host;
use crate::git;
use crate::disk_text::{Disk, failure_note};
use crate::document::Document;
use crate::file_write::{self, Expected, Target, WriteError};
use crate::syntax::Language;
use crate::theme::Theme;

/// Saves of file tabs running on the background executor; quitting waits for them.
static WRITES_RUNNING: AtomicUsize = AtomicUsize::new(0);

/// Whether a file tab's save is being written, also one of a tab that closed.
pub fn is_writing_any() -> bool {
    WRITES_RUNNING.load(Ordering::SeqCst) > 0
}

/// A file's tab: the file's path, which stays as the tab opened, its panel, and the file behind it.
pub struct FileTab {
    path: PathBuf,
    pub panel: Entity<Panel>,
    file: Entity<EditedFile>,
}

impl FileTab {
    /// Opens a document for editing, with the stamp its file had when it was read ([`Document::open_stamped`]).
    pub fn new(opened: (Document, Option<Stamp>), cx: &mut App) -> Self {
        let path = opened.0.path().to_path_buf();
        let panel = cx.new(Panel::new);
        codeview_host::serve(&panel, path.display().to_string(), cx);
        let file = cx.new(|cx| EditedFile::new(opened, Shown { panel: panel.clone(), shows: Shows::File }, cx));
        Self { path, panel, file }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// What a re-read compares the file with.
    pub fn stamp(&self, cx: &App) -> Option<Stamp> {
        self.file.read(cx).stamp()
    }

    pub fn take_reread(&mut self, reread: Reread, cx: &mut App) {
        self.file.update(cx, |file, cx| file.take_reread(reread, cx));
    }

    /// Something changed in a git dir: HEAD may have moved, so the change markers' base is read again.
    pub fn git_changed(&self, git_dir: &Path, cx: &mut App) {
        let git_dir = git_dir.to_path_buf();
        self.file.update(cx, |file, cx| file.send_change_base(Some(git_dir), cx));
    }

    pub fn cast(&self, cast: Cast, cx: &mut App) {
        self.panel.update(cx, |panel, cx| panel.handle(cast, cx));
    }

    /// MCP `send_text`: types `text` at the caret; an error while the tab is read-only.
    pub fn type_text(&self, text: &str, cx: &mut App) -> Result<(), String> {
        if self.panel.read(cx).takes_typing(cx) {
            self.cast(Cast::Type(text.to_owned()), cx);
            return Ok(());
        }
        Err(format!("{} is read-only here: another tab holds unsaved edits to it, or it was deleted", self.path.display()))
    }

    pub fn file_name(&self) -> String {
        self.path.file_name().map_or_else(|| self.path.display().to_string(), |name| name.to_string_lossy().into_owned())
    }

    /// The panel under the note about a failed re-read, or about why the file is read-only, if there is one.
    pub fn render(&self, theme: Theme, cx: &App) -> AnyElement {
        let file = self.file.read(cx);
        let note = match (file.disk.failure().cloned(), file.lock_note()) {
            (Some(failure), _) => failure_note(failure, theme),
            (None, Some(lock)) => lock_note(lock, theme),
            (None, None) => return self.panel.clone().into_any_element(),
        };
        div().flex().flex_col().flex_1().min_h_0().child(note).child(self.panel.clone()).into_any_element()
    }
}

/// Why a file is read-only, above its text.
pub(crate) fn lock_note(note: SharedString, theme: Theme) -> Div {
    div().flex_none().px_2().py_1().bg(rgb(theme.sidebar)).text_color(rgb(theme.error)).child(note)
}

/// What shows the file: its panel, and how a new text reaches it.
pub(crate) struct Shown {
    pub(crate) panel: Entity<Panel>,
    pub(crate) shows: Shows,
}

/// How the panel shows the file.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Shows {
    /// a file tab: the whole text, opened in Edit mode
    File,
    /// a diff tab's new side: the panel asks for the text to edit (after [`Cast::SetEditable`]) and the tab reads it
    DiffSide,
}

/// What an edited file tells the tab holding it.
pub(crate) enum FileEvent {
    /// IVE saved the file: the base is the text saved
    Saved,
    /// the file went away on disk
    Gone,
}

/// Whether a panel holds edits not on disk yet.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Holding {
    Nothing,
    Edits,
}

/// Every file opened for editing, by its resolved path; see the module doc.
#[derive(Default)]
struct OpenFiles(HashMap<PathBuf, Vec<WeakEntity<EditedFile>>>);

impl Global for OpenFiles {}

/// The files open for editing at `key` other than `me`.
fn peers_of(key: &Path, me: EntityId, cx: &mut App) -> Vec<Entity<EditedFile>> {
    let open = cx.default_global::<OpenFiles>().0.entry(key.to_path_buf()).or_default();
    open.retain(|file| file.upgrade().is_some());
    open.iter().filter_map(WeakEntity::upgrade).filter(|file| file.entity_id() != me).collect()
}

/// The path all names of a file share: links and `..` resolved; the path itself when it cannot be resolved.
fn resolved(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

/// The file behind a file tab's panel, or behind a diff tab's new side: the text the panel's edits apply to, and its
/// saves.
///
/// `base` is the panel's base at all times: it changes only with an answer that moves the panel onto it (`Saved`,
/// `Open`, a diff's text to edit), never while a write is in flight. `disk` is `Disk::Read` only while the file holds
/// `base`, so a save checked against it writes at the right offsets.
pub(crate) struct EditedFile {
    /// this entity's, kept for telling the others editing the file when it goes
    id: EntityId,
    path: PathBuf,
    panel: Entity<Panel>,
    shows: Shows,
    /// `path` resolved: what the others editing the file are found by
    key: PathBuf,
    holding: Holding,
    /// the others editing the file that hold edits: while there are any, this one is read-only
    locked_by: Vec<EntityId>,
    /// the panel's base: the file as last read or written
    base: Arc<Document>,
    disk: Disk,
    writing: Writing,
    /// counts the changes of `base` and `disk`: a re-read started before the last change is stale
    version: u64,
    /// counts the reads of the change markers' base, numbering them
    change_reads: u64,
    /// what the change markers were last given and why a read failed, so a read with the same HEAD is skipped and a
    /// failure is logged once
    change_sent: ChangeSent,
    _subscriptions: Vec<Subscription>,
}

/// What the change markers of a file were last given, and the last failure to read it.
#[derive(Default)]
struct ChangeSent {
    /// the `head_id` of the base last cast; empty when there is none (no markers)
    head_id: String,
    /// the message of the failure last logged; empty after a read that worked
    failure: String,
    /// the number of the read whose answer was last taken: an earlier read's answer is stale, while a read that
    /// skipped (another repository, the same HEAD) leaves earlier reads in flight standing
    taken: u64,
}

impl ChangeSent {
    /// What to cast for the answer of read number `read`, if anything: a skipped read, an answer older than the one
    /// last taken and a failure already logged cast nothing.
    fn took(&mut self, read: u64, answer: Result<Option<ChangeBase>, git::repo::GitError>) -> Option<ChangeBase> {
        if read < self.taken {
            return None;
        }
        match answer {
            Ok(None) => None,
            Ok(Some(base)) => {
                self.taken = read;
                self.failure.clear();
                self.head_id = match &base {
                    ChangeBase::Tracked { head_id, .. } => head_id.clone(),
                    ChangeBase::Untracked => String::new(),
                };
                Some(base)
            }
            Err(error) => {
                self.taken = read;
                let message = error.to_string();
                self.head_id.clear();
                if message == self.failure {
                    return None;
                }
                eprintln!("ive: no change markers: {message}");
                self.failure = message;
                Some(ChangeBase::Untracked)
            }
        }
    }
}

/// Whether IVE is writing the file: its own change on disk is no reason to reload, and nothing replaces the base
/// meanwhile.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Writing {
    Idle,
    Busy(AfterWrite),
}

/// What follows the write in flight.
#[derive(Clone, Copy, PartialEq, Eq)]
enum AfterWrite {
    /// look whether the file changed while it was written: re-reads were set aside meanwhile
    Check,
    /// the user chose Reload meanwhile
    Reload,
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
    /// the edits were written, but the file is not known to hold them now: it cannot be read again, or another
    /// program changed it right after
    Unconfirmed(SaveFailure),
    Failed(SaveFailure),
}

/// Counts a write in [`WRITES_RUNNING`] until dropped.
struct Running;

impl Running {
    fn start() -> Self {
        WRITES_RUNNING.fetch_add(1, Ordering::SeqCst);
        Self
    }
}

impl Drop for Running {
    fn drop(&mut self) {
        WRITES_RUNNING.fetch_sub(1, Ordering::SeqCst);
    }
}

impl EventEmitter<FileEvent> for EditedFile {}

impl EditedFile {
    /// The file `opened` (with the stamp it had when read) edited in `shown`; read-only from the start while another
    /// panel holds edits to it.
    pub(crate) fn new(opened: (Document, Option<Stamp>), shown: Shown, cx: &mut Context<Self>) -> Self {
        let (document, stamp) = opened;
        let (base, Shown { panel, shows }) = (Arc::new(document), shown);
        let subscriptions = vec![
            cx.subscribe(&panel, |file, _panel, event: &Event, cx| file.on_panel_event(event, cx)),
            // the workspace closes a tab once its saves landed; anything else releasing it gets this last chance
            cx.on_release(|file, cx| {
                file.flush(cx);
                file.leave(cx);
            }),
        ];
        let (path, id) = (base.path().to_path_buf(), cx.entity_id());
        let key = resolved(&path);
        let holders = peers_of(&key, id, cx).into_iter().filter(|peer| peer.read(cx).holding == Holding::Edits);
        let locked_by = holders.map(|peer| peer.entity_id()).collect();
        let me = cx.weak_entity();
        cx.default_global::<OpenFiles>().0.entry(key.clone()).or_default().push(me);
        let disk = stamp.map_or(Disk::Unstamped, Disk::Read);
        let file = Self { id, path, panel, shows, key, holding: Holding::Nothing, locked_by, base, disk, writing: Writing::Idle, version: 0, change_reads: 0, change_sent: ChangeSent::default(), _subscriptions: subscriptions };
        file.show_first(cx);
        file
    }

    /// A file tab opens the text; a diff's new side, which its panel is about to take, only stops it when locked.
    fn show_first(&self, cx: &mut App) {
        match (self.shows, self.locked_by.is_empty()) {
            (Shows::File, _) | (Shows::DiffSide, false) => self.present(cx),
            (Shows::DiffSide, true) => {}
        }
    }

    /// A diff's new side, read again to edit: it becomes the base, checked against its stamp on the next save, and
    /// the panel takes the base returned. The panel waits for that text (it holds no edits, nothing is being
    /// written), so nothing is lost.
    pub(crate) fn adopt(&mut self, opened: (Document, Option<Stamp>), cx: &mut Context<Self>) -> Arc<Document> {
        debug_assert!(self.writing == Writing::Idle, "a diff's text replaced while it is being saved");
        let (document, stamp) = opened;
        self.base = Arc::new(document);
        self.disk = stamp.map_or(Disk::Unstamped, Disk::Read);
        self.version += 1;
        self.answer(Cast::SetPolicy(policy(&self.base)), cx);
        self.base.clone()
    }

    /// The panel's base: the file as last read or written.
    pub(crate) fn base(&self) -> Arc<Document> {
        self.base.clone()
    }

    /// What a re-read compares the file with.
    pub(crate) fn stamp(&self) -> Option<Stamp> {
        self.disk.stamp()
    }

    /// Why the file is read-only here: another panel holds edits to it.
    pub(crate) fn lock_note(&self) -> Option<SharedString> {
        if self.locked_by.is_empty() {
            return None;
        }
        let name = self.path.file_name().map_or_else(|| self.path.display().to_string(), |name| name.to_string_lossy().into_owned());
        Some(format!("read-only: {name} has unsaved edits in another tab").into())
    }

    /// Shows the base: editable, or read-only while another panel holds edits to the file. Either way the panel
    /// starts over from the base, so its undo history goes (it holds no unsaved edits when this runs).
    fn present(&self, cx: &mut App) {
        let cast = match (self.shows, self.locked_by.is_empty()) {
            (Shows::File, true) => Cast::Open(edited(&self.base)),
            (Shows::File, false) => Cast::Open(viewed(&self.base)),
            (Shows::DiffSide, true) => Cast::SetEditable(Editable::NewSide { policy: policy(&self.base) }),
            (Shows::DiffSide, false) => Cast::SetEditable(Editable::No),
        };
        self.answer(cast, cx);
    }

    /// Tells the others editing the file whether this one holds edits, when that changed.
    fn notice_holding(&mut self, cx: &mut Context<Self>) {
        let panel = self.panel.read(cx);
        let edits = self.writing != Writing::Idle || panel.has_unsaved(cx) || panel.is_saving();
        let holding = if edits { Holding::Edits } else { Holding::Nothing };
        if holding == self.holding {
            return;
        }
        self.holding = holding;
        self.tell_peers(holding, cx);
    }

    fn tell_peers(&self, holding: Holding, cx: &mut App) {
        for peer in peers_of(&self.key, self.id, cx) {
            peer.update(cx, |peer, cx| peer.peer_holding(self.id, holding, cx));
        }
    }

    /// `peer`, editing the same file, holds edits, or no longer does: this one is read-only meanwhile, and reads
    /// the file again after.
    fn peer_holding(&mut self, peer: EntityId, holding: Holding, cx: &mut Context<Self>) {
        let was_locked = !self.locked_by.is_empty();
        self.locked_by.retain(|id| *id != peer);
        if holding == Holding::Edits {
            self.locked_by.push(peer);
        }
        match (was_locked, self.locked_by.is_empty()) {
            (false, false) => self.present(cx),
            (true, true) => self.reload(cx),
            _ => {}
        }
        cx.notify();
    }

    /// The panel goes away: the others editing the file stop waiting for its edits.
    fn leave(&self, cx: &mut App) {
        if self.holding == Holding::Edits {
            self.tell_peers(Holding::Nothing, cx);
        }
    }

    fn on_panel_event(&mut self, event: &Event, cx: &mut Context<Self>) {
        match event {
            Event::SaveRequested(request) => self.save(request.clone(), cx),
            Event::ReloadRequested => self.reload(cx),
            Event::NeedChangeBase => self.send_change_base(None, cx),
            // the workspace closes the tab on a discard
            Event::Edited { .. } | Event::DiscardRequested => {}
            // parsing is `codeview_host::serve`'s; the workspace saves scroll and caret on StateChanged and opens the
            // diff on OpenDiff
            Event::Refused(_)
            | Event::StateChanged
            | Event::LayoutToggled(_)
            | Event::NeedParse(_)
            | Event::NeedTexts(_)
            | Event::NeedChangeDiff(_)
            | Event::OpenAt { .. }
            | Event::OpenDiff { .. } => {}
        }
        self.notice_holding(cx);
    }

    /// The write of `request`, or why there is none: the file is not known to hold the base (a conflict), or a
    /// mapped base may hold another program's bytes, so it cannot be written over the file.
    fn job(&self, request: SaveRequest) -> Result<WriteJob, SaveFailure> {
        if let Writing::Busy(_) = self.writing {
            return Err(SaveFailure::Io(format!("{} is still being saved", self.path.display())));
        }
        let expected = match (request.kind, self.disk.stamp()) {
            (SaveKind::Checked, Some(stamp)) => Expected::Stamp(stamp),
            (SaveKind::Checked, None) => return Err(conflict(&self.base)),
            (SaveKind::Overwrite, _) if self.base.is_mapped() => return Err(conflict(&self.base)),
            (SaveKind::Overwrite, _) => Expected::Anything,
        };
        Ok(WriteJob { path: self.path.clone(), base: self.base.clone(), expected, edits: request.edits })
    }

    /// Reads what the panel's change markers compare the file with (HEAD's text of it) off the UI thread and casts
    /// it; a file whose base cannot be read gets no markers, with the reason on stderr once. With `git_dir`, only a
    /// file of that git dir is read, and only when HEAD is not the one the markers were last given.
    pub(crate) fn send_change_base(&mut self, git_dir: Option<PathBuf>, cx: &mut Context<Self>) {
        self.change_reads += 1;
        let (path, read) = (self.path.clone(), self.change_reads);
        let known = if git_dir.is_some() { self.change_sent.head_id.clone() } else { String::new() };
        let reading = cx.background_executor().spawn(async move {
            git::diff::change_base_since(&path, &git::diff::BaseAsk { known: &known, git_dir: git_dir.as_deref() })
        });
        cx.spawn(async move |file, cx| {
            let read_base = reading.await;
            file.update(cx, |file, cx| {
                if let Some(base) = file.change_sent.took(read, read_base) {
                    file.answer(Cast::SetChangeBase { base }, cx);
                }
            })
            .ok();
        })
        .detach();
    }

    /// Writes the edits on the background executor and answers the panel.
    fn save(&mut self, request: SaveRequest, cx: &mut Context<Self>) {
        let revision = request.revision;
        let job = match self.job(request) {
            Ok(job) => job,
            Err(reason) => return self.answer(Cast::SaveFailed { revision, reason }, cx),
        };
        self.writing = Writing::Busy(AfterWrite::Check);
        let running = Running::start();
        let written = cx.background_executor().spawn(async move {
            let _running = running;
            job.run()
        });
        cx.spawn(async move |file, cx| {
            let written = written.await;
            file.update(cx, |file, cx| file.written(revision, written, cx)).ok();
        })
        .detach();
    }

    fn written(&mut self, revision: u64, written: Written, cx: &mut Context<Self>) {
        let after = match self.writing {
            Writing::Busy(after) => after,
            Writing::Idle => AfterWrite::Check,
        };
        self.writing = Writing::Idle;
        self.version += 1;
        let answer = self.take_written(revision, written, cx);
        let saved = matches!(answer, Cast::Saved { .. });
        self.answer(answer, cx);
        // after the panel took the saved text as its base, so the tab's diff of it reaches the rows
        if saved {
            cx.emit(FileEvent::Saved);
        }
        self.notice_holding(cx);
        match after {
            AfterWrite::Check => self.check_disk(cx),
            AfterWrite::Reload => self.reload(cx),
        }
    }

    /// The answer to the save of `revision`: a saved file becomes the base.
    fn take_written(&mut self, revision: u64, written: Written, cx: &mut Context<Self>) -> Cast {
        match written {
            Written::Saved { stamp, document } => {
                self.base = Arc::new(document);
                self.disk = Disk::Read(stamp);
                self.answer(Cast::SetPolicy(policy(&self.base)), cx);
                Cast::Saved { revision, source: self.base.clone() }
            }
            Written::Unconfirmed(reason) => {
                // the file is not the base any more: the next ordinary save is a conflict
                self.disk = Disk::Unstamped;
                Cast::SaveFailed { revision, reason }
            }
            Written::Failed(reason) => Cast::SaveFailed { revision, reason },
        }
    }

    fn answer(&self, cast: Cast, cx: &mut App) {
        self.panel.update(cx, |panel, cx| panel.handle(cast, cx));
    }

    /// When the tab goes without the workspace waiting for its saves: writes what the panel has not saved yet,
    /// unless the file changed on disk, counted in [`WRITES_RUNNING`]. What cannot be written is logged.
    fn flush(&mut self, cx: &mut App) {
        if let Writing::Busy(_) = self.writing {
            eprintln!("ive: {} closed while being saved; edits typed meanwhile are not saved", self.path.display());
        }
        let Some(request) = self.panel.update(cx, |panel, cx| panel.take_save(SaveKind::Checked, cx)) else {
            return self.log_unsaved(cx);
        };
        let job = match self.job(request) {
            Ok(job) => job,
            Err(reason) => return eprintln!("ive: edits to {} not saved: {reason:?}", self.path.display()),
        };
        let running = Running::start();
        cx.background_executor()
            .spawn(async move {
                let _running = running;
                if let Written::Failed(reason) | Written::Unconfirmed(reason) = job.run() {
                    eprintln!("ive: edits to {} not saved or not confirmed: {reason:?}", job.path.display());
                }
            })
            .detach();
    }

    /// Logs edits left unsaved by a failed save or an unanswered conflict.
    fn log_unsaved(&self, cx: &App) {
        if self.panel.read(cx).has_unsaved(cx) {
            eprintln!("ive: edits to {} not saved: the last save failed or met a change on disk", self.path.display());
        }
    }

    /// Reads the file again and shows it as it is now, as [`Self::reload`] does.
    pub(crate) fn read_again(&mut self, cx: &mut Context<Self>) {
        self.reload(cx);
    }

    /// Reload on a conflict: the file as it is now replaces the edits; after the write in flight, if there is one.
    fn reload(&mut self, cx: &mut Context<Self>) {
        if let Writing::Busy(after) = &mut self.writing {
            *after = AfterWrite::Reload;
            return;
        }
        let (path, version) = (self.path.clone(), self.version);
        let reread = cx.background_executor().spawn(async move { document::reread(&path, None) });
        cx.spawn(async move |file, cx| {
            let reread = reread.await;
            file.update(cx, |file, cx| file.reloaded(reread, version, cx)).ok();
        })
        .detach();
    }

    /// Shows a reload's re-read, unless a write started or landed since it began: then it reads the file again.
    fn reloaded(&mut self, reread: Reread, version: u64, cx: &mut Context<Self>) {
        if self.writing != Writing::Idle || self.version != version {
            return self.reload(cx);
        }
        self.show(reread, cx);
    }

    /// Reads the file again if it changed since IVE last read or wrote it: changes seen while writing were set aside.
    fn check_disk(&mut self, cx: &mut Context<Self>) {
        let (path, known, version) = (self.path.clone(), self.disk.stamp(), self.version);
        let reread = cx.background_executor().spawn(async move { document::reread(&path, known) });
        cx.spawn(async move |file, cx| {
            let reread = reread.await;
            // a later change of the base comes with a check of its own
            file.update(cx, |file, cx| if file.version == version { file.take_reread(reread, cx) }).ok();
        })
        .detach();
    }

    /// A change on disk: the panel shows it unless it holds edits, which the next save checks against the file.
    /// IVE's own writes, seen late, change nothing; changes seen while writing wait for the write to land.
    pub(crate) fn take_reread(&mut self, reread: Reread, cx: &mut Context<Self>) {
        let own = matches!(&reread, Reread::Changed(_, Some(stamp)) if Some(*stamp) == self.disk.stamp());
        let editing = self.writing != Writing::Idle || self.panel.read(cx).has_unsaved(cx);
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
        self.version += 1;
        match (reread, self.shows) {
            (Reread::Changed(document, _), _) => {
                self.base = Arc::new(document);
                self.present(cx);
            }
            (Reread::Gone, Shows::File) => {
                self.answer(Cast::Open(viewed(&self.base)), cx);
                self.answer(Cast::MarkDeleted, cx);
            }
            (Reread::Gone, Shows::DiffSide) => {
                self.answer(Cast::SetEditable(Editable::No), cx);
                cx.emit(FileEvent::Gone);
            }
            (Reread::Unchanged | Reread::Failed(_), _) => {}
        }
        self.notice_holding(cx);
        cx.notify();
    }
}

impl WriteJob {
    /// Writes the edits, then reads the file again: its line index may have moved even after an in-place write.
    fn run(&self) -> Written {
        let target = Target { path: &self.path, base: self.base.bytes(), expected: self.expected };
        match file_write::apply_edits(&target, &self.edits) {
            Ok(written) => self.confirm(written.stamp, Document::open_stamped(&self.path)),
            Err(WriteError::Conflict) => Written::Failed(conflict(&self.base)),
            Err(error) => Written::Failed(SaveFailure::Io(error.to_string())),
        }
    }

    /// The file as `reread` after the write that left it with stamp `written`: saved when it still has that
    /// stamp and the edited length.
    fn confirm(&self, written: Stamp, reread: Result<(Document, Option<Stamp>), DocumentError>) -> Written {
        let length = self.base.byte_len() as i64 + self.edits.iter().map(Edit::growth).sum::<i64>();
        match reread {
            Ok((document, Some(stamp))) if stamp == written && document.byte_len() as i64 == length => {
                Written::Saved { stamp, document }
            }
            Ok(_) => Written::Unconfirmed(conflict(&self.base)),
            Err(error) => Written::Unconfirmed(SaveFailure::Io(format!("saved, but cannot read it again: {error}"))),
        }
    }
}

/// The conflict a save of `base` meets: a mapped base may already hold the other program's bytes, so only a reload
/// resolves it.
fn conflict(base: &Document) -> SaveFailure {
    if base.is_mapped() {
        return SaveFailure::Conflict(Resolve::Reload);
    }
    SaveFailure::Conflict(Resolve::ReloadOrOverwrite)
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

#[cfg(test)]
mod tests {
    use std::fs;

    use gpui::TestAppContext;

    use super::*;

    /// A file holding `text` in a new folder, edited.
    fn edited_file(text: &[u8], cx: &mut TestAppContext) -> (tempfile::TempDir, PathBuf, Entity<EditedFile>) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("file.txt");
        fs::write(&path, text).unwrap();
        let opened = Document::open_stamped(&path).unwrap();
        let panel = cx.new(Panel::new);
        let file = cx.new(|cx| EditedFile::new(opened, Shown { panel, shows: Shows::File }, cx));
        (dir, path, file)
    }

    fn base_of(file: &Entity<EditedFile>, cx: &mut TestAppContext) -> Vec<u8> {
        file.read_with(cx, |file, _| file.base.bytes().to_vec())
    }

    /// A job writing `edits` to the file holding `text`, as last read.
    fn job_of(text: &[u8], edits: Vec<Edit>) -> (tempfile::TempDir, WriteJob) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("file.txt");
        fs::write(&path, text).unwrap();
        let (document, stamp) = Document::open_stamped(&path).unwrap();
        let job = WriteJob { path, base: Arc::new(document), expected: Expected::Stamp(stamp.unwrap()), edits };
        (dir, job)
    }

    #[test]
    fn should_keep_the_base_read_when_another_repository_changes_meanwhile() {
        // given
        let mut sent = ChangeSent::default();
        let base = || ChangeBase::Tracked { head: codeview::document::Source::from_vec(b"one\n".to_vec()), head_id: "abc".into() };
        // when
        let skipped = sent.took(2, Ok(None));
        let first = sent.took(1, Ok(Some(base())));
        let newer = sent.took(3, Ok(Some(base())));
        let older = sent.took(2, Ok(Some(base())));
        // then
        assert!(skipped.is_none(), "a read skipped for another repository casts nothing");
        assert!(first.is_some(), "the earlier read in flight is still cast");
        assert!(newer.is_some());
        assert!(older.is_none(), "an answer older than one taken is stale");
    }

    #[test]
    fn should_not_take_a_file_changed_right_after_the_write_as_saved() {
        // given
        let (_dir, job) = job_of(b"one\n", vec![edit(0..0, b"x")]);
        let written = job.run();
        let Written::Saved { stamp, .. } = written else {
            panic!("the write did not go through");
        };
        fs::write(&job.path, b"theirs, longer\n").unwrap();

        // when
        let confirmed = job.confirm(stamp, Document::open_stamped(&job.path));

        // then
        assert!(matches!(confirmed, Written::Unconfirmed(SaveFailure::Conflict(Resolve::ReloadOrOverwrite))));
    }

    #[test]
    fn should_not_take_a_saved_text_of_another_length_as_saved() {
        // given
        let (_dir, job) = job_of(b"one\n", vec![edit(0..0, b"x")]);
        let other = tempfile::tempdir().unwrap();
        fs::write(other.path().join("other.txt"), b"other text\n").unwrap();
        let (document, stamp) = Document::open_stamped(&other.path().join("other.txt")).unwrap();

        // when
        let confirmed = job.confirm(stamp.unwrap(), Ok((document, stamp)));

        // then
        assert!(matches!(confirmed, Written::Unconfirmed(SaveFailure::Conflict(_))));
    }

    #[gpui::test]
    fn should_show_a_change_made_while_writing_once_the_write_landed(cx: &mut TestAppContext) {
        // given
        let (_dir, path, file) = edited_file(b"one\n", cx);
        let job = file.read_with(cx, |file, _| file.job(request(1, vec![edit(0..0, b"x")], SaveKind::Checked))).unwrap();
        file.update(cx, |file, _| file.writing = Writing::Busy(AfterWrite::Check));
        let written = job.run();
        fs::write(&path, b"theirs\n").unwrap();
        let known = file.read_with(cx, |file, _| file.disk.stamp());
        file.update(cx, |file, cx| file.take_reread(document::reread(&path, known), cx));

        // when
        file.update(cx, |file, cx| file.written(1, written, cx));
        cx.run_until_parked();

        // then
        assert_eq!(base_of(&file, cx), b"theirs\n");
    }

    #[gpui::test]
    fn should_refuse_to_overwrite_a_mapped_file(cx: &mut TestAppContext) {
        // given
        let large = vec![b'\n'; 8 * 1024 * 1024 + 1];
        let (_dir, _path, file) = edited_file(&large, cx);

        // when
        let job = file.read_with(cx, |file, _| file.job(request(1, vec![edit(0..0, b"x")], SaveKind::Overwrite)).err());

        // then
        assert_eq!(job, Some(SaveFailure::Conflict(Resolve::Reload)));
    }

    fn request(revision: u64, edits: Vec<Edit>, kind: SaveKind) -> SaveRequest {
        SaveRequest { base: revision - 1, revision, edits, kind }
    }

    fn edit(bytes: std::ops::Range<u64>, text: &[u8]) -> Edit {
        Edit { bytes, text: text.to_vec() }
    }

    fn save(file: &Entity<EditedFile>, request: SaveRequest, cx: &mut TestAppContext) {
        file.update(cx, |file, cx| file.save(request, cx));
    }

    #[gpui::test]
    fn should_refuse_the_next_save_once_a_saved_file_cannot_be_read_again(cx: &mut TestAppContext) {
        // given
        let (_dir, path, file) = edited_file(b"one\ntwo\n", cx);
        // a NUL byte at the start makes the file look binary: it cannot be opened again
        save(&file, request(1, vec![edit(0..0, b"\0")], SaveKind::Checked), cx);
        cx.run_until_parked();

        // when
        save(&file, request(2, vec![edit(0..0, b"\0"), edit(4..4, b"!")], SaveKind::Checked), cx);
        cx.run_until_parked();

        // then
        assert_eq!(fs::read(&path).unwrap(), b"\0one\ntwo\n");
    }

    #[gpui::test(iterations = 50)]
    fn should_reload_only_after_the_write_in_flight_landed(cx: &mut TestAppContext) {
        // given
        let (_dir, path, file) = edited_file(b"one\n", cx);
        fs::write(&path, b"theirs\n").unwrap();
        save(&file, request(1, vec![edit(0..0, b"x")], SaveKind::Overwrite), cx);

        // when
        file.update(cx, |file, cx| file.reload(cx));
        cx.run_until_parked();

        // then
        let (base, stamp) = file.read_with(cx, |file, _| (file.base.bytes().to_vec(), file.disk.stamp()));
        assert_eq!((base, stamp), (fs::read(&path).unwrap(), document::stamp(&path)));
    }

    fn typed(panel: &Entity<Panel>, text: &str, cx: &mut TestAppContext) {
        panel.update(cx, |panel, cx| panel.handle(Cast::Type(text.to_owned()), cx));
    }

    #[gpui::test]
    fn should_keep_a_file_read_only_while_another_panel_holds_edits_to_it(cx: &mut TestAppContext) {
        // given
        let (_dir, path, first) = edited_file(b"one\n", cx);
        let first_panel = first.read_with(cx, |file, _| file.panel.clone());
        let second_panel = cx.new(Panel::new);
        let opened = Document::open_stamped(&path).unwrap();
        let second = cx.new(|cx| EditedFile::new(opened, Shown { panel: second_panel.clone(), shows: Shows::File }, cx));

        // when
        typed(&first_panel, "a", cx);
        let locked = second.read_with(cx, |file, _| file.lock_note().is_some());
        typed(&second_panel, "b", cx);
        let refused = second_panel.read_with(cx, |panel, cx| !panel.has_unsaved(cx));
        first_panel.update(cx, |panel, cx| panel.handle(Cast::Command(codeview::host::Command::Flush), cx));
        cx.run_until_parked();

        // then
        assert_eq!((locked, refused), (true, true));
        assert_eq!(fs::read(&path).unwrap(), b"aone\n");
        assert_eq!((base_of(&second, cx), second.read_with(cx, |file, _| file.lock_note())), (b"aone\n".to_vec(), None));
        typed(&second_panel, "c", cx);
        assert!(second_panel.read_with(cx, |panel, cx| panel.has_unsaved(cx)));
    }

    #[gpui::test]
    fn should_refuse_text_sent_to_a_file_tab_while_another_tab_holds_edits_to_its_file(cx: &mut TestAppContext) {
        // given
        let (_dir, path, first) = edited_file(b"one\n", cx);
        let first_panel = first.read_with(cx, |file, _| file.panel.clone());
        let panel = cx.new(Panel::new);
        let opened = Document::open_stamped(&path).unwrap();
        let file = cx.new(|cx| EditedFile::new(opened, Shown { panel: panel.clone(), shows: Shows::File }, cx));
        let tab = FileTab { path: path.clone(), panel, file };
        typed(&first_panel, "a", cx);

        // when
        let sent = cx.update(|cx| tab.type_text("b", cx));

        // then
        let error = sent.expect_err("the tab is read-only");
        assert!(error.contains("file.txt") && error.contains("read-only"), "{error}");
    }

    #[gpui::test]
    fn should_fail_the_save_to_a_read_only_file_and_leave_it_unchanged(cx: &mut TestAppContext) {
        // given
        let (_dir, path, file) = edited_file(b"one\n", cx);
        let panel = file.read_with(cx, |file, _| file.panel.clone());
        let mut mode = fs::metadata(&path).unwrap().permissions();
        mode.set_readonly(true);
        fs::set_permissions(&path, mode).unwrap();
        panel.update(cx, |panel, cx| panel.handle(Cast::Type("x".to_owned()), cx));
        panel.update(cx, |panel, cx| panel.handle(Cast::CloseAsked, cx));
        panel.update(cx, |panel, cx| panel.handle(Cast::Command(codeview::host::Command::Flush), cx));
        cx.run_until_parked();
        let failed = panel.read_with(cx, |panel, cx| panel.has_unsaved(cx));

        // when
        panel.update(cx, |panel, cx| panel.handle(Cast::Command(codeview::host::Command::Flush), cx));
        file.update(cx, |file, cx| file.flush(cx));
        cx.run_until_parked();

        // then
        assert!(failed);
        assert_eq!(fs::read(&path).unwrap(), b"one\n");
    }
}
