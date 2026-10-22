//! PlantUML MIT 1.2026.9: BlockUmlBuilder, core/DiagramType; see provenance manifest.
use super::budget::{Budget, invalid, unsupported};
use crate::RenderError;

#[derive(Clone, Debug)]
pub(super) struct Line {
    pub number: usize,
    pub text: String,
}

pub(super) fn prepare(source: &str, budget: &mut Budget) -> Result<Vec<Line>, RenderError> {
    budget.charge(source.len())?;
    let mut lines = Vec::new();
    let mut block_comment = false;
    let mut state = 0;
    for (offset, raw) in source.trim_start_matches('\u{feff}').lines().enumerate() {
        budget.checkpoint()?;
        let line = offset + 1;
        let text = strip_comments(raw, &mut block_comment).trim().to_owned();
        if text.is_empty() {
            continue;
        }
        if text.starts_with('!') {
            return Err(unsupported(line, &text));
        }
        if text.eq_ignore_ascii_case("@startuml") && state == 0 {
            state = 1;
            continue;
        }
        if text.eq_ignore_ascii_case("@enduml") && state == 1 {
            state = 2;
            continue;
        }
        if state != 1 || text.starts_with('@') {
            return Err(invalid(
                line,
                format!("expected one @startuml/@enduml block, found {text:?}"),
            ));
        }
        lines.push(Line { number: line, text });
    }
    if block_comment {
        return Err(invalid(
            source.lines().count(),
            "unterminated block comment",
        ));
    }
    if state != 2 || lines.is_empty() {
        return Err(invalid(1, "expected a nonempty, complete @startuml block"));
    }
    Ok(lines)
}

fn strip_comments(raw: &str, block: &mut bool) -> String {
    let mut out = String::new();
    let mut rest = raw;
    let mut quoted = false;
    while let Some(ch) = rest.chars().next() {
        if *block {
            if let Some(end) = rest.find("'/") {
                rest = &rest[end + 2..];
                *block = false;
                continue;
            }
            break;
        }
        if quoted && ch == '\\' {
            out.push(ch);
            rest = &rest[1..];
            if let Some(next) = rest.chars().next() {
                out.push(next);
                rest = &rest[next.len_utf8()..];
            }
            continue;
        }
        if ch == '"' {
            quoted = !quoted;
        }
        if !quoted && rest.starts_with("/'") {
            *block = true;
            rest = &rest[2..];
            continue;
        }
        if !quoted && ch == '\'' && out.trim().is_empty() {
            break;
        }
        out.push(ch);
        rest = &rest[ch.len_utf8()..];
    }
    out
}
