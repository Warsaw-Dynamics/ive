//! Generated timing commands; raw source is consulted only when enriching a failure.
use super::{
    BodySource, context,
    error::{FailureKind, ParseFailure, generated_failure},
    ownership::Row,
};
use crate::{
    RenderError,
    plantuml::{
        budget::{Budget, bound},
        timing::{Builder, TIME_LIMIT, Timing, TrackKind},
    },
};
use std::ops::Range;
#[derive(Clone, Debug)]
pub(in crate::plantuml) struct Site {
    pub span: Range<usize>,
    pub line: usize,
}
pub(in crate::plantuml) fn fail(site: &Site, detail: &'static str) -> ParseFailure {
    ParseFailure::syntax(site.span.clone(), detail)
}
#[derive(Clone, Debug)]
pub(in crate::plantuml) struct Atom {
    pub text: String,
    pub gap: bool,
}
#[derive(Clone, Copy, Debug)]
pub(in crate::plantuml) struct Time {
    pub value: i64,
    pub relative: bool,
}
impl Time {
    pub fn resolve(self, base: Option<i64>, site: &Site) -> Result<i64, ParseFailure> {
        let value = if self.relative {
            base.unwrap_or(0).checked_add(self.value)
        } else {
            Some(self.value)
        };
        value
            .filter(|value| (-TIME_LIMIT..=TIME_LIMIT).contains(value))
            .ok_or_else(|| fail(site, "timing integer exceeds supported range"))
    }
}
#[derive(Clone, Debug)]
pub(in crate::plantuml) enum TimeChoice {
    Current,
    Explicit(Time),
}
#[derive(Clone, Debug)]
pub(in crate::plantuml) enum Target {
    Player(String),
    Time(Time),
}
pub(in crate::plantuml) enum Command {
    Declare(TrackKind, String, String),
    Player(String),
    Mark(Time),
    Change(Target, String),
    Constraint(String, Time, Time, String),
    Message((String, TimeChoice), (String, TimeChoice), String),
}
#[derive(Clone, Debug)]
pub(in crate::plantuml) enum Token {
    Robust(Atom),
    Concise(Atom),
    As(Atom),
    Is(Atom),
    Code(Atom),
    Numeric(Atom),
    Other(Atom),
    Quoted(String),
    At,
    Arrow,
    Constraint,
    Colon,
    Label(String),
    Newline(usize),
}
type Spanned = (usize, Token, usize);
pub(in crate::plantuml) fn code(atom: Atom, span: Range<usize>) -> Result<String, ParseFailure> {
    if is_code(&atom.text) {
        return Ok(atom.text);
    }
    Err(ParseFailure::unsupported(span, "timing track identifier"))
}
fn is_code(value: &str) -> bool {
    let mut chars = value.chars();
    chars
        .next()
        .is_some_and(|ch| ch.is_alphabetic() || ch == '_')
        && chars.all(|ch| ch.is_alphanumeric() || matches!(ch, '_' | '.'))
}
pub(in crate::plantuml) fn time(atom: Atom, span: Range<usize>) -> Result<Time, ParseFailure> {
    let raw = atom.text.as_str();
    let (relative, number) = raw
        .strip_prefix('+')
        .map_or((false, raw), |value| (true, value));
    let digits = number.strip_prefix('-').unwrap_or(number);
    if raw.len() > 12 || digits.is_empty() || !digits.bytes().all(|ch| ch.is_ascii_digit()) {
        return Err(ParseFailure::unsupported(
            span,
            "timing requires exact integer times",
        ));
    }
    let value = number
        .parse::<i64>()
        .map_err(|_| ParseFailure::syntax(span.clone(), "invalid timing integer"))?;
    if !(-TIME_LIMIT..=TIME_LIMIT).contains(&value) {
        return Err(ParseFailure::syntax(
            span,
            "timing integer exceeds supported range",
        ));
    }
    Ok(Time { value, relative })
}
pub(in crate::plantuml) enum StateForm {
    Bare,
    Quoted,
}
pub(in crate::plantuml) fn state(
    value: String,
    form: StateForm,
    span: Range<usize>,
) -> Result<String, ParseFailure> {
    bound(
        "label bytes",
        value.len()
            + if matches!(form, StateForm::Quoted) {
                2
            } else {
                0
            },
        16 * 1024,
    )?;
    if value.is_empty() || value.starts_with('{') {
        return Err(ParseFailure::unsupported(
            span,
            "timing special/empty state",
        ));
    }
    if matches!(form, StateForm::Quoted) {
        return Ok(value);
    }
    let mut chars = value.chars();
    if chars
        .next()
        .is_some_and(|ch| ch.is_alphanumeric() || matches!(ch, '_' | '-'))
        && chars.all(|ch| ch.is_alphanumeric() || matches!(ch, '_' | '-' | '.'))
    {
        return Ok(value);
    }
    Err(ParseFailure::unsupported(span, "timing state"))
}
pub(in crate::plantuml) fn declare(
    kind: (TrackKind, Atom),
    names: (String, String),
    span: Range<usize>,
) -> Result<Command, ParseFailure> {
    bound("label bytes", names.1.len(), 16 * 1024)?;
    if !kind.1.gap || names.1.is_empty() {
        return Err(ParseFailure::unsupported(span, "timing track declaration"));
    }
    Ok(Command::Declare(kind.0, names.0, names.1))
}
pub(in crate::plantuml) fn check_is(atom: Atom, span: Range<usize>) -> Result<(), ParseFailure> {
    if atom.gap {
        return Ok(());
    }
    Err(ParseFailure::unsupported(
        span,
        "timing state requires whitespace after is",
    ))
}
pub(in crate::plantuml) fn parse(
    body: BodySource<'_>,
    budget: &mut Budget,
) -> Result<Timing, RenderError> {
    parse_body(&body, budget).map_err(|error| diagnose(error, &body, budget))
}
pub(in crate::plantuml) fn named_at(
    name: String,
    boundary: (usize, usize),
) -> Result<String, ParseFailure> {
    if boundary.0 == boundary.1 {
        return Ok(name);
    }
    Err(ParseFailure::unsupported(
        boundary.0..boundary.1,
        "timing track and @ must be adjacent",
    ))
}
pub(in crate::plantuml) fn diagnose(
    mut error: ParseFailure,
    body: &BodySource<'_>,
    budget: &mut Budget,
) -> RenderError {
    if matches!(error.kind, FailureKind::Terminal(_)) {
        return error.into_document(body.document, budget);
    }
    let text = &body.document.text;
    let start = error.span.start.min(text.len());
    let head = text[..start].rfind('\n').map_or(0, |index| index + 1);
    let tail = text[start..]
        .find('\n')
        .map_or(text.len(), |index| start + index);
    let row = &text[head..tail];
    if let Err(error) = budget.charge(row.len().saturating_mul(4) + 128) {
        return error;
    }
    let kind = match error.kind {
        FailureKind::Syntax(detail) => FailureKind::Syntax(format!("{detail}: {row}").into()),
        FailureKind::Unsupported(detail) => {
            FailureKind::Unsupported(format!("{detail}: {row}").into())
        }
        FailureKind::Terminal(_) => unreachable!(),
    };
    error.kind = kind;
    error.into_document(body.document, budget)
}
fn parse_body(body: &BodySource<'_>, budget: &mut Budget) -> Result<Timing, ParseFailure> {
    bound(
        "timing source bytes",
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
    super::generated_timing::TimingParser::new()
        .parse(&mut builder, iterator)
        .map_err(|error| {
            generated_failure(error, body.range.end, "invalid or incomplete timing syntax")
        })?;
    builder.finish(Site {
        span: body.range.end..body.range.end,
        line,
    })
}
fn copy(value: &str, budget: &mut Budget) -> Result<String, ParseFailure> {
    bound(
        "timing literal bytes",
        value.len(),
        budget.limits.source_bytes,
    )?;
    budget.charge(value.len())?;
    for chunk in value.as_bytes().chunks(256) {
        budget.charge(chunk.len())?;
    }
    Ok(value.into())
}
fn lex(
    row: &Row,
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
        let (token, used) = if rest.starts_with("<->") {
            (Token::Constraint, 3)
        } else if rest.starts_with("->") {
            (Token::Arrow, 2)
        } else if ch == '@' {
            if rest[1..].starts_with(char::is_whitespace) {
                return Err(ParseFailure::unsupported(
                    row.span(base, cursor..cursor + 1),
                    "timing @ requires an adjacent player or time",
                ));
            }
            (Token::At, 1)
        } else if ch == ':' {
            context::push(
                tokens,
                (
                    row.span(base, cursor..cursor + 1).start,
                    Token::Colon,
                    row.span(base, cursor..cursor + 1).end,
                ),
                budget,
            )?;
            let value = copy(rest[1..].trim(), budget)?;
            let span = row.span(base, cursor + 1..row.text.len());
            context::push(tokens, (span.start, Token::Label(value), span.end), budget)?;
            cursor = row.text.len();
            continue;
        } else if ch == '"' {
            let mut escaped = false;
            let mut end = None;
            for (index, ch) in rest.char_indices().skip(1) {
                budget.charge(ch.len_utf8())?;
                if escaped {
                    escaped = false;
                    continue;
                }
                if ch == '\\' {
                    escaped = true;
                    continue;
                }
                if ch == '"' {
                    end = Some(index);
                    break;
                }
            }
            let end = end.ok_or_else(|| {
                ParseFailure::syntax(
                    row.span(base, cursor..row.text.len()),
                    "unterminated timing string",
                )
            })?;
            bound("label bytes", end - 1, 16 * 1024)?;
            (Token::Quoted(copy(&rest[1..end], budget)?), end + 1)
        } else {
            let mut used = ch.len_utf8();
            for (index, ch) in rest.char_indices().skip(1) {
                if ch.is_whitespace()
                    || matches!(ch, '@' | ':' | '"')
                    || rest[index..].starts_with("->")
                    || rest[index..].starts_with("<->")
                {
                    break;
                }
                budget.charge(ch.len_utf8())?;
                used = index + ch.len_utf8();
            }
            let atom = Atom {
                text: copy(&rest[..used], budget)?,
                gap: rest[used..].starts_with(char::is_whitespace),
            };
            let token = match atom.text.as_str() {
                "robust" => Token::Robust(atom),
                "concise" => Token::Concise(atom),
                "as" => Token::As(atom),
                "is" => Token::Is(atom),
                _ if is_code(&atom.text) => Token::Code(atom),
                _ if atom
                    .text
                    .starts_with(|ch: char| ch.is_ascii_digit() || matches!(ch, '+' | '-')) =>
                {
                    Token::Numeric(atom)
                }
                _ => Token::Other(atom),
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
        literals += match token {
            Token::Robust(value)
            | Token::Concise(value)
            | Token::As(value)
            | Token::Is(value)
            | Token::Code(value)
            | Token::Numeric(value)
            | Token::Other(value) => value.text.capacity(),
            Token::Quoted(value) | Token::Label(value) => value.capacity(),
            _ => 0,
        };
        items = items.saturating_add(match token {
            Token::Robust(_) | Token::Concise(_) | Token::At => 1,
            Token::Is(_) => 3,
            Token::Arrow | Token::Constraint => 1,
            _ => 0,
        });
    }
    // Exhaustive actual tables: 10 live states, four reductions, 14 terminals.
    // Capacity16/growth24 covers generated state and symbol vectors; diagnostic
    // copies, expected String slots and every terminal probe are included separately.
    let symbol = std::mem::size_of::<Token>()
        + std::mem::size_of::<Command>()
        + std::mem::size_of::<Atom>()
        + std::mem::size_of::<Target>()
        + std::mem::size_of::<TimeChoice>()
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
                + 14 * 32,
        )
        .saturating_add(std::mem::size_of::<Builder<'_>>())
        .saturating_add(Builder::retained_bytes(
            items.min(budget.limits.model_items),
        ))
        .saturating_add(budget.limits.source_bytes.saturating_mul(16));
    bound(
        "timing parser retained bytes",
        retained,
        budget.limits.source_bytes.saturating_mul(256),
    )?;
    bound(
        "timing parser stack slots",
        slots,
        budget.limits.layout_work,
    )?;
    budget.charge(
        tokens
            .len()
            .saturating_mul(5)
            .saturating_add(slots + 4 + 14 * (10 + 4)),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RenderOptions;
    fn model(body: &str) -> Result<Timing, RenderError> {
        let mut budget = Budget::new(&RenderOptions::default());
        let document = crate::plantuml::preprocess::prepare(
            &format!("@startuml\n{body}\n@enduml"),
            &mut budget,
        )?;
        let mut prepared = super::super::document::parse(&document, &mut budget)?;
        super::super::preamble::parse(&mut prepared, &mut budget)?;
        parse(prepared.body, &mut budget)
    }
    #[test]
    fn should_build_generated_timing_with_contextual_track_names_and_separate_cursors() {
        // given
        let body = "robust robust\nconcise concise\nrobust is Initial\n@0\nrobust is First\nconcise is Waiting\n@concise\n+5 is Running\nrobust is Changed\nrobust@0 <-> @+5 : interval\nrobust@0 -> concise@+5 : message\n@10";
        // when
        let result = model(body).unwrap();
        // then
        assert_eq!(result.tracks[0].code, "robust");
        assert_eq!(result.tracks[1].code, "concise");
        assert_eq!(result.tracks[0].changes[1].time, 5);
        assert_eq!(
            (result.constraints[0].start, result.constraints[0].end),
            (0, 5)
        );
        assert_eq!(result.messages[0].to.time, 10);
    }

    #[test]
    fn should_reject_whitespace_inside_timing_endpoint_primitives() {
        // given
        for row in ["@ V", "@ 0", "V @0 -> C@1", "V@ 0 -> C@1", "V@0 <-> @ 1"] {
            // when
            let result = model(&format!(
                "robust V\nconcise C\n@0\nV is A\nC is B\n@10\n{row}"
            ));
            // then
            assert!(result.is_err(), "accepted {row}");
        }
    }

    #[test]
    fn should_reject_generated_timing_eof_and_cancel_delivery_and_actions() {
        // given
        let atom = |text: &str| Atom {
            text: text.into(),
            gap: true,
        };
        for tokens in [
            vec![(0, Token::Robust(atom("robust")), 6)],
            vec![(0, Token::At, 1)],
            vec![(0, Token::Code(atom("V")), 1), (1, Token::Arrow, 3)],
        ] {
            let mut budget = Budget::new(&RenderOptions::default());
            admit((&tokens, tokens.capacity()), 0, &mut budget).unwrap();
            let iterator = context::Checked::new(tokens, budget.control.clone());
            let mut builder = Builder::new(&mut budget);
            // when
            let result =
                super::super::generated_timing::TimingParser::new().parse(&mut builder, iterator);
            // then
            assert!(result.is_err());
        }
        for checkpoint in 1..=6 {
            let tokens = vec![
                (0, Token::Robust(atom("robust")), 6),
                (7, Token::Code(atom("V")), 8),
                (8, Token::Newline(1), 9),
            ];
            let mut budget = Budget::new(&RenderOptions::default());
            admit((&tokens, tokens.capacity()), 0, &mut budget).unwrap();
            budget.control.cancel_after_checkpoints(checkpoint);
            let iterator = context::Checked::new(tokens, budget.control.clone());
            let mut builder = Builder::new(&mut budget);
            // when
            let result =
                super::super::generated_timing::TimingParser::new().parse(&mut builder, iterator);
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
    fn should_preserve_full_offending_statement_and_map_expansion_once() {
        // given
        for body in [
            "!define BAD V is A extra\nrobust V\n@0\nV is Initial\nBAD\n@10",
            "!define BAD unknown is A\nrobust V\n@0\nV is Initial\nBAD\n@10",
        ] {
            // when
            let error = match model(body) {
                Ok(_) => panic!("accepted bad statement"),
                Err(error) => error.to_string(),
            };
            // then
            assert!(error.contains("is A"), "{error}");
            assert_eq!(error.matches("expansion \"BAD\"").count(), 1, "{error}");
        }
    }

    #[test]
    fn should_bound_timing_tokens_depth_work_and_mid_lex_cancellation() {
        // given
        for resource in ["source", "depth", "work", "cancel"] {
            let mut budget = Budget::new(&RenderOptions::default());
            let document = crate::plantuml::preprocess::prepare(
                "@startuml\nrobust V\n@0\nV is A\n@10\n@enduml",
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
