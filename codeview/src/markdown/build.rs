//! The document Rendered shows, built off the UI thread: the text parsed, then its code blocks coloured as the editor
//! colours their languages.

use super::document::{BlockKind, Doc, Look, Run, parse};
use crate::syntax::{self, Cancel, Language, PARSE_BUDGET, spans};

/// Code blocks longer than this stay plain: colouring them would hold the document back.
const MAX_COLOURED_BLOCK_BYTES: usize = 256 * 1024;

/// `source` parsed (invalid UTF-8 as U+FFFD) with its code blocks coloured; stops colouring once `cancel` is set.
pub fn build(source: &[u8], cancel: &Cancel) -> Doc {
    let mut doc = parse(&String::from_utf8_lossy(source));
    let Doc { text, blocks } = &mut doc;
    for block in blocks.iter_mut() {
        let BlockKind::Code { language: Some(language) } = block.kind else {
            continue;
        };
        for piece in &mut block.pieces {
            piece.runs = coloured(&text[piece.text.clone()], language, cancel).map(|run| Run { bytes: run.bytes.start + piece.text.start..run.bytes.end + piece.text.start, ..run }).collect();
        }
    }
    doc
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
