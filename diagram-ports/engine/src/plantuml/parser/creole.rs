//! Generated label-local admitted Creole nesting, preserving run/style consumers.
use super::{
    context,
    error::{FailureKind, ParseFailure, generated_failure},
};
use crate::{
    RenderError,
    plantuml::{
        budget::{Budget, bound},
        creole::{self, Builder, Mark, Pair, Run, RunStyle, Scope},
    },
};
#[derive(Clone, Debug)]
pub(in crate::plantuml) enum Token {
    Text(String),
    Literal(&'static str),
    Break,
    TagBoldOpen,
    TagBoldClose,
    TagItalicOpen,
    TagItalicClose,
    TagUnderlineOpen,
    TagUnderlineClose,
    TagStrikeOpen,
    TagStrikeClose,
    BoldOpen,
    BoldClose,
    ItalicOpen,
    ItalicClose,
    UnderlineOpen,
    UnderlineClose,
    StrikeOpen,
    StrikeClose,
}
pub(in crate::plantuml) fn parse(
    text: &str,
    base: RunStyle,
    budget: &mut Budget,
) -> Result<Vec<Vec<Run>>, RenderError> {
    parse_text(text, base, budget).map_err(|error| label_failure(error, text, budget))
}
fn parse_text(
    text: &str,
    base: RunStyle,
    budget: &mut Budget,
) -> Result<Vec<Vec<Run>>, ParseFailure> {
    bound("label bytes", text.len(), 16 * 1024)?;
    let tokens = creole::lex(text, budget)?;
    reduce((text, tokens), base, budget)
}
pub(in crate::plantuml) fn parse_display(
    text: &str,
    base: RunStyle,
    budget: &mut Budget,
) -> Result<Vec<Vec<Run>>, RenderError> {
    let result = (|| {
        let tokens = creole::lex_display(text, budget)?;
        reduce((text, tokens), base, budget)
    })();
    result.map_err(|error| label_failure(error, text, budget))
}
fn reduce(
    input: (&str, Vec<creole::Spanned>),
    base: RunStyle,
    budget: &mut Budget,
) -> Result<Vec<Vec<Run>>, ParseFailure> {
    let (text, tokens) = input;
    admit((&tokens, tokens.capacity()), budget)?;
    let iterator = context::Checked::new(tokens, budget.control.clone())
        .map(|token| token.map(|(start, token, end)| (start, convert(token), end)));
    let mut builder = Builder::new(base, budget);
    super::generated_creole::CreoleParser::new()
        .parse(&mut builder, iterator)
        .map_err(|error| {
            generated_failure(
                error,
                text.len(),
                "crossed, mismatched or unclosed styled span",
            )
        })?;
    Ok(builder.finish())
}
fn convert(token: creole::Token) -> Token {
    use creole::Token as Lex;
    match token {
        Lex::Text(value) => Token::Text(value),
        Lex::Break => Token::Break,
        Lex::Symbol(mark, Pair::Literal) => Token::Literal(mark.symbol()),
        Lex::Symbol(Mark::Bold, Pair::Open) => Token::BoldOpen,
        Lex::Symbol(Mark::Bold, Pair::Close) => Token::BoldClose,
        Lex::Symbol(Mark::Italic, Pair::Open) => Token::ItalicOpen,
        Lex::Symbol(Mark::Italic, Pair::Close) => Token::ItalicClose,
        Lex::Symbol(Mark::Underline, Pair::Open) => Token::UnderlineOpen,
        Lex::Symbol(Mark::Underline, Pair::Close) => Token::UnderlineClose,
        Lex::Symbol(Mark::Strike, Pair::Open) => Token::StrikeOpen,
        Lex::Symbol(Mark::Strike, Pair::Close) => Token::StrikeClose,
        Lex::Open(Mark::Bold) => Token::TagBoldOpen,
        Lex::Close(Mark::Bold) => Token::TagBoldClose,
        Lex::Open(Mark::Italic) => Token::TagItalicOpen,
        Lex::Close(Mark::Italic) => Token::TagItalicClose,
        Lex::Open(Mark::Underline) => Token::TagUnderlineOpen,
        Lex::Close(Mark::Underline) => Token::TagUnderlineClose,
        Lex::Open(Mark::Strike) => Token::TagStrikeOpen,
        Lex::Close(Mark::Strike) => Token::TagStrikeClose,
        _ => unreachable!("unsupported paired marks are rejected during lexical eligibility"),
    }
}
fn label_failure(error: ParseFailure, text: &str, budget: &mut Budget) -> RenderError {
    let detail = match error.kind {
        FailureKind::Terminal(error) => return error,
        FailureKind::Syntax(detail) | FailureKind::Unsupported(detail) => detail,
    };
    if let Err(error) = budget.charge(text.len()) {
        return error;
    }
    let start = error.span.start.min(text.len());
    let end = error.span.end.min(text.len()).max(start);
    let prefix = &text[..start];
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count() + 1;
    let column = prefix.rfind('\n').map_or(start + 1, |last| start - last);
    let message = format!(
        "PlantUML label-local line {line}, column {column}: {detail} (label-local bytes {start}..{end})"
    );
    RenderError::UnsupportedFeature(message)
}
fn admit(input: (&[creole::Spanned], usize), budget: &mut Budget) -> Result<(), ParseFailure> {
    let (tokens, capacity) = input;
    let mut literals = 0usize;
    let mut opens = 0usize;
    for (_, token, _) in tokens {
        budget.charge(1)?;
        match token {
            creole::Token::Text(value) => literals += value.capacity(),
            creole::Token::Open(_) | creole::Token::Symbol(_, Pair::Open) => opens += 1,
            _ => {}
        }
    }
    let depth = opens.min(4).min(budget.limits.nesting_depth);
    let live = depth * 3 + 8;
    let slots = live.next_power_of_two().max(8);
    let growth = slots + slots / 2;
    let symbol = std::mem::size_of::<Token>()
        + std::mem::size_of::<Scope>()
        + std::mem::size_of::<RunStyle>()
        + 4 * std::mem::size_of::<usize>();
    let retained = capacity
        .saturating_mul(std::mem::size_of::<creole::Spanned>())
        .saturating_add(literals)
        .saturating_add(
            growth * symbol
                + growth * std::mem::size_of::<usize>()
                + 3 * (live + 1) * std::mem::size_of::<usize>()
                + 48 * std::mem::size_of::<String>()
                + 19 * 32,
        )
        .saturating_add(std::mem::size_of::<Builder<'_>>())
        .saturating_add(
            3 * (tokens.len() + 1) * (std::mem::size_of::<Run>() + std::mem::size_of::<Vec<Run>>()),
        )
        .saturating_add(16 * 1024 * 4);
    bound(
        "Creole parser retained bytes",
        retained,
        budget.limits.source_bytes.saturating_mul(256),
    )?;
    bound(
        "Creole parser stack slots",
        growth * 2 + 3 * (live + 1) + 48,
        budget.limits.layout_work,
    )?;
    budget.charge(tokens.len().saturating_mul(6) + growth * 2 + 48 + 19 * (live + 4))?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RenderOptions, plantuml::style::Preamble};
    fn base() -> RunStyle {
        RunStyle {
            font: Preamble::default().fonts[0].clone(),
            underline: false,
            strike: false,
        }
    }
    #[test]
    fn should_build_generated_nested_tag_symbol_styles_hard_and_blank_lines() {
        // given
        let mut budget = Budget::new(&RenderOptions::default());
        // when
        let lines = parse(
            "<b>bold //both//</b> plain\\n\\n<u>under --strike--</u>",
            base(),
            &mut budget,
        )
        .unwrap();
        // then
        assert_eq!(lines.len(), 3);
        assert!(lines[1].is_empty());
        assert_eq!(
            lines[0]
                .iter()
                .map(|run| run.text.as_str())
                .collect::<Vec<_>>(),
            ["bold ", "both", " plain"]
        );
        assert_eq!(lines[0][0].style.font.weight, 700);
        assert!(lines[0][1].style.font.italic);
        assert!(lines[2][0].style.underline);
        assert!(lines[2][1].style.underline && lines[2][1].style.strike);
    }

    #[test]
    fn should_reject_crossed_empty_duplicate_unclosed_and_unsupported_spans() {
        // given
        for text in [
            "<b></b>",
            "x ****",
            "<b><i>x</b></i>",
            "x **//y**//",
            "<b><b>x</b></b>",
            "<b>**x**</b>",
            "x **<b>y</b>**",
            "<u>x",
            "</s>",
            "x ~~wave~~",
            "x \"\"mono\"\"",
            "<img:resource>",
            "[[link]]",
        ] {
            let mut budget = Budget::new(&RenderOptions::default());
            // when
            let result = parse(text, base(), &mut budget);
            // then
            assert!(
                matches!(result, Err(RenderError::UnsupportedFeature(_))),
                "accepted {text}"
            );
        }
    }

    #[test]
    fn should_keep_unmatched_symbols_url_escapes_and_line_local_pair_eligibility_literal() {
        // given
        for (text, expected) in [
            ("left -- right", "left -- right"),
            ("unmatched ** and __", "unmatched ** and __"),
            ("x ~~ and \"\"", "x ~~ and \"\""),
            (
                "https://host.test//path ~* ~_ ~<tag>",
                "https://host.test//path * _ <tag>",
            ),
            ("**one\\nother**", "**one\nother**"),
            ("x **a** **b**", "x a b"),
        ] {
            let mut budget = Budget::new(&RenderOptions::default());
            // when
            let lines = parse(text, base(), &mut budget).unwrap();
            let actual = lines
                .iter()
                .map(|line| line.iter().map(|run| run.text.as_str()).collect::<String>())
                .collect::<Vec<_>>()
                .join("\n");
            // then
            assert_eq!(actual, expected, "{text}");
        }
    }

    #[test]
    fn should_coalesce_equivalent_run_styles_and_restore_authored_base_font() {
        // given
        let mut budget = Budget::new(&RenderOptions::default());
        let mut base = base();
        base.font.size = 26;
        base.font.weight = 900;
        base.underline = true;
        // when
        let lines = parse("<b>one</b>**two** plain", base.clone(), &mut budget).unwrap();
        // then
        assert_eq!(lines[0].len(), 2);
        assert_eq!(lines[0][0].text, "onetwo");
        assert_eq!(lines[0][0].style.font.weight, 700);
        assert_eq!(lines[0][0].style.font.size, 26);
        assert!(lines[0][0].style.underline);
        assert_eq!(lines[0][1].style, base);
    }

    #[test]
    fn should_report_truthful_label_local_utf8_spans_without_authored_document_claims() {
        // given
        let mut budget = Budget::new(&RenderOptions::default());
        // when
        let error = parse("Ω\nx <b></i>", base(), &mut budget)
            .unwrap_err()
            .to_string();
        // then
        assert!(error.contains("label-local line 2, column 6"), "{error}");
        assert!(error.contains("label-local bytes 8..12"), "{error}");
        assert!(!error.contains("authored bytes"), "{error}");
    }

    #[test]
    fn should_reject_direct_generated_suffix_eof_and_cancel_delivery_and_scope_actions() {
        // given
        for tokens in [
            vec![(0, Token::TagBoldOpen, 3), (3, Token::Text("x".into()), 4)],
            vec![(0, Token::TagBoldClose, 4)],
            vec![
                (0, Token::BoldOpen, 2),
                (2, Token::Text("x".into()), 3),
                (3, Token::ItalicClose, 5),
            ],
        ] {
            let mut budget = Budget::new(&RenderOptions::default());
            let iterator = context::Checked::new(tokens, budget.control.clone());
            let mut builder = Builder::new(base(), &mut budget);
            // when
            let result =
                super::super::generated_creole::CreoleParser::new().parse(&mut builder, iterator);
            // then
            assert!(result.is_err());
        }
        for checkpoint in 1..=8 {
            let tokens = vec![
                (0, Token::TagBoldOpen, 3),
                (3, Token::Text("x".into()), 4),
                (4, Token::TagBoldClose, 8),
            ];
            let mut budget = Budget::new(&RenderOptions::default());
            budget.control.cancel_after_checkpoints(checkpoint);
            let iterator = context::Checked::new(tokens, budget.control.clone());
            let mut builder = Builder::new(base(), &mut budget);
            // when
            let result =
                super::super::generated_creole::CreoleParser::new().parse(&mut builder, iterator);
            // then
            assert!(
                matches!(
                    result,
                    Err(lalrpop_util::ParseError::User {
                        error: ParseFailure {
                            kind: FailureKind::Terminal(RenderError::Cancelled(_)),
                            ..
                        }
                    })
                ),
                "checkpoint {checkpoint}"
            );
        }
    }

    #[test]
    fn should_admit_exact_four_style_depth_and_bound_run_text_work_and_cancellation() {
        // given
        let text = "<b><i><u><s>x</s></u></i></b>";
        let mut budget = Budget::new(&RenderOptions::default());
        // when
        let lines = parse(text, base(), &mut budget).unwrap();
        // then
        assert_eq!(lines[0][0].style.font.weight, 700);
        assert!(
            lines[0][0].style.font.italic
                && lines[0][0].style.underline
                && lines[0][0].style.strike
        );
        for resource in ["depth", "model", "text", "work", "retained", "cancel"] {
            let mut budget = Budget::new(&RenderOptions::default());
            match resource {
                "depth" => budget.limits.nesting_depth = 3,
                "model" => budget.limits.model_items = 1,
                "text" => budget.limits.svg_bytes = 1,
                "work" => budget.limits.layout_work = 1,
                "retained" => budget.limits.source_bytes = 1,
                _ => budget.control.cancel_after_checkpoints(15),
            };
            // when
            let error = parse(text, base(), &mut budget).unwrap_err();
            // then
            if resource == "cancel" {
                assert!(matches!(error, RenderError::Cancelled(_)), "{error}");
            } else {
                assert!(
                    matches!(error, RenderError::Backend(_))
                        && error.to_string().contains("exceeds limit"),
                    "{resource}: {error}"
                );
            }
        }
    }
}
