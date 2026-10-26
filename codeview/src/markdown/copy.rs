//! What Rendered copies, and the words and pieces a double or triple click selects.

use std::ops::Range;

use super::document::{Block, BlockKind, Doc, Marker, Piece, Tick};

impl Doc {
    /// The visible text of `range` (bytes of `text`) as plain text: blocks apart by a blank line, list items on their
    /// own lines with their markers, table rows on their own lines with their cells apart by tabs.
    pub fn copy_text(&self, range: Range<usize>) -> String {
        let mut copied = String::new();
        let mut previous: Option<&Block> = None;
        for block in self.blocks.iter().filter(|block| touches(block, &range)) {
            if let Some(before) = previous {
                copied.push_str(separator(before, block));
            }
            copied.push_str(&self.block_text(block, &range));
            previous = Some(block);
        }
        copied
    }

    /// Block `block`'s code as written, without the line break that ends it; `None` for a block that is not code.
    pub fn code_of(&self, block: usize) -> Option<String> {
        let block = self.blocks.get(block).filter(|block| matches!(block.kind, BlockKind::Code { .. }))?;
        let piece = block.pieces.first()?;
        let code = &self.text[piece.text.clone()];
        Some(code.strip_suffix('\n').unwrap_or(code).to_owned())
    }

    /// The word around `offset`, inside the piece holding it; the character at it when that is no word character.
    pub fn word_at(&self, offset: usize) -> Range<usize> {
        let piece = self.piece_range(offset);
        let text = &self.text[piece.clone()];
        let at = (offset.min(piece.end) - piece.start).min(text.len());
        let start = text[..at].char_indices().rev().take_while(|(_, char)| is_word(*char)).last().map_or(at, |(index, _)| index);
        let end = text[at..].char_indices().find(|(_, char)| !is_word(*char)).map_or(text.len(), |(index, _)| at + index);
        if start < end {
            return piece.start + start..piece.start + end;
        }
        let next = text[at..].chars().next().map_or(at, |char| at + char.len_utf8());
        piece.start + at..piece.start + next
    }

    /// The block and the piece in it holding `offset`: the last piece starting at or before it.
    pub fn piece_at(&self, offset: usize) -> (usize, usize) {
        let pieces = self.blocks.iter().enumerate().flat_map(|(block, found)| found.pieces.iter().enumerate().map(move |(index, piece)| (block, index, piece)));
        pieces.take_while(|(_, _, piece)| piece.text.start <= offset).last().map_or((0, 0), |(block, index, _)| (block, index))
    }

    /// The whole piece holding `offset`.
    pub fn piece_range(&self, offset: usize) -> Range<usize> {
        let (block, index) = self.piece_at(offset);
        self.blocks.get(block).and_then(|block| block.pieces.get(index)).map_or(0..0, |piece| piece.text.clone())
    }

    /// The part of `block` inside `range`, with a list item's marker when the range starts at or before its text.
    fn block_text(&self, block: &Block, range: &Range<usize>) -> String {
        let clipped = |piece: &Piece| &self.text[piece.text.start.max(range.start)..piece.text.end.min(range.end).max(piece.text.start.max(range.start))];
        match &block.kind {
            BlockKind::Table { columns, .. } => {
                let rows = block.pieces.chunks(columns.len().max(1)).filter(|row| row.iter().any(|cell| overlaps(&cell.text, range)));
                rows.map(|row| row.iter().map(clipped).collect::<Vec<_>>().join("\t")).collect::<Vec<_>>().join("\n")
            }
            BlockKind::Code { .. } => block.pieces.iter().map(clipped).collect::<String>().trim_end_matches('\n').to_owned(),
            BlockKind::ListItem(marker) => {
                let text: String = block.pieces.iter().map(clipped).collect();
                let starts_here = block.pieces.first().is_some_and(|piece| range.start <= piece.text.start);
                if starts_here {
                    return format!("{}{}{text}", "  ".repeat(usize::from(block.nesting.lists.saturating_sub(1))), marker_text(*marker));
                }
                text
            }
            _ => block.pieces.iter().map(clipped).collect(),
        }
    }
}

/// Between two copied blocks: a line break between list items of one list, a blank line otherwise.
fn separator(before: &Block, after: &Block) -> &'static str {
    let listed = matches!(before.kind, BlockKind::ListItem(_)) && matches!(after.kind, BlockKind::ListItem(_));
    if listed && after.lines.start <= before.lines.end {
        return "\n";
    }
    "\n\n"
}

fn marker_text(marker: Marker) -> String {
    match marker {
        Marker::Bullet => "- ".to_owned(),
        Marker::Number(number) => format!("{number}. "),
        Marker::Task(Tick::Open) => "- [ ] ".to_owned(),
        Marker::Task(Tick::Done) => "- [x] ".to_owned(),
    }
}

/// Whether `block` has text inside `range`, or is wholly inside it.
fn touches(block: &Block, range: &Range<usize>) -> bool {
    block.pieces.iter().any(|piece| overlaps(&piece.text, range))
}

fn overlaps(text: &Range<usize>, range: &Range<usize>) -> bool {
    text.start < range.end && range.start < text.end
}

fn is_word(char: char) -> bool {
    char.is_alphanumeric() || char == '_'
}

#[cfg(test)]
mod tests {
    use crate::markdown::parse;

    #[test]
    fn should_copy_blocks_separated_by_blank_lines() {
        // given
        let doc = parse("# A\n\npara one\n");

        // when
        let copied = doc.copy_text(0..doc.text.len());

        // then
        assert_eq!(copied, "A\n\npara one");
    }

    #[test]
    fn should_copy_list_items_with_their_markers() {
        // given
        let doc = parse("- a\n- [x] b\n  - [ ] n\n\n1. c\n");

        // when
        let copied = doc.copy_text(0..doc.text.len());

        // then
        assert_eq!(copied, "- a\n- [x] b\n  - [ ] n\n\n1. c");
    }

    #[test]
    fn should_copy_table_rows_with_tabs() {
        // given
        let doc = parse("| App | Stack |\n|:---|---:|\n| ive | Rust |\n");

        // when
        let copied = doc.copy_text(0..doc.text.len());

        // then
        assert_eq!(copied, "App\tStack\nive\tRust");
    }

    #[test]
    fn should_copy_a_part_of_one_piece() {
        // given
        let doc = parse("- para\n");

        // when
        let copied = doc.copy_text(1..3);

        // then
        assert_eq!(copied, "ar");
    }

    #[test]
    fn should_copy_code_as_written() {
        // given
        let doc = parse("```\nx\n  y\n```\n\npara\n");

        // when
        let code = (doc.code_of(0), doc.code_of(1), doc.copy_text(0..doc.text.len()));

        // then
        assert_eq!(code, (Some("x\n  y".to_owned()), None, "x\n  y\n\npara".to_owned()));
    }

    #[test]
    fn should_select_a_word_within_its_piece() {
        // given
        let doc = parse("| App | Stack |\n|---|---|\n");

        // when
        let words = (doc.word_at(1), doc.word_at(3), doc.word_at(5));

        // then
        assert_eq!(words, (0..3, 0..3, 4..9));
    }

    #[test]
    fn should_select_the_whole_piece() {
        // given
        let doc = parse("| App | Stack |\n|---|---|\n\npara two\n");

        // when
        let pieces = (doc.piece_range(5), doc.piece_at(5), doc.piece_at(12));

        // then
        assert_eq!(pieces, (4..9, (0, 1), (1, 0)));
    }
}
