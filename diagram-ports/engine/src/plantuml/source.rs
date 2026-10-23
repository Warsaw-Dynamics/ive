//! Generated-document wrapper identities.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Wrapper {
    Bpm,
    Chart,
    Ebnf,
    Regex,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RenderError, RenderOptions, plantuml::budget::Budget};

    fn inspect(source: &str, budget: &mut Budget) -> Result<(Wrapper, String, usize), RenderError> {
        let document = crate::plantuml::parser::test_support::document(source, budget)?;
        let parsed = crate::plantuml::parser::document::parse(&document, budget)?;
        let body = &document.text[parsed.body.range.clone()];
        let content = parsed.body.range.start + body.len() - body.trim_start().len();
        let line = document.text[..content]
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count()
            + 1;
        Ok((parsed.wrapper, body.to_owned(), line))
    }

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
                let (_, body, line) = inspect(&format!("{complete}{suffix}"), &mut budget).unwrap();
                // then
                assert_eq!(body.trim(), "A");
                assert_eq!(line, 2);
            }
            let accepts_comments = matches!(start, "uml" | "gantt" | "nwdiag" | "chen");
            let mut budget = Budget::new(&RenderOptions::default());
            assert_eq!(
                inspect(&format!("{complete}/' closed '/\n"), &mut budget).is_ok(),
                accepts_comments,
                "{start}"
            );
            let mut budget = Budget::new(&RenderOptions::default());
            assert_eq!(
                inspect(
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
                let result = inspect(&input, &mut budget);
                // then
                assert!(result.is_err(), "accepted {input:?}");
            }
        }
        for (source, accepted) in [
            ("@STARTUML\nA -> B\n@ENDUML", true),
            ("@STARTGANTT\nA\n@ENDGANTT", false),
        ] {
            let mut budget = Budget::new(&RenderOptions::default());
            assert_eq!(inspect(source, &mut budget).is_ok(), accepted);
        }
    }

    #[test]
    fn should_preserve_literal_raw_rows_around_nonleading_exclamation_marks() {
        // given
        let source = "@startuml\n:opening ! literal\n  body ! literal  \n\nending;\n@enduml";
        let mut budget = Budget::new(&RenderOptions::default());
        // when
        let (_, body, _) = inspect(source, &mut budget).unwrap();
        // then
        assert_eq!(
            body.trim_matches('\n').split('\n').collect::<Vec<_>>(),
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
        let result = inspect(&source, &mut budget);
        // then
        assert!(
            result.is_err(),
            "scan work was not charged before retaining the line"
        );
    }
}
