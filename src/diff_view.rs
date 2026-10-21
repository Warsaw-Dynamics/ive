//! Draws diffs: one file in the worktree pane and diff tabs, several collapsible files in commit tabs.
//! Only visible rows are built, so a 200,000-line view scrolls as cheaply as a short one.

pub mod rows;
pub mod selection;

use std::cell::Cell;
use std::collections::HashSet;
use std::ops::Range;
use std::rc::Rc;

use gpui::{
    App, Bounds, Context, CursorStyle, DispatchPhase, Div, FocusHandle, IntoElement, KeyDownEvent, ListHorizontalSizingBehavior,
    MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, Point, Render, SharedString, Stateful, Window, canvas, div,
    prelude::*, px, rgb, rgba, uniform_list,
};

use crate::click_targets::target;
use crate::clipboard;
use crate::git::diff::{DiffBody, DiffLine, FileDiff, Hunk, LineKind};
use crate::icons;
use crate::overview::git_ui::{counted, mark_label};
use crate::script::keymap::{Action, Builtin, ClipboardTarget, Place};
use crate::script::state::bound_action;
use crate::scrollbar::{Axis, ListScroll, ScrollState, Scrollable, horizontal_scrollbar, vertical_scrollbar, widest_row};
use crate::theme::{self, Theme};
use crate::theme_config::DiffSizes;
use crate::workspace::icon;
use rows::{COLLAPSE_LINES, DiffRow, Headers, diff_rows};
use selection::{DiffPosition, DiffSelection, word_around};

/// Approximate character slots of a file header's icons, for the widest-row guess.
const HEADER_SLOTS: usize = 6;
/// Width of the `+`/`-` column, in characters.
const SIGN_CHARS: f32 = 2.;

pub struct DiffView {
    files: Vec<FileDiff>,
    headers: Headers,
    collapsed: HashSet<usize>,
    rows: Vec<DiffRow>,
    /// digits of the largest line number, for the gutter
    number_width: usize,
    list: ListScroll,
    scrollbars: ScrollState,
    focus_handle: FocusHandle,
    /// what the mouse or Select All marked; dropped when the rows change
    selection: Option<DiffSelection>,
    /// a left-button drag that started in the view goes on
    dragging: bool,
    /// the monospace font's character width, as last measured
    char_width: Pixels,
    /// the theme's diff sizes, as of the last frame
    sizes: DiffSizes,
    /// where the last frame put the view, for the mouse
    bounds: Rc<Cell<Option<Bounds<Pixels>>>>,
}

impl DiffView {
    pub fn new(headers: Headers, cx: &mut Context<Self>) -> Self {
        let list = ListScroll::new();
        let scrollbars = ScrollState::for_list(&list);
        Self {
            files: Vec::new(),
            headers,
            collapsed: HashSet::new(),
            rows: Vec::new(),
            number_width: 1,
            list,
            scrollbars,
            focus_handle: cx.focus_handle(),
            selection: None,
            dragging: false,
            char_width: px(8.),
            sizes: cx.global::<Theme>().sizes.diff,
            bounds: Rc::default(),
        }
    }

    /// Shows `files`, keeping the scroll. The same files again (a reload) keep what was collapsed; new ones in a
    /// commit tab start collapsed when over [`COLLAPSE_LINES`].
    pub fn set_files(&mut self, files: Vec<FileDiff>, cx: &mut Context<Self>) {
        let same_files = files.len() == self.files.len() && files.iter().zip(&self.files).all(|(new, old)| new.path == old.path);
        self.collapsed = match (same_files, self.headers) {
            (true, _) => std::mem::take(&mut self.collapsed),
            (false, Headers::Shown) => files.iter().enumerate().filter(|(_, file)| file.line_count() > COLLAPSE_LINES).map(|(index, _)| index).collect(),
            (false, Headers::Hidden) => HashSet::new(),
        };
        self.number_width = files.iter().map(largest_line_number).max().unwrap_or(1).to_string().len();
        self.files = files;
        self.rebuild();
        cx.notify();
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
        self.rows = diff_rows(&self.files, self.headers, &self.collapsed);
        self.selection = None;
        self.dragging = false;
    }

    /// The text of row `row` when it is a code line.
    fn line_text(&self, row: usize) -> Option<&str> {
        match self.rows.get(row)? {
            DiffRow::Line { file, hunk, line } => Some(&self.hunk(*file, *hunk).lines[*line].text),
            DiffRow::File(_) | DiffRow::Hunk { .. } | DiffRow::Note { .. } => None,
        }
    }

    fn number_column_width(&self) -> Pixels {
        self.char_width * (self.number_width + 1) as f32
    }

    /// Where a code line's text starts, from the row's left edge.
    fn text_start(&self) -> Pixels {
        self.sizes.line_padding.px() + self.number_column_width() * 2. + self.char_width * SIGN_CHARS
    }

    /// The row under window point `point` (the first or last one above or below the rows) and the column boundary
    /// of its text nearest to it; column 0 on rows without text.
    fn position_at(&self, point: Point<Pixels>) -> Option<DiffPosition> {
        let bounds = self.bounds.get()?;
        let last = self.rows.len().checked_sub(1)?;
        let y = f64::from(f32::from(point.y - bounds.origin.y)) + self.list.extent(Axis::Vertical).scrolled;
        let row = ((y / f64::from(self.sizes.row_height.0)).max(0.).floor() as usize).min(last);
        let x = f64::from(f32::from(point.x - bounds.origin.x - self.text_start())) + self.list.extent(Axis::Horizontal).scrolled;
        let column = (x / f64::from(f32::from(self.char_width))).max(0.).round() as usize;
        let chars = self.line_text(row).map_or(0, |text| text.chars().count());
        Some(DiffPosition::new(row, column.min(chars)))
    }

    fn on_mouse_down(&mut self, event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.focus_handle, cx);
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
        self.dragging = true;
        cx.notify();
    }

    /// A mouse move anywhere in the window while a drag that started here goes on: past the view's edges the
    /// selection follows the pointer to rows and columns out of view, which scrolls them in.
    fn on_window_mouse_move(&mut self, event: &MouseMoveEvent, cx: &mut Context<Self>) {
        let held = self.dragging && event.pressed_button == Some(MouseButton::Left);
        let (Some(selection), Some(bounds)) = (self.selection.filter(|_| held), self.bounds.get()) else {
            return;
        };
        self.scroll_towards(event.position, bounds);
        if let Some(position) = self.position_at(event.position) {
            self.selection = Some(selection.extended_to(position));
            cx.notify();
        }
    }

    /// Scrolls by as far as `point` is past the view's edges.
    fn scroll_towards(&self, point: Point<Pixels>, bounds: Bounds<Pixels>) {
        let past = |at: Pixels, start: Pixels, end: Pixels| f64::from(f32::from((at - start).min(px(0.)) + (at - end).max(px(0.))));
        for (axis, beyond) in [
            (Axis::Vertical, past(point.y, bounds.top(), bounds.bottom())),
            (Axis::Horizontal, past(point.x, bounds.left(), bounds.right())),
        ] {
            if beyond != 0. {
                self.list.scroll_to(axis, self.list.extent(axis).scrolled + beyond);
            }
        }
    }

    fn on_mouse_up(&mut self, _event: &MouseUpEvent, _window: &mut Window, _cx: &mut Context<Self>) {
        self.dragging = false;
    }

    /// The editor scope's Copy and Select All; other keys go on to the window's bindings.
    fn on_key_down(&mut self, event: &KeyDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        match bound_action(event, Place::Editor, cx) {
            Some(Action::Builtin(Builtin::CopyTo(target))) => self.copy(target, cx),
            Some(Action::Builtin(Builtin::SelectAll)) => self.select_all(cx),
            _ => return,
        }
        cx.stop_propagation();
    }

    fn copy(&self, target: ClipboardTarget, cx: &App) {
        let Some(selection) = self.selection else {
            return;
        };
        let text = selection.text(|row| self.line_text(row));
        if !text.is_empty() {
            clipboard::write(target, text, cx);
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
            DiffRow::Note { note, .. } => note.text().chars().count(),
        }
    }

    fn hunk(&self, file: usize, hunk: usize) -> &Hunk {
        match &self.files[file].body {
            DiffBody::Text { hunks, .. } => &hunks[hunk],
            DiffBody::Binary | DiffBody::TooLarge => unreachable!("rows only point at hunks of text diffs"),
        }
    }

    fn render_rows(&self, range: Range<usize>, theme: Theme, cx: &mut Context<Self>) -> Vec<Stateful<Div>> {
        range.map(|index| self.render_row(index, theme, cx)).collect()
    }

    fn render_row(&self, index: usize, theme: Theme, cx: &mut Context<Self>) -> Stateful<Div> {
        // as wide as the view at least, so a line's tint reaches the right edge
        let row = div().id(("diff-row", index)).flex().items_center().min_w_full().h(self.sizes.row_height.px()).whitespace_nowrap();
        match &self.rows[index] {
            DiffRow::File(file) => self.render_file_header(row, *file, cx),
            DiffRow::Hunk { file, hunk } => row.pl(self.sizes.hunk_padding_left.px()).text_color(rgb(theme.muted)).child(self.hunk(*file, *hunk).header.clone()),
            DiffRow::Line { file, hunk, line } => self.render_line(row, index, &self.hunk(*file, *hunk).lines[*line], theme),
            DiffRow::Note { note, .. } => row.pl(self.sizes.hunk_padding_left.px()).italic().text_color(rgb(theme.muted)).child(note.text()),
        }
    }

    /// A commit tab's file header; a click (target `diff-file:<path>`) collapses or expands the file.
    fn render_file_header(&self, row: Stateful<Div>, file: usize, cx: &mut Context<Self>) -> Stateful<Div> {
        let theme = crate::theme::tabs(cx);
        let diff = &self.files[file];
        let chevron = if self.collapsed.contains(&file) { icons::CHEVRON_RIGHT } else { icons::CHEVRON_DOWN };
        let name = match &diff.old_path {
            Some(old) => format!("{} → {}", old.display(), diff.path.display()),
            None => diff.path.display(),
        };
        let toggle = target(format!("diff-file:{}", diff.path.display()), cx.listener(move |view, _event, _window, cx| view.toggle(file, cx)), cx);
        row.gap(self.sizes.header_gap.px())
            .px(self.sizes.header_padding_x.px())
            .bg(rgb(theme.sidebar))
            .font_weight(self.sizes.header_weight.font_weight())
            .cursor_pointer()
            .child(icon(theme, chevron))
            .child(mark_label(diff.mark, theme, cx))
            .child(name)
            .child(div().text_color(rgb(theme.muted)).child(counted(diff.line_count(), "line", "lines")))
            .on_click(toggle)
    }

    fn render_line(&self, row: Stateful<Div>, index: usize, line: &DiffLine, theme: Theme) -> Stateful<Div> {
        let number = |value: Option<u32>| {
            let text = value.map_or_else(String::new, |value| value.to_string());
            div()
                .flex_none()
                .w(self.number_column_width())
                .pr(self.sizes.number_padding_right.px())
                .text_color(rgb(theme.muted))
                .child(SharedString::from(format!("{text:>width$}", width = self.number_width)))
        };
        let row = match line.kind {
            LineKind::Added => row.bg(rgba(theme.diff_added)),
            LineKind::Removed => row.bg(rgba(theme.diff_removed)),
            LineKind::Context => row,
            LineKind::NoNewline => row.text_color(rgb(theme.muted)),
        };
        let sign = match line.kind {
            LineKind::Added => "+",
            LineKind::Removed => "-",
            LineKind::Context | LineKind::NoNewline => " ",
        };
        let selected = self.selection.and_then(|selection| selection.columns_on(index, line.text.chars().count()));
        let text = div()
            .relative()
            .h_full()
            .flex()
            .items_center()
            .children(selected.map(|columns| self.selection_bar(columns, theme)))
            .child(line.text.clone());
        row.pl(self.sizes.line_padding.px())
            .child(number(line.old))
            .child(number(line.new))
            .child(div().flex_none().w(self.char_width * SIGN_CHARS).child(sign))
            .child(text)
    }

    /// The selection's background behind `columns` of a line; glyphs of the monospace font sit on the column grid.
    fn selection_bar(&self, columns: Range<usize>, theme: Theme) -> Div {
        div()
            .absolute()
            .top_0()
            .h_full()
            .left(self.char_width * columns.start as f32)
            .w(self.char_width * columns.len() as f32)
            .bg(rgb(theme.selected))
    }
}

fn measure_char_width(window: &Window, cx: &App) -> Pixels {
    let text_system = window.text_system();
    let font_id = text_system.resolve_font(&gpui::font(theme::mono_font(cx)));
    let font_size = cx.global::<Theme>().sizes.diff.text_size.px();
    text_system.ch_width(font_id, font_size).unwrap_or(font_size * 0.6)
}

/// Records where the view was painted, and follows a selection drag over the whole window: an element's own
/// mouse-move listener hears only moves over it.
fn follow_drag(into: Rc<Cell<Option<Bounds<Pixels>>>>, view: gpui::Entity<DiffView>) -> impl IntoElement {
    canvas(
        |_bounds, _window, _cx| {},
        move |bounds, (), window, _cx| {
            into.set(Some(bounds));
            window.on_mouse_event(move |event: &MouseMoveEvent, phase, _window, cx| {
                if phase == DispatchPhase::Bubble {
                    view.update(cx, |view, cx| view.on_window_mouse_move(event, cx));
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

impl Render for DiffView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = crate::theme::tabs(cx);
        self.char_width = measure_char_width(window, cx);
        self.sizes = cx.global::<Theme>().sizes.diff;
        let widest = widest_row(self.list.visible_rows(self.rows.len()), |index| self.row_chars(index));
        let list = uniform_list(
            ("diff-rows", cx.entity_id().as_u64()),
            self.rows.len(),
            cx.processor(move |view: &mut DiffView, range, _window, cx| view.render_rows(range, theme, cx)),
        )
        .with_horizontal_sizing_behavior(ListHorizontalSizingBehavior::Unconstrained)
        .with_width_from_item(widest)
        .track_scroll(&self.list.list_handle())
        .size_full();
        div()
            .relative()
            .flex_1()
            .min_h_0()
            .min_w_0()
            .font_family(theme::mono_font(cx))
            .text_size(self.sizes.text_size.px())
            .track_focus(&self.focus_handle)
            .cursor(CursorStyle::IBeam)
            .on_key_down(cx.listener(Self::on_key_down))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            // a drag that ends outside the view still ends
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .child(follow_drag(self.bounds.clone(), cx.entity()))
            .child(list)
            .child(vertical_scrollbar(&self.scrollbars, theme))
            .child(horizontal_scrollbar(&self.scrollbars, theme))
    }
}
