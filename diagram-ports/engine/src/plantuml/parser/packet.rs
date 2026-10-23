//! Generated bounded packetdiag configuration, ranges, labels and attributes.
use super::{
    BodySource, context,
    error::{ParseFailure, generated_failure},
    outline::{self, Lexer},
    ownership::Row,
};
use crate::{
    RenderError,
    plantuml::{
        budget::{Budget, bound},
        packet::{Builder, Packet},
    },
};
use std::ops::Range;

#[derive(Clone, Debug)]
pub(in crate::plantuml) enum Token {
    Packetdiag,
    HeaderSpace,
    Open,
    Close,
    Columns,
    Interval,
    Direction,
    NodeHeight,
    SameHeight,
    Ltr,
    Rtl,
    True,
    False,
    Len,
    Height,
    Integer(usize),
    Unknown(String),
    Star,
    Minus,
    Colon,
    Equals,
    Semi,
    Gap,
    Label(String),
    AttrOpen,
    AttrClose,
    Comma,
    Newline,
}
type Spanned = (usize, Token, usize);

pub(in crate::plantuml) fn parse(
    body: BodySource<'_>,
    budget: &mut Budget,
) -> Result<Packet, RenderError> {
    parse_body(&body, budget).map_err(|failure| failure.into_document(body.document, budget))
}

fn parse_body(body: &BodySource<'_>, budget: &mut Budget) -> Result<Packet, ParseFailure> {
    let tokens = outline::lex(body, budget, |row, base, lexer| {
        row_tokens(
            View {
                row,
                base,
                text: row.text.trim(),
            },
            lexer,
        )
    })?;
    admit((&tokens, tokens.capacity()), body, budget)?;
    let iterator = context::Checked::new(tokens, budget.control.clone());
    let mut builder = Builder::new(budget);
    if let Err(error) = super::generated_packet::PacketParser::new().parse(&mut builder, iterator) {
        drop(builder);
        return Err(describe_failure(error, body, budget));
    }
    builder.finish(body.range.end..body.range.end)
}

fn describe_failure(
    error: lalrpop_util::ParseError<usize, Token, ParseFailure>,
    body: &BodySource<'_>,
    budget: &mut Budget,
) -> ParseFailure {
    if let lalrpop_util::ParseError::User { error } = error {
        return error;
    }
    let failure = generated_failure(
        error,
        body.range.end,
        "invalid or incomplete packetdiag range, configuration, attributes or delimiter",
    );
    let raw = &body.document.text[failure.span.clone()];
    if raw.is_empty() {
        return failure;
    }
    if let Err(error) = budget.text(raw.len().saturating_mul(4).saturating_add(128)) {
        return ParseFailure::terminal(error);
    }
    let super::error::FailureKind::Syntax(detail) = failure.kind else {
        return failure;
    };
    ParseFailure::owned_syntax(failure.span, format!("{detail}: {raw:?}"))
}

fn admit(
    input: (&[Spanned], usize),
    body: &BodySource<'_>,
    budget: &mut Budget,
) -> Result<(), ParseFailure> {
    let (tokens, capacity) = input;
    // Exact exhaustive generated-table witness is recorded in the admission proof.
    let (live, terminals, reductions) = (15usize, 28usize, 4usize);
    let stack = live.next_power_of_two().max(8);
    let slots = (stack + stack / 2) * 2 + 3 * (live + 1) + 3 * terminals.next_power_of_two();
    bound(
        "packet parser stack slots",
        slots,
        budget.limits.layout_work,
    )
    .map_err(ParseFailure::terminal)?;
    budget
        .charge(
            tokens
                .len()
                .saturating_mul(reductions + 1)
                .saturating_add(reductions + slots + terminals * (live + reductions)),
        )
        .map_err(ParseFailure::terminal)?;
    let symbol = measure_symbol();
    let models = budget.limits.model_items.saturating_mul(3).saturating_mul(
        std::mem::size_of::<crate::plantuml::packet::Field>()
            + std::mem::size_of::<crate::plantuml::packet::Fragment>(),
    );
    let bytes = capacity
        .saturating_mul(std::mem::size_of::<Spanned>())
        .saturating_add(body.adapters)
        .saturating_add((stack + stack / 2) * (symbol + 2 * std::mem::size_of::<usize>() + 2))
        .saturating_add(
            (live + 1) * 3 * 2
                + terminals.next_power_of_two() * 3 * std::mem::size_of::<String>()
                + terminals * 32,
        )
        .saturating_add(budget.limits.source_bytes.saturating_mul(20))
        .saturating_add(models)
        .saturating_add(std::mem::size_of::<Builder<'_>>());
    bound(
        "packet parser retained bytes",
        bytes,
        budget
            .limits
            .source_bytes
            .saturating_mul(256)
            .saturating_add(models),
    )
    .map_err(ParseFailure::terminal)
}

fn measure_symbol() -> usize {
    [
        std::mem::size_of::<Token>(),
        std::mem::size_of::<(crate::plantuml::packet::Attribute, Range<usize>)>(),
        std::mem::size_of::<crate::plantuml::packet::Attributes>(),
        std::mem::size_of::<crate::plantuml::packet::Configuration>(),
        std::mem::size_of::<crate::plantuml::packet::Range>(),
        std::mem::size_of::<String>(),
    ]
    .into_iter()
    .max()
    .unwrap_or(0)
        + std::mem::size_of::<usize>()
}

struct View<'r> {
    row: &'r Row,
    base: usize,
    text: &'r str,
}
impl View<'_> {
    fn span(&self, range: Range<usize>) -> Range<usize> {
        let start = self.text.as_ptr() as usize - self.row.text.as_ptr() as usize;
        self.row
            .span(self.base, start + range.start..start + range.end)
    }
}

#[derive(Clone, Copy)]
enum Mode {
    Header,
    ConfigKey,
    ConfigValue,
    Range,
    Attributes,
}

fn row_tokens(view: View<'_>, lexer: &mut Lexer<'_, Token>) -> Result<(), ParseFailure> {
    if view.text.is_empty() {
        return Ok(());
    }
    let first = view.text.as_bytes()[0];
    if (first.is_ascii_alphabetic() && view.text.contains('='))
        || first == b'}'
        || view.text.starts_with("packetdiag")
    {
        let mode =
            if first == b'}' || view.text.starts_with("packetdiag") && !view.text.contains('=') {
                Mode::Header
            } else {
                Mode::ConfigKey
            };
        syntax_tokens(&view, lexer, mode)?;
    } else {
        field_tokens(&view, lexer)?;
    }
    lexer.push(Token::Newline, view.span(view.text.len()..view.text.len()))
}

fn field_tokens(view: &View<'_>, lexer: &mut Lexer<'_, Token>) -> Result<(), ParseFailure> {
    let mut offset = 0;
    while let Some(ch) = view.text[offset..].chars().next() {
        lexer
            .budget
            .charge(ch.len_utf8())
            .map_err(ParseFailure::terminal)?;
        if ch.is_whitespace() {
            break;
        }
        offset += ch.len_utf8();
    }
    syntax_tokens(
        &View {
            row: view.row,
            base: view.base,
            text: &view.text[..offset],
        },
        lexer,
        Mode::Range,
    )?;
    if offset == view.text.len() {
        return Ok(());
    }
    let tail = view.text[offset..].trim();
    lexer.push(
        Token::Gap,
        view.span(offset..offset + view.text[offset..].chars().next().unwrap().len_utf8()),
    )?;
    let mut open = tail.len();
    for (index, ch) in tail.char_indices() {
        lexer
            .budget
            .charge(ch.len_utf8())
            .map_err(ParseFailure::terminal)?;
        if ch == '[' {
            open = index;
            break;
        }
    }
    let label = tail[..open].trim_end();
    let value = lexer.copy_label(label)?;
    let start = label.as_ptr() as usize - view.text.as_ptr() as usize;
    lexer.push(Token::Label(value), view.span(start..start + label.len()))?;
    if open < tail.len() {
        syntax_tokens(
            &View {
                row: view.row,
                base: view.base,
                text: &tail[open..],
            },
            lexer,
            Mode::Attributes,
        )?;
    }
    Ok(())
}

fn syntax_tokens(
    view: &View<'_>,
    lexer: &mut Lexer<'_, Token>,
    mut mode: Mode,
) -> Result<(), ParseFailure> {
    let mut offset = 0;
    while let Some(ch) = view.text[offset..].chars().next() {
        let start = offset;
        lexer
            .budget
            .charge(ch.len_utf8())
            .map_err(ParseFailure::terminal)?;
        if ch.is_whitespace() {
            offset += ch.len_utf8();
            while view.text[offset..]
                .chars()
                .next()
                .is_some_and(char::is_whitespace)
            {
                let ch = view.text[offset..].chars().next().unwrap();
                lexer
                    .budget
                    .charge(ch.len_utf8())
                    .map_err(ParseFailure::terminal)?;
                offset += ch.len_utf8();
            }
            if matches!(mode, Mode::Header) {
                let gap = &view.text[start..offset];
                let token = if gap == " " {
                    Token::HeaderSpace
                } else {
                    Token::Unknown(lexer.copy_syntax(gap)?)
                };
                lexer.push(token, view.span(start..offset))?;
            }
            continue;
        }
        let token = match ch {
            '0'..='9' => {
                offset += 1;
                while view
                    .text
                    .as_bytes()
                    .get(offset)
                    .is_some_and(u8::is_ascii_digit)
                {
                    if offset - start == 7 {
                        lexer.budget.text(128).map_err(ParseFailure::terminal)?;
                        return Err(ParseFailure::owned_syntax(
                            view.span(start..offset + 1),
                            format!(
                                "invalid packet integer {:?}: packet integer digits exceed 7",
                                &view.text[start..offset + 1]
                            ),
                        ));
                    }
                    lexer.budget.charge(1).map_err(ParseFailure::terminal)?;
                    offset += 1;
                }
                Token::Integer(view.text[start..offset].parse().map_err(|_| {
                    ParseFailure::syntax(view.span(start..offset), "invalid packet integer")
                })?)
            }
            '{' => {
                offset += 1;
                Token::Open
            }
            '}' => {
                offset += 1;
                Token::Close
            }
            '[' => {
                offset += 1;
                Token::AttrOpen
            }
            ']' => {
                offset += 1;
                Token::AttrClose
            }
            '*' => {
                offset += 1;
                Token::Star
            }
            '-' => {
                offset += 1;
                Token::Minus
            }
            ':' => {
                offset += 1;
                Token::Colon
            }
            '=' => {
                offset += 1;
                if matches!(mode, Mode::ConfigKey) {
                    mode = Mode::ConfigValue;
                }
                Token::Equals
            }
            ';' => {
                offset += 1;
                Token::Semi
            }
            ',' => {
                offset += 1;
                Token::Comma
            }
            _ => {
                offset += ch.len_utf8();
                while let Some(ch) = view.text[offset..].chars().next() {
                    if ch.is_whitespace() || "{}[]*-:=;,".contains(ch) {
                        break;
                    }
                    lexer
                        .budget
                        .charge(ch.len_utf8())
                        .map_err(ParseFailure::terminal)?;
                    offset += ch.len_utf8();
                }
                word(&view.text[start..offset], mode, lexer)?
            }
        };
        lexer.push(token, view.span(start..offset))?;
    }
    Ok(())
}

fn word(raw: &str, mode: Mode, lexer: &mut Lexer<'_, Token>) -> Result<Token, ParseFailure> {
    Ok(match (mode, raw) {
        (Mode::Header, "packetdiag") => Token::Packetdiag,
        (Mode::ConfigKey, "colwidth") => Token::Columns,
        (Mode::ConfigKey, "scale_interval") => Token::Interval,
        (Mode::ConfigKey, "scale_direction") => Token::Direction,
        (Mode::ConfigKey, "node_height") => Token::NodeHeight,
        (Mode::ConfigKey, "same_height") => Token::SameHeight,
        (Mode::ConfigValue, "ltr") => Token::Ltr,
        (Mode::ConfigValue, "rtl") => Token::Rtl,
        (Mode::ConfigValue, "true") => Token::True,
        (Mode::ConfigValue, "false") => Token::False,
        (Mode::Attributes, "len") => Token::Len,
        (Mode::Attributes, "height") => Token::Height,
        _ => Token::Unknown(lexer.copy_syntax(raw)?),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        RenderOptions,
        plantuml::{packet::Direction, parser::outline::test_parse, source::Line},
    };

    fn prepared_source(source: &str, budget: &mut Budget) -> crate::plantuml::origin::Document {
        crate::plantuml::preprocess::prepare(source, budget).unwrap()
    }

    #[test]
    fn should_admit_exact_source_quota_and_flat_packet_nesting() {
        for text in ["0 X", "packetdiag{\n0: [len=1]\n}"] {
            // given
            let source = format!("@startpacketdiag\n{text}\n@endpacketdiag");
            let mut options = RenderOptions::default();
            options.limits.source_bytes = source.len();
            options.limits.nesting_depth = 1;
            let mut budget = Budget::new(&options);
            let document = prepared_source(&source, &mut budget);
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
    }

    #[test]
    fn should_preserve_sparse_maximum_bit_identity_without_expanding_gaps() {
        // given
        let lines = [Line {
            number: 2,
            text: "9999999: Sparse".into(),
        }];
        let mut budget = Budget::new(&RenderOptions::default());
        // when
        let model = test_parse(&lines, "packetdiag", &mut budget, parse).unwrap();
        // then
        assert_eq!(
            (model.fields[0].start, model.fields[0].end),
            (9_999_999, 9_999_999)
        );
        assert_eq!(
            (model.rows, model.fragments.len(), model.fragments[0].width),
            (1, 1, 1)
        );
    }

    #[test]
    fn should_reject_eight_digit_ranges_before_scanning_or_retaining_the_label() {
        let mut observed = Vec::new();
        for suffix in ["unseen".to_owned(), "x".repeat(16_000)] {
            // given
            let source = format!("@startpacketdiag\n99999999: {suffix}\n@endpacketdiag");
            let mut budget = Budget::new(&RenderOptions::default());
            let document = prepared_source(&source, &mut budget);
            let mut prepared = super::super::document::parse(&document, &mut budget).unwrap();
            super::super::preamble::parse(&mut prepared, &mut budget).unwrap();
            let mut work = 0;
            let mut retained = usize::MAX;
            // when
            let result = outline::lex(&prepared.body, &mut budget, |row, base, lexer| {
                let before = lexer.budget.work;
                let result = row_tokens(
                    View {
                        row,
                        base,
                        text: row.text.trim(),
                    },
                    lexer,
                );
                work = lexer.budget.work - before;
                retained = lexer.tokens.len();
                result
            });
            // then
            assert!(format!("{:?}", result.err().unwrap()).contains("packet integer digits"));
            assert_eq!(retained, 0);
            observed.push(work);
        }
        assert_eq!(observed[0], observed[1]);
    }

    #[test]
    fn should_bound_packet_token_growth_before_retaining_a_fifth_token() {
        // given
        let source = "@startpacketdiag\n0-1: unseen\n@endpacketdiag";
        let mut budget = Budget::new(&RenderOptions::default());
        let document = prepared_source(source, &mut budget);
        let mut prepared = super::super::document::parse(&document, &mut budget).unwrap();
        super::super::preamble::parse(&mut prepared, &mut budget).unwrap();
        let mut retained = (0, 0);
        // when
        let result = outline::lex(&prepared.body, &mut budget, |row, base, lexer| {
            lexer.budget.limits.source_bytes = 4;
            let result = row_tokens(
                View {
                    row,
                    base,
                    text: row.text.trim(),
                },
                lexer,
            );
            retained = (lexer.tokens.len(), lexer.tokens.capacity());
            result
        });
        // then
        assert!(result.is_err());
        assert_eq!(retained, (4, 4));
    }

    #[test]
    fn should_reject_packet_work_before_generated_entry_and_bound_actual_payloads() {
        // given
        let body = BodySource {
            document: &crate::plantuml::origin::EMPTY,
            range: 0..0,
            ownership: Default::default(),
            adapters: 0,
        };
        let tokens = vec![(0, Token::Newline, 0)];
        let mut budget = Budget::new(&RenderOptions::default());
        budget.limits.layout_work = 1;
        // when
        let result = admit((&tokens, tokens.capacity()), &body, &mut budget);
        // then
        assert!(result.is_err());
        assert_eq!(measure_symbol(), 40);
        assert_eq!(std::mem::size_of::<crate::plantuml::packet::Field>(), 48);
        assert_eq!(std::mem::size_of::<crate::plantuml::packet::Fragment>(), 32);
    }

    #[test]
    fn should_stop_huge_packet_projection_at_the_existing_model_budget() {
        // given
        let lines = [Line {
            number: 2,
            text: "colwidth=1\n0-9999999: Huge".into(),
        }];
        let mut options = RenderOptions::default();
        options.limits.model_items = 3;
        let mut budget = Budget::new(&options);
        // when
        let error = test_parse(&lines, "packetdiag", &mut budget, parse)
            .err()
            .unwrap();
        // then
        assert!(
            error
                .to_string()
                .contains("model items value 4 exceeds limit 3"),
            "{error}"
        );
    }

    #[test]
    fn should_let_generated_packet_structure_reject_malformed_primitive_streams() {
        for tokens in [
            vec![
                Token::Integer(0),
                Token::Colon,
                Token::Colon,
                Token::Gap,
                Token::Label("X".into()),
                Token::Newline,
            ],
            vec![
                Token::Star,
                Token::Colon,
                Token::Gap,
                Token::Label("X".into()),
                Token::Newline,
            ],
            vec![
                Token::Columns,
                Token::Equals,
                Token::Integer(8),
                Token::Semi,
                Token::Semi,
                Token::Newline,
            ],
            vec![
                Token::Packetdiag,
                Token::Open,
                Token::Newline,
                Token::Integer(0),
                Token::Gap,
                Token::Label("X".into()),
                Token::Newline,
            ],
            vec![
                Token::Integer(0),
                Token::Gap,
                Token::Label("X".into()),
                Token::AttrOpen,
                Token::Len,
                Token::Equals,
                Token::Integer(1),
                Token::Comma,
                Token::AttrClose,
                Token::Newline,
            ],
        ] {
            // given
            let tokens = tokens
                .into_iter()
                .map(|token| (0, token, 0))
                .collect::<Vec<_>>();
            let body = BodySource {
                document: &crate::plantuml::origin::EMPTY,
                range: 0..0,
                ownership: Default::default(),
                adapters: 0,
            };
            let mut budget = Budget::new(&RenderOptions::default());
            admit((&tokens, tokens.capacity()), &body, &mut budget).unwrap();
            let iterator = context::Checked::new(tokens, budget.control.clone());
            let mut builder = Builder::new(&mut budget);
            // when
            let result =
                super::super::generated_packet::PacketParser::new().parse(&mut builder, iterator);
            // then
            assert!(result.is_err());
        }
    }

    #[test]
    fn should_cancel_generated_packet_delivery_and_retaining_field_actions() {
        for checkpoint in 1..=8 {
            // given
            let tokens = vec![
                (0, Token::Integer(0), 0),
                (0, Token::Gap, 0),
                (0, Token::Label("X".into()), 0),
                (0, Token::Newline, 0),
            ];
            let body = BodySource {
                document: &crate::plantuml::origin::EMPTY,
                range: 0..0,
                ownership: Default::default(),
                adapters: 0,
            };
            let mut budget = Budget::new(&RenderOptions::default());
            admit((&tokens, tokens.capacity()), &body, &mut budget).unwrap();
            let iterator = context::Checked::new(tokens, budget.control.clone());
            budget.control.cancel_after_checkpoints(checkpoint);
            let mut builder = Builder::new(&mut budget);
            // when
            let result =
                super::super::generated_packet::PacketParser::new().parse(&mut builder, iterator);
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
    fn should_cancel_packet_literal_lexing_and_fragment_projection() {
        // given
        let source = format!("@startpacketdiag\n0: {}\n@endpacketdiag", "x".repeat(4096));
        let mut budget = Budget::new(&RenderOptions::default());
        let document = prepared_source(&source, &mut budget);
        let mut prepared = super::super::document::parse(&document, &mut budget).unwrap();
        super::super::preamble::parse(&mut prepared, &mut budget).unwrap();
        budget.control.cancel_after_checkpoints(3);
        // when
        let error = parse(prepared.body, &mut budget).err().unwrap();
        // then
        assert!(matches!(error, RenderError::Cancelled(_)));
        // given
        let mut budget = Budget::new(&RenderOptions::default());
        let control = budget.control.clone();
        let mut builder = Builder::new(&mut budget);
        builder
            .field(
                crate::plantuml::packet::FieldInput {
                    range: crate::plantuml::packet::Range::Inclusive(0, 9_999_999),
                    label: "Huge".into(),
                    attributes: Default::default(),
                },
                0..0,
            )
            .unwrap();
        // when
        control.cancel_after_checkpoints(3);
        let error = builder.finish(0..0).err().unwrap();
        // then
        assert!(matches!(
            error.kind,
            super::super::error::FailureKind::Terminal(RenderError::Cancelled(_))
        ));
    }

    #[test]
    fn should_map_packet_attribute_failure_once_to_a_frozen_definition_use() {
        // given
        let source = "@startpacketdiag\n!define BAD rotate=90\n0: Label [BAD]\n@endpacketdiag";
        let mut budget = Budget::new(&RenderOptions::default());
        let document = prepared_source(source, &mut budget);
        let mut prepared = super::super::document::parse(&document, &mut budget).unwrap();
        super::super::preamble::parse(&mut prepared, &mut budget).unwrap();
        // when
        let error = parse(prepared.body, &mut budget).err().unwrap().to_string();
        // then
        assert_eq!(error.matches("PlantUML at line 3,").count(), 1, "{error}");
        assert_eq!(error.matches("expansion \"BAD\"").count(), 1, "{error}");
        assert!(error.contains("defined at line 2"), "{error}");
        assert!(error.contains("rotate"), "{error}");
    }

    #[test]
    fn should_report_the_offending_packet_configuration_value() {
        // given
        let lines = [Line {
            number: 2,
            text: "scale_direction=sideways\n0: X".into(),
        }];
        let mut budget = Budget::new(&RenderOptions::default());
        // when
        let error = test_parse(&lines, "packetdiag", &mut budget, parse)
            .err()
            .unwrap()
            .to_string();
        // then
        assert!(error.contains("sideways"), "{error}");
        assert!(error.contains("at line 2,"), "{error}");
    }

    #[test]
    fn should_keep_value_keywords_unsupported_as_configuration_names() {
        for key in [
            "ltr",
            "rtl",
            "true",
            "false",
            "len",
            "height",
            "packetdiag",
            "bogus",
        ] {
            // given
            let lines = [Line {
                number: 2,
                text: format!("{key}=1\n0: X"),
            }];
            let mut budget = Budget::new(&RenderOptions::default());
            // when
            let error = test_parse(&lines, "packetdiag", &mut budget, parse)
                .err()
                .unwrap();
            // then
            assert!(
                matches!(error, RenderError::UnsupportedFeature(_)),
                "{key}: {error}"
            );
        }
    }

    #[test]
    fn should_reject_packet_tick_work_and_output_limits_without_partial_output() {
        for resource in ["ticks", "work", "output"] {
            // given
            let mut options = RenderOptions {
                language: crate::Language::PlantUml,
                ..Default::default()
            };
            match resource {
                "ticks" => options.limits.model_items = 8,
                "work" => options.limits.layout_work = 64,
                _ => options.limits.svg_bytes = 128,
            }
            let source = "@startpacketdiag\ncolwidth=999\n0: X\n@endpacketdiag";
            // when
            let result = crate::render(source, &options);
            // then
            assert!(result.is_err(), "unbounded packet {resource}");
        }
    }

    #[test]
    fn should_generate_inclusive_auto_fields_and_conserved_payload_fragments() {
        for text in [
            "colwidth=8\n0-2: Prefix\n3-22: Payload\n* Flags [len=2,height=3]",
            "packetdiag {\ncolwidth=8;\n0-2: Prefix\n3-22: Payload\n* Flags [len=2,height=3]\n}",
        ] {
            // given
            let lines = [Line {
                number: 2,
                text: text.into(),
            }];
            let mut budget = Budget::new(&RenderOptions::default());
            // when
            let model = test_parse(&lines, "packetdiag", &mut budget, parse).unwrap();
            // then
            assert_eq!(
                model
                    .fields
                    .iter()
                    .map(|field| (field.start, field.end, field.label.as_str(), field.height))
                    .collect::<Vec<_>>(),
                [
                    (0, 2, "Prefix", 1),
                    (3, 22, "Payload", 1),
                    (23, 24, "Flags", 3)
                ]
            );
            assert_eq!(
                model
                    .fragments
                    .iter()
                    .filter(|fragment| fragment.field == 1)
                    .map(|fragment| fragment.width)
                    .collect::<Vec<_>>(),
                [5, 8, 7]
            );
        }
    }

    #[test]
    fn should_generate_final_configuration_effects_and_literal_packet_labels() {
        // given
        let lines = [Line { number: 2, text: "0-3: title **literal** /' # !include = value\n4-11: Eight [height=2]\ncolwidth=12\nscale_interval=9999999\nscale_direction=rtl\nnode_height=50\nsame_height=true".into() }];
        let mut budget = Budget::new(&RenderOptions::default());
        // when
        let model = test_parse(&lines, "packetdiag", &mut budget, parse).unwrap();
        // then
        assert_eq!(
            (
                model.config.columns,
                model.config.interval,
                model.config.node_height,
                model.config.same_height
            ),
            (12, 12, 50, true)
        );
        assert!(matches!(model.config.direction, Direction::Rtl));
        assert_eq!(
            model.fields[0].label,
            "title **literal** /' # !include = value"
        );
        assert_eq!(model.rows, 1);
        assert_eq!(
            model
                .fragments
                .iter()
                .map(|fragment| fragment.width)
                .collect::<Vec<_>>(),
            [4, 8]
        );
    }

    #[test]
    fn should_reject_packet_range_attribute_config_and_wrapper_suffixes() {
        for text in [
            "0:: X",
            "*: X",
            "0: X [rotate=90]",
            "0: X [len=1,len=2]",
            "0: X [height=2,]",
            "0: X [len=1] suffix",
            "0-1: X [len=2]",
            "colwidth=8;;\n0: X",
            "scale_direction=unknown\n0: X",
            "packetdiag {\n0: X",
            "0: X\n}",
            "packetdiag {\n0: X\n}\n0: trailing",
            "0: X\npacketdiag {\n1: Y\n}",
            "packetdiag  {\n0: X\n}",
            "packetdiag\t{\n0: X\n}",
            "packetdiag\n{\n0: X\n}",
        ] {
            // given
            let lines = [Line {
                number: 2,
                text: text.into(),
            }];
            let mut budget = Budget::new(&RenderOptions::default());
            // when
            let result = test_parse(&lines, "packetdiag", &mut budget, parse);
            // then
            assert!(result.is_err(), "{text}");
        }
    }
}
