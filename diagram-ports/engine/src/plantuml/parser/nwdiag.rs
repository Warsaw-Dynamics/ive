//! Generated nwdiag blocks/properties/statements, preserving shared lexical ownership.
use super::{
    BodySource, context,
    error::{ParseFailure, generated_failure},
    ownership::Row,
    timing::diagnose,
};
use crate::{
    RenderError,
    plantuml::{
        budget::{Budget, bound},
        nwdiag::{Builder, NwDiagram},
    },
};
use std::ops::Range;
#[derive(Clone, Copy, Debug)]
pub(in crate::plantuml) enum PropertyKey {
    Address,
    Description,
    Color,
    Width,
    Other,
}
#[derive(Default, Debug)]
pub(in crate::plantuml) enum Field {
    #[default]
    Missing,
    Text(String),
}
#[derive(Default, Debug)]
pub(in crate::plantuml) struct Properties {
    pub address: Field,
    pub description: Field,
    pub color: Field,
}
impl Properties {
    pub fn append(
        mut self,
        value: (PropertyKey, String),
        site: Range<usize>,
        budget: &mut Budget,
    ) -> Result<Self, ParseFailure> {
        let target = match value.0 {
            PropertyKey::Address => &mut self.address,
            PropertyKey::Description => &mut self.description,
            PropertyKey::Color => &mut self.color,
            _ => {
                return Err(ParseFailure::syntax(
                    site,
                    "unsupported nwdiag server property",
                ));
            }
        };
        if matches!(target, Field::Text(_)) {
            return Err(ParseFailure::syntax(
                site,
                "duplicate nwdiag server property",
            ));
        }
        budget.item()?;
        *target = Field::Text(value.1);
        Ok(self)
    }
}
pub(in crate::plantuml) fn key(value: &str) -> PropertyKey {
    match value {
        "address" => PropertyKey::Address,
        "description" => PropertyKey::Description,
        "color" => PropertyKey::Color,
        "width" => PropertyKey::Width,
        _ => PropertyKey::Other,
    }
}
#[derive(Clone, Debug)]
pub(in crate::plantuml) enum Token {
    Nwdiag(String),
    Network(String),
    Group(String),
    Address(String),
    Description(String),
    Color(String),
    Width(String),
    Atom(String),
    Quoted(String),
    Open,
    Close,
    Left,
    Right,
    Comma,
    Equal,
    End,
}
type Spanned = (usize, Token, usize);
pub(in crate::plantuml) fn name(value: String, span: Range<usize>) -> Result<String, ParseFailure> {
    if value
        .chars()
        .all(|ch| ch.is_alphanumeric() || matches!(ch, '-' | '.' | '_'))
    {
        return Ok(value);
    }
    Err(ParseFailure::syntax(
        span,
        "nwdiag styles/chrome must precede the body; invalid nwdiag identifier",
    ))
}
pub(in crate::plantuml) fn parse(
    body: BodySource<'_>,
    budget: &mut Budget,
) -> Result<NwDiagram, RenderError> {
    parse_body(&body, budget).map_err(|error| diagnose(error, &body, budget))
}
fn parse_body(body: &BodySource<'_>, budget: &mut Budget) -> Result<NwDiagram, ParseFailure> {
    bound(
        "nwdiag source bytes",
        body.range.len(),
        budget.limits.source_bytes,
    )?;
    let mut tokens = Vec::new();
    let mut ownership = body.ownership;
    let mut base = body.range.start;
    budget.charge(std::mem::size_of::<super::ownership::Ownership>())?;
    for physical in body.document.text[body.range.clone()].split_inclusive('\n') {
        let raw = physical
            .strip_suffix('\n')
            .map_or(physical, |row| row.strip_suffix('\r').unwrap_or(row));
        let row = ownership.classify(raw, budget)?;
        ownership.advance(&row, &row.text, budget)?;
        lex(&row, base, (&mut tokens, budget))?;
        base += physical.len();
    }
    ownership.finish(base)?;
    admit((&tokens, tokens.capacity()), body.adapters, budget)?;
    let iterator = context::Checked::new(tokens, budget.control.clone());
    let mut builder = Builder::new(budget);
    super::generated_nwdiag::NwdiagParser::new()
        .parse(&mut builder, iterator)
        .map_err(|error| {
            generated_failure(
                error,
                body.range.end,
                "invalid or incomplete nwdiag syntax; nwdiag styles/chrome must precede the body",
            )
        })?;
    builder.finish(body.range.end..body.range.end)
}
fn lex(
    row: &Row,
    base: usize,
    output: (&mut Vec<Spanned>, &mut Budget),
) -> Result<(), ParseFailure> {
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
        budget.item()?;
        let (token, used) = match ch {
            '{' => (Token::Open, 1),
            '}' => (Token::Close, 1),
            '[' => (Token::Left, 1),
            ']' => (Token::Right, 1),
            ',' => (Token::Comma, 1),
            '=' => (Token::Equal, 1),
            ';' => (Token::End, 1),
            '"' => {
                let mut end = None;
                for (index, ch) in rest.char_indices().skip(1) {
                    budget.charge(ch.len_utf8())?;
                    bound("label bytes", index - 1, 16 * 1024)?;
                    if ch == '"' {
                        end = Some(index);
                        break;
                    }
                    bound("label bytes", index - 1 + ch.len_utf8(), 16 * 1024)?;
                }
                let end = end.ok_or_else(|| {
                    ParseFailure::syntax(
                        row.span(base, cursor..row.text.len()),
                        "unclosed nwdiag quote",
                    )
                })?;
                budget.charge(end - 1)?;
                (Token::Quoted(rest[1..end].to_owned()), end + 1)
            }
            _ => {
                let mut used = ch.len_utf8();
                for (index, ch) in rest.char_indices().skip(1) {
                    if ch.is_whitespace()
                        || matches!(ch, '{' | '}' | '[' | ']' | ',' | '=' | ';' | '"')
                    {
                        break;
                    }
                    budget.charge(ch.len_utf8())?;
                    used = index + ch.len_utf8();
                }
                bound("nwdiag atom bytes", used, budget.limits.source_bytes)?;
                budget.charge(used)?;
                let value = rest[..used].to_owned();
                let token = match value.as_str() {
                    "nwdiag" => Token::Nwdiag(value),
                    "network" => Token::Network(value),
                    "group" => Token::Group(value),
                    "address" => Token::Address(value),
                    "description" => Token::Description(value),
                    "color" => Token::Color(value),
                    "width" => Token::Width(value),
                    _ => Token::Atom(value),
                };
                (token, used)
            }
        };
        let span = row.span(base, cursor..cursor + used);
        context::push(tokens, (span.start, token, span.end), budget)?;
        cursor += used;
    }
    if !row.text.trim().is_empty() {
        budget.item()?;
        let span = row.span(base, row.text.len()..row.text.len());
        context::push(tokens, (span.start, Token::End, span.end), budget)?;
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
    for (_, token, _) in tokens {
        budget.charge(1)?;
        literals += match token {
            Token::Nwdiag(value)
            | Token::Network(value)
            | Token::Group(value)
            | Token::Address(value)
            | Token::Description(value)
            | Token::Color(value)
            | Token::Width(value)
            | Token::Atom(value)
            | Token::Quoted(value) => value.capacity(),
            _ => 0,
        };
    }
    let symbol = std::mem::size_of::<Token>()
        + std::mem::size_of::<Properties>()
        + std::mem::size_of::<String>()
        + std::mem::size_of::<PropertyKey>()
        + 4 * std::mem::size_of::<usize>();
    let retained = capacity
        .saturating_mul(std::mem::size_of::<Spanned>())
        .saturating_add(literals)
        .saturating_add(adapters)
        .saturating_add(
            24 * symbol
                + 48 * std::mem::size_of::<usize>()
                + 48 * std::mem::size_of::<String>()
                + 16 * 32,
        )
        .saturating_add(std::mem::size_of::<Builder<'_>>())
        .saturating_add(Builder::retained_bytes(
            tokens.len().min(budget.limits.model_items),
        ))
        .saturating_add(budget.limits.source_bytes.saturating_mul(16));
    bound(
        "nwdiag parser retained bytes",
        retained,
        budget.limits.source_bytes.saturating_mul(256),
    )?;
    // Actual120-state/16-terminal tables:15 live states, four reductions,
    // twelve expected terminals; copied diagnostics grow to18/peak27.
    bound("nwdiag parser stack slots", 144, budget.limits.layout_work)?;
    budget.charge(tokens.len().saturating_mul(5) + 144 + 4 + 16 * (15 + 4))?;
    Ok(())
}
#[cfg(test)]
pub(in crate::plantuml) fn tokenize_test(
    text: &str,
    budget: &mut Budget,
) -> Result<(), ParseFailure> {
    let row = super::ownership::Ownership::default().classify(text, budget)?;
    lex(&row, 0, (&mut Vec::new(), budget))
}
#[cfg(test)]
pub(in crate::plantuml) fn parse_test(
    lines: &[crate::plantuml::source::Line],
    budget: &mut Budget,
) -> Result<NwDiagram, RenderError> {
    let text = format!(
        "@startnwdiag\n{}\n@endnwdiag",
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
    fn model(body: &str) -> Result<NwDiagram, RenderError> {
        let mut budget = Budget::new(&RenderOptions::default());
        let document = crate::plantuml::preprocess::prepare(
            &format!("@startnwdiag\n{body}\n@endnwdiag"),
            &mut budget,
        )?;
        let mut prepared = super::super::document::parse(&document, &mut budget)?;
        super::super::preamble::parse(&mut prepared, &mut budget)?;
        parse(prepared.body, &mut budget)
    }
    #[test]
    fn should_build_generated_global_servers_addresses_and_group_references() {
        // given
        let body = "nwdiag {\nnetwork edge { title [address=old]; title [address=new]; title [address=\"\"]; }\nnetwork core { title [address=other]; b; }\ngroup zone { description=\"Owned group\"; b; }\n}";
        // when
        let model = model(body).unwrap();
        // then
        assert_eq!(model.servers.len(), 2);
        assert_eq!(model.memberships.len(), 3);
        assert_eq!(model.memberships[0].address, "new");
        assert_eq!(model.memberships[1].address, "other");
        assert_eq!(model.groups[0].members[0].0, 1);
    }

    #[test]
    fn should_own_property_terminators_and_block_nonnesting_through_full_eof() {
        // given
        for body in [
            "nwdiag { network n { r[address=x color=\"#aabbcc\"]; } }",
            "nwdiag { network n { r[address=x,]; } }",
            "nwdiag { network n { r[address=x,address=y]; } }",
            "nwdiag { network n { r[address=x\n]; } }",
            "nwdiag { network n { r; network z { r; } } }",
            "nwdiag { network n { group g { r; group h { r; } } } }",
            "nwdiag { network n { r; } } extra",
        ] {
            // when
            let result = model(body);
            // then
            assert!(result.is_err(), "accepted {body}");
        }
        assert!(
            model(
                "nwdiag{network n{r[address=x,color=\"#aabbcc\"]}network z{r[address=y]}group g{r}}"
            )
            .is_ok()
        );
    }

    #[test]
    fn should_reject_direct_generated_unclosed_blocks_properties_and_cancel_actions() {
        // given
        for tokens in [
            vec![(0, Token::Nwdiag("nwdiag".into()), 6), (6, Token::Open, 7)],
            vec![
                (0, Token::Nwdiag("nwdiag".into()), 6),
                (6, Token::Open, 7),
                (8, Token::Network("network".into()), 15),
                (16, Token::Atom("n".into()), 17),
                (17, Token::Open, 18),
                (19, Token::Atom("r".into()), 20),
                (20, Token::Left, 21),
                (21, Token::Address("address".into()), 28),
                (28, Token::Equal, 29),
            ],
        ] {
            let mut budget = Budget::new(&RenderOptions::default());
            admit((&tokens, tokens.capacity()), 0, &mut budget).unwrap();
            let iterator = context::Checked::new(tokens, budget.control.clone());
            let mut builder = Builder::new(&mut budget);
            // when
            let result =
                super::super::generated_nwdiag::NwdiagParser::new().parse(&mut builder, iterator);
            // then
            assert!(result.is_err());
        }
        for checkpoint in 1..=6 {
            let tokens = vec![
                (0, Token::Nwdiag("nwdiag".into()), 6),
                (6, Token::Open, 7),
                (7, Token::Close, 8),
                (8, Token::End, 9),
            ];
            let mut budget = Budget::new(&RenderOptions::default());
            admit((&tokens, tokens.capacity()), 0, &mut budget).unwrap();
            budget.control.cancel_after_checkpoints(checkpoint);
            let iterator = context::Checked::new(tokens, budget.control.clone());
            let mut builder = Builder::new(&mut budget);
            // when
            let result =
                super::super::generated_nwdiag::NwdiagParser::new().parse(&mut builder, iterator);
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
    fn should_map_generated_and_semantic_expansion_failures_once_with_full_statement() {
        // given
        for statement in ["r[address=x,address=y];", "missing;"] {
            // when
            let result = model(&format!(
                "!define BAD {statement}\nnwdiag {{ network n {{ r; }} group g {{ BAD }} }}"
            ));
            let error = match result {
                Err(error) => error.to_string(),
                Ok(_) => panic!("accepted bad statement"),
            };
            // then
            assert!(error.contains(statement), "{error}");
            assert_eq!(error.matches("expansion \"BAD\"").count(), 1, "{error}");
        }
    }

    #[test]
    fn should_bound_nwdiag_parser_work_and_mid_lex_cancellation() {
        // given
        for resource in ["source", "depth", "work", "cancel"] {
            let mut budget = Budget::new(&RenderOptions::default());
            let document = crate::plantuml::preprocess::prepare(
                "@startnwdiag\nnwdiag { network n { r; } }\n@endnwdiag",
                &mut budget,
            )
            .unwrap();
            let mut prepared = super::super::document::parse(&document, &mut budget).unwrap();
            super::super::preamble::parse(&mut prepared, &mut budget).unwrap();
            match resource {
                "source" => budget.limits.source_bytes = 4,
                "depth" => budget.limits.nesting_depth = 1,
                "work" => budget.limits.layout_work = budget.work + 1,
                _ => budget.control.cancel_after_checkpoints(12),
            };
            // when
            let error = match parse(prepared.body, &mut budget) {
                Err(error) => error,
                Ok(_) => panic!("accepted {resource}"),
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
