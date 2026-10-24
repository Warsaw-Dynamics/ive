use super::{DotError, Id, IdKind, ParseOptions};

#[derive(Clone, Copy, PartialEq, Eq)]
enum HeaderKind {
    Unknown,
    Graph,
    Digraph,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Strict,
    Graph,
    Digraph,
    Node,
    Edge,
    Subgraph,
    Id(Id),
    Arrow,
    Dash,
    Open,
    Close,
    OpenAttrs,
    CloseAttrs,
    Equal,
    Colon,
    Plus,
    Comma,
    Semicolon,
}

pub fn lex(source: &str, options: &ParseOptions) -> Result<Vec<(usize, Token, usize)>, DotError> {
    let mut scanner = Scanner {
        source,
        offset: 0,
        tokens: Vec::new(),
        depth: 0,
        attrs_depth: 0,
        graph_kind: HeaderKind::Unknown,
        options,
    };
    while scanner.offset < source.len() {
        scanner
            .options
            .control
            .checkpoint()
            .map_err(|error| scanner.error(format!("DOT lexing was cancelled: {error}")))?;
        scanner.skip_trivia()?;
        if scanner.offset == source.len() {
            break;
        }
        scanner.next_token()?;
    }
    if scanner.depth != 0 {
        return Err(scanner.error("DOT graph has an unclosed brace"));
    }
    Ok(scanner.tokens)
}

struct Scanner<'a> {
    source: &'a str,
    offset: usize,
    tokens: Vec<(usize, Token, usize)>,
    depth: usize,
    attrs_depth: usize,
    graph_kind: HeaderKind,
    options: &'a ParseOptions,
}

impl Scanner<'_> {
    fn error(&self, message: impl Into<String>) -> DotError {
        DotError::new(
            self.offset..self.offset.saturating_add(1).min(self.source.len()),
            message,
        )
    }

    fn peek(&self) -> Option<char> {
        self.source[self.offset..].chars().next()
    }
    fn rest(&self) -> &str {
        &self.source[self.offset..]
    }
    fn take(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.offset += ch.len_utf8();
        Some(ch)
    }
    fn emit(&mut self, start: usize, token: Token) -> Result<(), DotError> {
        if self.tokens.len() >= self.options.tokens {
            return Err(DotError::new(
                start..self.offset,
                format!("DOT token count exceeds {}", self.options.tokens),
            ));
        }
        self.tokens.push((start, token, self.offset));
        Ok(())
    }
    fn skip_trivia(&mut self) -> Result<(), DotError> {
        loop {
            if self.rest().starts_with('\u{feff}') {
                self.take();
            }
            while self
                .peek()
                .is_some_and(|ch| matches!(ch, ' ' | '\t' | '\r' | '\n'))
            {
                self.take();
            }
            if self.rest().starts_with("//") || self.rest().starts_with('#') {
                while self.peek().is_some_and(|ch| ch != '\n') {
                    self.take();
                }
            } else if self.rest().starts_with("/*") {
                let start = self.offset;
                self.offset += 2;
                while self.offset < self.source.len() && !self.rest().starts_with("*/") {
                    self.take();
                }
                if self.offset == self.source.len() {
                    return Err(DotError::new(
                        start..self.offset,
                        "DOT block comment is unterminated",
                    ));
                }
                self.offset += 2;
            } else {
                return Ok(());
            }
        }
    }
    fn next_token(&mut self) -> Result<(), DotError> {
        let start = self.offset;
        let token = if self.rest().starts_with("->") {
            if self.graph_kind == HeaderKind::Graph {
                return Err(DotError::new(
                    start..start + 2,
                    "DOT undirected graph cannot use directed edge operator '->'",
                ));
            }
            self.offset += 2;
            Token::Arrow
        } else if self.rest().starts_with("--") {
            if self.graph_kind == HeaderKind::Digraph {
                return Err(DotError::new(
                    start..start + 2,
                    "DOT directed graph cannot use undirected edge operator '--'",
                ));
            }
            self.offset += 2;
            Token::Dash
        } else {
            match self
                .take()
                .ok_or_else(|| self.error("DOT token is missing"))?
            {
                '{' => {
                    self.depth += 1;
                    if self.depth > self.options.nesting_depth {
                        return Err(self.error(format!(
                            "DOT nesting exceeds {}",
                            self.options.nesting_depth
                        )));
                    }
                    Token::Open
                }
                '}' => {
                    if self.depth == 0 {
                        return Err(self.error("DOT has an unmatched closing brace"));
                    }
                    self.depth -= 1;
                    Token::Close
                }
                '[' => {
                    self.attrs_depth += 1;
                    Token::OpenAttrs
                }
                ']' => {
                    if self.attrs_depth == 0 {
                        return Err(self.error("DOT has an unmatched closing attribute bracket"));
                    }
                    self.attrs_depth -= 1;
                    Token::CloseAttrs
                }
                ';' => Token::Semicolon,
                ',' => Token::Comma,
                '=' => Token::Equal,
                ':' => Token::Colon,
                '+' => Token::Plus,
                '"' => Token::Id(Id {
                    text: self.quoted(start)?,
                    kind: IdKind::Quoted,
                }),
                '<' => Token::Id(Id {
                    text: self.html(start)?,
                    kind: IdKind::Html,
                }),
                ch if is_bare_start(ch) || is_number_start(ch, self.rest()) => {
                    self.bare(start, ch)?
                }
                ch => {
                    return Err(DotError::new(
                        start..self.offset,
                        format!("invalid DOT token {ch:?}"),
                    ));
                }
            }
        };
        self.emit(start, token)
    }
    fn quoted(&mut self, start: usize) -> Result<String, DotError> {
        let mut value = String::new();
        loop {
            let ch = self.take().ok_or_else(|| {
                DotError::new(start..self.offset, "DOT quoted ID is unterminated")
            })?;
            match ch {
                '"' => return Ok(value),
                '\\' => {
                    let next = self.take().ok_or_else(|| {
                        DotError::new(start..self.offset, "DOT quoted ID ends with a backslash")
                    })?;
                    match next {
                        '"' => value.push('"'),
                        '\n' => {}
                        _ => {
                            value.push('\\');
                            value.push(next);
                        }
                    }
                }
                _ => value.push(ch),
            }
        }
    }
    fn html(&mut self, start: usize) -> Result<String, DotError> {
        let mut depth = 1usize;
        while let Some(ch) = self.take() {
            match ch {
                '<' => {
                    depth += 1;
                    if depth > self.options.nesting_depth {
                        return Err(self.error("DOT HTML ID nesting exceeds limit"));
                    }
                }
                '>' => {
                    depth -= 1;
                    if depth == 0 {
                        return Ok(self.source[start + 1..self.offset - 1].to_owned());
                    }
                }
                _ => {}
            }
        }
        Err(DotError::new(
            start..self.offset,
            "DOT HTML ID is unterminated",
        ))
    }
    fn bare(&mut self, start: usize, first: char) -> Result<Token, DotError> {
        let numeral = first.is_ascii_digit() || first == '-' || first == '.';
        let mut decimal_point = first == '.';
        while self.peek().is_some_and(|ch| {
            if numeral {
                ch.is_ascii_digit() || (ch == '.' && !decimal_point)
            } else {
                is_bare_continue(ch)
            }
        }) {
            if self.peek() == Some('.') {
                decimal_point = true;
            }
            self.take();
        }
        let text = &self.source[start..self.offset];
        if numeral {
            if text.parse::<f64>().is_err() {
                return Err(DotError::new(
                    start..self.offset,
                    format!("invalid DOT numeral {text:?}"),
                ));
            }
            return Ok(Token::Id(Id {
                text: text.into(),
                kind: IdKind::Numeral,
            }));
        }
        let keyword = match text {
            "strict" => Token::Strict,
            "graph" => {
                if self.graph_kind == HeaderKind::Unknown {
                    self.graph_kind = HeaderKind::Graph;
                }
                Token::Graph
            }
            "digraph" => {
                if self.graph_kind == HeaderKind::Unknown {
                    self.graph_kind = HeaderKind::Digraph;
                }
                Token::Digraph
            }
            "node" => Token::Node,
            "edge" => Token::Edge,
            "subgraph" => Token::Subgraph,
            _ => Token::Id(Id {
                text: text.into(),
                kind: IdKind::Bare,
            }),
        };
        Ok(keyword)
    }
}

fn is_bare_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_' || !ch.is_ascii()
}
fn is_bare_continue(ch: char) -> bool {
    is_bare_start(ch) || ch.is_ascii_digit()
}
fn is_number_start(ch: char, rest: &str) -> bool {
    let first = rest.chars().next();
    match ch {
        ch if ch.is_ascii_digit() => true,
        '.' => first.is_some_and(|next| next.is_ascii_digit()),
        '-' => {
            first.is_some_and(|next| next.is_ascii_digit())
                || rest.strip_prefix('.').is_some_and(|tail| {
                    tail.chars()
                        .next()
                        .is_some_and(|next| next.is_ascii_digit())
                })
        }
        _ => false,
    }
}
