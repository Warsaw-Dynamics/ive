//! Generated preamble orchestration and lowering of the bounded MIT subset.
//! Semantic policy: Copyright 2009-2024 Arnaud Roques; notices/source traces in provenance.
use super::{context, error::ParseFailure, ownership::Row};
use crate::{
    RenderError,
    plantuml::{
        budget::{Budget, bound},
        origin::{Document, PreambleCursor},
        source::Wrapper,
        style::{self, Alignment, Preamble, PropertyKey, Region, Role, Target},
    },
};
use std::ops::Range;

#[derive(Clone, Debug)]
pub(in crate::plantuml) enum Token {
    Title,
    Legend,
    Header,
    Footer,
    Left,
    Center,
    Right,
    Skinparam,
    SkinKey((Target, PropertyKey)),
    SheetStart,
    SheetEnd,
    Root,
    Element,
    Document,
    Property(PropertyKey),
    Open,
    Close,
    Colon,
    Semi,
    Value(String),
    Unknown,
    Newline,
    Body,
}
pub(in crate::plantuml) type Spanned = (usize, Token, usize);

#[derive(Clone, Copy, PartialEq, Eq)]
enum TailKind {
    Region,
    Property,
}

#[derive(Default)]
pub(in crate::plantuml) struct Lexer {
    pub tokens: Vec<Spanned>,
    finished: bool,
    sheet: bool,
    literal_bytes: usize,
}

impl Lexer {
    pub fn finished(&self) -> bool {
        self.finished
    }
    pub fn retained_bytes(&self) -> usize {
        self.tokens
            .capacity()
            .saturating_mul(std::mem::size_of::<Spanned>())
            .saturating_add(self.literal_bytes)
    }
    pub fn row(
        &mut self,
        row: &Row,
        location: (usize, Wrapper, super::header::Evidence),
        budget: &mut Budget,
    ) -> Result<(), ParseFailure> {
        if self.finished {
            return Ok(());
        }
        let (base, wrapper, header) = location;
        let text = row.text.trim();
        if text.is_empty() || (wrapper == Wrapper::Hcl && text.starts_with('#')) {
            return Ok(());
        }
        let keyed =
            wrapper == Wrapper::Yaml && super::super::yaml::find_unquoted(text, ':').is_some();
        let marker = has_marker(text);
        let family = if self.sheet {
            false
        } else if matches!(wrapper, Wrapper::Ebnf | Wrapper::Regex | Wrapper::Hcl)
            || keyed
            || !marker
            || (wrapper == Wrapper::Chart && text.split_whitespace().next() == Some("legend"))
        {
            true
        } else if wrapper == Wrapper::Uml {
            header.identifier_arrow
        } else {
            super::header::parse(text, base, budget)?.identifier_arrow
        };
        if family {
            self.finished = true;
            let span = base..row.span(base, 0..row.text.len()).end;
            return self.push(Token::Body, span, budget);
        }
        // JSON/YAML only own the exact lowercase "title " prefix as chrome.
        if !self.sheet
            && matches!(wrapper, Wrapper::Json | Wrapper::Yaml)
            && matches!(primitive(text).0, Token::Title)
            && !text.starts_with("title ")
        {
            self.finished = true;
            return self.push(
                Token::Body,
                base..row.span(base, 0..row.text.len()).end,
                budget,
            );
        }
        self.scan(row, base, budget)
    }

    fn scan(&mut self, row: &Row, base: usize, budget: &mut Budget) -> Result<(), ParseFailure> {
        let mut cursor = 0;
        while cursor < row.text.len() {
            let rest = &row.text[cursor..];
            let ch = rest.chars().next().unwrap();
            if ch.is_whitespace() {
                budget
                    .charge(ch.len_utf8())
                    .map_err(ParseFailure::terminal)?;
                cursor += ch.len_utf8();
                continue;
            }
            let (token, bytes) = primitive(rest);
            let tail = match token {
                Token::Property(_) | Token::SkinKey(_) => Some(TailKind::Property),
                Token::Title | Token::Legend | Token::Header | Token::Footer if !self.sheet => {
                    Some(TailKind::Region)
                }
                _ => None,
            };
            self.sheet = match token {
                Token::SheetStart => true,
                Token::SheetEnd => false,
                _ => self.sheet,
            };
            let span = row.span(base, cursor..cursor + bytes);
            self.push(token, span, budget)?;
            budget.charge(bytes).map_err(ParseFailure::terminal)?;
            cursor += bytes;
            if let Some(kind) = tail {
                self.tail(row, (base, cursor, kind), budget)?;
                break;
            }
        }
        self.push(
            Token::Newline,
            row.span(base, row.text.len()..row.text.len()),
            budget,
        )
    }

    fn tail(
        &mut self,
        row: &Row,
        location: (usize, usize, TailKind),
        budget: &mut Budget,
    ) -> Result<(), ParseFailure> {
        let (base, cursor, kind) = location;
        let rest = &row.text[cursor..];
        for chunk in rest.as_bytes().chunks(256) {
            budget.charge(chunk.len()).map_err(ParseFailure::terminal)?;
        }
        let mut start = cursor + rest.len() - rest.trim_start().len();
        let mut end = row.text.trim_end().len().max(start);
        if kind == TailKind::Region && row.text[start..end].starts_with(':') {
            self.push(Token::Colon, row.span(base, start..start + 1), budget)?;
            start += 1;
            let rest = &row.text[start..end];
            start += rest.len() - rest.trim_start().len();
        }
        let semi = self.sheet && row.text[start..end].ends_with(';');
        if semi {
            end -= 1;
            end = row.text[..end].trim_end().len().max(start);
        }
        let value = &row.text[start..end];
        self.literal_bytes = self.literal_bytes.saturating_add(value.len());
        bound(
            "preamble literal bytes",
            self.literal_bytes,
            budget.limits.source_bytes,
        )
        .map_err(ParseFailure::terminal)?;
        // Literal copies are temporary syntax, not semantic model items/text.
        let span = row.span(base, start..end);
        self.push(Token::Value(value.to_owned()), span, budget)?;
        if semi {
            self.push(
                Token::Semi,
                row.span(
                    base,
                    row.text.trim_end().len() - 1..row.text.trim_end().len(),
                ),
                budget,
            )?;
        }
        Ok(())
    }

    fn push(
        &mut self,
        token: Token,
        span: Range<usize>,
        budget: &mut Budget,
    ) -> Result<(), ParseFailure> {
        context::push(&mut self.tokens, (span.start, token, span.end), budget)
    }
}

/// Lexical command ownership only; complete forms are checked by the generated grammar.
pub(in crate::plantuml) fn has_marker(text: &str) -> bool {
    let text = text.trim();
    if text.eq_ignore_ascii_case("<style>") || text.eq_ignore_ascii_case("</style>") {
        return true;
    }
    let head = marker_word(text);
    let (first, _) = primitive(head);
    match first {
        Token::Title | Token::Legend | Token::Header | Token::Footer | Token::Skinparam => true,
        Token::Left | Token::Center | Token::Right => matches!(
            primitive(marker_word(text[head.len()..].trim_start())).0,
            Token::Header | Token::Footer
        ),
        _ => false,
    }
}

fn marker_word(text: &str) -> &str {
    for (index, ch) in text.char_indices().take(12) {
        if ch.is_whitespace() || ch == ':' {
            return &text[..index];
        }
        if index >= 10 {
            return "";
        }
    }
    text
}

fn primitive(rest: &str) -> (Token, usize) {
    if rest.is_empty() {
        return (Token::Unknown, 0);
    }
    for (spelling, token) in [
        ("<style>", Token::SheetStart),
        ("</style>", Token::SheetEnd),
    ] {
        if rest
            .get(..spelling.len())
            .is_some_and(|s| s.eq_ignore_ascii_case(spelling))
        {
            return (token, spelling.len());
        }
    }
    let ch = rest.chars().next().unwrap();
    let punctuation = match ch {
        '{' => Some(Token::Open),
        '}' => Some(Token::Close),
        ':' => Some(Token::Colon),
        ';' => Some(Token::Semi),
        _ => None,
    };
    if let Some(token) = punctuation {
        return (token, ch.len_utf8());
    }
    let bytes = rest
        .char_indices()
        .take(32)
        .find(|(_, ch)| ch.is_whitespace() || matches!(ch, ':' | '{' | '}' | ';'))
        .map_or_else(|| rest.len().min(32), |(index, _)| index);
    let Some(word) = rest.get(..bytes) else {
        return (Token::Unknown, ch.len_utf8());
    };
    let token = [
        ("title", Token::Title),
        ("legend", Token::Legend),
        ("header", Token::Header),
        ("footer", Token::Footer),
        ("left", Token::Left),
        ("center", Token::Center),
        ("right", Token::Right),
        ("skinparam", Token::Skinparam),
        ("root", Token::Root),
        ("element", Token::Element),
        ("document", Token::Document),
        ("fontname", Token::Property(PropertyKey::FontName)),
        ("fontcolor", Token::Property(PropertyKey::FontColor)),
        ("fontsize", Token::Property(PropertyKey::FontSize)),
        ("fontstyle", Token::Property(PropertyKey::FontStyle)),
        (
            "backgroundcolor",
            Token::Property(PropertyKey::BackgroundColor),
        ),
    ]
    .into_iter()
    .find(|(name, _)| word.eq_ignore_ascii_case(name))
    .map(|(_, token)| token)
    .unwrap_or_else(|| skin_key(word).map_or(Token::Unknown, Token::SkinKey));
    if matches!(token, Token::Unknown) {
        return (token, ch.len_utf8());
    }
    (token, bytes)
}

fn skin_key(word: &str) -> Option<(Target, PropertyKey)> {
    for (name, key) in [
        ("defaultfontname", PropertyKey::FontName),
        ("defaultfontcolor", PropertyKey::FontColor),
        ("defaultfontsize", PropertyKey::FontSize),
    ] {
        if word.eq_ignore_ascii_case(name) {
            let target = if key == PropertyKey::FontSize {
                Target::Element
            } else {
                Target::Root
            };
            return Some((target, key));
        }
    }
    for role in [Role::Title, Role::Legend, Role::Header, Role::Footer] {
        if word
            .get(..role.name().len())
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case(role.name()))
        {
            let suffix = &word[role.name().len()..];
            for (name, key) in [
                ("fontname", PropertyKey::FontName),
                ("fontcolor", PropertyKey::FontColor),
                ("fontsize", PropertyKey::FontSize),
                ("fontstyle", PropertyKey::FontStyle),
            ] {
                if suffix.eq_ignore_ascii_case(name) {
                    return Some((Target::Region(role), key));
                }
            }
        }
    }
    None
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Syntax {
    Empty,
    Skinparam,
    Sheet,
}

pub(in crate::plantuml) struct Builder<'a> {
    pub budget: &'a mut Budget,
    origins: PreambleCursor<'a>,
    wrapper: Wrapper,
    pub preamble: Preamble,
    target: Target,
    syntax: Syntax,
    declarations: usize,
}

impl<'a> Builder<'a> {
    fn new(document: &'a Document, wrapper: Wrapper, budget: &'a mut Budget) -> Self {
        Self {
            budget,
            origins: PreambleCursor::new(document),
            wrapper,
            preamble: Preamble::default(),
            target: Target::Root,
            syntax: Syntax::Empty,
            declarations: 0,
        }
    }

    pub fn tick(&mut self) -> Result<(), ParseFailure> {
        self.budget.charge(1).map_err(ParseFailure::terminal)
    }

    pub fn region(
        &mut self,
        head: (Role, Alignment),
        value: String,
        span: Range<usize>,
    ) -> Result<(), ParseFailure> {
        if matches!(self.wrapper, Wrapper::Json | Wrapper::Yaml) && head.0 != Role::Title {
            return Err(ParseFailure::unsupported(
                span,
                "data preamble supports only title and style",
            ));
        }
        bound("label bytes", value.len(), 16 * 1024).map_err(ParseFailure::terminal)?;
        if value.is_empty() {
            return Err(ParseFailure::unsupported(
                span,
                "multiline/empty document region",
            ));
        }
        let data = matches!(self.wrapper, Wrapper::Json | Wrapper::Yaml);
        let text = if !data && value.starts_with('"') {
            value
                .strip_prefix('"')
                .and_then(|s| s.strip_suffix('"'))
                .ok_or_else(|| {
                    ParseFailure::syntax(span.clone(), "unclosed document label quote")
                })?
        } else {
            &value
        };
        let (visible, word) = self.validate_label(text)?;
        if !data && !value.starts_with('"') && !word {
            return Err(ParseFailure::syntax(
                span,
                "unquoted document label requires a letter, number, underscore or dot",
            ));
        }
        if !visible {
            return Err(ParseFailure::syntax(span, "empty document region"));
        }
        self.budget.item().map_err(ParseFailure::terminal)?;
        self.budget
            .text(text.len())
            .map_err(ParseFailure::terminal)?;
        self.preamble.regions.retain(|old| old.role != head.0);
        self.preamble.regions.push(Region {
            role: head.0,
            align: head.1,
            text: text.into(),
        });
        Ok(())
    }

    fn validate_label(&mut self, text: &str) -> Result<(bool, bool), ParseFailure> {
        let mut pending = 0;
        let mut visible = false;
        let mut word = false;
        for ch in text.chars() {
            pending += ch.len_utf8();
            visible |= !ch.is_whitespace();
            word |= ch.is_alphanumeric() || matches!(ch, '_' | '.');
            if pending >= 256 {
                self.budget
                    .charge(pending)
                    .map_err(ParseFailure::terminal)?;
                pending = 0;
            }
        }
        self.budget
            .charge(pending)
            .map_err(ParseFailure::terminal)?;
        Ok((visible, word))
    }

    pub fn skin(
        &mut self,
        key: (Target, PropertyKey),
        value: String,
        span: Range<usize>,
    ) -> Result<(), ParseFailure> {
        if self.syntax == Syntax::Sheet || matches!(self.wrapper, Wrapper::Json | Wrapper::Yaml) {
            return Err(ParseFailure::unsupported(
                span,
                "mixed skinparam/style declarations or data skinparam",
            ));
        }
        self.syntax = Syntax::Skinparam;
        self.declare_for(key.0, key.1, value, span)
    }

    pub fn sheet(&mut self, span: Range<usize>) -> Result<(), ParseFailure> {
        self.tick()?;
        if self.syntax == Syntax::Skinparam {
            return Err(ParseFailure::unsupported(
                span,
                "mixed skinparam/style declarations",
            ));
        }
        self.syntax = Syntax::Sheet;
        Ok(())
    }

    pub fn open(&mut self, target: Target, depth: usize) -> Result<usize, ParseFailure> {
        self.budget.depth(depth).map_err(ParseFailure::terminal)?;
        self.budget.item().map_err(ParseFailure::terminal)?;
        self.target = target;
        Ok(self.declarations)
    }

    pub fn close(&mut self, count: usize, span: Range<usize>) -> Result<(), ParseFailure> {
        self.tick()?;
        if self.declarations == count {
            return Err(ParseFailure::syntax(span, "empty style rule"));
        }
        self.target = Target::Document;
        Ok(())
    }

    pub fn finish_sheet(&mut self, span: Range<usize>) -> Result<(), ParseFailure> {
        self.origins
            .deny_sheet_definitions(span, self.budget)
            .map_err(ParseFailure::terminal)
    }

    pub fn declare(
        &mut self,
        key: PropertyKey,
        value: String,
        span: Range<usize>,
    ) -> Result<(), ParseFailure> {
        self.declare_for(self.target, key, value, span)
    }

    fn declare_for(
        &mut self,
        target: Target,
        key: PropertyKey,
        value: String,
        span: Range<usize>,
    ) -> Result<(), ParseFailure> {
        self.tick()?;
        if (target == Target::Document) != (key == PropertyKey::BackgroundColor) {
            return Err(ParseFailure::unsupported(
                span,
                "property is unsupported for this style target",
            ));
        }
        let roles = if target == Target::Root {
            5
        } else if target == Target::Document {
            0
        } else {
            1
        };
        for _ in 0..roles {
            self.budget
                .charge(value.len() + 1)
                .map_err(ParseFailure::terminal)?;
            self.budget
                .text(value.len())
                .map_err(ParseFailure::terminal)?;
        }
        let property = style::decode_property(key, &value)
            .map_err(|detail| ParseFailure::unsupported(span, detail))?;
        self.preamble
            .apply(target, property, self.budget)
            .map_err(ParseFailure::terminal)?;
        self.declarations += 1;
        Ok(())
    }
}

pub(in crate::plantuml) fn parse(
    prepared: &mut super::document::ParsedDocument<'_>,
    budget: &mut Budget,
) -> Result<(Preamble, Range<usize>), RenderError> {
    let tokens = std::mem::take(&mut prepared.preamble_tokens);
    let adapters = prepared.body.adapters;
    let (preamble, boundary) = parse_tokens(
        Input {
            tokens,
            document: prepared.body.document,
            wrapper: prepared.wrapper,
            adapters,
        },
        budget,
    )
    .map_err(|error| error.into_document(prepared.body.document, budget))?;
    prepared.body.range.start = boundary.start;
    Ok((preamble, boundary))
}

struct Input<'s> {
    tokens: Vec<Spanned>,
    document: &'s Document,
    wrapper: Wrapper,
    adapters: usize,
}

fn parse_tokens(
    input: Input<'_>,
    budget: &mut Budget,
) -> Result<(Preamble, Range<usize>), ParseFailure> {
    let Input {
        tokens,
        document,
        wrapper,
        adapters,
    } = input;
    // Admission below is renewed against this grammar's actual generated tables.
    admit(&tokens, (tokens.capacity(), adapters), budget)?;
    let eof = tokens.last().map_or(0, |token| token.2);
    let iterator = context::Checked::new(tokens, budget.control.clone());
    let mut builder = Builder::new(document, wrapper, budget);
    let body = super::generated_preamble::PreambleParser::new()
        .parse(&mut builder, iterator)
        .map_err(|error| {
            super::error::generated_failure(
                error,
                eof,
                "invalid or incomplete document preamble/style syntax",
            )
        })?;
    builder
        .origins
        .check_preamble(body.clone(), builder.budget)
        .map_err(ParseFailure::terminal)?;
    Ok((builder.preamble, body))
}

#[cfg(test)]
fn validate(
    tokens: Vec<Spanned>,
    budget: &mut Budget,
) -> Result<(Preamble, Range<usize>), ParseFailure> {
    parse_tokens(
        Input {
            tokens,
            document: &super::super::origin::EMPTY,
            wrapper: Wrapper::Uml,
            adapters: 0,
        },
        budget,
    )
}

fn admit(
    tokens: &[Spanned],
    buffers: (usize, usize),
    budget: &mut Budget,
) -> Result<(), ParseFailure> {
    let (capacity, adapters) = buffers;
    // Actual 77-state/23-terminal tables: 111 reachable shift stacks,
    // at most 12 live states and three consecutive reductions (see provenance).
    let mut literal_bytes = 0usize;
    for chunk in tokens.chunks(256) {
        budget.charge(chunk.len()).map_err(ParseFailure::terminal)?;
        for (_, token, _) in chunk {
            if let Token::Value(value) = token {
                literal_bytes = literal_bytes.saturating_add(value.capacity());
            }
        }
    }
    let symbol_bytes = std::mem::size_of::<Token>()
        + std::mem::size_of::<String>()
        + std::mem::size_of::<Range<usize>>()
        + std::mem::size_of::<(Target, PropertyKey)>()
        + std::mem::size_of::<(Role, Alignment)>()
        + 4 * std::mem::size_of::<usize>();
    let retained = capacity
        .saturating_mul(std::mem::size_of::<Spanned>())
        .saturating_add(adapters)
        .saturating_add(literal_bytes)
        .saturating_add(std::mem::size_of::<PreambleCursor<'_>>())
        // Sixteen state/symbol slots, their growth transients, copied diagnostic
        // states and all expected-token String slots/contents are reserved first.
        .saturating_add(24 * symbol_bytes + 56 + 48 * std::mem::size_of::<String>() + 23 * 16);
    bound(
        "preamble parser retained bytes",
        retained,
        budget.limits.source_bytes.saturating_mul(256),
    )
    .map_err(ParseFailure::terminal)?;
    let slots = 24 + 56 + 48;
    bound(
        "preamble parser stack slots",
        slots,
        budget.limits.layout_work,
    )
    .map_err(ParseFailure::terminal)?;
    budget
        .charge(tokens.len().saturating_mul(4).saturating_add(
            3 + slots
                + 23 * (12 + 3)
                + std::mem::size_of::<PreambleCursor<'_>>() / std::mem::size_of::<usize>(),
        ))
        .map_err(ParseFailure::terminal)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        RenderOptions,
        plantuml::style::{Alignment, Role},
    };

    #[test]
    fn should_emit_empty_literals_for_whitespace_only_value_tails() {
        // given
        for text in [
            "title   ",
            "left header\t\u{2003}",
            "skinparam defaultFontSize \t",
            "  FontSize \t\u{2003}",
        ] {
            let mut budget = Budget::new(&RenderOptions::default());
            let row = super::super::ownership::Ownership::default()
                .classify(text, &mut budget)
                .unwrap();
            let mut lexer = Lexer::default();
            // when
            lexer.scan(&row, 10, &mut budget).unwrap();
            // then
            assert!(
                lexer.tokens.iter().any(
                    |(start, token, end)| matches!(token, Token::Value(value) if value.is_empty())
                        && start == end
                ),
                "{text:?}: {:?}",
                lexer.tokens
            );
        }
    }

    #[test]
    fn should_lower_generated_region_tokens_and_preserve_the_body_span() {
        // given
        let tokens = vec![
            (10, Token::Title, 15),
            (15, Token::Colon, 16),
            (17, Token::Value("Top".into()), 20),
            (20, Token::Newline, 21),
            (21, Token::Left, 25),
            (26, Token::Header, 32),
            (33, Token::Value("Head".into()), 37),
            (37, Token::Newline, 38),
            (38, Token::Body, 49),
        ];
        let mut budget = Budget::new(&RenderOptions::default());
        // when
        let (preamble, body) = validate(tokens, &mut budget).unwrap();
        // then
        assert_eq!(body, 38..49);
        assert_eq!(preamble.regions.len(), 2);
        assert_eq!(preamble.regions[0].role, Role::Title);
        assert_eq!(preamble.regions[0].text, "Top");
        assert!(matches!(preamble.regions[1].align, Alignment::Left));
    }

    #[test]
    fn should_require_generated_whole_input_consumption_after_the_body_boundary() {
        // given
        let tokens = vec![(10, Token::Body, 20), (20, Token::Title, 25)];
        let mut budget = Budget::new(&RenderOptions::default());
        // when
        let error = validate(tokens, &mut budget).err().unwrap();
        // then
        assert_eq!(error.span, 20..25);
    }

    #[test]
    fn should_borrow_the_remaining_family_body_with_physical_first_row_whitespace() {
        // given
        let mut budget = Budget::new(&RenderOptions::default());
        let document = super::super::super::preprocess::prepare(
            "\u{feff}@startuml\r\n!define SIZE 20\r\nskinparam defaultFontSize SIZE\r\ntitle Top\r\n  A -> B : Ω\r\n@enduml\r\n", &mut budget).unwrap();
        let mut prepared = super::super::document::parse(&document, &mut budget).unwrap();
        // when
        let (preamble, _) = parse(&mut prepared, &mut budget).unwrap();
        // then
        assert_eq!(preamble.fonts[Role::Element.index()].size, 20);
        assert_eq!(&document.text[prepared.body.range], "  A -> B : Ω\r\n");
    }

    #[test]
    fn should_cancel_lexical_literal_scanning_before_allocating_the_value_token() {
        // given
        let text = format!("title {}", "Ω".repeat(4096));
        let mut budget = Budget::new(&RenderOptions::default());
        let row = super::super::ownership::Ownership::default()
            .classify(&text, &mut budget)
            .unwrap();
        let mut budget = Budget::new(&RenderOptions::default());
        budget.control.cancel_after_checkpoints(8);
        let mut lexer = Lexer::default();
        // when
        let result = lexer.scan(&row, 0, &mut budget);
        // then
        assert!(matches!(
            result,
            Err(ParseFailure {
                kind: super::super::error::FailureKind::Terminal(RenderError::Cancelled(_)),
                ..
            })
        ));
        assert!(
            lexer
                .tokens
                .iter()
                .all(|(_, token, _)| !matches!(token, Token::Value(_)))
        );
        assert!(budget.work > 0 && budget.work < text.len());
    }

    #[test]
    fn should_interrupt_region_validation_before_copying_a_long_label() {
        // given
        let text = format!("{}X", ";".repeat(4096));
        let mut budget = Budget::new(&RenderOptions::default());
        budget.control.cancel_after_checkpoints(2);
        let mut builder = Builder::new(
            &super::super::super::origin::EMPTY,
            Wrapper::Uml,
            &mut budget,
        );
        // when
        let result = builder.region((Role::Title, Alignment::Center), text, 0..4097);
        // then
        assert!(matches!(
            result,
            Err(ParseFailure {
                kind: super::super::error::FailureKind::Terminal(RenderError::Cancelled(_)),
                ..
            })
        ));
        assert!(builder.preamble.regions.is_empty());
        assert!(builder.budget.work < 4097);
    }

    fn style_tokens() -> Vec<Spanned> {
        vec![
            Token::SheetStart,
            Token::Newline,
            Token::Root,
            Token::Open,
            Token::Newline,
            Token::Property(PropertyKey::FontSize),
            Token::Value("20".into()),
            Token::Semi,
            Token::Newline,
            Token::Close,
            Token::Newline,
            Token::SheetEnd,
            Token::Newline,
            Token::Body,
        ]
        .into_iter()
        .enumerate()
        .map(|(index, token)| (index, token, index + 1))
        .collect()
    }

    #[test]
    fn should_lower_generated_style_rules_without_spending_syntax_model_items() {
        // given
        let mut options = RenderOptions::default();
        options.limits.model_items = 2;
        let mut budget = Budget::new(&options);
        // when
        let (preamble, body) = validate(style_tokens(), &mut budget).unwrap();
        // then
        assert_eq!(body, 13..14);
        assert!(preamble.fonts.iter().all(|font| font.size == 20));
        // given
        options.limits.model_items = 1;
        let mut budget = Budget::new(&options);
        // when
        let result = validate(style_tokens(), &mut budget);
        // then
        assert!(matches!(
            result,
            Err(ParseFailure {
                kind: super::super::error::FailureKind::Terminal(RenderError::Backend(_)),
                ..
            })
        ));
    }

    #[test]
    fn should_admit_actual_parser_storage_before_entry_and_cancel_generated_delivery_and_reductions()
     {
        // given
        let mut options = RenderOptions::default();
        options.limits.layout_work = 20;
        let mut budget = Budget::new(&options);
        // when
        let error = validate(style_tokens(), &mut budget).err().unwrap();
        // then
        assert!(matches!(
            error.kind,
            super::super::error::FailureKind::Terminal(RenderError::Backend(_))
        ));
        assert!(budget.work <= 20);
        for checkpoints in [0, 4, 7, 11, 15, 19, 23] {
            // given
            let mut budget = Budget::new(&RenderOptions::default());
            let tokens = style_tokens();
            admit(&tokens, (tokens.capacity(), 0), &mut budget).unwrap();
            budget.control.cancel_after_checkpoints(checkpoints);
            let iterator = context::Checked::new(tokens, budget.control.clone());
            let mut builder = Builder::new(
                &super::super::super::origin::EMPTY,
                Wrapper::Uml,
                &mut budget,
            );
            // when
            let error = super::super::generated_preamble::PreambleParser::new()
                .parse(&mut builder, iterator)
                .unwrap_err();
            // then
            assert!(
                matches!(
                    error,
                    lalrpop_util::ParseError::User {
                        error: ParseFailure {
                            kind: super::super::error::FailureKind::Terminal(
                                RenderError::Cancelled(_)
                            ),
                            ..
                        }
                    }
                ),
                "checkpoint {checkpoints}: {error:?}"
            );
        }
    }
}
