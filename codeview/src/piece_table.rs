//! The text of a file being edited: pieces of the host's [`LineSource`] (never copied) and of a buffer of typed text.
//!
//! Bytes are kept as stored, never decoded: invalid UTF-8, tabs and CRLF round-trip exactly. Lines end at `\n` only,
//! so a `\r` before it is part of the line, and a text ending in `\n` has an empty last line, as an editor shows it.

use std::borrow::Cow;
use std::mem;
use std::ops::Range;
use std::sync::Arc;

use crate::document::TextSource;
use crate::panel::{Edit, LineSource};

/// Editable text over a base revision; see the module doc.
pub struct PieceTable {
    base: Arc<dyn LineSource>,
    added: Added,
    pieces: Vec<Piece>,
    index: PieceIndex,
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

    /// Replaces `bytes` (clamped) of the current text with `text`. Ranges are expected on character boundaries.
    pub fn replace(&mut self, bytes: Range<u64>, text: &str) {
        let end = bytes.end.min(self.byte_len());
        let start = bytes.start.min(end);
        let first = self.split_at(start);
        let after = self.split_at(end);
        let typed = self.type_text(text);
        let typed_count = typed.len();
        self.pieces.splice(first..after, typed);
        self.join_typed(first + typed_count);
        self.reindex();
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

    /// The changes from the base to the current text, in base coordinates: sorted, non-overlapping, and merged where
    /// no base byte is kept between them.
    pub fn edits_since_base(&self) -> Vec<Edit> {
        let mut edits = EditCollector::default();
        for piece in &self.pieces {
            match piece.buffer {
                Buffer::Base => edits.keep(piece.bytes.clone()),
                Buffer::Added => edits.typed.extend_from_slice(&self.buffer_bytes(Buffer::Added, piece.bytes.clone())),
            }
        }
        edits.finish(self.base.byte_len())
    }

    /// Makes `base`, which must hold the current text (as after a save), the new base; the typed text is dropped.
    pub fn rebase(&mut self, base: Arc<dyn LineSource>) {
        assert_eq!(
            base.byte_len(),
            self.byte_len(),
            "rebase onto a base of {} bytes, but the text has {} bytes",
            base.byte_len(),
            self.byte_len()
        );
        *self = Self::new(base);
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
    fn type_text(&mut self, text: &str) -> Vec<Piece> {
        if text.is_empty() {
            return Vec::new();
        }
        let typed = self.added.push(text.as_bytes());
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
        let text = String::from_utf8_lossy(&mem::take(&mut self.typed)).into_owned();
        self.edits.push(Edit { bytes: removed, text });
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
            table.replace(offset..offset, &character.to_string());
        }

        // then
        assert_eq!(table.edits_since_base(), [Edit { bytes: 3..3, text: typed }]);
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
            [Edit { bytes: 2..4, text: String::new() }, Edit { bytes: 10..10, text: "new".to_owned() }]
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

    #[test]
    fn should_hold_the_current_text_after_rebase() {
        // given
        let mut table = PieceTable::from_text("ab\ncd\n");
        table.replace(1..4, "X\nY");
        let saved = String::from_utf8(text(&table)).unwrap();

        // when
        table.rebase(Arc::new(TextSource::new(&saved)));

        // then
        assert_eq!(table.edits_since_base(), []);
        assert_eq!(lines(&table), ["aX", "Yd", ""]);
        table.replace(0..1, "z");
        assert_eq!(table.edits_since_base(), [Edit { bytes: 0..1, text: "z".to_owned() }]);
    }

    #[test]
    fn should_match_a_plain_buffer_after_many_edits() {
        // given
        let mut table = PieceTable::from_text("first\nsecond\n\nfourth line\n");
        let mut expected = text(&table);
        let mut seed: u64 = 7;
        let mut next = |bound: u64| {
            seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
            (seed >> 33) % bound.max(1)
        };

        // when
        for step in 0..500 {
            let start = next(expected.len() as u64 + 1);
            let end = start + next(4).min(expected.len() as u64 - start);
            let inserted = ["", "q", "\n", "ab\ncd", "é"][step % 5];
            table.replace(start..end, inserted);
            expected.splice(start as usize..end as usize, inserted.bytes());
        }

        // then
        assert_eq!(text(&table), expected);
        let expected_lines: Vec<&[u8]> = expected.split(|&byte| byte == b'\n').collect();
        assert_eq!(table.line_count(), expected_lines.len());
        for (index, line) in expected_lines.iter().enumerate() {
            assert_eq!(table.line(index), *line, "line {index}");
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
