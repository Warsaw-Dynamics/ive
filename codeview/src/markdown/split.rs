//! Long code blocks and tables cut into parts of [`PART_LINES`] lines or rows, each a block of its own, so the view's
//! list lays out and draws only the parts in sight. The document's text stays as it was: a cut is at a line break,
//! which becomes the break between two pieces.

use std::ops::Range;

use super::document::{Block, BlockKind, Doc, Part, Piece, Run};

/// Lines of code, or rows of a table, in one part.
const PART_LINES: usize = 100;

/// `doc` with its long code blocks and tables cut into parts; `source` is the Markdown it was parsed from.
pub(super) fn split_long(doc: Doc, source: &str) -> Doc {
    let Doc { text, blocks } = doc;
    let blocks = blocks.into_iter().flat_map(|block| split_block(block, &text, source)).collect();
    Doc { text, blocks }
}

fn split_block(block: Block, text: &str, source: &str) -> Vec<Block> {
    match &block.kind {
        BlockKind::Code { .. } => split_code(block, text, source),
        BlockKind::Table { columns, .. } => {
            let columns = columns.len().max(1);
            split_table(block, columns)
        }
        _ => vec![block],
    }
}

/// A code block cut after every [`PART_LINES`]th line break.
fn split_code(block: Block, text: &str, source: &str) -> Vec<Block> {
    let Some(piece) = block.pieces.first() else {
        return vec![block];
    };
    let breaks = text[piece.text.clone()].match_indices('\n').map(|(at, _)| piece.text.start + at);
    let cuts: Vec<usize> =
        breaks.enumerate().filter(|(count, _)| (count + 1) % PART_LINES == 0).map(|(_, at)| at).filter(|at| at + 1 < piece.text.end).collect();
    if cuts.is_empty() {
        return vec![block];
    }
    let starts = std::iter::once(piece.text.start).chain(cuts.iter().map(|cut| cut + 1));
    let ends = cuts.iter().copied().chain(std::iter::once(piece.text.end));
    let texts: Vec<Range<usize>> = starts.zip(ends).map(|(start, end)| start..end).collect();
    // a fence's own line comes before the code's first line
    let fence = usize::from(source.lines().nth(block.lines.start).is_some_and(|line| line.trim_start().starts_with(['`', '~'])));
    let first_lines: Vec<usize> =
        (0..texts.len()).map(|part| if part == 0 { block.lines.start } else { block.lines.start + fence + part * PART_LINES }).collect();
    let pieces: Vec<Vec<Piece>> = texts.iter().map(|range| vec![Piece { text: range.clone(), runs: clipped(&piece.runs, range) }]).collect();
    parts_of(&block, pieces, &first_lines)
}

/// A table cut every [`PART_LINES`] rows, the header row in the first part.
fn split_table(block: Block, columns: usize) -> Vec<Block> {
    let rows: Vec<&[Piece]> = block.pieces.chunks(columns).collect();
    if rows.len() <= PART_LINES {
        return vec![block];
    }
    let pieces: Vec<Vec<Piece>> = rows.chunks(PART_LINES).map(|part| part.iter().flat_map(|row| row.iter().cloned()).collect()).collect();
    // the header's line, the separator's, then a row a line
    let first_lines: Vec<usize> =
        (0..pieces.len()).map(|part| if part == 0 { block.lines.start } else { block.lines.start + 1 + part * PART_LINES }).collect();
    parts_of(&block, pieces, &first_lines)
}

/// `block` as parts holding `pieces`, each starting at its source line of `first_lines`.
fn parts_of(block: &Block, pieces: Vec<Vec<Piece>>, first_lines: &[usize]) -> Vec<Block> {
    let last = pieces.len() - 1;
    pieces
        .into_iter()
        .enumerate()
        .map(|(index, pieces)| {
            let end = first_lines.get(index + 1).copied().unwrap_or(block.lines.end);
            let part = match index {
                0 => Part::First,
                at if at == last => Part::Last,
                _ => Part::Middle,
            };
            Block { kind: block.kind.clone(), nesting: block.nesting, lines: first_lines[index]..end, pieces, part }
        })
        .collect()
}

/// The parts of `runs` inside `range`.
fn clipped(runs: &[Run], range: &Range<usize>) -> Vec<Run> {
    let inside = runs.iter().filter(|run| run.bytes.start < range.end && range.start < run.bytes.end);
    inside.map(|run| Run { bytes: run.bytes.start.max(range.start)..run.bytes.end.min(range.end), look: run.look.clone() }).collect()
}
