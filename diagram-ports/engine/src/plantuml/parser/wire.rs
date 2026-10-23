//! Generated offside hierarchy, component/size/column and dotted wire statement syntax.
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
        style::Color,
        wire::{Builder, Direction, LinkType, Orientation, Style, Wire},
    },
};
#[derive(Clone, Debug)]
pub(in crate::plantuml) enum Token {
    Star,
    Column,
    Open,
    Close,
    Times,
    Dot,
    Colon,
    Number(usize),
    Name(String),
    Arrow(Style),
    Color(Color),
    Label(String),
    End,
    Indent,
    Dedent,
}
type Spanned = (usize, Token, usize);
pub(in crate::plantuml) fn parse(
    body: BodySource<'_>,
    budget: &mut Budget,
) -> Result<Wire, RenderError> {
    parse_body(&body, budget).map_err(|error| diagnose(error, &body, budget))
}
fn parse_body(body: &BodySource<'_>, budget: &mut Budget) -> Result<Wire, ParseFailure> {
    bound(
        "wire lexical retained bytes",
        body.range
            .len()
            .saturating_add(1)
            .saturating_mul(3 * std::mem::size_of::<Spanned>() + 4)
            .saturating_add(body.adapters),
        budget.limits.source_bytes.saturating_mul(256),
    )?;
    let mut level = 0usize;
    let mut tokens = outline::lex(body, budget, |row, base, lexer| {
        row_tokens((row, base), &mut level, lexer)
    })?;
    while level > 0 {
        context::push(
            &mut tokens,
            (body.range.end, Token::Dedent, body.range.end),
            budget,
        )?;
        level -= 1;
    }
    admit((&tokens, tokens.capacity()), body, budget)?;
    let iterator = context::Checked::new(tokens, budget.control.clone());
    let mut builder = Builder::new(budget, body.range.clone())?;
    super::generated_wire::WireParser::new()
        .parse(&mut builder, iterator)
        .map_err(|error| {
            generated_failure(
                error,
                body.range.end,
                "invalid or unsupported wire component, indentation, endpoint, arrow or suffix",
            )
        })?;
    builder.finish(body.range.end..body.range.end)
}
fn row_tokens(
    site: (&Row, usize),
    level: &mut usize,
    lexer: &mut outline::Lexer<'_, Token>,
) -> Result<(), ParseFailure> {
    let (row, base) = site;
    let text = &row.text;
    let trimmed = text.trim_matches([' ', '\t']);
    if matches!(row.kind, Kind::Empty | Kind::Trivia | Kind::Definition)
        || trimmed.starts_with('\'')
    {
        return Ok(());
    }
    let leading = text.len() - text.trim_start_matches([' ', '\t']).len();
    let depth = indent(&text[..leading], lexer.budget)?;
    lexer.budget.depth(depth + 1)?;
    if depth > *level + 1 {
        return Err(ParseFailure::unsupported(
            row.span(base, 0..leading),
            "wire indentation skips a missing parent level",
        ));
    }
    if depth > *level {
        lexer.push(Token::Indent, row.span(base, 0..leading))?;
        *level = depth;
    }
    while depth < *level {
        lexer.push(Token::Dedent, row.span(base, 0..leading))?;
        *level -= 1;
    }
    if trimmed.bytes().all(|b| b == b'-') {
        lexer.budget.charge(trimmed.len())?;
        lexer.push(
            Token::Column,
            row.span(base, leading..leading + trimmed.len()),
        )?;
    } else {
        scan(row, base, lexer)?;
    }
    lexer.push(Token::End, row.span(base, text.len()..text.len()))
}
fn indent(text: &str, budget: &mut Budget) -> Result<usize, ParseFailure> {
    let mut spaces = 0usize;
    let mut depth = 0usize;
    for byte in text.bytes() {
        budget.charge(1)?;
        if byte == b'\t' {
            depth += spaces / 4 + spaces % 4 + 1;
            spaces = 0;
        } else {
            spaces += 1;
        }
    }
    Ok(depth + spaces / 4 + spaces % 4)
}
fn scan(row: &Row, base: usize, lexer: &mut outline::Lexer<'_, Token>) -> Result<(), ParseFailure> {
    let text = &row.text;
    let mut offset = 0;
    let mut dimensions = false;
    while offset < text.len() {
        lexer.budget.charge(1)?;
        let rest = &text[offset..];
        let ch = rest.chars().next().unwrap();
        if matches!(ch, ' ' | '\t') {
            offset += 1;
            continue;
        }
        if ch == ':' {
            lexer.push(Token::Colon, row.span(base, offset..offset + 1))?;
            let label = rest[1..].trim_start();
            let start = text.len() - label.len();
            let value = lexer.copy_label(label)?;
            lexer.push(Token::Label(value), row.span(base, start..text.len()))?;
            return Ok(());
        }
        let (token, used) = match ch {
            '*' => (Token::Star, 1),
            '[' => {
                dimensions = true;
                (Token::Open, 1)
            }
            ']' => {
                dimensions = false;
                (Token::Close, 1)
            }
            '.' => (Token::Dot, 1),
            'x' if dimensions => (Token::Times, 1),
            '#' => {
                let end = rest
                    .find(|ch: char| ch.is_whitespace() || ch == ':')
                    .unwrap_or(rest.len());
                let hex = &rest[1..end];
                if hex.len() != 6 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
                    return Err(ParseFailure::unsupported(
                        row.span(base, offset..offset + end),
                        "wire colors require #RRGGBB",
                    ));
                }
                (
                    Token::Color(Color(u32::from_str_radix(hex, 16).unwrap())),
                    end,
                )
            }
            '-' | '<' => {
                let (style, used) = arrow(rest).ok_or_else(|| {
                    ParseFailure::unsupported(
                        row.span(base, offset..offset + ch.len_utf8()),
                        "unsupported wire arrow or bus",
                    )
                })?;
                (Token::Arrow(style), used)
            }
            _ if dimensions && ch.is_ascii_digit() => {
                let mut used = 0;
                for byte in rest.bytes() {
                    if !byte.is_ascii_digit() {
                        break;
                    }
                    lexer.budget.charge(1)?;
                    used += 1;
                    if used > 6 {
                        return Err(ParseFailure::unsupported(
                            row.span(base, offset..offset + used),
                            "wire dimension numeric token exceeds six digits",
                        ));
                    }
                }
                (Token::Number(rest[..used].parse().unwrap()), used)
            }
            _ if ch.is_alphanumeric() || ch == '_' => {
                let mut used = 0;
                for c in rest.chars() {
                    if !(c.is_alphanumeric() || c == '_') {
                        break;
                    }
                    lexer.budget.charge(c.len_utf8())?;
                    used += c.len_utf8();
                }
                (Token::Name(lexer.copy_label(&rest[..used])?), used)
            }
            _ => {
                return Err(ParseFailure::unsupported(
                    row.span(base, offset..offset + ch.len_utf8()),
                    "unsupported wire token, bus, anchor or cursor operation",
                ));
            }
        };
        lexer.push(token, row.span(base, offset..offset + used))?;
        offset += used;
    }
    Ok(())
}
fn arrow(text: &str) -> Option<(Style, usize)> {
    for (value, orientation, direction) in [
        ("<-->", Orientation::Vertical, Direction::Both),
        ("<->", Orientation::Horizontal, Direction::Both),
        ("-->", Orientation::Vertical, Direction::Forward),
        ("<--", Orientation::Vertical, Direction::Reverse),
        ("->", Orientation::Horizontal, Direction::Forward),
        ("<-", Orientation::Horizontal, Direction::Reverse),
        ("--", Orientation::Vertical, Direction::None),
        ("-", Orientation::Horizontal, Direction::None),
    ] {
        if text.starts_with(value) {
            return Some((
                Style {
                    orientation,
                    direction,
                    kind: LinkType::Normal,
                },
                value.len(),
            ));
        }
    }
    None
}
fn admit(
    input: (&[Spanned], usize),
    body: &BodySource<'_>,
    budget: &mut Budget,
) -> Result<(), ParseFailure> {
    let (tokens, capacity) = input;
    let mut depth = 0usize;
    let mut current = 0usize;
    for (_, token, _) in tokens {
        budget.charge(1)?;
        match token {
            Token::Indent => {
                current += 1;
                depth = depth.max(current);
            }
            Token::Dedent => current = current.saturating_sub(1),
            _ => {}
        }
    }
    // Actual-table witness:4d+9 live states,6 reductions and5 expected terminals.
    let live = 4 * depth + 12;
    let slots = live.next_power_of_two().max(8) * 3 / 2;
    let symbols = std::mem::size_of::<Token>()
        + std::mem::size_of::<crate::plantuml::wire::Size>()
        + std::mem::size_of::<crate::plantuml::wire::Scope>()
        + 4 * std::mem::size_of::<usize>();
    let models = budget
        .limits
        .model_items
        .saturating_add(4)
        .saturating_mul(3)
        .saturating_mul(
            std::mem::size_of::<crate::plantuml::wire::Block>()
                + std::mem::size_of::<crate::plantuml::wire::Link>()
                + std::mem::size_of::<crate::plantuml::wire::LinkInput>()
                + 512,
        );
    let bytes = capacity
        .saturating_mul(std::mem::size_of::<Spanned>())
        .saturating_add(body.adapters)
        .saturating_add(budget.limits.source_bytes.saturating_mul(64))
        .saturating_add(models)
        .saturating_add(
            slots * (symbols + 8)
                + 3 * (live + 1) * 8
                + 24 * std::mem::size_of::<String>()
                + 15 * 32,
        );
    bound(
        "wire parser retained bytes",
        bytes,
        budget
            .limits
            .source_bytes
            .saturating_mul(256)
            .saturating_add(models),
    )?;
    bound(
        "wire parser stack slots",
        slots * 2 + 3 * (live + 1) + 24,
        budget.limits.layout_work,
    )?;
    budget.charge(tokens.len().saturating_mul(8) + 15 * (live + 8) + slots * 2 + 24)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        RenderOptions,
        plantuml::wire::{Anchor, Direction, Size},
    };
    fn model(text: &str) -> Wire {
        super::super::test_support::body(
            (text, "wire"),
            &mut Budget::new(&RenderOptions::default()),
            parse,
        )
        .unwrap()
    }
    #[test]
    fn should_keep_scoped_identity_four_space_tab_and_remaining_space_levels() {
        // given
        let model = model(
            "* Parent\n    * Same [100x50]\n    ---\n    * Next\n* Other\n\t* Same\n* One\n * Child",
        );
        // when
        let keys = model
            .blocks
            .iter()
            .skip(1)
            .map(|b| b.key.as_str())
            .collect::<Vec<_>>();
        // then
        assert_eq!(
            keys,
            [
                "Parent",
                "Parent.Same",
                "Parent.Next",
                "Other",
                "Other.Same",
                "One",
                "One.Child"
            ]
        );
        assert_eq!(model.blocks[2].parent, 1);
        assert!(matches!(model.blocks[2].size, Size::Forced(100, 50)));
        assert!(matches!(
            model.blocks[1].placements[1],
            crate::plantuml::wire::Placement::Column
        ));
        for text in [
            "    * Orphan",
            "* A\n        * Skipped",
            "* A\n---\n    * No_parent",
            "* A\n    * B\n    * B",
        ] {
            assert!(
                super::super::test_support::body(
                    (text, "wire"),
                    &mut Budget::new(&RenderOptions::default()),
                    parse
                )
                .is_err(),
                "{text}"
            );
        }
    }
    #[test]
    fn should_resolve_global_dotted_endpoints_and_independent_ordered_axis_slots() {
        // given
        let model = model("A -> B\nA --> C\nA <- B\nA <-- C\n* A\n* C\n---\n* B");
        // when
        let slots = model
            .links
            .iter()
            .map(|l| match l.anchor {
                Anchor::Ordered(slot) => slot,
            })
            .collect::<Vec<_>>();
        // then
        assert_eq!(slots, [0, 0, 1, 1]);
        assert_eq!(model.links[2].style.direction, Direction::Reverse);
        assert_eq!(model.links[0].from, 1);
        assert_eq!(model.links[0].to, 3);
    }
    #[test]
    fn should_admit_depth64_reject65_and_bound_sizes_before_label_retention() {
        // given
        let nested = (0..64)
            .map(|depth| format!("{}* B{depth}", "    ".repeat(depth)))
            .collect::<Vec<_>>()
            .join("\n");
        // when
        let model = model(&nested);
        // then
        assert_eq!(model.blocks.last().unwrap().depth, 64);
        let excessive = format!("{nested}\n{}* B64", "    ".repeat(64));
        assert!(
            super::super::test_support::body(
                (&excessive, "wire"),
                &mut Budget::new(&RenderOptions::default()),
                parse
            )
            .is_err()
        );
        for text in [
            "* A [1234567x1]",
            "* A [100001x1]",
            "* A [0x1]",
            "* A [1x1] junk",
            "* A [1x1",
            "* A [1x1] #fff",
        ] {
            assert!(
                super::super::test_support::body(
                    (text, "wire"),
                    &mut Budget::new(&RenderOptions::default()),
                    parse
                )
                .is_err(),
                "{text}"
            );
        }
    }
    #[test]
    fn should_reject_direct_generated_dangling_scopes_suffixes_and_cancel_actions() {
        // given
        for tokens in [
            vec![(0, Token::Star, 1), (1, Token::Name("A".into()), 2)],
            vec![(0, Token::Dedent, 1)],
            vec![
                (0, Token::Column, 1),
                (1, Token::End, 1),
                (1, Token::Indent, 2),
            ],
        ] {
            let mut budget = Budget::new(&RenderOptions::default());
            let mut builder = Builder::new(&mut budget, 0..2).unwrap();
            // when
            let result = super::super::generated_wire::WireParser::new()
                .parse(&mut builder, tokens.into_iter().map(Ok));
            // then
            assert!(result.is_err());
        }
        for checkpoint in 1..=6 {
            let mut budget = Budget::new(&RenderOptions::default());
            let tokens = vec![
                (0, Token::Star, 1),
                (1, Token::Name("A".into()), 2),
                (2, Token::End, 2),
            ];
            let mut builder = Builder::new(&mut budget, 0..2).unwrap();
            builder.budget.control.cancel_after_checkpoints(checkpoint);
            let iterator = context::Checked::new(tokens, builder.budget.control.clone());
            // when
            let result =
                super::super::generated_wire::WireParser::new().parse(&mut builder, iterator);
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
    fn should_cancel_long_literal_scans_and_bound_generated_entry_retention() {
        // given
        let source = format!(
            "@startwire\n* A\n---\n* B\nA -> B : {}\n@endwire",
            "x".repeat(8000)
        );
        let mut budget = Budget::new(&RenderOptions::default());
        let document = super::super::test_support::document(&source, &mut budget).unwrap();
        let mut prepared = super::super::document::parse(&document, &mut budget).unwrap();
        super::super::preamble::parse(&mut prepared, &mut budget).unwrap();
        budget.control.cancel_after_checkpoints(30);
        // when
        let result = parse_body(&prepared.body, &mut budget);
        // then
        assert!(matches!(
            result,
            Err(ParseFailure {
                kind: super::super::error::FailureKind::Terminal(RenderError::Cancelled(_)),
                ..
            })
        ));
        let mut budget = Budget::new(&RenderOptions::default());
        budget.limits.source_bytes = 1;
        assert!(admit((&[(0, Token::Star, 1)], 1), &prepared.body, &mut budget).is_err());
        let exact = "@startwire\n* A\n@endwire";
        let mut budget = Budget::new(&RenderOptions::default());
        budget.limits.source_bytes = exact.len();
        budget.limits.nesting_depth = 1;
        assert!(super::super::test_support::parse(exact, &mut budget, parse).is_ok());
    }
}
