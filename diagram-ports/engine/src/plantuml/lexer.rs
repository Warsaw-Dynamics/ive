//! Bounded lexical replacement for MIT PlantUML NameAndCodeParser and arrow command grammars.
use super::{
    budget::{Budget, invalid, unsupported},
    source::Line,
};
use crate::RenderError;

#[derive(Clone, Debug)]
pub(super) struct Token {
    pub text: String,
    pub quoted: bool,
}

pub(super) fn tokens(line: &Line, budget: &mut Budget) -> Result<Vec<Token>, RenderError> {
    budget.charge(line.text.len())?;
    let mut rest = line.text.as_str();
    let mut out = Vec::new();
    while !rest.is_empty() {
        rest = rest.trim_start();
        if rest.is_empty() {
            break;
        }
        let (text, quoted, used) = next(rest, line.number)?;
        out.push(Token { text, quoted });
        rest = &rest[used..];
    }
    Ok(out)
}

fn next(rest: &str, line: usize) -> Result<(String, bool, usize), RenderError> {
    if rest.starts_with('"') {
        return quoted(rest, line);
    }
    if let Some(arrow) = arrow_at(rest) {
        return Ok((arrow.into(), false, arrow.len()));
    }
    let first = rest.chars().next().unwrap();
    if matches!(first, '{' | '}' | ':') {
        return Ok((first.to_string(), false, first.len_utf8()));
    }
    let end = rest
        .char_indices()
        .skip(1)
        .find(|(i, ch)| {
            ch.is_whitespace()
                || matches!(ch, '{' | '}' | ':' | '"')
                || arrow_at(&rest[*i..]).is_some()
        })
        .map_or(rest.len(), |(i, _)| i);
    Ok((rest[..end].into(), false, end))
}

fn quoted(rest: &str, line: usize) -> Result<(String, bool, usize), RenderError> {
    let mut out = String::new();
    let mut escaped = false;
    for (i, ch) in rest.char_indices().skip(1) {
        super::budget::bound("label bytes", i - 1, 16 * 1024)?;
        if ch != '"' || escaped {
            super::budget::bound("label bytes", i - 1 + ch.len_utf8(), 16 * 1024)?;
        }
        if escaped {
            match ch {
                'n' | '\\' | '~' => {
                    out.push('\\');
                    out.push(ch);
                }
                '"' => out.push(ch),
                _ => return Err(unsupported(line, &format!("quoted escape \\{ch}"))),
            }
            escaped = false;
            continue;
        }
        match ch {
            '\\' => escaped = true,
            '"' => return Ok((out, true, i + 1)),
            _ => out.push(ch),
        }
    }
    Err(invalid(line, "unterminated quoted identifier"))
}

pub(super) fn arrow_at(rest: &str) -> Option<&str> {
    if !["--", "->", "..", "<-", "<|", "o-", "o.", "*-", "*.", "<."]
        .iter()
        .any(|p| rest.starts_with(p))
    {
        return None;
    }
    let end = rest
        .find(|ch: char| !matches!(ch, '-' | '.' | '<' | '>' | '|' | 'o' | '*'))
        .unwrap_or(rest.len());
    Some(&rest[..end])
}

pub(super) fn split_label(line: &Line) -> (&str, &str) {
    let mut quoted = false;
    let mut escaped = false;
    for (i, ch) in line.text.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if quoted && ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == '"' {
            quoted = !quoted;
        }
        if ch == ':'
            && !quoted
            && !line.text[..i].ends_with(':')
            && !line.text[i + 1..].starts_with(':')
        {
            return (line.text[..i].trim(), line.text[i + 1..].trim());
        }
    }
    (&line.text, "")
}

pub(super) fn name(tokens: &[Token], line: usize) -> Result<(String, String), RenderError> {
    for token in tokens {
        if token.text != "as" {
            validate_name(token, line)?;
        }
    }
    match tokens {
        [one] => Ok((one.text.clone(), one.text.clone())),
        [display, join, alias] if join.text == "as" && !alias.quoted => {
            Ok((alias.text.clone(), display.text.clone()))
        }
        [alias, join, display] if join.text == "as" && display.quoted => {
            Ok((alias.text.clone(), display.text.clone()))
        }
        _ => Err(unsupported(
            line,
            "declaration syntax (expected name or quoted display as alias)",
        )),
    }
}

pub(super) fn validate_name(token: &Token, line: usize) -> Result<(), RenderError> {
    if !token.quoted {
        validate_identifier(&token.text, line)?;
    }
    super::text::validate_label(&token.text, line)
}

pub(super) fn validate_identifier(value: &str, line: usize) -> Result<(), RenderError> {
    if value.is_empty()
        || value
            .chars()
            .any(|ch| !(ch.is_alphanumeric() || matches!(ch, '_' | '.' | '@')))
    {
        return Err(unsupported(
            line,
            &format!("identifier {value:?} or attached syntax"),
        ));
    }
    Ok(())
}
