//! Generated physical-line/paragraph structure over raw borrowed source spans.
use super::{
    BodySource, context,
    error::{ParseFailure, generated_failure},
};
use crate::{
    RenderError,
    plantuml::{
        budget::{Budget, bound},
        creoledoc::{Builder, DocLine, Document, Paragraph},
    },
};

#[derive(Clone, Copy, Debug)]
pub(in crate::plantuml) enum Token {
    Text,
    Newline,
    Blank,
}
type Spanned = (usize, Token, usize);

pub(in crate::plantuml) fn parse<'s>(
    body: BodySource<'s>,
    budget: &mut Budget,
) -> Result<Document<'s>, RenderError> {
    parse_body(&body, budget).map_err(|failure| failure.into_document(body.document, budget))
}
fn parse_body<'s>(
    body: &BodySource<'s>,
    budget: &mut Budget,
) -> Result<Document<'s>, ParseFailure> {
    budget.depth(1)?;
    let tokens = lex(body, budget)?;
    admit((&tokens, tokens.capacity()), body, budget)?;
    let iterator = context::Checked::new(tokens, budget.control.clone());
    let mut builder = Builder::new(budget);
    super::generated_creoledoc::DocumentParser::new()
        .parse(&mut builder, iterator)
        .map_err(|error| {
            generated_failure(
                error,
                body.range.end,
                "invalid Creole document line, paragraph or suffix",
            )
        })?;
    Ok(builder.finish(&body.document.text))
}
fn lex(body: &BodySource<'_>, budget: &mut Budget) -> Result<Vec<Spanned>, ParseFailure> {
    let mut tokens = Vec::new();
    let mut base = body.range.start;
    for physical in body.document.text[body.range.clone()].split_inclusive('\n') {
        let raw = physical.strip_suffix('\n').unwrap_or(physical);
        let raw = raw.strip_suffix('\r').unwrap_or(raw);
        bound("Creole document line bytes", raw.len(), 16 * 1024)?;
        let mut cursor = 0;
        let mut content = false;
        let mut protected = false;
        while cursor < raw.len() {
            let start = cursor;
            let spaces = raw.as_bytes()[cursor] == b' ';
            while cursor < raw.len() && (raw.as_bytes()[cursor] == b' ') == spaces {
                let ch = raw[cursor..].chars().next().unwrap();
                budget.charge(ch.len_utf8())?;
                if ch.is_control() || matches!(ch, '\u{fffe}' | '\u{ffff}') {
                    return Err(ParseFailure::unsupported(
                        base + cursor..base + cursor + ch.len_utf8(),
                        "Creole document control character",
                    ));
                }
                if protected {
                    protected = false;
                } else {
                    reject_marker((&raw[cursor..], base + cursor), budget)?;
                    protected = ch == '\\'
                        || (ch == '~'
                            && raw[cursor + 1..]
                                .starts_with(['_', '-', '"', '#', ']', '[', '*', '.', '/', '<']));
                }
                cursor += ch.len_utf8();
            }
            content |= !spaces;
            context::push(
                &mut tokens,
                (base + start, Token::Text, base + cursor),
                budget,
            )?;
        }
        if content {
            context::push(
                &mut tokens,
                (base + raw.len(), Token::Newline, base + physical.len()),
                budget,
            )?;
        } else {
            // A space-only row is one lexical blank primitive; preserve its original span.
            if !raw.is_empty() {
                tokens.pop();
            }
            context::push(&mut tokens, (base, Token::Blank, base + raw.len()), budget)?;
        }
        base += physical.len();
    }
    Ok(tokens)
}
fn reject_marker(value: (&str, usize), budget: &mut Budget) -> Result<(), ParseFailure> {
    let (rest, offset) = value;
    if rest.starts_with("{{") || rest.starts_with("}}") {
        return Err(ParseFailure::unsupported(
            offset..offset + 2,
            "Creole code or embedded diagram blocks are unsupported",
        ));
    }
    let Some(tag) = rest.strip_prefix('<') else {
        return Ok(());
    };
    let tag = tag.strip_prefix('/').unwrap_or(tag);
    let end = tag
        .find(|ch: char| !ch.is_ascii_alphabetic())
        .unwrap_or(tag.len());
    budget.charge(end + 1)?;
    if matches!(
        tag.as_bytes().get(end),
        Some(b'>' | b'/' | b' ' | b'\t' | b':')
    ) && [
        "code",
        "pre",
        "html",
        "head",
        "body",
        "div",
        "p",
        "br",
        "ul",
        "ol",
        "li",
        "a",
        "svg",
        "path",
        "g",
        "foreignobject",
    ]
    .iter()
    .any(|name| tag[..end].eq_ignore_ascii_case(name))
    {
        return Err(ParseFailure::unsupported(
            offset..offset + end + 1,
            "Creole code, HTML or embedded SVG markup is unsupported",
        ));
    }
    Ok(())
}
fn admit(
    input: (&[Spanned], usize),
    body: &BodySource<'_>,
    budget: &mut Budget,
) -> Result<(), ParseFailure> {
    let (tokens, capacity) = input;
    // Actual table closure: 13 states, nine reachable configurations, five live states.
    let (live, terminals, reductions) = (5usize, 3usize, 4usize);
    let stack = live.next_power_of_two().max(8);
    let slots = 2 * (stack + stack / 2) + 3 * (live + 1) + 3 * terminals.next_power_of_two();
    let symbol = std::mem::size_of::<std::ops::Range<usize>>() + std::mem::size_of::<usize>();
    let mut lines = 0usize;
    for (_, token, _) in tokens {
        budget.charge(1)?;
        lines += usize::from(matches!(token, Token::Newline | Token::Blank));
    }
    let retained = capacity
        .saturating_mul(std::mem::size_of::<Spanned>())
        .saturating_add(body.adapters)
        .saturating_add((stack + stack / 2) * (symbol + 2 * std::mem::size_of::<usize>() + 2))
        .saturating_add(
            3 * (live + 1) * 2
                + 3 * terminals.next_power_of_two() * std::mem::size_of::<String>()
                + terminals * 32,
        )
        .saturating_add(lines.max(4).saturating_mul(
            3 * (std::mem::size_of::<DocLine>() + std::mem::size_of::<Paragraph>()),
        ))
        .saturating_add(std::mem::size_of::<Builder<'_>>());
    bound(
        "Creole document parser retained bytes",
        retained,
        budget.limits.source_bytes.saturating_mul(256),
    )?;
    bound(
        "Creole document parser slots",
        slots,
        budget.limits.layout_work,
    )?;
    budget.charge(
        tokens
            .len()
            .saturating_mul(reductions + 1)
            .saturating_add(slots + terminals * (live + reductions)),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RenderOptions, plantuml::origin};

    #[test]
    fn should_build_maximal_paragraphs_and_preserve_borrowed_physical_line_spans() {
        // given
        let document = origin::Document {
            text: "  first\r\nsecond\n\n \nlast\n".into(),
            ..Default::default()
        };
        let body = BodySource {
            document: &document,
            range: 0..document.text.len(),
            ownership: Default::default(),
            adapters: std::mem::size_of::<BodySource<'_>>(),
        };
        let mut budget = Budget::new(&RenderOptions::default());
        // when
        let model = parse(body, &mut budget).unwrap();
        // then
        assert_eq!(model.lines.len(), 5);
        assert_eq!(model.paragraphs.len(), 4);
        assert!(matches!(&model.paragraphs[0], Paragraph::Text(lines) if *lines == (0..2)));
        assert!(matches!(model.paragraphs[1], Paragraph::Blank(2)));
        assert!(matches!(model.paragraphs[2], Paragraph::Blank(3)));
        assert!(matches!(&model.paragraphs[3], Paragraph::Text(lines) if *lines == (4..5)));
        assert_eq!(
            model
                .lines
                .iter()
                .map(|line| &model.source[line.span.clone()])
                .collect::<Vec<_>>(),
            ["  first", "second", "", " ", "last"]
        );
    }

    #[test]
    fn should_reject_incomplete_generated_lines_and_cancel_each_token_and_eof() {
        // given
        let tokens = vec![(0, Token::Text, 4), (4, Token::Newline, 5)];
        // when / then
        for invalid in [vec![(0, Token::Newline, 1)], vec![(0, Token::Text, 4)]] {
            let mut budget = Budget::new(&RenderOptions::default());
            let mut builder = Builder::new(&mut budget);
            assert!(
                super::super::generated_creoledoc::DocumentParser::new()
                    .parse(
                        &mut builder,
                        context::Checked::new(invalid, Default::default())
                    )
                    .is_err()
            );
        }
        for checkpoint in [0, 1, 2] {
            let options = RenderOptions::default();
            options.control.cancel_after_checkpoints(checkpoint);
            let mut budget = Budget::new(&options);
            let mut builder = Builder::new(&mut budget);
            assert!(
                super::super::generated_creoledoc::DocumentParser::new()
                    .parse(
                        &mut builder,
                        context::Checked::new(tokens.clone(), options.control)
                    )
                    .is_err(),
                "ignored delivery/EOF checkpoint {checkpoint}"
            );
        }
    }

    #[test]
    fn should_cancel_lexical_scanning_and_refuse_tiny_generated_admission_before_model_growth() {
        // given
        let document = origin::Document {
            text: " ".repeat(16_000),
            ..Default::default()
        };
        let body = BodySource {
            document: &document,
            range: 0..document.text.len(),
            ownership: Default::default(),
            adapters: std::mem::size_of::<BodySource<'_>>(),
        };
        let mut budget = Budget::new(&RenderOptions::default());
        budget.control.cancel_after_checkpoints(20);
        // when
        let cancelled = lex(&body, &mut budget);
        // then
        assert!(matches!(
            cancelled.unwrap_err().kind,
            super::super::error::FailureKind::Terminal(RenderError::Cancelled(_))
        ));
        assert!(budget.work < document.text.len());
        let mut options = RenderOptions::default();
        options.limits.layout_work = 20;
        let mut tiny = Budget::new(&options);
        let failure = admit((&[], 0), &body, &mut tiny).unwrap_err();
        assert!(failure.into_line(1).to_string().contains("parser slots"));
        assert_eq!(tiny.work, 0);
    }
}
