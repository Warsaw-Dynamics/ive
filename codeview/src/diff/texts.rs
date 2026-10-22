//! The full texts behind a diff's files, asked for per file once it shows lines on screen, and the colours their
//! parses give the rows: inline, a removed line takes the old side's, an added or context line the new side's; side
//! by side, each panel takes its own side's.

use std::collections::HashMap;
use std::ops::Range;
use std::path::Path;

use super::data::{DiffLine, FileDiff, LineKind};
use super::rows::DiffRow;
use super::side_by_side::Side;
use crate::colors::ColorKey;
use crate::document::{DisplayLine, Source, TextSource};
use crate::panel::{Failed, LineSource, ParseJob, ParseTarget, Previous, TextPurpose, TextsRequest};
use crate::syntax::{self, Cancel, Language, Syntax};

/// A row's text as coloured ranges of it.
pub type Placed = (Range<usize>, ColorKey);

/// The answer to a parse a diff view asked for.
pub struct Parsed {
    pub target: ParseTarget,
    pub revision: u64,
    pub syntax: Syntax,
}

/// What a diff view knows of its files' full texts: which it asked for, and the sides that came.
#[derive(Default)]
pub(crate) struct DiffTexts {
    /// by file, the generation of the last ask: an answer to an older one is stale
    requested: HashMap<usize, u64>,
    /// counts the asks
    generation: u64,
    /// by [`ParseTarget::DiffOld`] / [`ParseTarget::DiffNew`]; a side that is missing, binary or too large is absent
    sides: HashMap<ParseTarget, SideText>,
    /// the last revision a side's parse was tagged with
    revision: u64,
}

/// One side's full text, plain until its parse lands.
struct SideText {
    text: TextSource,
    /// tags the parse asked for this text; an answer for another text is stale
    revision: u64,
    cancel: Cancel,
    colours: Colours,
}

enum Colours {
    Plain,
    Parsed(Syntax),
}

impl Drop for SideText {
    fn drop(&mut self) {
        self.cancel.cancel();
    }
}

impl DiffTexts {
    /// The asks for the files that show a line in `shown` (rows on screen) and were not asked for yet, each for what
    /// `purpose` says; they count as asked now.
    pub(crate) fn wanted(&mut self, shown: &[DiffRow], purpose: &dyn Fn(usize) -> TextPurpose) -> Vec<TextsRequest> {
        let mut wanted = Vec::new();
        for row in shown {
            if let DiffRow::Line { file, .. } | DiffRow::Pair { file, .. } = row
                && !self.requested.contains_key(file)
            {
                self.generation += 1;
                self.requested.insert(*file, self.generation);
                wanted.push(TextsRequest { file: *file, generation: self.generation, purpose: purpose(*file) });
            }
        }
        wanted
    }

    /// Whether `generation` is the last ask for file `file`'s texts.
    pub(crate) fn awaits(&self, file: usize, generation: u64) -> bool {
        self.requested.get(&file) == Some(&generation)
    }

    /// Forgets what is known of file `file`'s texts: they are asked for again when shown.
    pub(crate) fn forget(&mut self, file: usize) {
        self.requested.remove(&file);
        self.sides.retain(|target, _| file_of(*target) != file);
    }

    /// Forgets what is known of the files whose diff differs between `before` and `after`: their texts may have
    /// changed, so they are asked for again when shown.
    pub(crate) fn forget_changed(&mut self, before: &[FileDiff], after: &[FileDiff]) {
        let changed = |file: usize| before.get(file) != after.get(file);
        self.requested.retain(|file, _| !changed(*file));
        self.sides.retain(|target, _| !changed(file_of(*target)));
    }

    /// Keeps `source` as the side `target` of `diff` (file `n` of the view) when it answers the last ask, `generation`,
    /// for that file; the parse it needs when its language is known.
    pub(crate) fn set(&mut self, target: ParseTarget, generation: u64, source: Source, diff: &FileDiff) -> Option<ParseJob> {
        if !self.awaits(file_of(target), generation) {
            return None;
        }
        self.revision += 1;
        let side = SideText { text: TextSource::from_source(source.clone()), revision: self.revision, cancel: Cancel::default(), colours: Colours::Plain };
        let language = language_of(target, diff, source.as_bytes());
        let job = language.map(|language| ParseJob {
            source,
            language,
            cancel: side.cancel.clone(),
            target,
            revision: side.revision,
            previous: Previous::Nothing,
            failed: Failed::default(),
        });
        self.sides.insert(target, side);
        job
    }

    /// Colours the side `parsed` is for, unless its text was replaced since the parse was asked for. False when stale.
    pub(crate) fn parsed(&mut self, parsed: Parsed) -> bool {
        match self.sides.get_mut(&parsed.target) {
            Some(side) if side.revision == parsed.revision => {
                side.colours = Colours::Parsed(parsed.syntax);
                true
            }
            _ => false,
        }
    }

    /// `line` of file `file` as an inline diff shows it, as coloured ranges of its text; none while its side is not
    /// parsed.
    pub(crate) fn colours(&self, file: usize, line: &DiffLine) -> Vec<Placed> {
        let side = if line.kind == LineKind::Removed { Side::Old } else { Side::New };
        self.colours_on(file, line, side)
    }

    /// `line` of file `file` as panel `side` of a side-by-side diff shows it; a context line on the old side takes
    /// the new side's colours while the old text is missing.
    pub(crate) fn colours_on(&self, file: usize, line: &DiffLine, side: Side) -> Vec<Placed> {
        let old = self.sides.get(&ParseTarget::DiffOld(file));
        let (text, number) = match (side, line.kind, old) {
            (_, LineKind::NoNewline, _) => return Vec::new(),
            (Side::Old, LineKind::Context, None) => (self.sides.get(&ParseTarget::DiffNew(file)), line.new),
            (Side::Old, _, old) => (old, line.old),
            (Side::New, _, _) => (self.sides.get(&ParseTarget::DiffNew(file)), line.new),
        };
        match (text, number) {
            (Some(text), Some(number)) => text.colours(number as usize, &line.text),
            _ => Vec::new(),
        }
    }
}

impl SideText {
    /// Line `number` (from 1) coloured, as ranges of `shown`: the row's text, which git decoded like a
    /// [`DisplayLine`]. A text that does not have `shown` there (changed since the diff was read) colours nothing.
    fn colours(&self, number: usize, shown: &str) -> Vec<Placed> {
        let Colours::Parsed(syntax) = &self.colours else {
            return Vec::new();
        };
        let Some(line) = number.checked_sub(1).and_then(|index| self.text.lines(index..number).pop()) else {
            return Vec::new();
        };
        if !shows(&line, shown) {
            return Vec::new();
        }
        let source = self.text.bytes(0..self.text.byte_len());
        line.place(&syntax::spans(syntax, &source, line.file_bytes()))
    }
}

/// Whether `line` displays `row`, or the start of it when `line` is cut.
fn shows(line: &DisplayLine, row: &str) -> bool {
    match line.text.strip_suffix(" …") {
        Some(kept) => row == line.text.as_ref() || row.starts_with(kept),
        None => row == line.text.as_ref(),
    }
}

fn file_of(target: ParseTarget) -> usize {
    match target {
        ParseTarget::DiffOld(file) | ParseTarget::DiffNew(file) => file,
        // a text's own parse is no diff side
        ParseTarget::Text => usize::MAX,
    }
}

/// By the side's own name: a renamed file's old side by its old name.
fn language_of(target: ParseTarget, diff: &FileDiff, content: &[u8]) -> Option<Language> {
    let path = match target {
        ParseTarget::DiffOld(_) => diff.old_path.as_ref().unwrap_or(&diff.path),
        ParseTarget::DiffNew(_) | ParseTarget::Text => &diff.path,
    };
    Language::for_path(Path::new(&path.display()), content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::data::{DiffBody, GitPath, Hunk, Mark};

    fn line(kind: LineKind, old: Option<u32>, new: Option<u32>, text: &str) -> DiffLine {
        DiffLine { kind, old, new, text: text.to_owned() }
    }

    fn colours(_file: usize) -> TextPurpose {
        TextPurpose::Colours
    }

    fn rust_file(lines: Vec<DiffLine>) -> FileDiff {
        let hunk = Hunk { header: "@@ -1,3 +1,3 @@".to_owned(), lines };
        FileDiff { path: GitPath::from_bytes(b"src/a.rs"), old_path: None, mark: Mark::Modified, body: DiffBody::Text { hunks: vec![hunk], cut: None } }
    }

    /// `texts` with `text` set and parsed as side `target` of `diff`.
    fn parsed_side(texts: &mut DiffTexts, target: ParseTarget, text: &str, diff: &FileDiff) {
        let generation = texts.requested[&file_of(target)];
        let job = texts.set(target, generation, Source::from_vec(text.as_bytes().to_vec()), diff).expect("a Rust file is parsed");
        let revision = job.revision();
        texts.parsed(Parsed { target, revision, syntax: job.run().unwrap() });
    }

    fn keys(placed: &[Placed], text: &str) -> Vec<(String, String)> {
        placed.iter().map(|(range, key)| (text[range.clone()].to_owned(), key.id().to_owned())).collect()
    }

    #[test]
    fn should_colour_a_removed_line_from_the_old_side_and_an_added_line_from_the_new() {
        // given
        let removed = line(LineKind::Removed, Some(1), None, "let a = 1;");
        let added = line(LineKind::Added, None, Some(1), "\"text\"");
        let diff = rust_file(vec![removed.clone(), added.clone()]);
        let mut texts = DiffTexts::default();
        texts.wanted(&[DiffRow::Line { file: 0, hunk: 0, line: 0 }], &colours);
        parsed_side(&mut texts, ParseTarget::DiffOld(0), "let a = 1;\n", &diff);
        parsed_side(&mut texts, ParseTarget::DiffNew(0), "\"text\"\n", &diff);

        // when
        let colours = (texts.colours(0, &removed), texts.colours(0, &added));

        // then
        assert_eq!(keys(&colours.0, &removed.text)[0], ("let".to_owned(), "RUST.KEYWORD".to_owned()));
        assert_eq!(keys(&colours.1, &added.text), [("\"text\"".to_owned(), "RUST.STRING".to_owned())]);
    }

    #[test]
    fn should_colour_a_context_line_from_each_panels_side_and_from_the_new_side_without_an_old_text() {
        // given
        let context = line(LineKind::Context, Some(1), Some(2), "let a;");
        let diff = rust_file(vec![context.clone()]);
        let mut texts = DiffTexts::default();
        texts.wanted(&[DiffRow::Line { file: 0, hunk: 0, line: 0 }], &colours);
        parsed_side(&mut texts, ParseTarget::DiffNew(0), "\"x\"\nlet a;\n", &diff);
        let without_old = texts.colours_on(0, &context, Side::Old);
        parsed_side(&mut texts, ParseTarget::DiffOld(0), "/* old */\n", &diff);

        // when
        let (old, new) = (texts.colours_on(0, &context, Side::Old), texts.colours_on(0, &context, Side::New));

        // then
        assert_eq!(keys(&without_old, &context.text)[0], ("let".to_owned(), "RUST.KEYWORD".to_owned()));
        // the old text has another line 1: the old panel shows the line uncoloured
        assert_eq!((old, keys(&new, &context.text)[0].clone()), (vec![], ("let".to_owned(), "RUST.KEYWORD".to_owned())));
    }

    #[test]
    fn should_place_colours_after_a_tab_the_diff_shows_as_spaces() {
        // given
        let added = line(LineKind::Added, None, Some(1), "    let a;");
        let diff = rust_file(vec![added.clone()]);
        let mut texts = DiffTexts::default();
        texts.wanted(&[DiffRow::Line { file: 0, hunk: 0, line: 0 }], &colours);
        parsed_side(&mut texts, ParseTarget::DiffNew(0), "\tlet a;\n", &diff);

        // when
        let colours = texts.colours(0, &added);

        // then
        assert_eq!(keys(&colours, &added.text)[0], ("let".to_owned(), "RUST.KEYWORD".to_owned()));
    }

    #[test]
    fn should_not_colour_a_line_the_text_no_longer_has() {
        // given
        let added = line(LineKind::Added, None, Some(1), "let a;");
        let diff = rust_file(vec![added.clone()]);
        let mut texts = DiffTexts::default();
        texts.wanted(&[DiffRow::Line { file: 0, hunk: 0, line: 0 }], &colours);
        parsed_side(&mut texts, ParseTarget::DiffNew(0), "fn changed_since() {}\n", &diff);

        // when
        let colours = texts.colours(0, &added);

        // then
        assert_eq!(colours, []);
    }

    #[test]
    fn should_drop_a_parse_of_a_replaced_text_and_texts_no_longer_waited_for() {
        // given
        let added = line(LineKind::Added, None, Some(1), "let a;");
        let diff = rust_file(vec![added.clone()]);
        let mut texts = DiffTexts::default();
        let generation = texts.wanted(&[DiffRow::Line { file: 0, hunk: 0, line: 0 }], &colours)[0].generation;
        let first = texts.set(ParseTarget::DiffNew(0), generation, Source::from_vec(b"let a;\n".to_vec()), &diff).unwrap();
        let second = texts.set(ParseTarget::DiffNew(0), generation, Source::from_vec(b"let a;\n".to_vec()), &diff).unwrap();

        // when
        let stale = texts.parsed(Parsed { target: ParseTarget::DiffNew(0), revision: first.revision(), syntax: second.run().unwrap() });
        let unwanted = texts.set(ParseTarget::DiffNew(1), generation, Source::from_vec(b"let a;\n".to_vec()), &diff);

        // then
        assert!(!stale && unwanted.is_none() && first.cancel().is_cancelled());
        assert_eq!(texts.colours(0, &added), []);
    }

    #[test]
    fn should_ask_again_only_for_files_whose_diff_changed() {
        // given
        let before = vec![rust_file(vec![line(LineKind::Added, None, Some(1), "a")]), rust_file(vec![line(LineKind::Added, None, Some(1), "b")])];
        let mut after = before.clone();
        after[1] = rust_file(vec![line(LineKind::Added, None, Some(1), "c")]);
        let shown = [DiffRow::Line { file: 0, hunk: 0, line: 0 }, DiffRow::Line { file: 1, hunk: 0, line: 0 }];
        let mut texts = DiffTexts::default();
        let files = |asks: Vec<TextsRequest>| asks.iter().map(|ask| ask.file).collect::<Vec<_>>();
        let first = files(texts.wanted(&shown, &colours));

        // when
        texts.forget_changed(&before, &after);

        // then
        assert_eq!((first, files(texts.wanted(&shown, &colours))), (vec![0, 1], vec![1]));
    }
}
