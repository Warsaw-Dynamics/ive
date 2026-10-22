//! A file's tab: the file edited in a panel, its edits written by `file_write` as the panel asks, and what IVE
//! knows of the file on disk.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use codeview::document::{self, DocumentError, Reread, Stamp};
use codeview::panel::{Cast, Content, EditPolicy, Edit, Event, Panel, Resolve, SaveFailure, SaveKind, SaveRequest};
use gpui::{AnyElement, App, AppContext, Context, Entity, Subscription, div, prelude::*};

use crate::codeview_host;
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
        let file = cx.new(|cx| EditedFile::new(opened, panel.clone(), cx));
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
///
/// `base` is the panel's base at all times: it changes only with an answer that moves the panel onto it (`Saved`,
/// `Open`), never while a write is in flight. `disk` is `Disk::Read` only while the file holds `base`, so a save
/// checked against it writes at the right offsets.
struct EditedFile {
    path: PathBuf,
    panel: Entity<Panel>,
    /// the panel's base: the file as last read or written
    base: Arc<Document>,
    disk: Disk,
    writing: Writing,
    /// counts the changes of `base` and `disk`: a re-read started before the last change is stale
    version: u64,
    _subscriptions: Vec<Subscription>,
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

impl EditedFile {
    fn new(opened: (Document, Option<Stamp>), panel: Entity<Panel>, cx: &mut Context<Self>) -> Self {
        let (document, stamp) = opened;
        let base = Arc::new(document);
        panel.update(cx, |panel, cx| panel.handle(Cast::Open(edited(&base)), cx));
        let subscriptions = vec![
            cx.subscribe(&panel, |file, _panel, event: &Event, cx| file.on_panel_event(event, cx)),
            // the workspace closes a tab once its saves landed; anything else releasing it gets this last chance
            cx.on_release(|file, cx| file.flush(cx)),
        ];
        let disk = stamp.map_or(Disk::Unstamped, Disk::Read);
        let path = base.path().to_path_buf();
        Self { path, panel, base, disk, writing: Writing::Idle, version: 0, _subscriptions: subscriptions }
    }

    fn on_panel_event(&mut self, event: &Event, cx: &mut Context<Self>) {
        match event {
            Event::SaveRequested(request) => self.save(request.clone(), cx),
            Event::ReloadRequested => self.reload(cx),
            // parsing is `codeview_host::serve`'s; scroll and caret are saved with the tabs on redraw; the workspace
            // closes the tab on a discard
            Event::DiscardRequested
            | Event::Edited { .. }
            | Event::Refused(_)
            | Event::StateChanged
            | Event::LayoutToggled(_)
            | Event::NeedParse(_)
            | Event::NeedTexts { .. }
            | Event::OpenAt { .. } => {}
        }
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
        let answer = match written {
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
        };
        self.answer(answer, cx);
        match after {
            AfterWrite::Check => self.check_disk(cx),
            AfterWrite::Reload => self.reload(cx),
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
    fn take_reread(&mut self, reread: Reread, cx: &mut Context<Self>) {
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
        let file = cx.new(|cx| EditedFile::new(opened, panel, cx));
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
