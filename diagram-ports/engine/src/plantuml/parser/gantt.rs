//! Generated day-only Gantt syntax and finite lexical date/number values.
use super::{
    BodySource, context,
    error::{ParseFailure, generated_failure},
    timing::diagnose,
};
use crate::{
    RenderError,
    plantuml::{
        budget::{Budget, bound},
        gantt::{Builder, CivilDay, Endpoint, Gantt, Kind, MAX_SPAN},
        source::Line,
    },
};
#[derive(Clone, Debug)]
pub(in crate::plantuml) enum Schedule {
    Date(CivilDay),
    Reference {
        code: String,
        endpoint: Endpoint,
        offset: i32,
    },
}
pub(in crate::plantuml) enum Command {
    Duration(i32),
    Start(Kind, Schedule),
}
#[derive(Clone, Debug)]
pub(in crate::plantuml) enum Token {
    Project,
    Starts,
    Happens,
    Lasts,
    Requires,
    As,
    On,
    At,
    After,
    Before,
    With,
    Day,
    Start,
    End,
    Possessive,
    Bracket(String),
    Date(CivilDay),
    Number(i32),
    Unknown,
    Newline(usize),
}
type Spanned = (usize, Token, usize);
pub(in crate::plantuml) fn duration(
    days: i32,
    span: std::ops::Range<usize>,
) -> Result<Command, ParseFailure> {
    if days == 0 {
        return Err(ParseFailure::unsupported(
            span,
            "positive whole-day duration required",
        ));
    }
    Ok(Command::Duration(days))
}
pub(in crate::plantuml) fn parse(
    body: BodySource<'_>,
    budget: &mut Budget,
) -> Result<Gantt, RenderError> {
    parse_body(&body, budget).map_err(|error| diagnose(error, &body, budget))
}
fn parse_body(body: &BodySource<'_>, budget: &mut Budget) -> Result<Gantt, ParseFailure> {
    bound(
        "Gantt source bytes",
        body.range.len(),
        budget.limits.source_bytes,
    )?;
    budget.depth(1)?;
    let mut tokens = Vec::new();
    let mut ownership = body.ownership;
    let mut base = body.range.start;
    budget.charge(base)?;
    let mut line = body.document.text[..base]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count()
        + 1;
    for physical in body.document.text[body.range.clone()].split_inclusive('\n') {
        let raw = physical
            .strip_suffix('\n')
            .map_or(physical, |row| row.strip_suffix('\r').unwrap_or(row));
        let row = ownership.classify(raw, budget)?;
        ownership.advance(&row, &row.text, budget)?;
        lex(&row, (base, line), (&mut tokens, budget))?;
        base += physical.len();
        line += 1;
    }
    ownership.finish(base)?;
    admit((&tokens, tokens.capacity()), body.adapters, budget)?;
    let iterator = context::Checked::new(tokens, budget.control.clone());
    let mut builder = Builder::new(budget);
    super::generated_gantt::GanttParser::new().parse(&mut builder,iterator).map_err(|error|generated_failure(error,body.range.end,"Gantt requires Project starts YYYY-MM-DD; invalid or incomplete project/task syntax"))?;
    builder.finish(super::timing::Site {
        span: body.range.end..body.range.end,
        line,
    })
}
fn lex(
    row: &super::ownership::Row,
    location: (usize, usize),
    output: (&mut Vec<Spanned>, &mut Budget),
) -> Result<(), ParseFailure> {
    let (base, line) = location;
    let (tokens, budget) = output;
    let mut cursor = 0;
    while cursor < row.text.len() {
        let rest = &row.text[cursor..];
        let ch = rest.chars().next().unwrap();
        budget.charge(ch.len_utf8())?;
        if ch.is_whitespace() {
            cursor += ch.len_utf8();
            continue;
        }
        let (token, used) = if ch == '[' {
            let mut end = None;
            for (index, ch) in rest.char_indices().skip(1) {
                budget.charge(ch.len_utf8())?;
                bound("task identity bytes", index - 1, 16 * 1024)?;
                if ch == '[' {
                    return Err(ParseFailure::unsupported(
                        row.span(base, cursor..cursor + index + 1),
                        "nested Gantt task bracket",
                    ));
                }
                if ch == ']' {
                    end = Some(index);
                    break;
                }
            }
            let end = end.filter(|index| *index > 1).ok_or_else(|| {
                ParseFailure::syntax(
                    row.span(base, cursor..row.text.len()),
                    "empty or unclosed Gantt task bracket",
                )
            })?;
            budget.charge(end - 1)?;
            (Token::Bracket(rest[1..end].to_owned()), end + 1)
        } else if rest.starts_with("'s") {
            (Token::Possessive, 2)
        } else {
            let mut used = ch.len_utf8();
            for (index, ch) in rest.char_indices().skip(1) {
                if ch.is_whitespace() || matches!(ch, '[' | ']' | '\'') {
                    break;
                }
                budget.charge(ch.len_utf8())?;
                used = index + ch.len_utf8();
            }
            let value = &rest[..used];
            if matches!(value, "as" | "starts" | "happens" | "lasts" | "requires")
                && !rest[used..].starts_with(char::is_whitespace)
            {
                return Err(ParseFailure::unsupported(
                    row.span(base, cursor..cursor + used),
                    "Gantt alias and verb require following whitespace",
                ));
            }
            let token = match value {
                word if word.eq_ignore_ascii_case("project") => Token::Project,
                "starts" => Token::Starts,
                "happens" => Token::Happens,
                "lasts" => Token::Lasts,
                "requires" => Token::Requires,
                "as" => Token::As,
                "on" => Token::On,
                "at" => Token::At,
                "after" => Token::After,
                "before" => Token::Before,
                "with" => Token::With,
                "day" | "days" => Token::Day,
                "start" => Token::Start,
                "end" => Token::End,
                _ if value.contains('-')
                    && value
                        .bytes()
                        .next()
                        .is_some_and(|byte| byte.is_ascii_digit()) =>
                {
                    let diagnostic = Line {
                        number: line,
                        text: String::new(),
                    };
                    let day = CivilDay::parse(value, &diagnostic).map_err(|_| {
                        ParseFailure::syntax(
                            row.span(base, cursor..cursor + used),
                            "invalid explicit ISO civil date",
                        )
                    })?;
                    Token::Date(day)
                }
                _ if value.bytes().all(|byte| byte.is_ascii_digit()) => {
                    if value.len() > 4 {
                        return Err(ParseFailure::syntax(
                            row.span(base, cursor..cursor + used),
                            "bounded whole-day number required",
                        ));
                    }
                    let value = value.parse::<i32>().map_err(|_| {
                        ParseFailure::syntax(
                            row.span(base, cursor..cursor + used),
                            "bounded whole-day number required",
                        )
                    })?;
                    if value > MAX_SPAN {
                        return Err(ParseFailure::syntax(
                            row.span(base, cursor..cursor + used),
                            "day value exceeds supported range",
                        ));
                    }
                    Token::Number(value)
                }
                _ => Token::Unknown,
            };
            (token, used)
        };
        let span = row.span(base, cursor..cursor + used);
        context::push(tokens, (span.start, token, span.end), budget)?;
        cursor += used;
    }
    if !row.text.trim().is_empty() {
        let span = row.span(base, row.text.len()..row.text.len());
        context::push(tokens, (span.start, Token::Newline(line), span.end), budget)?;
    }
    Ok(())
}
fn admit(
    input: (&[Spanned], usize),
    adapters: usize,
    budget: &mut Budget,
) -> Result<(), ParseFailure> {
    let (tokens, capacity) = input;
    let mut literals = 0usize;
    let mut items = 0usize;
    for (_, token, _) in tokens {
        budget.charge(1)?;
        if let Token::Bracket(value) = token {
            literals += value.capacity();
        }
        items = items.saturating_add(match token {
            Token::Lasts | Token::Requires | Token::Starts | Token::Happens => 2,
            _ => 0,
        });
    }
    // Exhaustive actual tables: 12 live states, four reductions, 20 terminals.
    // Capacity16/growth24 covers generated state and symbol vectors; diagnostic
    // copies, expected String slots and every terminal probe are included separately.
    let symbol = std::mem::size_of::<Token>()
        + std::mem::size_of::<Schedule>()
        + std::mem::size_of::<Command>()
        + std::mem::size_of::<String>()
        + 4 * std::mem::size_of::<usize>();
    let slots = 48 + 48 + 48;
    let retained = capacity
        .saturating_mul(std::mem::size_of::<Spanned>())
        .saturating_add(literals)
        .saturating_add(adapters)
        .saturating_add(
            24 * symbol
                + 24 * std::mem::size_of::<i16>()
                + 48 * std::mem::size_of::<String>()
                + 20 * 32,
        )
        .saturating_add(std::mem::size_of::<Builder<'_>>())
        .saturating_add(Builder::retained_bytes(
            items.min(budget.limits.model_items),
        ))
        .saturating_add(budget.limits.source_bytes.saturating_mul(16));
    bound(
        "gantt parser retained bytes",
        retained,
        budget.limits.source_bytes.saturating_mul(256),
    )?;
    bound("gantt parser stack slots", slots, budget.limits.layout_work)?;
    budget.charge(
        tokens
            .len()
            .saturating_mul(5)
            .saturating_add(slots + 4 + 20 * (12 + 4)),
    )?;
    Ok(())
}

#[cfg(test)]
pub(in crate::plantuml) fn parse_test(
    lines: &[Line],
    budget: &mut Budget,
) -> Result<Gantt, RenderError> {
    let text = format!(
        "@startgantt\n{}\n@endgantt",
        lines
            .iter()
            .map(|line| line.text.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    );
    let document = crate::plantuml::preprocess::prepare(&text, budget)?;
    let mut prepared = super::document::parse(&document, budget)?;
    super::preamble::parse(&mut prepared, budget)?;
    parse(prepared.body, budget)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::RenderOptions;
    #[test]
    fn should_build_generated_aliases_half_open_dependencies_and_milestones() {
        // given
        let mut budget = Budget::new(&RenderOptions::default());
        let document=crate::plantuml::preprocess::prepare("@startgantt\nProject starts 2028-02-28\n[Alpha] as [A] lasts 3 days\n[B] starts at [A]'s end\n[M] happens 1 day after [A]'s end\n@endgantt", &mut budget).unwrap();
        let mut prepared = super::super::document::parse(&document, &mut budget).unwrap();
        super::super::preamble::parse(&mut prepared, &mut budget).unwrap();
        // when
        let result = parse(prepared.body, &mut budget).unwrap();
        // then
        assert_eq!(result.tasks[0].display, "Alpha");
        assert_eq!(result.tasks[1].start, result.tasks[0].end);
        assert_eq!(result.tasks[2].start, result.tasks[0].end);
        assert_eq!(result.dependencies.len(), 2);
    }

    #[test]
    fn should_reject_attached_gantt_alias_and_verb_primitives() {
        // given
        for row in [
            "[A]as[B] lasts 1 day",
            "[A] starts[Z]'s end",
            "[A] happens[Z]'s end",
        ] {
            let mut budget = Budget::new(&RenderOptions::default());
            let document = crate::plantuml::preprocess::prepare(
                &format!(
                    "@startgantt\nProject starts 2026-01-01\n[Z] lasts 1 day\n{row}\n@endgantt"
                ),
                &mut budget,
            )
            .unwrap();
            let mut prepared = super::super::document::parse(&document, &mut budget).unwrap();
            super::super::preamble::parse(&mut prepared, &mut budget).unwrap();
            // when
            let result = parse(prepared.body, &mut budget);
            // then
            assert!(result.is_err(), "accepted {row}");
        }
    }

    #[test]
    fn should_reject_generated_gantt_eof_and_cancel_delivery_and_actions() {
        // given
        for tokens in [
            vec![(0, Token::Project, 7)],
            vec![
                (0, Token::Project, 7),
                (8, Token::Starts, 14),
                (15, Token::Date(CivilDay(0)), 25),
            ],
        ] {
            let mut budget = Budget::new(&RenderOptions::default());
            admit((&tokens, tokens.capacity()), 0, &mut budget).unwrap();
            let iterator = context::Checked::new(tokens, budget.control.clone());
            let mut builder = Builder::new(&mut budget);
            // when
            let result =
                super::super::generated_gantt::GanttParser::new().parse(&mut builder, iterator);
            // then
            assert!(result.is_err());
        }
        for checkpoint in 1..=6 {
            let tokens = vec![
                (0, Token::Project, 7),
                (8, Token::Starts, 14),
                (15, Token::Date(CivilDay(0)), 25),
                (25, Token::Newline(1), 26),
            ];
            let mut budget = Budget::new(&RenderOptions::default());
            admit((&tokens, tokens.capacity()), 0, &mut budget).unwrap();
            budget.control.cancel_after_checkpoints(checkpoint);
            let iterator = context::Checked::new(tokens, budget.control.clone());
            let mut builder = Builder::new(&mut budget);
            // when
            let result =
                super::super::generated_gantt::GanttParser::new().parse(&mut builder, iterator);
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
                "checkpoint {checkpoint}"
            );
        }
    }

    #[test]
    fn should_map_gantt_semantic_and_syntax_failures_with_full_statement_once() {
        // given
        for command in ["[A] lasts 2 days extra", "[B] starts at [missing]'s end"] {
            let mut budget = Budget::new(&RenderOptions::default());
            let document = crate::plantuml::preprocess::prepare(
                &format!(
                    "@startgantt\n!define BAD {command}\nProject starts 2026-01-01\nBAD\n@endgantt"
                ),
                &mut budget,
            )
            .unwrap();
            let mut prepared = super::super::document::parse(&document, &mut budget).unwrap();
            super::super::preamble::parse(&mut prepared, &mut budget).unwrap();
            // when
            let error = match parse(prepared.body, &mut budget) {
                Ok(_) => panic!("accepted bad statement"),
                Err(error) => error.to_string(),
            };
            // then
            assert!(error.contains(command), "{error}");
            assert_eq!(error.matches("expansion \"BAD\"").count(), 1, "{error}");
        }
    }

    #[test]
    fn should_bound_gantt_tokens_depth_work_and_mid_lex_cancellation() {
        // given
        for resource in ["source", "depth", "work", "cancel"] {
            let mut budget = Budget::new(&RenderOptions::default());
            let document = crate::plantuml::preprocess::prepare(
                "@startgantt\nProject starts 2026-01-01\n[A] lasts 2 days\n@endgantt",
                &mut budget,
            )
            .unwrap();
            let mut prepared = super::super::document::parse(&document, &mut budget).unwrap();
            super::super::preamble::parse(&mut prepared, &mut budget).unwrap();
            match resource {
                "source" => budget.limits.source_bytes = 4,
                "depth" => budget.limits.nesting_depth = 0,
                "work" => budget.limits.layout_work = budget.work + 1,
                _ => budget.control.cancel_after_checkpoints(12),
            };
            // when
            let error = match parse(prepared.body, &mut budget) {
                Ok(_) => panic!("accepted {resource}"),
                Err(error) => error,
            };
            // then
            if resource == "cancel" {
                assert!(matches!(error, RenderError::Cancelled(_)), "{error}");
            } else {
                assert!(
                    matches!(error, RenderError::Backend(_))
                        && error.to_string().contains("exceeds limit"),
                    "{error}"
                );
            }
        }
    }
}
