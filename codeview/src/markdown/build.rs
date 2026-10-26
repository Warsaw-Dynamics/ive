//! The document Rendered shows, built off the UI thread: the text parsed, then its code blocks coloured as the editor
//! colours their languages.

use super::document::{BlockKind, Doc, Look, Run, parse};
use super::split::split_long;
use crate::syntax::{self, Cancel, Language, PARSE_BUDGET, spans};

/// Code blocks longer than this stay plain: colouring them would hold the document back.
const MAX_COLOURED_BLOCK_BYTES: usize = 256 * 1024;

/// `source` parsed (invalid UTF-8 as U+FFFD) with its code blocks coloured, long code blocks and tables cut into parts;
/// stops colouring once `cancel` is set.
pub fn build(source: &[u8], cancel: &Cancel) -> Doc {
    let source = String::from_utf8_lossy(source);
    let mut doc = parse(&source);
    let Doc { text, blocks } = &mut doc;
    for block in blocks.iter_mut() {
        let BlockKind::Code { language: Some(language) } = block.kind else {
            continue;
        };
        for piece in &mut block.pieces {
            piece.runs = coloured(&text[piece.text.clone()], language, cancel).map(|run| Run { bytes: run.bytes.start + piece.text.start..run.bytes.end + piece.text.start, ..run }).collect();
        }
    }
    split_long(doc, &source)
}

/// The colour runs of `code` in `language`, by bytes of `code`; none when cancelled, too long or not parsed.
fn coloured(code: &str, language: Language, cancel: &Cancel) -> impl Iterator<Item = Run> {
    let fits = code.len() <= MAX_COLOURED_BLOCK_BYTES && !cancel.is_cancelled();
    let syntax = fits.then(|| syntax::parse(code.as_bytes(), language, cancel, PARSE_BUDGET).ok()).flatten();
    let highlights = syntax.map_or_else(Vec::new, |syntax| spans(&syntax, code.as_bytes(), 0..code.len()));
    highlights.into_iter().map(|highlight| Run { bytes: highlight.bytes, look: Look::Syntax(highlight.key) })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markdown::Part;

    fn syntax_runs(doc: &Doc) -> Vec<&str> {
        let runs = doc.blocks.iter().flat_map(|block| block.pieces.iter()).flat_map(|piece| piece.runs.iter());
        runs.filter(|run| matches!(run.look, Look::Syntax(_))).map(|run| &doc.text[run.bytes.clone()]).collect()
    }

    #[test]
    fn should_colour_a_known_fence() {
        // given
        let source = b"```rust\nfn a() {}\n```\n";

        // when
        let doc = build(source, &Cancel::default());

        // then
        assert!(syntax_runs(&doc).contains(&"fn"), "{:?}", syntax_runs(&doc));
    }

    #[test]
    fn should_leave_an_unknown_fence_plain() {
        // given
        let source = b"```nosuchlanguage\nfn a() {}\n```\n";

        // when
        let doc = build(source, &Cancel::default());

        // then
        assert!(syntax_runs(&doc).is_empty());
        assert_eq!(doc.text, "fn a() {}\n");
    }

    fn long_code(lines: usize) -> String {
        let code: String = (0..lines).map(|line| format!("let line_{line} = {line};\n")).collect();
        format!("```rust\n{code}```\n")
    }

    #[test]
    fn should_split_a_long_code_block_into_parts() {
        // given
        let source = long_code(250);

        // when
        let doc = build(source.as_bytes(), &Cancel::default());

        // then
        let parts: Vec<Part> = doc.blocks.iter().map(|block| block.part).collect();
        assert_eq!(parts, vec![Part::First, Part::Middle, Part::Last]);
        assert_eq!(doc.text, parse(&source).text);
        let starts: Vec<usize> = doc.blocks.iter().map(|block| block.lines.start).collect();
        assert_eq!(starts, vec![0, 101, 201]);
        assert_eq!(doc.code_of(1), parse(&source).code_of(0));
        assert_eq!(doc.copy_text(0..doc.text.len()), parse(&source).copy_text(0..doc.text.len()));
        assert!(doc.blocks[1].pieces[0].runs.iter().any(|run| matches!(run.look, Look::Syntax(_))), "the middle part keeps its colours");
    }

    #[test]
    fn should_split_a_long_table_by_rows() {
        // given
        let rows: String = (0..250).map(|row| format!("| {row} | x |\n")).collect();
        let source = format!("| n | v |\n|---|---|\n{rows}");

        // when
        let doc = build(source.as_bytes(), &Cancel::default());

        // then
        let parts: Vec<(Part, usize)> = doc.blocks.iter().map(|block| (block.part, block.pieces.len())).collect();
        assert_eq!(parts, vec![(Part::First, 2 + 2 * 99), (Part::Middle, 200), (Part::Last, 2 * 51)]);
        assert_eq!(doc.copy_text(0..doc.text.len()), parse(&source).copy_text(0..doc.text.len()));
    }

    #[test]
    fn should_keep_a_short_block_whole() {
        // given
        let source = long_code(100);

        // when
        let doc = build(source.as_bytes(), &Cancel::default());

        // then
        assert_eq!(doc.blocks.iter().map(|block| block.part).collect::<Vec<_>>(), vec![Part::Whole]);
    }

    #[test]
    fn should_stop_when_cancelled() {
        // given
        let cancel = Cancel::default();
        cancel.cancel();

        // when
        let doc = build(b"```rust\nfn a() {}\n```\n", &cancel);

        // then
        assert!(syntax_runs(&doc).is_empty());
        assert_eq!(doc.text, "fn a() {}\n");
    }
}
