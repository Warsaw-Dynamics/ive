//! Plain runs from MIT CommandCreoleStyle, FontStyle, Display and CharHidder.
//! Copyright 2009-2024 Arnaud Roques; no PUA sentinels or resource commands.
use super::{
    budget::{Budget, bound, unsupported},
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
enum Mark {
    Bold,
    Italic,
    Underline,
    Strike,
    Wave,
    Mono,
}
impl Mark {
    fn symbol(self) -> &'static str {
        ["**", "//", "__", "--", "~~", "\"\""][self as usize]
    }
}
enum Token {
    Text(String),
    Symbol(Mark, bool),
    Open(Mark),
    Close(Mark),
    Break,
}

pub(super) fn parse(
    text: &str,
    base: RunStyle,
    budget: &mut Budget,
) -> Result<Vec<Vec<Run>>, RenderError> {
    bound("label bytes", text.len(), 16 * 1024)?;
    let mut tokens = lex(text, budget)?;
    activate(&mut tokens, budget)?;
    runs(tokens, base, budget)
}

fn lex(text: &str, budget: &mut Budget) -> Result<Vec<Token>, RenderError> {
    let mut tokens = Vec::new();
    let mut rest = text;
    let mut url = false;
    let mut line_start = true;
    while let Some(ch) = rest.chars().next() {
        budget.charge(ch.len_utf8())?;
        if line_start {
            reject_block(rest, budget)?;
            line_start = false;
        }
        if ch.is_whitespace() {
            url = false;
        }
        if let Some(scheme) = ["https://", "http://", "ftp://", "file://"]
            .into_iter()
            .find(|s| rest.starts_with(s))
        {
            literal(&mut tokens, scheme, budget)?;
            rest = &rest[scheme.len()..];
            url = true;
            continue;
        }
        if ch == '\\' {
            let Some(next) = rest[1..].chars().next() else {
                return Err(unsupported(1, "trailing label escape"));
            };
            match next {
                'n' => {
                    push(&mut tokens, Token::Break, budget)?;
                    line_start = true;
                }
                '\\' | '~' | '"' => literal(&mut tokens, &next.to_string(), budget)?,
                _ => return Err(unsupported(1, &format!("label escape \\{next}"))),
            }
            rest = &rest[1 + next.len_utf8()..];
            continue;
        }
        if ch == '~'
            && let Some(next) = rest[1..]
                .chars()
                .next()
                .filter(|c| matches!(c, '_' | '-' | '"' | '#' | ']' | '[' | '*' | '.' | '/' | '<'))
        {
            literal(&mut tokens, &next.to_string(), budget)?;
            rest = &rest[1 + next.len_utf8()..];
            continue;
        }
        if ch == '\n' {
            push(&mut tokens, Token::Break, budget)?;
            rest = &rest[1..];
            line_start = true;
            continue;
        }
        if let Some((token, used)) = tag(rest) {
            push(&mut tokens, token, budget)?;
            rest = &rest[used..];
            continue;
        }
        if rest.starts_with("[[") || (ch == '<' && rich_tag(&rest[1..])) {
            return Err(unsupported(1, "rich text, links, images, sprites or math"));
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
                literal(&mut tokens, "//", budget)?;
            } else {
                push(&mut tokens, Token::Symbol(mark, false), budget)?;
            }
            rest = &rest[2..];
            continue;
        }
        literal(&mut tokens, &rest[..ch.len_utf8()], budget)?;
        rest = &rest[ch.len_utf8()..];
    }
    Ok(tokens)
}

fn literal(tokens: &mut Vec<Token>, text: &str, budget: &mut Budget) -> Result<(), RenderError> {
    budget.text(text.len())?;
    if let Some(Token::Text(previous)) = tokens.last_mut() {
        previous.push_str(text);
    } else {
        bound("label tokens", tokens.len() + 1, 4096)?;
        budget.item()?;
        tokens.push(Token::Text(text.into()));
    }
    Ok(())
}
fn push(tokens: &mut Vec<Token>, token: Token, budget: &mut Budget) -> Result<(), RenderError> {
    bound("label tokens", tokens.len() + 1, 4096)?;
    budget.item()?;
    tokens.push(token);
    Ok(())
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

fn reject_block(rest: &str, budget: &mut Budget) -> Result<(), RenderError> {
    let line = rest.trim_start_matches([' ', '\t']);
    budget.charge(rest.len() - line.len() + 1)?;
    let list = line.trim_start_matches(['*', '#']);
    budget.charge(line.len() - list.len())?;
    if line.starts_with(['|', '='])
        || line.starts_with("----")
        || (list.len() < line.len() && list.starts_with([' ', '\t']))
    {
        return Err(unsupported(
            1,
            "Creole table, heading, list or horizontal rule",
        ));
    }
    Ok(())
}

// Pair each marker with the next occurrence on its physical/explicit line.
// An odd last occurrence stays literal. Each token is visited twice at most.
fn activate(tokens: &mut [Token], budget: &mut Budget) -> Result<(), RenderError> {
    let mut pending = [None; 6];
    for index in 0..tokens.len() {
        budget.charge(1)?;
        match tokens[index] {
            Token::Symbol(mark, _) => {
                if let Some(previous) = pending[mark as usize].take() {
                    if matches!(mark, Mark::Wave | Mark::Mono) {
                        return Err(unsupported(1, mark.symbol()));
                    }
                    tokens[previous] = Token::Symbol(mark, true);
                    tokens[index] = Token::Symbol(mark, true);
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

fn runs(
    tokens: Vec<Token>,
    base: RunStyle,
    budget: &mut Budget,
) -> Result<Vec<Vec<Run>>, RenderError> {
    let mut lines = vec![Vec::new()];
    let mut stack: Vec<(Mark, bool, RunStyle, usize)> = Vec::new();
    let mut style = base;
    let mut characters = 0;
    for token in tokens {
        budget.charge(1)?;
        match token {
            Token::Text(text) => {
                characters += text.len();
                append(lines.last_mut().unwrap(), text, &style, budget)?;
            }
            Token::Symbol(mark, false) => {
                characters += 2;
                append(
                    lines.last_mut().unwrap(),
                    mark.symbol().into(),
                    &style,
                    budget,
                )?;
            }
            Token::Break => {
                budget.item()?;
                lines.push(Vec::new());
            }
            token => {
                let (mark, close, tag) = match token {
                    Token::Open(mark) => (mark, false, true),
                    Token::Close(mark) => (mark, true, true),
                    Token::Symbol(mark, true) => (mark, stack.iter().any(|s| s.0 == mark), false),
                    _ => unreachable!(),
                };
                budget.charge(stack.len() + 1)?;
                if close {
                    let Some((opened, was_tag, parent, start)) = stack.pop() else {
                        return Err(unsupported(1, "closing unopened styled span"));
                    };
                    if opened != mark || was_tag != tag || start == characters {
                        return Err(unsupported(1, "crossed or empty styled span"));
                    }
                    style = parent;
                } else {
                    if stack.iter().any(|s| s.0 == mark) {
                        return Err(unsupported(1, "nested duplicate style"));
                    }
                    budget.depth(stack.len() + 1)?;
                    budget.item()?;
                    stack.push((mark, tag, style.clone(), characters));
                    match mark {
                        Mark::Bold => style.font.weight = 700,
                        Mark::Italic => style.font.italic = true,
                        Mark::Underline => style.underline = true,
                        Mark::Strike => style.strike = true,
                        _ => unreachable!(),
                    }
                }
            }
        }
    }
    if !stack.is_empty() {
        return Err(unsupported(1, "unclosed styled span"));
    }
    Ok(lines)
}

fn append(
    runs: &mut Vec<Run>,
    text: String,
    style: &RunStyle,
    budget: &mut Budget,
) -> Result<(), RenderError> {
    budget.text(text.len())?;
    if let Some(last) = runs.last_mut().filter(|run| run.style == *style) {
        last.text.push_str(&text);
    } else {
        budget.item()?;
        runs.push(Run {
            text,
            style: style.clone(),
        });
    }
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
