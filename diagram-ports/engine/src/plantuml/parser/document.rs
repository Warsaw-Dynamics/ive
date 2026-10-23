use super::{
    BodySource, context,
    error::{ParseFailure, generated_failure},
    ownership::{Kind, Ownership},
};
use crate::{
    RenderError,
    plantuml::{
        budget::{Budget, bound},
        origin::Document,
        source::Wrapper,
    },
};
use std::ops::Range;

#[derive(Clone, Copy, Debug)]
pub(in crate::plantuml) enum Token {
    StartUml,
    EndUml,
    StartMindmap,
    EndMindmap,
    StartWbs,
    EndWbs,
    StartJson,
    EndJson,
    StartYaml,
    EndYaml,
    StartGantt,
    EndGantt,
    StartSalt,
    EndSalt,
    StartNwdiag,
    EndNwdiag,
    StartFiles,
    EndFiles,
    StartPacket,
    EndPacket,
    StartBoard,
    EndBoard,
    StartHcl,
    EndHcl,
    StartChen,
    EndChen,
    StartGit,
    EndGit,
    Row,
    Definition,
    SheetStart,
    SheetEnd,
    Comment,
}

pub(in crate::plantuml) struct Bounds {
    pub wrapper: Wrapper,
    pub range: Range<usize>,
}

pub(in crate::plantuml) fn finish(
    wrapper: Wrapper,
    range: Range<usize>,
    count: usize,
    budget: &mut Budget,
) -> Result<Bounds, ParseFailure> {
    budget.charge(1).map_err(ParseFailure::terminal)?;
    if count == 0 {
        return Err(ParseFailure::syntax(
            range,
            "expected a nonempty complete PlantUML block",
        ));
    }
    Ok(Bounds { wrapper, range })
}

pub(in crate::plantuml) struct ParsedDocument<'s> {
    pub wrapper: Wrapper,
    pub body: BodySource<'s>,
    pub preamble_tokens: Vec<super::preamble::Spanned>,
}

pub(in crate::plantuml) type Spanned = (usize, Token, usize);

pub(in crate::plantuml) fn emit(
    kind: Kind,
    span: Range<usize>,
    tokens: &mut Vec<Spanned>,
    budget: &mut Budget,
) -> Result<(), ParseFailure> {
    let token = match kind {
        Kind::Start(Wrapper::Uml) => Token::StartUml,
        Kind::End(Wrapper::Uml) => Token::EndUml,
        Kind::Start(Wrapper::Mindmap) => Token::StartMindmap,
        Kind::End(Wrapper::Mindmap) => Token::EndMindmap,
        Kind::Start(Wrapper::Wbs) => Token::StartWbs,
        Kind::End(Wrapper::Wbs) => Token::EndWbs,
        Kind::Start(Wrapper::Json) => Token::StartJson,
        Kind::End(Wrapper::Json) => Token::EndJson,
        Kind::Start(Wrapper::Yaml) => Token::StartYaml,
        Kind::End(Wrapper::Yaml) => Token::EndYaml,
        Kind::Start(Wrapper::Gantt) => Token::StartGantt,
        Kind::End(Wrapper::Gantt) => Token::EndGantt,
        Kind::Start(Wrapper::Salt) => Token::StartSalt,
        Kind::End(Wrapper::Salt) => Token::EndSalt,
        Kind::Start(Wrapper::Nwdiag) => Token::StartNwdiag,
        Kind::End(Wrapper::Nwdiag) => Token::EndNwdiag,
        Kind::Start(Wrapper::Files) => Token::StartFiles,
        Kind::End(Wrapper::Files) => Token::EndFiles,
        Kind::Start(Wrapper::Packet) => Token::StartPacket,
        Kind::End(Wrapper::Packet) => Token::EndPacket,
        Kind::Start(Wrapper::Board) => Token::StartBoard,
        Kind::End(Wrapper::Board) => Token::EndBoard,
        Kind::Start(Wrapper::Hcl) => Token::StartHcl,
        Kind::End(Wrapper::Hcl) => Token::EndHcl,
        Kind::Start(Wrapper::Chen) => Token::StartChen,
        Kind::End(Wrapper::Chen) => Token::EndChen,
        Kind::Start(Wrapper::Git) => Token::StartGit,
        Kind::End(Wrapper::Git) => Token::EndGit,
        Kind::Empty => return Ok(()),
        Kind::Trivia => Token::Comment,
        Kind::Content => Token::Row,
        Kind::Definition => Token::Definition,
        Kind::SheetStart => Token::SheetStart,
        Kind::SheetEnd => Token::SheetEnd,
    };
    context::push(tokens, (span.start, token, span.end), budget)
}

fn validate_retained(
    tokens: Vec<Spanned>,
    location: (usize, usize),
    budget: &mut Budget,
) -> Result<Bounds, ParseFailure> {
    let (eof, retained) = location;
    context::admit_frontend(
        context::Frontend::Document(retained),
        tokens.len(),
        tokens.capacity(),
        std::mem::size_of::<Spanned>(),
        budget,
    )?;
    let iterator = context::Checked::new(tokens, budget.control.clone());
    super::generated_document::DocumentParser::new().parse(budget, iterator).map_err(|error| {
        if let lalrpop_util::ParseError::UnrecognizedToken { token: (start, Token::Definition, end), ref expected } = error
            && expected.iter().any(|token| token == "\"sheet end\"") {
                return ParseFailure::unsupported(start..end, "!define definition inside a stylesheet");
            }
        generated_failure(error, eof, "expected exactly one complete PlantUML block with paired wrappers; definition must be inside a single wrapper")
    })
}

pub(in crate::plantuml) fn validate(
    tokens: Vec<Spanned>,
    eof: usize,
    budget: &mut Budget,
) -> Result<Bounds, ParseFailure> {
    validate_retained(tokens, (eof, 0), budget)
}

pub(in crate::plantuml) fn parse<'s>(
    document: &'s Document,
    budget: &mut Budget,
) -> Result<ParsedDocument<'s>, RenderError> {
    scan(document, budget).map_err(|error| error.into_document(document, budget))
}

fn scan<'s>(
    document: &'s Document,
    budget: &mut Budget,
) -> Result<ParsedDocument<'s>, ParseFailure> {
    bound(
        "document source bytes",
        document.text.len(),
        budget.limits.source_bytes,
    )
    .map_err(ParseFailure::terminal)?;
    budget.checkpoint().map_err(ParseFailure::terminal)?;
    budget
        .charge(3 * std::mem::size_of::<Ownership>())
        .map_err(ParseFailure::terminal)?;
    let mut ownership = Ownership::default();
    let mut body_ownership = ownership;
    let mut tokens = Vec::new();
    let mut preamble = super::preamble::Lexer::default();
    let mut wrapper = Wrapper::Uml;
    let mut offset = 0;
    let mut timing_at = 0;
    let mut timing_declaration = false;
    for physical in document.text.split_inclusive('\n') {
        let raw = physical
            .strip_suffix('\n')
            .map_or(physical, |line| line.strip_suffix('\r').unwrap_or(line));
        let raw = if offset == 0 {
            raw.trim_start_matches('\u{feff}')
        } else {
            raw
        };
        budget.charge(1).map_err(ParseFailure::terminal)?;
        let checkpoint = ownership;
        let row = ownership.classify(raw, budget)?;
        if row.text.trim_start().starts_with('!') {
            return Err(ParseFailure::unsupported(
                offset..offset + raw.len(),
                "active preprocessing or generated directive",
            ));
        }
        if row.annotated_start {
            emit(
                Kind::Trivia,
                offset..offset + raw.len(),
                &mut tokens,
                budget,
            )?;
        }
        if !row.continuation {
            emit(
                row.kind,
                offset..offset + physical.len(),
                &mut tokens,
                budget,
            )?;
        }
        let evidence = ownership.advance(&row, &row.text, budget)?;
        if evidence.at && timing_at == 0 {
            timing_at = offset;
        }
        timing_declaration |= evidence.timing;
        match row.kind {
            Kind::Start(value) => wrapper = value,
            Kind::Content | Kind::Empty | Kind::Trivia | Kind::SheetStart | Kind::SheetEnd => {
                let finished = preamble.finished();
                preamble.row(&row, (offset, wrapper, evidence), budget)?;
                if !finished && preamble.finished() {
                    budget.charge(1).map_err(ParseFailure::terminal)?;
                    body_ownership = checkpoint;
                }
            }
            _ => {}
        }
        offset += physical.len();
    }
    ownership.finish(offset)?;
    let adapters = std::mem::size_of::<BodySource<'_>>();
    let retained = adapters.saturating_add(preamble.retained_bytes());
    let bounds = validate_retained(tokens, (offset, retained), budget)?;
    if timing_at > 0 && !timing_declaration {
        return Err(ParseFailure::syntax(
            timing_at..timing_at,
            "timing declaration required for @ statement",
        ));
    }
    Ok(ParsedDocument {
        wrapper: bounds.wrapper,
        body: BodySource {
            document,
            range: bounds.range,
            ownership: body_ownership,
            adapters,
        },
        preamble_tokens: preamble.tokens,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RenderOptions;

    #[test]
    fn should_preserve_authored_raw_rows_and_follow_expanded_next_row_terminators() {
        // given
        for (source, expected) in [
            (
                "@startuml\n!define FIN ;\n!define X Ready\nstart\n:open /' literal\n' X\nending FIN\n' !include inert\n:next X;\nstop\n@enduml",
                "\n\nstart\n:open /' literal\n' Ready\nending ;\n' !include inert\n:next Ready;\nstop\n",
            ),
            (
                "@startuml\n!define OPEN note over A\n!define CLOSE end note\n!define note body\n!define X Ready\nparticipant A\nOPEN\nend note\n' X\nCLOSE\nA -> B : X\n@enduml",
                "\n\n\n\nparticipant A\nnote over A\nend body\n' Ready\nend note\nA -> B : Ready\n",
            ),
            (
                "@startuml\n!define X Ready\n/'\n!define X ignored\n!include inert\n<style>\n@enduml\n'/\nA -> B : X\n@enduml",
                "\n/'\n!define X ignored\n!include inert\n<style>\n@enduml\n'/\nA -> B : Ready\n",
            ),
            (
                "@startyaml\n!define X Ready\nlabel: '\"/' X @enduml\"'\n@endyaml",
                "\nlabel: '\"/' Ready @enduml\"'\n",
            ),
        ] {
            let mut budget = Budget::new(&RenderOptions::default());
            let document = super::super::super::preprocess::prepare(source, &mut budget).unwrap();
            // when
            let parsed = parse(&document, &mut budget).unwrap();
            // then
            assert_eq!(
                &parsed.body.document.text[parsed.body.range.clone()],
                expected
            );
        }
    }

    #[test]
    fn should_let_generated_productions_own_pairing_placement_and_complete_eof() {
        // given
        for tokens in [
            vec![
                (0, Token::StartGantt, 1),
                (1, Token::Row, 2),
                (2, Token::EndUml, 3),
            ],
            vec![
                (0, Token::Definition, 1),
                (1, Token::StartUml, 2),
                (2, Token::Row, 3),
                (3, Token::EndUml, 4),
            ],
            vec![
                (0, Token::StartUml, 1),
                (1, Token::Row, 2),
                (2, Token::EndUml, 3),
                (3, Token::Row, 4),
            ],
        ] {
            let mut budget = Budget::new(&RenderOptions::default());
            // when
            let error = validate(tokens, 4, &mut budget).err().unwrap();
            // then
            assert!(
                matches!(error.kind, super::super::error::FailureKind::Syntax(_)),
                "{error:?}"
            );
        }
    }

    #[test]
    fn should_admit_stacks_and_preserve_cancellation_in_delivery_reductions_and_eof() {
        // given
        let tokens = vec![
            (0, Token::StartUml, 1),
            (1, Token::Row, 2),
            (2, Token::EndUml, 3),
        ];
        let mut options = RenderOptions::default();
        options.limits.layout_work = 20;
        let mut budget = Budget::new(&options);
        // when
        let error = validate(tokens.clone(), 3, &mut budget).err().unwrap();
        // then
        assert!(matches!(
            error.kind,
            super::super::error::FailureKind::Terminal(RenderError::Backend(_))
        ));
        for checkpoints in 1..7 {
            // given
            let mut budget = Budget::new(&RenderOptions::default());
            budget.control.cancel_after_checkpoints(checkpoints);
            // when
            let error = validate(tokens.clone(), 3, &mut budget).err().unwrap();
            // then
            assert!(
                matches!(
                    error.kind,
                    super::super::error::FailureKind::Terminal(RenderError::Cancelled(_))
                ),
                "checkpoint {checkpoints}: {error:?}"
            );
        }
        // given
        let options = RenderOptions::default();
        let mut empty = context::Checked::<Spanned>::new(Vec::new(), options.control.clone());
        options.control.cancel();
        // when
        let result = empty.next();
        // then
        assert!(matches!(
            result,
            Some(Err(ParseFailure {
                kind: super::super::error::FailureKind::Terminal(RenderError::Cancelled(_)),
                ..
            }))
        ));
    }

    #[test]
    fn should_map_generated_document_failures_to_one_frozen_definition_use() {
        // given
        let source = "\u{feff}@startuml\r\n!define CLOSE </style>\r\n!define UNUSED inert\r\nCLOSE\r\n@enduml";
        let mut budget = Budget::new(&RenderOptions::default());
        let document = super::super::super::preprocess::prepare(source, &mut budget).unwrap();
        // when
        let error = parse(&document, &mut budget).err().unwrap();
        // then
        assert!(matches!(error, RenderError::InvalidSource(_)));
        let message = error.to_string();
        assert!(message.contains("expanded bytes"), "{message}");
        assert!(
            message.contains("used at line 4, column 1, defined at line 2, column 1"),
            "{message}"
        );
        assert_eq!(message.matches("expansion").count(), 1);
        assert!(!message.contains("UNUSED"), "{message}");
    }

    #[test]
    fn should_borrow_body_bytes_and_preserve_physical_utf8_lines() {
        // given
        let mut budget = Budget::new(&RenderOptions::default());
        let document = super::super::super::preprocess::prepare(
            "\u{feff}@startuml\r\n!define X Ω\r\n  A -> B : X\r\n@enduml\r\n",
            &mut budget,
        )
        .unwrap();
        // when
        let parsed = parse(&document, &mut budget).unwrap();
        // then
        assert!(matches!(parsed.wrapper, Wrapper::Uml));
        assert_eq!(
            &parsed.body.document.text[parsed.body.range.clone()],
            "\r\n  A -> B : Ω\r\n"
        );
        let raw = &document.text[parsed.body.range.clone()];
        let message = parsed.body.range.start + raw.find('A').unwrap();
        assert_eq!(
            document.text[..message]
                .bytes()
                .filter(|byte| *byte == b'\n')
                .count()
                + 1,
            3
        );
        assert_eq!(parsed.body.adapters, std::mem::size_of::<BodySource<'_>>());
    }
}
