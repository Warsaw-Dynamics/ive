//! Actual generated chart command/list/order syntax; finite literals remain lexical.
use super::{
    BodySource, context,
    error::{ParseFailure, generated_failure},
    outline,
    ownership::{Kind, Row},
    timing::diagnose,
};
use crate::{
    RenderError,
    plantuml::{
        budget::{Budget, bound},
        chart::{Builder, Chart},
        style::Color,
    },
};
#[derive(Clone, Debug)]
pub(in crate::plantuml) enum Token {
    XAxis,
    YAxis,
    Bar,
    Line,
    Legend,
    Left,
    Right,
    Top,
    Bottom,
    Labels,
    Open,
    Close,
    Comma,
    Arrow,
    Text(String),
    Number(f64),
    Color(Color),
    Unknown(String),
    End,
}
type Spanned = (usize, Token, usize);
pub(in crate::plantuml) fn parse(
    body: BodySource<'_>,
    budget: &mut Budget,
) -> Result<Chart, RenderError> {
    parse_body(&body, budget).map_err(|error| diagnose(error, &body, budget))
}
fn parse_body(body: &BodySource<'_>, budget: &mut Budget) -> Result<Chart, ParseFailure> {
    bound(
        "chart lexical retained bytes",
        body.range
            .len()
            .saturating_add(1)
            .saturating_mul(3 * std::mem::size_of::<Spanned>() + 4)
            .saturating_add(body.adapters),
        budget.limits.source_bytes.saturating_mul(256),
    )?;
    let tokens = outline::lex(body, budget, tokenize)?;
    admit((&tokens, tokens.capacity()), body, budget)?;
    let iterator = context::Checked::new(tokens, budget.control.clone());
    let mut builder = Builder::new(budget);
    super::generated_chart::ChartParser::new()
        .parse(&mut builder, iterator)
        .map_err(|error| {
            generated_failure(
                error,
                body.range.end,
                "invalid or unsupported chart command, list or suffix",
            )
        })?;
    builder.finish(body.range.end..body.range.end)
}
fn tokenize(
    row: &Row,
    base: usize,
    lexer: &mut outline::Lexer<'_, Token>,
) -> Result<(), ParseFailure> {
    if matches!(row.kind, Kind::Empty | Kind::Trivia | Kind::Definition) {
        return Ok(());
    }
    let text = &row.text;
    let mut offset = 0;
    while offset < text.len() {
        lexer.budget.charge(1)?;
        let rest = &text[offset..];
        let ch = rest.chars().next().unwrap();
        if ch.is_whitespace() {
            offset += ch.len_utf8();
            continue;
        }
        let (token, used) = match ch {
            '[' => (Token::Open, 1),
            ']' => (Token::Close, 1),
            ',' => (Token::Comma, 1),
            '"' => {
                let mut end = 1;
                while end < rest.len() && rest.as_bytes()[end] != b'"' {
                    lexer.budget.charge(1)?;
                    end += 1;
                    bound("chart quoted label bytes", end - 1, 16 * 1024)?;
                }
                if end == rest.len() {
                    return Err(ParseFailure::syntax(
                        row.span(base, offset..text.len()),
                        "unclosed chart quoted label",
                    ));
                }
                if end == 1 {
                    return Err(ParseFailure::unsupported(
                        row.span(base, offset..offset + end + 1),
                        "chart quoted label must be nonempty",
                    ));
                }
                (Token::Text(lexer.copy_label(&rest[1..end])?), end + 1)
            }
            _ if rest.starts_with("-->") => (Token::Arrow, 3),
            _ if rest.starts_with("->") => (Token::Arrow, 2),
            _ => {
                let mut end = ch.len_utf8();
                while end < rest.len() {
                    lexer.budget.charge(1)?;
                    let tail = &rest[end..];
                    let next = tail.chars().next().unwrap();
                    if next.is_whitespace()
                        || matches!(next, '[' | ']' | ',' | '"')
                        || tail.starts_with("-->")
                        || tail.starts_with("->")
                    {
                        break;
                    }
                    end += next.len_utf8();
                }
                (
                    atom(&rest[..end], row.span(base, offset..offset + end), lexer)?,
                    end,
                )
            }
        };
        lexer.push(token, row.span(base, offset..offset + used))?;
        offset += used;
    }
    lexer.push(Token::End, row.span(base, text.len()..text.len()))
}
fn atom(
    value: &str,
    span: std::ops::Range<usize>,
    lexer: &mut outline::Lexer<'_, Token>,
) -> Result<Token, ParseFailure> {
    Ok(match value {
        "x-axis" | "h-axis" => Token::XAxis,
        "y-axis" | "v-axis" => Token::YAxis,
        "bar" => Token::Bar,
        "line" => Token::Line,
        "legend" => Token::Legend,
        "left" => Token::Left,
        "right" => Token::Right,
        "top" => Token::Top,
        "bottom" => Token::Bottom,
        "labels" => Token::Labels,
        _ if value.starts_with('#') => {
            let hex = &value[1..];
            if hex.len() != 6 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
                return Err(ParseFailure::unsupported(
                    span,
                    "chart colors require #RRGGBB",
                ));
            }
            Token::Color(Color(u32::from_str_radix(hex, 16).unwrap()))
        }
        _ if value
            .as_bytes()
            .first()
            .is_some_and(|b| b.is_ascii_digit() || matches!(b, b'-' | b'+' | b'.'))
            || matches!(value, "NaN" | "Infinity" | "inf") =>
        {
            let number = value.parse::<f64>().map_err(|_| {
                ParseFailure::unsupported(span.clone(), "invalid chart numeric value")
            })?;
            if !number.is_finite() || number.abs() > 1e12 {
                return Err(ParseFailure::unsupported(
                    span,
                    "chart numeric values must be finite and within +/-1000000000000",
                ));
            }
            Token::Number(number)
        }
        _ => Token::Unknown(lexer.copy_syntax(value)?),
    })
}
fn admit(
    input: (&[Spanned], usize),
    body: &BodySource<'_>,
    budget: &mut Budget,
) -> Result<(), ParseFailure> {
    let (tokens, capacity) = input;
    // Exhaustive actual ACTION/EOF/goto witness:9 live states,3 reductions,5 expected terminals.
    let live = 12usize;
    let terminals = 19usize;
    let slots = 24usize;
    let bytes = capacity
        .saturating_mul(std::mem::size_of::<Spanned>())
        .saturating_add(body.adapters)
        .saturating_add(budget.limits.source_bytes.saturating_mul(64))
        .saturating_add(
            slots * (128 + std::mem::size_of::<usize>())
                + 3 * (live + 1) * 8
                + 24 * std::mem::size_of::<String>()
                + terminals * 32,
        );
    bound(
        "chart parser retained bytes",
        bytes,
        budget.limits.source_bytes.saturating_mul(256),
    )?;
    bound(
        "chart parser stack slots",
        2 * slots + 3 * (live + 1) + 24,
        budget.limits.layout_work,
    )?;
    budget.charge(tokens.len().saturating_mul(8) + terminals * (live + 8) + 2 * slots + 24)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        RenderOptions,
        plantuml::chart::{LegendPosition, ScaleRange, SeriesKind},
    };
    fn model(text: &str) -> Chart {
        super::super::test_support::body(
            (text, "chart"),
            &mut Budget::new(&RenderOptions::default()),
            parse,
        )
        .unwrap()
    }

    #[test]
    fn should_keep_order_duplicate_display_identities_default_range_and_expand_only() {
        // given
        let chart = model(
            "legend right\nx-axis \"Week\" [\"b\",\"a\",\"b\"]\nline [3,5,4]\nbar \"same\" [0,20,40] labels\nline \"same\" [-25,150,10]",
        );
        // when
        let range = chart.value_axis.range.bounds();
        // then
        assert_eq!(chart.category_axis.categories, ["b", "a", "b"]);
        assert_eq!(
            chart
                .series
                .iter()
                .map(|s| s.name.as_str())
                .collect::<Vec<_>>(),
            ["line0", "same", "same"]
        );
        assert_eq!(chart.series[1].kind, SeriesKind::Bar);
        assert_eq!(chart.legend, LegendPosition::Right);
        assert_eq!(range, (-25., 150.));
        assert!(matches!(
            model("x-axis [\"a\"]\nbar [3]").value_axis.range,
            ScaleRange::Auto(0., 100.)
        ));
    }

    #[test]
    fn should_apply_fixed_range_and_title_in_authored_order_and_reject_missing_samples() {
        // given
        let chart =
            model("x-axis [\"a\",\"b\"]\nbar [-10,20]\ny-axis -10-->20\ny-axis \"Measured\"");
        // when
        let range = chart.value_axis.range.bounds();
        // then
        assert_eq!(range, (-10., 20.));
        assert_eq!(chart.value_axis.title, "Measured");
        for text in [
            "x-axis [\"a\"]\nbar [101]\ny-axis 0-->100",
            "x-axis [\"a\",\"b\"]\nline [1]",
            "x-axis [\"a\"]\nbar [1]\nbar [2]",
        ] {
            assert!(
                super::super::test_support::body(
                    (text, "chart"),
                    &mut Budget::new(&RenderOptions::default()),
                    parse
                )
                .is_err(),
                "{text}"
            );
        }
    }

    #[test]
    fn should_reject_direct_generated_list_suffix_eof_and_cancel_delivery_and_actions() {
        // given
        for tokens in [
            vec![
                (0, Token::Bar, 3),
                (4, Token::Open, 5),
                (5, Token::Number(1.), 6),
            ],
            vec![(0, Token::Legend, 6), (7, Token::Right, 12)],
            vec![(0, Token::Close, 1)],
        ] {
            let mut budget = Budget::new(&RenderOptions::default());
            let mut builder = Builder::new(&mut budget);
            // when
            let result = super::super::generated_chart::ChartParser::new()
                .parse(&mut builder, tokens.into_iter().map(Ok));
            // then
            assert!(result.is_err());
        }
        for checkpoint in 1..=8 {
            let mut budget = Budget::new(&RenderOptions::default());
            budget.control.cancel_after_checkpoints(checkpoint);
            let tokens = vec![
                (0, Token::Bar, 3),
                (3, Token::Open, 4),
                (4, Token::Number(1.), 5),
                (5, Token::Close, 6),
                (6, Token::End, 6),
            ];
            let iterator = context::Checked::new(tokens, budget.control.clone());
            let mut builder = Builder::new(&mut budget);
            // when
            let result =
                super::super::generated_chart::ChartParser::new().parse(&mut builder, iterator);
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
                "{checkpoint}"
            );
        }
    }

    #[test]
    fn should_cancel_mid_quote_and_refuse_generated_retention_before_entry() {
        // given
        let text = format!("x-axis [\"{}\"]\nbar [1]", "x".repeat(4000));
        let mut budget = Budget::new(&RenderOptions::default());
        let document = super::super::test_support::document(
            &format!("@startchart\n{text}\n@endchart"),
            &mut budget,
        )
        .unwrap();
        let mut prepared = super::super::document::parse(&document, &mut budget).unwrap();
        super::super::preamble::parse(&mut prepared, &mut budget).unwrap();
        budget.control.cancel_after_checkpoints(30);
        // when
        let error = match parse_body(&prepared.body, &mut budget) {
            Err(error) => error,
            Ok(_) => panic!("quote cancellation ignored"),
        };
        // then
        assert!(matches!(
            error.kind,
            super::super::error::FailureKind::Terminal(RenderError::Cancelled(_))
        ));
        let mut budget = Budget::new(&RenderOptions::default());
        budget.limits.source_bytes = 1;
        assert!(
            admit(
                (&[(0, Token::Text("x".into()), 1)], 1),
                &prepared.body,
                &mut budget
            )
            .is_err()
        );
        let text = format!(
            "x-axis [{}]\nline [1]\nline [2]\nline [3]",
            vec!["\"a\""; 100].join(",")
        );
        let mut budget = Budget::new(&RenderOptions::default());
        budget.limits.model_items = 200;
        let error = super::super::test_support::body((&text, "chart"), &mut budget, parse)
            .err()
            .expect("excess sample product accepted");
        assert!(
            error.to_string().contains("chart sample product"),
            "{error}"
        );
        assert!(
            std::mem::size_of::<crate::plantuml::chart::Command>()
                + 3 * std::mem::size_of::<usize>()
                <= 128
        );
        for length in [16 * 1024, 16 * 1024 + 1] {
            let text = format!("x-axis [\"{}\"]\nline [1]", "x".repeat(length));
            let result = super::super::test_support::body(
                (&text, "chart"),
                &mut Budget::new(&RenderOptions::default()),
                parse,
            );
            assert_eq!(result.is_ok(), length == 16 * 1024);
        }
        let exact = "@startchart\nx-axis [\"a\"]\nline [1]\n@endchart";
        let mut budget = Budget::new(&RenderOptions::default());
        budget.limits.source_bytes = exact.len();
        budget.limits.nesting_depth = 1;
        assert!(super::super::test_support::parse(exact, &mut budget, parse).is_ok());
    }
}
