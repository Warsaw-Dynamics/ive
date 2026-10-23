use super::super::budget::{Budget, bound};
use super::{SourceView, error::ParseFailure};
use crate::RenderControl;

#[derive(Clone, Copy, Debug)]
pub(in crate::plantuml) enum Token<'s> {
    Define,
    Gap,
    Name(&'s str),
    Literal(&'s str),
    Punctuation(char),
}

pub(super) type Spanned<'s> = (usize, Token<'s>, usize);

/// Emits header primitives and owns the raw tail after an identifier and horizontal trivia.
pub(super) fn lex<'s>(
    view: SourceView<'s>,
    budget: &mut Budget,
) -> Result<Vec<Spanned<'s>>, ParseFailure> {
    bound(
        "definition source bytes",
        view.text.len(),
        budget.limits.source_bytes,
    )
    .map_err(ParseFailure::terminal)?;
    budget.checkpoint().map_err(ParseFailure::terminal)?;
    let mut lexer = Lexer {
        view,
        cursor: 0,
        budget,
        mode: Mode::Header,
    };
    lexer.consume_while(|ch| match ch {
        '\n' => false,
        ch => ch.is_whitespace(),
    })?;
    let mut tokens = Vec::new();
    while lexer.cursor < view.text.len() {
        let token = lexer.next()?;
        push_token(&mut tokens, token, lexer.budget)?;
    }
    Ok(tokens)
}

fn push_token<'s>(
    tokens: &mut Vec<Spanned<'s>>,
    token: Spanned<'s>,
    budget: &mut Budget,
) -> Result<(), ParseFailure> {
    bound(
        "definition buffered tokens",
        tokens.len() + 1,
        budget.limits.source_bytes,
    )
    .map_err(ParseFailure::terminal)?;
    budget.charge(1).map_err(ParseFailure::terminal)?;
    if tokens.len() == tokens.capacity() {
        let capacity = tokens.capacity().saturating_mul(2).max(4);
        let bytes = capacity.saturating_mul(std::mem::size_of::<Spanned<'s>>());
        bound(
            "definition buffered token bytes",
            bytes,
            budget.limits.source_bytes.saturating_mul(256),
        )
        .map_err(ParseFailure::terminal)?;
        budget
            .charge(tokens.len())
            .map_err(ParseFailure::terminal)?;
        tokens
            .try_reserve_exact(capacity - tokens.len())
            .map_err(|error| {
                ParseFailure::terminal(crate::RenderError::Backend(format!(
                    "PlantUML definition token allocation for capacity {capacity} failed: {error}"
                )))
            })?;
    }
    tokens.push(token);
    Ok(())
}

#[derive(Clone, Copy)]
enum Mode {
    Header,
    NameTail,
    Literal,
}

struct Lexer<'s, 'b> {
    view: SourceView<'s>,
    cursor: usize,
    budget: &'b mut Budget,
    mode: Mode,
}

impl<'s> Lexer<'s, '_> {
    fn skip_indent(&mut self) -> Result<(), ParseFailure> {
        self.consume_while(|ch| matches!(ch, ' ' | '\t'))
    }

    fn consume_while(&mut self, predicate: impl Fn(char) -> bool) -> Result<(), ParseFailure> {
        let mut pending = 0;
        for ch in self.view.text[self.cursor..]
            .chars()
            .take_while(|&ch| predicate(ch))
        {
            pending += ch.len_utf8();
            self.cursor += ch.len_utf8();
            if pending >= 256 {
                self.budget
                    .charge(pending)
                    .map_err(ParseFailure::terminal)?;
                pending = 0;
            }
        }
        self.budget.charge(pending).map_err(ParseFailure::terminal)
    }

    fn next(&mut self) -> Result<Spanned<'s>, ParseFailure> {
        let start = self.cursor;
        let rest = &self.view.text[start..];
        let token = match self.mode {
            Mode::Literal => self.literal()?,
            _ if rest.starts_with("!define") => {
                self.budget.charge(7).map_err(ParseFailure::terminal)?;
                self.cursor += 7;
                Token::Define
            }
            _ if rest.starts_with([' ', '\t']) => {
                self.skip_indent()?;
                if matches!(self.mode, Mode::NameTail) {
                    self.mode = Mode::Literal;
                }
                Token::Gap
            }
            _ => self.name_or_punctuation()?,
        };
        let end = match token {
            Token::Literal(literal) => start + literal.len(),
            _ => self.cursor,
        };
        Ok((
            self.view.byte_base + start,
            token,
            self.view.byte_base + end,
        ))
    }

    fn name_or_punctuation(&mut self) -> Result<Token<'s>, ParseFailure> {
        let rest = &self.view.text[self.cursor..];
        self.consume_while(|ch| ch.is_ascii_alphanumeric() || ch == '_')?;
        let end = self.cursor - (self.view.text.len() - rest.len());
        if end == 0 {
            let ch = rest.chars().next().unwrap();
            if ch == '\n' {
                return Err(self.fail("definition requires a same-line literal and complete EOF"));
            }
            self.budget
                .charge(ch.len_utf8())
                .map_err(ParseFailure::terminal)?;
            self.cursor += ch.len_utf8();
            return Ok(Token::Punctuation(ch));
        }
        self.mode = Mode::NameTail;
        Ok(Token::Name(&rest[..end]))
    }

    fn literal(&mut self) -> Result<Token<'s>, ParseFailure> {
        let start = self.cursor;
        let mut end = start;
        let mut pending = 0;
        for ch in self.view.text[start..].chars() {
            if ch == '\n' {
                return Err(self.fail("definition requires a same-line literal and complete EOF"));
            }
            self.cursor += ch.len_utf8();
            pending += ch.len_utf8();
            match ch {
                ' ' | '\t' => {}
                _ => end = self.cursor,
            }
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
        Ok(Token::Literal(&self.view.text[start..end]))
    }

    fn fail(&self, message: &'static str) -> ParseFailure {
        ParseFailure::syntax(
            self.view.byte_base + self.cursor..self.view.byte_base + self.cursor,
            message,
        )
    }
}

pub(super) struct CheckedTokens<'s> {
    tokens: std::vec::IntoIter<Spanned<'s>>,
    control: RenderControl,
}

impl<'s> CheckedTokens<'s> {
    pub fn new(tokens: Vec<Spanned<'s>>, control: RenderControl) -> Self {
        Self {
            tokens: tokens.into_iter(),
            control,
        }
    }
}

impl<'s> Iterator for CheckedTokens<'s> {
    type Item = Result<Spanned<'s>, ParseFailure>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Err(error) = self.control.checkpoint() {
            return Some(Err(ParseFailure::terminal(crate::RenderError::Cancelled(
                error.to_string(),
            ))));
        }
        self.tokens.next().map(Ok)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RenderOptions;

    #[test]
    fn should_lex_leading_carriage_return_as_original_indentation() {
        // given
        let mut budget = Budget::new(&RenderOptions::default());
        let view = SourceView {
            text: "\r!define WHO World",
            byte_base: 10,
        };
        // when
        let tokens = lex(view, &mut budget).unwrap();
        // then
        assert!(matches!(tokens[0], (11, Token::Define, 18)));
        assert!(matches!(tokens[2], (19, Token::Name("WHO"), 22)));
        assert!(matches!(tokens[4], (23, Token::Literal("World"), 28)));
    }

    #[test]
    fn should_emit_primitive_tokens_without_requiring_header_separators() {
        // given
        let mut budget = Budget::new(&RenderOptions::default());
        let view = SourceView {
            text: "!defineWHO World",
            byte_base: 0,
        };
        // when
        let tokens = lex(view, &mut budget).unwrap();
        // then
        assert_eq!(tokens.len(), 4);
        assert!(matches!(tokens[0], (0, Token::Define, 7)));
        assert!(matches!(tokens[1], (7, Token::Name("WHO"), 10)));
        assert!(matches!(tokens[2], (10, Token::Gap, 11)));
        assert!(matches!(tokens[3], (11, Token::Literal("World"), 16)));
    }

    #[test]
    fn should_bound_wide_primitive_buffer_before_generated_syntax() {
        // given
        let raw = "!define ".repeat(1024);
        let mut budget = Budget::new(&RenderOptions::default());
        // when
        let tokens = lex(
            SourceView {
                text: &raw,
                byte_base: 0,
            },
            &mut budget,
        )
        .unwrap();
        // then
        assert_eq!(tokens.len(), 2048);
        assert_eq!(tokens.capacity(), 2048);
        assert!(budget.work < 30000);
    }

    #[test]
    fn should_cancel_before_token_buffer_growth_and_keep_retained_capacity() {
        // given
        let mut budget = Budget::new(&RenderOptions::default());
        let mut tokens = Vec::with_capacity(4);
        tokens.extend([(0, Token::Define, 7); 4]);
        budget.control.cancel();
        // when
        let error = push_token(&mut tokens, (7, Token::Gap, 8), &mut budget).unwrap_err();
        // then
        assert!(matches!(
            error.kind,
            super::super::error::FailureKind::Terminal(crate::RenderError::Cancelled(_))
        ));
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens.capacity(), 4);
    }
}
