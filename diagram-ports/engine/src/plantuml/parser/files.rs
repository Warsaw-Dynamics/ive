//! Generated slash-component and physical note-block syntax.
use super::{
    BodySource, context,
    error::{ParseFailure, generated_failure},
    outline::{self, Arena, Lexer, Syntax},
    ownership::Row,
};
use crate::{
    RenderError,
    plantuml::{
        budget::Budget,
        files::{Builder, Files},
    },
};
#[derive(Clone, Debug)]
pub(in crate::plantuml) enum Token {
    Slash,
    Component(String),
    Note,
    EndNote,
    Raw(String),
    Newline,
}
pub(in crate::plantuml) fn parse(
    body: BodySource<'_>,
    budget: &mut Budget,
) -> Result<Files, RenderError> {
    parse_body(&body, budget).map_err(|error| error.into_document(body.document, budget))
}
fn parse_body(body: &BodySource<'_>, budget: &mut Budget) -> Result<Files, ParseFailure> {
    let mut note = false;
    let tokens = outline::lex(body, budget, |row, base, lexer| {
        row_tokens(row, base, lexer, &mut note)
    })?;
    let arena = Arena::admit((&tokens, tokens.capacity()), body, budget, Syntax::Files)?;
    let iterator = context::Checked::new(tokens, budget.control.clone());
    let mut builder = Builder::new(budget, arena)?;
    super::generated_files::FilesParser::new()
        .parse(&mut builder, iterator)
        .map_err(|error| {
            generated_failure(
                error,
                body.range.end,
                "invalid or incomplete files path or <note> syntax",
            )
        })?;
    builder.finish(body.range.end..body.range.end)
}
fn row_tokens(
    row: &Row,
    base: usize,
    lexer: &mut Lexer<'_, Token>,
    note: &mut bool,
) -> Result<(), ParseFailure> {
    let raw = row.text.as_str();
    if *note {
        if raw.trim() == "</note>" {
            *note = false;
            lexer.push(Token::EndNote, row.span(base, 0..raw.len()))?;
        } else {
            let value = lexer.copy(raw)?;
            return lexer.push(Token::Raw(value), row.span(base, 0..raw.len()));
        }
    } else if raw.trim().is_empty() {
        return Ok(());
    } else if raw == "<note>" {
        *note = true;
        lexer.push(Token::Note, row.span(base, 0..raw.len()))?;
    } else if raw.starts_with('/') {
        path_tokens(row, base, lexer)?;
    } else {
        return Err(ParseFailure::unsupported(
            row.span(base, 0..raw.len()),
            "files command",
        ));
    }
    lexer.push(Token::Newline, row.span(base, raw.len()..raw.len()))
}
fn path_tokens(row: &Row, base: usize, lexer: &mut Lexer<'_, Token>) -> Result<(), ParseFailure> {
    let raw = row.text.as_str();
    let mut components = 1;
    for chunk in raw.as_bytes()[1..].chunks(256) {
        lexer
            .budget
            .charge(chunk.len())
            .map_err(ParseFailure::terminal)?;
        components += chunk.iter().filter(|&&byte| byte == b'/').count();
    }
    components -= usize::from(raw.ends_with('/'));
    lexer
        .budget
        .depth(components)
        .map_err(ParseFailure::terminal)?;
    let mut start = 0;
    for (index, ch) in raw.char_indices() {
        lexer
            .budget
            .charge(ch.len_utf8())
            .map_err(ParseFailure::terminal)?;
        if ch == '/' {
            if index > start {
                let value = lexer.copy(&raw[start..index])?;
                lexer.push(Token::Component(value), row.span(base, start..index))?;
            }
            lexer.push(Token::Slash, row.span(base, index..index + 1))?;
            start = index + 1;
        }
    }
    if start < raw.len() {
        let value = lexer.copy(&raw[start..])?;
        lexer.push(Token::Component(value), row.span(base, start..raw.len()))?;
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        RenderOptions,
        plantuml::{parser, preprocess},
    };
    #[test]
    fn should_generate_literal_paths_and_raw_sibling_notes() {
        // given
        let mut budget = Budget::new(&RenderOptions::default());
        let document = preprocess::prepare(
            "@startfiles\n/a/../x  \n/a/../x  \n<note>\n'raw /' café\n</note>\n/empty/\n@endfiles",
            &mut budget,
        )
        .unwrap();
        let mut prepared = parser::document::parse(&document, &mut budget).unwrap();
        parser::preamble::parse(&mut prepared, &mut budget).unwrap();
        // when
        let model = parse(prepared.body, &mut budget).unwrap();
        // then
        assert_eq!(model.entries[2].name, "..");
        assert_eq!(model.entries[3].name, "x  ");
        assert_eq!(model.entries[4].name, "x  ");
        assert_eq!(model.entries[5].parent, 2);
        assert_eq!(model.entries[5].name, "'raw /' café");
    }

    #[test]
    fn should_reject_direct_generated_empty_components_and_unclosed_notes() {
        for tokens in [
            vec![Token::Slash, Token::Slash, Token::Newline],
            vec![Token::Note, Token::Newline, Token::Raw("literal".into())],
            vec![Token::EndNote, Token::Newline],
        ] {
            // given
            let stream = tokens
                .into_iter()
                .enumerate()
                .map(|(index, token)| (index, token, index + 1))
                .collect::<Vec<_>>();
            let mut budget = Budget::new(&RenderOptions::default());
            let arena = Arena::for_stream((&stream, stream.capacity()), &mut budget, Syntax::Files)
                .unwrap();
            let iterator = context::Checked::new(stream, budget.control.clone());
            let mut builder = Builder::new(&mut budget, arena).unwrap();
            // when
            let result =
                super::super::generated_files::FilesParser::new().parse(&mut builder, iterator);
            // then
            assert!(result.is_err());
        }
    }

    #[test]
    fn should_cancel_generated_file_delivery_and_path_actions() {
        for checkpoint in 1..=5 {
            // given
            let stream = vec![
                (0, Token::Slash, 1),
                (1, Token::Component("file".into()), 5),
                (5, Token::Newline, 6),
            ];
            let mut budget = Budget::new(&RenderOptions::default());
            let arena = Arena::for_stream((&stream, stream.capacity()), &mut budget, Syntax::Files)
                .unwrap();
            let iterator = context::Checked::new(stream, budget.control.clone());
            let mut builder = Builder::new(&mut budget, arena).unwrap();
            builder.budget.control.cancel_after_checkpoints(checkpoint);
            // when
            let result =
                super::super::generated_files::FilesParser::new().parse(&mut builder, iterator);
            // then
            assert!(matches!(
                result,
                Err(lalrpop_util::ParseError::User {
                    error: ParseFailure {
                        kind: super::super::error::FailureKind::Terminal(RenderError::Cancelled(_)),
                        ..
                    }
                })
            ));
        }
    }
}
