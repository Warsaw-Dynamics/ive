//! Wrapper values and a temporary Line adapter for unmigrated family parser tests.
#[cfg(test)]
use super::budget::Budget;
#[cfg(test)]
use crate::RenderError;

#[derive(Clone, Debug)]
pub(super) struct Line {
    pub number: usize,
    pub text: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Wrapper {
    Uml,
    Mindmap,
    Wbs,
    Json,
    Yaml,
    Gantt,
    Salt,
    Nwdiag,
    Files,
    Packet,
    Board,
    Hcl,
    Chen,
}

// Transitional materialization for unmigrated family parser tests.
#[cfg(test)]
pub(super) fn prepare_document(
    source: &str,
    budget: &mut Budget,
) -> Result<(Wrapper, Vec<Line>), RenderError> {
    use super::origin::{Document, Origin, Segment, SourceSpan};
    super::budget::bound(
        "document source bytes",
        source.len(),
        budget.limits.source_bytes,
    )?;
    budget.text(source.len())?;
    let document = Document {
        text: source.to_owned(),
        segments: vec![Segment {
            output: 0..source.len(),
            origin: Origin::Direct(SourceSpan {
                start_byte: 0,
                end_byte: source.len(),
                line: 1,
                column: 1,
            }),
        }],
        definitions: Vec::new(),
    };
    super::parser::document::parse(&document, budget).map(|parsed| (parsed.wrapper, parsed.lines))
}

#[cfg(test)]
pub(super) fn prepare(source: &str, budget: &mut Budget) -> Result<Vec<Line>, RenderError> {
    prepare_document(source, budget).map(|(_, lines)| lines)
}

#[cfg(test)]
pub(super) fn prepare_styled(
    source: &str,
    budget: &mut Budget,
) -> Result<(super::style::Preamble, Vec<Line>), RenderError> {
    let document = super::preprocess::prepare(source, budget)?;
    let mut parsed = super::parser::document::parse(&document, budget)?;
    let (preamble, body) = super::parser::preamble::parse(&mut parsed, budget)?;
    let first = document.text[..body.start]
        .bytes()
        .filter(|&byte| byte == b'\n')
        .count()
        + 1;
    parsed.lines.retain(|line| line.number >= first);
    Ok((preamble, parsed.lines))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RenderOptions;

    #[test]
    fn should_consume_exactly_one_generated_document_and_validate_lexical_eof() {
        // given
        for (start, end) in [
            ("uml", "uml"),
            ("mindmap", "mindmap"),
            ("wbs", "wbs"),
            ("json", "json"),
            ("yaml", "yaml"),
            ("gantt", "gantt"),
            ("salt", "salt"),
            ("nwdiag", "nwdiag"),
            ("files", "files"),
            ("packetdiag", "packetdiag"),
            ("board", "board"),
            ("hcl", "hcl"),
            ("chen", "chen"),
        ] {
            let complete = format!("\u{feff}@start{start}\r\nA\r\n@end{end}\r\n");
            for suffix in ["", " \r\n"] {
                let mut budget = Budget::new(&RenderOptions::default());
                // when
                let (_, lines) =
                    prepare_document(&format!("{complete}{suffix}"), &mut budget).unwrap();
                // then
                assert_eq!((lines[0].number, lines[0].text.as_str()), (2, "A"));
            }
            let accepts_comments = matches!(start, "uml" | "gantt" | "nwdiag" | "chen");
            let mut budget = Budget::new(&RenderOptions::default());
            assert_eq!(
                prepare_document(&format!("{complete}/' closed '/\n"), &mut budget).is_ok(),
                accepts_comments,
                "{start}"
            );
            let mut budget = Budget::new(&RenderOptions::default());
            assert_eq!(
                prepare_document(
                    &format!("/' prefix '/\n{}", complete.trim_start_matches('\u{feff}')),
                    &mut budget
                )
                .is_ok(),
                start == "uml",
                "{start}"
            );
            for input in [
                format!("{complete}/' unterminated"),
                format!("{complete}@start{start}\nB\n@end{end}"),
                format!("{complete}extra"),
                format!("@start{start} output.png\nA\n@end{end}"),
                format!("@start{start}\nA\n@endunknown"),
            ] {
                let mut budget = Budget::new(&RenderOptions::default());
                // when
                let result = prepare_document(&input, &mut budget);
                // then
                assert!(result.is_err(), "accepted {input:?}");
            }
        }
        for (source, accepted) in [
            ("@STARTUML\nA -> B\n@ENDUML", true),
            ("@STARTGANTT\nA\n@ENDGANTT", false),
        ] {
            let mut budget = Budget::new(&RenderOptions::default());
            assert_eq!(prepare_document(source, &mut budget).is_ok(), accepted);
        }
    }

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
