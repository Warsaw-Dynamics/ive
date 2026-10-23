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
mod tests {
    use crate::{RenderError, plantuml::budget::Budget};
    #[test]
    fn should_scan_whitespace_only_multiline_actions_once() {
        // given
        let source = format!("@startuml\n:\n{};\nstop\n@enduml", " \n".repeat(60_000));
        let mut options = crate::RenderOptions::default();
        options.limits.timeout = std::time::Duration::from_millis(500);
        let mut budget = Budget::new(&options);
        // when
        let result = crate::plantuml::parser::test_support::literal(
            &source,
            &mut budget,
            crate::plantuml::parser::activity::parse,
        );
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
        let result = crate::plantuml::parser::test_support::literal(
            &source,
            &mut budget,
            crate::plantuml::parser::activity::parse,
        );
        options.control.cancel();
        let cancelled = crate::plantuml::parser::test_support::literal(
            &source,
            &mut budget,
            crate::plantuml::parser::activity::parse,
        );
        // then
        assert!(result.is_ok(), "{result:?}");
        assert!(budget.work < source.len() * 8);
        assert!(matches!(cancelled, Err(RenderError::Cancelled(_))));
    }
}
