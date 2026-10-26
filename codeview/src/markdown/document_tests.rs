use super::*;
use crate::syntax::Language;

/// Each block's kind.
fn kinds(doc: &Doc) -> Vec<BlockKind> {
    doc.blocks.iter().map(|block| block.kind.clone()).collect()
}

/// The text of each piece of each block, in order.
fn texts(doc: &Doc) -> Vec<&str> {
    doc.blocks.iter().flat_map(|block| block.pieces.iter()).map(|piece| &doc.text[piece.text.clone()]).collect()
}

/// The looks of the runs over `word` in block `block`.
fn looks_of<'a>(doc: &'a Doc, block: usize, word: &str) -> Vec<&'a Look> {
    let runs = doc.blocks[block].pieces.iter().flat_map(|piece| piece.runs.iter());
    runs.filter(|run| &doc.text[run.bytes.clone()] == word).map(|run| &run.look).collect()
}

fn nesting(doc: &Doc) -> Vec<(u8, u8)> {
    doc.blocks.iter().map(|block| (block.nesting.quotes, block.nesting.lists)).collect()
}

#[test]
fn should_parse_an_empty_text_to_no_blocks() {
    // given
    let texts = ["", " \n\n"];

    // when
    let docs: Vec<Doc> = texts.iter().map(|text| parse(text)).collect();

    // then
    assert!(docs.iter().all(|doc| doc.blocks.is_empty() && doc.text.is_empty()));
}

#[test]
fn should_parse_headings_paragraphs_and_inline_looks() {
    // given
    let source = "# Title\n\nSome **bold** and *it* and `code` and [l](u).\n";

    // when
    let doc = parse(source);

    // then
    assert_eq!(kinds(&doc), vec![BlockKind::Heading(1), BlockKind::Paragraph]);
    assert_eq!(texts(&doc), vec!["Title", "Some bold and it and code and l."]);
    assert_eq!(looks_of(&doc, 1, "bold"), vec![&Look::Strong]);
    assert_eq!(looks_of(&doc, 1, "it"), vec![&Look::Emphasis]);
    assert_eq!(looks_of(&doc, 1, "code"), vec![&Look::Code]);
    assert_eq!(looks_of(&doc, 1, "l"), vec![&Look::Link("u".into())]);
    assert_eq!(doc.blocks[1].lines, 2..3);
}

#[test]
fn should_nest_lists_and_tasks() {
    // given
    let source = "- [x] a\n- [ ] b\n  - c\n1. d\n";

    // when
    let doc = parse(source);

    // then
    assert_eq!(
        kinds(&doc),
        vec![
            BlockKind::ListItem(Marker::Task(Tick::Done)),
            BlockKind::ListItem(Marker::Task(Tick::Open)),
            BlockKind::ListItem(Marker::Bullet),
            BlockKind::ListItem(Marker::Number(1)),
        ]
    );
    assert_eq!(texts(&doc), vec!["a", "b", "c", "d"]);
    assert_eq!(nesting(&doc), vec![(0, 1), (0, 1), (0, 2), (0, 1)]);
}

#[test]
fn should_quote_nested_blocks() {
    // given
    let source = "> q\n> - i\n";

    // when
    let doc = parse(source);

    // then
    assert_eq!(kinds(&doc), vec![BlockKind::Paragraph, BlockKind::ListItem(Marker::Bullet)]);
    assert_eq!(texts(&doc), vec!["q", "i"]);
    assert_eq!(nesting(&doc), vec![(1, 0), (1, 1)]);
}

#[test]
fn should_make_table_cells_pieces() {
    // given
    let source = "| App | Stack |\n|:---|---:|\n| ive | Rust |\n";

    // when
    let doc = parse(source);

    // then
    assert_eq!(kinds(&doc), vec![BlockKind::Table { columns: vec![Align::Left, Align::Right], header_cells: 2 }]);
    assert_eq!(texts(&doc), vec!["App", "Stack", "ive", "Rust"]);
}

#[test]
fn should_pad_or_cut_ragged_table_rows() {
    // given
    let source = "| a | b |\n|---|---|\n| one |\n| x | y | z |\n";

    // when
    let doc = parse(source);

    // then
    assert_eq!(texts(&doc), vec!["a", "b", "one", "", "x", "y"]);
}

#[test]
fn should_keep_code_lines_and_language() {
    // given
    let fenced = "```rust\nfn a() {}\n\n```\n";
    let unknown = "```nosuchlanguage\nx\n```\n";
    let indented = "    indented\n";

    // when
    let docs = [parse(fenced), parse(unknown), parse(indented)];

    // then
    assert_eq!(kinds(&docs[0]), vec![BlockKind::Code { language: Language::for_alias("rust") }]);
    assert!(Language::for_alias("rust").is_some());
    assert_eq!(texts(&docs[0]), vec!["fn a() {}\n\n"]);
    assert_eq!(kinds(&docs[1]), vec![BlockKind::Code { language: None }]);
    assert_eq!(kinds(&docs[2]), vec![BlockKind::Code { language: None }]);
    assert_eq!(texts(&docs[2]), vec!["indented\n"]);
}

#[test]
fn should_close_an_unclosed_fence_at_the_end() {
    // given
    let source = "```\nx\ny";

    // when
    let doc = parse(source);

    // then
    assert_eq!(kinds(&doc), vec![BlockKind::Code { language: None }]);
    assert_eq!(texts(&doc), vec!["x\ny"]);
    assert_eq!(doc.blocks[0].lines, 0..3);
}

#[test]
fn should_show_html_and_images_as_text() {
    // given
    let source = "<div>hi</div>\n\n![logo](a.png)\n";

    // when
    let doc = parse(source);

    // then
    assert_eq!(kinds(&doc), vec![BlockKind::Html, BlockKind::Paragraph]);
    assert_eq!(texts(&doc), vec!["<div>hi</div>", "[image: logo](a.png)"]);
}

#[test]
fn should_join_pieces_with_line_breaks() {
    // given
    let source = "| App | Stack |\n|:---|---:|\n| ive | Rust |\n";

    // when
    let doc = parse(source);

    // then
    assert_eq!(doc.text, "App\nStack\nive\nRust");
    assert_eq!(doc.piece_starts(), vec![0, 4, 10, 14]);
}

#[test]
fn should_find_the_block_at_a_source_line() {
    // given
    let doc = parse("# A\n\npara\n\n- x\n");

    // when
    let blocks: Vec<usize> = [0, 1, 2, 4, 99].into_iter().map(|line| doc.block_at_line(line)).collect();

    // then
    assert_eq!(blocks, vec![0, 0, 1, 2, 2]);
}

#[test]
fn should_strike_through_and_draw_a_rule() {
    // given
    let source = "~~old~~ new\n\n---\n";

    // when
    let doc = parse(source);

    // then
    assert_eq!(kinds(&doc), vec![BlockKind::Paragraph, BlockKind::Rule]);
    assert_eq!(looks_of(&doc, 0, "old"), vec![&Look::Strike]);
    assert_eq!(texts(&doc), vec!["old new", ""]);
}

#[test]
fn should_end_a_list_item_where_its_text_ends() {
    // given
    let source = "- a\n  - b\n  - c\n";

    // when
    let doc = parse(source);

    // then
    let lines: Vec<_> = doc.blocks.iter().map(|block| block.lines.clone()).collect();
    assert_eq!(lines, vec![0..1, 1..2, 2..3]);
}
