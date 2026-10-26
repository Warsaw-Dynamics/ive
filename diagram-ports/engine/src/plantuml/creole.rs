//! Plain runs from MIT CommandCreoleStyle, FontStyle, Display and CharHidder.
//! Copyright 2009-2024 Arnaud Roques; no PUA sentinels or resource commands.
use super::{
    budget::{Budget, bound},
    parser::{context, error::ParseFailure},
    style::Font,
};
use crate::RenderError;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub(super) struct RunStyle {
    pub font: Font,
    pub underline: bool,
    pub strike: bool,
}
#[derive(Clone, Debug)]
pub(super) struct Run {
    pub text: String,
    pub style: RunStyle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::plantuml) enum Mark {
    Bold,
    Italic,
    Underline,
    Strike,
    Wave,
    Mono,
}
impl Mark {
    pub fn symbol(self) -> &'static str {
        ["**", "//", "__", "--", "~~", "\"\""][self as usize]
    }
}

#[derive(Clone, Copy, Debug)]
pub(in crate::plantuml) enum Pair {
    Literal,
    Open,
    Close,
}
#[derive(Clone, Debug)]
pub(in crate::plantuml) enum Token {
    Text(String),
    Symbol(Mark, Pair),
    Open(Mark),
    Close(Mark),
    Break,
}
pub(in crate::plantuml) type Spanned = (usize, Token, usize);
pub(super) fn parse(
    text: &str,
    base: RunStyle,
    budget: &mut Budget,
) -> Result<Vec<Vec<Run>>, RenderError> {
    super::parser::creole::parse(text, base, budget)
}
pub(in crate::plantuml) fn lex(
    text: &str,
    budget: &mut Budget,
) -> Result<Vec<Spanned>, ParseFailure> {
    lex_tokens((text, Escapes::Newlines), budget)
}
pub(in crate::plantuml) fn lex_display(
    text: &str,
    budget: &mut Budget,
) -> Result<Vec<Spanned>, ParseFailure> {
    lex_tokens((text, Escapes::Literal), budget)
}
#[derive(PartialEq, Eq)]
enum Escapes {
    Newlines,
    Literal,
}
fn lex_tokens(input: (&str, Escapes), budget: &mut Budget) -> Result<Vec<Spanned>, ParseFailure> {
    let (text, escapes) = input;
    bound("label bytes", text.len(), 16 * 1024)?;
    bound(
        "Creole lexical retained bytes",
        3usize
            .saturating_mul(text.len().saturating_add(1))
            .saturating_mul(std::mem::size_of::<Spanned>())
            .saturating_add(text.len().saturating_mul(4))
            .saturating_add(6 * std::mem::size_of::<Option<usize>>()),
        budget.limits.source_bytes.saturating_mul(256),
    )?;
    let mut tokens = Vec::new();
    let mut rest = text;
    let mut url = false;
    let mut line_start = true;
    while let Some(ch) = rest.chars().next() {
        let offset = text.len() - rest.len();
        budget.charge(ch.len_utf8())?;
        if line_start {
            reject_block((rest, offset), budget)?;
            line_start = false;
        }
        if ch.is_whitespace() {
            url = false;
        }
        if let Some(scheme) = ["https://", "http://", "ftp://", "file://"]
            .into_iter()
            .find(|scheme| rest.starts_with(scheme))
        {
            literal(&mut tokens, (scheme, offset..offset + scheme.len()), budget)?;
            rest = &rest[scheme.len()..];
            url = true;
            continue;
        }
        if escapes == Escapes::Literal && (ch == '\t' || rest.starts_with("\\t")) {
            return Err(ParseFailure::unsupported(
                offset..offset + ch.len_utf8(),
                "flow label tabulation",
            ));
        }
        if ch == '\\' && escapes == Escapes::Newlines {
            let Some(next) = rest[1..].chars().next() else {
                return Err(ParseFailure::unsupported(
                    offset..offset + 1,
                    "trailing label escape",
                ));
            };
            let span = offset..offset + 1 + next.len_utf8();
            match next {
                'n' => {
                    push(&mut tokens, (Token::Break, span), budget)?;
                    line_start = true;
                }
                '\\' | '~' | '"' => literal(&mut tokens, (&next.to_string(), span), budget)?,
                _ => {
                    return Err(ParseFailure {
                        span,
                        kind: super::parser::error::FailureKind::Unsupported(
                            format!("label escape \\{next}").into(),
                        ),
                    });
                }
            }
            rest = &rest[1 + next.len_utf8()..];
            continue;
        }
        if ch == '~'
            && let Some(next) = rest[1..].chars().next().filter(|ch| {
                matches!(
                    ch,
                    '_' | '-' | '"' | '#' | ']' | '[' | '*' | '.' | '/' | '<'
                )
            })
        {
            literal(
                &mut tokens,
                (&next.to_string(), offset..offset + 1 + next.len_utf8()),
                budget,
            )?;
            rest = &rest[1 + next.len_utf8()..];
            continue;
        }
        if ch == '\n' {
            push(&mut tokens, (Token::Break, offset..offset + 1), budget)?;
            rest = &rest[1..];
            line_start = true;
            continue;
        }
        if let Some((token, used)) = tag(rest) {
            push(&mut tokens, (token, offset..offset + used), budget)?;
            rest = &rest[used..];
            continue;
        }
        if rest.starts_with("[[") || (ch == '<' && rich_tag(&rest[1..])) {
            return Err(ParseFailure::unsupported(
                offset..offset + ch.len_utf8(),
                "rich text, links, images, sprites or math",
            ));
        }
        if let Some(mark) = [
            Mark::Bold,
            Mark::Italic,
            Mark::Underline,
            Mark::Strike,
            Mark::Wave,
            Mark::Mono,
        ]
        .into_iter()
        .find(|mark| rest.starts_with(mark.symbol()))
        {
            if mark == Mark::Italic && url {
                literal(&mut tokens, ("//", offset..offset + 2), budget)?;
            } else {
                push(
                    &mut tokens,
                    (Token::Symbol(mark, Pair::Literal), offset..offset + 2),
                    budget,
                )?;
            }
            rest = &rest[2..];
            continue;
        }
        literal(
            &mut tokens,
            (&rest[..ch.len_utf8()], offset..offset + ch.len_utf8()),
            budget,
        )?;
        rest = &rest[ch.len_utf8()..];
    }
    activate(&mut tokens, budget)?;
    Ok(tokens)
}
fn literal(
    tokens: &mut Vec<Spanned>,
    value: (&str, std::ops::Range<usize>),
    budget: &mut Budget,
) -> Result<(), ParseFailure> {
    let (text, span) = value;
    budget.text(text.len())?;
    if let Some((_, Token::Text(previous), end)) = tokens.last_mut() {
        append_text(previous, text, budget)?;
        *end = span.end;
    } else {
        push(tokens, (Token::Text(text.into()), span), budget)?;
    }
    Ok(())
}
fn push(
    tokens: &mut Vec<Spanned>,
    value: (Token, std::ops::Range<usize>),
    budget: &mut Budget,
) -> Result<(), ParseFailure> {
    bound("label tokens", tokens.len() + 1, 4096)?;
    budget.item()?;
    context::push(tokens, (value.1.start, value.0, value.1.end), budget)
}
fn tag(rest: &str) -> Option<(Token, usize)> {
    if !rest.starts_with('<') {
        return None;
    }
    for (open, close, mark) in [
        ("<b>", "</b>", Mark::Bold),
        ("<i>", "</i>", Mark::Italic),
        ("<u>", "</u>", Mark::Underline),
        ("<s>", "</s>", Mark::Strike),
    ] {
        if rest.get(..3).is_some_and(|s| s.eq_ignore_ascii_case(open)) {
            return Some((Token::Open(mark), 3));
        }
        if rest.get(..4).is_some_and(|s| s.eq_ignore_ascii_case(close)) {
            return Some((Token::Close(mark), 4));
        }
    }
    None
}

fn reject_block(value: (&str, usize), budget: &mut Budget) -> Result<(), ParseFailure> {
    let (rest, offset) = value;
    let line = rest.trim_start_matches([' ', '\t']);
    budget.charge(rest.len() - line.len() + 1)?;
    let list = line.trim_start_matches(['*', '#']);
    budget.charge(line.len() - list.len())?;
    if line.starts_with(['|', '='])
        || line.starts_with("----")
        || (list.len() < line.len() && list.starts_with([' ', '\t']))
    {
        return Err(ParseFailure::unsupported(
            offset..offset + rest.len() - line.len() + 1,
            "Creole table, heading, list or horizontal rule",
        ));
    }
    Ok(())
}
// Same-line pair eligibility alone; generated productions own nested structure.
fn activate(tokens: &mut [Spanned], budget: &mut Budget) -> Result<(), ParseFailure> {
    let mut pending: [Option<usize>; 6] = [None; 6];
    for index in 0..tokens.len() {
        budget.charge(1)?;
        match tokens[index].1 {
            Token::Symbol(mark, _) => {
                if let Some(previous) = pending[mark as usize].take() {
                    if matches!(mark, Mark::Wave | Mark::Mono) {
                        return Err(ParseFailure::unsupported(
                            tokens[previous].0..tokens[index].2,
                            "unsupported paired wave or monospace style",
                        ));
                    }
                    tokens[previous].1 = Token::Symbol(mark, Pair::Open);
                    tokens[index].1 = Token::Symbol(mark, Pair::Close);
                } else {
                    pending[mark as usize] = Some(index);
                }
            }
            Token::Break => pending = [None; 6],
            _ => {}
        }
    }
    Ok(())
}
pub(in crate::plantuml) struct Scope {
    pub parent: RunStyle,
    mark: Mark,
    start: usize,
}
pub(in crate::plantuml) struct Builder<'b> {
    pub budget: &'b mut Budget,
    lines: Vec<Vec<Run>>,
    style: RunStyle,
    active: u8,
    characters: usize,
}
impl<'b> Builder<'b> {
    pub fn new(base: RunStyle, budget: &'b mut Budget) -> Self {
        Self {
            budget,
            lines: vec![Vec::new()],
            style: base,
            active: 0,
            characters: 0,
        }
    }
    pub fn tick(&mut self) -> Result<(), ParseFailure> {
        self.budget.charge(1).map_err(Into::into)
    }
    pub fn open(
        &mut self,
        mark: Mark,
        span: std::ops::Range<usize>,
    ) -> Result<Scope, ParseFailure> {
        self.tick()?;
        if self.active & (1 << (mark as u8)) != 0 {
            return Err(ParseFailure::unsupported(span, "nested duplicate style"));
        }
        self.budget.depth(self.active.count_ones() as usize + 1)?;
        self.budget.item()?;
        let scope = Scope {
            parent: self.style.clone(),
            mark,
            start: self.characters,
        };
        self.active |= 1 << (mark as u8);
        match mark {
            Mark::Bold => self.style.font.weight = 700,
            Mark::Italic => self.style.font.italic = true,
            Mark::Underline => self.style.underline = true,
            Mark::Strike => self.style.strike = true,
            _ => unreachable!(),
        }
        Ok(scope)
    }
    pub fn close(
        &mut self,
        scope: Scope,
        span: std::ops::Range<usize>,
    ) -> Result<(), ParseFailure> {
        self.tick()?;
        if scope.start == self.characters {
            return Err(ParseFailure::unsupported(span, "empty styled span"));
        }
        self.active &= !(1 << (scope.mark as u8));
        self.style = scope.parent;
        Ok(())
    }
    pub fn text(&mut self, text: String) -> Result<(), ParseFailure> {
        self.tick()?;
        self.budget.text(text.len())?;
        self.characters += text.len();
        let runs = self.lines.last_mut().unwrap();
        if let Some(last) = runs.last_mut().filter(|run| run.style == self.style) {
            append_text(&mut last.text, &text, self.budget)?;
        } else {
            self.budget.item()?;
            context::push(
                runs,
                Run {
                    text,
                    style: self.style.clone(),
                },
                self.budget,
            )?;
        }
        Ok(())
    }
    pub fn literal(&mut self, text: &'static str) -> Result<(), ParseFailure> {
        self.text(text.into())
    }
    pub fn line(&mut self) -> Result<(), ParseFailure> {
        self.budget.item()?;
        context::push(&mut self.lines, Vec::new(), self.budget)
    }
    pub fn finish(self) -> Vec<Vec<Run>> {
        self.lines
    }
}
fn append_text(out: &mut String, text: &str, budget: &mut Budget) -> Result<(), ParseFailure> {
    bound(
        "label bytes",
        out.len().saturating_add(text.len()),
        16 * 1024,
    )?;
    if out.len().saturating_add(text.len()) > out.capacity() {
        let capacity = out
            .capacity()
            .saturating_mul(2)
            .max(out.len() + text.len())
            .max(8);
        budget.charge(out.len())?;
        out.try_reserve_exact(capacity - out.len())
            .map_err(|error| {
                ParseFailure::terminal(RenderError::Backend(format!(
                    "Creole text allocation for capacity {capacity} failed: {error}"
                )))
            })?;
    }
    out.push_str(text);
    Ok(())
}
fn rich_tag(rest: &str) -> bool {
    let rest = rest.strip_prefix('/').unwrap_or(rest);
    if rest.starts_with(['$', '&', ':', '#']) {
        return true;
    }
    let end = rest
        .find(|ch: char| !ch.is_ascii_alphabetic())
        .unwrap_or(rest.len());
    matches!(
        rest.as_bytes().get(end),
        Some(b'>' | b':' | b'/' | b' ' | b'\t')
    ) && [
        "b", "i", "u", "s", "strike", "del", "w", "plain", "back", "color", "size", "font", "text",
        "sup", "sub", "img", "image", "sprite", "math", "latex", "qrcode", "space", "table", "tr",
        "td", "left", "right", "center", "l", "c", "r", "style", "script", "strong", "em", "span",
    ]
    .iter()
    .any(|name| rest[..end].eq_ignore_ascii_case(name))
}
