//! What a diff gives the panel's find box: its searched lines as one text, the selection and the top row as
//! places in it, and a match selected and scrolled to; see `crate::panel::find`.

use std::ops::Range;
use std::rc::Rc;

use gpui::{Context, SharedString};

use super::rows::DiffRow;
use super::searched::Searched;
use super::selection::{DiffPosition, DiffSelection};
use super::side_by_side::{DiffLineRef, Side};
use super::{DiffEvent, DiffView};
use crate::find::FindText;
use crate::find::marks::{FindMarks, Mark, text_columns};
use crate::panel::DiffLayout;
use crate::scrollbar::{Axis, Scrollable, Tick};

impl DiffView {
    pub fn find_revision(&self) -> u64 {
        self.find_revision
    }

    pub fn find_text(&self) -> (u64, FindText) {
        (self.find_revision, self.searched().find_text())
    }

    /// The rows' selection when on one row, and the byte of the searched text it starts at.
    pub fn find_prefill(&self) -> Option<(String, usize)> {
        let range = self.selection?.range();
        if range.start.row != range.end.row {
            return None;
        }
        let text = self.selected_text().filter(|text| !text.is_empty())?;
        Some((text, self.byte_at(range.start)?))
    }

    /// The selection's bytes in the searched text, else the start of the first searched line from the top row.
    pub fn find_anchor(&self) -> Range<usize> {
        let selected = self.selection.and_then(|selection| {
            let range = selection.range();
            Some(self.byte_at(range.start)?..self.byte_at(range.end)?)
        });
        if let Some(selected) = selected {
            return selected;
        }
        let top = self.scroll_position().top_line as usize;
        let first = (top..self.rows.len()).find_map(|row| self.searched_line_at(row, self.side).or_else(|| self.collapsed_line_at(row)));
        let at = first.map_or(0, |index| self.searched().start(index));
        at..at
    }

    /// Selects bytes `bytes` of the searched text in the rows, expanding a collapsed file first, and scrolls the
    /// least to show them.
    pub fn show_match(&mut self, bytes: Range<usize>, cx: &mut Context<Self>) {
        let (Some(start), Some(end)) = (self.position_of(bytes.start), self.position_of(bytes.end)) else {
            return;
        };
        self.selection = Some(DiffSelection::of(start..end));
        self.reveal_position(start);
        cx.notify();
    }

    pub fn set_find_marks(&mut self, marks: Option<FindMarks>, cx: &mut Context<Self>) {
        self.find_marks = marks;
        cx.notify();
    }

    #[cfg(test)]
    pub(crate) fn has_find_marks(&self) -> bool {
        self.find_marks.is_some()
    }

    /// Side by side, which side the find box searches.
    pub fn find_detail(&self) -> Option<SharedString> {
        match (self.shape.layout, self.side) {
            (DiffLayout::Inline, _) => None,
            (DiffLayout::SideBySide, Side::Old) => Some("Searching the left side; click the right one to search it".into()),
            (DiffLayout::SideBySide, Side::New) => Some("Searching the right side; click the left one to search it".into()),
        }
    }

    pub fn selected_text(&self) -> Option<String> {
        Some(self.selection?.text(|row| self.line_text(row))).filter(|text| !text.is_empty())
    }

    /// Side by side, the find box searches `side` from now on, as after a press in that panel.
    #[cfg(test)]
    pub(crate) fn search_side(&mut self, side: Side, cx: &mut Context<Self>) {
        let before = self.side;
        self.side = side;
        // a press there would place a new selection, on the side pressed
        self.selection = None;
        self.searched_side_changed(before, cx);
    }

    /// After a press picked a panel: a side-by-side diff's searched lines are the other side's when it changed.
    pub(crate) fn searched_side_changed(&mut self, before: Side, cx: &mut Context<Self>) {
        if self.side == before || self.shape.layout == DiffLayout::Inline {
            return;
        }
        self.find_revision += 1;
        cx.emit(DiffEvent::SearchedChanged);
    }

    /// The find box's matches on row `row`'s line in panel `side` (inline: its only one), by character columns;
    /// none for the side not searched.
    pub(crate) fn find_columns_at(&self, row: usize, side: Side, text: &str) -> Vec<(Range<usize>, Mark)> {
        let searched_side = self.shape.layout == DiffLayout::Inline || side == self.side;
        let (Some(marks), true) = (&self.find_marks, searched_side) else {
            return Vec::new();
        };
        let Some(index) = self.searched_line_at(row, side) else {
            return Vec::new();
        };
        text_columns(text, self.searched().start(index), marks)
    }

    /// A scrollbar tick for each match, at the row its line is on (a collapsed file's: its header).
    pub(crate) fn find_ticks(&self) -> Rc<[Tick]> {
        let Some(marks) = &self.find_marks else {
            return Rc::from([]);
        };
        let rows = self.searched_rows();
        let count = self.rows.len().max(1) as f32;
        marks.ticks(|line| rows.get(line).copied().unwrap_or(0) as f32 / count)
    }

    /// The row of each searched line: its own, else (in a collapsed file) its file's header; built again once the
    /// searched text or the rows changed.
    fn searched_rows(&self) -> Rc<Vec<usize>> {
        let key = super::TickRowsKey { revision: self.find_revision, rows: self.rows.len(), collapsed: self.collapsed.len() };
        if let Some((built, rows)) = self.tick_rows.borrow().as_ref()
            && *built == key
        {
            return rows.clone();
        }
        let searched = self.searched();
        let headers: std::collections::HashMap<usize, usize> =
            self.rows.iter().enumerate().filter_map(|(row, shown)| if let DiffRow::File(file) = shown { Some((*file, row)) } else { None }).collect();
        let mut rows: Vec<usize> = (0..searched.len()).map(|index| headers.get(&searched.line(index).0).copied().unwrap_or(0)).collect();
        for row in 0..self.rows.len() {
            if let Some(index) = self.searched_line_at(row, self.side) {
                rows[index] = row;
            }
        }
        let rows = Rc::new(rows);
        *self.tick_rows.borrow_mut() = Some((key, rows.clone()));
        rows
    }

    /// The lines searched now; built again once what is searched changed.
    fn searched(&self) -> Rc<Searched> {
        let mut cache = self.searched.borrow_mut();
        if let Some((revision, searched)) = cache.as_ref()
            && *revision == self.find_revision
        {
            return searched.clone();
        }
        let searched = Rc::new(Searched::of(&self.files, self.shape.layout, self.side));
        *cache = Some((self.find_revision, searched.clone()));
        searched
    }

    /// The searched line row `row` shows in panel `side` (inline: its line).
    fn searched_line_at(&self, row: usize, side: Side) -> Option<usize> {
        let (file, at) = match self.rows.get(row)? {
            DiffRow::Line { file, hunk, line } => (*file, DiffLineRef { hunk: *hunk, line: *line }),
            DiffRow::Pair { file, row } => (*file, row.on(side)?),
            DiffRow::File(_) | DiffRow::Hunk { .. } | DiffRow::Note { .. } => return None,
        };
        self.searched().index_of(file, at)
    }

    /// The first searched line of the collapsed file whose header is row `row`.
    fn collapsed_line_at(&self, row: usize) -> Option<usize> {
        match self.rows.get(row)? {
            DiffRow::File(file) if self.collapsed.contains(file) => self.searched().first_of(*file),
            _ => None,
        }
    }

    /// The byte of the searched text at `position` of the rows.
    fn byte_at(&self, position: DiffPosition) -> Option<usize> {
        let index = self.searched_line_at(position.row, self.side)?;
        let text = self.line_text(position.row)?;
        let within = text.char_indices().nth(position.column).map_or(text.len(), |(at, _)| at);
        Some(self.searched().start(index) + within)
    }

    /// Where byte `byte` of the searched text is in the rows, expanding its file when collapsed.
    fn position_of(&mut self, byte: usize) -> Option<DiffPosition> {
        let searched = self.searched();
        let index = searched.line_of(byte);
        let (file, at) = searched.line(index);
        if self.collapsed.remove(&file) {
            self.rebuild();
        }
        let side = self.side;
        let row = self.rows.iter().position(|row| match row {
            DiffRow::Line { file: shown, hunk, line } => *shown == file && DiffLineRef { hunk: *hunk, line: *line } == at,
            DiffRow::Pair { file: shown, row } => *shown == file && row.on(side) == Some(at),
            DiffRow::File(_) | DiffRow::Hunk { .. } | DiffRow::Note { .. } => false,
        })?;
        let text = self.line_text(row)?;
        let within = (byte - searched.start(index)).min(text.len());
        Some(DiffPosition::new(row, text.char_indices().take_while(|(first, _)| *first < within).count()))
    }

    /// Scrolls the least to show `position`'s row, and its column when sideways it is out of view.
    fn reveal_position(&self, position: DiffPosition) {
        let height = f64::from(self.sizes.row_height);
        let top = position.row as f64 * height;
        let rows = self.list.extent(Axis::Vertical);
        if top < rows.scrolled {
            self.list.scroll_to(Axis::Vertical, top);
        } else if top + height > rows.scrolled + rows.viewport && rows.viewport > 0. {
            self.list.scroll_to(Axis::Vertical, top + height - rows.viewport);
        }
        let x = f64::from(f32::from(self.char_width)) * position.column as f64;
        let sideways = self.sideways().extent(Axis::Horizontal);
        let shown = rows.viewport > 0. && x >= sideways.scrolled && x < sideways.scrolled + sideways.viewport * 0.8;
        if !shown && sideways.viewport > 0. {
            self.sideways().scroll_to(Axis::Horizontal, x - sideways.viewport / 2.);
        }
    }
}
