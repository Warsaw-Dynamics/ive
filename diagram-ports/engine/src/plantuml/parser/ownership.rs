//! Shared byte ownership for both authored expansion and expanded document lexing.
use super::{error::ParseFailure, header};
use crate::plantuml::{budget::Budget, source::Wrapper};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::plantuml) enum Kind {
    Empty,
    Trivia,
    Start(Wrapper),
    End(Wrapper),
    Definition,
    SheetStart,
    SheetEnd,
    Content,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum Body {
    #[default]
    None,
    Note,
    Action,
}

pub(in crate::plantuml) struct Row {
    pub text: String,
    pub offsets: Vec<(usize, usize)>,
    pub kind: Kind,
    pub literal: bool,
    pub source_comment: bool,
    pub opening_action: bool,
    pub continuation: bool,
    pub annotated_start: bool,
}

#[derive(Clone, Copy)]
pub(in crate::plantuml) struct Ownership {
    block: bool,
    mode: Mode,
    body: Body,
    graph: bool,
    wrapper: Wrapper,
}

impl Default for Ownership {
    fn default() -> Self {
        Self {
            block: false,
            mode: Mode::Outside,
            body: Body::None,
            graph: false,
            wrapper: Wrapper::Uml,
        }
    }
}

#[derive(Clone, Copy, Default)]
enum Mode {
    #[default]
    Outside,
    Uml,
    Literal,
    Git,
    Wire,
    Other,
}

impl Ownership {
    pub fn railroad(&self) -> bool {
        matches!(self.wrapper, Wrapper::Ebnf | Wrapper::Regex) && matches!(self.mode, Mode::Literal)
    }
    pub fn classify(&mut self, raw: &str, budget: &mut Budget) -> Result<Row, ParseFailure> {
        for chunk in raw.as_bytes().chunks(256) {
            budget.charge(chunk.len()).map_err(ParseFailure::terminal)?;
        }
        let source_comment = self.block;
        let raw_body = self.body != Body::None
            || (matches!(self.mode, Mode::Uml)
                && !self.graph
                && !self.block
                && raw.trim_start().starts_with(':'));
        let literal = raw_body || matches!(self.mode, Mode::Literal | Mode::Git | Mode::Wire);
        let (text, offsets) = if literal || (!self.block && raw.trim_start().starts_with("!define"))
        {
            (raw.to_owned(), Vec::new())
        } else {
            strip(raw, &mut self.block, budget)?
        };
        let command = text.trim();
        let kind = match (raw_body, classify(command)) {
            (true, _) => Kind::Content,
            (_, Kind::Definition) if matches!(self.mode, Mode::Git) => Kind::Content,
            (_, Kind::SheetStart | Kind::SheetEnd)
                if matches!(self.mode, Mode::Literal | Mode::Git) =>
            {
                Kind::Content
            }
            (_, Kind::Empty) if !raw.trim().is_empty() => Kind::Trivia,
            (_, kind) => kind,
        };
        let annotated_start = matches!(kind, Kind::Start(_)) && raw.trim() != command;
        let opening_action = raw_body && self.body == Body::None;
        let continuation = raw_body && self.body != Body::None;
        Ok(Row {
            text,
            offsets,
            kind,
            literal,
            source_comment,
            opening_action,
            continuation,
            annotated_start,
        })
    }

    /// Advances next-row ownership from expanded text while preserving authored current-row ownership.
    pub fn advance(
        &mut self,
        row: &Row,
        expanded: &str,
        budget: &mut Budget,
    ) -> Result<header::Evidence, ParseFailure> {
        budget.checkpoint().map_err(ParseFailure::terminal)?;
        match row.kind {
            Kind::Start(wrapper) => {
                self.wrapper = wrapper;
                self.mode = match wrapper {
                    Wrapper::Wire => Mode::Wire,
                    Wrapper::Uml => Mode::Uml,
                    Wrapper::Flow
                    | Wrapper::Bpm
                    | Wrapper::Ebnf
                    | Wrapper::Regex
                    | Wrapper::Json
                    | Wrapper::Yaml
                    | Wrapper::Salt
                    | Wrapper::Files
                    | Wrapper::Packet
                    | Wrapper::Board
                    | Wrapper::Hcl => Mode::Literal,
                    Wrapper::Git => Mode::Git,
                    _ => Mode::Other,
                };
                self.graph = wrapper == Wrapper::Chen;
            }
            Kind::End(_) => self.mode = Mode::Outside,
            Kind::Content if matches!(self.mode, Mode::Uml) && !self.graph => {
                return self.advance_body(expanded, row.literal, budget);
            }
            _ => {}
        }
        Ok(header::Evidence::empty())
    }

    fn advance_body(
        &mut self,
        expanded: &str,
        literal: bool,
        budget: &mut Budget,
    ) -> Result<header::Evidence, ParseFailure> {
        match self.body {
            Body::Note => {
                self.body = if expanded.trim() == "end note" {
                    Body::None
                } else {
                    Body::Note
                };
            }
            Body::Action => {
                self.body = if expanded.trim_end().ends_with(';') {
                    Body::None
                } else {
                    Body::Action
                };
            }
            Body::None if literal => {
                self.body = if expanded.trim_end().ends_with(';') {
                    Body::None
                } else {
                    Body::Action
                };
            }
            Body::None => {
                let text = expanded.trim();
                if text.is_empty() {
                    return Ok(header::Evidence::empty());
                }
                let evidence = header::parse(text, 0, budget)?;
                self.graph |= evidence.graph_declaration;
                if self.graph {
                    return Ok(evidence);
                }
                if evidence.note_body {
                    self.body = Body::Note;
                }
                if text.starts_with(':') && !text.ends_with(';') {
                    self.body = Body::Action;
                }
                return Ok(evidence);
            }
        }
        Ok(header::Evidence::empty())
    }

    pub fn finish(&self, eof: usize) -> Result<(), ParseFailure> {
        if self.block {
            let detail = match self.wrapper {
                Wrapper::Nwdiag => "unterminated nwdiag block comment",
                Wrapper::Chen => "unterminated Chen block comment",
                _ => "unterminated block comment",
            };
            return Err(ParseFailure::syntax(eof..eof, detail));
        }
        Ok(())
    }
}

impl Row {
    pub fn span(&self, base: usize, range: std::ops::Range<usize>) -> std::ops::Range<usize> {
        let locate = |position| {
            let end = self
                .offsets
                .partition_point(|&(start, _)| start <= position);
            let delta = end.checked_sub(1).map_or(0, |index| {
                let (start, raw) = self.offsets[index];
                raw - start
            });
            base + position + delta
        };
        let end = if range.is_empty() {
            locate(range.end)
        } else {
            locate(range.end - 1) + 1
        };
        locate(range.start)..end
    }
}

fn classify(command: &str) -> Kind {
    if command.is_empty() {
        return Kind::Empty;
    }
    if command.eq_ignore_ascii_case("@startuml") {
        return Kind::Start(Wrapper::Uml);
    }
    if command.eq_ignore_ascii_case("@enduml") {
        return Kind::End(Wrapper::Uml);
    }
    for (name, wrapper) in [
        ("wire", Wrapper::Wire),
        ("chart", Wrapper::Chart),
        ("flow", Wrapper::Flow),
        ("mindmap", Wrapper::Mindmap),
        ("bpm", Wrapper::Bpm),
        ("ebnf", Wrapper::Ebnf),
        ("regex", Wrapper::Regex),
        ("wbs", Wrapper::Wbs),
        ("json", Wrapper::Json),
        ("yaml", Wrapper::Yaml),
        ("gantt", Wrapper::Gantt),
        ("salt", Wrapper::Salt),
        ("nwdiag", Wrapper::Nwdiag),
        ("files", Wrapper::Files),
        ("packetdiag", Wrapper::Packet),
        ("board", Wrapper::Board),
        ("hcl", Wrapper::Hcl),
        ("chen", Wrapper::Chen),
        ("git", Wrapper::Git),
    ] {
        if command.strip_prefix("@start") == Some(name) {
            return Kind::Start(wrapper);
        }
        if command.strip_prefix("@end") == Some(name) {
            return Kind::End(wrapper);
        }
    }
    if command.starts_with("!define ") || command.starts_with("!define\t") {
        return Kind::Definition;
    }
    if command.eq_ignore_ascii_case("<style>") {
        return Kind::SheetStart;
    }
    if command.eq_ignore_ascii_case("</style>") {
        return Kind::SheetEnd;
    }
    Kind::Content
}

/// Atomic two-byte delimiters are consumed before either byte can be reconsidered.
pub(in crate::plantuml) struct ByteScan {
    pub block: bool,
    quoted: bool,
    escaped: bool,
    content: bool,
    line_comment: bool,
}

pub(in crate::plantuml) enum Step {
    Active,
    Inert,
    Delimiter(usize),
}

impl ByteScan {
    pub fn new(block: bool) -> Self {
        Self {
            block,
            quoted: false,
            escaped: false,
            content: false,
            line_comment: false,
        }
    }

    pub fn consume(&mut self, rest: &str) -> Step {
        if self.line_comment {
            return Step::Inert;
        }
        if self.block {
            if rest.starts_with("'/") {
                self.block = false;
                return Step::Delimiter(2);
            }
            return Step::Inert;
        }
        let ch = rest.chars().next().unwrap();
        if self.quoted && self.escaped {
            self.escaped = false;
            return Step::Active;
        }
        if self.quoted && ch == '\\' {
            self.escaped = true;
            return Step::Active;
        }
        if ch == '"' {
            self.quoted = !self.quoted;
        }
        if !self.quoted && rest.starts_with("/'") {
            self.block = true;
            return Step::Delimiter(2);
        }
        if !self.quoted && ch == '\'' && !self.content {
            self.line_comment = true;
            return Step::Inert;
        }
        self.content |= !ch.is_whitespace();
        Step::Active
    }
}

fn strip(
    raw: &str,
    block: &mut bool,
    budget: &mut Budget,
) -> Result<(String, Vec<(usize, usize)>), ParseFailure> {
    let mut scanner = ByteScan::new(*block);
    let mut out = String::new();
    let mut offsets = Vec::new();
    let mut delta = 0;
    let mut cursor = 0;
    let mut charged = 0;
    while cursor < raw.len() {
        if cursor - charged >= 256 {
            budget
                .charge(cursor - charged)
                .map_err(ParseFailure::terminal)?;
            charged = cursor;
        }
        let ch = raw[cursor..].chars().next().unwrap();
        match scanner.consume(&raw[cursor..]) {
            Step::Delimiter(bytes) => {
                cursor += bytes;
                continue;
            }
            Step::Active => {
                if cursor - out.len() != delta {
                    delta = cursor - out.len();
                    super::context::push(&mut offsets, (out.len(), cursor), budget)?;
                }
                out.push(ch);
            }
            Step::Inert => {}
        }
        cursor += ch.len_utf8();
    }
    *block = scanner.block;
    budget
        .charge(cursor - charged)
        .map_err(ParseFailure::terminal)?;
    Ok((out, offsets))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RenderError, RenderOptions};

    #[test]
    fn should_check_cancellation_during_comment_ownership_before_retaining_text() {
        // given
        let raw = format!("/'{}'/ A", "x".repeat(4096));
        let mut budget = Budget::new(&RenderOptions::default());
        budget.control.cancel_after_checkpoints(18);
        // when
        let result = Ownership::default().classify(&raw, &mut budget);
        // then
        assert!(matches!(
            result,
            Err(ParseFailure {
                kind: super::super::error::FailureKind::Terminal(RenderError::Cancelled(_)),
                ..
            })
        ));
    }
}
