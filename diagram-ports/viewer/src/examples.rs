pub struct Example {
    pub name: &'static str,
    pub source: &'static str,
}
macro_rules! examples {
    ($($name:literal),+ $(,)?) => { pub const EXAMPLES: &[Example] = &[$(Example { name: $name, source: include_str!(concat!("../../fixtures/mermaid/", $name, ".mmd")) }),+]; };
}
examples!(
    "flowchart",
    "sequence",
    "class",
    "pie",
    "architecture",
    "state",
    "er",
    "gantt",
    "journey",
    "git",
    "mindmap",
    "timeline",
    "requirement",
    "c4",
    "block",
    "packet",
    "quadrant",
    "xychart",
    "sankey",
    "kanban",
    "treemap",
    "radar",
    "venn",
    "usecase",
    "tree",
    "swimlane",
    "ishikawa",
    "wardley",
    "cynefin",
    "eventmodeling",
    "agentflow",
    "zenuml",
    "railroad",
    "railroad-ebnf",
    "railroad-abnf",
    "railroad-peg",
    "info"
);

pub fn source(name: &str) -> Result<&'static str, String> {
    EXAMPLES
        .iter()
        .chain(PLANTUML_EXAMPLES.iter())
        .find(|example| example.name == name)
        .map(|example| example.source)
        .ok_or_else(|| {
            format!("unknown built-in example {name:?}; choose one from the viewer's example list")
        })
}

pub const PLANTUML_EXAMPLES: &[Example] = &[
    Example {
        name: "plantuml-sequence",
        source: include_str!("../../fixtures/plantuml/sequence.puml"),
    },
    Example {
        name: "plantuml-class",
        source: include_str!("../../fixtures/plantuml/class.puml"),
    },
    Example {
        name: "plantuml-object",
        source: include_str!("../../fixtures/plantuml/object.puml"),
    },
    Example {
        name: "plantuml-deployment",
        source: include_str!("../../fixtures/plantuml/deployment.puml"),
    },
    Example {
        name: "plantuml-usecase",
        source: include_str!("../../fixtures/plantuml/usecase.puml"),
    },
];

pub fn for_language(language: &str) -> &'static [Example] {
    if language == "plantuml" {
        PLANTUML_EXAMPLES
    } else {
        EXAMPLES
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn should_find_a_real_plantuml_example() {
        // given
        let name = "plantuml-sequence";
        // when
        let source = super::source(name).unwrap();
        // then
        let options = diagram_engine::RenderOptions {
            language: diagram_engine::Language::PlantUml,
            ..Default::default()
        };
        assert_eq!(
            diagram_engine::render(source, &options).unwrap().family,
            "sequence"
        );
    }
}
