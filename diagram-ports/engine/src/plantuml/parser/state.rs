//! Generated state syntax over the shared immutable body and ownership checkpoint.
use super::{
    BodySource, context,
    error::{ParseFailure, generated_failure},
    ownership::Row,
};
use crate::{
    RenderError,
    plantuml::{
        budget::{Budget, bound},
        state::{Builder, Caption, Declaration, Name, State},
    },
};
use std::ops::Range;

#[derive(Clone, Debug)]
pub(in crate::plantuml) enum Token {
    State,
    Gap,
    As,
    Begin,
    End,
    EndState,
    Name(Name),
    Label(String),
    Pseudo,
    Open,
    Close,
    Colon,
    AttachedColon,
    Less,
    Greater,
    Minus,
    Unknown,
    Newline,
}
type Spanned = (usize, Token, usize);

pub(in crate::plantuml) fn parse(
    body: BodySource<'_>,
    budget: &mut Budget,
) -> Result<State, RenderError> {
    parse_body(&body, budget).map_err(|error| error.into_document(body.document, budget))
}

fn parse_body(body: &BodySource<'_>, budget: &mut Budget) -> Result<State, ParseFailure> {
    bound(
        "state source bytes",
        body.range.len(),
        budget.limits.source_bytes,
    )
    .map_err(ParseFailure::terminal)?;
    budget
        .charge(std::mem::size_of::<super::ownership::Ownership>())
        .map_err(ParseFailure::terminal)?;
    let mut lexer = Lexer {
        tokens: vec![],
        literal_bytes: 0,
        budget,
    };
    let mut ownership = body.ownership;
    let mut base = body.range.start;
    for physical in body.document.text[body.range.clone()].split_inclusive('\n') {
        let raw = physical
            .strip_suffix('\n')
            .map_or(physical, |row| row.strip_suffix('\r').unwrap_or(row));
        let row = ownership.classify(raw, lexer.budget)?;
        ownership.advance(&row, &row.text, lexer.budget)?;
        if row.text.trim().is_empty() {
            lexer.budget.charge(1).map_err(ParseFailure::terminal)?;
        } else {
            lexer.row(&row, base)?;
        }
        base += physical.len();
    }
    ownership.finish(base)?;
    let tokens = lexer.tokens;
    let retained = admit((&tokens, tokens.capacity()), body.adapters, lexer.budget)?;
    let iterator = context::Checked::new(tokens, lexer.budget.control.clone());
    let mut builder = Builder::new(lexer.budget, retained);
    super::generated_state::StateParser::new()
        .parse(&mut builder, iterator)
        .map_err(|error| {
            generated_failure(error, body.range.end, "invalid or incomplete state syntax")
        })?;
    builder.finish()
}

struct Lexer<'b> {
    tokens: Vec<Spanned>,
    literal_bytes: usize,
    budget: &'b mut Budget,
}

impl Lexer<'_> {
    fn push(&mut self, token: Token, span: Range<usize>) -> Result<(), ParseFailure> {
        context::push(&mut self.tokens, (span.start, token, span.end), self.budget)
    }

    fn copy(&mut self, value: &str) -> Result<String, ParseFailure> {
        bound("label bytes", value.len(), 16 * 1024).map_err(ParseFailure::terminal)?;
        self.literal_bytes = self.literal_bytes.saturating_add(value.len());
        bound(
            "state syntax literal bytes",
            self.literal_bytes,
            self.budget.limits.source_bytes.saturating_mul(2),
        )
        .map_err(ParseFailure::terminal)?;
        for chunk in value.as_bytes().chunks(256) {
            self.budget
                .charge(chunk.len())
                .map_err(ParseFailure::terminal)?;
        }
        Ok(value.into())
    }

    fn row(&mut self, row: &Row, base: usize) -> Result<(), ParseFailure> {
        let mut cursor = 0;
        while cursor < row.text.len() {
            let rest = &row.text[cursor..];
            let ch = rest.chars().next().unwrap();
            self.budget
                .charge(ch.len_utf8())
                .map_err(ParseFailure::terminal)?;
            if ch.is_whitespace() {
                cursor += ch.len_utf8();
                continue;
            }
            if ch == ':' && !rest.starts_with("::") {
                let token = if row.text[..cursor]
                    .chars()
                    .next_back()
                    .is_some_and(char::is_whitespace)
                {
                    Token::Colon
                } else {
                    Token::AttachedColon
                };
                self.push(token, row.span(base, cursor..cursor + 1))?;
                let tail = rest[1..].trim();
                let start = cursor + 1 + rest[1..].len() - rest[1..].trim_start().len();
                let value = self.copy(tail)?;
                self.push(
                    Token::Label(value),
                    row.span(base, start..start + tail.len()),
                )?;
                break;
            }
            if ch == '"' {
                let (name, bytes) = self.quoted(row, (base, cursor))?;
                self.push(Token::Name(name), row.span(base, cursor..cursor + bytes))?;
                cursor += bytes;
                continue;
            }
            let (token, bytes) = if rest.starts_with("[*]") {
                (Token::Pseudo, 3)
            } else {
                match ch {
                    '{' => (Token::Open, 1),
                    '}' => (Token::Close, 1),
                    '<' => (Token::Less, 1),
                    '>' => (Token::Greater, 1),
                    '-' => (Token::Minus, 1),
                    _ => self.word(rest)?,
                }
            };
            let gap = matches!(token, Token::State) && rest[bytes..].starts_with(' ');
            self.push(token, row.span(base, cursor..cursor + bytes))?;
            cursor += bytes;
            if gap {
                self.push(Token::Gap, row.span(base, cursor..cursor + 1))?;
                cursor += 1;
            }
        }
        self.push(
            Token::Newline,
            row.span(base, row.text.len()..row.text.len()),
        )
    }

    fn word(&mut self, rest: &str) -> Result<(Token, usize), ParseFailure> {
        let mut bytes = 0;
        for (index, ch) in rest.char_indices() {
            if ch.is_alphanumeric() || matches!(ch, '_' | '.') {
                self.budget
                    .charge(ch.len_utf8())
                    .map_err(ParseFailure::terminal)?;
                bytes = index + ch.len_utf8();
            } else if ch == ':' && rest[index..].starts_with("::") {
                self.budget.charge(2).map_err(ParseFailure::terminal)?;
                bytes = index + 1;
            } else if ch == ':' && index > 0 && rest[..index].ends_with(':') {
                bytes = index + 1;
            } else {
                break;
            }
        }
        if bytes == 0 {
            return Ok((Token::Unknown, rest.chars().next().unwrap().len_utf8()));
        }
        let word = &rest[..bytes];
        let token = match word {
            "state" => Token::State,
            "as" => Token::As,
            "begin" => Token::Begin,
            "end"
                if rest.starts_with("end state")
                    && rest[9..].chars().next().is_none_or(char::is_whitespace) =>
            {
                Token::End
            }
            "endstate" => Token::EndState,
            _ => Token::Name(Name {
                text: self.copy(word)?,
                quoted: false,
            }),
        };
        Ok((token, bytes))
    }

    fn quoted(
        &mut self,
        row: &Row,
        location: (usize, usize),
    ) -> Result<(Name, usize), ParseFailure> {
        let (base, cursor) = location;
        let mut value = String::new();
        let mut escaped = false;
        for (index, ch) in row.text[cursor..].char_indices().skip(1) {
            self.budget
                .charge(ch.len_utf8())
                .map_err(ParseFailure::terminal)?;
            bound(
                "label bytes",
                value.len().saturating_add(ch.len_utf8() + 1),
                16 * 1024 + 2,
            )
            .map_err(ParseFailure::terminal)?;
            if value.len().saturating_add(ch.len_utf8() + 1) > value.capacity() {
                self.budget
                    .charge(value.len())
                    .map_err(ParseFailure::terminal)?;
            }
            if escaped {
                match ch {
                    'n' | '\\' | '~' => {
                        value.push('\\');
                        value.push(ch);
                    }
                    '"' => value.push(ch),
                    _ => {
                        return Err(ParseFailure::unsupported(
                            row.span(base, cursor + index..cursor + index + ch.len_utf8()),
                            "quoted state escape",
                        ));
                    }
                }
                escaped = false;
            } else {
                match ch {
                    '\\' => escaped = true,
                    '"' => {
                        return Ok((
                            Name {
                                text: self.copy(&value)?,
                                quoted: true,
                            },
                            index + 1,
                        ));
                    }
                    _ => value.push(ch),
                }
            }
        }
        Err(ParseFailure::syntax(
            row.span(base, cursor..row.text.len()),
            "unterminated quoted state display",
        ))
    }
}

fn admit(
    input: (&[Spanned], usize),
    adapters: usize,
    budget: &mut Budget,
) -> Result<usize, ParseFailure> {
    let (tokens, token_capacity) = input;
    let mut openers = 0usize;
    let mut literals = 0usize;
    for chunk in tokens.chunks(256) {
        budget.charge(chunk.len()).map_err(ParseFailure::terminal)?;
        for (_, token, _) in chunk {
            match token {
                Token::Open | Token::Begin => openers += 1,
                Token::Name(value) => literals = literals.saturating_add(value.text.capacity()),
                Token::Label(value) => literals = literals.saturating_add(value.capacity()),
                _ => {}
            }
        }
    }
    // Actual 59-state/18-terminal tables use at most 2d+8 live states and
    // four reductions between deliveries. Reserve four extra states.
    let live = openers
        .min(budget.limits.nesting_depth)
        .saturating_mul(2)
        .saturating_add(12);
    let capacity = live.next_power_of_two().max(8);
    let growing = capacity + capacity / 2;
    let diagnostic = 3 * (live + 1);
    let symbol = std::mem::size_of::<Token>()
        + std::mem::size_of::<Name>()
        + std::mem::size_of::<Declaration>()
        + std::mem::size_of::<Caption>()
        + 4 * std::mem::size_of::<usize>();
    let retained = token_capacity
        .saturating_mul(std::mem::size_of::<Spanned>())
        .saturating_add(literals)
        .saturating_add(adapters)
        .saturating_add(growing.saturating_mul(symbol + 1))
        .saturating_add(diagnostic + 48 * std::mem::size_of::<String>() + 18 * 32)
        .saturating_add(std::mem::size_of::<Builder<'_>>())
        .saturating_add(budget.limits.source_bytes.saturating_mul(16));
    bound(
        "state parser retained bytes",
        retained,
        budget.limits.source_bytes.saturating_mul(256),
    )
    .map_err(ParseFailure::terminal)?;
    let slots = growing.saturating_mul(2).saturating_add(diagnostic + 48);
    bound("state parser stack slots", slots, budget.limits.layout_work)
        .map_err(ParseFailure::terminal)?;
    budget
        .charge(
            tokens
                .len()
                .saturating_mul(5)
                .saturating_add(4 + slots + 18 * (live + 4)),
        )
        .map_err(ParseFailure::terminal)?;
    Ok(retained)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        RenderOptions,
        plantuml::state::{Kind, Scope},
    };

    fn model(body: &str) -> Result<State, RenderError> {
        let mut budget = Budget::new(&RenderOptions::default());
        let document = crate::plantuml::preprocess::prepare(
            &format!("@startuml\n{body}\n@enduml"),
            &mut budget,
        )?;
        let mut parsed = super::super::document::parse(&document, &mut budget)?;
        super::super::preamble::parse(&mut parsed, &mut budget)?;
        parse(parsed.body, &mut budget)
    }

    #[test]
    fn should_build_generated_state_forward_composites_local_ids_and_scoped_pseudostates() {
        // given
        let body = "[*] --> First\nstate \"First title\" as First {\nC : first\nC --> [*] : done\nstate C : second\n[*] --> C\n}\nstate Second begin\nstate C\n[*] --> C\nC --> [*]\nend state\nFirst <-- [*]";
        // when
        let result = model(body).unwrap();
        // then
        assert_eq!(result.nodes[0].name, "First");
        assert_eq!(result.nodes[0].kind, Kind::Composite);
        assert_eq!(result.nodes[1].name, "C");
        assert_eq!(result.nodes[1].scope, Scope::State(0));
        assert_eq!(result.nodes[1].descriptions, ["first", "second"]);
        assert_eq!(result.nodes[3].name, "C");
        assert_eq!(result.nodes[3].scope, Scope::State(2));
        assert_eq!(result.transitions.len(), 6);
        assert_eq!(
            result
                .nodes
                .iter()
                .filter(|n| n.kind == Kind::Initial)
                .count(),
            3
        );
        assert_eq!(
            result
                .nodes
                .iter()
                .filter(|n| n.kind == Kind::Final)
                .count(),
            2
        );
    }

    #[test]
    fn should_preserve_state_command_boundaries_and_reserved_left_endpoint_policy() {
        // given
        for body in [
            "state : text",
            "state A::B",
            "state A\nstate --> B",
            "state\tA",
            "state\"A\"",
            "state A {\nend  state",
        ] {
            // when
            let result = model(body);
            // then
            assert!(result.is_err(), "accepted {body:?}");
        }
    }

    #[test]
    fn should_reject_a_trailing_bare_opener_in_a_state_declaration() {
        // given
        for body in ["state begin", "state \"display\" as begin"] {
            // when
            let result = model(body);
            // then
            assert!(result.is_err(), "accepted {body:?}");
        }
        // given
        for body in [
            "state \"begin\"",
            "state S as \"begin\"",
            "state begin {\n}",
            "state \"display\" as begin begin\n}",
            "state \"A::B\"",
        ] {
            // when
            let result = model(body);
            // then
            assert!(result.is_ok(), "rejected {body:?}: {result:?}");
        }
    }

    #[test]
    fn should_preserve_transition_label_separators_and_literal_state_name_spelling() {
        // given
        for body in [
            "state A\nA --> B: label",
            "state A\nA --> B:label",
            "state A\nA --> \"B\"",
            "state A\n\"A\" --> B",
        ] {
            // when
            let result = model(body);
            // then
            assert!(result.is_err(), "accepted {body:?}");
        }
    }

    #[test]
    fn should_reject_generated_state_incomplete_structures_and_unknown_suffixes() {
        // given
        for body in [
            "state A {",
            "state A {\nstate B begin\n}",
            "state A\nendstate",
            "state A\nA --> B ignored",
            "state A\nA -->",
            "state A\nA --> B :",
            "state A\nA --> [H]",
            "state A <<choice>>",
            "state A {\n--\n}",
        ] {
            // when
            let error = model(body).unwrap_err();
            // then
            assert!(
                matches!(
                    error,
                    RenderError::InvalidSource(_) | RenderError::UnsupportedFeature(_)
                ),
                "{body}: {error}"
            );
        }
    }

    #[test]
    fn should_accept_supported_state_closers_arrows_and_empty_descriptions() {
        // given
        for closer in ["}", "endstate", "end state"] {
            for arrow in ["->", "-->", "------>", "<-", "<------"] {
                // when
                let result = model(&format!("state Outer begin\nstate A :\nA {arrow} B : payload: title -> endstate\n{closer}")).unwrap();
                // then
                assert_eq!(result.nodes[1].descriptions, [""]);
                assert_eq!(result.transitions[0].label, "payload: title -> endstate");
            }
        }
    }

    #[test]
    fn should_continue_shared_state_comment_ownership_and_keep_utf8_crlf_labels() {
        // given
        let source = "\u{feff}@startuml\r\ntitle Ω /' comment\r\n'/  state \"Ω-->title\" as A\r\nA : title: !include inert\r\nA --> [*]\r\n@enduml";
        let mut budget = Budget::new(&RenderOptions::default());
        let document = crate::plantuml::preprocess::prepare(source, &mut budget).unwrap();
        let mut parsed = super::super::document::parse(&document, &mut budget).unwrap();
        super::super::preamble::parse(&mut parsed, &mut budget).unwrap();
        // when
        let result = parse(parsed.body, &mut budget).unwrap();
        // then
        assert_eq!(result.nodes[0].label, "Ω-->title");
        assert_eq!(result.nodes[0].descriptions, ["title: !include inert"]);
        assert_eq!(result.transitions.len(), 1);
    }

    #[test]
    fn should_map_state_failure_once_at_its_frozen_expansion_use() {
        // given
        let source = "\u{feff}@startuml\r\n!define BAD unknown\r\n!define UNUSED inert\r\nstate A\r\nA --> B BAD\r\n@enduml";
        let options = RenderOptions {
            language: crate::Language::PlantUml,
            ..RenderOptions::default()
        };
        // when
        let error = crate::render(source, &options).unwrap_err().to_string();
        // then
        assert!(
            error.contains("used at line 5, column 9, defined at line 2, column 1"),
            "{error}"
        );
        assert_eq!(error.matches("expansion \"BAD\"").count(), 1, "{error}");
        assert!(!error.contains("UNUSED"), "{error}");
    }

    #[test]
    fn should_locate_generated_state_suffix_after_an_inline_utf8_comment() {
        // given
        let body = "state A\nA /' Ω '/ --> B extra";
        // when
        let error = model(body).unwrap_err().to_string();
        // then
        assert!(error.contains("line 3, column 18"), "{error}");
    }

    #[test]
    fn should_bound_generated_state_source_depth_work_items_and_cancellation() {
        // given
        for resource in ["source", "depth", "work", "items", "cancel"] {
            let mut budget = Budget::new(&RenderOptions::default());
            let document = crate::plantuml::preprocess::prepare(
                "@startuml\nstate Outer {\nstate Inner begin\n[*] --> A\nendstate\n}\n@enduml",
                &mut budget,
            )
            .unwrap();
            let mut parsed = super::super::document::parse(&document, &mut budget).unwrap();
            super::super::preamble::parse(&mut parsed, &mut budget).unwrap();
            match resource {
                "source" => budget.limits.source_bytes = 4,
                "depth" => budget.limits.nesting_depth = 1,
                "work" => budget.limits.layout_work = budget.work + 1,
                "items" => budget.limits.model_items = 1,
                _ => budget.control.cancel_after_checkpoints(30),
            }
            // when
            let error = parse(parsed.body, &mut budget).unwrap_err();
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

    #[test]
    fn should_bound_generated_state_openers_at_the_actual_nesting_limit() {
        // given
        for depth in [64, 65] {
            let body = (0..depth)
                .map(|id| format!("state S{id} {{\n"))
                .collect::<String>()
                + &"}\n".repeat(depth);
            // when
            let result = model(&body);
            // then
            if depth == 64 {
                assert_eq!(result.unwrap().nodes.len(), 64);
            } else {
                assert!(result.unwrap_err().to_string().contains("nesting depth"));
            }
        }
    }

    #[test]
    fn should_cancel_generated_state_delivery_and_retaining_reductions() {
        // given
        for checkpoint in [1, 3, 5, 7, 9, 11] {
            let mut budget = Budget::new(&RenderOptions::default());
            let tokens = vec![
                Token::State,
                Token::Gap,
                Token::Name(Name {
                    text: "A".into(),
                    quoted: false,
                }),
                Token::Newline,
                Token::Name(Name {
                    text: "A".into(),
                    quoted: false,
                }),
                Token::Minus,
                Token::Greater,
                Token::Name(Name {
                    text: "B".into(),
                    quoted: false,
                }),
                Token::Newline,
            ]
            .into_iter()
            .enumerate()
            .map(|(i, token)| (i, token, i + 1))
            .collect::<Vec<_>>();
            let retained = admit((&tokens, tokens.capacity()), 0, &mut budget).unwrap();
            budget.control.cancel_after_checkpoints(checkpoint);
            let iterator = context::Checked::new(tokens, budget.control.clone());
            let mut builder = Builder::new(&mut budget, retained);
            // when
            let error = super::super::generated_state::StateParser::new()
                .parse(&mut builder, iterator)
                .unwrap_err();
            // then
            assert!(
                matches!(
                    error,
                    lalrpop_util::ParseError::User {
                        error: ParseFailure {
                            kind: super::super::error::FailureKind::Terminal(
                                RenderError::Cancelled(_)
                            ),
                            ..
                        }
                    }
                ),
                "checkpoint {checkpoint}: {error:?}"
            );
        }
    }

    #[test]
    fn should_bound_state_tokens_and_cancel_literal_copy_before_allocation() {
        // given
        let mut budget = Budget::new(&RenderOptions::default());
        budget.limits.source_bytes = 4;
        let mut lexer = Lexer {
            tokens: vec![],
            literal_bytes: 0,
            budget: &mut budget,
        };
        for _ in 0..4 {
            lexer.push(Token::Newline, 0..0).unwrap();
        }
        // when
        let result = lexer.push(Token::Newline, 0..0);
        // then
        assert!(result.is_err());
        assert_eq!(lexer.tokens.len(), 4);
        assert_eq!(lexer.tokens.capacity(), 4);

        // given
        let mut budget = Budget::new(&RenderOptions::default());
        budget.control.cancel_after_checkpoints(4);
        let mut lexer = Lexer {
            tokens: vec![],
            literal_bytes: 0,
            budget: &mut budget,
        };
        // when
        let error = lexer.copy(&"x".repeat(16 * 1024)).unwrap_err();
        // then
        assert!(matches!(
            error.kind,
            super::super::error::FailureKind::Terminal(RenderError::Cancelled(_))
        ));
        assert_eq!(lexer.tokens.capacity(), 0);
    }
}
