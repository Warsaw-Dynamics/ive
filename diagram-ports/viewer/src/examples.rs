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
        .find(|example| example.name == name)
        .map(|example| example.source)
        .ok_or_else(|| {
            format!("unknown built-in example {name:?}; choose one from the viewer's example list")
        })
}
