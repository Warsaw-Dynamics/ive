//! The code view entity: a read-only view of a file that draws only the lines on screen, highlighted once parsed.

use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::ops::Range;
use std::rc::Rc;
use std::sync::Arc;

use gpui::{
    App, Context, CursorStyle, DispatchPhase, Div, Entity, FocusHandle, Focusable, KeyDownEvent, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, Pixels, Point, SharedString, Stateful, StyledText, Window, canvas, div, prelude::*, px, rgb,
};

use crate::colors::{ColorKey, ResolvedScheme};
use crate::document::DisplayLine;
use crate::host::{ClipboardTarget, Command, Host, KeyOutcome, KeyScope};
use crate::line_view::{self, LineScroll, Metrics};
use crate::panel::{Failed, LineSource, ParseJob, ParseTarget, Previous};
use crate::position::ScrollPosition;
use crate::scrollbar::{ScrollState, horizontal_scrollbar, vertical_scrollbar};
use crate::style::ViewStyle;
use crate::syntax::{self, Cancel, Highlight, Language, Syntax};
use crate::text_editor::Motion;
use crate::text_selection::{TextSelection, whole_line, word_around};

/// Columns kept between the pointer and the view's edge when a selection drag scrolls sideways.
const DRAG_MARGIN_CHARS: f32 = 2.;

/// A read-only view of a document that draws only the lines on screen.
/// It does its own scrolling instead of using GPUI's list: with 10M lines the list's f32 pixel
/// positions (200M px) are only accurate to 16 px, so rows would overlap; here the position is a
/// line number plus a fraction.
pub struct CodeView {
    state: State,
    focus_handle: FocusHandle,
    /// the mouse selects text; off, the view only shows it
    selectable: bool,
}

/// What the view shows and where: everything but its gpui handles.
struct State {
    source: Arc<dyn LineSource>,
    language: Option<Language>,
    scroll: Rc<LineScroll>,
    scrollbars: ScrollState,
    /// a bar above the text: "deleted on disk"
    notice: Option<SharedString>,
    /// set once the background parse finished; plain text until then
    syntax: Option<Syntax>,
    /// stops the background parse when this view goes away or its text is re-read
    cancel: Cancel,
    highlights: HighlightCache,
    /// what the mouse or Select All marked; dropped when the text is re-read, since it holds file offsets
    selection: Option<TextSelection>,
    /// a left-button drag that started in the text goes on
    dragging: bool,
    /// where the last frame put the view, for the mouse
    placed: Rc<Cell<Option<Placed>>>,
    /// the line `file:42` went to (from 0), drawn marked until the next press
    marked: Option<usize>,
}

/// The view's top-left corner in the window and its font metrics, as last painted.
#[derive(Clone, Copy)]
struct Placed {
    origin: Point<Pixels>,
    metrics: Metrics,
}

/// The line under the pointer and the file offset nearest to it.
struct Hit {
    index: usize,
    line: DisplayLine,
    column: usize,
    offset: usize,
}

impl Drop for State {
    fn drop(&mut self) {
        self.cancel.cancel();
    }
}

impl CodeView {
    /// A view of `source`, coloured as `language` once parsed, where the mouse selects text.
    pub fn new(source: Arc<dyn LineSource>, language: Option<Language>, cx: &mut Context<Self>) -> Self {
        Self { state: State::new(source, language), focus_handle: cx.focus_handle(), selectable: true }
    }

    /// This view without selecting by mouse.
    pub fn unselectable(self) -> Self {
        Self { selectable: false, ..self }
    }

    /// Scrolls line `line` (from 1; past the end, the last) a third of the way down the view and marks it.
    pub fn reveal_line(&mut self, line: usize, cx: &mut Context<Self>) {
        self.state.reveal_line(line);
        cx.notify();
    }

    pub fn scroll_position(&self) -> ScrollPosition {
        self.state.scroll_position()
    }

    /// Scrolls to `position`, kept inside the text.
    pub fn restore_scroll(&mut self, position: ScrollPosition, cx: &mut Context<Self>) {
        self.state.scroll.set_position(position);
        cx.notify();
    }

    /// The parse this text needs, tagged `revision`, if it gets highlighting at all: a known language, at most
    /// `MAX_HIGHLIGHT_BYTES`.
    pub fn parse_job(&self, revision: u64) -> Option<ParseJob> {
        self.state.parse_job(revision)
    }

    pub fn set_syntax(&mut self, syntax: Syntax, cx: &mut Context<Self>) {
        self.state.set_syntax(syntax);
        cx.notify();
    }

    /// Shows new text (the file changed on disk): the scroll stays, clamped; plain until parsed again.
    pub fn replace_source(&mut self, source: Arc<dyn LineSource>, language: Option<Language>, cx: &mut Context<Self>) {
        self.state.replace_source(source, language);
        cx.notify();
    }

    /// The file was removed or renamed away; the text stays until it comes back.
    pub fn mark_deleted(&mut self, cx: &mut Context<Self>) {
        self.state.mark_deleted();
        cx.notify();
    }

    /// Copy, Select All, and the moves to the start or end of the text; the view ignores other commands.
    pub fn run(&mut self, command: Command, cx: &mut Context<Self>) {
        match command {
            Command::Copy(target) => self.copy(target, cx),
            Command::SelectAll => self.select_all(cx),
            Command::Move(Motion::DocumentStart, _) => self.state.scroll.reveal_line(0),
            Command::Move(Motion::DocumentEnd, _) => self.state.reveal_end(),
            _ => return,
        }
        cx.notify();
    }

    pub fn select_all(&mut self, cx: &mut Context<Self>) {
        self.state.selection = Some(TextSelection::of(0..self.state.source.byte_len() as usize));
        cx.notify();
    }

    /// Puts the selected text in `target`; a selection too large to copy is refused, with the reason on stderr.
    pub fn copy(&self, target: ClipboardTarget, cx: &mut App) {
        let Some(selection) = self.state.selection else {
            return;
        };
        match selection.text(&self.state.text()) {
            Ok(text) if text.is_empty() => {}
            Ok(text) => Host::write_clipboard(target, text, cx),
            Err(reason) => eprintln!("selection not copied: {reason}"),
        }
    }

    /// A left press takes the keys, so the host's Copy and Select All reach the view.
    fn on_mouse_down(&mut self, event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.focus_handle, cx);
        self.state.press(event);
        cx.notify();
    }

    fn on_mouse_up(&mut self, _event: &MouseUpEvent, _window: &mut Window, _cx: &mut Context<Self>) {
        self.state.dragging = false;
    }

    /// The host's Copy and Select All; other keys go on to the window's bindings.
    fn on_key_down(&mut self, event: &KeyDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        match Host::key_outcome(event, KeyScope::Viewer, cx) {
            KeyOutcome::Command(Command::Copy(target)) => self.copy(target, cx),
            KeyOutcome::Command(Command::SelectAll) => self.select_all(cx),
            KeyOutcome::Handled => {}
            // a read-only view has no other commands; a window action is the host's to run
            KeyOutcome::Command(_) | KeyOutcome::Foreign | KeyOutcome::Unbound => return,
        }
        cx.stop_propagation();
    }
}

impl State {
    fn new(source: Arc<dyn LineSource>, language: Option<Language>) -> Self {
        let scroll = Rc::new(LineScroll::new(source.line_count()));
        let scrollbars = ScrollState::new(scroll.clone());
        Self {
            source,
            language,
            scroll,
            scrollbars,
            notice: None,
            syntax: None,
            cancel: Cancel::default(),
            highlights: HighlightCache::default(),
            selection: None,
            dragging: false,
            placed: Rc::default(),
            marked: None,
        }
    }

    /// The text as stored; borrowed from a file or memory, so cheap for every source the host has today.
    fn text(&self) -> Cow<'_, [u8]> {
        self.source.bytes(0..self.source.byte_len())
    }

    fn last_line(&self) -> usize {
        self.source.line_count().saturating_sub(1)
    }

    fn reveal_line(&mut self, line: usize) {
        let index = line.saturating_sub(1).min(self.last_line());
        let top = (index as f64 - self.scroll.viewport_lines() / 3.).max(0.).floor();
        self.scroll.set_position(ScrollPosition { top_line: top, left: 0. });
        self.marked = Some(index);
    }

    /// Scrolls to the last line, where a log's newest lines are.
    fn reveal_end(&self) {
        self.scroll.reveal_line(self.last_line());
    }

    /// The marked line, from 1.
    #[cfg(test)]
    fn marked_line(&self) -> Option<usize> {
        self.marked.map(|index| index + 1)
    }

    /// A left press: places the start of a selection, selects a word on a double click and a line on a triple
    /// one; with Shift, extends the selection.
    fn press(&mut self, event: &MouseDownEvent) {
        self.marked = None;
        let Some(hit) = self.hit(event.position) else {
            return;
        };
        let bytes = self.text();
        let selection = match event.click_count {
            2 => TextSelection::of(word_around(&bytes, &hit.line, hit.offset)),
            3.. => TextSelection::of(whole_line(&bytes, &hit.line)),
            _ if event.modifiers.shift => self.selection.unwrap_or(TextSelection::at(hit.offset)).extended_to(hit.offset),
            _ => TextSelection::at(hit.offset),
        };
        self.selection = Some(selection);
        self.dragging = true;
    }

    /// A move of the pointer anywhere in the window; while a drag that began in the text goes on, the selection
    /// follows it, and past the view's edges scrolls lines and columns into view. False when nothing changed.
    fn drag_to(&mut self, point: Point<Pixels>) -> bool {
        let Some(selection) = self.selection.filter(|_| self.dragging) else {
            return false;
        };
        let (Some(hit), Some(placed)) = (self.hit(point), self.placed.get()) else {
            return false;
        };
        self.scroll.reveal_line(hit.index);
        let char_width = placed.metrics.char_width;
        self.scroll.reveal_x(f64::from(f32::from(char_width * hit.column as f32)), f64::from(f32::from(char_width * DRAG_MARGIN_CHARS)));
        self.selection = Some(selection.extended_to(hit.offset));
        true
    }

    /// The line under window point `point` (the first or last one above or below the text) and the column
    /// boundary nearest to it.
    fn hit(&self, point: Point<Pixels>) -> Option<Hit> {
        let Placed { origin, metrics } = self.placed.get()?;
        let row = f64::from(f32::from(point.y - origin.y) / f32::from(metrics.line_height)) + self.scroll.top();
        let index = (row.max(0.).floor() as usize).min(self.last_line());
        let line = self.source.lines(index..index + 1).pop()?;
        let x = f64::from(f32::from(point.x - origin.x - metrics.gutter_width())) + self.scroll.left();
        let column = (x / f64::from(f32::from(metrics.char_width))).max(0.).round() as usize;
        let offset = line.offset_at(column);
        Some(Hit { index, line, column, offset })
    }

    fn scroll_position(&self) -> ScrollPosition {
        self.scroll.position()
    }

    fn parse_job(&self, revision: u64) -> Option<ParseJob> {
        Some(ParseJob {
            language: self.language?,
            source: self.source.source()?,
            cancel: self.cancel.clone(),
            target: ParseTarget::Text,
            revision,
            previous: Previous::Nothing,
            failed: Failed::default(),
        })
    }

    fn set_syntax(&mut self, syntax: Syntax) {
        self.syntax = Some(syntax);
        self.highlights = HighlightCache::default();
    }

    /// Highlights of what `line` displays, queried per line so a huge line costs only its shown part, and once
    /// while the line stays on screen.
    fn highlights_of(&self, line: &DisplayLine) -> Rc<[Highlight]> {
        let Some(syntax) = &self.syntax else {
            return Rc::from([]);
        };
        let bytes = line.file_bytes();
        self.highlights.get_or_query(bytes.start, || syntax::spans(syntax, &self.text(), bytes.clone()))
    }

    /// New text keeps the scroll position (clamped to the new length) and shows plain until it is parsed again
    /// (`parse_job`).
    fn replace_source(&mut self, source: Arc<dyn LineSource>, language: Option<Language>) {
        self.scroll.set_line_count(source.line_count());
        self.source = source;
        self.language = language;
        self.notice = None;
        self.selection = None;
        self.dragging = false;
        self.forget_syntax();
    }

    /// The old tree's offsets do not fit new text, and a parse still running for it is stale.
    fn forget_syntax(&mut self) {
        self.cancel.cancel();
        self.cancel = Cancel::default();
        self.syntax = None;
        self.highlights = HighlightCache::default();
    }

    fn mark_deleted(&mut self) {
        self.notice = Some("deleted on disk".into());
    }

    /// `interact` adds the mouse handling to the text area.
    fn render_with(
        &self,
        theme: &ViewStyle,
        window: &mut Window,
        cx: &App,
        interact: impl FnOnce(Stateful<Div>, Metrics) -> Stateful<Div>,
    ) -> Div {
        let scheme = cx.global::<ResolvedScheme>();
        let metrics = Metrics::measure(window, cx, self.source.line_count());
        let top = self.scroll.top();
        let first = top.floor() as usize;
        let visible = self.scroll.viewport_lines().ceil() as usize + 1;
        let lines = self.source.lines(first..first + visible);
        self.highlights.start_frame();
        let widest_chars = lines.iter().map(|line| line.text.chars().count()).max().unwrap_or(0);
        self.scroll.set_content_width(metrics.text_width(widest_chars));
        let shift = px(self.scroll.left() as f32);
        let rows: Vec<Div> = lines
            .iter()
            .enumerate()
            .map(|(offset, line)| {
                let text = styled_line(line, &self.highlights_of(line), scheme);
                let selected = self.selection.and_then(|selection| selection.columns_on(line));
                let marked = self.marked == Some(first + offset);
                render_line(first + offset, Row { text, selected, marked }, shift, &metrics, theme)
            })
            .collect();
        let wheel_scroll = self.scroll.clone();
        let line_height = metrics.line_height;
        let text = div()
            .id("editor")
            .relative()
            .flex_1()
            .min_h_0()
            .overflow_hidden()
            .font_family(theme.mono_font.clone())
            .text_size(metrics.font_size)
            .line_height(metrics.line_height)
            .text_color(rgb(scheme.get(ColorKey::TEXT).foreground.unwrap_or(theme.text)))
            .on_scroll_wheel(move |event, window, _cx| wheel_scroll.scroll_by_wheel(event, line_height, window))
            .child(line_view::measure_viewport(self.scroll.clone(), metrics));
        let text = interact(text, metrics)
            .child(
                div()
                    .absolute()
                    .left_0()
                    .right_0()
                    .top(-metrics.line_height * (top - first as f64) as f32)
                    .children(rows),
            )
            .child(vertical_scrollbar(&self.scrollbars, theme))
            .child(horizontal_scrollbar(&self.scrollbars, theme));
        let notice = self.notice.clone().map(|notice| {
            div()
                .flex_none()
                .px(px(theme.editor_notice_padding_x))
                .py(px(theme.editor_notice_padding_y))
                .bg(rgb(theme.hover))
                .text_size(px(theme.editor_notice_text_size))
                .text_color(rgb(theme.error))
                .child(notice)
        });
        div().flex().flex_col().flex_1().min_h_0().children(notice).child(text)
    }
}

impl Focusable for CodeView {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for CodeView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let style = ViewStyle::drawing(cx).clone();
        let selectable = self.selectable;
        let press = cx.listener(Self::on_mouse_down);
        let release = cx.listener(Self::on_mouse_up);
        let release_out = cx.listener(Self::on_mouse_up);
        let (view, placed) = (cx.entity(), self.state.placed.clone());
        let body = self
            .state
            .render_with(&style, window, cx, move |text, metrics| match selectable {
                true => text
                    .cursor(CursorStyle::IBeam)
                    .on_mouse_down(MouseButton::Left, press)
                    .on_mouse_up(MouseButton::Left, release)
                    // a drag that ends outside the text still ends
                    .on_mouse_up_out(MouseButton::Left, release_out)
                    .child(follow_drag(metrics, placed, view)),
                false => text,
            });
        // `track_focus` alone makes a press take focus, so an unselectable view leaves it to the parent
        match selectable {
            true => body.track_focus(&self.focus_handle).on_key_down(cx.listener(Self::on_key_down)),
            false => body,
        }
    }
}

/// Records where the view was painted, and follows a selection drag over the whole window: an element's own
/// mouse-move listener hears only moves over it.
fn follow_drag(metrics: Metrics, into: Rc<Cell<Option<Placed>>>, view: Entity<CodeView>) -> impl IntoElement {
    canvas(
        |_bounds, _window, _cx| {},
        move |bounds, (), window, _cx| {
            into.set(Some(Placed { origin: bounds.origin, metrics }));
            window.on_mouse_event(move |event: &MouseMoveEvent, phase, _window, cx| {
                if phase == DispatchPhase::Bubble && event.pressed_button == Some(MouseButton::Left) {
                    view.update(cx, |view, cx| {
                        if view.state.drag_to(event.position) {
                            cx.notify();
                        }
                    });
                }
            });
        },
    )
    .absolute()
    .size_full()
}

/// Highlights of the lines drawn in the last frame, by the file offset each starts at; repaints and horizontal
/// scrolling reuse them instead of querying the tree again.
#[derive(Default)]
pub(crate) struct HighlightCache {
    current: RefCell<HashMap<usize, Rc<[Highlight]>>>,
    previous: RefCell<HashMap<usize, Rc<[Highlight]>>>,
}

impl HighlightCache {
    /// Lines not asked for until the next frame starts are forgotten then.
    pub(crate) fn start_frame(&self) {
        let current = self.current.take();
        self.previous.replace(current);
    }

    pub(crate) fn get_or_query(&self, line_start: usize, query: impl FnOnce() -> Vec<Highlight>) -> Rc<[Highlight]> {
        let known = self.current.borrow().get(&line_start).cloned();
        let highlights = known
            .or_else(|| self.previous.borrow_mut().remove(&line_start))
            .unwrap_or_else(|| Rc::from(query()));
        self.current.borrow_mut().insert(line_start, highlights.clone());
        highlights
    }
}

fn styled_line(line: &DisplayLine, highlights: &[Highlight], scheme: &ResolvedScheme) -> StyledText {
    let styles = line.place(highlights).into_iter().map(|(range, key)| (range, scheme.get(key).highlight_style()));
    StyledText::new(line.text.clone()).with_highlights(styles)
}

/// A line's text and the columns of it that are selected.
struct Row {
    text: StyledText,
    selected: Option<Range<usize>>,
    /// the line `file:42` went to
    marked: bool,
}

fn render_line(index: usize, row: Row, shift: Pixels, metrics: &Metrics, theme: &ViewStyle) -> Div {
    let number = format!("{:>width$}", index + 1, width = metrics.number_chars);
    div()
        .flex()
        .h(metrics.line_height)
        .when(row.marked, |line| line.bg(rgb(theme.hover)))
        .child(
            div()
                .flex_none()
                .w(metrics.gutter_width())
                .pl(metrics.char_width)
                .text_color(rgb(theme.muted))
                .child(number),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .overflow_hidden()
                .child(div().relative().left(-shift).whitespace_nowrap().children(row.selected.map(|columns| selection_bar(columns, metrics, theme))).child(row.text)),
        )
}

/// The selection's background behind `columns` of a line; glyphs of the monospace font sit on the column grid.
fn selection_bar(columns: Range<usize>, metrics: &Metrics, theme: &ViewStyle) -> Div {
    div()
        .absolute()
        .top_0()
        .h(metrics.line_height)
        .left(metrics.char_width * columns.start as f32)
        .w(metrics.char_width * columns.len() as f32)
        .bg(rgb(theme.selected))
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::fs;
    use std::path::Path;

    use super::*;
    use crate::document::Document;
    use crate::syntax::MAX_HIGHLIGHT_BYTES;

    /// A view of the file at `path`, in the language its name says.
    fn opened(path: &Path) -> State {
        let document = Document::open(path).unwrap();
        let language = Language::for_path(path, document.bytes());
        State::new(Arc::new(document), language)
    }

    fn editor_for(name: &str, text: &[u8]) -> (tempfile::TempDir, State) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(name);
        fs::write(&path, text).unwrap();
        let editor = opened(&path);
        (dir, editor)
    }

    #[test]
    fn should_query_each_shown_line_once_across_repaints() {
        // given
        let cache = HighlightCache::default();
        let queries = Cell::new(0);
        let query = || {
            queries.set(queries.get() + 1);
            Vec::new()
        };

        // when
        for _ in 0..3 {
            cache.start_frame();
            cache.get_or_query(0, query);
            cache.get_or_query(10, query);
        }

        // then
        assert_eq!(queries.get(), 2);
    }

    #[test]
    fn should_forget_lines_not_shown_in_the_last_frame() {
        // given
        let cache = HighlightCache::default();
        let queries = Cell::new(0);
        let query = || {
            queries.set(queries.get() + 1);
            Vec::new()
        };
        cache.start_frame();
        cache.get_or_query(0, query);
        cache.start_frame();
        cache.get_or_query(10, query);

        // when
        cache.start_frame();
        cache.get_or_query(0, query);

        // then
        assert_eq!(queries.get(), 3);
    }

    #[test]
    fn should_parse_only_known_languages_up_to_the_limit() {
        // given
        let small = editor_for("data.json", b"{}");
        let big = editor_for("big.json", &vec![b' '; MAX_HIGHLIGHT_BYTES + 1]);
        let log = editor_for("server.log", b"plain");

        // when
        let jobs = [small.1.parse_job(0).is_some(), big.1.parse_job(0).is_some(), log.1.parse_job(0).is_some()];

        // then
        assert_eq!(jobs, [true, false, false]);
    }

    #[test]
    fn should_colour_java_line_with_tab_and_crlf() {
        // given
        let (_dir, mut editor) = editor_for("A.java", b"class A {\r\n\tint x;\r\n}\r\n");
        editor.set_syntax(editor.parse_job(0).unwrap().run().unwrap());

        // when
        let line = editor.source.lines(1..2).remove(0);
        let placed = line.place(&editor.highlights_of(&line));

        // then
        let int = placed.iter().find(|(range, _)| &line.text[range.clone()] == "int").map(|(_, key)| key.id());
        assert_eq!((line.text.as_ref(), int), ("    int x;", Some("JAVA_KEYWORD")));
    }

    #[test]
    fn should_bound_highlighting_to_displayed_part_of_long_line() {
        // given
        let json = format!("[{}1]", "1, ".repeat(1_000_000));
        let (_dir, mut editor) = editor_for("long.json", json.as_bytes());
        editor.set_syntax(editor.parse_job(0).unwrap().run().unwrap());
        let line = editor.source.lines(0..1).remove(0);

        // when
        let highlights = editor.highlights_of(&line);

        // then
        let end = highlights.iter().map(|highlight| highlight.bytes.end).max().unwrap();
        assert!(end <= line.file_bytes().end, "{end} > {:?}", line.file_bytes());
        assert!(line.file_bytes().len() <= crate::document::MAX_LINE_CHARS * 4);
    }

    #[test]
    fn should_show_new_text_and_say_when_it_is_deleted() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("notes.md");
        fs::write(&path, "one\ntwo\n").unwrap();
        let mut editor = opened(&path);
        fs::write(&path, "one\ntwo\nthree\nfour\n").unwrap();

        // when
        editor.replace_source(Arc::new(Document::open(&path).unwrap()), None);
        let lines = editor.source.line_count();
        fs::remove_file(&path).unwrap();
        editor.mark_deleted();

        // then
        assert_eq!((lines, editor.notice.as_ref().map(SharedString::as_ref)), (4, Some("deleted on disk")));
        assert_eq!(editor.source.line_count(), 4, "the text stays");
    }

    #[test]
    fn should_cancel_the_parse_of_replaced_text() {
        // given
        let (dir, mut editor) = editor_for("a.json", b"{}");
        let stale = editor.parse_job(0).unwrap().cancel();

        // when
        editor.replace_source(Arc::new(Document::open(&dir.path().join("a.json")).unwrap()), None);

        // then
        assert!(stale.is_cancelled());
    }

    #[test]
    fn should_mark_the_revealed_line() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.txt");
        std::fs::write(&path, (1..=100).map(|line| format!("line {line}\n")).collect::<String>()).unwrap();
        let mut editor = opened(&path);

        // when
        editor.reveal_line(42);

        // then
        let top = editor.scroll_position().top_line;
        assert_eq!(editor.marked_line(), Some(42));
        assert!(top <= 41. && 41. < top + editor.scroll.viewport_lines(), "top {top}");
    }

    #[test]
    fn should_keep_a_line_past_the_end_inside_the_text() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.txt");
        std::fs::write(&path, "one\ntwo\n").unwrap();
        let mut editor = opened(&path);

        // when
        editor.reveal_line(500);

        // then
        assert_eq!(editor.marked_line(), Some(editor.source.line_count()));
    }
}
