//! Rendered: a Markdown document drawn as a page, one list item per block, read-only; it scrolls by block, answers the
//! find box over its visible text, and copies a code block by its Copy button.

use std::cell::RefCell;
use std::ops::Range;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use gpui::{
    AnyElement, App, Context, FocusHandle, Focusable, Font, FontWeight, InteractiveText, ListAlignment, ListOffset, ListState,
    MouseButton, SharedString, StyledText, Task, WeakEntity, Window, div, list, prelude::*, px, relative, rgb, rgba,
};

use super::document::{Align, Block, BlockKind, Doc, Look, Marker, Piece, Run, Tick};
use super::paint::{Overlay, Paint, text_runs};
use crate::colors::ResolvedScheme;
use crate::find::FindText;
use crate::find::marks::{FindMarks, Mark};
use crate::host::{ClipboardTarget, Host};
use crate::scrollbar::{BlockScroll, ScrollState, Tick as ScrollTick, vertical_scrollbar_with_ticks};
use crate::style::ViewStyle;

/// Left and right of the page.
const PAGE_MARGIN: f32 = 32.;
/// The widest prose runs.
const MAX_WIDTH: f32 = 760.;
const TEXT_SIZE: f32 = 14.;
const LINE_HEIGHT: f32 = 1.6;
const CODE_TEXT_SIZE: f32 = 13.;
const CODE_LINE_HEIGHT: f32 = 20.;
/// A nested list's step to the right, and the column its markers stand in.
const LIST_INDENT: f32 = 24.;
const MARKER_WIDTH: f32 = 20.;
const BLOCK_GAP: f32 = 12.;
const ITEM_GAP: f32 = 2.;
/// How long a Copy button says "Copied".
const COPIED_FOR: Duration = Duration::from_millis(1500);
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
    /// the code block whose Copy button says "Copied"
    copied: Option<usize>,
    copied_reset: Task<()>,
    /// the blocks drawn in the last frame
    shown: Rc<RefCell<Vec<usize>>>,
}

impl RenderedView {
    pub fn new(doc: Arc<Doc>, cx: &mut Context<Self>) -> Self {
        let list = ListState::new(doc.blocks.len(), ListAlignment::Top, px(OVERDRAW));
        let scroll = ScrollState::new(Rc::new(BlockScroll::new(list.clone())));
        Self {
            found: found_text(&doc),
            code_blocks: code_blocks(&doc),
            doc,
            revision: 0,
            list,
            scroll,
            focus_handle: cx.focus_handle(),
            marks: None,
            copied: None,
            copied_reset: Task::ready(()),
            shown: Rc::default(),
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
        self.doc = doc;
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

    /// Where typing in the find box searches from: the block at the top.
    pub fn find_anchor(&self) -> Range<usize> {
        let start = self.doc.blocks.get(self.top_block()).and_then(|block| block.pieces.first()).map_or(0, |piece| piece.text.start);
        start..start
    }

    /// Scrolls the match's block into view: a block not wholly in view goes near the top, one block above it. By index,
    /// as blocks below the view are not measured yet.
    pub fn show_match(&mut self, bytes: Range<usize>, cx: &mut Context<Self>) {
        let (block, _) = self.doc.piece_at(bytes.start);
        let view = self.list.viewport_bounds();
        let in_view = self.list.bounds_for_item(block).is_some_and(|shown| shown.top() >= view.top() && shown.bottom() <= view.bottom());
        if in_view {
            return;
        }
        self.list.scroll_to(ListOffset { item_ix: block.saturating_sub(1), offset_in_item: px(0.) });
        cx.notify();
    }

    pub fn set_find_marks(&mut self, marks: Option<FindMarks>, cx: &mut Context<Self>) {
        self.marks = marks;
        cx.notify();
    }

    #[cfg(test)]
    /// The target of the link at `offset` of the document's text.
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

    /// Code block `block`'s code to the clipboard; its button says "Copied" for a while.
    fn copy_code(&mut self, block: usize, cx: &mut Context<Self>) {
        let Some(code) = self.doc.code_of(block) else {
            return;
        };
        Host::write_clipboard(copy_target(), code, cx);
        self.copied = Some(block);
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
    fn find_ticks(&self) -> Rc<[ScrollTick]> {
        let Some(marks) = &self.marks else {
            return Rc::from([]);
        };
        let blocks = self.doc.blocks.len().max(1) as f32;
        let at = |range: &Range<usize>| self.doc.piece_at(range.start).0 as f32 / blocks;
        marks.ranges.iter().enumerate().map(|(index, range)| ScrollTick { at: at(range), current: marks.current == Some(index) }).collect()
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
        }
    }
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

fn code_blocks(doc: &Doc) -> Arc<Vec<usize>> {
    Arc::new(doc.blocks.iter().enumerate().filter(|(_, block)| matches!(block.kind, BlockKind::Code { .. })).map(|(index, _)| index).collect())
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
}

impl Frame {
    /// What lies over `piece`, by bytes of the piece.
    fn overlays(&self, piece: &Piece) -> Vec<(Range<usize>, Overlay)> {
        let Some(marks) = &self.marks else {
            return Vec::new();
        };
        let start = piece.text.start;
        let overlay = |mark| if mark == Mark::Current { Overlay::Current } else { Overlay::Match };
        marks.on_bytes(piece.text.clone()).into_iter().map(|(bytes, mark)| (bytes.start - start..bytes.end - start, overlay(mark))).collect()
    }

    /// `piece` as text, its links' targets as tooltips.
    fn piece(&self, piece: &Piece, paint: &Paint) -> AnyElement {
        let text = &self.doc.text[piece.text.clone()];
        let shown = text.strip_suffix('\n').unwrap_or(text);
        let start = piece.text.start;
        let runs: Vec<Run> = piece.runs.iter().map(|run| Run { bytes: run.bytes.start - start..run.bytes.end - start, look: run.look.clone() }).collect();
        let styled = StyledText::new(SharedString::from(shown.to_owned())).with_runs(text_runs(shown.len(), &runs, &self.overlays(piece), paint));
        let has_links = piece.runs.iter().any(|run| matches!(run.look, Look::Link(_)));
        if !has_links {
            return styled.into_any_element();
        }
        let doc = self.doc.clone();
        InteractiveText::new(("md-piece", start), styled)
            .tooltip(move |index, window, cx| {
                let target = link_at(&doc, start + index)?;
                let make = cx.try_global::<Host>()?.tooltip.clone();
                Some(make(target, window, cx))
            })
            .into_any_element()
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
        let lists = f32::from(block.nesting.lists.saturating_sub(1));
        let gap = if matches!(block.kind, BlockKind::ListItem(_)) { ITEM_GAP } else { BLOCK_GAP };
        div()
            .w_full()
            .pl(px(PAGE_MARGIN + LIST_INDENT * lists))
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

    /// A code block: its lines unwrapped in a tinted box that scrolls sideways, a Copy button in its corner.
    fn code(&self, block: &Block, index: usize, cx: &App) -> AnyElement {
        let paint = Paint { font: self.paint.mono.clone(), ..self.paint.clone() };
        let lines = div()
            .id(("md-code", index))
            .overflow_x_scroll()
            .px(px(12.))
            .py(px(10.))
            .text_size(px(CODE_TEXT_SIZE))
            .line_height(px(CODE_LINE_HEIGHT))
            .whitespace_nowrap()
            .child(self.pieces(block, &paint));
        div().relative().rounded(px(6.)).bg(self.paint.code_background).child(lines).child(self.copy_button(index, cx)).into_any_element()
    }

    fn copy_button(&self, index: usize, cx: &App) -> AnyElement {
        let number = self.code_blocks.iter().position(|block| *block == index).map_or(0, |at| at + 1);
        let label = if self.copied == Some(index) { "Copied" } else { "Copy" };
        let view = self.view.clone();
        let click = Host::click_target(format!("markdown-copy:{number}"), move |_, _, cx| drop(view.update(cx, |view, cx| view.copy_code(index, cx))), cx);
        let (text, hover) = (rgb(self.style.text), rgb(self.style.hover));
        let button = div()
            .id(("md-copy", index))
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
            // a press on it starts no selection in the code under it
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .on_click(click)
            .child(label);
        match Host::tooltip("Copy the code", cx) {
            Some(tooltip) => button.tooltip(tooltip).into_any_element(),
            None => button.into_any_element(),
        }
    }

    /// A table: its cells in equal columns, bordered, the header row bold on a tint.
    fn table(&self, block: &Block, columns: &[Align], paint: &Paint) -> AnyElement {
        let bold = Paint { font: Font { weight: FontWeight::SEMIBOLD, ..paint.font.clone() }, ..paint.clone() };
        let border = rgb(self.style.border);
        let rows = block.pieces.chunks(columns.len().max(1)).enumerate().map(|(row, cells)| {
            let cells = cells.iter().zip(columns).enumerate().map(|(column, (cell, align))| {
                let text = self.piece(cell, if row == 0 { &bold } else { paint });
                let cell = div().flex_1().min_w_0().px(px(12.)).py(px(4.)).border_color(border).when(column > 0, |cell| cell.border_l_1()).child(text);
                match align {
                    Align::Left => cell,
                    Align::Center => cell.text_center(),
                    Align::Right => cell.text_right(),
                }
            });
            let line = div().flex().border_color(border).children(cells);
            if row > 0 { line.border_t_1() } else { line.bg(self.paint.code_background) }
        });
        div().flex().flex_col().border_1().border_color(border).children(rows).into_any_element()
    }
}

/// A task item's box, ticked when filled.
fn tick_box(colour: gpui::Rgba) -> gpui::Div {
    div().mt(px(5.)).size(px(13.)).rounded(px(3.)).border_1().border_color(colour)
}

/// Room above a block: the page's top above the first, more above a heading.
fn space_above(block: &Block, index: usize) -> f32 {
    match (index, &block.kind) {
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
            .child(blocks)
            .child(vertical_scrollbar_with_ticks(&self.scroll, &style, self.find_ticks()))
    }
}
