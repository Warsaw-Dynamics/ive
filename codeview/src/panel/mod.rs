//! The panel: the one entity a host holds for any code it shows. The host casts messages to it, calls it for
//! answers, and hears its events; see the spec's "The interface: actor-style messages".

mod protocol;
mod state;

use std::sync::Arc;

use gpui::{
    AnyElement, App, AppContext, Context, Entity, EventEmitter, FocusHandle, Focusable, IntoElement, Render, Subscription,
    Window, div,
};

pub use protocol::{
    Cast, Content, DiffInput, DiffLayout, Edit, EditPolicy, Editable, Event, LineSource, ParseJob, ParseTarget, SaveFailure,
};
pub use state::PanelState;

use crate::code_view::CodeView;
use crate::diff::{DiffView, FileDiff};
use crate::host::Command;
use crate::syntax::{Language, Syntax};
use crate::text_editor::{SizeLimit, TextEditor, TextEditorEvent};

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
            // Task 11 fetches diff texts; Task 10 saves edits and takes a new policy
            Cast::SetTexts { .. } | Cast::Saved { .. } | Cast::SaveFailed { .. } | Cast::SetPolicy(_) => {}
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

    /// Edits not saved yet; none until Task 10, while the host saves edits itself.
    pub fn has_unsaved(&self) -> bool {
        false
    }

    /// The edited text; `None` outside Edit mode. A bridge for the scratch tab's own saving until Task 10.
    pub fn text(&self, cx: &App) -> Option<String> {
        match &self.body {
            Body::Edit(editor) => Some(editor.read(cx).text()),
            Body::Empty | Body::View(_) | Body::Diff(_) => None,
        }
    }

    /// Types `text` at the caret as the keyboard would, `\r` a line break; for the host's automation. Fails
    /// outside Edit mode or over the size limit.
    pub fn type_text(&mut self, text: &str, cx: &mut Context<Self>) -> Result<(), String> {
        let Body::Edit(editor) = &self.body else {
            return Err("the panel is not editable".to_owned());
        };
        editor.update(cx, |editor, cx| editor.type_text(text, cx)).map_err(|error| error.to_string())
    }

    /// New content replaces the body, except new text for the same mode, which keeps scroll and caret.
    fn open(&mut self, content: Content, cx: &mut Context<Self>) {
        match (content, self.body.clone()) {
            (Content::Lines { source, language, edit: None }, Body::View(view)) => {
                view.update(cx, |view, cx| view.replace_source(source, language, cx));
                self.revision += 1;
                self.request_parse(cx);
            }
            (Content::Lines { source, edit: Some(policy), .. }, Body::Edit(editor)) => {
                let text = whole_text(source.as_ref());
                editor.update(cx, |editor, cx| editor.reset(&text, size_limit(policy), cx));
                self.revision = editor.read(cx).revision();
            }
            (Content::Diff(input), Body::Diff(diff)) => diff.update(cx, |diff, cx| diff.set_files(input.files, cx)),
            (content, _) => self.build(content, cx),
        }
        self.apply_pending(cx);
    }

    fn build(&mut self, content: Content, cx: &mut Context<Self>) {
        self._body_events.clear();
        match content {
            Content::Lines { source, language, edit: None } => self.build_view(source, language, cx),
            Content::Lines { source, edit: Some(policy), .. } => self.build_edit(source.as_ref(), policy, cx),
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

    /// Edit mode holds the whole text for now: scratch sizes only, until the piece table (Task 7).
    fn build_edit(&mut self, source: &dyn LineSource, policy: EditPolicy, cx: &mut Context<Self>) {
        let text = whole_text(source);
        let editor = cx.new(|cx| TextEditor::new(&text, size_limit(policy), cx));
        self._body_events.push(cx.subscribe(&editor, Self::on_editor_event));
        self.revision = editor.read(cx).revision();
        self.body = Body::Edit(editor);
    }

    fn build_diff(&mut self, input: DiffInput, cx: &mut Context<Self>) {
        // editable diffs come with Task 13
        self.body = Body::Diff(cx.new(|cx| {
            let mut diff = DiffView::new(input.headers, cx);
            diff.set_files(input.files, cx);
            diff
        }));
    }

    fn on_editor_event(&mut self, editor: Entity<TextEditor>, event: &TextEditorEvent, cx: &mut Context<Self>) {
        match event {
            TextEditorEvent::Edited => {
                self.revision = editor.read(cx).revision();
                cx.emit(Event::Edited { revision: self.revision });
            }
            TextEditorEvent::Refused(reason) => cx.emit(Event::Refused(reason.clone())),
        }
    }

    fn request_parse(&self, cx: &mut Context<Self>) {
        let Body::View(view) = &self.body else {
            return;
        };
        if let Some(job) = view.read(cx).parse_job(self.revision) {
            cx.emit(Event::NeedParse(job));
        }
    }

    /// A parse of an older text is dropped; diff parses come with Task 11.
    fn parsed(&self, target: ParseTarget, revision: u64, syntax: Syntax, cx: &mut Context<Self>) {
        match (&self.body, target) {
            (Body::View(view), ParseTarget::Text) if revision == self.revision => {
                view.update(cx, |view, cx| view.set_syntax(syntax, cx));
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

    fn run(&self, command: Command, cx: &mut Context<Self>) {
        match &self.body {
            Body::View(view) => view.update(cx, |view, cx| view.run(command, cx)),
            Body::Edit(editor) => editor.update(cx, |editor, cx| editor.run(command, cx)),
            Body::Diff(diff) => diff.update(cx, |diff, cx| diff.run(command, cx)),
            Body::Empty => {}
        }
    }

    fn reveal_line(&self, line: usize, cx: &mut Context<Self>) {
        if let Body::View(view) = &self.body {
            view.update(cx, |view, cx| view.reveal_line(line, cx));
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
                PanelState { scroll: diff.scroll_position(), collapsed: diff.collapsed_paths(), ..PanelState::default() }
            }
        }
    }
}

/// The whole of `source` as text; invalid UTF-8 is replaced.
fn whole_text(source: &dyn LineSource) -> String {
    String::from_utf8_lossy(&source.bytes(0..source.byte_len())).into_owned()
}

/// A policy without a limit still needs one for the editor: none that a text in memory could reach.
fn size_limit(policy: EditPolicy) -> SizeLimit {
    policy.limit.unwrap_or(SizeLimit { max_bytes: usize::MAX, newline_bytes: 1 })
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
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // the body scrolls and moves its caret without telling the panel; each frame catches up
        self.notice_state(cx);
        let body: AnyElement = match &self.body {
            Body::Empty => div().into_any_element(),
            Body::View(view) => view.clone().into_any_element(),
            Body::Edit(editor) => editor.clone().into_any_element(),
            Body::Diff(diff) => diff.clone().into_any_element(),
        };
        body
    }
}
