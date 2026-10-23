//! Generated stage-row syntax with typed board construction.
use super::{
    BodySource, context,
    error::{ParseFailure, generated_failure},
    outline::{self, Arena, Lexer, Syntax},
    ownership::Row,
};
use crate::{
    RenderError,
    plantuml::{
        board::{Board, Builder},
        budget::Budget,
    },
};
#[derive(Clone, Debug)]
pub(in crate::plantuml) enum Token {
    Plus,
    Label(String),
    Newline,
}
pub(in crate::plantuml) fn parse(
    body: BodySource<'_>,
    budget: &mut Budget,
) -> Result<Board, RenderError> {
    parse_body(&body, budget).map_err(|error| error.into_document(body.document, budget))
}
fn parse_body(body: &BodySource<'_>, budget: &mut Budget) -> Result<Board, ParseFailure> {
    let tokens = outline::lex(body, budget, row_tokens)?;
    let arena = Arena::admit((&tokens, tokens.capacity()), body, budget, Syntax::Board)?;
    let iterator = context::Checked::new(tokens, budget.control.clone());
    let mut builder = Builder::new(budget, arena);
    super::generated_board::BoardParser::new()
        .parse(&mut builder, iterator)
        .map_err(|error| {
            generated_failure(
                error,
                body.range.end,
                "invalid or incomplete board stage syntax",
            )
        })?;
    builder.finish(body.range.end..body.range.end)
}
fn row_tokens(row: &Row, base: usize, lexer: &mut Lexer<'_, Token>) -> Result<(), ParseFailure> {
    let text = row.text.trim();
    if text.is_empty() {
        return Ok(());
    }
    if super::preamble::has_marker(text)
        || matches!(
            text.split_whitespace().next(),
            Some("newpage" | "page" | "sprite" | "hide" | "show" | "scale" | "url")
        )
        || text.starts_with(['!', '@'])
    {
        return Err(ParseFailure::unsupported(
            row.span(base, 0..row.text.len()),
            "board command or style",
        ));
    }
    let start = row.text.len() - row.text.trim_start().len();
    let mut stage = 0;
    for _ in text.bytes().take_while(|&ch| ch == b'+') {
        lexer
            .budget
            .depth(stage + 2)
            .map_err(ParseFailure::terminal)?;
        lexer.push(
            Token::Plus,
            row.span(base, start + stage..start + stage + 1),
        )?;
        stage += 1;
    }
    let name = text[stage..].trim_start();
    let value = lexer.copy_label(name)?;
    lexer.push(
        Token::Label(value),
        row.span(base, start + text.len() - name.len()..start + text.len()),
    )?;
    lexer.push(
        Token::Newline,
        row.span(base, row.text.len()..row.text.len()),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        RenderOptions,
        plantuml::{board::Parent, parser, preprocess},
    };
    #[test]
    fn should_generate_nearest_lower_stage_parents_and_gaps() {
        // given
        let mut budget = Budget::new(&RenderOptions::default());
        let document = preprocess::prepare(
            "@startboard\nActivity\n+++ A\n+++++ B\n++ C\nNext\n@endboard",
            &mut budget,
        )
        .unwrap();
        let mut prepared = parser::document::parse(&document, &mut budget).unwrap();
        parser::preamble::parse(&mut prepared, &mut budget).unwrap();
        // when
        let model = parse(prepared.body, &mut budget).unwrap();
        // then
        assert_eq!(model.activities, [0, 4]);
        assert_eq!(model.cards[2].parent, Parent::Node(1));
        assert_eq!(model.cards[3].parent, Parent::Node(0));
        assert_eq!(model.cards[2].stage, 5);
    }

    #[test]
    fn should_reject_direct_generated_incomplete_stage_rows() {
        for tokens in [
            vec![Token::Plus, Token::Newline],
            vec![Token::Label("root".into()), Token::Plus, Token::Newline],
            vec![Token::Label("root".into())],
        ] {
            // given
            let stream = tokens
                .into_iter()
                .enumerate()
                .map(|(index, token)| (index, token, index + 1))
                .collect::<Vec<_>>();
            let mut budget = Budget::new(&RenderOptions::default());
            let arena = Arena::for_stream((&stream, stream.capacity()), &mut budget, Syntax::Board)
                .unwrap();
            let iterator = context::Checked::new(stream, budget.control.clone());
            let mut builder = Builder::new(&mut budget, arena);
            // when
            let result =
                super::super::generated_board::BoardParser::new().parse(&mut builder, iterator);
            // then
            assert!(result.is_err());
        }
    }

    #[test]
    fn should_cancel_generated_board_delivery_and_stage_actions() {
        for checkpoint in 1..=5 {
            // given
            let stream = vec![(0, Token::Label("root".into()), 4), (4, Token::Newline, 5)];
            let mut budget = Budget::new(&RenderOptions::default());
            let arena = Arena::for_stream((&stream, stream.capacity()), &mut budget, Syntax::Board)
                .unwrap();
            budget.control.cancel_after_checkpoints(checkpoint);
            let iterator = context::Checked::new(stream, budget.control.clone());
            let mut builder = Builder::new(&mut budget, arena);
            // when
            let result =
                super::super::generated_board::BoardParser::new().parse(&mut builder, iterator);
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
