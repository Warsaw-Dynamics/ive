//! The panel: the one entity a host holds for any code it shows. The host casts messages to it, calls it for
//! answers, and hears its events; see the spec's "The interface: actor-style messages".

#[cfg(test)]
mod change_popup_tests;
#[cfg(test)]
mod diff_tests;
#[cfg(test)]
mod edit_diff_tests;
mod protocol;
mod saving;
mod state;

use std::sync::Arc;
use std::time::Duration;

use gpui::{
    AnyElement, App, AppContext, Context, Entity, EventEmitter, FocusHandle, Focusable, IntoElement, ParentElement, Render,
    Styled, Subscription, Task, Window, div,
};

pub use protocol::{
    Cast, Content, DiffInput, DiffLayout, Edit, EditBase, EditPolicy, Editable, Event, LineSource, ParseJob, ParseTarget, Resolve,
    SaveFailure, SaveKind, SaveRequest, TextPurpose, TextsRequest,
};
pub(crate) use protocol::{Failed, Previous};
pub use state::PanelState;

use crate::changes::{BaseChange, ChangeBase, ChangeHunks};
use crate::code_view::CodeView;
use crate::diff::texts::Parsed;
use crate::diff::{DiffEvent, DiffView, FileDiff, Shape};
use crate::document::Source;
use crate::host::Command;
use crate::syntax::{Language, Syntax};
use crate::piece_table::PieceTable;
use crate::text_editor::{Setup, TextEditor, TextEditorEvent};
use saving::{Answer, BarActions, Blur, Discarded, Next, Saving, Then};

/// The pause after the last edit before the change markers are diffed again.
pub const CHANGE_DIFF_DELAY: Duration = Duration::from_millis(300);

/// A text, a diff, or nothing yet; viewed or edited as the last [`Cast::Open`] said.
pub struct Panel {
    body: Body,
    pointer: Pointer,
    focus_handle: FocusHandle,
    /// the state as last seen: what [`Panel::state`] answers and what [`Event::StateChanged`] compares with
    seen: PanelState,
    /// a [`Cast::Restore`] waiting for the content it applies to
    pending: Pending,
    /// the text's revision: counts new texts in View mode, edits in Edit mode
    revision: u64,
    /// Edit mode's saves
    saving: Saving,
    blur: Blur,
    /// the pause after the last edit before the change markers are diffed again
    change_pause: Task<()>,
    /// the body's own events, passed on as the panel's
    _body_events: Vec<Subscription>,
}

#[derive(Clone)]
enum Body {
    Empty,
    View(Entity<CodeView>),
    Edit(Entity<TextEditor>),
    Diff(Entity<DiffView>),
}

enum Pending {
    Nothing,
    Restore(PanelState),
}

/// What the mouse does in View mode.
#[derive(Clone, Copy)]
enum Pointer {
    Selects,
    /// the text is only shown, and a press leaves the focus where it is
    Ignored,
}

impl EventEmitter<Event> for Panel {}

impl Panel {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            body: Body::Empty,
            pointer: Pointer::Selects,
            focus_handle: cx.focus_handle(),
            seen: PanelState::default(),
            pending: Pending::Nothing,
            revision: 0,
            saving: Saving::new(Duration::ZERO, 0),
            blur: Blur::Unwatched,
            change_pause: Task::ready(()),
            _body_events: Vec::new(),
        }
    }

    /// This panel with View mode's mouse selecting nothing.
    pub fn unselectable(self) -> Self {
        Self { pointer: Pointer::Ignored, ..self }
    }

    pub fn handle(&mut self, cast: Cast, cx: &mut Context<Self>) {
        match cast {
            Cast::Open(content) => self.open(content, cx),
            Cast::Restore(state) => self.restore(state, cx),
            Cast::SetFiles(files) => self.set_files(files, cx),
            Cast::Parsed { target, revision, syntax } => self.parsed(target, revision, syntax, cx),
            Cast::Command(command) => self.run(command, cx),
            Cast::RevealLine(line) => self.reveal_line(line, cx),
            Cast::MarkDeleted => self.mark_deleted(cx),
            Cast::Saved { revision, source } => self.saved(revision, source, cx),
            Cast::SaveFailed { revision, reason } => self.save_failed(revision, reason),
            Cast::CloseAsked => self.saving.ask_close(),
            Cast::CloseWithdrawn => self.saving.withdraw_close(),
            Cast::SetPolicy(policy) => self.saving.delay = policy.save_delay,
            Cast::SetEditable(editable) => self.set_editable(editable, cx),
            Cast::Type(text) => self.type_text(&text, cx),
            Cast::SetTexts { file, generation, old, new } => {
                self.set_texts(generation, [(ParseTarget::DiffOld(file), old), (ParseTarget::DiffNew(file), new)], cx);
            }
            Cast::SetEditBase { generation, base } => self.set_edit_base(generation, base, cx),
            Cast::SetBaseDiff { base, diff } => self.set_base_diff(&base, diff, cx),
            Cast::SetChangeBase { base } => self.set_change_base(base, cx),
            Cast::ChangeDiff { revision, hunks } => self.change_diffed(revision, hunks, cx),
            Cast::ChangeCommitted => self.with_editor(cx, TextEditor::change_committed),
            Cast::ChangeCommitFailed(reason) => self.with_editor(cx, |editor, cx| editor.change_commit_failed(reason, cx)),
        }
        self.notice_state(cx);
        cx.notify();
    }

    /// Scroll, caret and collapsed files as of the last cast or frame; a restore still waiting for content.
    pub fn state(&self) -> PanelState {
        self.seen.clone()
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Edits not saved yet: in flight, waiting for the pause, or kept after a failed save.
    pub fn has_unsaved(&self, cx: &App) -> bool {
        self.edit_revision(cx).is_some_and(|revision| self.saving.has_unsaved(revision))
    }

    /// Whether typing reaches a text now: in Edit mode, or a diff's new side taken to edit.
    pub fn takes_typing(&self, cx: &App) -> bool {
        self.edit_revision(cx).is_some()
    }

    /// The revision of the text edited: in Edit mode, or a diff's new side; `None` when nothing is edited.
    fn edit_revision(&self, cx: &App) -> Option<u64> {
        match &self.body {
            Body::Edit(editor) => Some(editor.read(cx).revision()),
            Body::Diff(diff) => diff.read(cx).edited().map(|model| model.revision()),
            Body::Empty | Body::View(_) => None,
        }
    }

    /// Whether a diff takes the new side's text answering ask `generation` as the text it edits: the host pairs that
    /// text with what it knows of the file on disk before answering.
    pub fn waits_to_edit(&self, generation: u64, cx: &App) -> bool {
        match &self.body {
            Body::Diff(diff) => diff.read(cx).waits_to_edit(generation),
            Body::Empty | Body::View(_) | Body::Edit(_) => false,
        }
    }

    /// Whether a diff edits its new side, or waits for its text to.
    pub fn is_editable_diff(&self, cx: &App) -> bool {
        match &self.body {
            Body::Diff(diff) => diff.read(cx).is_editable(),
            Body::Empty | Body::View(_) | Body::Edit(_) => false,
        }
    }

    /// Whether a diff's rows show its edited new side: taken to edit, so [`Cast::SetFiles`] no longer reaches them.
    pub fn edits_new_side(&self, cx: &App) -> bool {
        match &self.body {
            Body::Diff(diff) => diff.read(cx).edits_rows(),
            Body::Empty | Body::View(_) | Body::Edit(_) => false,
        }
    }

    /// File `file` of the diff shown; `None` outside Diff mode or past its files.
    pub fn diff_file<'a>(&self, file: usize, cx: &'a App) -> Option<&'a FileDiff> {
        match &self.body {
            Body::Diff(diff) => diff.read(cx).file(file),
            Body::Empty | Body::View(_) | Body::Edit(_) => None,
        }
    }

    /// The diff's layout; `None` outside Diff mode.
    pub fn diff_layout(&self, cx: &App) -> Option<DiffLayout> {
        match &self.body {
            Body::Diff(diff) => Some(diff.read(cx).layout()),
            Body::Empty | Body::View(_) | Body::Edit(_) => None,
        }
    }

    /// A save is in flight: the host has not answered it yet.
    pub fn is_saving(&self) -> bool {
        self.saving.is_in_flight()
    }

    /// The save the panel would ask for now, handed over instead of emitted, for a host that must write before going
    /// on (a scratch closing, or about to run). It counts as in flight until answered; `None` when there is nothing
    /// to save, a save is in flight already, or a [`SaveKind::Checked`] one waits for the user after a conflict.
    pub fn take_save(&mut self, kind: SaveKind, cx: &mut Context<Self>) -> Option<SaveRequest> {
        match self.next_save(kind, cx) {
            Next::Send(request) => Some(request),
            Next::Nothing | Next::AlreadySaved => None,
        }
    }

    /// New content replaces the body, except new text for the same mode, which keeps scroll and caret, and text
    /// switching between View and Edit mode, which keeps the scroll (and a caret staying in Edit mode), clamped to
    /// the new mode's line count.
    fn open(&mut self, content: Content, cx: &mut Context<Self>) {
        match (content, self.body.clone()) {
            (Content::Lines { source, language, edit: None }, Body::View(view)) => {
                view.update(cx, |view, cx| view.replace_source(source, language, cx));
                self.revision += 1;
                self.request_parse(cx);
            }
            (Content::Lines { source, language, edit: Some(policy) }, Body::Edit(editor)) => {
                let setup = Setup { limit: policy.limit, language };
                editor.update(cx, |editor, cx| editor.reset(PieceTable::new(source), setup, cx));
                self.revision = editor.read(cx).revision();
                self.saving = Saving::new(policy.save_delay, self.revision);
                self.request_parse(cx);
                cx.emit(Event::NeedChangeBase);
                self.request_change_diff(cx);
            }
            (Content::Diff(input), Body::Diff(diff)) => diff.update(cx, |diff, cx| diff.set_files(input.files, cx)),
            (content @ Content::Lines { .. }, Body::View(_) | Body::Edit(_)) => {
                let carried = self.observed_state(cx);
                self.build(content, cx);
                self.apply(&carried, cx);
            }
            (content, _) => self.build(content, cx),
        }
        self.apply_pending(cx);
    }

    fn build(&mut self, content: Content, cx: &mut Context<Self>) {
        self._body_events.clear();
        self.blur = Blur::Unwatched;
        match content {
            Content::Lines { source, language, edit: None } => self.build_view(source, language, cx),
            Content::Lines { source, language, edit: Some(policy) } => {
                self.build_edit(source, Setup { limit: policy.limit, language }, cx);
                self.saving = Saving::new(policy.save_delay, self.revision);
            }
            Content::Diff(input) => self.build_diff(input, cx),
        }
    }

    fn build_view(&mut self, source: Arc<dyn LineSource>, language: Option<Language>, cx: &mut Context<Self>) {
        let pointer = self.pointer;
        self.body = Body::View(cx.new(|cx| match pointer {
            Pointer::Selects => CodeView::new(source, language, cx),
            Pointer::Ignored => CodeView::new(source, language, cx).unselectable(),
        }));
        self.revision += 1;
        self.request_parse(cx);
    }

    /// Edit mode reads `source` through a piece table: only the lines it shows or edits.
    fn build_edit(&mut self, source: Arc<dyn LineSource>, setup: Setup, cx: &mut Context<Self>) {
        let editor = cx.new(|cx| TextEditor::new(PieceTable::new(source), setup, cx));
        self._body_events.push(cx.subscribe(&editor, Self::on_editor_event));
        self.revision = editor.read(cx).revision();
        self.body = Body::Edit(editor);
        self.request_parse(cx);
        cx.emit(Event::NeedChangeBase);
    }

    fn build_diff(&mut self, input: DiffInput, cx: &mut Context<Self>) {
        let diff = cx.new(|cx| {
            let mut diff = DiffView::new(Shape { headers: input.headers, layout: input.layout }, cx);
            diff.set_files(input.files, cx);
            diff
        });
        self._body_events.push(cx.subscribe(&diff, Self::on_diff_event));
        self.body = Body::Diff(diff);
        self.set_editable(input.editable, cx);
    }

    /// A diff edits its new side, saved as `editable` says, or stops editing it.
    fn set_editable(&mut self, editable: Editable, cx: &mut Context<Self>) {
        let Body::Diff(diff) = &self.body else {
            return;
        };
        diff.update(cx, |diff, cx| diff.set_editable(editable, cx));
        if let Editable::NewSide { policy } = editable {
            // the text comes with revision 0, as saved
            self.saving = Saving::new(policy.save_delay, 0);
        }
    }

    fn on_diff_event(&mut self, _diff: Entity<DiffView>, event: &DiffEvent, cx: &mut Context<Self>) {
        match event {
            DiffEvent::NeedTexts(request) => cx.emit(Event::NeedTexts(*request)),
            DiffEvent::LayoutToggled(layout) => {
                cx.emit(Event::LayoutToggled(*layout));
                self.notice_state(cx);
            }
            DiffEvent::Edited => self.edited(cx),
        }
    }

    /// The edited text changed: it is unsaved, and saved after the pause.
    fn edited(&mut self, cx: &mut Context<Self>) {
        self.revision = self.edit_revision(cx).unwrap_or(self.revision);
        cx.emit(Event::Edited { revision: self.revision });
        self.restart_pause(cx);
    }

    /// A diff file's sides, each the text of the side its target names, answering ask `generation`; a missing side
    /// stays plain.
    fn set_texts(&self, generation: u64, sides: [(ParseTarget, Option<Source>); 2], cx: &mut Context<Self>) {
        let Body::Diff(diff) = &self.body else {
            return;
        };
        for (target, source) in sides {
            let job = diff.update(cx, |diff, cx| diff.set_text(target, generation, source, cx));
            if let Some(job) = job {
                cx.emit(Event::NeedParse(job));
            }
        }
    }

    /// The new side to edit, answering ask `generation`, and the parses that colour its texts.
    fn set_edit_base(&self, generation: u64, base: EditBase, cx: &mut Context<Self>) {
        let Body::Diff(diff) = &self.body else {
            return;
        };
        let jobs = diff.update(cx, |diff, cx| diff.set_edit_base(generation, base, cx));
        for job in jobs {
            cx.emit(Event::NeedParse(job));
        }
    }

    fn set_base_diff(&self, base: &Arc<dyn LineSource>, file: FileDiff, cx: &mut Context<Self>) {
        if let Body::Diff(diff) = &self.body {
            diff.update(cx, |diff, cx| diff.set_base_diff(base, file, cx));
        }
    }

    fn on_editor_event(&mut self, _editor: Entity<TextEditor>, event: &TextEditorEvent, cx: &mut Context<Self>) {
        match event {
            TextEditorEvent::Edited => {
                self.edited(cx);
                self.request_parse(cx);
                self.diff_changes_later(cx);
            }
            TextEditorEvent::NeedChanges => self.diff_changes_now(cx),
            TextEditorEvent::Refused(reason) => cx.emit(Event::Refused(reason.clone())),
            TextEditorEvent::CommitChange { head_id, block, text, message } => {
                let (head_id, block, text, message) = (head_id.clone(), block.clone(), text.clone(), message.clone());
                cx.emit(Event::CommitChange { head_id, block, text, message });
            }
            TextEditorEvent::OpenDiff { line } => {
                // the diff shows the file on disk: the edits go there first
                self.request_save(SaveKind::Checked, cx);
                cx.emit(Event::OpenDiff { line: *line });
            }
        }
    }

    /// Runs `update` on the editor, in Edit mode.
    fn with_editor(&self, cx: &mut Context<Self>, update: impl FnOnce(&mut TextEditor, &mut Context<TextEditor>)) {
        if let Body::Edit(editor) = &self.body {
            editor.update(cx, update);
        }
    }

    /// Asks the host for the parse of the text as it is now; in Edit mode, only once the one running landed.
    fn request_parse(&self, cx: &mut Context<Self>) {
        let job = match &self.body {
            Body::View(view) => view.read(cx).parse_job(self.revision),
            Body::Edit(editor) => editor.update(cx, |editor, _| editor.parse_job()),
            Body::Empty | Body::Diff(_) => None,
        };
        if let Some(job) = job {
            cx.emit(Event::NeedParse(job));
        }
    }

    /// The change markers compare the edited text with `base` from now on: diffed now, unless it is the same HEAD.
    fn set_change_base(&self, base: ChangeBase, cx: &mut Context<Self>) {
        let Body::Edit(editor) = &self.body else {
            return;
        };
        if editor.update(cx, |editor, cx| editor.set_change_base(base, cx)) == BaseChange::Moved {
            self.request_change_diff(cx);
        }
    }

    /// Asks the host for the diff of the edited text with the markers' base, unless one is in flight or not needed.
    fn request_change_diff(&self, cx: &mut Context<Self>) {
        let Body::Edit(editor) = &self.body else {
            return;
        };
        if let Some(job) = editor.update(cx, |editor, _| editor.change_job()) {
            cx.emit(Event::NeedChangeDiff(job));
        }
    }

    /// A diff landed; edits made while it ran are diffed after the pause, or at once while the popup waits for it.
    fn change_diffed(&mut self, revision: u64, hunks: ChangeHunks, cx: &mut Context<Self>) {
        let Body::Edit(editor) = &self.body else {
            return;
        };
        let current = editor.update(cx, |editor, cx| {
            editor.change_diffed(revision, hunks, cx);
            editor.has_current_changes()
        });
        if current {
            return;
        }
        self.diff_changes_later(cx);
    }

    /// The change markers are diffed again after the pause, or at once while the change popup waits for them.
    fn diff_changes_later(&mut self, cx: &mut Context<Self>) {
        let Body::Edit(editor) = &self.body else {
            return;
        };
        if editor.read(cx).waits_for_changes() {
            return self.diff_changes_now(cx);
        }
        self.restart_change_pause(cx);
    }

    /// Diffs the change markers now (once the diff in flight landed), instead of after the pause.
    fn diff_changes_now(&mut self, cx: &mut Context<Self>) {
        self.change_pause = Task::ready(());
        self.request_change_diff(cx);
    }

    /// The edit that just happened starts the pause before the change markers are diffed again.
    fn restart_change_pause(&mut self, cx: &mut Context<Self>) {
        self.change_pause = cx.spawn(async move |panel, cx| {
            cx.background_executor().timer(CHANGE_DIFF_DELAY).await;
            panel.update(cx, |panel, cx| panel.request_change_diff(cx)).ok();
        });
    }

    /// In View mode a parse of an older text is dropped, in Edit mode the editor takes it, a diff takes it for the
    /// side it was asked for unless that side's text was replaced since.
    fn parsed(&self, target: ParseTarget, revision: u64, syntax: Syntax, cx: &mut Context<Self>) {
        match (&self.body, target) {
            (Body::View(view), ParseTarget::Text) if revision == self.revision => {
                view.update(cx, |view, cx| view.set_syntax(syntax, cx));
            }
            (Body::Edit(editor), ParseTarget::Text) => {
                editor.update(cx, |editor, cx| editor.parsed(revision, syntax, cx));
                // edits made while it ran are parsed next
                self.request_parse(cx);
            }
            (Body::Diff(diff), ParseTarget::DiffOld(_) | ParseTarget::DiffNew(_)) => {
                diff.update(cx, |diff, cx| diff.parsed(Parsed { target, revision, syntax }, cx));
            }
            _ => {}
        }
    }

    fn set_files(&mut self, files: Vec<FileDiff>, cx: &mut Context<Self>) {
        if let Body::Diff(diff) = &self.body {
            diff.update(cx, |diff, cx| diff.set_files(files, cx));
        }
        self.apply_pending(cx);
    }

    fn run(&mut self, command: Command, cx: &mut Context<Self>) {
        if command == Command::Flush {
            return self.request_save(SaveKind::Checked, cx);
        }
        match &self.body {
            Body::View(view) => view.update(cx, |view, cx| view.run(command, cx)),
            Body::Edit(editor) => editor.update(cx, |editor, cx| editor.run(command, cx)),
            Body::Diff(diff) => diff.update(cx, |diff, cx| diff.run(command, cx)),
            Body::Empty => {}
        }
    }

    /// Types `text` at the caret; a refusal is reported as [`Event::Refused`], and outside Edit mode nothing happens.
    fn type_text(&self, text: &str, cx: &mut Context<Self>) {
        match &self.body {
            // a refused edit was already reported through TextEditorEvent::Refused
            Body::Edit(editor) => drop(editor.update(cx, |editor, cx| editor.type_text(text, cx))),
            Body::Diff(diff) => diff.update(cx, |diff, cx| diff.type_text(text, cx)),
            Body::Empty | Body::View(_) => {}
        }
    }

    /// The edit that just happened starts the pause before a save again.
    fn restart_pause(&mut self, cx: &mut Context<Self>) {
        let delay = self.saving.delay;
        let pause = cx.spawn(async move |panel, cx| {
            cx.background_executor().timer(delay).await;
            panel.update(cx, |panel, cx| panel.request_save(SaveKind::Checked, cx)).ok();
        });
        self.saving.restart_pause(pause);
    }

    /// Asks the host to save the edits, unless one save is in flight already: then right after its answer.
    fn request_save(&mut self, kind: SaveKind, cx: &mut Context<Self>) {
        if let Next::Send(request) = self.next_save(kind, cx) {
            cx.emit(Event::SaveRequested(request));
        }
        cx.notify();
    }

    fn next_save(&mut self, kind: SaveKind, cx: &mut Context<Self>) -> Next {
        let Some(revision) = self.edit_revision(cx) else {
            return Next::Nothing;
        };
        let body = &self.body;
        self.saving.next(kind, revision, || match body {
            Body::Edit(editor) => editor.read(cx).edits_since_base(),
            Body::Diff(diff) => diff.read(cx).edited().map_or_else(Vec::new, |model| model.edits_since_base()),
            Body::Empty | Body::View(_) => Vec::new(),
        })
    }

    fn saved(&mut self, revision: u64, source: Arc<dyn LineSource>, cx: &mut Context<Self>) {
        let body = self.body.clone();
        let rebase = |edits: &[Edit]| match body {
            Body::Edit(editor) => editor.update(cx, |editor, _cx| editor.rebase(source, edits)),
            Body::Diff(diff) => diff.update(cx, |diff, cx| diff.rebase(source, edits, cx)),
            Body::Empty | Body::View(_) => Ok(()),
        };
        if let Answer::Saved { then: Then::SaveAgain } = self.saving.saved(revision, rebase) {
            self.request_save(SaveKind::Checked, cx);
        }
    }

    /// The user chose closing without saving: the edits count as saved, and the host is told to close.
    fn discard_edits(&mut self, cx: &mut Context<Self>) {
        if self.saving.discard() == Discarded::Done {
            cx.emit(Event::DiscardRequested);
            cx.notify();
        }
    }

    fn save_failed(&mut self, revision: u64, reason: SaveFailure) {
        // the bar shows the failure; the edits wait for the next pause or flush
        let _ = self.saving.failed(revision, reason);
    }

    fn reveal_line(&self, line: usize, cx: &mut Context<Self>) {
        match &self.body {
            Body::View(view) => view.update(cx, |view, cx| view.reveal_line(line, cx)),
            Body::Edit(editor) => editor.update(cx, |editor, cx| editor.reveal_line(line, cx)),
            Body::Diff(diff) => diff.update(cx, |diff, cx| diff.reveal_hunk_at(line, cx)),
            Body::Empty => {}
        }
    }

    fn mark_deleted(&self, cx: &mut Context<Self>) {
        if let Body::View(view) = &self.body {
            view.update(cx, |view, cx| view.mark_deleted(cx));
        }
    }

    fn restore(&mut self, state: PanelState, cx: &mut Context<Self>) {
        self.pending = Pending::Restore(state);
        self.apply_pending(cx);
    }

    /// Applies a waiting restore once there is content to clamp it to.
    fn apply_pending(&mut self, cx: &mut Context<Self>) {
        if self.has_content(cx)
            && let Pending::Restore(state) = std::mem::replace(&mut self.pending, Pending::Nothing)
        {
            self.apply(&state, cx);
        }
    }

    fn has_content(&self, cx: &App) -> bool {
        match &self.body {
            Body::Empty => false,
            Body::View(_) | Body::Edit(_) => true,
            Body::Diff(diff) => diff.read(cx).has_files(),
        }
    }

    fn apply(&self, state: &PanelState, cx: &mut Context<Self>) {
        match &self.body {
            Body::View(view) => view.update(cx, |view, cx| view.restore_scroll(state.scroll, cx)),
            Body::Edit(editor) => editor.update(cx, |editor, cx| editor.restore_view(state.caret.unwrap_or_default(), state.scroll, cx)),
            Body::Diff(diff) => diff.update(cx, |diff, cx| {
                if let Some(layout) = state.layout {
                    diff.set_layout(layout, cx);
                }
                // no saved list is the same as none known: the diff keeps the collapse its files started with
                if !state.collapsed.is_empty() {
                    diff.restore_collapsed(&state.collapsed, cx);
                }
                diff.restore_scroll(state.scroll, cx);
            }),
            Body::Empty => {}
        }
    }

    /// Reads the state off the body, and tells the host when it changed.
    fn notice_state(&mut self, cx: &mut Context<Self>) {
        let state = self.observed_state(cx);
        if state == self.seen {
            return;
        }
        self.seen = state;
        cx.emit(Event::StateChanged);
    }

    fn observed_state(&self, cx: &App) -> PanelState {
        if let Pending::Restore(state) = &self.pending {
            return state.clone();
        }
        match &self.body {
            Body::Empty => PanelState::default(),
            Body::View(view) => PanelState { scroll: view.read(cx).scroll_position(), ..PanelState::default() },
            Body::Edit(editor) => {
                let editor = editor.read(cx);
                PanelState { scroll: editor.scroll_position(), caret: Some(editor.caret_position()), ..PanelState::default() }
            }
            Body::Diff(diff) => {
                let diff = diff.read(cx);
                PanelState { scroll: diff.scroll_position(), layout: Some(diff.layout()), collapsed: diff.collapsed_paths(), ..PanelState::default() }
            }
        }
    }
}

impl Panel {
    /// The edited body (an editor, or a diff editing its new side) under the bar of a failed save; watches the body's
    /// focus to save when it leaves.
    fn render_edit(&mut self, editor: impl Focusable + IntoElement, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        if let Blur::Unwatched = self.blur {
            let focus = editor.focus_handle(cx);
            let watch = cx.on_blur(&focus, window, |panel, _window, cx| panel.request_save(SaveKind::Checked, cx));
            self.blur = Blur::Watched { _subscription: watch };
        }
        let this = cx.weak_entity();
        let other = this.clone();
        let last = this.clone();
        let actions = BarActions {
            reload: Box::new(move |_window, cx| drop(this.update(cx, |_panel, cx| cx.emit(Event::ReloadRequested)))),
            overwrite: Box::new(move |_window, cx| drop(other.update(cx, |panel, cx| panel.request_save(SaveKind::Overwrite, cx)))),
            discard: Box::new(move |_window, cx| drop(last.update(cx, |panel, cx| panel.discard_edits(cx)))),
        };
        let Some(bar) = saving::bar(&self.saving, actions, cx) else {
            return editor.into_any_element();
        };
        div().flex().flex_col().flex_1().min_h_0().child(bar).child(editor).into_any_element()
    }
}

impl Focusable for Panel {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        match &self.body {
            Body::View(view) => view.focus_handle(cx),
            Body::Edit(editor) => editor.focus_handle(cx),
            Body::Diff(diff) => diff.focus_handle(cx),
            Body::Empty => self.focus_handle.clone(),
        }
    }
}

impl Render for Panel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // the body scrolls and moves its caret without telling the panel; each frame catches up
        self.notice_state(cx);
        let body: AnyElement = match self.body.clone() {
            Body::Empty => div().into_any_element(),
            Body::View(view) => view.into_any_element(),
            Body::Edit(editor) => self.render_edit(editor, window, cx),
            Body::Diff(diff) if diff.read(cx).is_editable() => self.render_edit(diff, window, cx),
            Body::Diff(diff) => diff.into_any_element(),
        };
        body
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::time::Duration;

    use gpui::TestAppContext;

    use super::*;
    use crate::document::TextSource;
    use crate::syntax::lang;

    /// The parse jobs `panel` asks for, in order.
    fn jobs_of(panel: &Entity<Panel>, cx: &mut TestAppContext) -> Rc<RefCell<Vec<ParseJob>>> {
        let jobs: Rc<RefCell<Vec<ParseJob>>> = Rc::default();
        let seen = jobs.clone();
        cx.update(|cx| {
            cx.subscribe(panel, move |_, event: &Event, _| {
                if let Event::NeedParse(job) = event {
                    seen.borrow_mut().push(job.clone());
                }
            })
            .detach();
        });
        jobs
    }

    #[gpui::test]
    fn should_ask_for_the_newest_text_once_the_parse_in_flight_lands(cx: &mut TestAppContext) {
        // given
        let panel = cx.new(Panel::new);
        let jobs = jobs_of(&panel, cx);
        let policy = EditPolicy { save_delay: Duration::from_millis(500), limit: None };
        let source = Arc::new(TextSource::new("fn main() {}\n"));
        let content = Content::Lines { source, language: Some(lang("Rust")), edit: Some(policy) };
        panel.update(cx, |panel, cx| panel.handle(Cast::Open(content), cx));
        panel.update(cx, |panel, cx| {
            panel.handle(Cast::Type("a".to_owned()), cx);
            panel.handle(Cast::Type("b".to_owned()), cx);
        });
        let asked_while_parsing = jobs.borrow().len();

        // when
        let first = jobs.borrow()[0].clone();
        let (target, revision, syntax) = (first.target(), first.revision(), first.run().unwrap());
        panel.update(cx, |panel, cx| panel.handle(Cast::Parsed { target, revision, syntax }, cx));

        // then
        let revisions: Vec<u64> = jobs.borrow().iter().map(ParseJob::revision).collect();
        assert_eq!((asked_while_parsing, revisions), (1, vec![0, 2]));
    }

    /// The saves `panel` asks for, in order.
    fn saves_of(panel: &Entity<Panel>, cx: &mut TestAppContext) -> Rc<RefCell<Vec<SaveRequest>>> {
        let saves: Rc<RefCell<Vec<SaveRequest>>> = Rc::default();
        let seen = saves.clone();
        cx.update(|cx| {
            cx.subscribe(panel, move |_, event: &Event, _| {
                if let Event::SaveRequested(request) = event {
                    seen.borrow_mut().push(request.clone());
                }
            })
            .detach();
        });
        saves
    }

    /// A panel editing `text`, saved after a pause of 500 ms.
    fn editing(text: &str, cx: &mut TestAppContext) -> Entity<Panel> {
        let panel = cx.new(Panel::new);
        let policy = EditPolicy { save_delay: Duration::from_millis(500), limit: None };
        let content = Content::Lines { source: Arc::new(TextSource::new(text)), language: None, edit: Some(policy) };
        panel.update(cx, |panel, cx| panel.handle(Cast::Open(content), cx));
        panel
    }

    fn cast(panel: &Entity<Panel>, cast: Cast, cx: &mut TestAppContext) {
        panel.update(cx, |panel, cx| panel.handle(cast, cx));
    }

    fn typed(text: &str) -> Cast {
        Cast::Type(text.to_owned())
    }

    fn wait(duration: Duration, cx: &mut TestAppContext) {
        cx.executor().advance_clock(duration);
        cx.run_until_parked();
    }

    fn edit(bytes: std::ops::Range<u64>, text: &str) -> Edit {
        Edit { bytes, text: text.as_bytes().to_vec() }
    }

    #[gpui::test]
    fn should_request_one_save_after_the_pause(cx: &mut TestAppContext) {
        // given
        let panel = editing("text\n", cx);
        let saves = saves_of(&panel, cx);
        cast(&panel, typed("a"), cx);
        wait(Duration::from_millis(400), cx);
        cast(&panel, typed("b"), cx);
        wait(Duration::from_millis(400), cx);
        let during_typing = saves.borrow().len();

        // when
        wait(Duration::from_millis(200), cx);

        // then
        let expected = SaveRequest { base: 0, revision: 2, edits: vec![edit(0..0, "ab")], kind: SaveKind::Checked };
        assert_eq!((during_typing, saves.borrow().clone()), (0, vec![expected]));
        assert!(panel.read_with(cx, |panel, cx| panel.has_unsaved(cx) && panel.is_saving()));
    }

    #[gpui::test]
    fn should_hold_edits_typed_while_a_save_is_in_flight(cx: &mut TestAppContext) {
        // given
        let panel = editing("text\n", cx);
        let saves = saves_of(&panel, cx);
        cast(&panel, typed("a"), cx);
        cast(&panel, Cast::Command(Command::Flush), cx);
        cast(&panel, typed("b"), cx);
        cast(&panel, Cast::Command(Command::Flush), cx);
        let while_in_flight = saves.borrow().len();

        // when
        cast(&panel, Cast::Saved { revision: 1, source: Arc::new(TextSource::new("atext\n")) }, cx);

        // then
        let second = SaveRequest { base: 1, revision: 2, edits: vec![edit(1..1, "b")], kind: SaveKind::Checked };
        assert_eq!((while_in_flight, saves.borrow().last().cloned()), (1, Some(second)));
        cast(&panel, Cast::Saved { revision: 2, source: Arc::new(TextSource::new("abtext\n")) }, cx);
        assert!(panel.read_with(cx, |panel, cx| !panel.has_unsaved(cx) && !panel.is_saving()));
    }

    #[gpui::test]
    fn should_keep_edits_after_a_failed_save(cx: &mut TestAppContext) {
        // given
        let panel = editing("text\n", cx);
        let saves = saves_of(&panel, cx);
        cast(&panel, typed("a"), cx);
        cast(&panel, Cast::Command(Command::Flush), cx);

        // when
        cast(&panel, Cast::SaveFailed { revision: 1, reason: SaveFailure::Io("disk full".to_owned()) }, cx);
        let kept = panel.read_with(cx, |panel, cx| (panel.has_unsaved(cx), panel.is_saving(), panel.saving.problem().clone()));
        cast(&panel, typed("b"), cx);
        wait(Duration::from_millis(500), cx);

        // then
        assert_eq!(kept, (true, false, saving::Problem::Failed("disk full".to_owned())));
        let retried = SaveRequest { base: 0, revision: 2, edits: vec![edit(0..0, "ab")], kind: SaveKind::Checked };
        assert_eq!(saves.borrow().last().cloned(), Some(retried));
    }

    #[gpui::test]
    fn should_wait_for_the_user_after_a_conflict_and_overwrite_when_asked(cx: &mut TestAppContext) {
        // given
        let panel = editing("text\n", cx);
        let saves = saves_of(&panel, cx);
        cast(&panel, typed("a"), cx);
        cast(&panel, Cast::Command(Command::Flush), cx);
        cast(&panel, Cast::SaveFailed { revision: 1, reason: SaveFailure::Conflict(Resolve::ReloadOrOverwrite) }, cx);
        cast(&panel, typed("b"), cx);
        wait(Duration::from_millis(500), cx);
        cast(&panel, Cast::Command(Command::Flush), cx);
        let while_conflicting = saves.borrow().len();

        // when
        panel.update(cx, |panel, cx| panel.request_save(SaveKind::Overwrite, cx));

        // then
        let overwrite = SaveRequest { base: 0, revision: 2, edits: vec![edit(0..0, "ab")], kind: SaveKind::Overwrite };
        assert_eq!((while_conflicting, saves.borrow().last().cloned()), (1, Some(overwrite)));
    }

    #[gpui::test]
    fn should_keep_the_edits_when_the_saved_text_is_not_the_edited_one(cx: &mut TestAppContext) {
        // given
        let panel = editing("text\n", cx);
        cast(&panel, typed("a"), cx);
        cast(&panel, Cast::Command(Command::Flush), cx);

        // when
        cast(&panel, Cast::Saved { revision: 1, source: Arc::new(TextSource::new("another program's text\n")) }, cx);

        // then
        assert!(panel.read_with(cx, |panel, cx| panel.has_unsaved(cx) && !panel.is_saving()));
        let problem = panel.read_with(cx, |panel, _| (panel.saving.problem().clone(), panel.saving.choices()));
        assert_eq!(problem, (saving::Problem::Conflict(Resolve::Reload), saving::Choices::Reload));
    }

    #[gpui::test]
    fn should_offer_no_choice_while_an_overwrite_is_in_flight(cx: &mut TestAppContext) {
        // given
        let panel = editing("text\n", cx);
        cast(&panel, typed("a"), cx);
        cast(&panel, Cast::Command(Command::Flush), cx);
        cast(&panel, Cast::SaveFailed { revision: 1, reason: SaveFailure::Conflict(Resolve::ReloadOrOverwrite) }, cx);
        let before = panel.read_with(cx, |panel, _| panel.saving.choices());

        // when
        panel.update(cx, |panel, cx| panel.request_save(SaveKind::Overwrite, cx));

        // then
        let during = panel.read_with(cx, |panel, _| panel.saving.choices());
        assert_eq!((before, during), (saving::Choices::ReloadOrOverwrite, saving::Choices::Nothing));
    }

    #[gpui::test]
    fn should_refuse_to_overwrite_a_conflict_that_only_a_reload_resolves(cx: &mut TestAppContext) {
        // given
        let panel = editing("text\n", cx);
        let saves = saves_of(&panel, cx);
        cast(&panel, typed("a"), cx);
        cast(&panel, Cast::Command(Command::Flush), cx);
        cast(&panel, Cast::SaveFailed { revision: 1, reason: SaveFailure::Conflict(Resolve::Reload) }, cx);

        // when
        panel.update(cx, |panel, cx| panel.request_save(SaveKind::Overwrite, cx));

        // then
        let choices = panel.read_with(cx, |panel, _| panel.saving.choices());
        assert_eq!((saves.borrow().len(), choices), (1, saving::Choices::Reload));
    }

    /// How often `panel` told its host to close after a discard.
    fn discards_of(panel: &Entity<Panel>, cx: &mut TestAppContext) -> Rc<RefCell<usize>> {
        let discards: Rc<RefCell<usize>> = Rc::default();
        let seen = discards.clone();
        cx.update(|cx| {
            cx.subscribe(panel, move |_, event: &Event, _| {
                if let Event::DiscardRequested = event {
                    *seen.borrow_mut() += 1;
                }
            })
            .detach();
        });
        discards
    }

    #[gpui::test]
    fn should_offer_closing_without_saving_once_a_close_waits_on_a_failed_save(cx: &mut TestAppContext) {
        // given
        let panel = editing("text\n", cx);
        cast(&panel, typed("a"), cx);
        cast(&panel, Cast::Command(Command::Flush), cx);
        let offers = |panel: &Entity<Panel>, cx: &mut TestAppContext| panel.read_with(cx, |panel, _| panel.saving.offers_discard());
        cast(&panel, Cast::SaveFailed { revision: 1, reason: SaveFailure::Io("read-only".to_owned()) }, cx);
        let before_close = offers(&panel, cx);

        // when
        cast(&panel, Cast::CloseAsked, cx);

        // then
        let conflict = {
            cast(&panel, typed("b"), cx);
            cast(&panel, Cast::Command(Command::Flush), cx);
            cast(&panel, Cast::SaveFailed { revision: 2, reason: SaveFailure::Conflict(Resolve::Reload) }, cx);
            cast(&panel, Cast::CloseAsked, cx);
            offers(&panel, cx)
        };
        assert_eq!((before_close, offers(&panel, cx), conflict), (false, true, true));
    }

    #[gpui::test]
    fn should_not_offer_closing_without_saving_when_nothing_failed_or_a_save_is_in_flight(cx: &mut TestAppContext) {
        // given
        let panel = editing("text\n", cx);
        cast(&panel, typed("a"), cx);
        cast(&panel, Cast::CloseAsked, cx);
        let unsent = panel.read_with(cx, |panel, _| panel.saving.offers_discard());

        // when
        cast(&panel, Cast::Command(Command::Flush), cx);

        // then
        let in_flight = panel.read_with(cx, |panel, _| panel.saving.offers_discard());
        assert_eq!((unsent, in_flight), (false, false));
    }

    #[gpui::test]
    fn should_drop_the_edits_and_close_when_closing_without_saving(cx: &mut TestAppContext) {
        // given
        let panel = editing("text\n", cx);
        let saves = saves_of(&panel, cx);
        let discards = discards_of(&panel, cx);
        cast(&panel, typed("a"), cx);
        cast(&panel, Cast::Command(Command::Flush), cx);
        cast(&panel, Cast::SaveFailed { revision: 1, reason: SaveFailure::Io("read-only".to_owned()) }, cx);
        cast(&panel, Cast::CloseAsked, cx);

        // when
        panel.update(cx, |panel, cx| panel.discard_edits(cx));
        cast(&panel, Cast::Command(Command::Flush), cx);
        wait(Duration::from_millis(500), cx);

        // then
        assert!(panel.read_with(cx, |panel, cx| !panel.has_unsaved(cx)));
        assert_eq!((*discards.borrow(), saves.borrow().len()), (1, 1));
    }

    #[gpui::test]
    fn should_not_offer_closing_without_saving_after_the_close_was_abandoned(cx: &mut TestAppContext) {
        // given
        let panel = editing("text\n", cx);
        let offers = |panel: &Entity<Panel>, cx: &mut TestAppContext| panel.read_with(cx, |panel, _| panel.saving.offers_discard());
        let failed = |revision, cx: &mut TestAppContext| {
            cast(&panel, Cast::Command(Command::Flush), cx);
            cast(&panel, Cast::SaveFailed { revision, reason: SaveFailure::Io("read-only".to_owned()) }, cx);
            cast(&panel, Cast::CloseAsked, cx);
        };
        cast(&panel, typed("a"), cx);
        failed(1, cx);
        let asked = offers(&panel, cx);

        // when
        cast(&panel, Cast::CloseWithdrawn, cx);
        let withdrawn = offers(&panel, cx);
        failed(1, cx);
        cast(&panel, typed("b"), cx);
        let edited = offers(&panel, cx);

        // then
        assert_eq!((asked, withdrawn, edited), (true, false, false));
    }

    #[gpui::test]
    fn should_not_close_without_saving_when_no_close_waits(cx: &mut TestAppContext) {
        // given
        let panel = editing("text\n", cx);
        let discards = discards_of(&panel, cx);
        cast(&panel, typed("a"), cx);
        cast(&panel, Cast::Command(Command::Flush), cx);
        cast(&panel, Cast::SaveFailed { revision: 1, reason: SaveFailure::Io("read-only".to_owned()) }, cx);

        // when
        panel.update(cx, |panel, cx| panel.discard_edits(cx));

        // then
        assert!(panel.read_with(cx, |panel, cx| panel.has_unsaved(cx)));
        assert_eq!(*discards.borrow(), 0);
    }

    /// What `panel` asks of its change markers' host: how often the base, and the diff jobs, in order.
    fn change_asks_of(panel: &Entity<Panel>, cx: &mut TestAppContext) -> Rc<RefCell<(usize, Vec<crate::changes::ChangeDiffJob>)>> {
        let asks: Rc<RefCell<(usize, Vec<crate::changes::ChangeDiffJob>)>> = Rc::default();
        let seen = asks.clone();
        cx.update(|cx| {
            cx.subscribe(panel, move |_, event: &Event, _| match event {
                Event::NeedChangeBase => seen.borrow_mut().0 += 1,
                Event::NeedChangeDiff(job) => seen.borrow_mut().1.push(job.clone()),
                _ => {}
            })
            .detach();
        });
        asks
    }

    fn head(text: &str, head_id: &str) -> Cast {
        let head = Source::from_vec(text.as_bytes().to_vec());
        Cast::SetChangeBase { base: ChangeBase::Tracked { head, head_id: head_id.to_owned() } }
    }

    #[gpui::test]
    fn should_diff_the_changes_once_the_base_comes_and_again_after_typing_stops(cx: &mut TestAppContext) {
        // given
        let panel = cx.new(Panel::new);
        let asks = change_asks_of(&panel, cx);
        let policy = EditPolicy { save_delay: Duration::from_secs(10), limit: None };
        let content = Content::Lines { source: Arc::new(TextSource::new("one\n")), language: None, edit: Some(policy) };
        cast(&panel, Cast::Open(content), cx);
        cx.run_until_parked();
        let asked_for_base = asks.borrow().0;
        cast(&panel, head("one\n", "a"), cx);
        cast(&panel, head("one\n", "a"), cx);
        let first = asks.borrow().1.clone();
        let (revision, hunks) = (first[0].revision(), first[0].clone().run());
        cast(&panel, Cast::ChangeDiff { revision, hunks }, cx);

        // when
        cast(&panel, typed("x"), cx);
        wait(CHANGE_DIFF_DELAY / 2, cx);
        cast(&panel, typed("y"), cx);
        wait(CHANGE_DIFF_DELAY / 2, cx);
        let while_typing = asks.borrow().1.len();
        wait(CHANGE_DIFF_DELAY, cx);

        // then
        let revisions: Vec<u64> = asks.borrow().1.iter().map(crate::changes::ChangeDiffJob::revision).collect();
        assert_eq!((asked_for_base, first.len(), while_typing), (1, 1, 1));
        assert_eq!(revisions, [0, 2]);
    }

    #[gpui::test]
    fn should_count_edits_undone_back_to_the_saved_text_as_saved(cx: &mut TestAppContext) {
        // given
        let panel = editing("text\n", cx);
        let saves = saves_of(&panel, cx);
        cast(&panel, typed("a"), cx);
        cast(&panel, Cast::Command(Command::Undo), cx);

        // when
        wait(Duration::from_millis(500), cx);

        // then
        assert_eq!(saves.borrow().len(), 0);
        assert!(panel.read_with(cx, |panel, cx| !panel.has_unsaved(cx)));
    }
}
