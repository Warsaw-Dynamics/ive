//! PlantUML MIT 1.2026.9: BlockUmlBuilder, core/DiagramType; see provenance manifest.
use super::budget::{Budget, invalid, unsupported};
use crate::RenderError;

#[derive(Clone, Debug)]
pub(super) struct Line {
    pub number: usize,
    pub text: String,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Wrapper {
    Uml,
    Mindmap,
    Wbs,
    Json,
    Yaml,
    Gantt,
    Salt,
}

pub(super) fn prepare_document(
    source: &str,
    budget: &mut Budget,
) -> Result<(Wrapper, Vec<Line>), RenderError> {
    let first = source
        .trim_start_matches('\u{feff}')
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or("")
        .trim();
    let (wrapper, end) = match first {
        "@startmindmap" => (Wrapper::Mindmap, "@endmindmap"),
        "@startwbs" => (Wrapper::Wbs, "@endwbs"),
        "@startjson" => (Wrapper::Json, "@endjson"),
        "@startyaml" => (Wrapper::Yaml, "@endyaml"),
        "@startgantt" => (Wrapper::Gantt, "@endgantt"),
        "@startsalt" => (Wrapper::Salt, "@endsalt"),
        _ => return prepare(source, budget).map(|lines| (Wrapper::Uml, lines)),
    };
    budget.charge(source.len())?;
    let mut state = 0;
    let mut lines = Vec::new();
    let mut block_comment = false;
    for (offset, raw) in source.trim_start_matches('\u{feff}').lines().enumerate() {
        budget.charge(raw.len() + 1)?;
        let normalized;
        let raw = if wrapper == Wrapper::Gantt {
            normalized = strip_comments(raw, &mut block_comment, budget)?;
            normalized.trim()
        } else {
            raw
        };
        let trimmed = raw.trim();
        if state == 0 && trimmed == first {
            state = 1;
        } else if state == 1 && trimmed == end {
            state = 2;
        } else if state == 1 {
            reject_active_preprocessor(raw, offset + 1)?;
            if trimmed.starts_with("@start") || trimmed.starts_with("@end") {
                return Err(invalid(
                    offset + 1,
                    format!("expected matching {end}, found {trimmed:?}"),
                ));
            }
            budget.text(raw.len())?;
            if wrapper == Wrapper::Gantt && raw.is_empty() {
                continue;
            }
            lines.push(Line {
                number: offset + 1,
                text: raw.to_owned(),
            });
        } else if !trimmed.is_empty() {
            return Err(invalid(
                offset + 1,
                "expected exactly one complete PlantUML block",
            ));
        }
    }
    if state != 2 || lines.iter().all(|line| line.text.trim().is_empty()) {
        return Err(invalid(
            1,
            format!("expected nonempty complete {first}/{end} block"),
        ));
    }
    Ok((wrapper, lines))
}

pub(super) fn reject_active_preprocessor(raw: &str, line: usize) -> Result<(), RenderError> {
    let Some(command) = raw.trim_start().strip_prefix('!') else {
        return Ok(());
    };
    let word = command
        .trim_start()
        .split(|ch: char| !ch.is_alphanumeric() && ch != '_')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    if command.contains('=')
        || matches!(
            word.as_str(),
            "include"
                | "includeurl"
                | "include_many"
                | "include_once"
                | "includedef"
                | "includesub"
                | "define"
                | "definelong"
                | "undef"
                | "if"
                | "ifdef"
                | "ifndef"
                | "else"
                | "elseif"
                | "endif"
                | "while"
                | "endwhile"
                | "foreach"
                | "endfor"
                | "function"
                | "procedure"
                | "endfunction"
                | "endprocedure"
                | "enddefinelong"
                | "end"
                | "final"
                | "unquoted"
                | "return"
                | "assert"
                | "theme"
                | "import"
                | "startsub"
                | "endsub"
                | "log"
                | "dump_memory"
                | "option"
        )
    {
        return Err(unsupported(line, raw));
    }
    Ok(())
}

pub(super) fn prepare(source: &str, budget: &mut Budget) -> Result<Vec<Line>, RenderError> {
    budget.charge(source.len())?;
    let mut lines = Vec::new();
    let mut block_comment = false;
    let mut state = 0;
    let mut body = TextBody::None;
    let mut graph_grammar = false;
    let mut timing_statement = None;
    let mut timing_declaration = false;
    for (offset, raw) in source.trim_start_matches('\u{feff}').lines().enumerate() {
        budget.checkpoint()?;
        let line = offset + 1;
        if (body != TextBody::None || (state == 1 && !graph_grammar && !block_comment))
            && let Some(next) = body.consume_raw(raw)
        {
            if raw.trim_start().starts_with('!') {
                return Err(unsupported(line, raw));
            }
            budget.text(raw.len())?;
            let text = if body == TextBody::Note && raw.trim() == "end note" {
                "end note".to_owned()
            } else if body == TextBody::None {
                raw.trim_start().to_owned()
            } else {
                raw.to_owned()
            };
            body = next;
            lines.push(Line { number: line, text });
            continue;
        }
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
        if state != 1
            || text.eq_ignore_ascii_case("@startuml")
            || text.eq_ignore_ascii_case("@enduml")
        {
            return Err(invalid(
                line,
                format!("expected one @startuml/@enduml block, found {text:?}"),
            ));
        }
        let statement = Line { number: line, text };
        if statement.text.starts_with('@') && timing_statement.is_none() {
            timing_statement = Some(lines.len());
        }
        timing_declaration |= matches!(
            statement.text.split_whitespace().next(),
            Some("robust" | "concise")
        );
        graph_grammar |= graph_declaration(&statement);
        if !graph_grammar {
            body = text_body(&statement);
        }
        lines.push(statement);
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
    // Raw note/action rows never grant or require timing syntax.
    if timing_declaration {
        return Ok(lines);
    }
    if let Some(index) = timing_statement {
        let line = &lines[index];
        return Err(invalid(
            line.number,
            format!("timing declaration required for {:?}", line.text),
        ));
    }
    Ok(lines)
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum TextBody {
    None,
    Action,
    Note,
}

impl TextBody {
    /// Returns the next body state when this row belongs to raw note/action text.
    pub fn consume_raw(self, raw: &str) -> Option<Self> {
        match self {
            Self::Note => Some(if raw.trim() == "end note" {
                Self::None
            } else {
                Self::Note
            }),
            Self::Action => Some(if raw.trim_end().ends_with(';') {
                Self::None
            } else {
                Self::Action
            }),
            Self::None if raw.trim_start().starts_with(':') => Self::Action.consume_raw(raw),
            Self::None => None,
        }
    }
}

pub(super) fn text_body(line: &Line) -> TextBody {
    if line.text.starts_with(':') && !line.text.ends_with(';') {
        return TextBody::Action;
    }
    if ["note over ", "note left", "note right", "note beside "]
        .iter()
        .any(|prefix| line.text.starts_with(prefix))
        && super::lexer::split_label(line).0 == line.text
    {
        return TextBody::Note;
    }
    TextBody::None
}

pub(super) fn graph_declaration(line: &Line) -> bool {
    !super::has_identifier_arrow(line)
        && matches!(
            line.text.split_whitespace().next().unwrap_or(""),
            "class"
                | "abstract"
                | "interface"
                | "enum"
                | "annotation"
                | "object"
                | "state"
                | "component"
                | "node"
                | "rectangle"
                | "package"
                | "namespace"
                | "usecase"
        )
}

pub(super) fn strip_comments(
    raw: &str,
    block: &mut bool,
    budget: &mut Budget,
) -> Result<String, RenderError> {
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
    fn should_preserve_literal_raw_rows_around_nonleading_exclamation_marks() {
        // given
        let source = "@startuml\n:opening ! literal\n  body ! literal  \n\nending;\n@enduml";
        let mut budget = Budget::new(&RenderOptions::default());
        // when
        let lines = prepare(source, &mut budget).unwrap();
        // then
        assert_eq!(
            lines.iter().map(|l| l.text.as_str()).collect::<Vec<_>>(),
            [":opening ! literal", "  body ! literal  ", "", "ending;"]
        );
    }

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
