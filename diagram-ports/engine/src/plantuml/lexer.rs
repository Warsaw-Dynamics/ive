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

pub(super) fn model_tokens(line: &Line, budget: &mut Budget) -> Result<Vec<Token>, RenderError> {
    scan_tokens(line, budget)
}

fn scan_tokens(line: &Line, budget: &mut Budget) -> Result<Vec<Token>, RenderError> {
    budget.charge(line.text.len())?;
    let mut rest = line.text.as_str();
    let mut out = Vec::new();
    while !rest.is_empty() {
        rest = rest.trim_start();
        if rest.is_empty() {
            break;
        }
        budget.item()?;
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
        super::budget::bound("Chen token bytes", arrow.len(), 16 * 1024)?;
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
    super::budget::bound("Chen token bytes", end, 16 * 1024)?;
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
