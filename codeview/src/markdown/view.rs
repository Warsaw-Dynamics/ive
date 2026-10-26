//! Rendered: a Markdown document drawn as a page, one list item per block, read-only; it scrolls by block, answers the
//! find box over its visible text, and copies a code block by its Copy button.

use std::cell::{Cell, RefCell};
use std::ops::Range;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use gpui::{
    AnyElement, App, Context, CursorStyle, FocusHandle, Focusable, Font, FontWeight, InteractiveText, KeyDownEvent, ListAlignment, ListOffset,
    ListState, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, Point, SharedString, StyledText, Task, TextLayout, WeakEntity,
    Window, canvas, div, list, prelude::*, px, relative, rgb, rgba,
};

use super::document::{Align, Block, BlockKind, Doc, Look, Marker, Piece, Run, Tick};
use super::paint::{Overlay, Paint, text_runs};
use crate::colors::ResolvedScheme;
use crate::find::FindText;
use crate::find::marks::{FindMarks, Mark};
use crate::host::{ClipboardTarget, Command, Host, KeyOutcome, KeyScope};
use crate::scrollbar::{BlockScroll, ScrollState, Tick as ScrollTick, vertical_scrollbar_with_ticks};
use crate::style::ViewStyle;
use crate::text_selection::TextSelection;

/// Left and right of the page.
const PAGE_MARGIN: f32 = 32.;
/// The widest prose runs.
const MAX_WIDTH: f32 = 760.;
const TEXT_SIZE: f32 = 14.;
const LINE_HEIGHT: f32 = 1.6;
const CODE_TEXT_SIZE: f32 = 13.;
const CODE_LINE_HEIGHT: f32 = 20.;
/// Above and below a code block's lines, inside its box.
const CODE_PADDING_Y: f32 = 10.;
/// A nested list's step to the right, and the column its markers stand in.
const LIST_INDENT: f32 = 24.;
const MARKER_WIDTH: f32 = 20.;
const BLOCK_GAP: f32 = 12.;
const ITEM_GAP: f32 = 2.;
/// How long a Copy button says "Copied".
const COPIED_FOR: Duration = Duration::from_millis(1500);
/// How far an arrow key scrolls.
const ARROW_SCROLL: f32 = 40.;
/// How far a drag past the top or bottom scrolls per mouse move.
const DRAG_SCROLL: f32 = 20.;
/// Laid out beyond the view, so a scroll shows blocks already measured.
const OVERDRAW: f32 = 400.;

pub struct RenderedView {
    doc: Arc<Doc>,
    /// the find box's text: the document's, its pieces apart
    found: (Arc<Vec<u8>>, Arc<Vec<usize>>),
    /// the code blocks, by block index, in order: Copy button `n` is code block `n - 1`
    code_blocks: Arc<Vec<usize>>,
    revision: u64,
    list: ListState,
    scroll: ScrollState,
    focus_handle: FocusHandle,
    marks: Option<FindMarks>,
    /// the marks on the scrollbar, made when the marks change rather than at each frame
    ticks: Rc<[ScrollTick]>,
    /// the block of each piece, in order
    piece_blocks: Arc<Vec<usize>>,
    /// the code block whose Copy button says "Copied"
    copied: Option<usize>,
    copied_reset: Task<()>,
    /// the blocks drawn in the last frame
    shown: Rc<RefCell<Vec<usize>>>,
    /// the pieces drawn in the last frame, where the mouse finds the text under it
    placed: Rc<RefCell<Vec<Placed>>>,
    selection: Option<TextSelection>,
    drag: Drag,
}

/// A piece rendered: where it starts in the document's text, its laid out text, and whether it was drawn. gpui's list
/// also renders blocks just to measure them (past the view's edges); their text has no place on screen to hit.
pub(super) struct Placed {
    start: usize,
    layout: TextLayout,
    drawn: Rc<Cell<bool>>,
}

impl Placed {
    pub(super) fn new(start: usize, layout: TextLayout) -> Self {
        Self { start, layout, drawn: Rc::default() }
    }
}

/// Whether a press in the text is extending the selection as the mouse moves.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Drag {
    Idle,
    Selecting,
}

impl RenderedView {
    pub fn new(doc: Arc<Doc>, cx: &mut Context<Self>) -> Self {
        let list = ListState::new(doc.blocks.len(), ListAlignment::Top, px(OVERDRAW));
        let scroll = ScrollState::new(Rc::new(BlockScroll::new(list.clone())));
        Self {
            found: found_text(&doc),
            code_blocks: code_blocks(&doc),
            piece_blocks: piece_blocks(&doc),
            doc,
            revision: 0,
            list,
            scroll,
            focus_handle: cx.focus_handle(),
            marks: None,
            ticks: Rc::from([]),
            copied: None,
            copied_reset: Task::ready(()),
            shown: Rc::default(),
            placed: Rc::default(),
            selection: None,
            drag: Drag::Idle,
        }
    }

    pub fn doc(&self) -> &Arc<Doc> {
        &self.doc
    }

    /// A new document for the same text (edited, reloaded); the block at the top stays there, or the last one does.
    pub fn set_doc(&mut self, doc: Arc<Doc>, cx: &mut Context<Self>) {
        let top = self.list.logical_scroll_top();
        let last = doc.blocks.len().saturating_sub(1);
        self.list.reset(doc.blocks.len());
        let offset = if top.item_ix > last { px(0.) } else { top.offset_in_item };
        self.list.scroll_to(ListOffset { item_ix: top.item_ix.min(last), offset_in_item: offset });
        self.found = found_text(&doc);
        self.code_blocks = code_blocks(&doc);
        self.piece_blocks = piece_blocks(&doc);
        self.doc = doc;
        self.selection = None;
        self.revision += 1;
        cx.notify();
    }

    /// Counts the documents set after the first.
    pub fn doc_revision(&self) -> u64 {
        self.revision
    }

    pub fn top_block(&self) -> usize {
        self.list.logical_scroll_top().item_ix.min(self.doc.blocks.len().saturating_sub(1))
    }

    /// The first source line (from 0) of the block at the top.
    pub fn top_line(&self) -> usize {
        self.doc.blocks.get(self.top_block()).map_or(0, |block| block.lines.start)
    }

    /// Scrolls the block holding source line `line` (from 0) to the top.
    pub fn reveal_source_line(&mut self, line: usize, cx: &mut Context<Self>) {
        self.list.scroll_to(ListOffset { item_ix: self.doc.block_at_line(line), offset_in_item: px(0.) });
        cx.notify();
    }

    #[cfg(test)]
    /// The blocks drawn in the last frame, the ones laid out past the view's edges included.
    pub(crate) fn shown_blocks(&self) -> Vec<usize> {
        self.shown.borrow().clone()
    }

    pub fn find_text(&self) -> FindText {
        FindText::Parts { bytes: self.found.0.clone(), starts: self.found.1.clone() }
    }

    /// Where typing in the find box searches from: the selection, else the block at the top.
    pub fn find_anchor(&self) -> Range<usize> {
        if let Some(selection) = self.selection {
            return selection.range();
        }
        let start = self.doc.blocks.get(self.top_block()).and_then(|block| block.pieces.first()).map_or(0, |piece| piece.text.start);
        start..start
    }

    /// Scrolls the match into view, a third of the way down: by the drawn text where it is drawn, else by the lines
    /// above it in a code block (they do not wrap); a block not measured yet is placed by its index.
    pub fn show_match(&mut self, bytes: Range<usize>, cx: &mut Context<Self>) {
        let (block, _) = self.doc.piece_at(bytes.start);
        let view = self.list.viewport_bounds();
        let depth = self.depth_in_block(block, bytes.start);
        let in_view = self.list.bounds_for_item(block).is_some_and(|shown| {
            let top = shown.top() + depth;
            top >= view.top() && top + px(CODE_LINE_HEIGHT) <= view.bottom()
        });
        if in_view {
            return;
        }
        let third = view.size.height / 3.;
        let offset = match depth > third {
            true => ListOffset { item_ix: block, offset_in_item: depth - third },
            false => ListOffset { item_ix: block.saturating_sub(1), offset_in_item: px(0.) },
        };
        self.list.scroll_to(offset);
        cx.notify();
    }

    /// How far below its block's top the text at `offset` is.
    fn depth_in_block(&self, block: usize, offset: usize) -> Pixels {
        let drawn = self.placed.borrow().iter().filter(|placed| placed.drawn.get() && placed.start <= offset).max_by_key(|placed| placed.start).and_then(|placed| placed.layout.position_for_index(offset - placed.start));
        if let (Some(at), Some(shown)) = (drawn, self.list.bounds_for_item(block)) {
            return at.y - shown.top();
        }
        let Some(Block { kind: BlockKind::Code { .. }, pieces, .. }) = self.doc.blocks.get(block) else {
            return px(0.);
        };
        let start = pieces.first().map_or(offset, |piece| piece.text.start);
        let lines_above = self.doc.text[start..offset.max(start)].matches('\n').count();
        let padding = self.doc.blocks.get(block).map_or(CODE_PADDING_Y, |block| pad_unless(block.part.continues()));
        px(padding + CODE_LINE_HEIGHT * lines_above as f32)
    }

    pub fn set_find_marks(&mut self, marks: Option<FindMarks>, cx: &mut Context<Self>) {
        self.ticks = self.ticks_of(marks.as_ref());
        self.marks = marks;
        cx.notify();
    }

    #[cfg(test)]
    pub(crate) fn has_find_marks(&self) -> bool {
        self.marks.is_some()
    }

    /// The target of the link at `offset` of the document's text.
    #[cfg(test)]
    pub(crate) fn link_at(&self, offset: usize) -> Option<SharedString> {
        link_at(&self.doc, offset)
    }

    #[cfg(test)]
    /// What Copy button `number` (from 1) says.
    pub(crate) fn copy_label(&self, number: usize) -> &'static str {
        let block = number.checked_sub(1).and_then(|index| self.code_blocks.get(index).copied());
        match block {
            Some(block) if self.copied == Some(block) => "Copied",
            _ => "Copy",
        }
    }

    #[cfg(test)]
    pub(crate) fn piece_bounds(&self, start: usize) -> Option<gpui::Bounds<gpui::Pixels>> {
        self.placed.borrow().iter().find(|placed| placed.start == start && placed.drawn.get()).map(|placed| placed.layout.bounds())
    }

    /// Where the text at `offset` is drawn, from the window's top; none when it is not drawn.
    #[cfg(test)]
    pub(crate) fn position_of(&self, offset: usize) -> Option<Pixels> {
        let placed = self.placed.borrow();
        let piece = placed.iter().filter(|placed| placed.drawn.get() && placed.start <= offset).max_by_key(|placed| placed.start)?;
        piece.layout.position_for_index(offset - piece.start).map(|point| point.y)
    }

    /// The selection as copied: the visible text, blocks apart; none when nothing is selected.
    pub fn selected_text(&self) -> Option<String> {
        self.selection.map(|selection| selection.range()).filter(|range| !range.is_empty()).map(|range| self.doc.copy_text(range))
    }

    /// The selection when it is one line, and where it starts: what Ctrl+F opens with.
    pub fn find_prefill(&self) -> Option<(String, usize)> {
        let range = self.selection?.range();
        let text = self.selected_text().filter(|text| !text.contains('\n'))?;
        Some((text, range.start))
    }

    pub fn select_all(&mut self, cx: &mut Context<Self>) {
        self.selection = Some(TextSelection::of(0..self.doc.text.len()));
        cx.notify();
    }

    pub fn copy(&self, target: ClipboardTarget, cx: &mut App) {
        if let Some(text) = self.selected_text() {
            Host::write_clipboard(target, text, cx);
        }
    }

    /// A press selects: a click places the start, a double click a word, a triple click the piece, Shift+click
    /// extends; the view takes the keyboard.
    fn on_mouse_down(&mut self, event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.focus_handle, cx);
        let Some(at) = self.hit(event.position) else {
            return;
        };
        self.selection = Some(match (event.click_count, self.selection) {
            (2, _) => TextSelection::of(self.doc.word_at(at)),
            (count, _) if count >= 3 => TextSelection::of(self.doc.piece_range(at)),
            (_, Some(selection)) if event.modifiers.shift => selection.extended_to(at),
            _ => TextSelection::at(at),
        });
        self.drag = Drag::Selecting;
        cx.notify();
    }

    /// A drag extends the selection; past the top or the bottom it scrolls.
    fn on_mouse_move(&mut self, event: &MouseMoveEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.drag == Drag::Idle || event.pressed_button != Some(MouseButton::Left) {
            return;
        }
        let view = self.list.viewport_bounds();
        if event.position.y < view.top() {
            self.list.scroll_by(-px(DRAG_SCROLL));
        }
        if event.position.y > view.bottom() {
            self.list.scroll_by(px(DRAG_SCROLL));
        }
        let (Some(at), Some(selection)) = (self.hit(event.position), self.selection) else {
            return;
        };
        self.selection = Some(selection.extended_to(at));
        cx.notify();
    }

    fn on_mouse_up(&mut self, _event: &MouseUpEvent, _window: &mut Window, _cx: &mut Context<Self>) {
        self.drag = Drag::Idle;
    }

    /// The host's Copy and Select All, and the reading keys (Page Up and Down, Home, End, the arrows) when nothing is
    /// bound to them; other keys go on to the panel (find) and the window's bindings.
    fn on_key_down(&mut self, event: &KeyDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        match Host::key_outcome(event, KeyScope::Viewer, cx) {
            KeyOutcome::Command(Command::Copy(target)) => self.copy(target, cx),
            KeyOutcome::Command(Command::SelectAll) => self.select_all(cx),
            KeyOutcome::Handled => {}
            KeyOutcome::Unbound if self.scroll_by_key(&event.keystroke.key) => cx.notify(),
            KeyOutcome::Command(_) | KeyOutcome::Foreign | KeyOutcome::Unbound => return,
        }
        cx.stop_propagation();
    }

    /// Scrolls as reading key `key` says; false for any other key.
    fn scroll_by_key(&mut self, key: &str) -> bool {
        let page = self.list.viewport_bounds().size.height * 0.9;
        match key {
            "pagedown" => self.list.scroll_by(page),
            "pageup" => self.list.scroll_by(-page),
            "down" => self.list.scroll_by(px(ARROW_SCROLL)),
            "up" => self.list.scroll_by(-px(ARROW_SCROLL)),
            "home" => self.list.scroll_to(ListOffset { item_ix: 0, offset_in_item: px(0.) }),
            "end" => self.list.scroll_to_end(),
            _ => return false,
        }
        true
    }

    fn hit(&self, point: Point<Pixels>) -> Option<usize> {
        hit_in(&self.placed.borrow(), point)
    }

    /// Code block `block`'s code to the clipboard; its button says "Copied" for a while.
    fn copy_code(&mut self, block: usize, cx: &mut Context<Self>) {
        let Some(code) = self.doc.code_of(block) else {
            return;
        };
        Host::write_clipboard(copy_target(), code, cx);
        self.copied = Some(self.doc.parts_around(block).start);
        self.copied_reset = cx.spawn(async move |view, cx| {
            cx.background_executor().timer(COPIED_FOR).await;
            view.update(cx, |view, cx| {
                view.copied = None;
                cx.notify();
            })
            .ok();
        });
        cx.notify();
    }

    /// A scrollbar tick per match, at its block's share of the document.
    fn ticks_of(&self, marks: Option<&FindMarks>) -> Rc<[ScrollTick]> {
        let Some(marks) = marks else {
            return Rc::from([]);
        };
        let blocks = self.doc.blocks.len().max(1) as f32;
        let starts = &self.found.1;
        let block_of = |offset: usize| self.piece_blocks.get(starts.partition_point(|start| *start <= offset).saturating_sub(1)).copied().unwrap_or(0);
        marks.ranges.iter().enumerate().map(|(index, range)| ScrollTick { at: block_of(range.start) as f32 / blocks, current: marks.current == Some(index) }).collect()
    }

    fn frame(&self, window: &Window, cx: &mut Context<Self>) -> Frame {
        Frame {
            doc: self.doc.clone(),
            paint: paint(window, cx),
            style: ViewStyle::drawing(cx).clone(),
            marks: self.marks.clone(),
            copied: self.copied,
            code_blocks: self.code_blocks.clone(),
            view: cx.weak_entity(),
            shown: self.shown.clone(),
            placed: self.placed.clone(),
            selection: self.selection.map(|selection| selection.range()),
        }
    }
}

/// The offset of the document's text under `point`: in the piece on that line nearest to it, else the start of the
/// first piece below, else the end of the last.
pub(super) fn hit_in(placed: &[Placed], point: Point<Pixels>) -> Option<usize> {
    let drawn: Vec<&Placed> = placed.iter().filter(|placed| placed.drawn.get()).collect();
    let placed = drawn.as_slice();
    let on_line = placed.iter().filter(|placed| {
        let bounds = placed.layout.bounds();
        bounds.top() <= point.y && point.y <= bounds.bottom()
    });
    if let Some(nearest) = on_line.min_by(|one, other| across(one, point).total_cmp(&across(other, point))) {
        return Some(nearest.start + nearest.layout.index_for_position(point).unwrap_or_else(|index| index));
    }
    let below = placed.iter().filter(|placed| placed.layout.bounds().top() > point.y);
    if let Some(next) = below.min_by(|one, other| f32::from(one.layout.bounds().top()).total_cmp(&f32::from(other.layout.bounds().top()))) {
        return Some(next.start);
    }
    placed.iter().max_by(|one, other| f32::from(one.layout.bounds().bottom()).total_cmp(&f32::from(other.layout.bounds().bottom()))).map(|last| last.start + last.layout.len())
}

/// How far `point` is from `placed` sideways; 0 over it.
fn across(placed: &Placed, point: Point<Pixels>) -> f32 {
    let bounds = placed.layout.bounds();
    f32::from((bounds.left() - point.x).max(point.x - bounds.right()).max(px(0.)))
}

/// The Copy button's clipboard: the editor's Copy writes the primary selection too on Linux.
fn copy_target() -> ClipboardTarget {
    if cfg!(target_os = "linux") {
        return ClipboardTarget::Both;
    }
    ClipboardTarget::Clipboard
}

fn found_text(doc: &Doc) -> (Arc<Vec<u8>>, Arc<Vec<usize>>) {
    (Arc::new(doc.text.as_bytes().to_vec()), Arc::new(doc.piece_starts()))
}

fn piece_blocks(doc: &Doc) -> Arc<Vec<usize>> {
    Arc::new(doc.blocks.iter().enumerate().flat_map(|(index, block)| std::iter::repeat_n(index, block.pieces.len())).collect())
}

/// The code blocks, by the index of their first part.
fn code_blocks(doc: &Doc) -> Arc<Vec<usize>> {
    let firsts = doc.blocks.iter().enumerate().filter(|(_, block)| matches!(block.kind, BlockKind::Code { .. }) && !block.part.continues());
    Arc::new(firsts.map(|(index, _)| index).collect())
}

fn link_at(doc: &Doc, offset: usize) -> Option<SharedString> {
    let (block, piece) = doc.piece_at(offset);
    let runs = &doc.blocks.get(block)?.pieces.get(piece)?.runs;
    runs.iter().filter(|run| run.bytes.contains(&offset)).find_map(|run| match &run.look {
        Look::Link(target) => Some(target.clone()),
        _ => None,
    })
}

/// The fonts and colours of the page: the window's font for prose, the code font for code.
fn paint(window: &Window, cx: &App) -> Paint {
    let style = ViewStyle::drawing(cx);
    let font = window.text_style().font();
    let mono = Font { family: style.mono_font.clone(), ..font.clone() };
    let scheme = cx.try_global::<ResolvedScheme>().cloned();
    Paint {
        font,
        mono,
        color: rgb(style.text).into(),
        link: rgb(style.accent).into(),
        code_background: rgb(style.hover).into(),
        selected: rgb(style.selected).into(),
        found: rgba(style.find_match).into(),
        current: rgba(style.find_current).into(),
        syntax: Rc::new(move |key| scheme.as_ref().and_then(|scheme| scheme.get(key).foreground).map(|colour| rgb(colour).into())),
    }
}

/// What every block of one frame is drawn with.
struct Frame {
    doc: Arc<Doc>,
    paint: Paint,
    style: ViewStyle,
    marks: Option<FindMarks>,
    copied: Option<usize>,
    code_blocks: Arc<Vec<usize>>,
    view: WeakEntity<RenderedView>,
    shown: Rc<RefCell<Vec<usize>>>,
    placed: Rc<RefCell<Vec<Placed>>>,
    selection: Option<Range<usize>>,
}

impl Frame {
    /// What lies over `piece`, by bytes of the piece: find's matches and the selection.
    fn overlays(&self, piece: &Piece) -> Vec<(Range<usize>, Overlay)> {
        let start = piece.text.start;
        let local = |bytes: Range<usize>| bytes.start - start..bytes.end - start;
        let overlay = |mark| if mark == Mark::Current { Overlay::Current } else { Overlay::Match };
        let found = self.marks.iter().flat_map(|marks| marks.on_bytes(piece.text.clone())).map(|(bytes, mark)| (local(bytes), overlay(mark)));
        let selected = self.selection.clone().map(|range| range.start.max(start)..range.end.min(piece.text.end)).filter(|range| range.start < range.end);
        found.chain(selected.map(|range| (local(range), Overlay::Selected))).collect()
    }

    /// `piece` as text, its links' targets as tooltips.
    fn piece(&self, piece: &Piece, paint: &Paint) -> AnyElement {
        let text = &self.doc.text[piece.text.clone()];
        let shown = text.strip_suffix('\n').unwrap_or(text);
        let start = piece.text.start;
        let runs: Vec<Run> = piece.runs.iter().map(|run| Run { bytes: run.bytes.start - start..run.bytes.end - start, look: run.look.clone() }).collect();
        let styled = StyledText::new(SharedString::from(shown.to_owned())).with_runs(text_runs(shown.len(), &runs, &self.overlays(piece), paint));
        let placed = Placed::new(start, styled.layout().clone());
        let drawn = placed.drawn.clone();
        self.placed.borrow_mut().push(placed);
        // runs only when the piece is prepainted, not when the list just measures it
        let mark_drawn = canvas(move |_, _, _| drawn.set(true), |_, _, _, _| {}).absolute().size_0();
        let has_links = piece.runs.iter().any(|run| matches!(run.look, Look::Link(_)));
        if !has_links {
            return div().relative().child(styled).child(mark_drawn).into_any_element();
        }
        let doc = self.doc.clone();
        let text = InteractiveText::new(("md-piece", start), styled).tooltip(move |index, window, cx| {
            let target = link_at(&doc, start + index)?;
            let make = cx.try_global::<Host>()?.tooltip.clone();
            Some(make(target, window, cx))
        });
        div().relative().child(text).child(mark_drawn).into_any_element()
    }

    fn block(&self, index: usize, cx: &App) -> AnyElement {
        self.shown.borrow_mut().push(index);
        let block = &self.doc.blocks[index];
        let paint = match block.nesting.quotes {
            0 => self.paint.clone(),
            _ => Paint { color: rgb(self.style.muted).into(), ..self.paint.clone() },
        };
        let body = self.body(block, index, &paint, cx);
        let quoted = (0..block.nesting.quotes).fold(body, |inner, _| div().border_l(px(3.)).border_color(rgb(self.style.border)).pl(px(12.)).child(inner).into_any_element());
        let gap = match (&block.kind, block.part.goes_on()) {
            (_, true) => 0.,
            (BlockKind::ListItem(_), false) => ITEM_GAP,
            _ => BLOCK_GAP,
        };
        div()
            .w_full()
            .pl(px(indent_of(block)))
            .pr(px(PAGE_MARGIN))
            .pt(px(space_above(block, index)))
            .pb(px(gap))
            .text_size(px(TEXT_SIZE))
            .line_height(relative(LINE_HEIGHT))
            .text_color(paint.color)
            .child(div().max_w(px(MAX_WIDTH)).child(quoted))
            .into_any_element()
    }

    fn body(&self, block: &Block, index: usize, paint: &Paint, cx: &App) -> AnyElement {
        match &block.kind {
            BlockKind::Heading(level) => self.heading(block, *level, paint),
            BlockKind::Paragraph => self.pieces(block, paint),
            BlockKind::ListItem(marker) => self.list_item(block, *marker, paint),
            BlockKind::Code { .. } => self.code(block, index, cx),
            BlockKind::Rule => div().h(px(1.)).my(px(4.)).bg(rgb(self.style.border)).into_any_element(),
            BlockKind::Table { columns, .. } => self.table(block, columns, paint),
            BlockKind::Html => {
                let mono = Paint { font: paint.mono.clone(), ..paint.clone() };
                div().whitespace_nowrap().text_size(px(CODE_TEXT_SIZE)).child(self.pieces(block, &mono)).into_any_element()
            }
        }
    }

    fn pieces(&self, block: &Block, paint: &Paint) -> AnyElement {
        div().children(block.pieces.iter().map(|piece| self.piece(piece, paint))).into_any_element()
    }

    fn heading(&self, block: &Block, level: u8, paint: &Paint) -> AnyElement {
        let size = match level {
            1 => 24.,
            2 => 19.,
            3 => 16.,
            _ => TEXT_SIZE,
        };
        let font = Font { weight: FontWeight::SEMIBOLD, ..paint.font.clone() };
        let heading = div().text_size(px(size)).line_height(relative(1.25)).child(self.pieces(block, &Paint { font, ..paint.clone() }));
        if level > 2 {
            return heading.into_any_element();
        }
        heading.pb(px(6.)).border_b_1().border_color(rgb(self.style.border)).into_any_element()
    }

    fn list_item(&self, block: &Block, marker: Marker, paint: &Paint) -> AnyElement {
        let muted = rgb(self.style.muted);
        let mark = match marker {
            Marker::Bullet => div().child("•"),
            Marker::Number(number) => div().child(format!("{number}.")),
            Marker::Task(Tick::Done) => tick_box(muted).bg(muted),
            Marker::Task(Tick::Open) => tick_box(muted),
        };
        div()
            .flex()
            .child(div().flex_none().w(px(MARKER_WIDTH)).text_color(muted).child(mark))
            .child(div().flex_1().min_w_0().child(self.pieces(block, paint)))
            .into_any_element()
    }

    /// A code block, or a part of one: its lines unwrapped in a tinted box that scrolls sideways, rounded and padded
    /// only at the block's ends, and a Copy button in its corner while the mouse is over it.
    fn code(&self, block: &Block, index: usize, cx: &App) -> AnyElement {
        let paint = Paint { font: self.paint.mono.clone(), ..self.paint.clone() };
        let (above, below) = (pad_unless(block.part.continues()), pad_unless(block.part.goes_on()));
        let lines = div()
            .id(("md-code", index))
            .overflow_x_scroll()
            .px(px(12.))
            .pt(px(above))
            .pb(px(below))
            .text_size(px(CODE_TEXT_SIZE))
            .line_height(px(CODE_LINE_HEIGHT))
            .whitespace_nowrap()
            .child(self.pieces(block, &paint));
        let group = SharedString::from(format!("md-code-{index}"));
        let boxed = div().group(group.clone()).relative().bg(self.paint.code_background).child(lines).child(self.copy_button(index, group, cx));
        let boxed = if block.part.continues() { boxed } else { boxed.rounded_t(px(6.)) };
        let boxed = if block.part.goes_on() { boxed } else { boxed.rounded_b(px(6.)) };
        boxed.into_any_element()
    }

    /// The Copy button of the code block part `index` is of: named and copying for the whole block, shown while the
    /// mouse is over `group` (the part) and while it says Copied.
    fn copy_button(&self, index: usize, group: SharedString, cx: &App) -> AnyElement {
        let first = self.doc.parts_around(index).start;
        let number = self.code_blocks.iter().position(|block| *block == first).map_or(0, |at| at + 1);
        let copied = self.copied == Some(first);
        let label = if copied { "Copied" } else { "Copy" };
        let view = self.view.clone();
        let click = Host::click_target(format!("markdown-copy:{number}"), move |_, _, cx| drop(view.update(cx, |view, cx| view.copy_code(first, cx))), cx);
        let (text, hover) = (rgb(self.style.text), rgb(self.style.hover));
        let button = div()
            .id(("md-copy", index))
            .debug_selector(move || format!("markdown-copy-{number}"))
            .absolute()
            .top(px(6.))
            .right(px(6.))
            .px(px(8.))
            .py(px(1.))
            .rounded(px(6.))
            .border_1()
            .border_color(rgb(self.style.border))
            .bg(rgb(self.style.background))
            .text_size(px(12.))
            .line_height(px(18.))
            .text_color(rgb(self.style.muted))
            .cursor_pointer()
            .hover(move |style| style.text_color(text).bg(hover))
            .when(!copied, |button| button.invisible().group_hover(group, |style| style.visible()))
            // a press on it starts no selection in the code under it
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .on_click(click)
            .child(label);
        match Host::tooltip("Copy the code", cx) {
            Some(tooltip) => button.tooltip(tooltip).into_any_element(),
            None => button.into_any_element(),
        }
    }

    /// A table, or a part of one: its cells in equal columns, bordered, the header row bold on a tint.
    fn table(&self, block: &Block, columns: &[Align], paint: &Paint) -> AnyElement {
        let bold = Paint { font: Font { weight: FontWeight::SEMIBOLD, ..paint.font.clone() }, ..paint.clone() };
        let border = rgb(self.style.border);
        let continues = block.part.continues();
        let rows = block.pieces.chunks(columns.len().max(1)).enumerate().map(|(row, cells)| {
            let header = row == 0 && !continues;
            let cells = cells.iter().zip(columns).enumerate().map(|(column, (cell, align))| {
                let text = self.piece(cell, if header { &bold } else { paint });
                let cell = div().flex_1().min_w_0().px(px(12.)).py(px(4.)).border_color(border).when(column > 0, |cell| cell.border_l_1()).child(text);
                match align {
                    Align::Left => cell,
                    Align::Center => cell.text_center(),
                    Align::Right => cell.text_right(),
                }
            });
            let line = div().flex().border_color(border).children(cells);
            match (header, row > 0 || continues) {
                (true, _) => line.bg(self.paint.code_background),
                (false, true) => line.border_t_1(),
                (false, false) => line,
            }
        });
        let table = div().flex().flex_col().border_x_1().border_color(border).children(rows);
        let table = if continues { table } else { table.border_t_1() };
        let table = if block.part.goes_on() { table } else { table.border_b_1() };
        table.into_any_element()
    }
}

/// A code box's padding above or below its lines: none where the block goes on in another part.
fn pad_unless(continued: bool) -> f32 {
    if continued {
        return 0.;
    }
    CODE_PADDING_Y
}

/// How far right of the view's left edge a block's content starts: past the page margin and the lists it is in; a
/// block inside a list item that is not the item's first starts under the item's text, past its marker.
pub(super) fn indent_of(block: &Block) -> f32 {
    let lists = f32::from(block.nesting.lists.saturating_sub(1));
    let under_text = block.nesting.lists > 0 && !matches!(block.kind, BlockKind::ListItem(_));
    PAGE_MARGIN + LIST_INDENT * lists + if under_text { MARKER_WIDTH } else { 0. }
}

/// A task item's box, ticked when filled.
fn tick_box(colour: gpui::Rgba) -> gpui::Div {
    div().mt(px(5.)).size(px(13.)).rounded(px(3.)).border_1().border_color(colour)
}

/// Room above a block: the page's top above the first, more above a heading.
fn space_above(block: &Block, index: usize) -> f32 {
    match (index, &block.kind) {
        _ if block.part.continues() => 0.,
        (0, _) => 16.,
        (_, BlockKind::Heading(_)) => 8.,
        _ => 0.,
    }
}

impl Focusable for RenderedView {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for RenderedView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.shown.borrow_mut().clear();
        self.placed.borrow_mut().clear();
        let frame = self.frame(window, cx);
        let style = frame.style.clone();
        let blocks = list(self.list.clone(), move |index, _window, cx| frame.block(index, cx)).size_full();
        div()
            .id("rendered")
            .track_focus(&self.focus_handle)
            .relative()
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .size_full()
            .cursor(CursorStyle::IBeam)
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_move(cx.listener(Self::on_mouse_move))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_key_down(cx.listener(Self::on_key_down))
            .child(blocks)
            .child(vertical_scrollbar_with_ticks(&self.scroll, &style, self.ticks.clone()))
    }
}
