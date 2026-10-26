//! Generated ordered literal HCL block syntax.
use super::{
    BodySource, context,
    error::{FailureKind, ParseFailure, generated_failure},
    structured::{Arena, Cursor, DepthToken, Syntax},
};
use crate::{
    RenderError,
    plantuml::{
        budget::{Budget, bound},
        hcl::{Block, Document},
    },
};
use std::ops::Range;

#[derive(Clone, Debug)]
pub(in crate::plantuml) enum Token<'s> {
    Object,
    EndObject,
    Array,
    EndArray,
    Equals,
    Colon,
    Comma,
    Bare(&'s str),
    Quoted(&'s str),
    Newline,
}

impl DepthToken for Token<'_> {
    fn depth_delta(&self) -> isize {
        match self {
            Self::Object | Self::Array => 1,
            Self::EndObject | Self::EndArray => -1,
            _ => 0,
        }
    }
}

pub(in crate::plantuml) fn parse(
    body: BodySource<'_>,
    budget: &mut Budget,
) -> Result<Document, RenderError> {
    parse_body(&body, budget).map_err(|error| error.into_document(body.document, budget))
}

fn parse_body(body: &BodySource<'_>, budget: &mut Budget) -> Result<Document, ParseFailure> {
    let tokens = lex(body, budget)?;
    let arena = Arena::admit((&tokens, tokens.capacity()), body, budget, Syntax::Hcl)?;
    let iterator = context::Checked::new(tokens, budget.control.clone());
    let mut builder = Builder {
        input: &body.document.text,
        budget,
        arena,
        header_span: 0..0,
    };
    super::generated_hcl::HclParser::new()
        .parse(&mut builder, iterator)
        .map_err(|error| {
            let failure = generated_failure(
                error,
                body.range.end,
                "invalid or incomplete HCL block, field or literal array",
            );
            builder.failure(failure)
        })
}

fn lex<'s>(
    body: &BodySource<'s>,
    budget: &mut Budget,
) -> Result<Vec<(usize, Token<'s>, usize)>, ParseFailure> {
    budget
        .text(body.range.len())
        .map_err(ParseFailure::terminal)?;
    let mut cursor = Cursor::new(body, budget);
    tokens(&mut cursor)
}

fn tokens<'s>(cursor: &mut Cursor<'s, '_>) -> Result<Vec<(usize, Token<'s>, usize)>, ParseFailure> {
    let mut tokens = Vec::new();
    let mut nesting = Vec::new();
    let mut content = false;
    let mut header = false;
    let mut previous = ' ';
    while let Some(ch) = cursor.peek() {
        let start = cursor.offset;
        if ch.is_whitespace() {
            cursor.take()?;
            if ch == '\n' {
                content = false;
                if nesting.is_empty() {
                    context::push(
                        &mut tokens,
                        (start, Token::Newline, cursor.offset),
                        cursor.budget,
                    )?;
                }
            }
            continue;
        }
        if ch == '#' && !content {
            while cursor.peek().is_some_and(|ch| ch != '\n') {
                cursor.take()?;
            }
            continue;
        }
        content = true;
        let token = match ch {
            '{' | '[' => {
                cursor
                    .budget
                    .depth(nesting.len() + 1)
                    .map_err(ParseFailure::terminal)?;
                cursor.budget.item().map_err(ParseFailure::terminal)?;
                if nesting.last() == Some(&'[') && ch == '[' {
                    return Err(ParseFailure::unsupported(
                        start..start + 1,
                        "HCL arrays admit quoted strings and objects only",
                    ));
                }
                context::push(&mut nesting, ch, cursor.budget)?;
                cursor.take()?;
                if ch == '{' {
                    Token::Object
                } else {
                    Token::Array
                }
            }
            '}' | ']' => {
                cursor.take()?;
                nesting.pop();
                if nesting.is_empty() {
                    header = false;
                }
                if ch == '}' {
                    Token::EndObject
                } else {
                    Token::EndArray
                }
            }
            '=' => {
                cursor.take()?;
                Token::Equals
            }
            ':' => {
                cursor.take()?;
                Token::Colon
            }
            ',' => {
                cursor.take()?;
                Token::Comma
            }
            _ => {
                if nesting.is_empty() {
                    if !header {
                        cursor.budget.item().map_err(ParseFailure::terminal)?;
                        header = true;
                    }
                } else {
                    if matches!(previous, '=' | ':') || nesting.last() == Some(&'[') {
                        cursor
                            .budget
                            .depth(nesting.len() + 1)
                            .map_err(ParseFailure::terminal)?;
                    }
                    cursor.budget.item().map_err(ParseFailure::terminal)?;
                }
                if nesting.last() == Some(&'[') && ch != '"' {
                    return Err(ParseFailure::unsupported(
                        start..start + ch.len_utf8(),
                        "HCL arrays admit quoted strings and objects only",
                    ));
                }
                atom(cursor)?
            }
        };
        context::push(&mut tokens, (start, token, cursor.offset), cursor.budget)?;
        previous = ch;
    }
    Ok(tokens)
}

fn atom<'s>(cursor: &mut Cursor<'s, '_>) -> Result<Token<'s>, ParseFailure> {
    let start = cursor.offset;
    if cursor.peek() == Some('"') {
        cursor.take()?;
        let mut decoded = 0usize;
        loop {
            let ch = cursor.take()?;
            if ch == '"' {
                return Ok(Token::Quoted(&cursor.input[start + 1..cursor.offset - 1]));
            }
            let ch = if ch == '\\' { cursor.take()? } else { ch };
            if matches!(ch, '\n' | '\r') {
                return Err(ParseFailure::syntax(
                    start..cursor.offset,
                    "physical newline in HCL quoted string or escape",
                ));
            }
            decoded = decoded.saturating_add(ch.len_utf8());
            bound("label bytes", decoded, 16 * 1024).map_err(ParseFailure::terminal)?;
        }
    }
    while let Some(ch) = cursor.peek() {
        if ch.is_whitespace() || "{}[]=:,".contains(ch) {
            break;
        }
        if ch.is_alphanumeric() || "_.-".contains(ch) {
            bound(
                "label bytes",
                cursor.offset - start + ch.len_utf8(),
                16 * 1024,
            )
            .map_err(ParseFailure::terminal)?;
            cursor.take()?;
        } else {
            return Err(ParseFailure::unsupported(
                cursor.offset..cursor.offset + ch.len_utf8(),
                "HCL expression/call token",
            ));
        }
    }
    if cursor.offset == start {
        return Err(ParseFailure::syntax(start..start, "HCL expected string"));
    }
    Ok(Token::Bare(&cursor.input[start..cursor.offset]))
}

pub(in crate::plantuml) enum Quote {
    Bare,
    Quoted,
}
pub(in crate::plantuml) struct Word {
    pub value: String,
    pub quote: Quote,
    pub span: Range<usize>,
}
pub(in crate::plantuml) struct Header {
    pub name: String,
    pub span: Range<usize>,
}
pub(in crate::plantuml) struct Builder<'s, 'b> {
    input: &'s str,
    budget: &'b mut Budget,
    arena: Arena,
    header_span: Range<usize>,
}

impl Builder<'_, '_> {
    pub fn word(
        &mut self,
        raw: &str,
        quote: Quote,
        span: Range<usize>,
    ) -> Result<Word, ParseFailure> {
        self.budget
            .text(raw.len())
            .map_err(ParseFailure::terminal)?;
        let mut value = String::with_capacity(raw.len());
        let mut chars = raw.chars();
        while let Some(ch) = chars.next() {
            self.budget
                .charge(ch.len_utf8())
                .map_err(ParseFailure::terminal)?;
            let ch = if matches!(quote, Quote::Quoted) && ch == '\\' {
                chars
                    .next()
                    .ok_or_else(|| ParseFailure::syntax(span.clone(), "unterminated HCL escape"))?
            } else {
                ch
            };
            value.push(ch);
        }
        self.budget
            .charge(value.len().saturating_mul(2))
            .map_err(ParseFailure::terminal)?;
        if value.contains("${") || value.contains("%{") {
            return Err(ParseFailure::unsupported(span, "HCL interpolation"));
        }
        Ok(Word { value, quote, span })
    }

    pub fn header(&mut self, mut header: Header, word: Word) -> Result<Header, ParseFailure> {
        let quoted = usize::from(matches!(word.quote, Quote::Quoted));
        let extra = word.value.len() + 2 * quoted + usize::from(!header.name.is_empty());
        bound(
            "label bytes",
            header.name.len().saturating_add(extra),
            16 * 1024,
        )
        .map_err(ParseFailure::terminal)?;
        self.budget.text(extra).map_err(ParseFailure::terminal)?;
        if header.name.is_empty() {
            header.span.start = word.span.start;
        } else {
            header.name.push(' ');
        }
        if quoted == 1 {
            header.name.push('"');
        }
        header.name.push_str(&word.value);
        if quoted == 1 {
            header.name.push('"');
        }
        header.span.end = word.span.end;
        self.header_span = header.span.clone();
        Ok(header)
    }

    pub fn open(&mut self, header: Header) -> Result<Header, ParseFailure> {
        self.budget.charge(1).map_err(ParseFailure::terminal)?;
        self.header_span = 0..0;
        Ok(header)
    }

    pub fn block(
        &mut self,
        mut blocks: Vec<Block>,
        block: Block,
        span: Range<usize>,
    ) -> Result<Vec<Block>, ParseFailure> {
        for previous in &blocks {
            self.budget
                .charge(previous.name.len() + block.name.len() + 1)
                .map_err(ParseFailure::terminal)?;
            if previous.name == block.name {
                self.budget
                    .text(block.name.len().saturating_mul(4) + 128)
                    .map_err(ParseFailure::terminal)?;
                return Err(ParseFailure {
                    span,
                    kind: FailureKind::Unsupported(
                        format!("duplicate HCL top block {:?}", block.name).into(),
                    ),
                });
            }
        }
        self.arena.push(&mut blocks, block, self.budget)?;
        Ok(blocks)
    }

    pub fn push<T>(&mut self, mut values: Vec<T>, value: T) -> Result<Vec<T>, ParseFailure> {
        self.arena.push(&mut values, value, self.budget)?;
        Ok(values)
    }

    fn failure(&mut self, failure: ParseFailure) -> ParseFailure {
        if self.header_span.is_empty() || matches!(failure.kind, FailureKind::Terminal(_)) {
            return failure;
        }
        let raw = &self.input[self.header_span.clone()];
        if let Err(error) = self.budget.text(raw.len().saturating_mul(4) + 128) {
            return ParseFailure::terminal(error);
        }
        ParseFailure::owned_syntax(
            self.header_span.clone(),
            format!(
                "HCL block header {raw:?} crosses a physical line before '{{' or is incomplete"
            ),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        RenderOptions,
        plantuml::{data::Value, hcl::lower},
    };

    #[test]
    fn should_check_hcl_depth_and_items_before_scanning_scalar_content() {
        for resource in ["depth", "items"] {
            // given
            let mut options = RenderOptions::default();
            if resource == "depth" {
                options.limits.nesting_depth = 1;
            } else {
                options.limits.model_items = 3;
            }
            let mut budget = Budget::new(&options);
            let input = "a{x=\"unseen\"}";
            let mut cursor = Cursor {
                input,
                offset: 0,
                end: input.len(),
                budget: &mut budget,
            };
            // when
            let result = tokens(&mut cursor);
            // then
            assert!(result.is_err());
            assert_eq!(cursor.offset, 4);
        }
    }

    #[test]
    fn should_interrupt_hcl_literal_scanning_before_retention() {
        // given
        let input = format!("a{{x=\"{}\"}}", "x".repeat(4096));
        let mut budget = Budget::new(&RenderOptions::default());
        budget.control.cancel_after_checkpoints(32);
        let mut cursor = Cursor {
            input: &input,
            offset: 0,
            end: input.len(),
            budget: &mut budget,
        };
        // when
        let error = tokens(&mut cursor).unwrap_err();
        // then
        assert!(matches!(
            error.kind,
            FailureKind::Terminal(RenderError::Cancelled(_))
        ));
        assert!(cursor.offset > 5 && cursor.offset < 40);
    }

    #[test]
    fn should_admit_hcl_depth_64_and_stop_before_the_65th_opener() {
        for depth in [64usize, 65] {
            // given
            let input = format!("a{{{}{}", "x={".repeat(depth - 1), "}".repeat(depth));
            let lines = [input.clone()].join("\n");
            let mut budget = Budget::new(&RenderOptions::default());
            // when
            let result =
                crate::plantuml::parser::test_support::body((&lines, "hcl"), &mut budget, parse);
            // then
            assert_eq!(result.is_ok(), depth == 64);
            if depth == 65 {
                let mut budget = Budget::new(&RenderOptions::default());
                let mut cursor = Cursor {
                    input: &input,
                    offset: 0,
                    end: input.len(),
                    budget: &mut budget,
                };
                let error = tokens(&mut cursor).unwrap_err();
                assert!(format!("{error:?}").contains("nesting depth"));
                assert_eq!(cursor.offset, 2 + 3 * 63 + 2);
            }
        }
    }

    #[test]
    fn should_admit_deep_hcl_at_its_actual_source_limit() {
        // given
        let input = format!("a{{{}{}", "x={".repeat(63), "}".repeat(64));
        let source = format!("@starthcl\n{input}\n@endhcl");
        let mut options = RenderOptions::default();
        options.limits.source_bytes = source.len();
        let mut budget = Budget::new(&options);
        let document = crate::plantuml::preprocess::prepare(&source, &mut budget).unwrap();
        let mut prepared = super::super::document::parse(&document, &mut budget).unwrap();
        super::super::preamble::parse(&mut prepared, &mut budget).unwrap();
        // when
        let result = parse(prepared.body, &mut budget);
        // then
        assert!(
            result.is_ok(),
            "{}",
            result
                .err()
                .map(|error| error.to_string())
                .unwrap_or_default()
        );
    }

    #[test]
    fn should_let_generated_hcl_structure_reject_headers_fields_arrays_and_eof() {
        for stream in [
            vec![
                Token::Bare("a"),
                Token::Newline,
                Token::Bare("b"),
                Token::Object,
                Token::EndObject,
            ],
            vec![
                Token::Bare("a"),
                Token::Object,
                Token::Bare("x"),
                Token::Quoted("y"),
                Token::EndObject,
            ],
            vec![
                Token::Bare("a"),
                Token::Object,
                Token::Bare("x"),
                Token::Equals,
                Token::Array,
                Token::Bare("y"),
                Token::EndArray,
                Token::EndObject,
            ],
            vec![Token::Bare("a"), Token::Object],
        ] {
            // given
            let mut budget = Budget::new(&RenderOptions::default());
            let source = BodySource {
                document: &crate::plantuml::origin::EMPTY,
                range: 0..0,
                ownership: Default::default(),
                adapters: 0,
            };
            let stream = stream
                .into_iter()
                .map(|token| (0, token, 0))
                .collect::<Vec<_>>();
            let arena = Arena::admit(
                (&stream, stream.capacity()),
                &source,
                &mut budget,
                Syntax::Hcl,
            )
            .unwrap();
            let iterator = context::Checked::new(stream, budget.control.clone());
            let mut builder = Builder {
                input: "",
                budget: &mut budget,
                arena,
                header_span: 0..0,
            };
            // when
            let result =
                super::super::generated_hcl::HclParser::new().parse(&mut builder, iterator);
            // then
            assert!(result.is_err());
        }
    }

    #[test]
    fn should_cancel_hcl_generated_delivery_and_retaining_actions() {
        for checkpoint in 1..=8 {
            // given
            let mut budget = Budget::new(&RenderOptions::default());
            let source = BodySource {
                document: &crate::plantuml::origin::EMPTY,
                range: 0..0,
                ownership: Default::default(),
                adapters: 0,
            };
            let stream = vec![
                (0, Token::Bare("a"), 0),
                (0, Token::Object, 0),
                (0, Token::EndObject, 0),
            ];
            let arena = Arena::admit(
                (&stream, stream.capacity()),
                &source,
                &mut budget,
                Syntax::Hcl,
            )
            .unwrap();
            let iterator = context::Checked::new(stream, budget.control.clone());
            let mut builder = Builder {
                input: "",
                budget: &mut budget,
                arena,
                header_span: 0..0,
            };
            builder.budget.control.cancel_after_checkpoints(checkpoint);
            // when
            let result =
                super::super::generated_hcl::HclParser::new().parse(&mut builder, iterator);
            // then
            assert!(matches!(
                result,
                Err(lalrpop_util::ParseError::User {
                    error: ParseFailure {
                        kind: FailureKind::Terminal(RenderError::Cancelled(_)),
                        ..
                    }
                })
            ));
        }
    }

    #[test]
    fn should_generate_named_blocks_and_ordered_literal_fields() {
        // given
        let lines = ["title \"billing\" { x=3 x=true y=[\"north\",{ inside=\"yes\" },] }\nskinparam { duplicate=one duplicate=two }".to_owned()].join("\n");
        let mut budget = Budget::new(&RenderOptions::default());
        // when
        let document =
            crate::plantuml::parser::test_support::body((&lines, "hcl"), &mut budget, parse)
                .unwrap();
        let model = lower(document, &mut budget).unwrap();
        // then
        assert_eq!(
            model,
            Value::Object(vec![
                (
                    "title \"billing\"".into(),
                    Value::Object(vec![
                        ("x".into(), Value::String("3".into())),
                        ("x".into(), Value::String("true".into())),
                        (
                            "y".into(),
                            Value::Array(vec![
                                Value::String("north".into()),
                                Value::Object(vec![("inside".into(), Value::String("yes".into()))])
                            ])
                        )
                    ])
                ),
                (
                    "skinparam".into(),
                    Value::Object(vec![
                        ("duplicate".into(), Value::String("one".into())),
                        ("duplicate".into(), Value::String("two".into()))
                    ])
                )
            ])
        );
    }

    #[test]
    fn should_preserve_hcl_decoded_label_bounds_and_escape_consumption() {
        for (raw, accepted) in [
            ("a".repeat(16_384), true),
            ("a".repeat(16_385), false),
            ("\\a".repeat(16_384), true),
            ("\\a".repeat(16_385), false),
        ] {
            // given
            let lines = [format!("a {{ x=\"{raw}\" }}")].join("\n");
            let mut budget = Budget::new(&RenderOptions::default());
            // when
            let document =
                crate::plantuml::parser::test_support::body((&lines, "hcl"), &mut budget, parse);
            // then
            assert_eq!(document.is_ok(), accepted);
            if let Ok(document) = document {
                let value = lower(document, &mut budget).unwrap();
                assert_eq!(
                    value,
                    Value::Object(vec![("x".into(), Value::String("a".repeat(16_384)))])
                );
            }
        }
    }

    #[test]
    fn should_map_hcl_failure_once_to_its_frozen_definition_use() {
        // given
        let source = "@starthcl\n!define BAD a {x=}\nBAD\n@endhcl";
        let mut budget = Budget::new(&RenderOptions::default());
        let document = crate::plantuml::preprocess::prepare(source, &mut budget).unwrap();
        let mut prepared = super::super::document::parse(&document, &mut budget).unwrap();
        super::super::preamble::parse(&mut prepared, &mut budget).unwrap();
        // when
        let error = parse(prepared.body, &mut budget).err().unwrap().to_string();
        // then
        assert_eq!(error.matches("PlantUML at line 3,").count(), 1, "{error}");
        assert_eq!(error.matches("expansion \"BAD\"").count(), 1, "{error}");
        assert!(error.contains("defined at line 2"), "{error}");
    }

    #[test]
    fn should_reject_hcl_header_lines_expressions_and_unclosed_structure() {
        for text in [
            "service\nother { x=one }",
            "service { x=[one] }",
            "service { x=[[\"one\"]] }",
            "service { x=call(a) }",
            "service { x=\"${one}\" }",
            "service { x=\"one\n two\" }",
            "service { x=\"one\"",
            "service {} service {}",
            "service { x=[\"a\" \"b\"] }",
        ] {
            // given
            let lines = [text.to_owned()].join("\n");
            let mut budget = Budget::new(&RenderOptions::default());
            // when
            let result =
                crate::plantuml::parser::test_support::body((&lines, "hcl"), &mut budget, parse);
            // then
            assert!(result.is_err(), "{text}");
        }
    }
}
