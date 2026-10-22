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
        let text = strip_comments(raw, &mut block_comment, budget)?
            .trim()
            .to_owned();
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

fn strip_comments(raw: &str, block: &mut bool, budget: &mut Budget) -> Result<String, RenderError> {
    let mut out = String::new();
    let mut rest = raw;
    let mut quoted = false;
    let mut content = false;
    let mut charged = 0;
    while let Some(ch) = rest.chars().next() {
        let consumed = raw.len() - rest.len();
        if consumed - charged >= 512 {
            budget.charge(consumed - charged)?;
            charged = consumed;
        }
        if *block {
            if rest.starts_with("'/") {
                rest = &rest[2..];
                *block = false;
            } else {
                rest = &rest[ch.len_utf8()..];
            }
            continue;
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
        if !quoted && ch == '\'' && !content {
            break;
        }
        content |= !ch.is_whitespace();
        out.push(ch);
        rest = &rest[ch.len_utf8()..];
    }
    budget.charge(raw.len() - rest.len() - charged)?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RenderOptions;

    #[test]
    fn should_charge_long_line_scanning_before_finishing_the_line() {
        // given
        let source = format!(
            "@startuml\n{}A{}\n@enduml",
            " ".repeat(2048),
            "'".repeat(2048)
        );
        let mut config = RenderOptions::default();
        config.limits.layout_work = source.len() + 512;
        let mut budget = Budget::new(&config);
        // when
        let result = prepare(&source, &mut budget);
        // then
        assert!(
            result.is_err(),
            "scan work was not charged before retaining the line"
        );
    }
}
