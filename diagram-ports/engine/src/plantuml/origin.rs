//! Compact authored-source locations; no expanded provenance trees are retained.
use super::budget::{Budget, bound, invalid};
use crate::RenderError;
use std::ops::Range;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SourceSpan {
    pub start_byte: usize,
    pub end_byte: usize,
    pub line: usize,
    /// One-based UTF-8 byte column in the authored physical line.
    pub column: usize,
}

impl SourceSpan {
    pub fn slice(self, range: Range<usize>) -> Self {
        Self {
            start_byte: self.start_byte + range.start,
            end_byte: self.start_byte + range.end,
            column: self.column + range.start,
            ..self
        }
    }
}

#[derive(Debug)]
pub(super) enum Origin {
    Direct(SourceSpan),
    Expansion {
        use_span: SourceSpan,
        definition_id: usize,
    },
}

#[derive(Debug)]
pub(super) struct Segment {
    pub output: Range<usize>,
    pub origin: Origin,
}

#[derive(Debug)]
pub(super) struct Definition {
    pub name: String,
    pub value: String,
    pub span: SourceSpan,
    pub prior: Vec<usize>,
    pub depth: usize,
}

#[derive(Default)]
pub(super) struct Document {
    pub text: String,
    pub segments: Vec<Segment>,
    pub definitions: Vec<Definition>,
}

pub(super) static EMPTY: Document = Document {
    text: String::new(),
    segments: Vec::new(),
    definitions: Vec::new(),
};

impl Document {
    pub fn map_failure(
        &self,
        failure: super::parser::error::ParseFailure,
        budget: &mut Budget,
    ) -> RenderError {
        use super::parser::error::FailureKind;
        let (detail, unsupported) = match failure.kind {
            FailureKind::Terminal(error) => return error,
            FailureKind::Syntax(detail) => (detail, false),
            FailureKind::Unsupported(detail) => (detail, true),
        };
        if let Err(error) = budget.charge(self.text.len() + self.segments.len() * 9) {
            return error;
        }
        let start = failure.span.start.min(self.text.len());
        let end = failure.span.end.min(self.text.len()).max(start);
        let segment = self
            .segments
            .iter()
            .find(|segment| segment.output.contains(&start))
            .or_else(|| {
                self.segments
                    .last()
                    .filter(|segment| segment.output.end == start)
            });
        let (line, column) = match segment.map(|segment| (&segment.origin, segment.output.start)) {
            Some((Origin::Expansion { use_span, .. }, _)) => (use_span.line, use_span.column),
            Some((Origin::Direct(span), base)) => {
                let prefix = &self.text[base..start];
                let lines = prefix.bytes().filter(|&byte| byte == b'\n').count();
                (
                    span.line + lines,
                    prefix
                        .rfind('\n')
                        .map_or(span.column + prefix.len(), |last| prefix.len() - last),
                )
            }
            None => (1, 1),
        };
        let mut message = format!(
            "PlantUML at line {line}, column {column}: {detail} (expanded bytes {start}..{end})"
        );
        self.describe_candidate(&(start..end.max(start.saturating_add(1))), &mut message);
        if let Some(Segment {
            origin: Origin::Expansion { definition_id, .. },
            ..
        }) = segment
        {
            message.push_str(&self.describe_lineage(*definition_id));
        }
        if unsupported {
            RenderError::UnsupportedFeature(message)
        } else {
            RenderError::InvalidSource(message)
        }
    }

    pub fn check_preamble(
        &self,
        lines: &[super::source::Line],
        start: usize,
        budget: &mut Budget,
    ) -> Result<(), RenderError> {
        let first_body = lines[start].number;
        for definition in &self.definitions {
            budget.charge(1)?;
            if definition.span.line >= first_body {
                return Err(invalid(
                    definition.span.line,
                    format!(
                        "!define definition {:?} must precede the first family statement at line {first_body}",
                        definition.name
                    ),
                ));
            }
        }
        let mut definitions = self.definitions.iter().peekable();
        let mut sheet = false;
        for line in &lines[..start] {
            budget.charge(line.text.len() + 1)?;
            while let Some(definition) = definitions
                .peek()
                .filter(|definition| definition.span.line < line.number)
            {
                budget.charge(1)?;
                if sheet {
                    return Err(invalid(
                        definition.span.line,
                        "!define definition inside an expanded stylesheet",
                    ));
                }
                definitions.next();
            }
            if line.text.trim().eq_ignore_ascii_case("<style>") {
                sheet = true;
            }
            if line.text.trim().eq_ignore_ascii_case("</style>") {
                sheet = false;
            }
        }
        Ok(())
    }

    pub fn map_error(&self, error: RenderError, budget: &mut Budget) -> RenderError {
        if matches!(
            error,
            RenderError::InvalidSource(_) | RenderError::UnsupportedFeature(_)
        ) && let Err(error) = budget.charge(self.segments.len())
        {
            return error;
        }
        match error {
            RenderError::InvalidSource(message) => {
                RenderError::InvalidSource(self.describe(message))
            }
            RenderError::UnsupportedFeature(message) => {
                RenderError::UnsupportedFeature(self.describe(message))
            }
            other => other,
        }
    }

    pub fn label_error(&self, text: &str, error: RenderError, budget: &mut Budget) -> RenderError {
        if text.is_empty() {
            return error;
        }
        if let Err(error) = budget.charge(self.text.len() * 9 + self.segments.len() * 8) {
            return error;
        }
        // Family models do not yet carry label spans. Exact complete-label
        // occurrences supply bounded candidates, not a guessed unique location.
        let mut ranges = Vec::new();
        let mut count = 0;
        for (start, _) in self.text.match_indices(text) {
            count += 1;
            if ranges.len() < 8 {
                ranges.push(start..start + text.len());
            }
        }
        let mut detail = if count > 1 {
            "; ambiguous textual candidates; candidate expansion uses".to_owned()
        } else {
            String::new()
        };
        for range in &ranges {
            self.describe_candidate(range, &mut detail);
        }
        if count > ranges.len() {
            detail.push_str(&format!(
                "; {} textual occurrences omitted",
                count - ranges.len()
            ));
        }
        match error {
            RenderError::UnsupportedFeature(message) => RenderError::UnsupportedFeature(format!(
                "{}{detail}",
                message.replacen("PlantUML at line", "PlantUML label-local line", 1)
            )),
            RenderError::InvalidSource(message) => RenderError::InvalidSource(format!(
                "{}{detail}",
                message.replacen("PlantUML at line", "PlantUML label-local line", 1)
            )),
            other => other,
        }
    }

    fn describe_candidate(&self, range: &Range<usize>, detail: &mut String) {
        let mut count = 0;
        for segment in self
            .segments
            .iter()
            .filter(|segment| segment.output.start < range.end && segment.output.end > range.start)
        {
            if let Origin::Expansion {
                use_span,
                definition_id,
            } = segment.origin
            {
                count += 1;
                if count <= 8 {
                    detail.push_str(&self.describe_use(use_span, definition_id));
                }
            }
        }
        if count > 8 {
            detail.push_str(&format!("; {} expansion uses omitted", count - 8));
        }
        if count > 0 {
            return;
        }
        if let Some(segment) = self
            .segments
            .iter()
            .find(|segment| segment.output.contains(&range.start))
            && let Origin::Direct(span) = segment.origin
        {
            let prefix = &self.text[segment.output.start..range.start];
            let lines = prefix.bytes().filter(|&byte| byte == b'\n').count();
            let column = prefix
                .rfind('\n')
                .map_or(span.column + prefix.len(), |last| prefix.len() - last);
            detail.push_str(&format!(
                "; direct candidate at line {}, column {column}",
                span.line + lines
            ));
        }
    }

    fn describe(&self, message: String) -> String {
        let Some(rest) = message.strip_prefix("PlantUML at line ") else {
            return message;
        };
        let Some((number, _)) = rest.split_once(", column 1: ") else {
            return message;
        };
        let Ok(line) = number.parse::<usize>() else {
            return message;
        };
        let expansions = self
            .segments
            .iter()
            .filter_map(|segment| match segment.origin {
                Origin::Expansion {
                    use_span,
                    definition_id,
                } if use_span.line == line => Some((use_span, definition_id)),
                _ => None,
            });
        let mut result = message;
        let mut count = 0;
        for (span, id) in expansions {
            count += 1;
            if count <= 8 {
                result.push_str(&self.describe_use(span, id));
                result.push_str(&self.describe_lineage(id));
            }
        }
        if count > 8 {
            result.push_str(&format!("; {} expansion uses omitted", count - 8));
        }
        result
    }

    fn describe_use(&self, span: SourceSpan, id: usize) -> String {
        let definition = &self.definitions[id];
        format!(
            "; expansion {:?} [definition #{id}] used at line {}, column {}, defined at line {}, column {}",
            definition.name, span.line, span.column, definition.span.line, definition.span.column
        )
    }

    fn describe_lineage(&self, id: usize) -> String {
        let definition = &self.definitions[id];
        if definition.prior.is_empty() {
            return String::new();
        }
        // IDs share the original definitions. Only immediate lineage is printed.
        let names: Vec<_> = definition
            .prior
            .iter()
            .take(8)
            .map(|&prior| {
                let ancestor = &self.definitions[prior];
                format!("{} at line {}", ancestor.name, ancestor.span.line)
            })
            .collect();
        format!(
            " (frozen from {}; depth {}; {} prior IDs omitted)",
            names.join(", "),
            definition.depth,
            definition.prior.len().saturating_sub(8)
        )
    }
}

pub(super) struct Storage {
    retained: usize,
    segments: usize,
}

impl Storage {
    pub fn new() -> Self {
        Self {
            retained: 0,
            segments: 0,
        }
    }

    pub fn retain(&mut self, bytes: usize, budget: &mut Budget) -> Result<(), RenderError> {
        let next = self
            .retained
            .checked_add(bytes)
            .ok_or_else(|| invalid(1, "origin retention arithmetic overflow"))?;
        bound(
            "retained expanded source and definition bytes",
            next,
            budget.limits.source_bytes,
        )?;
        budget.text(bytes)?;
        self.retained = next;
        Ok(())
    }

    pub fn segment(&mut self, budget: &mut Budget) -> Result<(), RenderError> {
        let limit = budget
            .limits
            .model_items
            .checked_mul(4)
            .ok_or_else(|| invalid(1, "origin segment quota arithmetic overflow"))?;
        bound("origin segments and lineage IDs", self.segments + 1, limit)?;
        budget.charge(1)?;
        self.segments += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RenderOptions;
    use crate::plantuml::parser::error::ParseFailure;

    #[test]
    fn should_map_only_overlapping_expanded_bytes_and_preserve_utf8_byte_columns() {
        // given
        let mut budget = Budget::new(&RenderOptions::default());
        let document = crate::plantuml::preprocess::prepare(
            "@startuml\r\n!define X Ready\r\n!define UNUSED inert\r\nA -> B : Ω X\r\n@enduml",
            &mut budget,
        )
        .unwrap();
        let expanded = document.text.find("Ready").unwrap();
        let direct = document.text.find('Ω').unwrap();
        // when
        let expansion_error =
            ParseFailure::unsupported(expanded..expanded + 5, "generated failure")
                .into_document(&document, &mut budget)
                .to_string();
        let direct_error = ParseFailure::syntax(direct..direct + 2, "direct failure")
            .into_document(&document, &mut budget)
            .to_string();
        let terminal = ParseFailure::terminal(RenderError::Backend("quota terminal".into()))
            .into_document(&document, &mut budget);
        // then
        assert!(
            expansion_error.contains("used at line 4, column 13, defined at line 2, column 1"),
            "{expansion_error}"
        );
        assert!(!expansion_error.contains("UNUSED"), "{expansion_error}");
        assert!(
            direct_error.contains("at line 4, column 10"),
            "{direct_error}"
        );
        assert!(!direct_error.contains("expansion"), "{direct_error}");
        assert!(matches!(terminal, RenderError::Backend(message) if message == "quota terminal"));
    }
}
