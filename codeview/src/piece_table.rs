//! The text of a file being edited: pieces of the host's [`LineSource`] (never copied) and of a buffer of typed text.
//!
//! Bytes are kept as stored, never decoded: invalid UTF-8, tabs and CRLF round-trip exactly. Lines end at `\n` only,
//! so a `\r` before it is part of the line, and a text ending in `\n` has an empty last line, as an editor shows it.

use std::borrow::Cow;
use std::mem;
use std::ops::Range;
use std::sync::Arc;

use tree_sitter::{InputEdit, Point};

use crate::document::TextSource;
use crate::panel::{Edit, LineSource};

/// Editable text over a base revision; see the module doc.
pub struct PieceTable {
    base: Arc<dyn LineSource>,
    added: Added,
    pieces: Vec<Piece>,
    index: PieceIndex,
}

/// A new base that cannot be the old one with the saved edits applied: it has `found` bytes, not `expected`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WrongBase {
    pub expected: u64,
    pub found: u64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Buffer {
    Base,
    Added,
}

/// A run of bytes of one buffer, with the line numbers that let lookups skip scanning it.
#[derive(Clone)]
struct Piece {
    buffer: Buffer,
    bytes: Range<u64>,
    /// newlines in the buffer before `bytes.start`
    first_line: usize,
    newlines: usize,
}

/// Typed text, append-only, so pieces pointing into it stay valid.
#[derive(Default)]
struct Added {
    bytes: Vec<u8>,
    newlines: Vec<u64>,
}

/// Cumulative counts over the pieces, rebuilt per edit: lookups binary-search them instead of walking pieces.
#[derive(Default)]
struct PieceIndex {
    starts: Vec<u64>,
    lines_before: Vec<usize>,
    byte_len: u64,
    newlines: usize,
}

impl PieceTable {
    pub fn new(base: Arc<dyn LineSource>) -> Self {
        let mut table = Self { base, added: Added::default(), pieces: Vec::new(), index: PieceIndex::default() };
        let whole = 0..table.base.byte_len();
        if whole.is_empty() {
            return table;
        }
        table.pieces = vec![table.piece(Buffer::Base, whole)];
        table.reindex();
        table
    }

    /// A table over `text` held in memory: a scratch's text.
    pub fn from_text(text: &str) -> Self {
        Self::new(Arc::new(TextSource::new(text)))
    }

    pub fn byte_len(&self) -> u64 {
        self.index.byte_len
    }

    /// The text as opened or last saved, which [`Self::edits_since_base`] counts from.
    pub fn base(&self) -> &Arc<dyn LineSource> {
        &self.base
    }

    /// Never 0: an empty text has one empty line.
    pub fn line_count(&self) -> usize {
        self.index.newlines + 1
    }

    /// Line `index` (clamped) as stored, without its `\n`.
    pub fn line(&self, index: usize) -> Vec<u8> {
        let index = index.min(self.index.newlines);
        let start = self.line_start(index);
        if index == self.index.newlines {
            return self.bytes(start..self.byte_len());
        }
        self.bytes(start..self.line_start(index + 1) - 1)
    }

    /// Byte offset where line `index` (clamped) starts in the current text.
    pub fn line_start(&self, index: usize) -> u64 {
        let index = index.min(self.index.newlines);
        if index == 0 {
            return 0;
        }
        // the piece holding newline number `index - 1`, which ends the line before
        let at = self.index.lines_before.partition_point(|&before| before < index) - 1;
        let piece = &self.pieces[at];
        let within = index - self.index.lines_before[at];
        let buffer_start = self.buffer_line_start(piece.buffer, piece.first_line + within);
        self.index.starts[at] + (buffer_start - piece.bytes.start)
    }

    /// Replaces `bytes` (clamped) of the current text with `text`, stored as given. Ranges are expected on character
    /// boundaries. Returns the replacement as tree-sitter describes an edit, for re-parsing incrementally.
    pub fn replace(&mut self, bytes: Range<u64>, text: impl AsRef<[u8]>) -> InputEdit {
        let end = bytes.end.min(self.byte_len());
        let start = bytes.start.min(end);
        let (start_position, old_end_position) = (self.point_of(start), self.point_of(end));
        let first = self.split_at(start);
        let after = self.split_at(end);
        let typed = self.type_text(text.as_ref());
        let typed_count = typed.len();
        self.pieces.splice(first..after, typed);
        self.join_typed(first + typed_count);
        self.reindex();
        let new_end = start + text.as_ref().len() as u64;
        InputEdit {
            start_byte: start as usize,
            old_end_byte: end as usize,
            new_end_byte: new_end as usize,
            start_position,
            old_end_position,
            new_end_position: self.point_of(new_end),
        }
    }

    /// Where byte `offset` (at most the length) is as tree-sitter counts: line, and bytes into it.
    fn point_of(&self, offset: u64) -> Point {
        let row = self.line_of(offset);
        Point { row, column: (offset - self.line_start(row)) as usize }
    }

    /// The line byte `offset` (at most the length) is on: the newlines before it.
    fn line_of(&self, offset: u64) -> usize {
        if offset >= self.byte_len() {
            return self.index.newlines;
        }
        let at = self.piece_at(offset);
        let piece = &self.pieces[at];
        let within = self.buffer_line_of(piece.buffer, piece.bytes.start + (offset - self.index.starts[at]));
        self.index.lines_before[at] + within - piece.first_line
    }

    /// Bytes `range` (clamped) of the current text.
    pub fn bytes(&self, range: Range<u64>) -> Vec<u8> {
        let end = range.end.min(self.byte_len());
        let start = range.start.min(end);
        let mut out = Vec::with_capacity(usize::try_from(end - start).unwrap_or(0));
        if start == end {
            return out;
        }
        let first = self.piece_at(start);
        for (piece, &piece_start) in self.pieces[first..].iter().zip(&self.index.starts[first..]) {
            if piece_start >= end {
                break;
            }
            let from = start.max(piece_start) - piece_start + piece.bytes.start;
            let to = end.min(piece_start + piece.len()) - piece_start + piece.bytes.start;
            out.extend_from_slice(&self.buffer_bytes(piece.buffer, from..to));
        }
        out
    }

    /// The changes from the base to the current text, in base coordinates: sorted, non-overlapping, merged where
    /// no base byte is kept between them, and without changes that put back the bytes they replace.
    pub fn edits_since_base(&self) -> Vec<Edit> {
        let mut edits = EditCollector::default();
        for piece in &self.pieces {
            match piece.buffer {
                Buffer::Base => edits.keep(piece.bytes.clone()),
                Buffer::Added => edits.typed.extend_from_slice(&self.buffer_bytes(Buffer::Added, piece.bytes.clone())),
            }
        }
        let mut edits = edits.finish(self.base.byte_len());
        edits.retain(|edit| self.changes_base(edit));
        edits
    }

    /// Makes `base` the new base once `saved` is written: `base` must be the old base with `saved` (an earlier
    /// [`Self::edits_since_base`]) applied. The text stays as it is, edits made since `saved` was taken included;
    /// afterwards [`Self::edits_since_base`] reports only those. A `base` of another length changes nothing.
    pub fn rebase(&mut self, base: Arc<dyn LineSource>, saved: &[Edit]) -> Result<(), WrongBase> {
        let growth: i64 = saved.iter().map(Edit::growth).sum();
        let expected = (self.base.byte_len() as i64 + growth) as u64;
        if base.byte_len() != expected {
            return Err(WrongBase { expected, found: base.byte_len() });
        }
        let mut shift = Shift::new(saved);
        let moved: Vec<(Buffer, Range<u64>)> = self
            .pieces
            .iter()
            .map(|piece| match piece.buffer {
                Buffer::Base => (Buffer::Base, shift.map(piece.bytes.clone())),
                Buffer::Added => (Buffer::Added, piece.bytes.clone()),
            })
            .collect();
        self.base = base;
        let folded = self.fold_saved_text(moved);
        self.pieces = folded.into_iter().map(|(buffer, bytes)| self.piece(buffer, bytes)).collect();
        self.reindex();
        Ok(())
    }

    /// Whether `edit` replaces base bytes with different ones.
    fn changes_base(&self, edit: &Edit) -> bool {
        edit.growth() != 0 || *self.base.bytes(edit.bytes.clone()) != edit.text[..]
    }

    /// Runs of typed pieces that hold exactly the base bytes between their neighbouring base pieces (saved text)
    /// become base pieces, and neighbouring base pieces join.
    fn fold_saved_text(&self, pieces: Vec<(Buffer, Range<u64>)>) -> Vec<(Buffer, Range<u64>)> {
        let mut folded = Vec::with_capacity(pieces.len());
        let mut typed = Vec::new();
        let mut base_end = 0;
        let end = self.base.byte_len();
        // an empty base piece at the end closes the last run
        for (buffer, bytes) in pieces.into_iter().chain([(Buffer::Base, end..end)]) {
            if buffer == Buffer::Added {
                typed.push(bytes);
                continue;
            }
            self.fold_run(&mut folded, mem::take(&mut typed), base_end..bytes.start);
            base_end = bytes.end;
            push_joined(&mut folded, Buffer::Base, bytes);
        }
        folded.retain(|(_, bytes)| !bytes.is_empty());
        folded
    }

    /// Appends the typed run `typed` found where base bytes `gap` are: as much of its start and its end as holds
    /// the same bytes as the gap there becomes those base bytes.
    fn fold_run(&self, folded: &mut Vec<(Buffer, Range<u64>)>, typed: Vec<Range<u64>>, gap: Range<u64>) {
        let text: Vec<u8> = typed.iter().flat_map(|bytes| self.buffer_bytes(Buffer::Added, bytes.clone()).into_owned()).collect();
        let len = text.len() as u64;
        let head = self.base.bytes(gap.start..gap.start + len.min(span(&gap)));
        let prefix = text.iter().zip(head.iter()).take_while(|(typed, base)| typed == base).count() as u64;
        let rest = gap.start + prefix..gap.end;
        let tail = self.base.bytes(rest.end - (len - prefix).min(span(&rest))..rest.end);
        let suffix = text[prefix as usize..].iter().rev().zip(tail.iter().rev()).take_while(|(typed, base)| typed == base).count() as u64;
        push_joined(folded, Buffer::Base, gap.start..gap.start + prefix);
        folded.extend(sub_run(&typed, prefix..len - suffix).into_iter().map(|bytes| (Buffer::Added, bytes)));
        push_joined(folded, Buffer::Base, gap.end - suffix..gap.end);
    }

    fn piece(&self, buffer: Buffer, bytes: Range<u64>) -> Piece {
        let first_line = self.buffer_line_of(buffer, bytes.start);
        let newlines = self.buffer_line_of(buffer, bytes.end) - first_line;
        Piece { buffer, bytes, first_line, newlines }
    }

    /// Index of the piece holding `offset`, which must be inside the text.
    fn piece_at(&self, offset: u64) -> usize {
        self.index.starts.partition_point(|&start| start <= offset) - 1
    }

    /// Splits the piece holding `offset` there; returns the index of the piece starting at `offset`.
    fn split_at(&mut self, offset: u64) -> usize {
        if offset >= self.byte_len() {
            return self.pieces.len();
        }
        let at = self.piece_at(offset);
        let piece_start = self.index.starts[at];
        if piece_start == offset {
            return at;
        }
        let Piece { buffer, bytes, .. } = self.pieces[at].clone();
        let middle = bytes.start + (offset - piece_start);
        let halves = [self.piece(buffer, bytes.start..middle), self.piece(buffer, middle..bytes.end)];
        self.pieces.splice(at..=at, halves);
        self.reindex();
        at + 1
    }

    /// Appends `text` to the typed buffer; the piece showing it, if any.
    fn type_text(&mut self, text: &[u8]) -> Vec<Piece> {
        if text.is_empty() {
            return Vec::new();
        }
        let typed = self.added.push(text);
        vec![self.piece(Buffer::Added, typed)]
    }

    /// Joins the two pieces before index `at` when both are contiguous typed text, so typing a line keeps one piece.
    fn join_typed(&mut self, at: usize) {
        let [.., before, last] = &self.pieces[..at] else { return };
        let contiguous =
            before.buffer == Buffer::Added && last.buffer == Buffer::Added && before.bytes.end == last.bytes.start;
        if contiguous {
            let joined = self.piece(Buffer::Added, before.bytes.start..last.bytes.end);
            self.pieces.splice(at - 2..at, [joined]);
        }
    }

    fn reindex(&mut self) {
        self.index = PieceIndex::of(&self.pieces);
    }

    fn buffer_bytes(&self, buffer: Buffer, range: Range<u64>) -> Cow<'_, [u8]> {
        match buffer {
            Buffer::Base => self.base.bytes(range),
            Buffer::Added => Cow::Borrowed(&self.added.bytes[range.start as usize..range.end as usize]),
        }
    }

    /// Newlines in `buffer` before `byte`.
    fn buffer_line_of(&self, buffer: Buffer, byte: u64) -> usize {
        match buffer {
            Buffer::Base => self.base.line_of(byte),
            Buffer::Added => self.added.newlines.partition_point(|&at| at < byte),
        }
    }

    /// Where line `line` (≥ 1, ending a newline in `buffer`) starts in `buffer`.
    fn buffer_line_start(&self, buffer: Buffer, line: usize) -> u64 {
        match buffer {
            Buffer::Base => self.base.line_start(line),
            Buffer::Added => self.added.newlines[line - 1] + 1,
        }
    }
}

impl Piece {
    fn len(&self) -> u64 {
        self.bytes.end - self.bytes.start
    }
}

impl Added {
    /// Appends `bytes`; returns where they landed.
    fn push(&mut self, bytes: &[u8]) -> Range<u64> {
        let start = self.bytes.len() as u64;
        self.newlines.extend(memchr::memchr_iter(b'\n', bytes).map(|at| start + at as u64));
        self.bytes.extend_from_slice(bytes);
        start..self.bytes.len() as u64
    }
}

impl PieceIndex {
    fn of(pieces: &[Piece]) -> Self {
        let mut index = Self {
            starts: Vec::with_capacity(pieces.len()),
            lines_before: Vec::with_capacity(pieces.len()),
            ..Self::default()
        };
        for piece in pieces {
            index.starts.push(index.byte_len);
            index.lines_before.push(index.newlines);
            index.byte_len += piece.len();
            index.newlines += piece.newlines;
        }
        index
    }
}

fn span(bytes: &Range<u64>) -> u64 {
    bytes.end - bytes.start
}

/// Bytes `within` of the run `pieces` (ranges of one buffer read one after another), as ranges of that buffer.
fn sub_run(pieces: &[Range<u64>], within: Range<u64>) -> Vec<Range<u64>> {
    let mut ranges = Vec::new();
    let mut at = 0;
    for piece in pieces {
        let (from, to) = (within.start.max(at), within.end.min(at + span(piece)));
        if from < to {
            ranges.push(piece.start + (from - at)..piece.start + (to - at));
        }
        at += span(piece);
    }
    ranges
}

/// Appends `bytes` of `buffer`, joined with the last piece when both are of the base and touch.
fn push_joined(pieces: &mut Vec<(Buffer, Range<u64>)>, buffer: Buffer, bytes: Range<u64>) {
    if let Some((Buffer::Base, last)) = pieces.last_mut()
        && buffer == Buffer::Base
        && last.end == bytes.start
    {
        last.end = bytes.end;
        return;
    }
    pieces.push((buffer, bytes));
}

/// Maps ranges of an old base, in order, to where saved edits moved them in the new base.
struct Shift<'a> {
    edits: &'a [Edit],
    /// the edits before this one end at or before the ranges still to map
    next: usize,
    growth: i64,
}

impl<'a> Shift<'a> {
    fn new(edits: &'a [Edit]) -> Self {
        Self { edits, next: 0, growth: 0 }
    }

    /// `old`, at or after the last range mapped, in the new base. Base pieces never hold bytes a saved edit
    /// replaced: typed text always goes into the typed buffer, so base bytes once replaced never come back.
    fn map(&mut self, old: Range<u64>) -> Range<u64> {
        while let Some(edit) = self.edits.get(self.next).filter(|edit| edit.bytes.end <= old.start) {
            self.growth += edit.growth();
            self.next += 1;
        }
        if let Some(edit) = self.edits.get(self.next) {
            let overlaps = edit.bytes.start < old.end && edit.bytes.end > old.start;
            assert!(!overlaps, "base bytes {old:?} overlap the saved edit of bytes {:?}", edit.bytes);
        }
        let start = (old.start as i64 + self.growth) as u64;
        start..start + (old.end - old.start)
    }
}

/// Walks the pieces in order: base pieces are kept base bytes, everything between two of them is one edit.
#[derive(Default)]
struct EditCollector {
    edits: Vec<Edit>,
    /// base bytes before this are accounted for
    kept_to: u64,
    typed: Vec<u8>,
}

impl EditCollector {
    fn keep(&mut self, base: Range<u64>) {
        self.close(base.start);
        self.kept_to = base.end;
    }

    /// Ends the edit that removes base bytes up to `until` and inserts the text typed since the last kept bytes.
    fn close(&mut self, until: u64) {
        let removed = self.kept_to..until;
        if removed.is_empty() && self.typed.is_empty() {
            return;
        }
        self.edits.push(Edit { bytes: removed, text: mem::take(&mut self.typed) });
    }

    fn finish(mut self, base_len: u64) -> Vec<Edit> {
        self.close(base_len);
        self.edits
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    fn table_of(bytes: &[u8]) -> PieceTable {
        PieceTable::new(Arc::new(TextSource::from_bytes(bytes)))
    }

    fn lines(table: &PieceTable) -> Vec<String> {
        (0..table.line_count()).map(|index| String::from_utf8(table.line(index)).unwrap()).collect()
    }

    fn text(table: &PieceTable) -> Vec<u8> {
        table.bytes(0..table.byte_len())
    }

    #[test]
    fn should_read_lines_of_an_untouched_base() {
        // given
        let table = PieceTable::from_text("one\ntwo\r\n\nfour");

        // when
        let read = lines(&table);

        // then
        assert_eq!(read, ["one", "two\r", "", "four"]);
        assert_eq!(table.line_start(1), 4);
        assert_eq!(table.line_start(3), 10);
        assert_eq!(table.byte_len(), 14);
    }

    #[test]
    fn should_insert_and_delete_across_lines() {
        // given
        let mut table = PieceTable::from_text("ab\ncd\n");

        // when
        table.replace(1..4, "X");

        // then
        assert_eq!(text(&table), b"aXd\n");
        assert_eq!(lines(&table), ["aXd", ""]);
        assert_eq!(table.line_start(1), 4);
    }

    #[test]
    fn should_merge_neighbouring_edits_into_one() {
        // given
        let mut table = PieceTable::from_text("abcdef\n");
        let typed: String = (0..100).map(|index| char::from(b'a' + (index % 26) as u8)).collect();

        // when
        for (offset, character) in (3..).zip(typed.chars()) {
            table.replace(offset..offset, character.to_string());
        }

        // then
        assert_eq!(table.edits_since_base(), [Edit { bytes: 3..3, text: typed.into_bytes() }]);
    }

    #[test]
    fn should_express_edits_in_base_coordinates() {
        // given
        let mut table = PieceTable::from_text("0123456789abcdef\n");

        // when
        table.replace(10..10, "new");
        table.replace(2..4, "");

        // then
        assert_eq!(
            table.edits_since_base(),
            [Edit { bytes: 2..4, text: Vec::new() }, Edit { bytes: 10..10, text: b"new".to_vec() }]
        );
    }

    #[test]
    fn should_express_a_deletion_of_typed_text_as_no_edit() {
        // given
        let mut table = PieceTable::from_text("abc\n");

        // when
        table.replace(1..1, "xyz");
        table.replace(1..4, "");

        // then
        assert_eq!(table.edits_since_base(), []);
        assert_eq!(text(&table), b"abc\n");
    }

    #[test]
    fn should_describe_a_replacement_in_bytes_and_points() {
        // given
        let mut table = PieceTable::from_text("ab\ncd\nef");
        table.replace(4..4, "X\nY");

        // when
        let edit = table.replace(1..7, "12\n345");

        // then
        assert_eq!(text(&table), b"a12\n345d\nef");
        let point = |row, column| Point { row, column };
        assert_eq!(
            edit,
            InputEdit {
                start_byte: 1,
                old_end_byte: 7,
                new_end_byte: 7,
                start_position: point(0, 1),
                old_end_position: point(2, 1),
                new_end_position: point(1, 3),
            }
        );
    }

    #[test]
    fn should_keep_invalid_utf8_bytes_untouched() {
        // given
        let mut table = table_of(b"a\xff\nb\n");

        // when
        table.replace(5..5, "c");

        // then
        assert_eq!(table.bytes(0..3), b"a\xff\n");
        assert_eq!(table.line(0), b"a\xff");
        assert_eq!(table.line(2), b"c");
    }

    #[test]
    fn should_keep_crlf_and_tabs() {
        // given
        let mut table = PieceTable::from_text("a\tb\r\nc\r\n");

        // when
        table.replace(0..0, "x");

        // then
        assert_eq!(text(&table), b"xa\tb\r\nc\r\n");
        assert_eq!(table.line(1), b"c\r");
    }

    /// `base` with `edits` applied; checks they are sorted and do not overlap.
    fn applied(base: &[u8], edits: &[Edit]) -> Vec<u8> {
        let mut out = Vec::new();
        let mut kept_from = 0;
        for edit in edits {
            assert!(edit.bytes.start >= kept_from && edit.bytes.start <= edit.bytes.end, "edits out of order: {edits:?}");
            out.extend_from_slice(&base[kept_from as usize..edit.bytes.start as usize]);
            out.extend_from_slice(&edit.text);
            kept_from = edit.bytes.end;
        }
        out.extend_from_slice(&base[kept_from as usize..]);
        out
    }

    /// A pseudo-random replacement in a text of `len` bytes, from `next`.
    fn random_replacement(next: &mut impl FnMut(u64) -> u64, len: u64, step: usize) -> (Range<u64>, &'static str) {
        let start = next(len + 1);
        let end = start + next(4).min(len - start);
        (start..end, ["", "q", "\n", "ab\ncd", "é"][step % 5])
    }

    fn generator(mut seed: u64) -> impl FnMut(u64) -> u64 {
        move |bound: u64| {
            seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
            (seed >> 33) % bound.max(1)
        }
    }

    #[test]
    fn should_refuse_a_new_base_of_the_wrong_length() {
        // given
        let mut table = PieceTable::from_text("ab\n");
        table.replace(0..0, "x");
        let saved = table.edits_since_base();

        // when
        let rebased = table.rebase(Arc::new(TextSource::new("another program's text\n")), &saved);

        // then
        assert_eq!(rebased, Err(WrongBase { expected: 4, found: 23 }));
        assert_eq!(table.edits_since_base(), saved);
        assert_eq!(lines(&table), ["xab", ""]);
    }

    #[test]
    fn should_hold_the_current_text_after_rebase() {
        // given
        let mut table = PieceTable::from_text("ab\ncd\n");
        table.replace(1..4, "X\nY");
        let saved = table.edits_since_base();

        // when
        table.rebase(Arc::new(TextSource::new("aX\nYd\n")), &saved).unwrap();

        // then
        assert_eq!(table.edits_since_base(), []);
        assert_eq!(lines(&table), ["aX", "Yd", ""]);
        table.replace(0..1, "z");
        assert_eq!(table.edits_since_base(), [Edit { bytes: 0..1, text: b"z".to_vec() }]);
    }

    #[test]
    fn should_keep_edits_typed_while_a_save_was_written() {
        // given
        let mut table = PieceTable::from_text("0123456789\n");
        table.replace(2..4, "ab");
        table.replace(8..8, "XYZ");
        let saved = table.edits_since_base();
        table.replace(0..1, "-");
        table.replace(9..10, "");

        // when
        table.rebase(Arc::new(TextSource::new("01ab4567XYZ89\n")), &saved).unwrap();

        // then
        assert_eq!(text(&table), b"-1ab4567XZ89\n");
        assert_eq!(table.edits_since_base(), [Edit { bytes: 0..1, text: b"-".to_vec() }, Edit { bytes: 9..10, text: Vec::new() }]);
    }

    #[test]
    fn should_leave_out_edits_that_put_back_what_they_replace() {
        // given
        let mut table = PieceTable::from_text("abc\n");

        // when
        table.replace(1..2, "x");
        table.replace(1..2, "b");

        // then
        assert_eq!(table.edits_since_base(), []);
    }

    #[test]
    fn should_match_a_plain_buffer_after_many_edits() {
        // given
        let original = b"first\nsecond\n\nfourth line\n".to_vec();
        let mut table = table_of(&original);
        let mut expected = original.clone();
        let mut next = generator(7);

        // when
        for step in 0..500 {
            let (bytes, inserted) = random_replacement(&mut next, expected.len() as u64, step);
            table.replace(bytes.clone(), inserted);
            expected.splice(bytes.start as usize..bytes.end as usize, inserted.bytes());
        }

        // then
        assert_eq!(text(&table), expected);
        assert_eq!(applied(&original, &table.edits_since_base()), expected);
        let expected_lines: Vec<&[u8]> = expected.split(|&byte| byte == b'\n').collect();
        assert_eq!(table.line_count(), expected_lines.len());
        for (index, line) in expected_lines.iter().enumerate() {
            assert_eq!(table.line(index), *line, "line {index}");
        }
    }

    #[test]
    fn should_keep_the_text_across_saves_with_edits_in_flight() {
        // given
        let mut base = b"first\nsecond\n\nfourth line\n".to_vec();
        let mut table = table_of(&base);
        let mut next = generator(11);
        let mut step = 0;

        // when
        for _save in 0..60 {
            for _ in 0..next(6) {
                let (bytes, inserted) = random_replacement(&mut next, table.byte_len(), step);
                table.replace(bytes, inserted);
                step += 1;
            }
            let saved = table.edits_since_base();
            let written = applied(&base, &saved);
            for _ in 0..next(4) {
                let (bytes, inserted) = random_replacement(&mut next, table.byte_len(), step);
                table.replace(bytes, inserted);
                step += 1;
            }
            let before = text(&table);
            table.rebase(Arc::new(TextSource::from_bytes(&written)), &saved).unwrap();
            base = written;

            // then
            assert_eq!(text(&table), before);
            assert_eq!(applied(&base, &table.edits_since_base()), before);
            assert_eq!(table.line_count(), before.split(|&byte| byte == b'\n').count());
        }
    }

    #[test]
    fn should_find_lines_in_a_large_base_quickly() {
        // given
        let line_count = 10_000_000;
        let mut table = PieceTable::from_text(&"some text\n".repeat(line_count));

        // when
        let started = Instant::now();
        for edit in 0..1_000 {
            let at = table.line_start(edit * 9_973 + 5);
            table.replace(at..at + 4, "edit\nmore");
        }
        let lookups: Vec<Vec<u8>> = (0..10_000).map(|lookup| table.line(lookup * 997 + 1)).collect();
        let elapsed = started.elapsed();

        // then
        assert_eq!(table.line_count(), line_count + 1_001);
        assert!(lookups.iter().all(|line| [&b"some text"[..], b"edit", b"more text"].contains(&line.as_slice())));
        assert!(elapsed < Duration::from_millis(200), "1 000 edits and 10 000 lookups took {elapsed:?}");
    }
}
