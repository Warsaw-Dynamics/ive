//! The panel's find box: the query and its options, the search running off the UI thread, the matches found and
//! the current one; see the spec's "Find in file".

use std::ops::Range;
use std::rc::Rc;
use std::sync::Arc;

use gpui::{AnyElement, App, AppContext, Context, Entity, Focusable, KeyDownEvent, SharedString, Subscription, Task, Window};

use super::{Body, Panel};
use crate::changes::Toward;
use crate::find::bar::{BarActions, BarView, bar};
use crate::find::marks::FindMarks;
#[cfg(test)]
use crate::find::FindOptions;
use crate::find::{Compiled, FindMemory, FindOption, FindQuery, FindText, Matches};
use crate::host::{ClickListener, Command, Host, KeyOutcome, KeyScope};
use crate::line_field::{LineField, LineFieldEvent, Tone};

/// The find box's state; the field is its query.
pub(crate) struct Finder {
    field: Entity<LineField>,
    shown: Shown,
    query: FindQuery,
    /// where typing in the field searches from: where the box opened, or the match last stepped to
    anchor: usize,
    found: Found,
    /// counts searches, so only the last one's result lands
    generation: u64,
    search: Task<()>,
    keyboard: Keyboard,
    focus: PendingFocus,
    /// a step asked for while a search ran, taken once it lands
    step: PendingStep,
    /// the body's revision the matches shown are of
    landed: u64,
    _field_events: Subscription,
    focus_watch: FocusWatch,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Shown {
    Closed,
    Open,
}

enum Found {
    /// an empty query
    Nothing,
    /// a regex that does not compile, and why
    Bad(String),
    /// a search runs; the last one's matches stay shown while the text they are of has not changed
    Searching { kept: Option<Hits> },
    Ready(Hits),
}

/// Matches found and the current one; the marks the body draws are made once, so a step shares them.
#[derive(Clone)]
struct Hits {
    matches: Arc<Matches>,
    marks: FindMarks,
}

impl Hits {
    fn new(matches: Matches, current: Option<usize>) -> Self {
        let marks = FindMarks { ranges: Arc::from(matches.ranges.as_slice()), lines: Arc::from(matches.lines.as_slice()), current };
        Self { matches: Arc::new(matches), marks }
    }

    fn at(&self, current: usize) -> Self {
        Self { matches: self.matches.clone(), marks: FindMarks { current: Some(current), ..self.marks.clone() } }
    }

    fn current(&self) -> Option<usize> {
        self.marks.current
    }
}

/// Where the keyboard is, as the field's focus events said.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Keyboard {
    Elsewhere,
    InField,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PendingStep {
    Nothing,
    Step(Toward),
}

/// A focus change waiting for the next frame, which has the window.
#[derive(Clone, Copy, PartialEq, Eq)]
enum PendingFocus {
    Nothing,
    Field,
    Body,
}

enum FocusWatch {
    Unwatched,
    Watched { _focus: Subscription, _blur: Subscription },
}

/// Which match is current once a search lands.
#[derive(Clone, Copy)]
enum Landing {
    /// the first at or after the byte, selected and scrolled to
    Show(usize),
    /// the last before the byte, selected and scrolled to
    ShowBefore(usize),
    /// the first at or after the byte; the body's selection and scroll stay
    Keep(usize),
}

impl Landing {
    fn current(self, matches: &Matches) -> Option<usize> {
        match self {
            Self::Show(at) | Self::Keep(at) => matches.first_from(at),
            Self::ShowBefore(at) => matches.first_from(at).map(|after| matches.stepped(after, Toward::Previous)),
        }
    }

    fn shows(self) -> bool {
        matches!(self, Self::Show(_) | Self::ShowBefore(_))
    }
}

/// The box as tests see it.
#[cfg(test)]
pub(crate) struct FindView {
    pub open: bool,
    pub query: String,
    pub count: String,
    pub options: FindOptions,
    /// the body's selected text
    pub selected: Option<String>,
    /// the body draws matches
    pub marked: bool,
}

impl Finder {
    pub(crate) fn new(cx: &mut Context<Panel>) -> Self {
        let field = cx.new(|cx| LineField::new("Find", "find-query", cx).with_find_options());
        let events = cx.subscribe(&field, |panel, _field, event: &LineFieldEvent, cx| panel.on_field_event(event, cx));
        Self {
            field,
            shown: Shown::Closed,
            query: FindQuery::default(),
            anchor: 0,
            found: Found::Nothing,
            generation: 0,
            search: Task::ready(()),
            keyboard: Keyboard::Elsewhere,
            focus: PendingFocus::Nothing,
            step: PendingStep::Nothing,
            landed: 0,
            _field_events: events,
            focus_watch: FocusWatch::Unwatched,
        }
    }

    pub(crate) fn is_typing(&self) -> bool {
        self.shown == Shown::Open && self.keyboard == Keyboard::InField
    }

    /// The matches shown: found, or kept from the last search while the next one runs over the same text.
    fn hits(&self) -> Option<&Hits> {
        match &self.found {
            Found::Ready(hits) | Found::Searching { kept: Some(hits) } => Some(hits),
            Found::Nothing | Found::Bad(_) | Found::Searching { kept: None } => None,
        }
    }

    fn count(&self) -> String {
        if let Found::Bad(_) = self.found {
            return "Bad regex".to_owned();
        }
        self.hits().map_or_else(String::new, |hits| hits.matches.count_text(hits.current()))
    }

    fn tone(&self) -> Tone {
        let none = self.hits().is_some_and(|hits| hits.matches.ranges.is_empty());
        if none || matches!(self.found, Found::Bad(_)) { Tone::Error } else { Tone::Plain }
    }

    fn marks(&self) -> Option<FindMarks> {
        self.hits().map(|hits| hits.marks.clone())
    }

    fn current_range(&self) -> Option<Range<usize>> {
        self.hits().and_then(|hits| hits.marks.current_range())
    }
}

impl Panel {
    /// The find box's commands, from a key or the host.
    pub(crate) fn run_find(&mut self, command: Command, cx: &mut Context<Self>) {
        match command {
            Command::Find => self.open_find(cx),
            Command::FindNext => self.step_find(Toward::Next, cx),
            Command::FindPrevious => self.step_find(Toward::Previous, cx),
            _ => {}
        }
    }

    /// Opens the box with the selection (on one line) or the word at the caret, else the last query, all of it
    /// selected, and goes to the first match from there; when open, gives the field the keyboard back.
    fn open_find(&mut self, cx: &mut Context<Self>) {
        if self.finds_nothing() {
            return;
        }
        let reopened = self.finder.shown == Shown::Open;
        // the current match is selected: only another selection is new
        let current = self.finder.current_range();
        let picked = self.body_find_prefill(cx).filter(|_| !reopened || Some(self.body_find_anchor(cx)) != current);
        self.finder.focus = PendingFocus::Field;
        if reopened && picked.is_none() {
            return self.finder.field.update(cx, |field, cx| field.select_all(cx));
        }
        let memory = cx.default_global::<FindMemory>().query.clone();
        let (text, anchor) = picked.unwrap_or_else(|| (memory.text.clone(), self.body_find_anchor(cx).start));
        self.show_find(FindQuery { text, options: memory.options }, cx);
        self.finder.anchor = anchor;
        self.search_find(Landing::Show(anchor), cx);
    }

    fn show_find(&mut self, query: FindQuery, cx: &mut Context<Self>) {
        let text = query.text.clone();
        self.finder.query = query;
        self.finder.shown = Shown::Open;
        self.finder.field.update(cx, |field, cx| field.set_text(&text, cx));
    }

    /// The next or previous match, going round; with the box closed, it opens with the last query (the keyboard
    /// stays in the code) and goes to the match after (or before) the caret.
    fn step_find(&mut self, toward: Toward, cx: &mut Context<Self>) {
        if self.finds_nothing() {
            return;
        }
        if self.finder.shown == Shown::Closed {
            let query = cx.default_global::<FindMemory>().query.clone();
            let anchor = self.body_find_anchor(cx);
            self.show_find(query, cx);
            self.finder.anchor = anchor.start;
            let landing = match toward {
                Toward::Next => Landing::Show(anchor.end),
                Toward::Previous => Landing::ShowBefore(anchor.start),
            };
            return self.search_find(landing, cx);
        }
        let hits = match &self.finder.found {
            Found::Ready(hits) => hits.clone(),
            Found::Searching { .. } => {
                self.finder.step = PendingStep::Step(toward);
                return;
            }
            Found::Nothing | Found::Bad(_) => return,
        };
        let matches = &hits.matches;
        let Some(next) = hits.current().map(|current| matches.stepped(current, toward)).or_else(|| matches.first_from(self.finder.anchor)) else {
            return;
        };
        self.finder.anchor = matches.ranges[next].start;
        self.finder.found = Found::Ready(hits.at(next));
        self.show_found(cx);
    }

    /// Closes the box: the marks go, the current match stays selected, the code takes the keyboard.
    pub(crate) fn close_find(&mut self, cx: &mut Context<Self>) {
        self.finder.shown = Shown::Closed;
        self.finder.generation += 1;
        self.finder.found = Found::Nothing;
        self.finder.focus = PendingFocus::Body;
        self.body_set_find_marks(None, cx);
        cx.notify();
    }

    fn on_field_event(&mut self, event: &LineFieldEvent, cx: &mut Context<Self>) {
        match event {
            LineFieldEvent::Edited => {
                self.finder.query.text = self.finder.field.read(cx).text().to_owned();
                self.remember_query(cx);
                self.search_find(Landing::Show(self.finder.anchor), cx);
            }
            LineFieldEvent::Submitted(toward) => self.step_find(*toward, cx),
            LineFieldEvent::Toggled(option) => self.toggle_find_option(*option, cx),
        }
    }

    pub(crate) fn toggle_find_option(&mut self, option: FindOption, cx: &mut Context<Self>) {
        self.finder.query.options = self.finder.query.options.toggled(option);
        self.remember_query(cx);
        self.search_find(Landing::Show(self.finder.anchor), cx);
    }

    fn remember_query(&self, cx: &mut Context<Self>) {
        cx.default_global::<FindMemory>().query = self.finder.query.clone();
    }

    /// Types `text` over the query, as the keyboard would.
    pub(crate) fn type_find_query(&mut self, text: &str, cx: &mut Context<Self>) {
        self.finder.field.update(cx, |field, cx| field.type_over(text, cx));
    }

    /// The text changed (an edit, a reload, new diff files, another layout or side): the matches follow it, and
    /// the selection and scroll stay.
    pub(crate) fn refind(&mut self, cx: &mut Context<Self>) {
        if self.finder.shown == Shown::Closed {
            return;
        }
        let from = self.finder.current_range().map_or(self.finder.anchor, |current| current.start);
        self.search_find(Landing::Keep(from), cx);
    }

    /// Another side of a diff is searched: the current match is the first from the place the user clicked.
    pub(crate) fn refind_from_anchor(&mut self, cx: &mut Context<Self>) {
        if self.finder.shown == Shown::Closed {
            return;
        }
        let anchor = self.body_find_anchor(cx).start;
        self.search_find(Landing::Keep(anchor), cx);
    }

    /// Searches the body's text for the query on the background executor; a result lands only while it is of the
    /// last search and the text it searched is still the body's.
    fn search_find(&mut self, landing: Landing, cx: &mut Context<Self>) {
        self.finder.generation += 1;
        self.finder.step = PendingStep::Nothing;
        let regex = match self.finder.query.compile() {
            Compiled::Empty => return self.set_found(Found::Nothing, cx),
            Compiled::Bad(reason) => return self.set_found(Found::Bad(reason), cx),
            Compiled::Ready(regex) => regex,
        };
        let (revision, text) = self.body_find_text(cx);
        // the matches shown stay until this search lands, unless the text moved under them
        let kept = self.finder.hits().filter(|_| self.finder.landed == revision).cloned();
        self.set_found(Found::Searching { kept }, cx);
        let generation = self.finder.generation;
        self.finder.search = cx.spawn(async move |panel, cx| {
            let matches = cx.background_executor().spawn(async move { text.search(&regex) }).await;
            panel.update(cx, |panel, cx| panel.searched(Searched { generation, revision, landing, matches }, cx)).ok();
        });
    }

    fn searched(&mut self, searched: Searched, cx: &mut Context<Self>) {
        if searched.generation != self.finder.generation || searched.revision != self.body_find_revision(cx) {
            return;
        }
        let current = searched.landing.current(&searched.matches);
        self.finder.landed = searched.revision;
        self.finder.found = Found::Ready(Hits::new(searched.matches, current));
        if let PendingStep::Step(toward) = std::mem::replace(&mut self.finder.step, PendingStep::Nothing) {
            return self.step_find(toward, cx);
        }
        if searched.landing.shows() {
            return self.show_found(cx);
        }
        self.set_found_marks(cx);
    }

    fn set_found(&mut self, found: Found, cx: &mut Context<Self>) {
        self.finder.found = found;
        self.set_found_marks(cx);
    }

    /// The body draws the matches, and the field's border says whether there are any.
    fn set_found_marks(&mut self, cx: &mut Context<Self>) {
        let (marks, tone) = (self.finder.marks(), self.finder.tone());
        self.body_set_find_marks(marks, cx);
        self.finder.field.update(cx, |field, cx| field.set_tone(tone, cx));
        cx.notify();
    }

    /// Marks the matches, and selects and scrolls to the current one.
    fn show_found(&mut self, cx: &mut Context<Self>) {
        self.set_found_marks(cx);
        if let Some(range) = self.finder.current_range() {
            self.body_show_match(range, cx);
        }
    }

    /// Keys reaching the panel from its body or the field: the find commands, the Markdown switch, and Escape while
    /// the box is open.
    pub(crate) fn on_find_key(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if let KeyOutcome::Command(Command::ToggleMarkdownView) = Host::key_outcome(event, KeyScope::Editor, cx) {
            cx.stop_propagation();
            self.toggle_markdown_view(cx);
            self.notice_state(cx);
            return cx.notify();
        }
        if let KeyOutcome::Command(command) = Host::key_outcome(event, KeyScope::Editor, cx)
            && command.is_find()
        {
            cx.stop_propagation();
            self.run_find(command, cx);
            return self.settle_find_focus(window, cx);
        }
        let escape = event.keystroke.key == "escape" && !event.keystroke.modifiers.modified();
        if escape && self.finder.shown == Shown::Open {
            cx.stop_propagation();
            self.close_find(cx);
            self.settle_find_focus(window, cx);
        }
    }

    /// Gives the field or the body the keyboard as asked since the last frame, and follows the field's focus.
    pub(crate) fn settle_find_focus(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let FocusWatch::Unwatched = self.finder.focus_watch {
            let handle = self.finder.field.focus_handle(cx);
            let focus = cx.on_focus(&handle, window, |panel, _window, _cx| panel.finder.keyboard = Keyboard::InField);
            let blur = cx.on_blur(&handle, window, |panel, _window, _cx| panel.finder.keyboard = Keyboard::Elsewhere);
            self.finder.focus_watch = FocusWatch::Watched { _focus: focus, _blur: blur };
        }
        match std::mem::replace(&mut self.finder.focus, PendingFocus::Nothing) {
            // typing sent right after (MCP's send_text) goes to the field, before its focus event lands
            PendingFocus::Field => {
                window.focus(&self.finder.field.focus_handle(cx), cx);
                self.finder.keyboard = Keyboard::InField;
            }
            PendingFocus::Body => window.focus(&self.focus_handle(cx), cx),
            PendingFocus::Nothing => {}
        }
    }

    /// Types into the find box's query while it has the keyboard (MCP's `send_text`), else at the body's caret.
    pub(crate) fn type_where_the_keyboard_is(&mut self, text: &str, cx: &mut Context<Self>) {
        if self.finder.is_typing() {
            return self.type_find_query(text, cx);
        }
        self.type_text(text, cx);
    }

    pub(crate) fn find_is_open(&self) -> bool {
        self.finder.shown == Shown::Open
    }

    /// The box over the body, while open.
    pub(crate) fn find_bar(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        if self.finder.shown == Shown::Closed {
            return None;
        }
        let detail = match &self.finder.found {
            Found::Bad(reason) => Some(SharedString::from(reason.clone())),
            Found::Nothing | Found::Searching { .. } | Found::Ready { .. } => self.body_find_detail(cx),
        };
        let view = BarView { field: self.finder.field.clone(), count: self.finder.count(), detail, options: self.finder.query.options };
        Some(bar(view, self.find_actions(cx), cx))
    }

    fn find_actions(&self, cx: &mut Context<Self>) -> BarActions {
        let on = |run: fn(&mut Panel, &mut Context<Panel>), cx: &mut Context<Self>| -> ClickListener {
            let panel = cx.weak_entity();
            Rc::new(move |_, window, cx| {
                drop(panel.update(cx, |panel, cx| {
                    run(panel, cx);
                    panel.settle_find_focus(window, cx);
                }));
            })
        };
        let toggling = cx.weak_entity();
        BarActions {
            focus_query: on(|panel, cx| {
                panel.finder.focus = PendingFocus::Field;
                cx.notify();
            }, cx),
            previous: on(|panel, cx| panel.step_find(Toward::Previous, cx), cx),
            next: on(|panel, cx| panel.step_find(Toward::Next, cx), cx),
            toggle: Rc::new(move |option, _window, cx| drop(toggling.update(cx, |panel, cx| panel.toggle_find_option(option, cx)))),
            close: on(Panel::close_find, cx),
        }
    }

    /// Registers click target `find:open`, as Ctrl+F, for a panel with something to search.
    pub(crate) fn register_find_open(&self, cx: &mut Context<Self>) {
        if self.finds_nothing() {
            return;
        }
        let panel = cx.weak_entity();
        let open = move |_: &gpui::ClickEvent, window: &mut Window, cx: &mut App| {
            drop(panel.update(cx, |panel, cx| {
                panel.open_find(cx);
                panel.settle_find_focus(window, cx);
            }));
        };
        drop(Host::click_target("find:open".to_owned(), open, cx));
    }

    #[cfg(test)]
    pub(crate) fn find_view(&self, cx: &App) -> FindView {
        FindView {
            open: self.finder.shown == Shown::Open,
            query: self.finder.field.read(cx).text().to_owned(),
            count: self.finder.count(),
            options: self.finder.query.options,
            selected: self.body_selected_text(cx),
            marked: self.body_has_find_marks(cx),
        }
    }

    #[cfg(test)]
    pub(crate) fn select_find_query(&mut self, cx: &mut Context<Self>) {
        self.finder.field.update(cx, |field, cx| field.select_all(cx));
    }

    #[cfg(test)]
    pub(crate) fn search_diff_side_for_tests(&mut self, side: crate::diff::side_by_side::Side, cx: &mut Context<Self>) {
        if let Body::Diff(diff) = &self.body {
            diff.update(cx, |diff, cx| diff.search_side(side, cx));
        }
    }

    #[cfg(test)]
    pub(crate) fn diff_ticks_for_tests(&self, cx: &App) -> Vec<crate::scrollbar::Tick> {
        match &self.body {
            Body::Diff(diff) => diff.read(cx).find_ticks().to_vec(),
            Body::Edit(_) | Body::View(_) | Body::Empty => Vec::new(),
        }
    }

    /// As if the field had the keyboard: tests have no window to focus it in.
    #[cfg(test)]
    pub(crate) fn find_takes_typing_for_tests(&mut self) {
        self.finder.keyboard = Keyboard::InField;
    }
}

/// A search's result, with what it was searched for.
struct Searched {
    generation: u64,
    revision: u64,
    landing: Landing,
    matches: Matches,
}

/// What the panel asks of its body; each body answers in its own terms.
impl Panel {
    /// Nothing to search, or a panel that never takes the keyboard (a scratch's read-only preview).
    fn finds_nothing(&self) -> bool {
        matches!(self.body, Body::Empty) || matches!(self.pointer, super::Pointer::Ignored)
    }

    /// The text the find box searches: Rendered's while it shows, else the body's.
    pub(super) fn body_find_text(&self, cx: &App) -> (u64, FindText) {
        if let Some(rendered) = self.shown_rendered() {
            return (self.rendered_revision(), rendered.read(cx).find_text());
        }
        self.text_find_text(cx)
    }

    /// The body's text, whether Rendered shows or not.
    pub(super) fn text_find_text(&self, cx: &App) -> (u64, FindText) {
        match &self.body {
            Body::Edit(editor) => editor.read(cx).find_text(),
            Body::View(view) => (self.revision, view.read(cx).find_text()),
            Body::Diff(diff) => diff.read(cx).find_text(),
            Body::Empty => (0, FindText::Owned(Arc::default())),
        }
    }

    fn body_find_revision(&self, cx: &App) -> u64 {
        if self.shown_rendered().is_some() {
            return self.rendered_revision();
        }
        match &self.body {
            Body::Edit(editor) => editor.read(cx).revision(),
            Body::View(_) | Body::Empty => self.revision,
            Body::Diff(diff) => diff.read(cx).find_revision(),
        }
    }

    /// The selection on one line, or the word at the caret, and the byte it starts at; not when it holds invalid
    /// UTF-8, which the query (text) would hold as U+FFFD and never match.
    fn body_find_prefill(&self, cx: &App) -> Option<(String, usize)> {
        if let Some(rendered) = self.shown_rendered() {
            return rendered.read(cx).find_prefill();
        }
        let picked = match &self.body {
            Body::Edit(editor) => editor.read(cx).find_prefill(),
            Body::View(view) => view.read(cx).find_prefill(),
            Body::Diff(diff) => diff.read(cx).find_prefill(),
            Body::Empty => None,
        };
        picked.filter(|(text, _)| !text.contains(char::REPLACEMENT_CHARACTER))
    }

    /// The selection's bytes, or the caret's, or the top line's start.
    fn body_find_anchor(&self, cx: &App) -> Range<usize> {
        if let Some(rendered) = self.shown_rendered() {
            return rendered.read(cx).find_anchor();
        }
        match &self.body {
            Body::Edit(editor) => editor.read(cx).find_anchor(),
            Body::View(view) => view.read(cx).find_anchor(),
            Body::Diff(diff) => diff.read(cx).find_anchor(),
            Body::Empty => 0..0,
        }
    }

    fn body_show_match(&self, bytes: Range<usize>, cx: &mut Context<Self>) {
        if let Some(rendered) = self.shown_rendered() {
            return rendered.update(cx, |view, cx| view.show_match(bytes, cx));
        }
        match &self.body {
            Body::Edit(editor) => editor.update(cx, |editor, cx| editor.show_match(bytes, cx)),
            Body::View(view) => view.update(cx, |view, cx| view.show_match(bytes, cx)),
            Body::Diff(diff) => diff.update(cx, |diff, cx| diff.show_match(bytes, cx)),
            Body::Empty => {}
        }
    }

    fn body_set_find_marks(&self, marks: Option<FindMarks>, cx: &mut Context<Self>) {
        if let Some(rendered) = self.shown_rendered() {
            return rendered.update(cx, |view, cx| view.set_find_marks(marks, cx));
        }
        match &self.body {
            Body::Edit(editor) => editor.update(cx, |editor, cx| editor.set_find_marks(marks, cx)),
            Body::View(view) => view.update(cx, |view, cx| view.set_find_marks(marks, cx)),
            Body::Diff(diff) => diff.update(cx, |diff, cx| diff.set_find_marks(marks, cx)),
            Body::Empty => {}
        }
    }

    /// Which side of a side-by-side diff is searched, for the count's tooltip.
    fn body_find_detail(&self, cx: &App) -> Option<SharedString> {
        match &self.body {
            Body::Diff(diff) => diff.read(cx).find_detail(),
            Body::Edit(_) | Body::View(_) | Body::Empty => None,
        }
    }

    #[cfg(test)]
    fn body_selected_text(&self, cx: &App) -> Option<String> {
        if let Some(rendered) = self.shown_rendered() {
            return rendered.read(cx).selected_text();
        }
        match &self.body {
            Body::Edit(editor) => editor.read(cx).selected_text(),
            Body::View(view) => view.read(cx).selected_text(),
            Body::Diff(diff) => diff.read(cx).selected_text(),
            Body::Empty => None,
        }
    }

    #[cfg(test)]
    fn body_has_find_marks(&self, cx: &App) -> bool {
        if let Some(rendered) = self.shown_rendered() {
            return rendered.read(cx).has_find_marks();
        }
        match &self.body {
            Body::Edit(editor) => editor.read(cx).has_find_marks(),
            Body::View(view) => view.read(cx).has_find_marks(),
            Body::Diff(diff) => diff.read(cx).has_find_marks(),
            Body::Empty => false,
        }
    }
}
