//! The editor's colours: the last parse that landed, shifted by the edits made since, until the next one lands.

use std::ops::Range;

use tree_sitter::InputEdit;

use crate::document::Source;
use crate::panel::{ParseJob, ParseTarget, Previous};
use crate::piece_table::PieceTable;
use crate::syntax::{self, Cancel, Highlight, Language, MAX_HIGHLIGHT_BYTES, Syntax};

/// Highlighting of an edited text: a parse is asked for after each edit, and the newest one asked for lands.
pub struct Colours {
    highlighting: Highlighting,
    landed: Landed,
    /// the edits since the landed parse's text, in order
    since: Vec<InputEdit>,
    wanted: Wanted,
}

enum Highlighting {
    Plain,
    In(Language),
}

enum Landed {
    Nothing,
    Parse { syntax: Syntax, source: Source },
}

/// The parse asked for last, of the text at `revision`.
enum Wanted {
    Nothing,
    Parse { revision: u64, source: Source, cancel: Cancel },
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
        Self { highlighting, landed: Landed::Nothing, since: Vec::new(), wanted: Wanted::Nothing }
    }

    /// The text changed by `edits`; the colours shown move with it.
    pub fn edited(&mut self, edits: Vec<InputEdit>) {
        if let Landed::Parse { .. } = self.landed {
            self.since.extend(edits);
        }
    }

    /// The parse of `table`'s text at `revision`, reusing the landed one; it cancels the parse asked for before.
    /// `None` without a language, and over [`MAX_HIGHLIGHT_BYTES`], where the text shows plain.
    pub fn job(&mut self, revision: u64, table: &PieceTable) -> Option<ParseJob> {
        self.cancel();
        let Highlighting::In(language) = self.highlighting else {
            return None;
        };
        if table.byte_len() > MAX_HIGHLIGHT_BYTES as u64 {
            self.landed = Landed::Nothing;
            self.since.clear();
            return None;
        }
        // the one copy of the whole text, under 1 ms at the 8 MiB limit; the highlights read it too
        let source = Source::from_vec(table.bytes(0..table.byte_len()));
        let cancel = Cancel::default();
        let previous = match &self.landed {
            Landed::Nothing => Previous::Nothing,
            Landed::Parse { syntax, .. } => Previous::Edited { syntax: syntax.clone(), edits: self.since.clone() },
        };
        self.wanted = Wanted::Parse { revision, source: source.clone(), cancel: cancel.clone() };
        Some(ParseJob { source, language, cancel, target: ParseTarget::Text, revision, previous })
    }

    /// A parse landed; one of another revision than the last asked for is stale and dropped.
    pub fn parsed(&mut self, revision: u64, syntax: Syntax) {
        let Wanted::Parse { revision: wanted, source, .. } = &self.wanted else {
            return;
        };
        if *wanted != revision {
            return;
        }
        self.landed = Landed::Parse { syntax, source: source.clone() };
        self.since.clear();
        self.wanted = Wanted::Nothing;
    }

    /// Highlights of `bytes` of the current text: the landed parse's, moved by the edits since; text an edit put in
    /// stays plain until the next parse lands.
    pub fn highlights(&self, bytes: Range<u64>) -> Vec<Highlight> {
        let Landed::Parse { syntax, source } = &self.landed else {
            return Vec::new();
        };
        let current = bytes.start as usize..bytes.end as usize;
        let len = source.as_bytes().len();
        let old = self.since.iter().rev().fold(current.clone(), |range, edit| {
            before(range.start, edit, End::Start)..before(range.end, edit, End::End)
        });
        let mut highlights = syntax::spans(syntax, source.as_bytes(), old.start.min(len)..old.end.min(len));
        for edit in &self.since {
            highlights = highlights.into_iter().flat_map(|highlight| after(highlight, edit)).collect();
        }
        highlights.into_iter().filter_map(|highlight| clipped(highlight, &current)).collect()
    }

    fn cancel(&mut self) {
        if let Wanted::Parse { cancel, .. } = &self.wanted {
            cancel.cancel();
        }
        self.wanted = Wanted::Nothing;
    }
}

impl Drop for Colours {
    fn drop(&mut self) {
        self.cancel();
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
    use crate::syntax::{PARSE_BUDGET, lang, parse, spans};

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
        colours.edited(vec![edit]);
        let job = colours.job(1, &table).unwrap();
        let shifted = colours.highlights(all(&table));
        colours.parsed(1, job.run().unwrap());

        // then
        assert_eq!(shifted, fresh(&table));
        assert_eq!(colours.highlights(all(&table)), fresh(&table));
    }

    #[test]
    fn should_drop_a_parse_of_an_older_revision() {
        // given
        let mut table = table("fn main() {}\n");
        let mut colours = landed(&table);
        colours.edited(vec![table.replace(0..0, "// ")]);
        let older = colours.job(1, &table).unwrap();
        let commented = table.bytes(all(&table));
        colours.edited(vec![table.replace(table.byte_len()..table.byte_len(), "x")]);
        let newer = colours.job(2, &table).unwrap();
        let shown = colours.highlights(all(&table));

        // when
        colours.parsed(1, parse(&commented, lang("Rust"), &Cancel::default(), PARSE_BUDGET).unwrap());

        // then
        assert!(older.cancel().is_cancelled());
        assert_eq!(colours.highlights(all(&table)), shown);
        colours.parsed(2, newer.run().unwrap());
        assert_eq!(colours.highlights(all(&table)), fresh(&table));
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
