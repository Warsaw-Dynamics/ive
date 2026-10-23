//! MIT PlantUML modern activity grammar and structured instruction semantics.
//! Source/notice mappings are recorded in the provenance manifest.
pub(super) type Block = Vec<Instruction>;
#[derive(Debug)]
pub(super) enum Instruction {
    Start,
    Stop,
    End,
    Action(String),
    If {
        branches: Vec<Branch>,
        otherwise: Block,
        guard: String,
    },
    While {
        condition: Condition,
        body: Block,
        out: String,
    },
    Repeat {
        entry: String,
        body: Block,
        condition: Condition,
        out: String,
    },
    Fork {
        lanes: Vec<Block>,
        join: Join,
        label: String,
    },
    Group {
        name: String,
        kind: GroupKind,
        body: Block,
    },
}
#[derive(Debug)]
pub(super) struct Condition {
    pub test: String,
    pub guard: String,
}
#[derive(Debug)]
pub(super) struct Branch {
    pub condition: Condition,
    pub body: Block,
}
#[derive(Clone, Copy, Debug)]
pub(super) enum Join {
    Synchronize,
    Merge,
}
#[derive(Clone, Copy, Debug)]
pub(super) enum GroupKind {
    Partition,
    Group,
}

#[cfg(test)]
pub(super) fn parse(
    source: &str,
    budget: &mut super::budget::Budget,
) -> Result<Block, crate::RenderError> {
    use super::origin::{Document, Origin, Segment, SourceSpan};
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
    let mut prepared = super::parser::document::parse(&document, budget)?;
    super::parser::preamble::parse(&mut prepared, budget)?;
    super::parser::activity::parse(prepared.body, budget)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RenderError, plantuml::budget::Budget};
    #[test]
    fn should_scan_whitespace_only_multiline_actions_once() {
        // given
        let source = format!("@startuml\n:\n{};\nstop\n@enduml", " \n".repeat(60_000));
        let mut options = crate::RenderOptions::default();
        options.limits.timeout = std::time::Duration::from_millis(500);
        let mut budget = Budget::new(&options);
        // when
        let result = parse(&source, &mut budget);
        // then
        assert!(
            matches!(result, Err(RenderError::Backend(ref message)) if message == "PlantUML label bytes value 16385 exceeds limit 16384"),
            "{result:?}"
        );
        assert!(budget.work < source.len() * 4);
    }

    #[test]
    fn should_parse_below_cap_multiline_actions_and_check_cancellation() {
        // given
        let source = format!("@startuml\n:\n{};\nstop\n@enduml", " \n".repeat(4_000));
        let options = crate::RenderOptions::default();
        let mut budget = Budget::new(&options);
        // when
        let result = parse(&source, &mut budget);
        options.control.cancel();
        let cancelled = parse(&source, &mut budget);
        // then
        assert!(result.is_ok(), "{result:?}");
        assert!(budget.work < source.len() * 8);
        assert!(matches!(cancelled, Err(RenderError::Cancelled(_))));
    }
}
