//! The editor's colours: the last parse that landed, shifted by the edits made since, until the next one lands.
//! One parse runs at a time; edits made meanwhile are recorded, and when it lands the newest text is parsed next.

use std::ops::Range;
use std::rc::Rc;

use tree_sitter::InputEdit;

use crate::code_view::HighlightCache;
use crate::document::Source;
use crate::panel::{Failed, ParseJob, ParseTarget, Previous};
use crate::piece_table::PieceTable;
use crate::syntax::{self, Cancel, Highlight, Language, MAX_HIGHLIGHT_BYTES, Syntax};

/// Highlighting of an edited text: at most one parse in flight, and every parse that lands is shown.
pub struct Colours {
    highlighting: Highlighting,
    landed: Landed,
    /// the edits after the landed parse's text (before any landed, after the text in flight), in order
    since: Vec<Recorded>,
    flight: Flight,
    /// highlights of the lines shown, for the current text and landed parse
    cache: HighlightCache,
}

enum Highlighting {
    Plain,
    In(Language),
}

enum Landed {
    Nothing,
    Parse { syntax: Syntax, source: Source, revision: u64 },
}

/// An edit and the revision it made.
struct Recorded {
    revision: u64,
    edit: InputEdit,
}

/// The parse running, of the text at `revision`.
enum Flight {
    Idle,
    Parsing { revision: u64, source: Source, cancel: Cancel, failed: Failed },
}

/// Which end of a range an offset is.
#[derive(Clone, Copy)]
enum End {
    Start,
    End,
}

impl Colours {
    /// Colours in `language`; plain text without one.
    pub fn new(language: Option<Language>) -> Self {
        let highlighting = language.map_or(Highlighting::Plain, Highlighting::In);
        let (landed, flight) = (Landed::Nothing, Flight::Idle);
        Self { highlighting, landed, since: Vec::new(), flight, cache: HighlightCache::default() }
    }

    /// `edits` made the text of `revision`; the colours shown move with them.
    pub fn edited(&mut self, revision: u64, edits: Vec<InputEdit>) {
        self.cache = HighlightCache::default();
        if let (Landed::Nothing, Flight::Idle) = (&self.landed, &self.flight) {
            return;
        }
        self.since.extend(edits.into_iter().map(|edit| Recorded { revision, edit }));
    }

    /// The parse of `table`'s text at `revision`, reusing the landed one. `None` while a parse runs (the newest
    /// text is parsed once it lands), when the landed parse is of this text, without a language, and over
    /// [`MAX_HIGHLIGHT_BYTES`], where the text shows plain.
    pub fn job(&mut self, revision: u64, table: &PieceTable) -> Option<ParseJob> {
        let Highlighting::In(language) = self.highlighting else {
            return None;
        };
        if self.is_parsing() || self.has_landed(revision) {
            return None;
        }
        if table.byte_len() > MAX_HIGHLIGHT_BYTES as u64 {
            self.forget();
            return None;
        }
        // the one copy of the whole text, under 1 ms at the 8 MiB limit; the highlights read it too
        let source = Source::from_vec(table.bytes(0..table.byte_len()));
        let previous = self.previous();
        let (cancel, failed) = (Cancel::default(), Failed::default());
        self.flight = Flight::Parsing { revision, source: source.clone(), cancel: cancel.clone(), failed: failed.clone() };
        Some(ParseJob { source, language, cancel, target: ParseTarget::Text, revision, previous, failed })
    }

    /// The parse in flight landed: it is shown from now on, moved by the edits made after its text. A parse of
    /// another revision is dropped.
    pub fn parsed(&mut self, revision: u64, syntax: Syntax) {
        let Flight::Parsing { revision: running, source, .. } = &self.flight else {
            return;
        };
        if *running != revision {
            return;
        }
        self.landed = Landed::Parse { syntax, source: source.clone(), revision };
        self.flight = Flight::Idle;
        let done = self.since.partition_point(|recorded| recorded.revision <= revision);
        self.since.drain(..done);
        self.cache = HighlightCache::default();
    }

    /// Lines not asked for by [`Colours::line_highlights`] until the next frame starts are forgotten then.
    pub fn start_frame(&self) {
        self.cache.start_frame();
    }

    /// [`Colours::highlights`] of the line taking `bytes`, kept until the text changes or a parse lands.
    pub fn line_highlights(&self, bytes: Range<u64>) -> Rc<[Highlight]> {
        self.cache.get_or_query(bytes.start as usize, || self.highlights(bytes))
    }

    /// Highlights of `bytes` of the current text: the landed parse's, moved by the edits since; text an edit put in
    /// stays plain until the next parse lands.
    pub fn highlights(&self, bytes: Range<u64>) -> Vec<Highlight> {
        let Landed::Parse { syntax, source, .. } = &self.landed else {
            return Vec::new();
        };
        let current = bytes.start as usize..bytes.end as usize;
        let len = source.as_bytes().len();
        let old = self.since.iter().rev().fold(current.clone(), |range, Recorded { edit, .. }| {
            before(range.start, edit, End::Start)..before(range.end, edit, End::End)
        });
        let mut highlights = syntax::spans(syntax, source.as_bytes(), old.start.min(len)..old.end.min(len));
        for Recorded { edit, .. } in &self.since {
            highlights = highlights.into_iter().flat_map(|highlight| after(highlight, edit)).collect();
        }
        highlights.into_iter().filter_map(|highlight| clipped(highlight, &current)).collect()
    }

    /// A parse runs whose answer is still to come; one that failed sends none.
    fn is_parsing(&self) -> bool {
        match &self.flight {
            Flight::Parsing { failed, .. } if failed.is_set() => false,
            Flight::Parsing { .. } => true,
            Flight::Idle => false,
        }
    }

    fn has_landed(&self, revision: u64) -> bool {
        matches!(self.landed, Landed::Parse { revision: landed, .. } if landed == revision)
    }

    /// What the next parse reuses: the landed one and the edits since; with nothing landed, nothing, and the
    /// recorded edits are of no use then.
    fn previous(&mut self) -> Previous {
        let Landed::Parse { syntax, .. } = &self.landed else {
            self.since.clear();
            return Previous::Nothing;
        };
        let edits = self.since.iter().map(|recorded| recorded.edit).collect();
        Previous::Edited { syntax: syntax.clone(), edits }
    }

    /// The text grew too large to colour: it shows plain until it shrinks; a parse running is not waited for.
    fn forget(&mut self) {
        self.landed = Landed::Nothing;
        self.flight = Flight::Idle;
        self.since.clear();
        self.cache = HighlightCache::default();
    }
}

impl Drop for Colours {
    fn drop(&mut self) {
        if let Flight::Parsing { cancel, .. } = &self.flight {
            cancel.cancel();
        }
    }
}

/// Where `offset` of the text after `edit` was before it; inside the inserted text, where the replaced text started
/// or ended, as `end` says.
fn before(offset: usize, edit: &InputEdit, end: End) -> usize {
    if offset <= edit.start_byte {
        return offset;
    }
    if offset >= edit.new_end_byte {
        return offset - edit.new_end_byte + edit.old_end_byte;
    }
    match end {
        End::Start => edit.start_byte,
        End::End => edit.old_end_byte,
    }
}

/// `highlight` of the text before `edit` in the text after it: the parts outside what the edit replaced.
fn after(highlight: Highlight, edit: &InputEdit) -> Vec<Highlight> {
    let Highlight { bytes, key } = highlight;
    let moved = |offset: usize| offset - edit.old_end_byte + edit.new_end_byte;
    let kept_before = bytes.start..bytes.end.min(edit.start_byte);
    let kept_after = moved(bytes.start.max(edit.old_end_byte))..moved(bytes.end.max(edit.old_end_byte));
    [kept_before, kept_after]
        .into_iter()
        .filter(|part| part.start < part.end)
        .map(|bytes| Highlight { bytes, key })
        .collect()
}

fn clipped(highlight: Highlight, range: &Range<usize>) -> Option<Highlight> {
    let bytes = highlight.bytes.start.max(range.start)..highlight.bytes.end.min(range.end);
    (bytes.start < bytes.end).then_some(Highlight { bytes, key: highlight.key })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::{PARSE_BUDGET, lang, parse, reparse, spans};
    use crate::text_editor::model::{EditKind, EditorModel, Position, Selecting};

    fn table(text: &str) -> PieceTable {
        PieceTable::from_text(text)
    }

    fn all(table: &PieceTable) -> Range<u64> {
        0..table.byte_len()
    }

    fn fresh(table: &PieceTable) -> Vec<Highlight> {
        let text = table.bytes(all(table));
        let syntax = parse(&text, lang("Rust"), &Cancel::default(), PARSE_BUDGET).unwrap();
        spans(&syntax, &text, 0..text.len())
    }

    /// Colours with the parse of `table`'s text, as revision 0, landed.
    fn landed(table: &PieceTable) -> Colours {
        let mut colours = Colours::new(Some(lang("Rust")));
        let job = colours.job(0, table).unwrap();
        colours.parsed(0, job.run().unwrap());
        colours
    }

    #[test]
    fn should_shift_highlights_after_an_insert_until_the_parse_lands() {
        // given
        let mut table = table("fn main() {\n    let n = 1;\n}\n");
        let mut colours = landed(&table);

        // when
        let edit = table.replace(0..0, "\n\n");
        colours.edited(1, vec![edit]);
        let job = colours.job(1, &table).unwrap();
        let shifted = colours.highlights(all(&table));
        colours.parsed(1, job.run().unwrap());

        // then
        assert_eq!(shifted, fresh(&table));
        assert_eq!(colours.highlights(all(&table)), fresh(&table));
    }

    #[test]
    fn should_apply_a_late_parse_shifted_by_later_edits() {
        // given
        let mut table = table("fn main() {}\n");
        let mut colours = landed(&table);
        colours.edited(1, vec![table.replace(0..0, "// ")]);
        let late = colours.job(1, &table).unwrap();
        let commented = table.bytes(all(&table));
        colours.edited(2, vec![table.replace(0..0, "\n")]);
        let while_parsing = colours.job(2, &table);
        colours.edited(3, vec![table.replace(0..0, "\n")]);

        // when
        colours.parsed(1, late.run().unwrap());
        let shown = colours.highlights(all(&table));
        let next = colours.job(3, &table).unwrap();
        let again = colours.job(3, &table);

        // then
        let parsed = parse(&commented, lang("Rust"), &Cancel::default(), PARSE_BUDGET).unwrap();
        let shifted: Vec<_> = spans(&parsed, &commented, 0..commented.len())
            .into_iter()
            .map(|Highlight { bytes, key }| Highlight { bytes: bytes.start + 2..bytes.end + 2, key })
            .collect();
        assert_eq!(shown, shifted);
        assert!(while_parsing.is_none());
        assert_eq!(next.revision(), 3);
        assert!(again.is_none());
        colours.parsed(3, next.run().unwrap());
        assert_eq!(colours.highlights(all(&table)), fresh(&table));
    }

    #[test]
    fn should_keep_one_parse_in_flight_while_typing() {
        // given
        let mut table = table("fn main() {\n}\n");
        let mut colours = landed(&table);
        colours.edited(1, vec![table.replace(12..12, "l")]);
        let first = colours.job(1, &table).unwrap();

        // when
        let while_parsing: Vec<_> = (2..22)
            .filter_map(|revision| {
                let end = table.byte_len() - 2;
                colours.edited(revision, vec![table.replace(end..end, "x")]);
                colours.job(revision, &table)
            })
            .collect();
        colours.parsed(1, first.run().unwrap());
        let next = colours.job(21, &table).unwrap();
        let again = colours.job(21, &table);

        // then
        assert!(while_parsing.is_empty());
        assert!(again.is_none());
        colours.parsed(21, next.run().unwrap());
        assert_eq!(colours.highlights(all(&table)), fresh(&table));
    }

    #[test]
    fn should_parse_again_after_a_parse_failed() {
        // given
        let mut table = table("fn main() {}\n");
        let mut colours = landed(&table);
        colours.edited(1, vec![table.replace(0..0, "// ")]);
        let failing = colours.job(1, &table).unwrap();
        // what `run` does when the parse ends without a result, e.g. over its time budget
        failing.failed.set();

        // when
        colours.edited(2, vec![table.replace(0..0, "\n")]);
        let next = colours.job(2, &table).unwrap();

        // then
        colours.parsed(2, next.run().unwrap());
        assert_eq!(colours.highlights(all(&table)), fresh(&table));
    }

    #[test]
    fn should_reuse_line_highlights_until_an_edit_or_a_landed_parse() {
        // given
        let mut table = table("fn main() {}\nfn other() {}\n");
        let mut colours = landed(&table);
        let first = colours.line_highlights(0..13);
        let moved: Vec<_> =
            first.iter().map(|Highlight { bytes, key }| Highlight { bytes: bytes.start + 3..bytes.end + 3, key: *key }).collect();
        colours.start_frame();

        // when
        let next_frame = colours.line_highlights(0..13);
        colours.edited(1, vec![table.replace(0..0, "// ")]);
        let after_edit = colours.line_highlights(0..16);
        let job = colours.job(1, &table).unwrap();
        colours.parsed(1, job.run().unwrap());
        let after_parse = colours.line_highlights(0..16);

        // then
        assert!(Rc::ptr_eq(&first, &next_frame));
        assert_eq!(*after_edit, moved);
        assert_eq!(*after_parse, colours_of(&fresh(&table), 0..16));
    }

    /// `highlights` within `range`.
    fn colours_of(highlights: &[Highlight], range: Range<usize>) -> Vec<Highlight> {
        highlights.iter().cloned().filter_map(|highlight| clipped(highlight, &range)).collect()
    }

    #[test]
    fn should_reparse_like_a_fresh_parse_after_undo_of_a_grouped_change() {
        // given
        let mut model = EditorModel::new(table("fn main() {\n    let n = 1;\n}\nfn other() {}\n"), None);
        model.set_caret(Position::new(1, 4), Selecting::Move);
        for typed in ["/", "*", " "] {
            model.insert(typed, EditKind::Typing).unwrap();
        }
        let typed = model.table().bytes(all(model.table()));
        let old = parse(&typed, lang("Rust"), &Cancel::default(), PARSE_BUDGET).unwrap();
        model.take_replacements();

        // when
        model.undo();
        let edits = model.take_replacements();
        let text = model.table().bytes(all(model.table()));
        let reparsed = reparse(&old, &edits, &text, &Cancel::default(), PARSE_BUDGET).unwrap();

        // then
        assert_eq!(edits.len(), 3);
        assert_eq!(spans(&reparsed, &text, 0..text.len()), fresh(model.table()));
    }

    #[test]
    fn should_reparse_like_a_fresh_parse_across_crlf_and_multibyte_columns() {
        // given
        let mut model = EditorModel::new(table("fn main() {\r\n    let s = \"żółć\";\r\n    let n = 1;\r\n}\r\n"), None);
        let old_text = model.table().bytes(all(model.table()));
        let old = parse(&old_text, lang("Rust"), &Cancel::default(), PARSE_BUDGET).unwrap();

        // when
        model.set_caret(Position::new(1, 19), Selecting::Move);
        model.insert("\r    let m = \"ąę\"; /* c */", EditKind::Other).unwrap();
        model.set_caret(Position::new(1, 17), Selecting::Move);
        model.insert("\" + x; // ", EditKind::Other).unwrap();
        let edits = model.take_replacements();
        let text = model.table().bytes(all(model.table()));
        let reparsed = reparse(&old, &edits, &text, &Cancel::default(), PARSE_BUDGET).unwrap();

        // then
        assert_eq!(text.windows(2).filter(|pair| pair == b"\r\n").count(), 5);
        assert_eq!(spans(&reparsed, &text, 0..text.len()), fresh(model.table()));
    }

    #[test]
    fn should_cut_the_replaced_text_out_of_a_highlight() {
        // given
        let highlight = Highlight { bytes: 2..10, key: crate::colors::ColorKey::TEXT };
        let edit = InputEdit {
            start_byte: 4,
            old_end_byte: 6,
            new_end_byte: 9,
            start_position: tree_sitter::Point::default(),
            old_end_position: tree_sitter::Point::default(),
            new_end_position: tree_sitter::Point::default(),
        };

        // when
        let parts: Vec<_> = after(highlight, &edit).into_iter().map(|part| part.bytes).collect();

        // then
        assert_eq!(parts, [2..4, 9..13]);
    }
}
