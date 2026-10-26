//! Generated raw ASCII Git rows, commit names and parenthesized literal comments.
use super::{
    BodySource, context,
    error::{ParseFailure, generated_failure},
};
use crate::{
    RenderError,
    plantuml::{
        budget::{Budget, bound},
        git::{Builder, Cell, Comment, GitHistory, Kind, Name, RowInfo},
    },
};
use std::ops::Range;

#[derive(Clone, Debug)]
pub(in crate::plantuml) enum Token {
    Blank,
    Pipe(Cell),
    Slash(Cell),
    Backslash(Cell),
    Star(usize),
    Name(Name),
    Comment(String),
    Newline(RowInfo),
}
type Spanned = (usize, Token, usize);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Grid,
    Named,
}

pub(in crate::plantuml) fn parse(
    body: BodySource<'_>,
    budget: &mut Budget,
) -> Result<GitHistory, RenderError> {
    parse_body(&body, budget).map_err(|failure| failure.into_document(body.document, budget))
}
fn parse_body(body: &BodySource<'_>, budget: &mut Budget) -> Result<GitHistory, ParseFailure> {
    bound(
        "Git body bytes",
        body.range.len(),
        budget.limits.source_bytes,
    )?;
    budget.depth(1)?;
    let mut tokens = Vec::new();
    let mut base = body.range.start;
    for physical in body.document.text[body.range.clone()].split_inclusive('\n') {
        let raw = physical.strip_suffix('\n').unwrap_or(physical);
        let raw = raw.strip_suffix('\r').unwrap_or(raw);
        row(raw, base, (&mut tokens, budget))?;
        push(
            &mut tokens,
            (
                base + raw.len(),
                Token::Newline(RowInfo {
                    base,
                    width: raw.len(),
                }),
                base + physical.len(),
            ),
            budget,
        )?;
        base += physical.len();
    }
    let retained = admit((&tokens, tokens.capacity()), body, budget)?;
    let iterator = context::Checked::new(tokens, budget.control.clone());
    let mut builder = Builder::new(budget, retained);
    super::generated_git::HistoryParser::new()
        .parse(&mut builder, iterator)
        .map_err(|error| {
            generated_failure(
                error,
                body.range.end,
                "invalid Git row, commit name, comment or suffix",
            )
        })?;
    builder.finish()
}
fn row(
    raw: &str,
    base: usize,
    output: (&mut Vec<Spanned>, &mut Budget),
) -> Result<(), ParseFailure> {
    let (tokens, budget) = output;
    let mut cursor = 0;
    let mut phase = Phase::Grid;
    while cursor < raw.len() {
        budget.charge(1)?;
        let start = cursor;
        let ch = raw[cursor..].chars().next().unwrap();
        let token = match ch {
            ' ' => {
                while raw.as_bytes().get(cursor) == Some(&b' ') {
                    budget.charge(1)?;
                    cursor += 1;
                }
                Token::Blank
            }
            '(' if phase == Phase::Named => {
                cursor += 1;
                let content = cursor;
                while cursor < raw.len() && raw.as_bytes()[cursor] != b')' {
                    budget.charge(1)?;
                    if raw.as_bytes()[cursor] == b'(' {
                        return Err(ParseFailure::unsupported(
                            base + cursor..base + cursor + 1,
                            "nested Git comments are unsupported",
                        ));
                    }
                    cursor += raw[cursor..].chars().next().unwrap().len_utf8();
                }
                if cursor == raw.len() {
                    return Err(ParseFailure::syntax(
                        base + start..base + cursor,
                        "unterminated Git comment",
                    ));
                }
                let comment = copy(&raw[content..cursor], base + content..base + cursor, budget)?;
                cursor += 1;
                Token::Comment(comment)
            }
            '|' | '/' | '\\' if phase == Phase::Grid => {
                cursor += 1;
                let kind = match ch {
                    '|' => Kind::Pipe,
                    '/' => Kind::Slash,
                    _ => Kind::Backslash,
                };
                let cell = Cell::new(start, kind);
                match kind {
                    Kind::Pipe => Token::Pipe(cell),
                    Kind::Slash => Token::Slash(cell),
                    Kind::Backslash => Token::Backslash(cell),
                }
            }
            '*' if phase == Phase::Grid => {
                cursor += 1;
                Token::Star(start)
            }
            '-' | '_' | '.' if phase == Phase::Grid => {
                return Err(ParseFailure::unsupported(
                    base + start..base + start + 1,
                    "horizontal, underscore and octopus Git connectors are unsupported",
                ));
            }
            '\t' | ')' | '*' => {
                return Err(ParseFailure::unsupported(
                    base + start..base + start + ch.len_utf8(),
                    "unexpected Git grid marker, tab or comment suffix",
                ));
            }
            _ if phase == Phase::Grid => {
                while cursor < raw.len() && !matches!(raw.as_bytes()[cursor], b' ' | b'(' | b')') {
                    budget.charge(1)?;
                    let value = raw[cursor..].chars().next().unwrap();
                    if value.is_control() || value == '*' {
                        return Err(ParseFailure::unsupported(
                            base + cursor..base + cursor + value.len_utf8(),
                            "control character or extra Git commit marker",
                        ));
                    }
                    cursor += value.len_utf8();
                }
                let raw_name = &raw[start..cursor];
                let mut text = copy(raw_name, base + start..base + cursor, budget)?;
                budget.charge(text.len())?;
                text.retain(|ch| !matches!(ch, '-' | '.' | '|' | '/' | '\\'));
                if text.is_empty() {
                    return Err(ParseFailure::syntax(
                        base + start..base + cursor,
                        "empty Git commit name",
                    ));
                }
                phase = Phase::Named;
                Token::Name(Name {
                    text,
                    column: start,
                })
            }
            _ => {
                return Err(ParseFailure::unsupported(
                    base + start..base + raw.len(),
                    "unexpected substantive Git row suffix",
                ));
            }
        };
        push(tokens, (base + start, token, base + cursor), budget)?;
    }
    Ok(())
}
fn push(
    tokens: &mut Vec<Spanned>,
    token: Spanned,
    budget: &mut Budget,
) -> Result<(), ParseFailure> {
    bound(
        "Git grammar tokens",
        tokens.len().saturating_add(1),
        budget
            .limits
            .model_items
            .saturating_mul(8)
            .saturating_add(16),
    )?;
    context::push(tokens, token, budget)
}
pub(in crate::plantuml) fn copy(
    text: &str,
    span: Range<usize>,
    budget: &mut Budget,
) -> Result<String, ParseFailure> {
    bound("label bytes", text.len(), 16 * 1024)?;
    budget.text(text.len())?;
    for ch in text.chars() {
        budget.charge(ch.len_utf8())?;
        if ch.is_control() || matches!(ch, '\u{fffe}' | '\u{ffff}') {
            return Err(ParseFailure::unsupported(
                span,
                "Git label control character",
            ));
        }
    }
    let mut result = String::new();
    result.try_reserve_exact(text.len()).map_err(|error| {
        ParseFailure::terminal(RenderError::Backend(format!(
            "Git literal allocation for {} bytes failed: {error}",
            text.len()
        )))
    })?;
    result.push_str(text);
    Ok(result)
}
fn admit(
    input: (&[Spanned], usize),
    body: &BodySource<'_>,
    budget: &mut Budget,
) -> Result<usize, ParseFailure> {
    let (tokens, capacity) = input;
    let mut literals = 0usize;
    for (_, token, _) in tokens {
        budget.charge(1)?;
        literals = literals.saturating_add(match token {
            Token::Name(name) => name.text.capacity(),
            Token::Comment(text) => text.capacity(),
            _ => 0,
        });
    }
    // The generated left-recursive row/cell lists have a constant live stack.
    let (live, terminals, reductions) = (10usize, 8usize, 4usize);
    let stack = live.next_power_of_two().max(8);
    let slots = (stack + stack / 2) * 2 + 3 * (live + 1) + 3 * terminals.next_power_of_two();
    bound("Git parser stack slots", slots, budget.limits.layout_work)?;
    budget.charge(
        tokens
            .len()
            .saturating_mul(reductions + 1)
            .saturating_add(slots + terminals * (live + reductions))
            .saturating_add(literals.saturating_mul(3)),
    )?;
    let symbol = std::mem::size_of::<Token>()
        .max(std::mem::size_of::<Vec<Cell>>())
        .max(std::mem::size_of::<Comment>())
        + std::mem::size_of::<usize>();
    let bytes = capacity
        .saturating_mul(std::mem::size_of::<Spanned>())
        .saturating_add(literals.saturating_mul(4))
        .saturating_add(body.adapters)
        .saturating_add((stack + stack / 2) * (symbol + 2 * std::mem::size_of::<usize>() + 2))
        .saturating_add(
            3 * (live + 1) * 2
                + 3 * terminals.next_power_of_two() * std::mem::size_of::<String>()
                + terminals * 32,
        )
        .saturating_add(
            std::mem::size_of::<Builder<'_>>()
                + std::mem::size_of::<crate::plantuml::git::CommitRow>(),
        );
    bound(
        "Git parser retained bytes",
        bytes,
        budget.limits.source_bytes.saturating_mul(256),
    )?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RenderOptions, plantuml::parser::error::FailureKind};

    #[test]
    fn should_cancel_long_raw_grid_lexing_before_retaining_a_named_token() {
        // given
        let raw = format!("{}* tip", " ".repeat(16_000));
        let mut budget = Budget::new(&RenderOptions::default());
        let mut tokens = Vec::new();
        budget.control.cancel_after_checkpoints(20);
        // when
        let result = row(&raw, 0, (&mut tokens, &mut budget));
        // then
        assert!(matches!(
            result,
            Err(ParseFailure {
                kind: FailureKind::Terminal(RenderError::Cancelled(_)),
                ..
            })
        ));
        assert!(tokens.is_empty());
        assert!(budget.work > 0 && budget.work < raw.len());
    }

    #[test]
    fn should_reject_generated_suffixes_and_cancel_generated_delivery_and_eof() {
        // given
        let info = RowInfo { base: 0, width: 3 };
        let tokens = vec![
            (0, Token::Star(0), 1),
            (1, Token::Blank, 2),
            (
                2,
                Token::Name(Name {
                    text: "a".into(),
                    column: 2,
                }),
                3,
            ),
            (3, Token::Newline(info), 4),
        ];
        let mut budget = Budget::new(&RenderOptions::default());
        let mut invalid = tokens.clone();
        invalid.push((
            4,
            Token::Name(Name {
                text: "suffix".into(),
                column: 4,
            }),
            10,
        ));
        // when
        let mut builder = Builder::new(&mut budget, 0);
        let suffix = super::super::generated_git::HistoryParser::new().parse(
            &mut builder,
            context::Checked::new(invalid, crate::RenderControl::default()),
        );
        // then
        assert!(suffix.is_err());
        for checkpoints in [0, 1, 2, 3, 4] {
            let options = RenderOptions::default();
            options.control.cancel_after_checkpoints(checkpoints);
            let mut budget = Budget::new(&options);
            let mut builder = Builder::new(&mut budget, 0);
            let result = super::super::generated_git::HistoryParser::new().parse(
                &mut builder,
                context::Checked::new(tokens.clone(), options.control.clone()),
            );
            assert!(
                result.is_err(),
                "delivery/EOF ignored checkpoint {checkpoints}"
            );
        }
    }
}
