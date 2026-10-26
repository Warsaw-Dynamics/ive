//! A Markdown text as the blocks Rendered draws: pieces of visible text with styled runs, each block knowing the source
//! lines it came from.

use std::ops::Range;

use gpui::SharedString;

use pulldown_cmark::{Alignment, CodeBlockKind, Event, Options, Parser, Tag, TagEnd};

use crate::colors::ColorKey;
use crate::syntax::Language;

/// A parsed Markdown text. `text` is every piece's visible text, the pieces joined by line breaks, so a search never
/// runs from one piece into the next; ranges in pieces and runs are bytes of it.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Doc {
    pub text: String,
    pub blocks: Vec<Block>,
}

/// One drawn block, top to bottom.
#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub kind: BlockKind,
    pub nesting: Nesting,
    /// the source lines it came from, from 0, the end excluded
    pub lines: Range<usize>,
    /// one piece, or a table's cells row by row (the header first)
    pub pieces: Vec<Piece>,
}

/// How deep a block sits in block quotes and lists.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Nesting {
    pub quotes: u8,
    pub lists: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockKind {
    Heading(u8),
    Paragraph,
    ListItem(Marker),
    Code { language: Option<Language> },
    /// a horizontal rule; its one piece is empty
    Rule,
    Table { columns: Vec<Align>, header_cells: usize },
    /// raw HTML, shown as its source
    Html,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Marker {
    Bullet,
    Number(u64),
    Task(Tick),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tick {
    Open,
    Done,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Piece {
    pub text: Range<usize>,
    pub runs: Vec<Run>,
}

/// A styled part of a piece.
#[derive(Debug, Clone, PartialEq)]
pub struct Run {
    pub bytes: Range<usize>,
    pub look: Look,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Look {
    Strong,
    Emphasis,
    StrongEmphasis,
    Strike,
    Code,
    /// a link to its target
    Link(SharedString),
    /// a code block's colour
    Syntax(ColorKey),
}

/// `source` as blocks of visible text.
pub fn parse(source: &str) -> Doc {
    let options = Options::ENABLE_TABLES | Options::ENABLE_TASKLISTS | Options::ENABLE_STRIKETHROUGH;
    let mut builder = Builder::new(source);
    for (event, bytes) in Parser::new_ext(source, options).into_offset_iter() {
        builder.read(event, bytes);
    }
    builder.finish()
}

impl Doc {
    /// The block holding source line `line` (from 0): the last one starting at or before it; 0 for no blocks.
    pub fn block_at_line(&self, line: usize) -> usize {
        self.blocks.partition_point(|block| block.lines.start <= line).saturating_sub(1)
    }

    /// Where each piece starts in `text`, in order.
    pub fn piece_starts(&self) -> Vec<usize> {
        self.blocks.iter().flat_map(|block| block.pieces.iter()).map(|piece| piece.text.start).collect()
    }
}

/// The inline styles open at a point of the text.
#[derive(Default)]
struct Inline {
    strong: u8,
    emphasis: u8,
    strike: u8,
    links: Vec<SharedString>,
}

impl Inline {
    /// The look of text here; one per run, so a link wins over strike, which wins over bold and italic.
    fn look(&self) -> Option<Look> {
        if let Some(target) = self.links.last() {
            return Some(Look::Link(target.clone()));
        }
        if self.strike > 0 {
            return Some(Look::Strike);
        }
        match (self.strong > 0, self.emphasis > 0) {
            (true, true) => Some(Look::StrongEmphasis),
            (true, false) => Some(Look::Strong),
            (false, true) => Some(Look::Emphasis),
            (false, false) => None,
        }
    }
}

/// A block being filled.
struct Open {
    kind: BlockKind,
    nesting: Nesting,
    start: usize,
    pieces: Vec<Piece>,
}

/// A table's columns and where its current row is.
struct TableRow {
    columns: usize,
    cells: usize,
}

/// Turns the parser's events into a [`Doc`]; `at` is the source bytes of the event being read.
struct Builder<'a> {
    source: &'a str,
    line_starts: Vec<usize>,
    at: Range<usize>,
    /// where the last event that was not a start ended
    reached: usize,
    doc: Doc,
    pieces: usize,
    open: Option<Open>,
    piece: Option<Piece>,
    inline: Inline,
    quotes: u8,
    /// each open list's next number; `None` for a bullet list
    lists: Vec<Option<u64>>,
    /// a list item whose first block has not opened yet: its marker and where it starts
    item: Option<(Marker, usize)>,
    row: Option<TableRow>,
}

impl<'a> Builder<'a> {
    fn new(source: &'a str) -> Self {
        let line_starts = std::iter::once(0).chain(source.match_indices('\n').map(|(at, _)| at + 1)).collect();
        Self {
            source,
            line_starts,
            at: 0..0,
            reached: 0,
            doc: Doc::default(),
            pieces: 0,
            open: None,
            piece: None,
            inline: Inline::default(),
            quotes: 0,
            lists: Vec::new(),
            item: None,
            row: None,
        }
    }

    /// An event over source `bytes`. A block ends where the last event in it ended; a container's start or end (a
    /// nested list's, an item's with the blank line after it) does not reach into the block it closes.
    fn read(&mut self, event: Event, bytes: Range<usize>) {
        let container = matches!(
            event,
            Event::Start(_) | Event::End(TagEnd::Item | TagEnd::List(_) | TagEnd::BlockQuote(_) | TagEnd::FootnoteDefinition)
        );
        if !container {
            self.reached = bytes.end;
        }
        self.at = bytes;
        self.event(event);
    }

    fn event(&mut self, event: Event) {
        match event {
            Event::Start(tag) => self.start(tag),
            Event::End(tag) => self.end(tag),
            Event::Text(text) => self.text(&text),
            Event::Code(text) => self.styled(&text, Look::Code),
            Event::InlineHtml(text) | Event::InlineMath(text) | Event::DisplayMath(text) => self.text(&text),
            Event::Html(text) => self.raw(&text),
            Event::FootnoteReference(label) => self.text(&format!("[^{label}]")),
            Event::SoftBreak => self.text(" "),
            Event::HardBreak => self.text("\n"),
            Event::Rule => self.rule(),
            Event::TaskListMarker(done) => self.task(if done { Tick::Done } else { Tick::Open }),
        }
    }

    fn start(&mut self, tag: Tag) {
        match tag {
            Tag::Paragraph => self.open_text_block(),
            Tag::Heading { level, .. } => self.open_block(BlockKind::Heading(level as u8)),
            Tag::BlockQuote(_) => self.enter_quote(),
            Tag::CodeBlock(kind) => self.open_block(BlockKind::Code { language: language_of(&kind) }),
            Tag::HtmlBlock => self.open_block(BlockKind::Html),
            Tag::List(first) => self.enter_list(first),
            Tag::Item => self.enter_item(),
            Tag::Table(columns) => self.open_table(&columns),
            Tag::TableHead | Tag::TableRow => self.start_row(),
            Tag::TableCell => self.start_cell(),
            Tag::Emphasis => self.inline.emphasis += 1,
            Tag::Strong => self.inline.strong += 1,
            Tag::Strikethrough => self.inline.strike += 1,
            Tag::Link { dest_url, .. } => self.inline.links.push(SharedString::from(dest_url.to_string())),
            Tag::Image { .. } => self.text("[image: "),
            Tag::FootnoteDefinition(_)
            | Tag::DefinitionList
            | Tag::DefinitionListTitle
            | Tag::DefinitionListDefinition
            | Tag::Superscript
            | Tag::Subscript
            | Tag::MetadataBlock(_) => {}
        }
    }

    fn end(&mut self, tag: TagEnd) {
        match tag {
            TagEnd::Paragraph | TagEnd::Heading(_) | TagEnd::CodeBlock | TagEnd::Table => self.close_block(),
            TagEnd::HtmlBlock => self.close_html(),
            TagEnd::BlockQuote(_) => self.leave_quote(),
            TagEnd::List(_) => self.leave_list(),
            TagEnd::Item => self.leave_item(),
            TagEnd::TableHead | TagEnd::TableRow => self.end_row(),
            TagEnd::TableCell => self.end_cell(),
            TagEnd::Emphasis => self.inline.emphasis -= 1,
            TagEnd::Strong => self.inline.strong -= 1,
            TagEnd::Strikethrough => self.inline.strike -= 1,
            TagEnd::Link => drop(self.inline.links.pop()),
            TagEnd::Image => self.end_image(),
            TagEnd::FootnoteDefinition
            | TagEnd::DefinitionList
            | TagEnd::DefinitionListTitle
            | TagEnd::DefinitionListDefinition
            | TagEnd::Superscript
            | TagEnd::Subscript
            | TagEnd::MetadataBlock(_) => {}
        }
    }

    fn finish(mut self) -> Doc {
        self.close_block();
        self.doc
    }

    fn line_of(&self, byte: usize) -> usize {
        self.line_starts.partition_point(|start| *start <= byte).saturating_sub(1)
    }

    fn nesting(&self) -> Nesting {
        Nesting { quotes: self.quotes, lists: self.lists.len().min(u8::MAX as usize) as u8 }
    }

    /// A paragraph, or a list item's first block.
    fn open_text_block(&mut self) {
        match self.item.take() {
            Some((marker, start)) => self.open_block_at(BlockKind::ListItem(marker), start),
            None => self.open_block(BlockKind::Paragraph),
        }
    }

    fn open_block(&mut self, kind: BlockKind) {
        let start = self.at.start;
        self.item = None;
        self.open_block_at(kind, start);
    }

    fn open_block_at(&mut self, kind: BlockKind, start: usize) {
        self.close_block();
        let table = matches!(kind, BlockKind::Table { .. });
        self.open = Some(Open { kind, nesting: self.nesting(), start, pieces: Vec::new() });
        if table {
            return;
        }
        self.begin_piece();
    }

    fn close_block(&mut self) {
        self.end_piece();
        let Some(open) = self.open.take() else {
            return;
        };
        let end = self.reached.max(open.start + 1);
        let lines = self.line_of(open.start)..self.line_of(end - 1) + 1;
        self.doc.blocks.push(Block { kind: open.kind, nesting: open.nesting, lines, pieces: open.pieces });
    }

    /// Raw HTML is its source, without the line break that ends it.
    fn close_html(&mut self) {
        if let Some(piece) = &mut self.piece {
            let kept = self.doc.text[piece.text.start..].trim_end_matches('\n').len();
            self.doc.text.truncate(piece.text.start + kept);
        }
        self.close_block();
    }

    fn begin_piece(&mut self) {
        if self.pieces > 0 {
            self.doc.text.push('\n');
        }
        self.pieces += 1;
        let start = self.doc.text.len();
        self.piece = Some(Piece { text: start..start, runs: Vec::new() });
    }

    fn end_piece(&mut self) {
        let Some(mut piece) = self.piece.take() else {
            return;
        };
        piece.text.end = self.doc.text.len();
        if let Some(open) = &mut self.open {
            open.pieces.push(piece);
        }
    }

    fn text(&mut self, text: &str) {
        match self.inline.look() {
            Some(look) => self.styled(text, look),
            None => self.raw(text),
        }
    }

    fn styled(&mut self, text: &str, look: Look) {
        let start = self.doc.text.len();
        self.raw(text);
        let end = self.doc.text.len();
        let Some(piece) = &mut self.piece else {
            return;
        };
        match piece.runs.last_mut() {
            Some(last) if last.bytes.end == start && last.look == look => last.bytes.end = end,
            _ => piece.runs.push(Run { bytes: start..end, look }),
        }
    }

    /// Text into the current piece; text outside any block (a tight list item's) opens one, text in a table cell past
    /// the header's columns is dropped.
    fn raw(&mut self, text: &str) {
        if self.row.as_ref().is_some_and(|row| row.cells > row.columns) {
            return;
        }
        if self.piece.is_none() {
            self.open_text_block();
        }
        self.doc.text.push_str(text);
    }

    fn rule(&mut self) {
        self.open_block(BlockKind::Rule);
        self.close_block();
    }

    /// A task item's box: the marker of the item open or about to open.
    fn task(&mut self, tick: Tick) {
        if let Some((marker, _)) = &mut self.item {
            *marker = Marker::Task(tick);
            return;
        }
        if let Some(Open { kind: BlockKind::ListItem(marker), .. }) = &mut self.open {
            *marker = Marker::Task(tick);
        }
    }

    fn enter_quote(&mut self) {
        self.close_block();
        self.quotes += 1;
    }

    fn leave_quote(&mut self) {
        self.close_block();
        self.quotes = self.quotes.saturating_sub(1);
    }

    fn enter_list(&mut self, first: Option<u64>) {
        self.close_block();
        self.lists.push(first);
    }

    fn leave_list(&mut self) {
        self.close_block();
        self.lists.pop();
    }

    fn enter_item(&mut self) {
        self.close_block();
        let marker = match self.lists.last_mut() {
            Some(Some(next)) => {
                let number = *next;
                *next += 1;
                Marker::Number(number)
            }
            _ => Marker::Bullet,
        };
        self.item = Some((marker, self.at.start));
    }

    fn leave_item(&mut self) {
        self.close_block();
        self.item = None;
    }

    fn open_table(&mut self, columns: &[Alignment]) {
        let aligns: Vec<Align> = columns.iter().map(|align| align_of(*align)).collect();
        let header_cells = aligns.len();
        self.open_block(BlockKind::Table { columns: aligns, header_cells });
    }

    fn start_row(&mut self) {
        let columns = match &self.open {
            Some(Open { kind: BlockKind::Table { columns, .. }, .. }) => columns.len(),
            _ => 0,
        };
        self.row = Some(TableRow { columns, cells: 0 });
    }

    /// A short row is filled with empty cells.
    fn end_row(&mut self) {
        let Some(row) = self.row.take() else {
            return;
        };
        for _ in row.cells.min(row.columns)..row.columns {
            self.begin_piece();
            self.end_piece();
        }
    }

    fn start_cell(&mut self) {
        let Some(row) = &mut self.row else {
            return;
        };
        row.cells += 1;
        if row.cells <= row.columns {
            self.begin_piece();
        }
    }

    fn end_cell(&mut self) {
        if self.row.as_ref().is_some_and(|row| row.cells <= row.columns) {
            self.end_piece();
        }
    }

    /// The image's alt text was written after `[image: `; its target follows.
    fn end_image(&mut self) {
        let target = image_target(&self.source[self.at.clone()]);
        self.text(&format!("]({target})"));
    }
}

/// The target of `![alt](target "title")`, as written.
fn image_target(markdown: &str) -> &str {
    let after = markdown.rfind("](").map_or("", |at| &markdown[at + 2..]);
    let inner = after.strip_suffix(')').unwrap_or(after);
    inner.split_whitespace().next().unwrap_or("")
}

fn language_of(kind: &CodeBlockKind) -> Option<Language> {
    match kind {
        CodeBlockKind::Fenced(info) => info.split_whitespace().next().and_then(Language::for_alias),
        CodeBlockKind::Indented => None,
    }
}

fn align_of(align: Alignment) -> Align {
    match align {
        Alignment::None | Alignment::Left => Align::Left,
        Alignment::Center => Align::Center,
        Alignment::Right => Align::Right,
    }
}
