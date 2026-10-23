//! Generated marker syntax with independent mindmap and WBS entries.
use super::{
    BodySource, context,
    error::{ParseFailure, generated_failure},
    outline::{self, Arena, Lexer, Syntax},
    ownership::Row,
};
use crate::{
    RenderError,
    plantuml::{
        budget::Budget,
        tree::{Mindmap, MindmapBuilder, Outline, Side, Wbs, WbsBuilder},
    },
};
use std::ops::Range;
#[derive(Clone, Debug)]
pub(in crate::plantuml) enum Token {
    Prefix(String),
    Zero,
    Under,
    Left,
    Right,
    Open,
    Close,
    Alias(String),
    Space,
    Label(String),
    Quoted(String),
    As,
    Side(Side),
    Newline,
    Unknown,
}
pub(in crate::plantuml) struct Marker {
    pub prefix: String,
    pub label: String,
    pub alias: String,
    pub side: Side,
    pub outline: Outline,
}
pub(in crate::plantuml) fn marker(
    prefix: String,
    modifiers: (Outline, Direction),
    alias: String,
    label: (String, String),
) -> Marker {
    let side = match modifiers.1 {
        Direction::Left => Side::Left,
        Direction::Right => Side::Right,
        Direction::Default => {
            if prefix.contains('-') {
                Side::Left
            } else {
                Side::Right
            }
        }
    };
    Marker {
        prefix,
        side,
        outline: modifiers.0,
        alias: if label.1.is_empty() { alias } else { label.1 },
        label: label.0,
    }
}
pub(in crate::plantuml) enum Direction {
    Default,
    Left,
    Right,
}
pub(in crate::plantuml) trait Actions {
    fn node(&mut self, marker: Marker, span: Range<usize>) -> Result<(), ParseFailure>;
    fn set_side(&mut self, side: Side) -> Result<(), ParseFailure>;
}
impl Actions for MindmapBuilder<'_> {
    fn node(&mut self, marker: Marker, span: Range<usize>) -> Result<(), ParseFailure> {
        self.node(marker, span)
    }
    fn set_side(&mut self, side: Side) -> Result<(), ParseFailure> {
        self.set_side(side)
    }
}
impl Actions for WbsBuilder<'_> {
    fn node(&mut self, marker: Marker, span: Range<usize>) -> Result<(), ParseFailure> {
        self.node(marker, span)
    }
    fn set_side(&mut self, _: Side) -> Result<(), ParseFailure> {
        Err(ParseFailure::unsupported(0..0, "WBS side command"))
    }
}
pub(in crate::plantuml) fn alias(
    value: String,
    span: Range<usize>,
) -> Result<String, ParseFailure> {
    if value.is_empty() || value.chars().any(|ch| !(ch.is_alphanumeric() || ch == '_')) {
        return Err(ParseFailure::syntax(span, "invalid WBS alias"));
    }
    Ok(value)
}
pub(in crate::plantuml) fn parse_mindmap(
    body: BodySource<'_>,
    budget: &mut Budget,
) -> Result<Mindmap, RenderError> {
    parse_mindmap_body(&body, budget).map_err(|error| error.into_document(body.document, budget))
}
fn parse_mindmap_body(body: &BodySource<'_>, budget: &mut Budget) -> Result<Mindmap, ParseFailure> {
    let tokens = outline::lex(body, budget, |row, base, lexer| {
        row_tokens(row, base, lexer, Family::Mindmap)
    })?;
    let arena = Arena::admit((&tokens, tokens.capacity()), body, budget, Syntax::Mindmap)?;
    let iterator = context::Checked::new(tokens, budget.control.clone());
    let mut builder = MindmapBuilder::new(budget, arena);
    super::generated_tree::MindmapParser::new()
        .parse(&mut builder, iterator)
        .map_err(|error| {
            generated_failure(
                error,
                body.range.end,
                "invalid or incomplete mindmap marker syntax",
            )
        })?;
    builder.finish(body.range.end..body.range.end)
}
pub(in crate::plantuml) fn parse_wbs(
    body: BodySource<'_>,
    budget: &mut Budget,
) -> Result<Wbs, RenderError> {
    parse_wbs_body(&body, budget).map_err(|error| error.into_document(body.document, budget))
}
fn parse_wbs_body(body: &BodySource<'_>, budget: &mut Budget) -> Result<Wbs, ParseFailure> {
    let tokens = outline::lex(body, budget, |row, base, lexer| {
        row_tokens(row, base, lexer, Family::Wbs)
    })?;
    let arena = Arena::admit((&tokens, tokens.capacity()), body, budget, Syntax::Wbs)?;
    let iterator = context::Checked::new(tokens, budget.control.clone());
    let mut builder = WbsBuilder::new(budget, arena);
    super::generated_tree::WbsParser::new()
        .parse(&mut builder, iterator)
        .map_err(|error| {
            generated_failure(
                error,
                body.range.end,
                "invalid or incomplete WBS marker syntax",
            )
        })?;
    builder.finish(body.range.end..body.range.end)
}
#[derive(Clone, Copy)]
enum Family {
    Mindmap,
    Wbs,
}
fn row_tokens(
    row: &Row,
    base: usize,
    lexer: &mut Lexer<'_, Token>,
    family: Family,
) -> Result<(), ParseFailure> {
    let raw = row.text.as_str();
    if raw.trim().is_empty() {
        return Ok(());
    }
    if matches!(family, Family::Mindmap) {
        let side = match raw.trim() {
            "left side" | "top side" => Some(Side::Left),
            "right side" | "bottom side" => Some(Side::Right),
            _ => None,
        };
        if let Some(side) = side {
            lexer.push(Token::Side(side), row.span(base, 0..raw.len()))?;
            return lexer.push(Token::Newline, row.span(base, raw.len()..raw.len()));
        }
    }
    let cursor = prefix(row, base, lexer, family)?;
    tail(row, (base, cursor), lexer, family)?;
    lexer.push(Token::Newline, row.span(base, raw.len()..raw.len()))
}
fn prefix(
    row: &Row,
    base: usize,
    lexer: &mut Lexer<'_, Token>,
    family: Family,
) -> Result<usize, ParseFailure> {
    let raw = row.text.as_str();
    if matches!(family, Family::Mindmap) && raw.starts_with('0') {
        lexer.push(Token::Zero, row.span(base, 0..1))?;
        return Ok(1);
    }
    let text = raw.trim_start_matches([' ', '\t']);
    let indent = raw.len() - text.len();
    let allowed = |ch: u8| match family {
        Family::Wbs => matches!(ch, b'*' | b'+' | b'-'),
        Family::Mindmap => matches!(ch, b'*' | b'#'),
    };
    let mut count = 0;
    for ch in text.bytes().take_while(|&ch| allowed(ch)) {
        lexer.budget.charge(1).map_err(ParseFailure::terminal)?;
        let _ = ch;
        count += 1;
    }
    let end = if count > 0 {
        indent + count
    } else if matches!(family, Family::Mindmap) {
        let mut count = 0;
        for _ in raw.bytes().take_while(|ch| matches!(ch, b'+' | b'-')) {
            lexer.budget.charge(1).map_err(ParseFailure::terminal)?;
            count += 1;
        }
        count
    } else {
        0
    };
    if end == 0 {
        return Err(ParseFailure::unsupported(
            row.span(base, 0..raw.len()),
            "tree command or crosslink",
        ));
    }
    let value = lexer.copy(&raw[..end])?;
    lexer.push(Token::Prefix(value), row.span(base, 0..end))?;
    Ok(end)
}
fn tail(
    row: &Row,
    location: (usize, usize),
    lexer: &mut Lexer<'_, Token>,
    family: Family,
) -> Result<(), ParseFailure> {
    let (base, mut cursor) = location;
    let raw = row.text.as_str();
    if matches!(family, Family::Mindmap) {
        if cursor == 1 && raw.starts_with('0') {
            let value = lexer.copy(raw[cursor..].trim_start())?;
            return lexer.push(Token::Label(value), row.span(base, cursor..raw.len()));
        }
        if raw[cursor..].starts_with('[') {
            return Err(ParseFailure::unsupported(
                row.span(base, cursor..raw.len()),
                "mindmap colors",
            ));
        }
        if raw[cursor..].starts_with('_') {
            lexer.push(Token::Under, row.span(base, cursor..cursor + 1))?;
            cursor += 1;
        }
        if raw[cursor..].starts_with(':') {
            return Err(ParseFailure::unsupported(
                row.span(base, cursor..raw.len()),
                "mindmap multiline command variant",
            ));
        }
        let value = lexer.copy(raw[cursor..].trim_start())?;
        return lexer.push(Token::Label(value), row.span(base, cursor..raw.len()));
    }
    while let Some(ch) = raw[cursor..]
        .chars()
        .next()
        .filter(|ch| matches!(ch, '_' | '<' | '>'))
    {
        let token = match ch {
            '_' => Token::Under,
            '<' => Token::Left,
            _ => Token::Right,
        };
        lexer.push(token, row.span(base, cursor..cursor + 1))?;
        cursor += 1;
    }
    if raw[cursor..].starts_with('(') {
        lexer.push(Token::Open, row.span(base, cursor..cursor + 1))?;
        cursor += 1;
        let count = scan(&raw[cursor..], ')', lexer.budget)?;
        let value = lexer.copy(&raw[cursor..cursor + count])?;
        lexer.push(Token::Alias(value), row.span(base, cursor..cursor + count))?;
        cursor += count;
        if raw[cursor..].starts_with(')') {
            lexer.push(Token::Close, row.span(base, cursor..cursor + 1))?;
            cursor += 1;
        }
    }
    if raw[cursor..].starts_with(['[', ':']) {
        return Err(ParseFailure::unsupported(
            row.span(base, cursor..raw.len()),
            "WBS colors or multiline command variant",
        ));
    }
    if cursor == raw.len() {
        return Ok(());
    }
    if raw[cursor..].starts_with(char::is_whitespace) {
        let next = raw.len() - raw[cursor..].trim_start().len();
        lexer.push(Token::Space, row.span(base, cursor..next))?;
        cursor = next;
    } else {
        lexer.push(Token::Unknown, row.span(base, cursor..raw.len()))?;
        return Ok(());
    }
    let label = &raw[cursor..];
    if let Some(quoted) = label.strip_prefix('"') {
        if let Some((display, code)) = quoted.rsplit_once("\" as ") {
            let value = lexer.copy(display)?;
            lexer.push(
                Token::Quoted(value),
                row.span(base, cursor..cursor + display.len() + 2),
            )?;
            lexer.push(
                Token::As,
                row.span(base, cursor + display.len() + 2..cursor + display.len() + 6),
            )?;
            let value = lexer.copy(code)?;
            lexer.push(
                Token::Alias(value),
                row.span(base, raw.len() - code.len()..raw.len()),
            )?;
            return Ok(());
        }
        let value = lexer.copy(quoted)?;
        return lexer.push(Token::Quoted(value), row.span(base, cursor..raw.len()));
    }
    let value = lexer.copy(label)?;
    lexer.push(Token::Label(value), row.span(base, cursor..raw.len()))
}
fn scan(text: &str, delimiter: char, budget: &mut Budget) -> Result<usize, ParseFailure> {
    for (index, ch) in text.char_indices() {
        budget
            .charge(ch.len_utf8())
            .map_err(ParseFailure::terminal)?;
        if ch == delimiter {
            return Ok(index);
        }
    }
    Ok(text.len())
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        RenderOptions,
        plantuml::{
            parser, preprocess,
            tree::{Parent, Side},
        },
    };
    #[test]
    fn should_generate_independent_mindmap_sides() {
        // given
        let mut budget = Budget::new(&RenderOptions::default());
        let document = preprocess::prepare("@startmindmap\n* Root\nleft side\n*** L\nright side\n*** R\nleft side\n***** Child\n@endmindmap", &mut budget).unwrap();
        let mut prepared = parser::document::parse(&document, &mut budget).unwrap();
        parser::preamble::parse(&mut prepared, &mut budget).unwrap();
        // when
        let model = parse_mindmap(prepared.body, &mut budget).unwrap();
        // then
        assert_eq!(model.nodes[3].parent, Parent::Node(1));
        assert_eq!(model.nodes[3].side, Side::Left);
    }
    #[test]
    fn should_generate_distinct_wbs_aliases_and_pseudo_nodes() {
        // given
        let mut budget = Budget::new(&RenderOptions::default());
        let document = preprocess::prepare(
            "@startwbs\n* Root\n**(id) Child\n***_\n@endwbs",
            &mut budget,
        )
        .unwrap();
        let mut prepared = parser::document::parse(&document, &mut budget).unwrap();
        parser::preamble::parse(&mut prepared, &mut budget).unwrap();
        // when
        let model = parse_wbs(prepared.body, &mut budget).unwrap();
        // then
        assert_eq!(model.nodes[1].alias, "id");
        assert_eq!(model.nodes[2].parent, Parent::Node(1));
    }

    #[test]
    fn should_reject_direct_generated_tree_openers_modifiers_and_suffixes() {
        // given
        let streams = [
            vec![
                Token::Prefix("*".into()),
                Token::Open,
                Token::Alias("id".into()),
                Token::Newline,
            ],
            vec![
                Token::Prefix("*".into()),
                Token::Under,
                Token::Under,
                Token::Newline,
            ],
            vec![
                Token::Prefix("*".into()),
                Token::Label("missing separator".into()),
                Token::Newline,
            ],
        ];
        for tokens in streams {
            let stream = tokens
                .into_iter()
                .enumerate()
                .map(|(index, token)| (index, token, index + 1))
                .collect::<Vec<_>>();
            let mut budget = Budget::new(&RenderOptions::default());
            let arena =
                Arena::for_stream((&stream, stream.capacity()), &mut budget, Syntax::Wbs).unwrap();
            let iterator = context::Checked::new(stream, budget.control.clone());
            let mut builder = WbsBuilder::new(&mut budget, arena);
            // when
            let result =
                super::super::generated_tree::WbsParser::new().parse(&mut builder, iterator);
            // then
            assert!(result.is_err());
        }
    }

    #[test]
    fn should_cancel_both_generated_tree_entries_during_delivery_and_actions() {
        for checkpoint in 1..=5 {
            // given
            let stream = vec![
                (0, Token::Prefix("*".into()), 1),
                (1, Token::Label("Root".into()), 5),
                (5, Token::Newline, 6),
            ];
            let mut budget = Budget::new(&RenderOptions::default());
            let arena =
                Arena::for_stream((&stream, stream.capacity()), &mut budget, Syntax::Mindmap)
                    .unwrap();
            budget.control.cancel_after_checkpoints(checkpoint);
            let iterator = context::Checked::new(stream, budget.control.clone());
            let mut builder = MindmapBuilder::new(&mut budget, arena);
            // when
            let result =
                super::super::generated_tree::MindmapParser::new().parse(&mut builder, iterator);
            // then
            assert!(matches!(
                result,
                Err(lalrpop_util::ParseError::User {
                    error: ParseFailure {
                        kind: super::super::error::FailureKind::Terminal(RenderError::Cancelled(_)),
                        ..
                    }
                })
            ));
            let stream = vec![
                (0, Token::Prefix("*".into()), 1),
                (1, Token::Space, 2),
                (2, Token::Label("Root".into()), 6),
                (6, Token::Newline, 7),
            ];
            let mut budget = Budget::new(&RenderOptions::default());
            let arena =
                Arena::for_stream((&stream, stream.capacity()), &mut budget, Syntax::Wbs).unwrap();
            budget.control.cancel_after_checkpoints(checkpoint);
            let iterator = context::Checked::new(stream, budget.control.clone());
            let mut builder = WbsBuilder::new(&mut budget, arena);
            assert!(matches!(
                super::super::generated_tree::WbsParser::new().parse(&mut builder, iterator),
                Err(lalrpop_util::ParseError::User {
                    error: ParseFailure {
                        kind: super::super::error::FailureKind::Terminal(RenderError::Cancelled(_)),
                        ..
                    }
                })
            ));
        }
    }

    #[test]
    fn should_preserve_zero_marker_literal_tail_and_shared_comment_checkpoint() {
        // given
        let mut budget = Budget::new(&RenderOptions::default());
        let document = preprocess::prepare(
            "\u{feff}@startmindmap\r\ntitle Ω /'comment\r\n'/0_[literal]: café\r\n@endmindmap",
            &mut budget,
        )
        .unwrap();
        let mut prepared = parser::document::parse(&document, &mut budget).unwrap();
        parser::preamble::parse(&mut prepared, &mut budget).unwrap();
        // when
        let model = parse_mindmap(prepared.body, &mut budget).unwrap();
        // then
        assert_eq!(model.nodes[0].label, "_[literal]: café");
        assert_eq!(model.nodes[0].outline, Outline::Box);
    }

    #[test]
    fn should_map_generated_alias_failure_once_to_frozen_definition_use() {
        // given
        let source = "\u{feff}@startwbs\r\n!define BAD **(bad!) Child\r\n!define UNUSED inert\r\n* Root\r\nBAD\r\n@endwbs";
        let options = RenderOptions {
            language: crate::Language::PlantUml,
            ..RenderOptions::default()
        };
        // when
        let error = crate::render(source, &options).unwrap_err().to_string();
        // then
        assert!(
            error.contains("used at line 5, column 1, defined at line 2, column 1"),
            "{error}"
        );
        assert_eq!(error.matches("expansion \"BAD\"").count(), 1, "{error}");
    }
}
