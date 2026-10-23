use super::super::budget::{Budget, bound};
use super::{SourceView, error::ParseFailure};
use crate::RenderControl;

#[derive(Clone, Copy, Debug)]
pub(in crate::plantuml) enum Token<'s> {
    Define,
    Gap,
    Name(&'s str),
    Literal(&'s str),
}

pub(super) type Spanned<'s> = (usize, Token<'s>, usize);

/// Scans one definition header, then owns the remaining same-line literal bytes.
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
    bound(
        "definition buffered token bytes",
        5 * std::mem::size_of::<Spanned<'s>>(),
        budget.limits.source_bytes.saturating_mul(256),
    )
    .map_err(ParseFailure::terminal)?;
    budget.checkpoint().map_err(ParseFailure::terminal)?;
    let mut lexer = Lexer {
        view,
        cursor: 0,
        budget,
    };
    lexer.consume_while(|ch| match ch {
        '\r' | '\n' => false,
        ch => ch.is_whitespace(),
    })?;
    let mut tokens = Vec::with_capacity(5);
    while lexer.cursor < view.text.len() {
        bound("definition buffered tokens", tokens.len() + 1, 5).map_err(ParseFailure::terminal)?;
        lexer.budget.charge(1).map_err(ParseFailure::terminal)?;
        let token = lexer.next(tokens.len())?;
        tokens.push(token);
    }
    Ok(tokens)
}

struct Lexer<'s, 'b> {
    view: SourceView<'s>,
    cursor: usize,
    budget: &'b mut Budget,
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

    fn next(&mut self, index: usize) -> Result<Spanned<'s>, ParseFailure> {
        let start = self.cursor;
        let rest = &self.view.text[start..];
        let token = match index {
            0 if rest.starts_with("!define") => {
                self.budget.charge(7).map_err(ParseFailure::terminal)?;
                self.cursor += 7;
                Token::Define
            }
            1 | 3 if rest.starts_with([' ', '\t']) => {
                self.skip_indent()?;
                Token::Gap
            }
            2 => self.name()?,
            4 => self.literal()?,
            _ => {
                return Err(
                    self.fail("definition requires !define, name and a same-line nonempty literal")
                );
            }
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

    fn name(&mut self) -> Result<Token<'s>, ParseFailure> {
        let rest = &self.view.text[self.cursor..];
        self.consume_while(|ch| ch.is_ascii_alphanumeric() || ch == '_')?;
        let end = self.cursor - (self.view.text.len() - rest.len());
        if end == 0 {
            return Err(self.fail("invalid literal definition name"));
        }
        let name = &rest[..end];
        if rest[end..].starts_with('(') {
            return Err(ParseFailure::unsupported(
                self.view.byte_base + self.cursor..self.view.byte_base + self.cursor + 1,
                "parameterized definition",
            ));
        }
        Ok(Token::Name(name))
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
