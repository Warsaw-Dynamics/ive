//! Generated Chen structure and contextual lexical primitives.
use super::{
    BodySource, context,
    error::{ParseFailure, generated_failure},
    ownership::Row,
};
use crate::{
    RenderError,
    plantuml::{
        budget::{Budget, bound},
        chen::{Builder, Chen, Name},
    },
};
use std::ops::Range;

#[derive(Clone, Debug)]
pub(in crate::plantuml) enum Token {
    Entity(String),
    Relationship(String),
    As(String),
    Word(String),
    Quoted(String),
    Space(String),
    Open,
    Close,
    Minus,
    Equal,
    ParenOpen,
    ParenClose,
    Comma,
    Newline,
}
type Spanned = (usize, Token, usize);

pub(in crate::plantuml) fn parse(
    body: BodySource<'_>,
    budget: &mut Budget,
) -> Result<Chen, RenderError> {
    parse_body(&body, budget).map_err(|error| error.into_document(body.document, budget))
}

fn parse_body(body: &BodySource<'_>, budget: &mut Budget) -> Result<Chen, ParseFailure> {
    bound(
        "Chen source bytes",
        body.range.len(),
        budget.limits.source_bytes,
    )
    .map_err(ParseFailure::terminal)?;
    budget
        .charge(std::mem::size_of::<super::ownership::Ownership>())
        .map_err(ParseFailure::terminal)?;
    let mut lexer = Lexer {
        tokens: Vec::new(),
        budget,
        literals: 0,
        depth: 0,
        maximum_depth: 0,
        adapters: body.adapters,
    };
    let mut ownership = body.ownership;
    let mut base = body.range.start;
    for physical in body.document.text[body.range.clone()].split_inclusive('\n') {
        let raw = physical
            .strip_suffix('\n')
            .map_or(physical, |row| row.strip_suffix('\r').unwrap_or(row));
        let row = ownership.classify(raw, lexer.budget)?;
        ownership.advance(&row, &row.text, lexer.budget)?;
        lexer.row(&row, base)?;
        base += physical.len();
    }
    ownership.finish(base)?;
    let retained = admit(&mut lexer)?;
    let mut builder = Builder::new(lexer.budget, retained);
    let iterator = context::Checked::new(lexer.tokens, builder.control());
    super::generated_chen::ChenParser::new()
        .parse(&mut builder, iterator)
        .map_err(|error| {
            generated_failure(error, body.range.end, "invalid or incomplete Chen syntax")
        })?;
    Ok(builder.into_model())
}

struct Lexer<'b> {
    tokens: Vec<Spanned>,
    budget: &'b mut Budget,
    literals: usize,
    depth: usize,
    maximum_depth: usize,
    adapters: usize,
}

impl Lexer<'_> {
    fn storage(&self, extra: usize) -> Result<(), ParseFailure> {
        let bytes = self
            .adapters
            .saturating_add(self.literals)
            .saturating_add(
                self.tokens
                    .capacity()
                    .saturating_mul(std::mem::size_of::<Spanned>()),
            )
            .saturating_add(extra);
        bound(
            "Chen lexer retained bytes",
            bytes,
            self.budget.limits.source_bytes.saturating_mul(256),
        )
        .map_err(ParseFailure::terminal)
    }

    fn push(&mut self, token: Token, span: Range<usize>) -> Result<(), ParseFailure> {
        if self.tokens.len() == self.tokens.capacity() {
            self.storage(
                self.tokens
                    .capacity()
                    .saturating_mul(2)
                    .max(4)
                    .saturating_mul(std::mem::size_of::<Spanned>()),
            )?;
        }
        context::push(&mut self.tokens, (span.start, token, span.end), self.budget)
    }

    fn copy(&mut self, value: &str) -> Result<String, ParseFailure> {
        bound("label bytes", value.len(), 16 * 1024).map_err(ParseFailure::terminal)?;
        self.storage(value.len())?;
        self.budget
            .charge(value.len())
            .map_err(ParseFailure::terminal)?;
        self.keep_literal(value.len())?;
        Ok(value.into())
    }

    fn keep_literal(&mut self, bytes: usize) -> Result<(), ParseFailure> {
        let next = self.literals.saturating_add(bytes);
        bound(
            "Chen syntax literal bytes",
            next,
            self.budget.limits.source_bytes.saturating_mul(2),
        )
        .map_err(ParseFailure::terminal)?;
        self.literals = next;
        Ok(())
    }

    fn row(&mut self, row: &Row, base: usize) -> Result<(), ParseFailure> {
        let text = row.text.trim();
        if text.is_empty() {
            return Ok(());
        }
        if text.ends_with('{') {
            self.budget
                .depth(self.depth + 1)
                .map_err(ParseFailure::terminal)?;
        }
        let start = row.text.len() - row.text.trim_start().len();
        let end = row.text.trim_end().len();
        let mut cursor = start;
        let mut markers = 0usize;
        let mut marker = '\0';
        let first_end = self.scan(text, |ch: char| {
            ch.is_whitespace() || matches!(ch, '{' | '}' | '-' | '=' | '"')
        })?;
        let first = &text[..first_end];
        let suffix = text[first_end..].trim_start();
        let syntax_items = self.depth > 0
            || (matches!(first, "entity" | "relationship") && !suffix.starts_with(['-', '=']));
        while cursor < end {
            let rest = &row.text[cursor..end];
            let ch = rest.chars().next().unwrap();
            if ch.is_whitespace() {
                let bytes = self.scan(rest, |ch| !ch.is_whitespace())?;
                if markers == 1 {
                    let value = self.copy(&rest[..bytes])?;
                    self.push(Token::Space(value), row.span(base, cursor..cursor + bytes))?;
                }
                cursor += bytes;
                continue;
            }
            if syntax_items && ch != '}' {
                self.budget.item().map_err(ParseFailure::terminal)?;
            }
            let (token, bytes) = match ch {
                '"' => {
                    let (value, bytes) =
                        self.quoted(rest, row.span(base, cursor..row.text.len()))?;
                    (Token::Quoted(value), bytes)
                }
                '{' => {
                    self.budget
                        .depth(self.depth + 1)
                        .map_err(ParseFailure::terminal)?;
                    self.depth += 1;
                    self.maximum_depth = self.maximum_depth.max(self.depth);
                    (Token::Open, 1)
                }
                '}' => {
                    self.depth = self.depth.saturating_sub(1);
                    (Token::Close, 1)
                }
                '-' => {
                    if markers == 1 && marker != '-' {
                        return Err(ParseFailure::unsupported(
                            row.span(base, cursor..cursor + 1),
                            "mismatched Chen participation markers",
                        ));
                    }
                    marker = '-';
                    markers += 1;
                    (Token::Minus, 1)
                }
                '=' => {
                    if markers == 1 && marker != '=' {
                        return Err(ParseFailure::unsupported(
                            row.span(base, cursor..cursor + 1),
                            "mismatched Chen participation markers",
                        ));
                    }
                    marker = '=';
                    markers += 1;
                    (Token::Equal, 1)
                }
                '(' => (Token::ParenOpen, 1),
                ')' => (Token::ParenClose, 1),
                ',' => (Token::Comma, 1),
                _ => {
                    let bytes = self.scan(rest, |ch| {
                        ch.is_whitespace()
                            || matches!(ch, '"' | '{' | '}' | '-' | '=' | '(' | ')' | ',')
                    })?;
                    let word = &rest[..bytes];
                    if word
                        .chars()
                        .any(|ch| !(ch.is_alphanumeric() || matches!(ch, '_' | '.')))
                    {
                        return Err(ParseFailure::unsupported(
                            row.span(base, cursor..cursor + bytes),
                            "Chen alias or unported decoration/subtype",
                        ));
                    }
                    let value = self.copy(word)?;
                    let token = match word {
                        "entity"
                            if cursor != start
                                || rest[bytes..].chars().next().is_none_or(char::is_whitespace) =>
                        {
                            Token::Entity(value)
                        }
                        "relationship"
                            if cursor != start
                                || rest[bytes..].chars().next().is_none_or(char::is_whitespace) =>
                        {
                            Token::Relationship(value)
                        }
                        "entity" | "relationship" if self.depth > 0 => {
                            if word == "entity" {
                                Token::Entity(value)
                            } else {
                                Token::Relationship(value)
                            }
                        }
                        "as" => Token::As(value),
                        _ => Token::Word(value),
                    };
                    (token, bytes)
                }
            };
            self.push(token, row.span(base, cursor..cursor + bytes))?;
            cursor += bytes;
        }
        self.push(Token::Newline, row.span(base, cursor..cursor))
    }

    fn scan(&mut self, value: &str, stop: impl Fn(char) -> bool) -> Result<usize, ParseFailure> {
        let mut pending = 0;
        for (index, ch) in value.char_indices() {
            if stop(ch) {
                self.budget
                    .charge(pending)
                    .map_err(ParseFailure::terminal)?;
                return Ok(index);
            }
            pending += ch.len_utf8();
            if pending >= 256 {
                self.budget
                    .charge(pending)
                    .map_err(ParseFailure::terminal)?;
                pending = 0;
            }
        }
        self.budget
            .charge(pending)
            .map_err(ParseFailure::terminal)?;
        Ok(value.len())
    }

    fn quoted(
        &mut self,
        source: &str,
        span: Range<usize>,
    ) -> Result<(String, usize), ParseFailure> {
        let mut value = String::new();
        let mut escaped = false;
        for (index, ch) in source.char_indices().skip(1) {
            self.budget
                .charge(ch.len_utf8())
                .map_err(ParseFailure::terminal)?;
            bound("label bytes", index - 1, 16 * 1024).map_err(ParseFailure::terminal)?;
            if ch != '"' || escaped {
                bound("label bytes", index - 1 + ch.len_utf8(), 16 * 1024)
                    .map_err(ParseFailure::terminal)?;
            }
            if escaped {
                match ch {
                    'n' | '\\' | '~' => {
                        self.append_quote(&mut value, "\\")?;
                        self.append_quote(&mut value, ch.encode_utf8(&mut [0; 4]))?;
                    }
                    '"' => self.append_quote(&mut value, "\"")?,
                    _ => return Err(ParseFailure::unsupported(span, "quoted Chen escape")),
                }
                escaped = false;
            } else {
                match ch {
                    '\\' => escaped = true,
                    '"' => {
                        self.keep_literal(value.capacity())?;
                        return Ok((value, index + 1));
                    }
                    _ => self.append_quote(&mut value, ch.encode_utf8(&mut [0; 4]))?,
                }
            }
        }
        Err(ParseFailure::syntax(span, "unterminated quoted identifier"))
    }

    fn append_quote(&mut self, value: &mut String, part: &str) -> Result<(), ParseFailure> {
        let next = value.len() + part.len();
        bound("label bytes", next, 16 * 1024).map_err(ParseFailure::terminal)?;
        if next > value.capacity() {
            let capacity = value.capacity().saturating_mul(2).max(next).max(8);
            self.storage(value.capacity().saturating_add(capacity))?;
            self.budget
                .charge(value.len())
                .map_err(ParseFailure::terminal)?;
            value
                .try_reserve_exact(capacity - value.len())
                .map_err(|error| {
                    ParseFailure::terminal(RenderError::Backend(format!(
                        "PlantUML Chen quoted allocation for capacity {capacity} failed: {error}"
                    )))
                })?;
        }
        value.push_str(part);
        Ok(())
    }
}

fn admit(lexer: &mut Lexer<'_>) -> Result<usize, ParseFailure> {
    // Actual 50-state/14-terminal tables have at most 2d+10 live states,
    // four consecutive reductions and seven expected terminals.
    let live = lexer.maximum_depth.saturating_mul(2).saturating_add(12);
    let capacity = live.next_power_of_two().max(8);
    let growing = capacity + capacity / 2;
    let symbol = std::mem::size_of::<Token>()
        + std::mem::size_of::<Name>()
        + 8 * std::mem::size_of::<usize>();
    let retained = lexer
        .tokens
        .capacity()
        .saturating_mul(std::mem::size_of::<Spanned>())
        .saturating_add(lexer.literals)
        .saturating_add(lexer.adapters)
        .saturating_add(growing.saturating_mul(symbol + 1))
        .saturating_add(3 * (live + 1) + 16 * std::mem::size_of::<String>() + 14 * 32)
        .saturating_add(
            std::mem::size_of::<Builder<'_>>()
                + std::mem::size_of::<Lexer<'_>>()
                + std::mem::size_of::<super::ownership::Ownership>(),
        );
    bound(
        "Chen parser retained bytes",
        retained,
        lexer.budget.limits.source_bytes.saturating_mul(256),
    )
    .map_err(ParseFailure::terminal)?;
    let slots = growing
        .saturating_mul(2)
        .saturating_add(3 * (live + 1) + 16);
    bound(
        "Chen parser stack slots",
        slots,
        lexer.budget.limits.layout_work,
    )
    .map_err(ParseFailure::terminal)?;
    lexer
        .budget
        .charge(
            lexer
                .tokens
                .len()
                .saturating_mul(5)
                .saturating_add(4 + slots + 14 * (live + 4)),
        )
        .map_err(ParseFailure::terminal)?;
    Ok(retained)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        RenderOptions,
        plantuml::chen::{AttributeOwner, ChenNode, Link, Participation},
    };

    fn model(body: &str, options: &RenderOptions) -> Result<Chen, RenderError> {
        crate::plantuml::parser::test_support::parse(
            &format!("@startchen\n{body}\n@endchen"),
            &mut Budget::new(options),
            crate::plantuml::parser::chen::parse,
        )
    }

    #[test]
    fn should_build_generated_chen_identity_ownership_and_parallel_self_associations() {
        // given
        let body = "entity entity {\nentity{\nleaf\n}\n}\nrelationship relationship {\nattribute\n}\nentity -(0, N)- relationship\nentity =N= relationship\nentity -- entity";
        // when
        let model = model(body, &RenderOptions::default()).unwrap();
        // then
        assert_eq!(
            model
                .nodes
                .iter()
                .map(|node| node.key.as_str())
                .collect::<Vec<_>>(),
            [
                "entity",
                "entity/entity",
                "entity/entity/leaf",
                "relationship",
                "relationship/attribute"
            ]
        );
        assert_eq!(
            model.nodes[2].kind,
            ChenNode::Attribute(AttributeOwner::Attribute(1))
        );
        assert_eq!(
            model.nodes[4].kind,
            ChenNode::Attribute(AttributeOwner::Relationship(3))
        );
        assert_eq!(
            model.links.iter().map(Link::endpoints).collect::<Vec<_>>(),
            [(0, 1), (1, 2), (3, 4), (0, 3), (0, 3), (0, 0)]
        );
        assert_eq!(model.links[3].cardinality(), "(0, N)");
        assert!(matches!(
            model.links[4],
            Link::Association {
                participation: Participation::Bold,
                ..
            }
        ));
    }

    #[test]
    fn should_reject_generated_chen_malformed_structures_and_all_unported_decorations() {
        // given
        for body in [
            "entity A {\nentity B {\n}\n}",
            "entity A {\nx",
            "entity A {\n}\n}",
            "entity A <<weak>> {\n}",
            "relationship R <<identifying>> {\n}",
            "entity A {\nx <<key>>\n}",
            "entity A {\nx <<multi>>\n}",
            "entity A {\nx <<derived>>\n}",
            "entity A {\n}\nA ->- d {A,A}",
            "entity A {\n}\nA =1- A",
            "entity A {\n}\nA - 1 - A",
            "entity A {\n}\nA -(0, N )- A",
            "entity\"label\" as A {\n}",
            "entity A {\n}\nA -- A trailing",
        ] {
            // when
            let result = model(body, &RenderOptions::default());
            // then
            assert!(result.is_err(), "accepted {body}: {result:?}");
        }
    }

    #[test]
    fn should_enforce_chen_depth_before_retaining_the_next_opener() {
        // given
        let options = RenderOptions::default();
        for (depth, accepted) in [(64, true), (65, false)] {
            let body = format!(
                "entity C {{\n{}leaf\n{}",
                "x{\n".repeat(depth - 1),
                "}\n".repeat(depth)
            );
            // when
            let result = model(&body, &options);
            // then
            assert_eq!(result.is_ok(), accepted, "{result:?}");
            if let Err(error) = result {
                assert!(error.to_string().contains("nesting depth"), "{error}");
            }
        }
        let mut budget = Budget::new(&options);
        let mut lexer = Lexer {
            tokens: Vec::new(),
            budget: &mut budget,
            literals: 0,
            depth: 64,
            maximum_depth: 64,
            adapters: 0,
        };
        let row = super::super::ownership::Ownership::default()
            .classify("entity{", lexer.budget)
            .unwrap();
        // when
        let result = lexer.row(&row, 0);
        // then
        assert!(result.is_err());
        assert_eq!(
            (lexer.tokens.capacity(), lexer.tokens.len(), lexer.literals),
            (0, 0, 0)
        );
    }

    #[test]
    fn should_map_chen_failures_through_comment_gaps_and_frozen_definition_uses() {
        // given
        for (body, expected) in [
            ("entity A {\n}\nA -- /'gap'/ Missing", "line 4, column 1"),
            (
                "!define MISSING Absent\nentity A {\n}\nA -- MISSING",
                "used at line 5, column 6",
            ),
        ] {
            // when
            let error = model(body, &RenderOptions::default())
                .unwrap_err()
                .to_string();
            // then
            assert!(error.contains(expected), "{error}");
        }
    }

    #[test]
    fn should_cancel_chen_generated_delivery_and_retaining_actions() {
        // given
        for checkpoint in 1..=8 {
            let stream = vec![
                (0, Token::Entity("entity".into()), 6),
                (7, Token::Word("A".into()), 8),
                (9, Token::Open, 10),
                (10, Token::Newline, 11),
                (11, Token::Close, 12),
                (12, Token::Newline, 13),
            ];
            let mut budget = Budget::new(&RenderOptions::default());
            budget.control.cancel_after_checkpoints(checkpoint);
            let iterator = context::Checked::new(stream, budget.control.clone());
            let mut builder = Builder::new(&mut budget, 0);
            // when
            let result =
                super::super::generated_chen::ChenParser::new().parse(&mut builder, iterator);
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
    fn should_require_generated_chen_pairing_continuations_and_eof() {
        // given
        for stream in [
            vec![(0, Token::Close, 1), (1, Token::Newline, 2)],
            vec![
                (0, Token::Entity("entity".into()), 6),
                (7, Token::Word("A".into()), 8),
                (9, Token::Open, 10),
                (10, Token::Newline, 11),
            ],
            vec![
                (0, Token::Entity("entity".into()), 6),
                (7, Token::Word("A".into()), 8),
                (9, Token::Open, 10),
                (10, Token::Newline, 11),
                (11, Token::Close, 12),
                (12, Token::Newline, 13),
                (13, Token::Word("suffix".into()), 19),
            ],
        ] {
            let mut budget = Budget::new(&RenderOptions::default());
            let iterator = context::Checked::new(stream, budget.control.clone());
            let mut builder = Builder::new(&mut budget, 0);
            // when
            let result =
                super::super::generated_chen::ChenParser::new().parse(&mut builder, iterator);
            // then
            assert!(result.is_err(), "{result:?}");
        }
    }

    #[test]
    fn should_admit_chen_storage_and_work_before_generated_allocation() {
        // given
        for resource in ["storage", "work"] {
            let mut options = RenderOptions::default();
            if resource == "storage" {
                options.limits.source_bytes = 1;
            } else {
                options.limits.layout_work = 1;
            }
            let mut budget = Budget::new(&options);
            let mut lexer = Lexer {
                tokens: Vec::new(),
                budget: &mut budget,
                literals: 0,
                depth: 0,
                maximum_depth: 64,
                adapters: 0,
            };
            // when
            let result = admit(&mut lexer);
            // then
            assert!(
                matches!(
                    result,
                    Err(ParseFailure {
                        kind: super::super::error::FailureKind::Terminal(RenderError::Backend(_)),
                        ..
                    })
                ),
                "{result:?}"
            );
        }
    }

    #[test]
    fn should_charge_chen_primitives_before_unterminated_quote_retention() {
        // given
        let mut options = RenderOptions::default();
        options.limits.model_items = 1;
        let mut budget = Budget::new(&options);
        let row = super::super::ownership::Ownership::default()
            .classify("entity \"unterminated", &mut budget)
            .unwrap();
        let mut lexer = Lexer {
            tokens: Vec::new(),
            budget: &mut budget,
            literals: 0,
            depth: 0,
            maximum_depth: 0,
            adapters: 0,
        };
        // when
        let result = lexer.row(&row, 0);
        // then
        assert!(
            matches!(result, Err(ParseFailure { kind: super::super::error::FailureKind::Terminal(RenderError::Backend(ref message)), .. }) if message.contains("model items value 2 exceeds limit 1"))
        );
        assert_eq!(lexer.tokens.len(), 1);
        let builder = Builder::new(lexer.budget, 0);
        assert!(builder.model.nodes.is_empty());
        assert!(builder.model.links.is_empty());
    }

    #[test]
    fn should_bound_raw_chen_quote_bytes_and_cancel_before_literal_retention() {
        // given
        for (count, accepted) in [(8192, true), (8193, false)] {
            let source = format!("\"{}\"", "\\\"".repeat(count));
            let mut budget = Budget::new(&RenderOptions::default());
            let mut lexer = Lexer {
                tokens: Vec::new(),
                budget: &mut budget,
                literals: 0,
                depth: 0,
                maximum_depth: 0,
                adapters: 0,
            };
            // when
            let result = lexer.quoted(&source, 0..source.len());
            // then
            assert_eq!(result.is_ok(), accepted, "{result:?}");
        }
        let source = format!("\"{}\"", "x".repeat(4096));
        let mut budget = Budget::new(&RenderOptions::default());
        budget.control.cancel_after_checkpoints(4);
        let mut lexer = Lexer {
            tokens: Vec::new(),
            budget: &mut budget,
            literals: 0,
            depth: 0,
            maximum_depth: 0,
            adapters: 0,
        };
        // when
        let result = lexer.quoted(&source, 0..source.len());
        // then
        assert!(matches!(
            result,
            Err(ParseFailure {
                kind: super::super::error::FailureKind::Terminal(RenderError::Cancelled(_)),
                ..
            })
        ));
        assert_eq!((lexer.tokens.len(), lexer.literals), (0, 0));
    }
}
