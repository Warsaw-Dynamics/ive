use super::super::{budget::Budget, origin::SourceSpan};
use crate::RenderError;

#[derive(Debug)]
pub(in crate::plantuml) struct DefinitionSyntax<'s> {
    pub name: &'s str,
    pub literal: &'s str,
    #[allow(dead_code)] // Authored-name metadata is also retained for later directive diagnostics.
    pub name_span: SourceSpan,
    pub literal_span: SourceSpan,
}

pub(in crate::plantuml) fn parse_definition<'s>(
    raw: &'s str,
    authored: SourceSpan,
    budget: &mut Budget,
) -> Result<DefinitionSyntax<'s>, RenderError> {
    use super::{
        SourceView,
        context::Context,
        token::{CheckedTokens, lex},
    };
    if authored.start_byte.checked_add(raw.len()) != Some(authored.end_byte)
        || authored.column.checked_add(raw.len()).is_none()
    {
        return Err(RenderError::InvalidSource(format!(
            "PlantUML definition source view bytes {}..{}, column {} cannot contain {} bytes",
            authored.start_byte,
            authored.end_byte,
            authored.column,
            raw.len()
        )));
    }
    let view = SourceView {
        text: raw,
        byte_base: authored.start_byte,
    };
    let tokens = lex(view, budget).map_err(|error| error.into_authored(authored))?;
    // Actual generated table: five shifts, two EOF reductions, no recovery.
    // Include runtime vector growth, fixed allocation and expected-token simulation.
    super::context::DEFINITION_ADMISSION.admit(tokens.len(), budget)?;
    let iterator = CheckedTokens::new(tokens, budget.control.clone());
    let mut context = Context { budget, authored };
    super::generated_definition::DefinitionParser::new()
        .parse(&mut context, iterator)
        .map_err(|error| failure(error, view).into_authored(authored))
}

fn failure(
    error: lalrpop_util::ParseError<usize, super::token::Token<'_>, super::error::ParseFailure>,
    view: super::SourceView<'_>,
) -> super::error::ParseFailure {
    use super::error::ParseFailure;
    use lalrpop_util::ParseError;
    match error {
        ParseError::User { error } => error,
        ParseError::InvalidToken { location } => {
            ParseFailure::syntax(location..location, "invalid literal definition token")
        }
        ParseError::UnrecognizedEof { .. } => ParseFailure::syntax(
            view.byte_base + view.text.len()..view.byte_base + view.text.len(),
            "definition requires a same-line nonempty literal",
        ),
        ParseError::UnrecognizedToken {
            token: (start, _, end),
            ..
        }
        | ParseError::ExtraToken {
            token: (start, _, end),
        } => ParseFailure::syntax(start..end, "unexpected literal definition token"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RenderOptions;

    fn span(raw: &str) -> SourceSpan {
        SourceSpan {
            start_byte: 0,
            end_byte: raw.len(),
            line: 1,
            column: 1,
        }
    }

    #[test]
    fn should_parse_literal_definition_with_generated_grammar() {
        // given
        let raw = "!define WHO World";
        let mut budget = Budget::new(&RenderOptions::default());
        // when
        let syntax = parse_definition(raw, span(raw), &mut budget).unwrap();
        // then
        assert_eq!(syntax.name, "WHO");
        assert_eq!(syntax.literal, "World");
        assert_eq!(
            syntax.name_span,
            SourceSpan {
                start_byte: 8,
                end_byte: 11,
                line: 1,
                column: 9
            }
        );
        assert_eq!(
            syntax.literal_span,
            SourceSpan {
                start_byte: 12,
                end_byte: 17,
                line: 1,
                column: 13
            }
        );
    }

    #[test]
    fn should_map_nonzero_utf8_definition_spans() {
        // given
        let raw = "\t!define WHO café 🌍\t ";
        let authored = SourceSpan {
            start_byte: 81,
            end_byte: 81 + raw.len(),
            line: 4,
            column: 3,
        };
        let mut budget = Budget::new(&RenderOptions::default());
        // when
        let syntax = parse_definition(raw, authored, &mut budget).unwrap();
        // then
        assert_eq!(
            syntax.name_span,
            SourceSpan {
                start_byte: 90,
                end_byte: 93,
                line: 4,
                column: 12
            }
        );
        assert_eq!(
            syntax.literal_span,
            SourceSpan {
                start_byte: 94,
                end_byte: 104,
                line: 4,
                column: 16
            }
        );
        assert_eq!(syntax.literal, "café 🌍");
    }

    #[test]
    fn should_preserve_unicode_literal_edges_and_unicode_leading_indentation() {
        // given
        let raw = "\u{2003}!define WHO \u{2003}World\u{a0} \t";
        let mut budget = Budget::new(&RenderOptions::default());
        // when
        let syntax = parse_definition(raw, span(raw), &mut budget).unwrap();
        // then
        assert_eq!(syntax.name, "WHO");
        assert_eq!(syntax.literal, "\u{2003}World\u{a0}");
        assert_eq!(
            syntax.name_span,
            SourceSpan {
                start_byte: 11,
                end_byte: 14,
                line: 1,
                column: 12
            }
        );
        assert_eq!(
            syntax.literal_span,
            SourceSpan {
                start_byte: 15,
                end_byte: 25,
                line: 1,
                column: 16
            }
        );
    }

    #[test]
    fn should_reject_overflowing_or_mismatched_source_views_before_lexing() {
        // given
        let raw = "!define X value";
        for authored in [
            SourceSpan {
                start_byte: usize::MAX,
                end_byte: usize::MAX,
                line: 1,
                column: 1,
            },
            SourceSpan {
                end_byte: 1,
                ..span(raw)
            },
        ] {
            let mut budget = Budget::new(&RenderOptions::default());
            // when
            let error = parse_definition(raw, authored, &mut budget).unwrap_err();
            // then
            assert!(matches!(error, RenderError::InvalidSource(_)), "{error}");
            assert_eq!(budget.work, 0);
        }
    }

    #[test]
    fn should_bound_ascii_names_and_require_same_line_nonempty_literal() {
        // given
        let invalid = [
            "!define 1WHO World".to_owned(),
            "!define WÖ World".to_owned(),
            format!("!define {} World", "W".repeat(65)),
            "!define WHO".into(),
            "!define WHO \t".into(),
            "!define  World".into(),
            "!define WHO\nWorld".into(),
        ];
        for raw in invalid {
            let mut budget = Budget::new(&RenderOptions::default());
            // when
            let error = parse_definition(&raw, span(&raw), &mut budget).unwrap_err();
            // then
            assert!(
                matches!(error, RenderError::InvalidSource(_)),
                "{raw:?}: {error}"
            );
        }
    }

    #[test]
    fn should_accept_boundary_names_literals_and_literal_comment_bytes() {
        // given
        for raw in [
            format!("!define {} {}", "W".repeat(64), "x".repeat(16384)),
            "!define _WHO /' literal '/ !include harmless \\\\".into(),
        ] {
            let mut budget = Budget::new(&RenderOptions::default());
            // when
            let result = parse_definition(&raw, span(&raw), &mut budget);
            // then
            assert!(result.is_ok(), "{result:?}");
        }
    }

    #[test]
    fn should_reject_controls_continuations_and_parameterized_definitions() {
        // given
        for raw in [
            "!define X bad\0value",
            "!define X bad\rvalue",
            "!define X bad\u{ffff}value",
            "!define X ends\\",
            "!define X(a) value",
        ] {
            let mut budget = Budget::new(&RenderOptions::default());
            // when
            let error = parse_definition(raw, span(raw), &mut budget).unwrap_err();
            // then
            assert!(
                matches!(error, RenderError::UnsupportedFeature(_)),
                "{raw:?}: {error}"
            );
        }
    }

    #[test]
    fn should_reject_overlong_literals_and_complete_input_after_a_line_break() {
        // given
        for raw in [
            format!("!define X {}", "x".repeat(16385)),
            "!define X value\n!define Y other".into(),
        ] {
            let mut budget = Budget::new(&RenderOptions::default());
            // when
            let result = parse_definition(&raw, span(&raw), &mut budget);
            // then
            assert!(result.is_err());
        }
    }

    #[test]
    fn should_admit_generated_stacks_before_parser_entry() {
        // given
        let raw = "!define X value";
        let mut options = RenderOptions::default();
        options.limits.layout_work = 25;
        let mut budget = Budget::new(&options);
        // when
        let error = parse_definition(raw, span(raw), &mut budget).unwrap_err();
        // then
        assert!(error.to_string().contains("parser stack"), "{error}");
    }

    #[test]
    fn should_reject_insufficient_work_before_generated_parser_entry() {
        // given
        let raw = "!define X value";
        let mut options = RenderOptions::default();
        options.limits.layout_work = 40;
        let mut budget = Budget::new(&options);
        // when
        let error = parse_definition(raw, span(raw), &mut budget).unwrap_err();
        // then
        assert!(error.to_string().contains("work units"), "{error}");
        assert!(budget.work < 40);
    }

    #[test]
    fn should_cancel_during_lexical_trivia_scanning_before_retention() {
        // given
        let raw = format!("{}!define X value", " ".repeat(4096));
        let mut budget = Budget::new(&RenderOptions::default());
        budget.control.cancel_after_checkpoints(3);
        // when
        let error = parse_definition(&raw, span(&raw), &mut budget).unwrap_err();
        // then
        assert!(matches!(error, RenderError::Cancelled(_)), "{error}");
        assert!(budget.work > 0 && budget.work < raw.len());
    }

    #[test]
    fn should_cancel_generated_token_delivery_without_fake_eof() {
        // given
        use super::super::{
            SourceView,
            context::Context,
            token::{CheckedTokens, lex},
        };
        let raw = "!define X value";
        let mut budget = Budget::new(&RenderOptions::default());
        let tokens = lex(
            SourceView {
                text: raw,
                byte_base: 0,
            },
            &mut budget,
        )
        .unwrap();
        super::super::context::DEFINITION_ADMISSION
            .admit(tokens.len(), &mut budget)
            .unwrap();
        budget.control.cancel_after_checkpoints(2);
        let iterator = CheckedTokens::new(tokens, budget.control.clone());
        let mut context = Context {
            budget: &mut budget,
            authored: span(raw),
        };
        // when
        let error = super::super::generated_definition::DefinitionParser::new()
            .parse(&mut context, iterator)
            .unwrap_err();
        // then
        let error = failure(
            error,
            SourceView {
                text: raw,
                byte_base: 0,
            },
        )
        .into_authored(span(raw));
        assert!(matches!(error, RenderError::Cancelled(_)), "{error}");
    }

    #[test]
    fn should_cancel_generated_reduction_and_literal_validation() {
        // given
        use super::super::{
            SourceView,
            context::Context,
            token::{CheckedTokens, lex},
        };
        let raw = format!("!define X {}", "é".repeat(2048));
        for successful_checkpoints in [6, 8] {
            let mut budget = Budget::new(&RenderOptions::default());
            let tokens = lex(
                SourceView {
                    text: &raw,
                    byte_base: 0,
                },
                &mut budget,
            )
            .unwrap();
            super::super::context::DEFINITION_ADMISSION
                .admit(tokens.len(), &mut budget)
                .unwrap();
            budget
                .control
                .cancel_after_checkpoints(successful_checkpoints);
            let iterator = CheckedTokens::new(tokens, budget.control.clone());
            let mut context = Context {
                budget: &mut budget,
                authored: span(&raw),
            };
            // when
            let error = super::super::generated_definition::DefinitionParser::new()
                .parse(&mut context, iterator)
                .unwrap_err();
            // then
            let error = failure(
                error,
                SourceView {
                    text: &raw,
                    byte_base: 0,
                },
            )
            .into_authored(span(&raw));
            assert!(matches!(error, RenderError::Cancelled(_)), "{error}");
        }
    }

    #[test]
    fn should_deliver_cancelled_at_exhausted_iterator_instead_of_eof() {
        // given
        use super::super::{error::FailureKind, token::CheckedTokens};
        let control = crate::RenderControl::default();
        let mut iterator = CheckedTokens::new(Vec::new(), control.clone());
        control.cancel();
        // when
        let result = iterator.next().unwrap().unwrap_err();
        // then
        assert!(matches!(
            result.kind,
            FailureKind::Terminal(RenderError::Cancelled(_))
        ));
    }
}
