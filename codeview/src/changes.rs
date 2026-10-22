//! Change markers: which lines of an edited text differ from its base (the file in HEAD). The line diff runs off the
//! UI thread, one at a time; the markers shown are the last diff that landed, moved by the edits made since, until
//! the next one lands.

use std::ops::Range;

use tree_sitter::InputEdit;

use crate::document::Source;
use crate::piece_table::PieceTable;
use crate::syntax::MAX_HIGHLIGHT_BYTES;

/// What an edited text's markers compare it with.
#[derive(Clone)]
pub enum ChangeBase {
    /// the file in HEAD as a checkout writes it (its filters applied), and HEAD's commit id
    Tracked { head: Source, head_id: String },
    /// nothing to compare with: the file is not in HEAD or not in a repository, or HEAD's text is binary or too
    /// large to diff; no markers
    Untracked,
}

/// One change: lines `base` of the base became lines `current` of the text (line indices, from 0); one of them may
/// be empty.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeHunk {
    pub base: Range<usize>,
    pub current: Range<usize>,
}

/// How a change shows in the gutter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Marker {
    /// a bar on lines the base lacks
    Added,
    /// a bar on lines that replaced others
    Changed,
    /// a triangle at the top of line `current.start`: lines were deleted there
    Deleted,
}

impl ChangeHunk {
    pub fn marker(&self) -> Marker {
        match (self.base.is_empty(), self.current.is_empty()) {
            (true, _) => Marker::Added,
            (false, true) => Marker::Deleted,
            (false, false) => Marker::Changed,
        }
    }
}

/// A line diff of a text with its base, to run off the UI thread.
#[derive(Clone)]
pub struct ChangeDiffJob {
    base: Source,
    text: Source,
    revision: u64,
    epoch: u64,
}

impl ChangeDiffJob {
    /// The revision of the text diffed.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn run(self) -> ChangeHunks {
        ChangeHunks { epoch: self.epoch, hunks: diff_lines(self.base.as_bytes(), self.text.as_bytes()) }
    }
}

/// What a [`ChangeDiffJob`] found, for the text and base it was made of: a panel whose text was replaced or whose
/// base moved since drops it.
pub struct ChangeHunks {
    epoch: u64,
    hunks: Vec<ChangeHunk>,
}

impl ChangeHunks {
    pub fn hunks(&self) -> &[ChangeHunk] {
        &self.hunks
    }
}

/// The changes that turn `base` into `text`, by lines with their line breaks (histogram diff, as `git diff
/// --histogram`), in order.
pub fn diff_lines(base: &[u8], text: &[u8]) -> Vec<ChangeHunk> {
    let input = imara_diff::InternedInput::new(imara_diff::sources::byte_lines(base), imara_diff::sources::byte_lines(text));
    let mut diff = imara_diff::Diff::compute(imara_diff::Algorithm::Histogram, &input);
    diff.postprocess_lines(&input);
    diff.hunks().map(|hunk| ChangeHunk { base: span(hunk.before), current: span(hunk.after) }).collect()
}

fn span(lines: Range<u32>) -> Range<usize> {
    lines.start as usize..lines.end as usize
}

/// The markers of an edited text: at most one diff in flight, and every diff that lands is shown.
pub(crate) struct ChangeMarkers {
    base: Base,
    /// counts the bases and texts replaced: a diff of an earlier epoch is of another text or base
    epoch: u64,
    /// the landed diff, moved by the edits made since, in the current text's lines
    hunks: Vec<ChangeHunk>,
    landed: Landed,
    flight: Flight,
    /// the edits made since the diff in flight's text, in order
    since: Vec<InputEdit>,
}

enum Base {
    /// the host has not said yet
    Unknown,
    Untracked,
    Tracked { head: Source, head_id: String },
}

enum Landed {
    Nothing,
    Diff { revision: u64 },
}

enum Flight {
    Idle,
    Diffing { revision: u64 },
}

/// Whether a new base changed anything.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaseChange {
    Same,
    Moved,
}

impl ChangeMarkers {
    /// No markers until the host gives the base.
    pub(crate) fn new() -> Self {
        Self { base: Base::Unknown, epoch: 0, hunks: Vec::new(), landed: Landed::Nothing, flight: Flight::Idle, since: Vec::new() }
    }

    /// Compares the text with `base` from now on; the same HEAD again changes nothing.
    pub(crate) fn set_base(&mut self, base: ChangeBase) -> BaseChange {
        if let (Base::Tracked { head: known, head_id: known_id }, ChangeBase::Tracked { head, head_id }) = (&self.base, &base)
            && known_id == head_id
            && known.as_bytes() == head.as_bytes()
        {
            return BaseChange::Same;
        }
        self.base = match base {
            ChangeBase::Tracked { head, head_id } => Base::Tracked { head, head_id },
            ChangeBase::Untracked => Base::Untracked,
        };
        self.forget();
        BaseChange::Moved
    }

    /// The text was replaced (a reload): its markers go until a diff of the new one lands; the base stays.
    pub(crate) fn reset(&mut self) {
        self.forget();
    }

    /// `edits` changed the text: the markers move with them.
    pub(crate) fn edited(&mut self, edits: &[InputEdit]) {
        for edit in edits {
            self.hunks = merged(self.hunks.iter().map(|hunk| shifted(hunk, edit)).filter(is_change));
        }
        if let Flight::Diffing { .. } = self.flight {
            self.since.extend_from_slice(edits);
        }
    }

    /// The diff of `table`'s text at `revision` with the base. `None` without a base, while a diff runs, when the
    /// landed diff is of this text, and over [`MAX_HIGHLIGHT_BYTES`], where the markers go.
    pub(crate) fn job(&mut self, revision: u64, table: &PieceTable) -> Option<ChangeDiffJob> {
        let Base::Tracked { head, .. } = &self.base else {
            return None;
        };
        if self.is_diffing() || self.is_current(revision) {
            return None;
        }
        if table.byte_len() > MAX_HIGHLIGHT_BYTES as u64 {
            self.hunks.clear();
            self.landed = Landed::Nothing;
            return None;
        }
        let job = ChangeDiffJob { base: head.clone(), text: Source::from_vec(table.bytes(0..table.byte_len())), revision, epoch: self.epoch };
        self.flight = Flight::Diffing { revision };
        Some(job)
    }

    /// The diff in flight landed: shown from now on, moved by the edits made after its text. One of another text,
    /// base or revision is dropped.
    pub(crate) fn landed(&mut self, revision: u64, diff: ChangeHunks) {
        let Flight::Diffing { revision: running } = self.flight else {
            return;
        };
        if running != revision || diff.epoch != self.epoch {
            return;
        }
        let since = std::mem::take(&mut self.since);
        let moved = diff.hunks.into_iter().map(|hunk| since.iter().fold(hunk, |hunk, edit| shifted(&hunk, edit)));
        self.hunks = merged(moved.filter(is_change));
        self.landed = Landed::Diff { revision };
        self.flight = Flight::Idle;
    }

    /// Whether the markers shown are the diff of the text at `revision`.
    pub(crate) fn is_current(&self, revision: u64) -> bool {
        matches!(self.landed, Landed::Diff { revision: landed } if landed == revision)
    }

    /// The changes, in the current text's lines, in order.
    #[cfg(test)]
    pub(crate) fn hunks(&self) -> &[ChangeHunk] {
        &self.hunks
    }

    /// The changes that show on lines `rows`: bars on them, or a triangle at the top of one or below the last.
    pub(crate) fn shown(&self, rows: Range<usize>) -> impl Iterator<Item = &ChangeHunk> {
        let first = self.hunks.partition_point(|hunk| hunk.current.end < rows.start);
        let candidates = self.hunks[first..].iter().take_while(move |hunk| hunk.current.start <= rows.end);
        candidates.filter(move |hunk| match hunk.marker() {
            Marker::Deleted => rows.contains(&hunk.current.start) || hunk.current.start == rows.end,
            Marker::Added | Marker::Changed => hunk.current.start < rows.end && hunk.current.end > rows.start,
        })
    }

    /// The change whose marker starts on line `line`.
    pub(crate) fn starting_at(&self, line: usize) -> Option<&ChangeHunk> {
        self.hunks.iter().find(|hunk| hunk.current.start == line)
    }

    /// The change that shares lines with `block`, or touches it where either has none: where a change went after an
    /// edit or a new diff.
    pub(crate) fn meeting(&self, block: &Range<usize>) -> Option<&ChangeHunk> {
        self.hunks.iter().find(|hunk| meets(&hunk.current, block))
    }

    /// The change before or after the one starting on line `line`, going round from the last to the first.
    pub(crate) fn beside(&self, line: usize, toward: Toward) -> Option<&ChangeHunk> {
        match toward {
            Toward::Next => self.hunks.iter().find(|hunk| hunk.current.start > line).or(self.hunks.first()),
            Toward::Previous => self.hunks.iter().rev().find(|hunk| hunk.current.start < line).or(self.hunks.last()),
        }
    }

    /// The base's lines `hunk.base`, line breaks included; empty without a base.
    pub(crate) fn base_text(&self, hunk: &ChangeHunk) -> Vec<u8> {
        let Base::Tracked { head, .. } = &self.base else {
            return Vec::new();
        };
        let text = head.as_bytes();
        text[line_offset(text, hunk.base.start)..line_offset(text, hunk.base.end)].to_vec()
    }

    /// Forgets the change starting on line `line`, until the next diff: it was rolled back.
    pub(crate) fn drop_starting_at(&mut self, line: usize) {
        self.hunks.retain(|hunk| hunk.current.start != line);
    }

    fn is_diffing(&self) -> bool {
        matches!(self.flight, Flight::Diffing { .. })
    }

    /// Drops the markers and any diff running: they are of another text or base.
    fn forget(&mut self) {
        self.epoch += 1;
        self.hunks.clear();
        self.landed = Landed::Nothing;
        self.flight = Flight::Idle;
        self.since.clear();
    }
}

/// Which way to go from a change to the next.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Toward {
    Previous,
    Next,
}

/// Whether line ranges `a` and `b` share a line, or touch where one of them is empty.
fn meets(a: &Range<usize>, b: &Range<usize>) -> bool {
    if a.is_empty() || b.is_empty() {
        return a.start <= b.end && b.start <= a.end;
    }
    a.start < b.end && b.start < a.end
}

/// Where line `line` starts in `text`; past the last line, its end.
fn line_offset(text: &[u8], line: usize) -> usize {
    let Some(before) = line.checked_sub(1) else {
        return 0;
    };
    text.iter().enumerate().filter(|(_, byte)| **byte == b'\n').nth(before).map_or(text.len(), |(at, _)| at + 1)
}

/// `hunk` of the text before `edit` in the text after it: lines past the edit move by the lines it added or removed,
/// and an end inside the lines it touched moves to the end of the lines it left there.
pub(crate) fn shifted(hunk: &ChangeHunk, edit: &InputEdit) -> ChangeHunk {
    let lines = EditLines::of(edit);
    let start = lines.start_after(hunk.current.start);
    let end = match hunk.current.end.checked_sub(1) {
        Some(last) if !hunk.current.is_empty() => lines.end_after(last).max(start),
        _ => start,
    };
    ChangeHunk { base: hunk.base.clone(), current: start..end }
}

/// `hunks`, in order, with those an edit brought together made one: a hunk that starts on or before the end of the
/// one before, or on its first line, joins it. What is left starts on distinct lines, in order, and does not
/// overlap, so the end of the hunks grows with the start.
fn merged(hunks: impl Iterator<Item = ChangeHunk>) -> Vec<ChangeHunk> {
    let mut merged: Vec<ChangeHunk> = Vec::new();
    for hunk in hunks {
        match merged.last_mut() {
            Some(last) if hunk.current.start < last.current.end || hunk.current.start == last.current.start => {
                last.base = last.base.start.min(hunk.base.start)..last.base.end.max(hunk.base.end);
                last.current = last.current.start..last.current.end.max(hunk.current.end);
            }
            _ => merged.push(hunk),
        }
    }
    merged
}

/// Whether `hunk` still changes something: an added one whose lines were all deleted does not.
fn is_change(hunk: &ChangeHunk) -> bool {
    !(hunk.base.is_empty() && hunk.current.is_empty())
}

/// The lines an edit touched, before and after it.
struct EditLines {
    /// the first line touched
    first: usize,
    /// the first line before the edit that it left whole, and where that line is after it
    kept_before: usize,
    kept_after: usize,
    /// the end of the lines the touched ones became: past `first` at least when the edit kept the start of it
    touched_end: usize,
}

impl EditLines {
    fn of(edit: &InputEdit) -> Self {
        let first = edit.start_position.row;
        // an edit ending at the start of a line leaves that line whole
        let end = edit.old_end_position;
        let kept_before = if end.column == 0 { end.row } else { end.row + 1 }.max(first);
        let kept_after = kept_before - end.row + edit.new_end_position.row;
        let touched_end = if edit.start_position.column > 0 { kept_after.max(first + 1) } else { kept_after };
        Self { first, kept_before, kept_after, touched_end }
    }

    /// Where a range starting at line `line` starts after the edit.
    fn start_after(&self, line: usize) -> usize {
        if line >= self.kept_before {
            return line - self.kept_before + self.kept_after;
        }
        line.min(self.first)
    }

    /// Where a range ending with line `last` ends (exclusive) after the edit.
    fn end_after(&self, last: usize) -> usize {
        if last >= self.kept_before {
            return last + 1 - self.kept_before + self.kept_after;
        }
        if last < self.first {
            return last + 1;
        }
        self.touched_end
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hunk(base: Range<usize>, current: Range<usize>) -> ChangeHunk {
        ChangeHunk { base, current }
    }

    fn tracked(text: &str, head_id: &str) -> ChangeBase {
        ChangeBase::Tracked { head: Source::from_vec(text.as_bytes().to_vec()), head_id: head_id.to_owned() }
    }

    /// Markers of `table`'s text against `base`, with the diff of revision 0 landed.
    fn landed(base: &str, table: &PieceTable) -> ChangeMarkers {
        let mut markers = ChangeMarkers::new();
        markers.set_base(tracked(base, "head"));
        let job = markers.job(0, table).unwrap();
        markers.landed(0, job.run());
        markers
    }

    fn kinds(markers: &ChangeMarkers) -> Vec<(Range<usize>, Marker)> {
        markers.hunks().iter().map(|hunk| (hunk.current.clone(), hunk.marker())).collect()
    }

    #[test]
    fn should_mark_added_changed_and_deleted_lines() {
        // given
        let base = "one\ntwo\nthree\nfour\nfive\n";
        let table = PieceTable::from_text("zero\none\nTWO\nthree\nfive\n");

        // when
        let markers = landed(base, &table);

        // then
        assert_eq!(kinds(&markers), [(0..1, Marker::Added), (2..3, Marker::Changed), (4..4, Marker::Deleted)]);
        assert_eq!(markers.hunks()[2].base, 3..4);
    }

    #[test]
    fn should_show_no_markers_for_an_unchanged_crlf_file() {
        // given
        let text = "one\r\ntwo\r\nthree\r\n";
        let table = PieceTable::from_text(text);

        // when
        let markers = landed(text, &table);

        // then
        assert!(markers.hunks().is_empty());
        assert_eq!(diff_lines(b"one\r\ntwo\r\n", b"one\ntwo\r\n"), [hunk(0..1, 0..1)]);
    }

    #[test]
    fn should_shift_markers_with_edits_until_the_new_diff_lands() {
        // given
        let base = "one\ntwo\nthree\nfour\n";
        let mut table = PieceTable::from_text("one\nTWO\nthree\nfour\n");
        let mut markers = landed(base, &table);
        let four = table.line_start(3);
        let typed = table.replace(four..four, "x");
        markers.edited(&[typed]);
        let job = markers.job(1, &table).unwrap();

        // when
        let inserted = table.replace(0..0, "new\nnew\n");
        markers.edited(&[inserted]);
        let shifted = kinds(&markers);
        let three = table.line_start(4);
        let typed_above = table.replace(three..three, "!");
        markers.edited(&[typed_above]);
        markers.landed(1, job.run());

        // then
        assert_eq!(shifted, [(3..4, Marker::Changed)]);
        assert_eq!(kinds(&markers), [(3..4, Marker::Changed), (5..6, Marker::Changed)]);
        assert!(!markers.is_current(3));
        let next = markers.job(3, &table).unwrap();
        markers.landed(3, next.run());
        assert_eq!(kinds(&markers), [(0..2, Marker::Added), (3..6, Marker::Changed)]);
    }

    #[test]
    fn should_merge_hunks_an_edit_joined() {
        // given
        let base = "a\nb\nc\nd\ne\n";
        let mut table = PieceTable::from_text("A\nb\nC\nd\ne\n");
        let mut markers = landed(base, &table);
        let before = kinds(&markers);

        // when
        let end = table.line_start(2) + 1;
        let joined = table.replace(1..end, "");
        markers.edited(&[joined]);

        // then
        assert_eq!(before, [(0..1, Marker::Changed), (2..3, Marker::Changed)]);
        assert_eq!(markers.hunks(), [hunk(0..3, 0..1)]);
        assert_eq!(markers.shown(0..5).count(), 1);
    }

    #[test]
    fn should_move_markers_when_only_the_base_bytes_changed() {
        // given
        let mut markers = ChangeMarkers::new();
        markers.set_base(tracked("a\n", "head"));

        // when
        let same = markers.set_base(tracked("a\n", "head"));
        let filtered = markers.set_base(tracked("a\r\n", "head"));

        // then
        assert_eq!((same, filtered), (BaseChange::Same, BaseChange::Moved));
    }

    #[test]
    fn should_keep_a_joined_line_marked_and_unmark_deleted_lines() {
        // given
        let base = "one\ntwo\nthree\nfour\n";
        let mut table = PieceTable::from_text("one\nTWO\nthree\nfour\n");
        let mut markers = landed(base, &table);

        // when
        let line_end = table.line_start(2) - 1;
        let joined = table.replace(line_end..line_end + 1, "");
        markers.edited(&[joined]);
        let after_join = kinds(&markers);
        let (start, end) = (table.line_start(1), table.line_start(2));
        let deleted = table.replace(start..end, "");
        markers.edited(&[deleted]);

        // then
        assert_eq!(after_join, [(1..2, Marker::Changed)]);
        assert_eq!(kinds(&markers), [(1..1, Marker::Deleted)]);
    }

    #[test]
    fn should_move_a_deleted_marker_below_lines_inserted_above_it_and_drop_lines_deleted_inside_a_hunk() {
        // given
        let base = "one\ntwo\nthree\n";
        let mut table = PieceTable::from_text("one\nthree\nA\nB\n");
        let mut markers = landed(base, &table);

        // when
        let inserted = table.replace(0..0, "zero\n");
        markers.edited(&[inserted]);
        let after_insert = kinds(&markers);
        let (a, end) = (table.line_start(3), table.line_start(4));
        let deleted = table.replace(a..end, "");
        markers.edited(&[deleted]);

        // then
        assert_eq!(after_insert, [(2..2, Marker::Deleted), (3..5, Marker::Added)]);
        assert_eq!(kinds(&markers), [(2..2, Marker::Deleted), (3..4, Marker::Added)]);
    }

    #[test]
    fn should_drop_a_diff_of_an_older_revision() {
        // given
        let mut table = PieceTable::from_text("one\n");
        let mut markers = landed("one\n", &table);
        let typed = table.replace(0..0, "x");
        markers.edited(&[typed]);
        let old = markers.job(1, &table).unwrap();
        let typed_again = table.replace(0..0, "y");
        markers.edited(&[typed_again]);

        // when
        markers.reset();
        let new = markers.job(2, &table).unwrap();
        markers.landed(1, old.run());
        let after_old = kinds(&markers);
        markers.landed(1, new.run());
        let wrong_revision = kinds(&markers);

        // then
        assert!(after_old.is_empty() && wrong_revision.is_empty());
        assert!(!markers.is_current(2));
    }

    #[test]
    fn should_drop_a_diff_of_an_earlier_base() {
        // given
        let mut table = PieceTable::from_text("one\n");
        let mut markers = landed("one\n", &table);
        let typed = table.replace(0..0, "x");
        markers.edited(&[typed]);
        let old = markers.job(1, &table).unwrap();

        // when
        let same = markers.set_base(tracked("one\n", "head"));
        let moved = markers.set_base(tracked("xone\n", "next"));
        let new = markers.job(1, &table).unwrap();
        markers.landed(1, old.run());
        let after_old = kinds(&markers);
        markers.landed(1, new.run());

        // then
        assert_eq!((same, moved), (BaseChange::Same, BaseChange::Moved));
        assert_eq!(after_old, []);
        assert_eq!(kinds(&markers), []);
        assert!(markers.is_current(1));
    }

    #[test]
    fn should_show_the_changes_on_the_lines_asked_for() {
        // given
        let table = PieceTable::from_text("A\none\nTWO\nthree\nfive\nsix\nSEVEN\n");
        let markers = landed("one\ntwo\nthree\nfour\nfive\nsix\nseven\n", &table);

        // when
        let shown = [0..1, 1..3, 3..4, 5..7, 7..9].map(|rows| markers.shown(rows).map(|hunk| (hunk.current.start, hunk.current.end)).collect::<Vec<_>>());

        // then
        assert_eq!(shown, [vec![(0, 1)], vec![(2, 3)], vec![(4, 4)], vec![(6, 7)], vec![]]);
    }

    #[test]
    fn should_not_mark_untracked_or_large_files() {
        // given
        let small = PieceTable::from_text("one\n");
        let large = PieceTable::from_text(&"x\n".repeat(MAX_HIGHLIGHT_BYTES / 2 + 1));
        let mut untracked = ChangeMarkers::new();
        untracked.set_base(ChangeBase::Untracked);
        let mut unknown = ChangeMarkers::new();
        let mut grown = landed("two\n", &small);

        // when
        let jobs = (untracked.job(0, &small).is_some(), unknown.job(0, &small).is_some(), grown.job(1, &large).is_some());

        // then
        assert_eq!(jobs, (false, false, false));
        assert!(grown.hunks().is_empty());
    }
}
