//! Eager literal definitions from PlantUML MIT EaterAffectationDefine/TContext/VariableManager.
//! Copyright 2009-2024 Arnaud Roques; exact source traces/notices in provenance manifest.
use super::{
    budget::{Budget, bound, invalid, unsupported},
    origin::{Definition, Document, Origin, Segment, SourceSpan, Storage},
};
use crate::RenderError;
use std::collections::BTreeMap;

struct Processor<'a> {
    document: Document,
    names: BTreeMap<String, usize>,
    storage: Storage,
    budget: &'a mut Budget,
    source_comment: bool,
    literal_input: bool,
}

pub(super) fn prepare(source: &str, budget: &mut Budget) -> Result<Document, RenderError> {
    let mut processor = Processor {
        document: Document::default(),
        names: BTreeMap::new(),
        storage: Storage::new(),
        budget,
        source_comment: false,
        literal_input: false,
    };
    processor.scan(source)?;
    Ok(processor.document)
}

impl Processor<'_> {
    fn scan(&mut self, source: &str) -> Result<(), RenderError> {
        let mut offset = 0;
        let mut inside = false;
        let mut block = false;
        let mut raw_data = false;
        let mut sheet = false;
        let mut body = super::source::TextBody::None;
        let mut graph = false;
        for (index, physical) in source.split_inclusive('\n').enumerate() {
            // Precharge the bounded lexical/classification passes before their
            // temporary strings or any retained output can be allocated.
            for chunk in physical.as_bytes().chunks(512) {
                self.budget.charge(chunk.len() * 4)?;
            }
            let raw = physical
                .strip_suffix('\n')
                .map_or(physical, |line| line.strip_suffix('\r').unwrap_or(line));
            let span = SourceSpan {
                start_byte: offset,
                end_byte: offset + raw.len(),
                line: index + 1,
                column: 1,
            };
            self.source_comment = block;
            let raw_text = (body != super::source::TextBody::None
                || (inside && !graph && !block && !raw_data))
                && body.consume_raw(raw).is_some();
            self.literal_input = raw_text || raw_data;
            let classified =
                if self.literal_input || (!block && raw.trim_start().starts_with("!define")) {
                    std::borrow::Cow::Borrowed(raw)
                } else {
                    std::borrow::Cow::Owned(super::source::strip_comments(
                        raw,
                        &mut block,
                        self.budget,
                    )?)
                };
            let command = classified.trim().trim_start_matches('\u{feff}');
            if !self.literal_input && command.eq_ignore_ascii_case("<style>") {
                sheet = true;
            }
            if !self.literal_input && command.eq_ignore_ascii_case("</style>") {
                sheet = false;
            }
            if !self.literal_input
                && (command.eq_ignore_ascii_case("@startuml")
                    || matches!(
                        command,
                        "@startmindmap"
                            | "@startwbs"
                            | "@startjson"
                            | "@startyaml"
                            | "@startgantt"
                            | "@startsalt"
                            | "@startnwdiag"
                    ))
            {
                inside = true;
                raw_data = matches!(command, "@startjson" | "@startyaml" | "@startsalt");
                self.direct(physical, span.slice(0..physical.len()))?;
            } else if !raw_text
                && (command.eq_ignore_ascii_case("@enduml")
                    || matches!(
                        command,
                        "@endmindmap"
                            | "@endwbs"
                            | "@endjson"
                            | "@endyaml"
                            | "@endgantt"
                            | "@endsalt"
                            | "@endnwdiag"
                    ))
            {
                inside = false;
                self.direct(physical, span.slice(0..physical.len()))?;
            } else if command.is_empty() {
                self.direct(physical, span.slice(0..physical.len()))?;
            } else if command.starts_with("!define ") || command.starts_with("!define\t") {
                if !inside {
                    return Err(invalid(
                        span.line,
                        "definition must be inside a single wrapper",
                    ));
                }
                if sheet {
                    return Err(unsupported(
                        span.line,
                        "!define definition inside a stylesheet",
                    ));
                }
                raw.trim_start()
                    .strip_prefix("!define")
                    .ok_or_else(|| unsupported(span.line, "definition after inline comment"))?;
                self.define(raw, span)?;
                self.direct(
                    &physical[raw.len()..],
                    span.slice(raw.len()..physical.len()),
                )?;
            } else {
                if command.starts_with('!') {
                    super::source::reject_active_preprocessor(command, span.line)?;
                }
                let start = self.document.text.len();
                self.expand(raw, span, None)?;
                let expanded = self.document.text[start..].trim_start();
                if !raw_text
                    && (expanded.starts_with('!') || expanded.starts_with('@'))
                    && expanded != raw.trim_start()
                {
                    return Err(self.document.map_error(
                        unsupported(span.line, "generated directive or wrapper"),
                        self.budget,
                    ));
                }
                if raw_text {
                    // Current-row ownership precedes normalization; next-row
                    // ownership follows the expanded terminator seen by parsing.
                    self.budget.charge(expanded.len())?;
                    body = body
                        .consume_raw(expanded)
                        .unwrap_or(super::source::TextBody::None);
                } else if !raw_data && inside && !block {
                    self.budget.text(expanded.len())?;
                    let statement = super::source::Line {
                        number: span.line,
                        text: expanded.trim().to_owned(),
                    };
                    graph |= super::source::graph_declaration(&statement);
                    if !graph {
                        body = super::source::text_body(&statement);
                    }
                }
                self.direct(
                    &physical[raw.len()..],
                    span.slice(raw.len()..physical.len()),
                )?;
            }
            offset += physical.len();
        }
        Ok(())
    }

    fn define(&mut self, raw: &str, span: SourceSpan) -> Result<(), RenderError> {
        let declaration = raw
            .trim_start()
            .strip_prefix("!define")
            .unwrap()
            .trim_start_matches([' ', '\t']);
        let end = declaration
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
            .unwrap_or(declaration.len());
        let name = &declaration[..end];
        if name.is_empty() || name.len() > 64 || name.as_bytes()[0].is_ascii_digit() {
            return Err(invalid(
                span.line,
                format!("invalid literal definition name {name:?}"),
            ));
        }
        let tail = &declaration[end..];
        if tail.starts_with('(') {
            return Err(unsupported(span.line, "parameterized definition"));
        }
        match tail.chars().next() {
            Some(' ' | '\t') => {}
            _ => {
                return Err(invalid(
                    span.line,
                    "definition requires a same-line literal",
                ));
            }
        }
        let literal = tail
            .trim_start_matches([' ', '\t'])
            .trim_end_matches([' ', '\t']);
        if literal.is_empty() {
            return Err(invalid(span.line, "definition requires a nonempty literal"));
        }
        bound("definition literal bytes", literal.len(), 16 * 1024)?;
        if literal
            .chars()
            .any(|ch| (ch.is_control() && ch != '\t') || matches!(ch, '\u{fffe}' | '\u{ffff}'))
        {
            return Err(unsupported(
                span.line,
                "literal definition control character",
            ));
        }
        if literal
            .bytes()
            .rev()
            .take_while(|&byte| byte == b'\\')
            .count()
            % 2
            == 1
        {
            return Err(unsupported(span.line, "literal definition continuation"));
        }
        self.budget
            .charge(name.len() * (self.names.len().checked_ilog2().unwrap_or(0) as usize + 1))?;
        if self.names.contains_key(name) {
            return Err(invalid(
                span.line,
                format!("immutable definition {name:?} already exists"),
            ));
        }
        self.budget.item()?;
        self.budget.text(name.len() * 2)?;
        let id = self.document.definitions.len();
        self.document.definitions.push(Definition {
            name: name.to_owned(),
            value: String::new(),
            span,
            prior: Vec::new(),
            depth: 1,
        });
        let start = literal.as_ptr() as usize - raw.as_ptr() as usize;
        self.expand(literal, span.slice(start..start + literal.len()), Some(id))?;
        self.names.insert(name.to_owned(), id);
        Ok(())
    }

    fn expand(
        &mut self,
        raw: &str,
        span: SourceSpan,
        target: Option<usize>,
    ) -> Result<(), RenderError> {
        let mut cursor = 0;
        let mut direct = 0;
        let mut capability = CapabilityScan {
            comment: self.source_comment,
            quoted: false,
            escaped: false,
        };
        while cursor < raw.len() {
            self.budget.charge(1)?;
            let ch = raw[cursor..].chars().next().unwrap();
            if target.is_some() || self.literal_input || capability.allows(&raw[cursor..]) {
                reject_capability(&raw[cursor..], span.line)?;
            }
            let end = word_end(raw, cursor);
            let name = &raw[cursor..end];
            self.budget.charge(
                name.len().max(1) * (self.names.len().checked_ilog2().unwrap_or(0) as usize + 1),
            )?;
            if let Some(&id) = self.names.get(name) {
                check_boundary(raw, cursor..end, span.line)?;
                if raw[..cursor].ends_with("##") || raw[end..].starts_with("##") {
                    return Err(unsupported(span.line, "## constant concatenation"));
                }
                self.append(
                    &raw[direct..cursor],
                    Origin::Direct(span.slice(direct..cursor)),
                    target,
                )?;
                self.insert(id, span.slice(cursor..end), target)?;
                cursor = end;
                direct = end;
            } else if ch == 'n' && cursor > 0 && raw.as_bytes()[cursor - 1] == b'\\' {
                cursor += 1;
            } else {
                cursor = end.max(cursor + ch.len_utf8());
            }
        }
        self.append(
            &raw[direct..],
            Origin::Direct(span.slice(direct..raw.len())),
            target,
        )
    }

    fn insert(
        &mut self,
        id: usize,
        span: SourceSpan,
        target: Option<usize>,
    ) -> Result<(), RenderError> {
        let definition = &self.document.definitions[id];
        let bytes = definition.value.len();
        if let Some(target) = target {
            let depth = definition.depth + 1;
            bound(
                "definition lineage depth",
                depth,
                self.budget.limits.nesting_depth.min(32),
            )?;
            bound(
                "definition literal bytes",
                self.document.definitions[target]
                    .value
                    .len()
                    .saturating_add(bytes),
                16 * 1024,
            )?;
            self.storage.segment(self.budget)?;
            self.storage.retain(bytes, self.budget)?;
            let (prior, current) = self.document.definitions.split_at_mut(target);
            current[0].depth = current[0].depth.max(depth);
            current[0].prior.push(id);
            current[0].value.push_str(&prior[id].value);
        } else {
            self.storage.segment(self.budget)?;
            self.storage.retain(bytes, self.budget)?;
            let start = self.document.text.len();
            self.document.text.push_str(&definition.value);
            self.document.segments.push(Segment {
                output: start..start + bytes,
                origin: Origin::Expansion {
                    use_span: span,
                    definition_id: id,
                },
            });
        }
        Ok(())
    }

    fn append(
        &mut self,
        text: &str,
        origin: Origin,
        target: Option<usize>,
    ) -> Result<(), RenderError> {
        if text.is_empty() {
            return Ok(());
        }
        if let Some(id) = target {
            bound(
                "definition literal bytes",
                self.document.definitions[id]
                    .value
                    .len()
                    .saturating_add(text.len()),
                16 * 1024,
            )?;
            self.storage.retain(text.len(), self.budget)?;
            self.document.definitions[id].value.push_str(text);
        } else {
            self.storage.retain(text.len(), self.budget)?;
            if let Origin::Direct(span) = origin
                && let Some(Segment {
                    output,
                    origin: Origin::Direct(previous),
                }) = self.document.segments.last_mut()
                && previous.end_byte == span.start_byte
            {
                previous.end_byte = span.end_byte;
                output.end += text.len();
                self.document.text.push_str(text);
                return Ok(());
            }
            self.storage.segment(self.budget)?;
            let start = self.document.text.len();
            self.document.text.push_str(text);
            self.document.segments.push(Segment {
                output: start..start + text.len(),
                origin,
            });
        }
        Ok(())
    }

    fn direct(&mut self, text: &str, span: SourceSpan) -> Result<(), RenderError> {
        self.append(text, Origin::Direct(span), None)
    }
}

fn word_end(raw: &str, start: usize) -> usize {
    let length = raw[start..]
        .bytes()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == b'_')
        .count();
    start + length
}

struct CapabilityScan {
    comment: bool,
    quoted: bool,
    escaped: bool,
}

impl CapabilityScan {
    fn allows(&mut self, rest: &str) -> bool {
        if self.comment {
            if rest.starts_with("'/") {
                self.comment = false;
            }
            return false;
        }
        if self.quoted && self.escaped {
            self.escaped = false;
            return true;
        }
        if self.quoted && rest.starts_with('\\') {
            self.escaped = true;
            return true;
        }
        if rest.starts_with('"') {
            self.quoted = !self.quoted;
        }
        if !self.quoted && rest.starts_with("/'") {
            self.comment = true;
            return false;
        }
        true
    }
}

fn check_boundary(
    raw: &str,
    range: std::ops::Range<usize>,
    line: usize,
) -> Result<(), RenderError> {
    let previous = raw[..range.start].chars().next_back();
    let next = raw[range.end..].chars().next();
    if previous.into_iter().chain(next).any(|c| !c.is_ascii()) {
        return Err(unsupported(
            line,
            "non-ASCII constant boundary; separate known constants with ASCII punctuation or whitespace",
        ));
    }
    Ok(())
}

fn reject_capability(rest: &str, line: usize) -> Result<(), RenderError> {
    if rest.starts_with(['%', '$']) {
        let end = word_end(rest, 1);
        if end > 1 && rest[end..].trim_start_matches([' ', '\t']).starts_with('(') {
            return Err(unsupported(
                line,
                &format!("TIM function capability {}", &rest[..end]),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RenderOptions;

    #[test]
    fn should_retain_exact_direct_bytes_and_expansion_use_spans() {
        // given
        let source = "\u{feff}@startuml\r\n!define X Ready\r\n  A -> B : X\r\n@enduml\r\n";
        let mut budget = Budget::new(&RenderOptions::default());
        // when
        let document = prepare(source, &mut budget).unwrap();
        // then
        for segment in &document.segments {
            match segment.origin {
                Origin::Direct(span) => assert_eq!(
                    &document.text[segment.output.clone()],
                    &source[span.start_byte..span.end_byte]
                ),
                Origin::Expansion {
                    use_span,
                    definition_id,
                } => {
                    assert_eq!(&source[use_span.start_byte..use_span.end_byte], "X");
                    assert_eq!((use_span.line, use_span.column, definition_id), (3, 12, 0));
                    assert_eq!(&document.text[segment.output.clone()], "Ready");
                }
            }
        }
    }

    fn processor(budget: &mut Budget) -> Processor<'_> {
        Processor {
            document: Document::default(),
            names: BTreeMap::new(),
            storage: Storage::new(),
            budget,
            source_comment: false,
            literal_input: false,
        }
    }

    #[test]
    fn should_bound_retained_frozen_values_and_output_before_copying() {
        // given
        let mut config = RenderOptions::default();
        config.limits.source_bytes = 121;
        let mut budget = Budget::new(&config);
        let mut processor = processor(&mut budget);
        let input = format!(
            "@startuml\n!define X {}\nA -> B : X X\n@enduml",
            "v".repeat(50)
        );
        // when
        let error = processor.scan(&input).unwrap_err().to_string();
        // then
        assert!(
            error.contains("retained expanded source and definition bytes"),
            "{error}"
        );
        assert_eq!(processor.document.definitions[0].value.len(), 50);
        assert!(processor.document.text.len() <= 71);
        assert!(processor.document.text.ends_with("v "));
    }

    #[test]
    fn should_bound_frozen_literal_size_and_lineage_before_copying() {
        // given
        let mut config = RenderOptions::default();
        config.limits.nesting_depth = 2;
        let mut budget = Budget::new(&config);
        let mut processor = processor(&mut budget);
        // when
        let error = processor
            .scan("@startuml\n!define A value\n!define B A\n!define C B\nA -> B\n@enduml")
            .unwrap_err()
            .to_string();
        // then
        assert!(
            error.contains("definition lineage depth value 3 exceeds limit 2"),
            "{error}"
        );
        assert_eq!(processor.document.definitions[1].prior, [0]);
        assert!(processor.document.definitions[2].value.is_empty());
        assert!(processor.document.definitions[2].prior.is_empty());
    }

    #[test]
    fn should_bound_origin_segments_before_adding_another_expansion() {
        // given
        let mut config = RenderOptions::default();
        config.limits.model_items = 1;
        let mut budget = Budget::new(&config);
        let mut processor = processor(&mut budget);
        // when
        let error = processor
            .scan("@startuml\n!define X value\nA -> B : X X X X\n@enduml")
            .unwrap_err()
            .to_string();
        // then
        assert!(
            error.contains("origin segments and lineage IDs value 5 exceeds limit 4"),
            "{error}"
        );
        assert_eq!(processor.document.segments.len(), 4);
        assert_eq!(processor.document.text.matches("value").count(), 1);
    }

    #[test]
    fn should_charge_raw_scanning_and_observe_cancel_before_retention() {
        // given
        let mut config = RenderOptions::default();
        config.limits.layout_work = 100;
        let mut budget = Budget::new(&config);
        let mut processor = processor(&mut budget);
        // when
        let error = processor.scan(&" ".repeat(2048)).unwrap_err().to_string();
        // then
        assert!(error.contains("work units"), "{error}");
        assert!(processor.document.text.is_empty());
        assert!(processor.document.segments.is_empty());
        // given
        let config = RenderOptions::default();
        config.control.cancel();
        let mut budget = Budget::new(&config);
        let mut processor = self::processor(&mut budget);
        // when
        let error = processor.scan("@startuml\n!define X value\nA -> B : X\n@enduml");
        // then
        assert!(matches!(error, Err(RenderError::Cancelled(_))));
        assert!(processor.document.text.is_empty());
        assert!(processor.document.definitions.is_empty());
    }

    #[test]
    fn should_bound_definition_count_and_authored_or_expanded_literal_bytes() {
        // given
        let mut config = RenderOptions::default();
        config.limits.model_items = 1;
        let mut budget = Budget::new(&config);
        let mut processor = processor(&mut budget);
        // when
        let error = processor
            .scan("@startuml\n!define A one\n!define B two\nA -> B\n@enduml")
            .unwrap_err()
            .to_string();
        // then
        assert!(
            error.contains("model items value 2 exceeds limit 1"),
            "{error}"
        );
        assert_eq!(processor.document.definitions.len(), 1);
        // given
        for input in [
            format!(
                "@startuml\n!define A {}\nA -> B\n@enduml",
                "x".repeat(16_385)
            ),
            format!(
                "@startuml\n!define A {}\n!define B A A\nA -> B\n@enduml",
                "x".repeat(8192)
            ),
        ] {
            let mut budget = Budget::new(&RenderOptions::default());
            let mut processor = self::processor(&mut budget);
            // when
            let error = processor.scan(&input).unwrap_err().to_string();
            // then
            assert!(
                error.contains("definition literal bytes value 16385 exceeds limit 16384"),
                "{error}"
            );
            assert!(
                processor
                    .document
                    .definitions
                    .iter()
                    .all(|definition| definition.value.len() <= 16_384)
            );
        }
    }

    #[test]
    fn should_bound_lineage_to_thirty_two_without_recursive_expansion() {
        // given
        let mut source = "@startuml\n!define V0 frozen\n".to_owned();
        for index in 1..33 {
            source.push_str(&format!("!define V{index} V{}\n", index - 1));
        }
        let mut budget = Budget::new(&RenderOptions::default());
        let mut processor = processor(&mut budget);
        // when
        let error = processor.scan(&source).unwrap_err().to_string();
        // then
        assert!(
            error.contains("definition lineage depth value 33 exceeds limit 32"),
            "{error}"
        );
        assert_eq!(processor.document.definitions[31].value, "frozen");
        assert_eq!(processor.document.definitions[31].prior, [30]);
        assert!(processor.document.definitions[32].value.is_empty());
    }
}
