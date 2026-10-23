//! Generated strict ordered JSON syntax.
use super::{
    BodySource, context,
    error::{ParseFailure, generated_failure},
    structured::{Arena, Cursor, DepthToken, Syntax},
};
use crate::{
    RenderError,
    plantuml::{
        budget::{Budget, bound},
        data::Value,
    },
};
use std::ops::Range;

#[derive(Clone, Debug)]
pub(in crate::plantuml) enum Token<'s> {
    Object,
    EndObject,
    Array,
    EndArray,
    Colon,
    Comma,
    String(&'s str),
    Zero,
    Digits,
    Minus,
    Plus,
    Dot,
    Exponent,
    True,
    False,
    Null,
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
) -> Result<Value, RenderError> {
    parse_body(&body, budget).map_err(|error| error.into_document(body.document, budget))
}

fn parse_body(body: &BodySource<'_>, budget: &mut Budget) -> Result<Value, ParseFailure> {
    let tokens = lex(body, budget)?;
    let arena = Arena::admit((&tokens, tokens.capacity()), body, budget, Syntax::Json)?;
    let iterator = context::Checked::new(tokens, budget.control.clone());
    let mut builder = Builder {
        input: &body.document.text,
        budget,
        arena,
    };
    super::generated_json::JsonParser::new()
        .parse(&mut builder, iterator)
        .map_err(|error| {
            generated_failure(
                error,
                body.range.end,
                "invalid or incomplete JSON object, array, number or separator",
            )
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
    let mut previous = ' ';
    let mut number_end = usize::MAX;
    while let Some(ch) = cursor.peek() {
        if matches!(ch, ' ' | '\t' | '\r' | '\n') {
            cursor.take()?;
            continue;
        }
        let start = cursor.offset;
        let token = match ch {
            '{' | '[' => {
                admit_value(cursor, nesting.len() + 1)?;
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
                if ch == '}' {
                    Token::EndObject
                } else {
                    Token::EndArray
                }
            }
            ':' => {
                cursor.take()?;
                Token::Colon
            }
            ',' => {
                cursor.take()?;
                Token::Comma
            }
            '"' => {
                if nesting.last() == Some(&'{') && matches!(previous, '{' | ',') {
                    cursor.budget.item().map_err(ParseFailure::terminal)?;
                } else {
                    admit_value(cursor, nesting.len() + 1)?;
                }
                Token::String(string(cursor)?)
            }
            't' | 'f' | 'n' => {
                admit_value(cursor, nesting.len() + 1)?;
                keyword(
                    cursor,
                    if ch == 't' {
                        "true"
                    } else if ch == 'f' {
                        "false"
                    } else {
                        "null"
                    },
                )?;
                if ch == 't' {
                    Token::True
                } else if ch == 'f' {
                    Token::False
                } else {
                    Token::Null
                }
            }
            '-' | '+' | '.' | 'e' | 'E' | '0'..='9' => {
                if number_end != start {
                    admit_value(cursor, nesting.len() + 1)?;
                }
                let token = numeric(cursor)?;
                number_end = cursor.offset;
                token
            }
            '/' | '#' | '!' => {
                return Err(ParseFailure::unsupported(
                    start..start + ch.len_utf8(),
                    "JSON comments, directives or styles outside strict JSON subset",
                ));
            }
            _ => {
                return Err(ParseFailure::syntax(
                    start..start + ch.len_utf8(),
                    "expected JSON value or punctuation",
                ));
            }
        };
        context::push(&mut tokens, (start, token, cursor.offset), cursor.budget)?;
        previous = ch;
    }
    Ok(tokens)
}

fn admit_value(cursor: &mut Cursor<'_, '_>, depth: usize) -> Result<(), ParseFailure> {
    cursor.budget.depth(depth).map_err(ParseFailure::terminal)?;
    cursor.budget.item().map_err(ParseFailure::terminal)
}

fn string<'s>(cursor: &mut Cursor<'s, '_>) -> Result<&'s str, ParseFailure> {
    let start = cursor.offset;
    cursor.take()?;
    loop {
        bound("JSON string lexeme bytes", cursor.offset - start, 16 * 1024)
            .map_err(ParseFailure::terminal)?;
        match cursor.peek() {
            Some('"') => {
                cursor.take()?;
                return Ok(&cursor.input[start..cursor.offset]);
            }
            Some('\\') => {
                cursor.take()?;
                cursor.take()?;
            }
            Some('\0'..='\u{1f}') | None => {
                return Err(ParseFailure::syntax(
                    start..cursor.offset,
                    "unterminated or control character in JSON string",
                ));
            }
            _ => {
                cursor.take()?;
            }
        }
    }
}

fn keyword(cursor: &mut Cursor<'_, '_>, word: &str) -> Result<(), ParseFailure> {
    for ch in word.chars() {
        if cursor.peek() != Some(ch) {
            return Err(ParseFailure::syntax(
                cursor.offset..cursor.offset,
                "invalid JSON keyword",
            ));
        }
        cursor.take()?;
    }
    Ok(())
}

fn numeric(cursor: &mut Cursor<'_, '_>) -> Result<Token<'static>, ParseFailure> {
    let start = cursor.offset;
    let ch = cursor.take()?;
    Ok(match ch {
        '-' => Token::Minus,
        '+' => Token::Plus,
        '.' => Token::Dot,
        'e' | 'E' => Token::Exponent,
        '0' => Token::Zero,
        _ => {
            while cursor.peek().is_some_and(|ch| ch.is_ascii_digit()) {
                bound("JSON numeric digits", cursor.offset - start + 1, 128)
                    .map_err(ParseFailure::terminal)?;
                cursor.take()?;
            }
            Token::Digits
        }
    })
}

pub(in crate::plantuml) struct Builder<'s, 'b> {
    input: &'s str,
    budget: &'b mut Budget,
    arena: Arena,
}

impl Builder<'_, '_> {
    pub fn string(&mut self, raw: &str, span: Range<usize>) -> Result<String, ParseFailure> {
        self.budget
            .text(raw.len())
            .map_err(ParseFailure::terminal)?;
        serde_json::from_str::<String>(raw)
            .map_err(|_| ParseFailure::syntax(span, "invalid JSON string escape or Unicode scalar"))
    }

    pub fn number(&mut self, span: Range<usize>) -> Result<Value, ParseFailure> {
        let raw = &self.input[span.clone()];
        bound("JSON number lexeme bytes", raw.len(), 128).map_err(ParseFailure::terminal)?;
        if let Some((_, exponent)) = raw.split_once(['e', 'E']) {
            let digits = exponent.strip_prefix(['+', '-']).unwrap_or(exponent);
            if digits.parse::<u16>().unwrap_or(u16::MAX) > 308 {
                return Err(ParseFailure::syntax(
                    span,
                    "JSON numeric exponent exceeds native bound 308",
                ));
            }
        }
        if raw.parse::<f64>().is_ok_and(f64::is_finite) {
            self.budget
                .text(raw.len())
                .map_err(ParseFailure::terminal)?;
            return Ok(Value::Number(raw.to_owned()));
        }
        Err(ParseFailure::syntax(
            span,
            "nonfinite or malformed JSON number",
        ))
    }

    pub fn push<T>(&mut self, mut values: Vec<T>, value: T) -> Result<Vec<T>, ParseFailure> {
        self.arena.push(&mut values, value, self.budget)?;
        Ok(values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        RenderOptions,
        plantuml::{parser::outline::test_parse, source::Line},
    };

    fn body(input: &str) -> (crate::plantuml::origin::Document, Range<usize>) {
        let mut budget = Budget::new(&RenderOptions::default());
        let source = format!("@startjson\n{input}\n@endjson");
        let document = crate::plantuml::preprocess::prepare(&source, &mut budget).unwrap();
        let range = 11..11 + input.len();
        (document, range)
    }

    #[test]
    fn should_check_json_depth_and_items_before_scanning_nested_content() {
        for resource in ["depth", "items"] {
            // given
            let mut options = RenderOptions::default();
            if resource == "depth" {
                options.limits.nesting_depth = 1;
            } else {
                options.limits.model_items = 1;
            }
            let mut budget = Budget::new(&options);
            let mut cursor = Cursor {
                input: "[\"unseen\"]",
                offset: 0,
                end: 10,
                budget: &mut budget,
            };
            // when
            let result = tokens(&mut cursor);
            // then
            assert!(result.is_err());
            assert_eq!(cursor.offset, 1);
        }
    }

    #[test]
    fn should_interrupt_json_literal_scanning_before_retention() {
        // given
        let input = format!("\"{}\"", "x".repeat(4096));
        let mut budget = Budget::new(&RenderOptions::default());
        budget.control.cancel_after_checkpoints(16);
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
            super::super::error::FailureKind::Terminal(RenderError::Cancelled(_))
        ));
        assert!(cursor.offset > 0 && cursor.offset < 32);
    }

    #[test]
    fn should_admit_json_depth_64_and_stop_at_the_65th_opener() {
        for depth in [64usize, 65] {
            // given
            let input = format!("{}{}", "[".repeat(depth), "]".repeat(depth));
            let (document, range) = body(&input);
            let source = BodySource {
                document: &document,
                range,
                ownership: Default::default(),
                adapters: 0,
            };
            let mut budget = Budget::new(&RenderOptions::default());
            // when
            let result = parse(source, &mut budget);
            // then
            assert_eq!(result.is_ok(), depth == 64);
        }
        // given
        let input = "[".repeat(65) + "\"unseen\"";
        let mut budget = Budget::new(&RenderOptions::default());
        let mut cursor = Cursor {
            input: &input,
            offset: 0,
            end: input.len(),
            budget: &mut budget,
        };
        // when
        let error = tokens(&mut cursor).unwrap_err();
        // then
        assert!(format!("{error:?}").contains("nesting depth"));
        assert_eq!(cursor.offset, 64);
    }

    #[test]
    fn should_admit_deep_json_at_its_actual_source_limit() {
        // given
        let input = format!("{}{}", "[".repeat(64), "]".repeat(64));
        let source = format!("@startjson\n{input}\n@endjson");
        let mut options = RenderOptions::default();
        options.limits.source_bytes = source.len();
        let mut budget = Budget::new(&options);
        let document = crate::plantuml::preprocess::prepare(&source, &mut budget).unwrap();
        let mut prepared = super::super::document::parse(&document, &mut budget).unwrap();
        super::super::preamble::parse(&mut prepared, &mut budget).unwrap();
        // when
        let result = parse(prepared.body, &mut budget);
        // then
        assert!(result.is_ok(), "{result:?}");
    }

    #[test]
    fn should_let_generated_json_structure_reject_invalid_token_orders_and_eof() {
        for stream in [
            vec![
                Token::Object,
                Token::String("\"x\""),
                Token::True,
                Token::EndObject,
            ],
            vec![Token::Array, Token::Null, Token::Comma, Token::EndArray],
            vec![Token::Array, Token::Object, Token::EndObject],
            vec![Token::Zero, Token::Digits],
        ] {
            // given
            let (document, range) = body("{}");
            let source = BodySource {
                document: &document,
                range,
                ownership: Default::default(),
                adapters: 0,
            };
            let mut budget = Budget::new(&RenderOptions::default());
            let stream = stream
                .into_iter()
                .map(|token| (11, token, 11))
                .collect::<Vec<_>>();
            let arena = Arena::admit(
                (&stream, stream.capacity()),
                &source,
                &mut budget,
                Syntax::Json,
            )
            .unwrap();
            let iterator = context::Checked::new(stream, budget.control.clone());
            let mut builder = Builder {
                input: &document.text,
                budget: &mut budget,
                arena,
            };
            // when
            let result =
                super::super::generated_json::JsonParser::new().parse(&mut builder, iterator);
            // then
            assert!(result.is_err());
        }
    }

    #[test]
    fn should_cancel_json_generated_delivery_and_retaining_actions() {
        for checkpoint in 1..=6 {
            // given
            let (document, range) = body("[\"retained\"]");
            let source = BodySource {
                document: &document,
                range,
                ownership: Default::default(),
                adapters: 0,
            };
            let mut budget = Budget::new(&RenderOptions::default());
            let stream = lex(&source, &mut budget).unwrap();
            let arena = Arena::admit(
                (&stream, stream.capacity()),
                &source,
                &mut budget,
                Syntax::Json,
            )
            .unwrap();
            let iterator = context::Checked::new(stream, budget.control.clone());
            let mut builder = Builder {
                input: &document.text,
                budget: &mut budget,
                arena,
            };
            builder.budget.control.cancel_after_checkpoints(checkpoint);
            // when
            let result =
                super::super::generated_json::JsonParser::new().parse(&mut builder, iterator);
            // then
            assert!(
                matches!(
                    result,
                    Err(lalrpop_util::ParseError::User {
                        error: ParseFailure {
                            kind: super::super::error::FailureKind::Terminal(
                                RenderError::Cancelled(_)
                            ),
                            ..
                        }
                    })
                ),
                "checkpoint {checkpoint}: {result:?}"
            );
        }
    }

    #[test]
    fn should_generate_ordered_duplicate_members_and_exact_numbers() {
        // given
        let lines = [Line {
            number: 2,
            text: r#"{"z":-0.00e+02,"a":[null,true,"2",{},[]],"z":3}"#.into(),
        }];
        let mut budget = Budget::new(&RenderOptions::default());
        // when
        let value = test_parse(&lines, "json", &mut budget, parse).unwrap();
        // then
        assert_eq!(
            value,
            Value::Object(vec![
                ("z".into(), Value::Number("-0.00e+02".into())),
                (
                    "a".into(),
                    Value::Array(vec![
                        Value::Null,
                        Value::Boolean(true),
                        Value::String("2".into()),
                        Value::Object(vec![]),
                        Value::Array(vec![])
                    ])
                ),
                ("z".into(), Value::Number("3".into()))
            ])
        );
    }

    #[test]
    fn should_preserve_json_raw_quote_and_numeric_spelling_bounds() {
        for (input, accepted) in [
            (format!("\"{}\"", "a".repeat(16_383)), true),
            (format!("\"{}\"", "a".repeat(16_384)), false),
            (format!("\"{}\"", "\\u0061".repeat(2731)), false),
            ("9".repeat(128), true),
            ("9".repeat(129), false),
            ("1e308".into(), true),
            ("1e309".into(), false),
            ("0e-309".into(), false),
            ("1.2 3".into(), false),
        ] {
            // given
            let lines = [Line {
                number: 2,
                text: input.clone(),
            }];
            let mut budget = Budget::new(&RenderOptions::default());
            // when
            let result = test_parse(&lines, "json", &mut budget, parse);
            // then
            assert_eq!(result.is_ok(), accepted, "{input:?}");
        }
    }

    #[test]
    fn should_map_json_failure_once_to_its_frozen_definition_use() {
        // given
        let source = "@startjson\n!define BAD {\"x\":}\nBAD\n@endjson";
        let mut budget = Budget::new(&RenderOptions::default());
        let document = crate::plantuml::preprocess::prepare(source, &mut budget).unwrap();
        let mut prepared = super::super::document::parse(&document, &mut budget).unwrap();
        super::super::preamble::parse(&mut prepared, &mut budget).unwrap();
        // when
        let error = parse(prepared.body, &mut budget).unwrap_err().to_string();
        // then
        assert_eq!(error.matches("PlantUML at line 3,").count(), 1, "{error}");
        assert_eq!(error.matches("expansion \"BAD\"").count(), 1, "{error}");
        assert!(error.contains("defined at line 2"), "{error}");
    }

    #[test]
    fn should_reject_malformed_json_separators_and_incomplete_containers() {
        for text in [
            "[1 2]",
            "{\"x\" 1}",
            "[1,]",
            "{\"x\":1,}",
            "{\"x\":[1]",
            "{} []",
            "01",
            "1e+",
        ] {
            // given
            let lines = [Line {
                number: 2,
                text: text.into(),
            }];
            let mut budget = Budget::new(&RenderOptions::default());
            // when
            let result = test_parse(&lines, "json", &mut budget, parse);
            // then
            assert!(result.is_err(), "{text}");
        }
    }
}
