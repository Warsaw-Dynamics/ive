//! Generated Salt structure and direct lexical escapes.
use super::{
    BodySource, context,
    error::{ParseFailure, generated_failure},
    ownership::Row,
};
use crate::{
    RenderError,
    plantuml::{
        budget::{Budget, bound},
        salt::{Builder, Salt},
    },
};
use std::ops::Range;

#[derive(Clone, Debug)]
pub(in crate::plantuml) enum Token {
    Open,
    Outer,
    Grid,
    Close,
    Text(String),
    Caption(String),
    Tree(String),
    Quote,
    BracketOpen,
    BracketClose,
    ParenOpen,
    ParenClose,
    X,
    Space,
    Star,
    Column,
    Row,
}
type Spanned = (usize, Token, usize);

pub(in crate::plantuml) fn parse(
    body: BodySource<'_>,
    budget: &mut Budget,
) -> Result<Salt, RenderError> {
    parse_body(&body, budget).map_err(|error| error.into_document(body.document, budget))
}

fn parse_body(body: &BodySource<'_>, budget: &mut Budget) -> Result<Salt, ParseFailure> {
    bound(
        "Salt source bytes",
        body.range.len(),
        budget.limits.source_bytes,
    )
    .map_err(ParseFailure::terminal)?;
    budget
        .charge(std::mem::size_of::<super::ownership::Ownership>())
        .map_err(ParseFailure::terminal)?;
    let mut lexer = Lexer {
        tokens: Vec::new(),
        text: String::new(),
        literals: 0,
        adapters: body.adapters,
        depth: 0,
        maximum_depth: 0,
        budget,
    };
    let mut ownership = body.ownership;
    let mut base = body.range.start;
    for physical in body.document.text[body.range.clone()].split_inclusive('\n') {
        let raw = physical
            .strip_suffix('\n')
            .map_or(physical, |line| line.strip_suffix('\r').unwrap_or(line));
        let row = ownership.classify(raw, lexer.budget)?;
        ownership.advance(&row, &row.text, lexer.budget)?;
        lexer.row(&row, base)?;
        base += physical.len();
    }
    ownership.finish(base)?;
    let (retained, limit) = admit(&mut lexer)?;
    let mut builder = Builder::new(lexer.budget, retained, limit);
    let iterator = context::Checked::new(lexer.tokens, builder.control());
    super::generated_salt::SaltParser::new()
        .parse(&mut builder, iterator)
        .map_err(|error| {
            generated_failure(
                error,
                body.range.end,
                "invalid or incomplete Salt table/widget syntax",
            )
        })
}

struct Lexer<'b> {
    tokens: Vec<Spanned>,
    text: String,
    literals: usize,
    adapters: usize,
    depth: usize,
    maximum_depth: usize,
    budget: &'b mut Budget,
}

impl Lexer<'_> {
    fn storage(&self, extra: usize) -> Result<(), ParseFailure> {
        let bytes = self
            .adapters
            .saturating_add(self.literals)
            .saturating_add(self.text.capacity())
            .saturating_add(
                self.tokens
                    .capacity()
                    .saturating_mul(std::mem::size_of::<Spanned>()),
            )
            .saturating_add(extra);
        bound(
            "Salt lexer retained bytes",
            bytes,
            self.budget.limits.source_bytes.saturating_mul(256),
        )
        .map_err(ParseFailure::terminal)
    }

    fn push(&mut self, token: Token, span: Range<usize>) -> Result<(), ParseFailure> {
        bound(
            "Salt buffered tokens",
            self.tokens.len() + 1,
            self.budget.limits.source_bytes.saturating_mul(2),
        )
        .map_err(ParseFailure::terminal)?;
        self.budget.charge(1).map_err(ParseFailure::terminal)?;
        if self.tokens.len() == self.tokens.capacity() {
            let capacity = self
                .tokens
                .capacity()
                .saturating_add(self.tokens.capacity() / 2)
                .max(4);
            self.storage(capacity.saturating_mul(std::mem::size_of::<Spanned>()))?;
            self.budget
                .charge(self.tokens.len())
                .map_err(ParseFailure::terminal)?;
            self.tokens
                .try_reserve_exact(capacity - self.tokens.len())
                .map_err(|error| {
                    ParseFailure::terminal(RenderError::Backend(format!(
                        "PlantUML Salt token allocation for capacity {capacity} failed: {error}"
                    )))
                })?;
            self.storage(0)?;
        }
        self.tokens.push((span.start, token, span.end));
        Ok(())
    }

    fn append(&mut self, value: &str) -> Result<(), ParseFailure> {
        let next = self.text.len().saturating_add(value.len());
        bound("Salt token bytes", next, 16 * 1024 + 2).map_err(ParseFailure::terminal)?;
        self.budget
            .text(value.len())
            .map_err(ParseFailure::terminal)?;
        if next > self.text.capacity() {
            let capacity = self.text.capacity().saturating_mul(2).max(next).max(8);
            self.storage(capacity)?;
            self.budget
                .charge(self.text.len())
                .map_err(ParseFailure::terminal)?;
            self.text
                .try_reserve_exact(capacity - self.text.len())
                .map_err(|error| {
                    ParseFailure::terminal(RenderError::Backend(format!(
                        "PlantUML Salt lexical allocation for capacity {capacity} failed: {error}"
                    )))
                })?;
            self.storage(0)?;
        }
        self.text.push_str(value);
        Ok(())
    }

    fn copy(&mut self, value: &str) -> Result<String, ParseFailure> {
        bound("Salt token bytes", value.len(), 16 * 1024 + 2).map_err(ParseFailure::terminal)?;
        self.storage(value.len())?;
        self.budget
            .charge(value.len())
            .map_err(ParseFailure::terminal)?;
        self.literals = self.literals.saturating_add(value.len());
        Ok(value.into())
    }

    fn row(&mut self, row: &Row, base: usize) -> Result<(), ParseFailure> {
        let first = self.tokens.len();
        let mut cursor = 0;
        let mut start = 0;
        while cursor < row.text.len() {
            let rest = &row.text[cursor..];
            let ch = rest.chars().next().unwrap();
            self.budget
                .charge(ch.len_utf8() + 1)
                .map_err(ParseFailure::terminal)?;
            if ch == '\\'
                && let Some(next @ ('|' | '{' | '}')) = rest[1..].chars().next()
            {
                self.append(next.encode_utf8(&mut [0; 4]))?;
                cursor += 1 + next.len_utf8();
                continue;
            }
            if matches!(ch, '|' | '{' | '}') {
                self.flush(row.span(base, start..cursor))?;
                let structure_start = cursor;
                cursor += 1;
                if ch == '|' {
                    start = cursor;
                    continue;
                }
                let marker = row.text[cursor..]
                    .chars()
                    .next()
                    .filter(|ch| matches!(ch, '+' | '#' | '-' | '!' | '^' | '*' | '/' | 'S'));
                if ch == '{' {
                    self.budget
                        .depth(self.depth + 1)
                        .map_err(ParseFailure::terminal)?;
                    self.depth += 1;
                    self.maximum_depth = self.maximum_depth.max(self.depth);
                } else {
                    self.depth = self.depth.saturating_sub(1);
                }
                let token = match (ch, marker) {
                    ('}', _) => Token::Close,
                    ('{', Some('+')) => {
                        cursor += 1;
                        Token::Outer
                    }
                    ('{', Some('#')) => {
                        cursor += 1;
                        Token::Grid
                    }
                    ('{', Some(_)) => {
                        return Err(ParseFailure::unsupported(
                            row.span(base, cursor - 1..cursor + 1),
                            "Salt table opener",
                        ));
                    }
                    _ => Token::Open,
                };
                self.budget.item().map_err(ParseFailure::terminal)?;
                self.budget
                    .text((cursor - structure_start).saturating_mul(2))
                    .map_err(ParseFailure::terminal)?;
                self.push(token, row.span(base, structure_start..cursor))?;
                self.push(Token::Column, row.span(base, cursor..cursor))?;
                start = cursor;
            } else {
                self.append(ch.encode_utf8(&mut [0; 4]))?;
                cursor += ch.len_utf8();
            }
        }
        self.flush(row.span(base, start..cursor))?;
        if self.tokens.len() > first {
            let end = self.tokens.last_mut().unwrap();
            let tail = row.text.trim_end();
            end.1 = if tail.ends_with('|') && !tail.ends_with("\\|") {
                Token::Column
            } else {
                Token::Row
            };
        }
        Ok(())
    }

    fn flush(&mut self, span: Range<usize>) -> Result<(), ParseFailure> {
        self.budget
            .charge(self.text.len() + 1)
            .map_err(ParseFailure::terminal)?;
        if self.text.trim().is_empty() {
            self.text.clear();
            return Ok(());
        }
        let text = std::mem::take(&mut self.text);
        self.literals = self.literals.saturating_add(text.capacity());
        self.budget.item().map_err(ParseFailure::terminal)?;
        self.budget
            .text(text.trim().len())
            .map_err(ParseFailure::terminal)?;
        self.primitives(text.trim(), span.clone())?;
        self.push(Token::Column, span)
    }

    fn payload(&mut self, value: &str, span: Range<usize>) -> Result<(), ParseFailure> {
        let value = self.copy(value)?;
        self.push(Token::Text(value), span)
    }

    fn primitives(&mut self, text: &str, span: Range<usize>) -> Result<(), ParseFailure> {
        if super::preamble::has_marker(text) {
            return Err(ParseFailure::unsupported(
                span,
                "Salt styles/chrome must precede the table",
            ));
        }
        if text.starts_with("[[")
            || text.starts_with("<<")
            || (text.starts_with('^') && text.ends_with('^'))
            || matches!(text, "N=" | "S=" | "E=" | "W=" | "C=")
            || ["--", "==", "~~", ".."]
                .iter()
                .any(|marker| text.starts_with(marker) && text.ends_with(marker))
        {
            return Err(ParseFailure::unsupported(
                span,
                "Salt resource, dropdown or border-region token",
            ));
        }
        if let Some(tail) = text.strip_prefix('"') {
            self.push(Token::Quote, span.clone())?;
            self.payload(tail.strip_suffix('"').unwrap_or(tail), span.clone())?;
            if tail.ends_with('"') {
                self.push(Token::Quote, span)?;
            }
            return Ok(());
        }
        for (prefix, open, close) in [
            ("[X]", Token::BracketOpen, Token::BracketClose),
            ("[]", Token::BracketOpen, Token::BracketClose),
            ("[ ]", Token::BracketOpen, Token::BracketClose),
            ("(X)", Token::ParenOpen, Token::ParenClose),
            ("()", Token::ParenOpen, Token::ParenClose),
            ("( )", Token::ParenOpen, Token::ParenClose),
        ] {
            if let Some(tail) = text.strip_prefix(prefix) {
                self.push(open, span.clone())?;
                if prefix.as_bytes()[1] == b'X' {
                    self.push(Token::X, span.clone())?;
                } else if prefix.as_bytes()[1] == b' ' {
                    self.push(Token::Space, span.clone())?;
                }
                self.push(close, span.clone())?;
                let value = self.copy(tail.trim())?;
                return self.push(Token::Caption(value), span);
            }
        }
        if let Some(tail) = text.strip_prefix('[') {
            self.push(Token::BracketOpen, span.clone())?;
            self.payload(tail.strip_suffix(']').unwrap_or(tail), span.clone())?;
            if tail.ends_with(']') {
                self.push(Token::BracketClose, span)?;
            }
            return Ok(());
        }
        let token = match text {
            "*" => Token::Star,
            "T" | "T+" | "T#" | "T-" | "T!" | "T^" => Token::Tree(self.copy(text)?),
            _ => Token::Text(self.copy(text)?),
        };
        self.push(token, span)
    }
}

fn admit(lexer: &mut Lexer<'_>) -> Result<(usize, usize), ParseFailure> {
    // Actual 49-state/17-terminal tables: 3d+5 live states, seven
    // reductions and ten expected terminals; leave three extra states.
    let live = lexer.maximum_depth.saturating_mul(3).saturating_add(8);
    let capacity = live.next_power_of_two().max(8);
    let growing = capacity + capacity / 2;
    let payload = [
        std::mem::size_of::<Token>(),
        std::mem::size_of::<Salt>(),
        std::mem::size_of::<(usize, crate::plantuml::salt::End)>(),
    ]
    .into_iter()
    .max()
    .unwrap_or(0);
    let symbol = payload + 3 * std::mem::size_of::<usize>();
    let fixed = growing
        .saturating_mul(symbol + 1)
        .saturating_add(3 * (live + 1) + 32 * std::mem::size_of::<String>() + 17 * 32);
    let limit = lexer
        .budget
        .limits
        .source_bytes
        .saturating_mul(256)
        .saturating_add(fixed);
    let retained = lexer
        .tokens
        .capacity()
        .saturating_mul(std::mem::size_of::<Spanned>())
        .saturating_add(lexer.literals)
        .saturating_add(lexer.text.capacity())
        .saturating_add(lexer.adapters)
        .saturating_add(fixed)
        .saturating_add(
            std::mem::size_of::<Builder<'_>>()
                + std::mem::size_of::<Lexer<'_>>()
                + std::mem::size_of::<super::ownership::Ownership>(),
        );
    bound("Salt parser retained bytes", retained, limit).map_err(ParseFailure::terminal)?;
    let slots = growing
        .saturating_mul(2)
        .saturating_add(3 * (live + 1) + 32);
    bound(
        "Salt parser stack slots",
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
                .saturating_mul(8)
                .saturating_add(7 + slots + 17 * (live + 7)),
        )
        .map_err(ParseFailure::terminal)?;
    Ok((retained, limit))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        RenderOptions,
        plantuml::salt::{Border, Kind, LeafKind},
    };

    fn model(body: &str, options: &RenderOptions) -> Result<Salt, RenderError> {
        let source = format!("@startsalt\n{body}\n@endsalt");
        let mut budget = Budget::new(options);
        let document = crate::plantuml::preprocess::prepare(&source, &mut budget)?;
        let mut parsed = super::super::document::parse(&document, &mut budget)?;
        super::super::preamble::parse(&mut parsed, &mut budget)?;
        parse(parsed.body, &mut budget)
    }

    #[test]
    fn should_build_generated_salt_postorder_tables_controls_and_spans() {
        // given
        let body = "{#\nA | {+\n[X] | [x] | () radio\nWide | *\n}\n}";
        // when
        let model = model(body, &RenderOptions::default()).unwrap();
        // then
        let Kind::Table(root) = &model.widgets[model.root].kind else {
            panic!("{model:?}")
        };
        assert_eq!(
            (
                model.widgets[model.root].id,
                root.border,
                root.rows,
                root.columns
            ),
            (0, Border::Grid, 1, 2)
        );
        let Kind::Table(nested) = &model.widgets[root.cells[1].widget].kind else {
            panic!()
        };
        assert_eq!(
            (
                nested.border,
                nested.rows,
                nested.columns,
                nested.cells.len()
            ),
            (Border::Outer, 2, 3, 4)
        );
        assert_eq!((nested.cells[3].row, nested.cells[3].column_span), (1, 2));
        let leaves = model
            .widgets
            .iter()
            .filter_map(|widget| match &widget.kind {
                Kind::Leaf(leaf) => Some((leaf.kind, leaf.raw.as_str())),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            leaves,
            [
                (LeafKind::Text, "A"),
                (LeafKind::CheckboxOn, ""),
                (LeafKind::Button, "x"),
                (LeafKind::RadioOff, "radio"),
                (LeafKind::Text, "Wide")
            ]
        );
        for (index, widget) in model.widgets.iter().enumerate() {
            if let Kind::Table(table) = &widget.kind {
                assert!(table.cells.iter().all(|cell| cell.widget < index));
            }
        }
    }

    #[test]
    fn should_preserve_generated_salt_direct_escapes_blank_rows_and_widget_precedence() {
        // given
        let body = "{\n\n\\{ | \\} | \\| | [X] trailing] | [ ] off | (X) selected\n\n\"title: field  \" | [left header: button] | Text | T\n}";
        // when
        let model = model(body, &RenderOptions::default()).unwrap();
        // then
        let leaves = model
            .widgets
            .iter()
            .filter_map(|widget| match &widget.kind {
                Kind::Leaf(leaf) => Some((leaf.kind, leaf.raw.as_str())),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            leaves,
            [
                (LeafKind::Text, "{"),
                (LeafKind::Text, "}"),
                (LeafKind::Text, "|"),
                (LeafKind::CheckboxOn, "trailing]"),
                (LeafKind::CheckboxOff, "off"),
                (LeafKind::RadioOn, "selected"),
                (LeafKind::Field, "title: field  "),
                (LeafKind::Button, "left header: button"),
                (LeafKind::Text, "Text"),
                (LeafKind::Text, "T")
            ]
        );
        let Kind::Table(root) = &model.widgets[model.root].kind else {
            panic!()
        };
        assert_eq!((root.border, root.rows, root.columns), (Border::None, 2, 6));
    }

    #[test]
    fn should_reject_generated_salt_unpaired_tables_orphan_spans_and_late_chrome() {
        // given
        for body in [
            "{ A",
            "{ A }}",
            "{ A } extra",
            "{ }",
            "{ * | A }",
            "{ A\n*\n}",
            "{ T#\nRoot\n}",
            "{\nA\ntitle: late\n}",
            "{^ title\nA\n}",
        ] {
            // when
            let result = model(body, &RenderOptions::default());
            // then
            assert!(
                matches!(
                    result,
                    Err(RenderError::InvalidSource(_) | RenderError::UnsupportedFeature(_))
                ),
                "{body}: {result:?}"
            );
        }
    }

    fn raw_row(text: &str) -> Row {
        Row {
            text: text.into(),
            offsets: Vec::new(),
            kind: super::super::ownership::Kind::Content,
            literal: true,
            source_comment: false,
            opening_action: false,
            continuation: false,
            annotated_start: false,
        }
    }

    #[test]
    fn should_keep_escaped_braces_lexically_distinct_from_structural_tokens() {
        // given
        let options = RenderOptions::default();
        let mut budget = Budget::new(&options);
        let mut lexer = Lexer {
            tokens: Vec::new(),
            text: String::new(),
            literals: 0,
            adapters: 0,
            depth: 0,
            maximum_depth: 0,
            budget: &mut budget,
        };
        let row = raw_row("{ \\{ | \\} | \\| | \u{e000}\u{e001}\u{e002} }");
        // when
        lexer.row(&row, 0).unwrap();
        // then
        assert!(
            matches!(lexer.tokens.as_slice(), [(_, Token::Open, _), (_, Token::Column, _), (_, Token::Text(open), _), (_, Token::Column, _), (_, Token::Text(close), _), (_, Token::Column, _), (_, Token::Text(pipe), _), (_, Token::Column, _), (_, Token::Text(pua), _), (_, Token::Column, _), (_, Token::Close, _), (_, Token::Row, _)] if open == "{" && close == "}" && pipe == "|" && pua == "\u{e000}\u{e001}\u{e002}"),
            "{:?}",
            lexer.tokens
        );
    }

    #[test]
    fn should_bound_escape_accumulation_and_lexical_work_before_retention() {
        // given
        let options = RenderOptions::default();
        let mut budget = Budget::new(&options);
        let mut lexer = Lexer {
            tokens: Vec::new(),
            text: String::new(),
            literals: 0,
            adapters: 0,
            depth: 0,
            maximum_depth: 0,
            budget: &mut budget,
        };
        let row = raw_row(&"\\|".repeat(16 * 1024 + 3));
        // when
        let error = lexer.row(&row, 0).unwrap_err().into_line(1).to_string();
        // then
        assert!(error.contains("Salt token bytes"), "{error}");

        // given
        let mut options = RenderOptions::default();
        options.limits.layout_work = 5;
        let mut budget = Budget::new(&options);
        let mut lexer = Lexer {
            tokens: Vec::new(),
            text: String::new(),
            literals: 0,
            adapters: 0,
            depth: 0,
            maximum_depth: 0,
            budget: &mut budget,
        };
        // when
        let result = lexer.row(&raw_row("{ [Large token] }"), 0);
        // then
        assert!(result.is_err());
        assert!(lexer.budget.work <= 5);
    }

    #[test]
    fn should_enforce_salt_opener_depth_before_retaining_the_next_table() {
        // given
        let options = RenderOptions::default();
        for (depth, accepted) in [(64, true), (65, false)] {
            let body = format!("{}A{}", "{".repeat(depth), "}".repeat(depth));
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
            text: String::new(),
            literals: 0,
            adapters: 0,
            depth: 64,
            maximum_depth: 64,
            budget: &mut budget,
        };
        // when
        let result = lexer.row(&raw_row("{#"), 0);
        // then
        assert!(result.is_err());
        assert_eq!(
            (
                lexer.tokens.len(),
                lexer.tokens.capacity(),
                lexer.text.len()
            ),
            (0, 0, 0)
        );
    }

    #[test]
    fn should_cancel_salt_literal_scanning_and_generated_delivery_and_actions() {
        // given
        let mut budget = Budget::new(&RenderOptions::default());
        budget.control.cancel_after_checkpoints(4);
        let mut lexer = Lexer {
            tokens: Vec::new(),
            text: String::new(),
            literals: 0,
            adapters: 0,
            depth: 0,
            maximum_depth: 0,
            budget: &mut budget,
        };
        // when
        let result = lexer.row(&raw_row(&"\\|".repeat(1024)), 0);
        // then
        assert!(matches!(
            result,
            Err(ParseFailure {
                kind: super::super::error::FailureKind::Terminal(RenderError::Cancelled(_)),
                ..
            })
        ));
        assert!(lexer.tokens.is_empty());
        for checkpoint in 1..=8 {
            // given
            let mut budget = Budget::new(&RenderOptions::default());
            let stream = vec![
                (0, Token::Open, 1),
                (1, Token::Column, 1),
                (2, Token::Text("A".into()), 3),
                (3, Token::Column, 3),
                (4, Token::Close, 5),
                (5, Token::Row, 5),
            ];
            budget.control.cancel_after_checkpoints(checkpoint);
            let iterator = context::Checked::new(stream, budget.control.clone());
            let mut builder = Builder::new(
                &mut budget,
                0,
                RenderOptions::default().limits.source_bytes * 256,
            );
            // when
            let result =
                super::super::generated_salt::SaltParser::new().parse(&mut builder, iterator);
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
    fn should_require_complete_generated_salt_widget_and_table_syntax() {
        // given
        for stream in [
            vec![(0, Token::Close, 1), (1, Token::Row, 1)],
            vec![
                (0, Token::Open, 1),
                (1, Token::Column, 1),
                (2, Token::Quote, 3),
                (3, Token::Text("open".into()), 7),
                (7, Token::Column, 7),
                (8, Token::Close, 9),
                (9, Token::Row, 9),
            ],
            vec![
                (0, Token::Open, 1),
                (1, Token::Column, 1),
                (2, Token::BracketOpen, 3),
                (3, Token::X, 4),
                (4, Token::ParenClose, 5),
            ],
        ] {
            let mut budget = Budget::new(&RenderOptions::default());
            let iterator = context::Checked::new(stream, budget.control.clone());
            let mut builder = Builder::new(
                &mut budget,
                0,
                RenderOptions::default().limits.source_bytes * 256,
            );
            // when
            let result =
                super::super::generated_salt::SaltParser::new().parse(&mut builder, iterator);
            // then
            assert!(result.is_err(), "{result:?}");
        }
    }

    #[test]
    fn should_admit_salt_default_depth_at_its_exact_source_quota() {
        // given
        let body = format!("{}A{}", "{".repeat(64), "}".repeat(64));
        let mut options = RenderOptions::default();
        options.limits.source_bytes = format!("@startsalt\n{body}\n@endsalt").len();
        // when
        let result = model(&body, &options);
        // then
        assert!(result.is_ok(), "{result:?}");
    }

    #[test]
    fn should_admit_salt_work_and_retention_before_generated_allocation() {
        // given
        for resource in ["work", "storage"] {
            let mut options = RenderOptions::default();
            if resource == "work" {
                options.limits.layout_work = 1;
            } else {
                options.limits.source_bytes = 1;
            }
            let mut budget = Budget::new(&options);
            let mut lexer = Lexer {
                tokens: Vec::new(),
                text: String::new(),
                literals: 0,
                adapters: 300,
                depth: 0,
                maximum_depth: 64,
                budget: &mut budget,
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
    fn should_bound_salt_retained_text_before_emitting_the_widget_payload() {
        // given
        let mut options = RenderOptions::default();
        options.limits.source_bytes = 1;
        let mut budget = Budget::new(&options);
        let mut lexer = Lexer {
            tokens: Vec::new(),
            text: String::new(),
            literals: 0,
            adapters: 0,
            depth: 0,
            maximum_depth: 0,
            budget: &mut budget,
        };
        // when
        let error = lexer
            .row(&raw_row("Long retained text"), 0)
            .unwrap_err()
            .into_line(1)
            .to_string();
        // then
        assert!(error.contains("expanded model text bytes"), "{error}");
        assert!(lexer.tokens.is_empty());
        assert_eq!(lexer.text.len(), 8);
    }

    #[test]
    fn should_map_salt_span_and_expanded_late_chrome_failures_once() {
        // given
        for (body, expected) in [
            ("{ A\n*\n}", "line 3, column 1"),
            (
                "!define LATE title: late\n{\nA\nLATE\n}",
                "used at line 5, column 1",
            ),
        ] {
            // when
            let error = model(body, &RenderOptions::default())
                .unwrap_err()
                .to_string();
            // then
            assert!(error.contains(expected), "{error}");
            if body.starts_with("!define") {
                assert!(error.contains("defined at line 2, column 1"), "{error}");
                assert_eq!(error.matches("used at line").count(), 1, "{error}");
            }
        }
    }
}
