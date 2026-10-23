//! Generated sequence syntax and typed event construction.
use super::{
    BodySource, context,
    error::{ParseFailure, generated_failure},
    ownership::Row,
};
use crate::{
    RenderError,
    plantuml::{
        arrow::{Arrow, End},
        budget::{Budget, bound},
        scene::Dash,
        sequence::{Builder, Declaration, FrameKind, Lifecycle, Name, ParticipantKind, Sequence},
    },
};
use std::ops::Range;

#[derive(Clone, Debug)]
pub(in crate::plantuml) enum Token {
    Participant(ParticipantKind),
    Lifecycle(Lifecycle),
    Frame(FrameKind),
    Note,
    EndNote,
    End,
    Else,
    As,
    Left,
    Right,
    Of,
    Over,
    Name(Name),
    Label(String),
    Caption(String),
    Raw(String),
    Colon,
    ArrowStart,
    ArrowEnd,
    Less,
    Greater,
    Minus,
    Dot,
    Unknown,
    Newline,
}
type Spanned = (usize, Token, usize);

enum RowMode {
    Command,
    Note,
}

enum Suffix {
    Header,
    Label,
}

pub(in crate::plantuml) fn declare(left: Name, right: Name) -> Declaration {
    if right.quoted {
        return Declaration {
            name: left.text,
            label: right.text,
        };
    }
    Declaration {
        name: right.text,
        label: left.text,
    }
}

pub(in crate::plantuml) fn make_arrow(
    parts: (End, Dash, End),
    span: Range<usize>,
) -> Result<(Arrow, bool), ParseFailure> {
    let (start, dash, end) = parts;
    if start != End::None && end != End::None {
        return Err(ParseFailure::unsupported(
            span,
            "bidirectional sequence arrow",
        ));
    }
    let reverse = start != End::None;
    Ok((
        Arrow {
            start: End::None,
            end: if reverse { start } else { end },
            dash,
        },
        reverse,
    ))
}

pub(in crate::plantuml) fn parse(
    body: BodySource<'_>,
    budget: &mut Budget,
) -> Result<Sequence, RenderError> {
    parse_body(&body, budget).map_err(|error| error.into_document(body.document, budget))
}

fn parse_body(body: &BodySource<'_>, budget: &mut Budget) -> Result<Sequence, ParseFailure> {
    bound(
        "sequence source bytes",
        body.range.len(),
        budget.limits.source_bytes,
    )
    .map_err(ParseFailure::terminal)?;
    budget
        .charge(std::mem::size_of::<super::ownership::Ownership>())
        .map_err(ParseFailure::terminal)?;
    let mut lexer = Lexer {
        tokens: Vec::new(),
        literal_bytes: 0,
        budget,
    };
    let mut ownership = body.ownership;
    let mut base = body.range.start;
    let mut note = false;
    for physical in body.document.text[body.range.clone()].split_inclusive('\n') {
        let raw = physical
            .strip_suffix('\n')
            .map_or(physical, |row| row.strip_suffix('\r').unwrap_or(row));
        let row = ownership.classify(raw, lexer.budget)?;
        ownership.advance(&row, &row.text, lexer.budget)?;
        if note {
            if row.text.trim() == "end note" {
                lexer.push(Token::EndNote, row.span(base, 0..row.text.len()))?;
                lexer.push(Token::Newline, base + raw.len()..base + physical.len())?;
                note = false;
            } else {
                lexer.note_row(&row, base)?;
            }
        } else if row.text.trim().is_empty() {
            lexer.budget.charge(1).map_err(ParseFailure::terminal)?;
        } else {
            note = matches!(lexer.row(&row, base)?, RowMode::Note);
        }
        base += physical.len();
    }
    ownership.finish(base)?;
    let tokens = lexer.tokens;
    let retained = admit((&tokens, tokens.capacity()), body.adapters, lexer.budget)?;
    let iterator = context::Checked::new(tokens, lexer.budget.control.clone());
    let mut builder = Builder::new(lexer.budget, retained);
    super::generated_sequence::SequenceParser::new()
        .parse(&mut builder, iterator)
        .map_err(|error| {
            generated_failure(
                error,
                body.range.end,
                "invalid or incomplete sequence syntax",
            )
        })?;
    builder.finish(body.range.end..body.range.end)
}

struct Lexer<'b> {
    tokens: Vec<Spanned>,
    literal_bytes: usize,
    budget: &'b mut Budget,
}

impl Lexer<'_> {
    fn push(&mut self, token: Token, span: Range<usize>) -> Result<(), ParseFailure> {
        context::push(&mut self.tokens, (span.start, token, span.end), self.budget)
    }

    fn copy(&mut self, value: &str, _span: Range<usize>) -> Result<String, ParseFailure> {
        bound("label bytes", value.len(), 16 * 1024).map_err(ParseFailure::terminal)?;
        self.literal_bytes = self.literal_bytes.saturating_add(value.len());
        bound(
            "sequence syntax literal bytes",
            self.literal_bytes,
            self.budget.limits.source_bytes.saturating_mul(2),
        )
        .map_err(ParseFailure::terminal)?;
        for chunk in value.as_bytes().chunks(256) {
            self.budget
                .charge(chunk.len())
                .map_err(ParseFailure::terminal)?;
        }
        Ok(value.to_owned())
    }

    fn note_row(&mut self, row: &Row, base: usize) -> Result<(), ParseFailure> {
        self.budget.charge(1).map_err(ParseFailure::terminal)?;
        let text = match (row.literal, row.opening_action) {
            (true, true) => row.text.trim_start(),
            (true, false) => row.text.as_str(),
            _ => row.text.trim(),
        };
        if matches!((row.literal, text.is_empty()), (false, true)) {
            return Ok(());
        }
        let start = match (row.literal, row.opening_action) {
            (true, false) => 0,
            _ => row.text.len() - row.text.trim_start().len(),
        };
        let span = row.span(base, start..start + text.len());
        let value = self.copy(text, span.clone())?;
        self.push(Token::Raw(value), span)
    }

    fn row(&mut self, row: &Row, base: usize) -> Result<RowMode, ParseFailure> {
        let start = row.text.len() - row.text.trim_start().len();
        let (head, first_end) = self.first_primitive(row, (base, start))?;
        let command = match command_token(&head.text) {
            Some(Token::Frame(kind)) => {
                Some(Token::Frame(self.frame_kind(kind, row, (base, start))?))
            }
            command => command,
        };
        let tail = matches!(command, Some(Token::Frame(_) | Token::Else));
        let is_else = matches!(command, Some(Token::Else));
        let mode = match &command {
            Some(Token::Note) => RowMode::Note,
            _ => RowMode::Command,
        };
        let cursor = start + first_end;
        let caption_start = match &command {
            Some(Token::Frame(kind)) => start + kind.name().len(),
            Some(Token::Else) => start + 4,
            _ => cursor,
        };
        let span = row.span(base, start..caption_start.max(cursor));
        if let Some(command) = command {
            self.push(command, span)?;
        } else if head.quoted
            || head
                .text
                .chars()
                .all(|ch| ch.is_alphanumeric() || matches!(ch, '_' | '.' | '@'))
        {
            self.push(Token::Name(head), span)?;
        } else {
            return Err(ParseFailure::unsupported(
                span,
                "sequence identifier or attached syntax",
            ));
        }
        let suffix = if tail {
            let raw = &row.text[caption_start..row.text.trim_end().len()];
            let raw = if is_else {
                raw.trim()
            } else {
                raw.strip_prefix(' ').unwrap_or(raw)
            };
            let raw_start = if is_else {
                caption_start + row.text[caption_start..].len()
                    - row.text[caption_start..].trim_start().len()
            } else {
                caption_start + usize::from(row.text[caption_start..].starts_with(' '))
            };
            let value = self.copy(raw, row.span(base, raw_start..raw_start + raw.len()))?;
            self.push(
                Token::Caption(value),
                row.span(base, raw_start..row.text.trim_end().len()),
            )?;
            Suffix::Header
        } else {
            self.primitives(row, (base, cursor))?
        };
        self.push(
            Token::Newline,
            row.span(base, row.text.len()..row.text.len()),
        )?;
        Ok(match (mode, suffix) {
            (RowMode::Note, Suffix::Header) => RowMode::Note,
            _ => RowMode::Command,
        })
    }

    fn primitives(&mut self, row: &Row, location: (usize, usize)) -> Result<Suffix, ParseFailure> {
        let (base, mut cursor) = location;
        while cursor < row.text.len() {
            let rest = &row.text[cursor..];
            let ch = rest.chars().next().unwrap();
            self.budget
                .charge(ch.len_utf8())
                .map_err(ParseFailure::terminal)?;
            if ch.is_whitespace() {
                cursor += ch.len_utf8();
                continue;
            }
            if ch == ':' && !rest.starts_with("::") && !row.text[..cursor].ends_with(':') {
                self.push(Token::Colon, row.span(base, cursor..cursor + 1))?;
                let tail = rest[1..].trim();
                let start = cursor + 1 + rest[1..].len() - rest[1..].trim_start().len();
                let span = row.span(base, start..(start + tail.len()));
                let value = self.copy(tail, span.clone())?;
                self.push(Token::Label(value), span)?;
                return Ok(Suffix::Label);
            }
            if ch == '"' {
                let (name, bytes) = self.quoted(row, (base, cursor))?;
                self.push(Token::Name(name), row.span(base, cursor..cursor + bytes))?;
                cursor += bytes;
                continue;
            }
            if operator_prefix(rest)
                || (ch == '-' && rest[1..].chars().next().is_none_or(char::is_whitespace))
            {
                let bytes = self.scan_end(rest, |ch| {
                    !matches!(ch, '-' | '.' | '<' | '>' | '|' | 'o' | '*')
                })?;
                self.push(Token::ArrowStart, row.span(base, cursor..cursor))?;
                for (index, ch) in rest[..bytes].char_indices() {
                    let token = match ch {
                        '-' => Token::Minus,
                        '.' => Token::Dot,
                        '<' => Token::Less,
                        '>' => Token::Greater,
                        _ => Token::Unknown,
                    };
                    self.budget.charge(1).map_err(ParseFailure::terminal)?;
                    self.push(token, row.span(base, cursor + index..cursor + index + 1))?;
                }
                self.push(
                    Token::ArrowEnd,
                    row.span(base, cursor + bytes..cursor + bytes),
                )?;
                cursor += bytes;
                continue;
            }
            let bytes = self.scan_primitive_end(rest)?;
            let word = &rest[..bytes];
            let span = row.span(base, cursor..cursor + bytes);
            let token = match word {
                "as" => Token::As,
                "left" => Token::Left,
                "right" => Token::Right,
                "of" => Token::Of,
                "over" => Token::Over,
                _ if word
                    .chars()
                    .all(|ch| ch.is_alphanumeric() || matches!(ch, '_' | '.' | '@')) =>
                {
                    Token::Name(Name {
                        text: self.copy(word, span.clone())?,
                        quoted: false,
                    })
                }
                _ => {
                    return Err(ParseFailure::unsupported(
                        span,
                        "sequence identifier or attached syntax",
                    ));
                }
            };
            self.push(token, span)?;
            cursor += bytes;
        }
        Ok(Suffix::Header)
    }

    fn frame_kind(
        &mut self,
        kind: FrameKind,
        row: &Row,
        location: (usize, usize),
    ) -> Result<FrameKind, ParseFailure> {
        let (base, start) = location;
        let text = &row.text[start..row.text.trim_end().len()];
        let bytes = self.scan_end(text, |ch| ch == ' ')?;
        let spelling = &text[..bytes];
        if spelling == kind.name() {
            return Ok(kind);
        }
        let value = self.copy(spelling, row.span(base, start..start + bytes))?;
        Ok(FrameKind::Literal(value))
    }

    fn first_primitive(
        &mut self,
        row: &Row,
        location: (usize, usize),
    ) -> Result<(Name, usize), ParseFailure> {
        let (base, cursor) = location;
        let rest = &row.text[cursor..];
        self.budget
            .charge(rest.chars().next().unwrap().len_utf8())
            .map_err(ParseFailure::terminal)?;
        if rest.starts_with('"') {
            return self.quoted(row, location);
        }
        let bytes = self.scan_primitive_end(rest)?;
        let text = self.copy(&rest[..bytes], row.span(base, cursor..cursor + bytes))?;
        Ok((
            Name {
                text,
                quoted: false,
            },
            bytes,
        ))
    }

    fn scan_primitive_end(&mut self, text: &str) -> Result<usize, ParseFailure> {
        if operator_prefix(text) {
            return self.scan_end(text, |ch| {
                !matches!(ch, '-' | '.' | '<' | '>' | '|' | 'o' | '*')
            });
        }
        let first = text.chars().next().unwrap();
        let mut end = first.len_utf8();
        if matches!(first, '{' | '}' | ':') {
            return Ok(end);
        }
        for (index, ch) in text.char_indices().skip(1) {
            if ch.is_whitespace()
                || matches!(ch, '{' | '}' | ':' | '"')
                || operator_prefix(&text[index..])
            {
                return Ok(index);
            }
            self.budget
                .charge(ch.len_utf8())
                .map_err(ParseFailure::terminal)?;
            end = index + ch.len_utf8();
        }
        Ok(end)
    }

    fn scan_end(&mut self, text: &str, stop: impl Fn(char) -> bool) -> Result<usize, ParseFailure> {
        let mut pending = 0;
        for (index, ch) in text.char_indices() {
            if stop(ch) {
                self.budget
                    .charge(pending)
                    .map_err(ParseFailure::terminal)?;
                return Ok(index);
            }
            pending += ch.len_utf8();
            if pending >= 256 {
                self.budget
                    .charge(pending)
                    .map_err(ParseFailure::terminal)?;
                pending = 0;
            }
        }
        self.budget
            .charge(pending)
            .map_err(ParseFailure::terminal)?;
        Ok(text.len())
    }

    fn quoted(
        &mut self,
        row: &Row,
        location: (usize, usize),
    ) -> Result<(Name, usize), ParseFailure> {
        let (base, cursor) = location;
        let mut value = String::new();
        let mut escaped = false;
        for (index, ch) in row.text[cursor..].char_indices().skip(1) {
            self.budget
                .charge(ch.len_utf8())
                .map_err(ParseFailure::terminal)?;
            bound(
                "label bytes",
                value.len().saturating_add(ch.len_utf8() + 1),
                16 * 1024 + 2,
            )
            .map_err(ParseFailure::terminal)?;
            if value.len().saturating_add(ch.len_utf8() + 1) > value.capacity() {
                self.budget
                    .charge(value.len())
                    .map_err(ParseFailure::terminal)?;
            }
            if escaped {
                match ch {
                    'n' | '\\' | '~' => {
                        value.push('\\');
                        value.push(ch);
                    }
                    '"' => value.push(ch),
                    _ => {
                        return Err(ParseFailure::unsupported(
                            row.span(base, cursor + index..cursor + index + ch.len_utf8()),
                            "quoted sequence escape",
                        ));
                    }
                }
                escaped = false;
            } else {
                match ch {
                    '\\' => escaped = true,
                    '"' => {
                        let text = self.copy(&value, row.span(base, cursor..cursor + index + 1))?;
                        return Ok((Name { text, quoted: true }, index + 1));
                    }
                    _ => value.push(ch),
                }
            }
        }
        Err(ParseFailure::syntax(
            row.span(base, cursor..row.text.len()),
            "unterminated quoted identifier",
        ))
    }
}

fn command_token(first: &str) -> Option<Token> {
    match first {
        "participant" => Some(Token::Participant(ParticipantKind::Participant)),
        "actor" => Some(Token::Participant(ParticipantKind::Actor)),
        "database" => Some(Token::Participant(ParticipantKind::Database)),
        "activate" => Some(Token::Lifecycle(Lifecycle::Activate)),
        "deactivate" => Some(Token::Lifecycle(Lifecycle::Deactivate)),
        "create" => Some(Token::Lifecycle(Lifecycle::Create)),
        "destroy" => Some(Token::Lifecycle(Lifecycle::Destroy)),
        "group" => Some(Token::Frame(FrameKind::Group)),
        "alt" => Some(Token::Frame(FrameKind::Alt)),
        "opt" => Some(Token::Frame(FrameKind::Opt)),
        "loop" => Some(Token::Frame(FrameKind::Loop)),
        "par" => Some(Token::Frame(FrameKind::Par)),
        "break" => Some(Token::Frame(FrameKind::Break)),
        "critical" => Some(Token::Frame(FrameKind::Critical)),
        "else" => Some(Token::Else),
        "end" => Some(Token::End),
        "note" => Some(Token::Note),
        _ => None,
    }
}

fn operator_prefix(rest: &str) -> bool {
    ["--", "->", "..", "<-", "<|", "o-", "o.", "*-", "*.", "<."]
        .iter()
        .any(|prefix| rest.starts_with(prefix))
}

fn admit(
    input: (&[Spanned], usize),
    adapters: usize,
    budget: &mut Budget,
) -> Result<usize, ParseFailure> {
    let (tokens, token_capacity) = input;
    // Actual 72-state/25-terminal tables: at most 2d+8 live states and
    // four reductions between deliveries. Reserve four extra states.
    let mut frames = 0usize;
    let mut literals = 0usize;
    for chunk in tokens.chunks(256) {
        budget.charge(chunk.len()).map_err(ParseFailure::terminal)?;
        for (_, token, _) in chunk {
            match token {
                Token::Frame(kind) => {
                    frames += 1;
                    if let FrameKind::Literal(value) = kind {
                        literals = literals.saturating_add(value.capacity());
                    }
                }
                Token::Name(name) => literals = literals.saturating_add(name.text.capacity()),
                Token::Label(value) | Token::Caption(value) | Token::Raw(value) => {
                    literals = literals.saturating_add(value.capacity())
                }
                _ => {}
            }
        }
    }
    let depth = frames.min(budget.limits.nesting_depth);
    let live = depth.saturating_mul(2).saturating_add(12);
    let capacity = live.next_power_of_two().max(8);
    let growing = capacity + capacity / 2;
    let diagnostic = 3 * (live + 1);
    // Token, Name, Declaration and Anchor are the actual largest generated
    // symbol candidates. Sum rather than assume a compiler enum layout.
    let symbol = std::mem::size_of::<Token>()
        + std::mem::size_of::<Name>()
        + std::mem::size_of::<Declaration>()
        + std::mem::size_of::<crate::plantuml::sequence::Anchor>()
        + 4 * std::mem::size_of::<usize>();
    let retained = token_capacity
        .saturating_mul(std::mem::size_of::<Spanned>())
        .saturating_add(literals)
        .saturating_add(adapters)
        .saturating_add(growing.saturating_mul(symbol + 1))
        .saturating_add(diagnostic + 48 * std::mem::size_of::<String>() + 25 * 32)
        .saturating_add(std::mem::size_of::<Builder<'_>>())
        // All semantic text copies and geometric String growth fit twice the
        // existing source*8 model-text quota, independently of model items.
        .saturating_add(budget.limits.source_bytes.saturating_mul(16));
    bound(
        "sequence parser retained bytes",
        retained,
        budget.limits.source_bytes.saturating_mul(256),
    )
    .map_err(ParseFailure::terminal)?;
    let slots = growing.saturating_mul(2).saturating_add(diagnostic + 48);
    bound(
        "sequence parser stack slots",
        slots,
        budget.limits.layout_work,
    )
    .map_err(ParseFailure::terminal)?;
    budget
        .charge(
            tokens
                .len()
                .saturating_mul(5)
                .saturating_add(4 + slots + 25 * (live + 4)),
        )
        .map_err(ParseFailure::terminal)?;
    Ok(retained)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RenderOptions;

    fn prepared(source: &str, budget: &mut Budget) -> crate::plantuml::origin::Document {
        crate::plantuml::preprocess::prepare(source, budget).unwrap()
    }

    fn model(source: &str, options: &RenderOptions) -> Result<Sequence, RenderError> {
        let mut budget = Budget::new(options);
        let document = crate::plantuml::preprocess::prepare(source, &mut budget)?;
        let mut parsed = super::super::document::parse(&document, &mut budget)?;
        super::super::preamble::parse(&mut parsed, &mut budget)?;
        parse(parsed.body, &mut budget)
    }

    #[test]
    fn should_recognize_adjacent_quoted_sequence_command_primitives() {
        // given
        for body in [
            "participant\"A\"\nA -> B : hello",
            "participant A\nactivate\"A\"\nA -> B : hello\ndeactivate\"A\"",
            "\"participant\"\"Alpha\" as A\n\"activate\"\"A\"\nA -> B : hello\n\"deactivate\"\"A\"",
        ] {
            // when
            let model = model(
                &format!("@startuml\n{body}\n@enduml"),
                &RenderOptions::default(),
            )
            .unwrap();
            // then
            assert_eq!(
                model
                    .participants
                    .iter()
                    .map(|p| p.name.as_str())
                    .collect::<Vec<_>>(),
                ["A", "B"],
                "{body}"
            );
            assert!(format!("{model:?}").contains("from: 0, to: 1"), "{body}");
        }
    }

    #[test]
    fn should_reject_reserved_first_primitives_as_sequence_message_endpoints() {
        // given
        for first in [
            "participant",
            "actor",
            "database",
            "activate",
            "deactivate",
            "create",
            "destroy",
            "note",
            "end",
        ] {
            for endpoint in [
                format!("{first}->B : hello"),
                format!("\"{first}\" -> B : hello"),
            ] {
                // when
                let result = model(
                    &format!("@startuml\n{endpoint}\n@enduml"),
                    &RenderOptions::default(),
                );
                // then
                assert!(
                    matches!(
                        result,
                        Err(RenderError::InvalidSource(_) | RenderError::UnsupportedFeature(_))
                    ),
                    "accepted {endpoint}: {result:?}"
                );
            }
        }
    }

    #[test]
    fn should_preserve_primitive_command_caption_and_note_ownership() {
        // given
        for (body, expected) in [
            (
                "participant A\n\"alt\" first\nA -> A : body\nend",
                "kind: \"\\\"alt\\\"\"",
            ),
            (
                "participant A\ngroup\"joined\" title\nA -> A : body\nend",
                "kind: \"group\\\"joined\\\"\"",
            ),
            (
                "participant A\nalt first\n\"else\" later\nA -> A : body\nend",
                "Else(\"e\\\" later\")",
            ),
            (
                "participant A\n\"note\" over A\n  keep\n\n' ignored\nend note\nA -> A : body",
                "label: \"keep\"",
            ),
        ] {
            // when
            let model = model(
                &format!("@startuml\n{body}\n@enduml"),
                &RenderOptions::default(),
            )
            .unwrap();
            // then
            assert!(format!("{model:?}").contains(expected), "{body}: {model:?}");
        }
    }

    #[test]
    fn should_reject_else_under_a_noncanonical_frame_spelling() {
        // given
        for head in ["\"alt\" first", "alt\"joined\" first"] {
            // when
            let result = model(
                &format!(
                    "@startuml\nparticipant A\n{head}\nelse later\nA -> A : body\nend\n@enduml"
                ),
                &RenderOptions::default(),
            );
            // then
            assert!(
                matches!(result, Err(RenderError::InvalidSource(_))),
                "accepted {head}: {result:?}"
            );
        }
    }

    #[test]
    fn should_build_generated_sequence_aliases_messages_frames_notes_and_lifecycle() {
        // given
        let source = "@startuml\nparticipant \"Alpha\" as A\nactor B as \"Beta\"\nalt first\nA <- B : reply\nloop nested\nactivate A\nA -> A : self\ndeactivate A\nend\nelse second\nnote right of A\n  raw /'\n\n' literal\nend note\nend\ncreate C\nA --> C : created\ndestroy C\n@enduml";
        // when
        let model = model(source, &RenderOptions::default()).unwrap();
        // then
        assert_eq!(
            model
                .participants
                .iter()
                .map(|p| (p.name.as_str(), p.label.as_str()))
                .collect::<Vec<_>>(),
            [("A", "Alpha"), ("B", "Beta"), ("C", "C")]
        );
        let events = format!("{model:?}");
        assert!(events.contains("from: 1, to: 0"), "{events}");
        assert!(events.contains("from: 0, to: 0"), "{events}");
        assert!(events.contains("  raw /'\\n\\n' literal"), "{events}");
        assert!(events.contains("Create(2)"), "{events}");
        assert!(events.contains("Destroy(2)"), "{events}");
    }

    #[test]
    fn should_continue_shared_comment_ownership_at_the_preamble_boundary() {
        // given
        let source = "@startuml\ntitle Ω /' comment\n'/  A.part -> @B : title: literal\n@enduml";
        // when
        let model = model(source, &RenderOptions::default()).unwrap();
        // then
        assert_eq!(model.participants[0].name, "A.part");
        assert_eq!(model.participants[1].name, "@B");
        assert!(format!("{model:?}").contains("title: literal"));
    }

    #[test]
    fn should_map_sequence_failure_once_at_a_frozen_expansion_use() {
        // given
        let source = "\u{feff}@startuml\r\n!define BAD end\r\n!define UNUSED inert\r\nA -> B\r\nBAD\r\n@enduml";
        let options = RenderOptions {
            language: crate::Language::PlantUml,
            ..RenderOptions::default()
        };
        // when
        let error = crate::render(source, &options).unwrap_err().to_string();
        // then
        assert!(
            error.contains("used at line 5, column 1, defined at line 2, column 1"),
            "{error}"
        );
        assert_eq!(error.matches("expansion \"BAD\"").count(), 1, "{error}");
        assert!(!error.contains("UNUSED"), "{error}");
    }

    #[test]
    fn should_preserve_raw_expanded_note_rows_crlf_and_original_comment_gaps() {
        // given
        let source = "\u{feff}@startuml\r\n!define OPEN note over A\r\n!define CLOSE end note\r\n!define note body\r\n!define X Ready\r\ntitle Ω /' comment\r\n'/  participant A\r\nOPEN\r\nend note\r\n  ' X\r\n\r\nCLOSE\r\nA.part -> @B : X: label\r\n@enduml";
        // when
        let model = model(source, &RenderOptions::default()).unwrap();
        // then
        let events = format!("{model:?}");
        assert!(events.contains("end body\\n  ' Ready\\n"), "{events}");
        assert!(events.contains("Ready: label"), "{events}");
        assert_eq!(
            model
                .participants
                .iter()
                .map(|p| p.name.as_str())
                .collect::<Vec<_>>(),
            ["A", "A.part", "@B"]
        );
    }

    #[test]
    fn should_locate_generated_sequence_suffix_after_an_inline_utf8_comment() {
        // given
        let source = "@startuml\ntitle Ω /' comment\n'/  participant A : suffix\n@enduml";
        // when
        let error = model(source, &RenderOptions::default())
            .unwrap_err()
            .to_string();
        // then
        assert!(error.contains("at line 3, column 19"), "{error}");
    }

    #[test]
    fn should_bound_token_retention_before_appending_a_fifth_token() {
        // given
        let mut budget = Budget::new(&RenderOptions::default());
        budget.limits.source_bytes = 4;
        let mut lexer = Lexer {
            tokens: Vec::new(),
            literal_bytes: 0,
            budget: &mut budget,
        };
        for _ in 0..4 {
            lexer.push(Token::Newline, 0..0).unwrap();
        }
        // when
        let error = lexer.push(Token::Newline, 0..0).unwrap_err();
        // then
        assert!(format!("{error:?}").contains("buffered tokens"));
        assert_eq!(lexer.tokens.len(), 4);
        assert_eq!(lexer.tokens.capacity(), 4);
    }

    #[test]
    fn should_reject_direct_generated_suffix_eof_and_cancel_delivery_and_actions() {
        // given
        let declaration = || {
            vec![
                (0, Token::Participant(ParticipantKind::Participant), 11),
                (
                    12,
                    Token::Name(Name {
                        text: "A".into(),
                        quoted: false,
                    }),
                    13,
                ),
                (13, Token::Newline, 14),
            ]
        };
        for stream in [
            vec![
                (0, Token::Frame(FrameKind::Alt), 3),
                (4, Token::Caption("open".into()), 8),
                (8, Token::Newline, 9),
            ],
            vec![(0, Token::End, 3), (3, Token::Newline, 4)],
            {
                let mut tokens = declaration();
                tokens.insert(2, (13, Token::Colon, 14));
                tokens
            },
        ] {
            let mut budget = Budget::new(&RenderOptions::default());
            let retained = admit((&stream, stream.capacity()), 0, &mut budget).unwrap();
            let iterator = context::Checked::new(stream, budget.control.clone());
            let mut builder = Builder::new(&mut budget, retained);
            // when
            let result = super::super::generated_sequence::SequenceParser::new()
                .parse(&mut builder, iterator);
            // then
            assert!(result.is_err());
        }
        for checkpoint in 1..=7 {
            // given
            let stream = declaration();
            let mut budget = Budget::new(&RenderOptions::default());
            let retained = admit((&stream, stream.capacity()), 0, &mut budget).unwrap();
            budget.control.cancel_after_checkpoints(checkpoint);
            let iterator = context::Checked::new(stream, budget.control.clone());
            let mut builder = Builder::new(&mut budget, retained);
            // when
            let result = super::super::generated_sequence::SequenceParser::new()
                .parse(&mut builder, iterator);
            // then
            assert!(
                matches!(
                    result,
                    Err(lalrpop_util::ParseError::User {
                        error: ParseFailure {
                            kind: super::super::error::FailureKind::Terminal(
                                RenderError::Cancelled(_)
                            ),
                            ..
                        }
                    })
                ),
                "checkpoint {checkpoint}: {result:?}"
            );
        }
    }

    #[test]
    fn should_reject_generated_sequence_suffix_and_unclosed_structures() {
        // given
        for body in [
            "participant A : suffix",
            "A -> B extra : suffix",
            "alt open\nA -> B",
            "note over A\nunterminated",
            "activate A",
            "A -> B\nend",
        ] {
            // when
            let error = model(
                &format!("@startuml\n{body}\n@enduml"),
                &RenderOptions::default(),
            )
            .unwrap_err();
            // then
            assert!(
                matches!(
                    error,
                    RenderError::InvalidSource(_) | RenderError::UnsupportedFeature(_)
                ),
                "{body}: {error}"
            );
        }
    }

    #[test]
    fn should_bound_generated_sequence_tokens_depth_work_and_midparse_cancellation() {
        // given
        for resource in ["token", "depth", "work", "cancel"] {
            let mut budget = Budget::new(&RenderOptions::default());
            let document = prepared(
                "@startuml\nalt outer\nloop inner\nA -> B : value\nend\nend\n@enduml",
                &mut budget,
            );
            let mut parsed = super::super::document::parse(&document, &mut budget).unwrap();
            super::super::preamble::parse(&mut parsed, &mut budget).unwrap();
            match resource {
                "token" => budget.limits.source_bytes = 4,
                "depth" => budget.limits.nesting_depth = 1,
                "work" => budget.limits.layout_work = budget.work + 1,
                _ => budget.control.cancel_after_checkpoints(40),
            }
            // when
            let error = parse(parsed.body, &mut budget).unwrap_err();
            // then
            if resource == "cancel" {
                assert!(matches!(error, RenderError::Cancelled(_)), "{error}");
            } else {
                assert!(
                    matches!(error, RenderError::Backend(_))
                        && error.to_string().contains("exceeds limit"),
                    "{resource}: {error}"
                );
            }
        }
    }
}
