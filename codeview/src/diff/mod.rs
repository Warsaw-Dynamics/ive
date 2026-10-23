//! Draws diffs: one file in the worktree pane and diff tabs, several collapsible files in commit tabs; inline, or old
//! and new side by side. Only visible rows are built, so a 200,000-line view scrolls as cheaply as a short one.

pub mod data;
mod editing;
mod patch;
pub mod rows;
pub mod selection;
pub mod side_by_side;
mod split;
pub mod texts;

use std::cell::Cell;
use std::collections::HashSet;
use std::ops::Range;
use std::rc::Rc;

use gpui::{
    AnyElement, App, Bounds, Context, CursorStyle, DispatchPhase, Div, ElementInputHandler, EventEmitter, FocusHandle, Focusable, IntoElement,
    KeyDownEvent, ListHorizontalSizingBehavior, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, Point, Render,
    ScrollWheelEvent, SharedString, Stateful, StyledText, UniformList, Window, canvas, div, img, point, prelude::*, px, relative,
    rgb, rgba, uniform_list,
};

use crate::code_view::HighlightCache;
use crate::colors::ResolvedScheme;
use crate::document::Source;
use crate::host::{ClipboardTarget, Command, Host, KeyOutcome, KeyScope};
use crate::panel::{DiffLayout, ParseJob, ParseTarget, TextPurpose, TextsRequest};
use crate::icons;
use crate::position::ScrollPosition;
use crate::scrollbar::{
    Axis, ListScroll, OffsetScroll, ScrollState, Scrollable, horizontal_scrollbar, vertical_scrollbar, wheel_axes, widest_row,
};
use crate::style::{DiffSizes, ViewStyle};
pub use data::{Cut, DiffBody, DiffLine, FILE_LINE_LIMIT, FileDiff, GitPath, Hunk, LineKind, MAX_FILE_BYTES, Mark, VIEW_LINE_LIMIT};
pub use rows::{Headers, Shape};
use rows::{COLLAPSE_LINES, DiffRow, diff_rows};
use selection::{DiffPosition, DiffSelection, word_around};
use side_by_side::{DiffLineRef, Side};
use split::Split;
use editing::{EditMarks, Editing};
use texts::{DiffTexts, Parsed, Placed};

/// `1 file`, `2 files`.
pub fn counted(count: usize, one: &str, many: &str) -> String {
    if count == 1 {
        return format!("1 {one}");
    }
    format!("{count} {many}")
}

/// Approximate character slots of a file header's icons, for the widest-row guess.
const HEADER_SLOTS: usize = 6;
/// Width of the `+`/`-` column, in characters.
const SIGN_CHARS: f32 = 2.;
/// Width of the divider between side-by-side panels that the mouse can grab, in pixels.
const DIVIDER_GRAB: f32 = 6.;

pub enum DiffEvent {
    /// a file showed lines on screen for the first time since its diff was read: the host answers with its texts
    NeedTexts(TextsRequest),
    /// the user switched the layout
    LayoutToggled(DiffLayout),
    /// the edited new side changed
    Edited,
}

pub struct DiffView {
    files: Vec<FileDiff>,
    /// `layout` is the one shown: [`DiffLayout::Inline`] for one-sided files, else [`Self::picked`]
    shape: Shape,
    /// the layout the user picked
    picked: DiffLayout,
    collapsed: HashSet<usize>,
    rows: Vec<DiffRow>,
    /// digits of the largest line number, for the gutter
    number_width: usize,
    list: ListScroll,
    scrollbars: ScrollState,
    /// side by side, each panel's own sideways scroll
    panels: PanelScroll,
    /// side by side, how the width is shared between the panels
    split: Split,
    focus_handle: FocusHandle,
    /// what the mouse or Select All marked; dropped when the rows change
    selection: Option<DiffSelection>,
    /// side by side, the panel the selection is in
    side: Side,
    /// a left-button drag that started in the view goes on
    dragging: bool,
    /// the monospace font's character width, as last measured
    char_width: Pixels,
    /// the style's diff sizes, as of the last frame
    sizes: DiffSizes,
    /// where the last frame put the view, for the mouse
    bounds: Rc<Cell<Option<Bounds<Pixels>>>>,
    texts: DiffTexts,
    /// the colours of the rows drawn, by row; side by side, by row and panel
    colours: HighlightCache<Placed>,
    editing: Editing,
    /// what the left button pressed: rows to select, or the edited text
    pressed: Pressed,
    /// the view has the keyboard, as of the last frame: the caret shows
    focused: bool,
    /// the caret moved: the next frame scrolls it into view
    reveal_caret: bool,
}

/// What a left-button drag selects, or moves.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Pressed {
    Rows,
    Text,
    /// the divider between the side-by-side panels
    Divider,
}

/// The sideways scroll both side-by-side panels share and the scrollbars that move it.
struct PanelScroll {
    offset: OffsetScroll,
    bar: ScrollState,
}

impl PanelScroll {
    fn new() -> Self {
        let offset = OffsetScroll::default();
        Self { bar: ScrollState::new(Rc::new(offset.clone())), offset }
    }
}

/// How far the panels can scroll sideways together: far enough to see the end of the widest text on either side.
/// Each side is its viewport's and its widest text's width.
pub(crate) fn shared_max_scroll(sides: [(f64, f64); 2]) -> f64 {
    sides.into_iter().map(|(viewport, content)| content - viewport).fold(0., f64::max)
}

impl EventEmitter<DiffEvent> for DiffView {}

impl DiffView {
    pub fn new(shape: Shape, cx: &mut Context<Self>) -> Self {
        let list = ListScroll::new();
        let scrollbars = ScrollState::for_list(&list);
        Self {
            files: Vec::new(),
            picked: shape.layout,
            shape,
            collapsed: HashSet::new(),
            rows: Vec::new(),
            number_width: 1,
            list,
            scrollbars,
            panels: PanelScroll::new(),
            split: Split::default(),
            focus_handle: cx.focus_handle(),
            selection: None,
            side: Side::New,
            dragging: false,
            char_width: px(8.),
            sizes: ViewStyle::get(cx).diff,
            bounds: Rc::default(),
            texts: DiffTexts::default(),
            colours: HighlightCache::default(),
            editing: Editing::Off,
            pressed: Pressed::Rows,
            focused: false,
            reveal_caret: false,
        }
    }

    /// Shows `files`, keeping the scroll. The same files again (a reload) keep what was collapsed; new ones in a
    /// commit tab start collapsed when over [`COLLAPSE_LINES`].
    pub fn set_files(&mut self, files: Vec<FileDiff>, cx: &mut Context<Self>) {
        // the edited text's rows come from diffs of its base only
        if self.edits_rows() {
            return;
        }
        let same_files = files.len() == self.files.len() && files.iter().zip(&self.files).all(|(new, old)| new.path == old.path);
        self.collapsed = match (same_files, self.shape.headers) {
            (true, _) => std::mem::take(&mut self.collapsed),
            (false, Headers::Shown) => files.iter().enumerate().filter(|(_, file)| file.line_count() > COLLAPSE_LINES).map(|(index, _)| index).collect(),
            (false, Headers::Hidden) => HashSet::new(),
        };
        self.texts.forget_changed(&self.files, &files);
        self.files = files;
        self.shape.layout = self.shown_layout();
        self.measure_numbers();
        self.rebuild();
        cx.notify();
    }

    /// Sizes the gutter for the largest line number.
    fn measure_numbers(&mut self) {
        self.number_width = self.files.iter().map(largest_line_number).max().unwrap_or(1).to_string().len();
    }

    pub fn file(&self, file: usize) -> Option<&FileDiff> {
        self.files.get(file)
    }

    pub fn has_files(&self) -> bool {
        !self.files.is_empty()
    }

    /// The layout picked; one-sided files show inline whatever it is.
    pub fn layout(&self) -> DiffLayout {
        self.picked
    }

    /// Every file is added or deleted (see [`Mark::is_one_sided`]): the layout picked does not show.
    pub fn is_one_sided(&self) -> bool {
        self.has_files() && self.files.iter().all(|file| file.mark.is_one_sided())
    }

    fn shown_layout(&self) -> DiffLayout {
        if self.is_one_sided() {
            return DiffLayout::Inline;
        }
        self.picked
    }

    /// Picks `layout`; see [`Self::layout`].
    pub fn set_layout(&mut self, layout: DiffLayout, cx: &mut Context<Self>) {
        self.picked = layout;
        self.show_layout(self.shown_layout(), cx);
    }

    /// Lays the files out as `layout`, with the line at the top staying at the top.
    fn show_layout(&mut self, layout: DiffLayout, cx: &mut Context<Self>) {
        if layout == self.shape.layout {
            return;
        }
        let top = self.scroll_position().top_line;
        let shown = self.rows.get(top as usize).cloned();
        self.shape.layout = layout;
        self.rebuild();
        let row = shown.and_then(|shown| self.rows.iter().position(|row| row.meets(&shown))).unwrap_or(0);
        self.restore_scroll(ScrollPosition { top_line: row as f64 + top.fract(), left: 0. }, cx);
    }

    fn toggle_layout(&mut self, cx: &mut Context<Self>) {
        if self.is_one_sided() {
            return;
        }
        let layout = match self.picked {
            DiffLayout::Inline => DiffLayout::SideBySide,
            DiffLayout::SideBySide => DiffLayout::Inline,
        };
        self.set_layout(layout, cx);
        cx.emit(DiffEvent::LayoutToggled(layout));
    }

    /// The top row (with the fraction scrolled past) and the pixels scrolled sideways.
    pub fn scroll_position(&self) -> ScrollPosition {
        let top = self.list.extent(Axis::Vertical).scrolled / f64::from(self.sizes.row_height);
        ScrollPosition { top_line: top, left: self.list.extent(Axis::Horizontal).scrolled }
    }

    /// Scrolls to `position`; the list keeps it inside the rows when it is next laid out.
    pub fn restore_scroll(&mut self, position: ScrollPosition, cx: &mut Context<Self>) {
        let top = position.top_line.max(0.) * f64::from(self.sizes.row_height);
        self.list.set_offset(point(px(-(position.left.max(0.) as f32)), px(-(top as f32))));
        cx.notify();
    }

    /// Scrolls the hunk showing new-side line `line` (from 1) of the first file to the top; see
    /// [`FileDiff::hunk_at_new_line`].
    pub fn reveal_hunk_at(&mut self, line: usize, cx: &mut Context<Self>) {
        let Some(hunk) = self.files.first().and_then(|file| file.hunk_at_new_line(line)) else {
            return;
        };
        if let Some(row) = self.rows.iter().position(|row| *row == DiffRow::Hunk { file: 0, hunk }) {
            self.restore_scroll(ScrollPosition { top_line: row as f64, left: 0. }, cx);
        }
    }

    /// The collapsed files, by path.
    pub fn collapsed_paths(&self) -> Vec<String> {
        let mut paths: Vec<String> = self.collapsed.iter().map(|file| self.files[*file].path.display()).collect();
        paths.sort();
        paths
    }

    /// Collapses the files at `paths` and expands the others.
    pub fn restore_collapsed(&mut self, paths: &[String], cx: &mut Context<Self>) {
        let collapsed = self.files.iter().enumerate().filter(|(_, file)| paths.contains(&file.path.display()));
        self.collapsed = collapsed.map(|(index, _)| index).collect();
        self.rebuild();
        cx.notify();
    }

    /// Keeps `source` as side `target` of its file when it answers the last ask, `generation`, for that file; the
    /// parse that colours it.
    pub fn set_text(&mut self, target: ParseTarget, generation: u64, source: Option<Source>, cx: &mut Context<Self>) -> Option<ParseJob> {
        let file = match target {
            ParseTarget::DiffOld(file) | ParseTarget::DiffNew(file) => self.files.get(file)?,
            ParseTarget::Text => return None,
        };
        let job = self.texts.set(target, generation, source?, file);
        self.colours = HighlightCache::default();
        cx.notify();
        job
    }

    /// Colours a side with `parsed`, unless its text was replaced meanwhile.
    pub fn parsed(&mut self, parsed: Parsed, cx: &mut Context<Self>) {
        if self.texts.parsed(parsed) {
            self.colours = HighlightCache::default();
            cx.notify();
        }
    }

    /// Asks for the texts of the files showing lines on screen that it has not asked for yet, and for the text to
    /// edit while it waits for one, rows or not (a diff with no changes left still edits its file). Run after a
    /// frame's layout: the rows the list laid out to measure them are not all on screen.
    fn request_texts(&mut self, cx: &mut Context<Self>) {
        let shown = self.list.visible_rows(self.rows.len());
        let waiting = self.waits_for_text();
        let purpose = |file: usize| if waiting && file == 0 { TextPurpose::Editing } else { TextPurpose::Colours };
        let to_edit = if waiting { self.texts.want(0, TextPurpose::Editing) } else { None };
        for request in to_edit.into_iter().chain(self.texts.wanted(&self.rows[shown], &purpose)) {
            cx.emit(DiffEvent::NeedTexts(request));
        }
    }

    /// The colours of inline row `row`'s text: none for rows without code, and until its side's text is parsed.
    pub(crate) fn row_colours(&self, row: usize) -> Rc<[Placed]> {
        let DiffRow::Line { file, hunk, line } = &self.rows[row] else {
            return Rc::from([]);
        };
        let line = self.coloured_line(&self.hunk(*file, *hunk).lines[*line]);
        self.colours.get_or_query(row, || line.map_or_else(Vec::new, |line| self.texts.colours(*file, &line)))
    }

    /// The colours of panel `side` of side-by-side row `row`: none for fillers and rows without code, and until the
    /// side's text is parsed.
    pub(crate) fn cell_colours(&self, row: usize, side: Side) -> Rc<[Placed]> {
        let DiffRow::Pair { file, row: pair } = &self.rows[row] else {
            return Rc::from([]);
        };
        let Some(at) = pair.on(side) else {
            return Rc::from([]);
        };
        let line = self.coloured_line(self.line(*file, at));
        self.colours.get_or_query(2 * row + side as usize, || line.map_or_else(Vec::new, |line| self.texts.colours_on(*file, &line, side)))
    }

    /// Copy, Select All and Toggle Layout; while the new side is edited, the editing commands too.
    pub fn run(&mut self, command: Command, cx: &mut Context<Self>) {
        match command {
            Command::Copy(target) => self.copy(target, cx),
            Command::SelectAll => self.select_all(cx),
            Command::ToggleLayout => self.toggle_layout(cx),
            command => self.run_editing(command, cx),
        }
    }

    fn toggle(&mut self, file: usize, cx: &mut Context<Self>) {
        if self.collapsed.contains(&file) {
            self.collapsed.remove(&file);
        } else {
            self.collapsed.insert(file);
        }
        self.rebuild();
        cx.notify();
    }

    fn rebuild(&mut self) {
        self.rows = diff_rows(&self.files, self.shape, &self.collapsed);
        self.colours = HighlightCache::default();
        self.selection = None;
        self.dragging = false;
    }

    /// The code line of row `row`; side by side, the one in the selection's panel.
    fn line_at(&self, row: usize) -> Option<&DiffLine> {
        match self.rows.get(row)? {
            DiffRow::Line { file, hunk, line } => Some(&self.hunk(*file, *hunk).lines[*line]),
            DiffRow::Pair { file, row } => row.on(self.side).map(|at| self.line(*file, at)),
            DiffRow::File(_) | DiffRow::Hunk { .. } | DiffRow::Note { .. } => None,
        }
    }

    /// The text of row `row` when it is a code line; side by side, in the selection's panel.
    fn line_text(&self, row: usize) -> Option<&str> {
        self.line_at(row).map(|line| line.text.as_str())
    }

    fn number_column_width(&self) -> Pixels {
        self.char_width * (self.number_width + 1) as f32
    }

    /// Where a code line's text starts, from the left edge of its row: inline after both line numbers and the sign;
    /// side by side, from its panel's left edge, after its side's number (the tint tells the sign).
    fn text_start(&self) -> Pixels {
        let gutter = match self.shape.layout {
            DiffLayout::Inline => self.number_column_width() * 2. + self.char_width * SIGN_CHARS,
            DiffLayout::SideBySide => self.number_column_width(),
        };
        px(self.sizes.line_padding) + gutter
    }

    /// The window's horizontal range of the text the selection is in: the view's inline, its panel's side by side.
    fn text_span(&self, bounds: Bounds<Pixels>) -> Range<Pixels> {
        match self.shape.layout {
            DiffLayout::Inline => bounds.left()..bounds.right(),
            DiffLayout::SideBySide => self.split.span(self.side, bounds),
        }
    }

    /// How far the side-by-side panels' text is scrolled sideways, which is the same for both.
    #[cfg(test)]
    pub(crate) fn text_offset(&self) -> f64 {
        self.panels.offset.scrolled()
    }

    /// What scrolls the selection's text sideways.
    fn sideways(&self) -> &dyn Scrollable {
        match self.shape.layout {
            DiffLayout::Inline => &self.list,
            DiffLayout::SideBySide => &self.panels.offset,
        }
    }

    /// The side-by-side panel under window x `x`.
    fn side_at(&self, x: Pixels, bounds: Bounds<Pixels>) -> Side {
        self.split.side_at(x, bounds)
    }

    /// A press on the divider between the panels: a drag moves it, a double-click puts it back in the middle.
    pub(crate) fn on_divider_down(&mut self, event: &MouseDownEvent, cx: &mut Context<Self>) {
        cx.stop_propagation();
        self.dragging = true;
        self.pressed = Pressed::Divider;
        if event.click_count == 2 {
            self.split = Split::default();
        }
        cx.notify();
    }

    /// The row under window point `point` (the first or last one above or below the rows) and the column boundary
    /// of its text nearest to it; column 0 on rows without text. Side by side, in the selection's panel.
    fn position_at(&self, point: Point<Pixels>) -> Option<DiffPosition> {
        let bounds = self.bounds.get()?;
        let last = self.rows.len().checked_sub(1)?;
        let y = f64::from(f32::from(point.y - bounds.origin.y)) + self.list.extent(Axis::Vertical).scrolled;
        let row = ((y / f64::from(self.sizes.row_height)).max(0.).floor() as usize).min(last);
        let text_left = self.text_span(bounds).start + self.text_start();
        let x = f64::from(f32::from(point.x - text_left)) + self.sideways().extent(Axis::Horizontal).scrolled;
        let column = (x / f64::from(f32::from(self.char_width))).max(0.).round() as usize;
        let chars = self.line_text(row).map_or(0, |text| text.chars().count());
        Some(DiffPosition::new(row, column.min(chars)))
    }

    pub(crate) fn on_mouse_down(&mut self, event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.focus_handle, cx);
        self.pick_side(event);
        self.dragging = true;
        if self.press_text(event) {
            self.selection = None;
            self.pressed = Pressed::Text;
            return cx.notify();
        }
        self.deselect_text();
        self.pressed = Pressed::Rows;
        let Some(position) = self.position_at(event.position) else {
            return;
        };
        let selection = match event.click_count {
            2 => {
                let word = word_around(self.line_text(position.row).unwrap_or_default(), position.column);
                DiffSelection::of(DiffPosition::new(position.row, word.start)..DiffPosition::new(position.row, word.end))
            }
            3.. => DiffSelection::of(DiffPosition::new(position.row, 0)..DiffPosition::new(position.row + 1, 0)),
            _ if event.modifiers.shift => self.selection.unwrap_or(DiffSelection::at(position)).extended_to(position),
            _ => DiffSelection::at(position),
        };
        self.selection = Some(selection);
        cx.notify();
    }

    /// Side by side, a press starts a selection in the panel under it; Shift extends the one there is.
    fn pick_side(&mut self, event: &MouseDownEvent) {
        let press = (self.shape.layout, event.modifiers.shift, self.bounds.get());
        if let (DiffLayout::SideBySide, false, Some(bounds)) = press {
            self.side = self.side_at(event.position.x, bounds);
        }
    }

    /// A mouse move anywhere in the window while a drag that started here goes on: past the view's edges the
    /// selection follows the pointer to rows and columns out of view, which scrolls them in.
    pub(crate) fn on_window_mouse_move(&mut self, event: &MouseMoveEvent, cx: &mut Context<Self>) {
        let held = self.dragging && event.pressed_button == Some(MouseButton::Left);
        if let (true, Pressed::Divider, Some(bounds)) = (held, self.pressed, self.bounds.get()) {
            self.split = Split::at(event.position.x, bounds);
            return cx.notify();
        }
        if let (true, Pressed::Text, Some(bounds)) = (held, self.pressed, self.bounds.get()) {
            self.scroll_towards(event.position, bounds);
            self.drag_text(event);
            return cx.notify();
        }
        let (Some(selection), Some(bounds)) = (self.selection.filter(|_| held), self.bounds.get()) else {
            return;
        };
        self.scroll_towards(event.position, bounds);
        if let Some(position) = self.position_at(event.position) {
            self.selection = Some(selection.extended_to(position));
            cx.notify();
        }
    }

    /// Scrolls by as far as `point` is past the view's edges; sideways, past the edges of the selection's text.
    fn scroll_towards(&self, point: Point<Pixels>, bounds: Bounds<Pixels>) {
        let past = |at: Pixels, span: Range<Pixels>| f64::from(f32::from((at - span.start).min(px(0.)) + (at - span.end).max(px(0.))));
        for (target, axis, beyond) in [
            (&self.list as &dyn Scrollable, Axis::Vertical, past(point.y, bounds.top()..bounds.bottom())),
            (self.sideways(), Axis::Horizontal, past(point.x, self.text_span(bounds))),
        ] {
            if beyond != 0. {
                target.scroll_to(axis, target.extent(axis).scrolled + beyond);
            }
        }
    }

    /// Side by side, a sideways wheel (or Shift+wheel) over either panel scrolls both.
    pub(crate) fn on_window_wheel(&mut self, event: &ScrollWheelEvent, window: &Window, cx: &mut Context<Self>) {
        if !self.bounds.get().is_some_and(|bounds| bounds.contains(&event.position)) {
            return;
        }
        let delta = event.delta.pixel_delta(window.line_height());
        let (sideways, _) = wheel_axes(f32::from(delta.x), f32::from(delta.y), event.modifiers.shift);
        if sideways == 0. {
            return;
        }
        let offset = &self.panels.offset;
        offset.scroll_to(Axis::Horizontal, offset.scrolled() - f64::from(sideways));
        if delta.x == px(0.) {
            // a Shift+wheel turned sideways: keep the list from also scrolling down
            cx.stop_propagation();
        }
        cx.notify();
    }

    fn on_mouse_up(&mut self, _event: &MouseUpEvent, _window: &mut Window, _cx: &mut Context<Self>) {
        self.dragging = false;
    }

    /// The host's Copy, Select All and Toggle Layout; other keys go on to the window's bindings. While the new side
    /// is edited, keys work as in an editor.
    fn on_key_down(&mut self, event: &KeyDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.edited().is_some() {
            return self.on_editing_key(event, cx);
        }
        match Host::key_outcome(event, KeyScope::Viewer, cx) {
            KeyOutcome::Command(command @ (Command::Copy(_) | Command::SelectAll | Command::ToggleLayout)) => self.run(command, cx),
            KeyOutcome::Handled => {}
            // the diff has no other commands; a window action is the workspace's to run
            KeyOutcome::Command(_) | KeyOutcome::Foreign | KeyOutcome::Unbound => return,
        }
        cx.stop_propagation();
    }

    /// Copies the selected text: the edited text's selection, else the rows' (side by side, of the selection's panel
    /// only).
    fn copy(&self, target: ClipboardTarget, cx: &mut App) {
        if self.copy_edited(target, cx) {
            return;
        }
        let Some(selection) = self.selection else {
            return;
        };
        let text = selection.text(|row| self.line_text(row));
        if !text.is_empty() {
            Host::write_clipboard(target, text, cx);
        }
    }

    fn select_all(&mut self, cx: &mut Context<Self>) {
        let Some(last) = self.rows.len().checked_sub(1) else {
            return;
        };
        let end = DiffPosition::new(last, self.line_text(last).map_or(0, |text| text.chars().count()));
        self.selection = Some(DiffSelection::of(DiffPosition::new(0, 0)..end));
        cx.notify();
    }

    fn row_chars(&self, index: usize) -> usize {
        match &self.rows[index] {
            DiffRow::File(file) => HEADER_SLOTS + self.files[*file].path.display().chars().count(),
            DiffRow::Hunk { file, hunk } => self.hunk(*file, *hunk).header.chars().count(),
            DiffRow::Line { file, hunk, line } => 2 * self.number_width + 4 + self.hunk(*file, *hunk).lines[*line].text.chars().count(),
            DiffRow::Pair { .. } => self.cell_chars(index, Side::Old) + self.cell_chars(index, Side::New),
            DiffRow::Note { note, .. } => note.text().chars().count(),
        }
    }

    /// The characters of the text in panel `side` of row `index`; none outside side-by-side code rows.
    fn cell_chars(&self, index: usize, side: Side) -> usize {
        match &self.rows[index] {
            DiffRow::Pair { file, row } => row.on(side).map_or(0, |at| self.line(*file, at).text.chars().count()),
            DiffRow::File(_) | DiffRow::Hunk { .. } | DiffRow::Line { .. } | DiffRow::Note { .. } => 0,
        }
    }

    fn hunk(&self, file: usize, hunk: usize) -> &Hunk {
        match &self.files[file].body {
            DiffBody::Text { hunks, .. } => &hunks[hunk],
            DiffBody::Binary | DiffBody::TooLarge => unreachable!("rows only point at hunks of text diffs"),
        }
    }

    fn line(&self, file: usize, at: DiffLineRef) -> &DiffLine {
        &self.hunk(file, at.hunk).lines[at.line]
    }

    /// Fits the sideways scroll the side-by-side panels share to the widest text either shows now. Its viewport is the
    /// new panel's, which the caret is kept in; the old panel's text may end sooner or later than that.
    fn size_panels(&self) {
        let Some(bounds) = self.bounds.get() else {
            return;
        };
        let shown = self.list.visible_rows(self.rows.len());
        let [old, new] = [Side::Old, Side::New].map(|side| {
            let viewport = f64::from(f32::from(bounds.size.width * self.split.share(side) - self.text_start()));
            let chars = shown.clone().map(|row| self.cell_chars(row, side)).max().unwrap_or(0);
            // one column more for a selection that goes on past the end
            (viewport, f64::from(f32::from(self.char_width)) * (chars + 1) as f64)
        });
        self.panels.offset.set_size(new.0, new.0 + shared_max_scroll([old, new]));
    }

    fn render_rows(&self, range: Range<usize>, theme: &ViewStyle, cx: &mut Context<Self>) -> Vec<Stateful<Div>> {
        range.map(|index| self.render_row(index, theme, cx)).collect()
    }

    fn render_row(&self, index: usize, theme: &ViewStyle, cx: &mut Context<Self>) -> Stateful<Div> {
        // as wide as the view at least, so a line's tint reaches the right edge
        let row = div().id(("diff-row", index)).flex().items_center().min_w_full().h(px(self.sizes.row_height)).whitespace_nowrap();
        match &self.rows[index] {
            DiffRow::File(file) => self.render_file_header(row, *file, theme, cx),
            DiffRow::Hunk { file, hunk } => row.pl(px(self.sizes.hunk_padding_left)).text_color(rgb(theme.muted)).child(self.hunk(*file, *hunk).header.clone()),
            DiffRow::Line { file, hunk, line } => {
                let line = &self.hunk(*file, *hunk).lines[*line];
                self.render_line(row, CodeRow { index, line, text: self.styled_text(line, self.row_colours(index), cx) }, theme)
            }
            DiffRow::Pair { file, row: pair } => {
                let cell = |side: Side| PanelCell { index, side, line: pair.on(side).map(|at| self.line(*file, at)) };
                row.child(self.render_cell(cell(Side::Old), theme, cx)).child(self.render_cell(cell(Side::New), theme, cx))
            }
            DiffRow::Note { note, .. } => row.pl(px(self.sizes.hunk_padding_left)).italic().text_color(rgb(theme.muted)).child(note.text()),
        }
    }

    /// A commit tab's file header; a click (target `diff-file:<path>`) collapses or expands the file.
    fn render_file_header(&self, row: Stateful<Div>, file: usize, theme: &ViewStyle, cx: &mut Context<Self>) -> Stateful<Div> {
        let diff = &self.files[file];
        let chevron = if self.collapsed.contains(&file) { icons::CHEVRON_RIGHT } else { icons::CHEVRON_DOWN };
        let name = match &diff.old_path {
            Some(old) => format!("{} → {}", old.display(), diff.path.display()),
            None => diff.path.display(),
        };
        let toggle = Host::click_target(format!("diff-file:{}", diff.path.display()), cx.listener(move |view, _event, _window, cx| view.toggle(file, cx)), cx);
        row.gap(px(self.sizes.header_gap))
            .px(px(self.sizes.header_padding_x))
            .bg(rgb(theme.sidebar))
            .font_weight(self.sizes.header_weight)
            .cursor_pointer()
            .child(img(icons::embedded(theme.appearance, chevron)).flex_none().size(px(theme.icon_size)))
            .children(Host::diff_mark(diff.mark, cx))
            .child(name)
            .child(div().text_color(rgb(theme.muted)).child(counted(diff.line_count(), "line", "lines")))
            .on_click(toggle)
    }

    /// `line`'s text in `colours`.
    fn styled_text(&self, line: &DiffLine, colours: Rc<[Placed]>, cx: &App) -> StyledText {
        let text = StyledText::new(line.text.clone());
        if colours.is_empty() {
            return text;
        }
        let scheme = cx.global::<ResolvedScheme>();
        text.with_highlights(colours.iter().map(|(range, key)| (range.clone(), scheme.get(*key).highlight_style())))
    }

    fn render_line(&self, row: Stateful<Div>, code: CodeRow<'_>, theme: &ViewStyle) -> Stateful<Div> {
        let CodeRow { index, line, text } = code;
        let marks = self.edit_marks(line);
        let selected = self.selection.and_then(|selection| selection.columns_on(index, line.text.chars().count())).or(marks.selected.clone());
        let text = div()
            .relative()
            .h_full()
            .flex()
            .items_center()
            .children(selected.map(|columns| self.selection_bar(columns, theme)))
            .child(text)
            .children(self.caret_bar(&marks, theme));
        tinted(row, line.kind, theme)
            .pl(px(self.sizes.line_padding))
            .child(self.line_number(line.old, theme))
            .child(self.line_number(line.new, theme))
            .child(div().flex_none().w(self.char_width * SIGN_CHARS).child(sign(line.kind)))
            .child(text)
    }

    /// One panel of a side-by-side row: its side's line number and text, scrolled sideways on its own; a filler when
    /// the side has no line there.
    fn render_cell(&self, cell: PanelCell<'_>, theme: &ViewStyle, cx: &App) -> Div {
        // a basis of 0 px, not flex_1's 0 %: the row's width is not definite, and the panels must take their shares of
        // it whatever they hold; the padding is a child, as the panel's own would be added to its share
        let panel = div().flex().flex_grow(self.split.share(cell.side)).flex_basis(px(0.)).min_w_0().h_full().items_center().overflow_hidden();
        let Some(line) = cell.line else {
            // the muted text colour, faint
            return panel.bg(rgba((theme.muted << 8) | 0x1a));
        };
        let number = match cell.side {
            Side::Old => line.old,
            Side::New => line.new,
        };
        let selection = self.selection.filter(|_| self.side == cell.side);
        let marks = match cell.side {
            Side::Old => EditMarks { selected: None, caret: None },
            Side::New => self.edit_marks(line),
        };
        let selected = selection.and_then(|selection| selection.columns_on(cell.index, line.text.chars().count())).or(marks.selected.clone());
        let text = div()
            .absolute()
            .top_0()
            .h_full()
            .left(-px(self.panels.offset.scrolled() as f32))
            .flex()
            .items_center()
            .children(selected.map(|columns| self.selection_bar(columns, theme)))
            .child(self.styled_text(line, self.cell_colours(cell.index, cell.side), cx))
            .children(self.caret_bar(&marks, theme));
        tinted(panel, line.kind, theme)
            .child(div().flex_none().w(px(self.sizes.line_padding)))
            .child(self.line_number(number, theme))
            .child(div().relative().flex_1().min_w_0().h_full().overflow_hidden().child(text))
    }

    fn line_number(&self, value: Option<u32>, theme: &ViewStyle) -> Div {
        let text = value.map_or_else(String::new, |value| value.to_string());
        div()
            .flex_none()
            .w(self.number_column_width())
            .pr(px(self.sizes.number_padding_right))
            .text_color(rgb(theme.muted))
            .child(SharedString::from(format!("{text:>width$}", width = self.number_width)))
    }

    /// The selection's background behind `columns` of a line; glyphs of the monospace font sit on the column grid.
    fn selection_bar(&self, columns: Range<usize>, theme: &ViewStyle) -> Div {
        div()
            .absolute()
            .top_0()
            .h_full()
            .left(self.char_width * columns.start as f32)
            .w(self.char_width * columns.len() as f32)
            .bg(rgb(theme.selected))
    }

    /// The edited text's caret on a row, at its display column.
    fn caret_bar(&self, marks: &EditMarks, theme: &ViewStyle) -> Option<Div> {
        let column = marks.caret?;
        Some(div().absolute().top_0().h_full().left(self.char_width * column as f32).w(px(theme.editor_caret_width)).bg(rgb(theme.text)))
    }

    /// The list of rows: inline as wide as the widest row on screen, side by side as wide as the view, each panel
    /// scrolling sideways on its own.
    fn render_list(&self, theme: &ViewStyle, cx: &mut Context<Self>) -> UniformList {
        let (sizing, widest) = match self.shape.layout {
            DiffLayout::Inline => {
                (ListHorizontalSizingBehavior::Unconstrained, widest_row(self.list.visible_rows(self.rows.len()), |index| self.row_chars(index)))
            }
            DiffLayout::SideBySide => (ListHorizontalSizingBehavior::FitList, None),
        };
        let theme = theme.clone();
        uniform_list(
            ("diff-rows", cx.entity_id().as_u64()),
            self.rows.len(),
            cx.processor(move |view: &mut DiffView, range, _window, cx| view.render_rows(range, &theme, cx)),
        )
        .with_horizontal_sizing_behavior(sizing)
        .with_width_from_item(widest)
        .track_scroll(&self.list.list_handle())
        .size_full()
    }

    /// The horizontal scrollbars: the list's inline, one under each panel side by side.
    fn horizontal_scrollbars(&self, theme: &ViewStyle) -> Vec<AnyElement> {
        let under = |side: Side| {
            let bar = horizontal_scrollbar(&self.panels.bar, theme);
            div().absolute().top_0().bottom_0().w(relative(self.split.share(side))).child(bar)
        };
        match self.shape.layout {
            DiffLayout::Inline => vec![horizontal_scrollbar(&self.scrollbars, theme).into_any_element()],
            DiffLayout::SideBySide => vec![under(Side::Old).left_0().into_any_element(), under(Side::New).right_0().into_any_element()],
        }
    }

    /// Side by side, the line between the panels down the whole view, and the strip around it that drags it.
    fn render_divider(&self, theme: &ViewStyle, cx: &mut Context<Self>) -> Option<Div> {
        if self.shape.layout == DiffLayout::Inline {
            return None;
        }
        let moving = self.dragging && self.pressed == Pressed::Divider;
        let line = div().absolute().top_0().bottom_0().left(px(DIVIDER_GRAB / 2.)).w(px(1.)).bg(rgb(if moving { theme.accent } else { theme.border }));
        let grab = div()
            .group("diff-divider")
            .absolute()
            .top_0()
            .bottom_0()
            .left(px(-DIVIDER_GRAB / 2.))
            .w(px(DIVIDER_GRAB))
            .cursor(CursorStyle::ResizeLeftRight)
            .on_mouse_down(MouseButton::Left, cx.listener(|view, event, _window, cx| view.on_divider_down(event, cx)))
            .child(line.group_hover("diff-divider", |line| line.bg(rgb(theme.accent))));
        Some(div().absolute().top_0().bottom_0().left(relative(self.split.share(Side::Old))).w(px(0.)).child(grab))
    }
}

/// A code line's row: its index, the line, and its text as drawn.
struct CodeRow<'a> {
    index: usize,
    line: &'a DiffLine,
    text: StyledText,
}

/// Panel `side` of side-by-side row `index`: its line, `None` for a filler.
struct PanelCell<'a> {
    index: usize,
    side: Side,
    line: Option<&'a DiffLine>,
}

/// `element` in the colours of a line of `kind`.
fn tinted<E: Styled>(element: E, kind: LineKind, theme: &ViewStyle) -> E {
    match kind {
        LineKind::Added => element.bg(rgba(theme.diff_added)),
        LineKind::Removed => element.bg(rgba(theme.diff_removed)),
        LineKind::Context => element,
        LineKind::NoNewline => element.text_color(rgb(theme.muted)),
    }
}

fn sign(kind: LineKind) -> &'static str {
    match kind {
        LineKind::Added => "+",
        LineKind::Removed => "-",
        LineKind::Context | LineKind::NoNewline => " ",
    }
}

fn measure_char_width(window: &Window, cx: &App) -> Pixels {
    let text_system = window.text_system();
    let font_id = text_system.resolve_font(&gpui::font(ViewStyle::drawing(cx).mono_font.clone()));
    let font_size = px(ViewStyle::get(cx).diff.text_size);
    text_system.ch_advance(font_id, font_size).unwrap_or(font_size * 0.6)
}

/// Records where the view was painted, and follows a selection drag and the sideways wheel over side-by-side panels
/// across the whole window: an element's own listeners hear only events over it, and only after the list took them.
fn follow_mouse(into: Rc<Cell<Option<Bounds<Pixels>>>>, view: gpui::Entity<DiffView>) -> impl IntoElement {
    canvas(
        |_bounds, _window, _cx| {},
        move |bounds, (), window, cx| {
            into.set(Some(bounds));
            let dragged = view.clone();
            window.on_mouse_event(move |event: &MouseMoveEvent, phase, _window, cx| {
                if phase == DispatchPhase::Bubble {
                    dragged.update(cx, |view, cx| view.on_window_mouse_move(event, cx));
                }
            });
            // while the new side is edited, the platform's text input (an input method too) types into it
            if let Some(focus) = view.read(cx).input_focus() {
                window.handle_input(&focus, ElementInputHandler::new(bounds, view.clone()), cx);
            }
            window.on_mouse_event(move |event: &ScrollWheelEvent, phase, window, cx| {
                if phase == DispatchPhase::Capture && view.read(cx).shape.layout == DiffLayout::SideBySide {
                    view.update(cx, |view, cx| view.on_window_wheel(event, window, cx));
                }
            });
        },
    )
    .absolute()
    .size_full()
}

fn largest_line_number(file: &FileDiff) -> u32 {
    match &file.body {
        DiffBody::Text { hunks, .. } => hunks.iter().flat_map(|hunk| &hunk.lines).flat_map(|line| line.old.into_iter().chain(line.new)).max().unwrap_or(1),
        DiffBody::Binary | DiffBody::TooLarge => 1,
    }
}

impl Focusable for DiffView {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for DiffView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = ViewStyle::drawing(cx).clone();
        self.char_width = measure_char_width(window, cx);
        self.sizes = ViewStyle::get(cx).diff;
        self.colours.start_frame();
        self.focused = self.focus_handle.is_focused(window);
        cx.defer_in(window, |view, _window, cx| view.request_texts(cx));
        self.size_panels();
        self.reveal();
        div()
            .relative()
            .flex_1()
            .min_h_0()
            .min_w_0()
            .font_family(theme.mono_font.clone())
            .text_size(px(self.sizes.text_size))
            .track_focus(&self.focus_handle)
            .cursor(if self.dragging && self.pressed == Pressed::Divider { CursorStyle::ResizeLeftRight } else { CursorStyle::IBeam })
            .on_key_down(cx.listener(Self::on_key_down))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            // a drag that ends outside the view still ends
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .child(follow_mouse(self.bounds.clone(), cx.entity()))
            .child(self.render_list(&theme, cx))
            .children(self.render_divider(&theme, cx))
            .child(vertical_scrollbar(&self.scrollbars, &theme))
            .children(self.horizontal_scrollbars(&theme))
    }
}
