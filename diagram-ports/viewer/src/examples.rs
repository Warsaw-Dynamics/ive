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
        name: "plantuml-ebnf",
        source: include_str!("../../fixtures/plantuml/ebnf.puml"),
    },
    Example {
        name: "plantuml-regex",
        source: include_str!("../../fixtures/plantuml/regex.puml"),
    },
    Example {
        name: "plantuml-chart",
        source: include_str!("../../fixtures/plantuml/chart.puml"),
    },
    Example {
        name: "plantuml-chen",
        source: include_str!("../../fixtures/plantuml/chen.puml"),
    },
    Example {
        name: "plantuml-archimate",
        source: include_str!("../../fixtures/plantuml/archimate.puml"),
    },
    Example {
        name: "plantuml-files",
        source: include_str!("../../fixtures/plantuml/files.puml"),
    },
    Example {
        name: "plantuml-packetdiag",
        source: include_str!("../../fixtures/plantuml/packetdiag.puml"),
    },
    Example {
        name: "plantuml-board",
        source: include_str!("../../fixtures/plantuml/board.puml"),
    },
    Example {
        name: "plantuml-hcl",
        source: include_str!("../../fixtures/plantuml/hcl.puml"),
    },
    Example {
        name: "plantuml-nwdiag",
        source: include_str!("../../fixtures/plantuml/nwdiag.puml"),
    },
    Example {
        name: "plantuml-salt",
        source: include_str!("../../fixtures/plantuml/salt.puml"),
    },
    Example {
        name: "plantuml-gantt",
        source: include_str!("../../fixtures/plantuml/gantt.puml"),
    },
    Example {
        name: "plantuml-timing",
        source: include_str!("../../fixtures/plantuml/timing.puml"),
    },
    Example {
        name: "plantuml-constant-sequence",
        source: include_str!("../../fixtures/plantuml/constantsequence.puml"),
    },
    Example {
        name: "plantuml-constant-graph",
        source: include_str!("../../fixtures/plantuml/constantgraph.puml"),
    },
    Example {
        name: "plantuml-styled-sequence",
        source: include_str!("../../fixtures/plantuml/styledsequence.puml"),
    },
    Example {
        name: "plantuml-styled-graph",
        source: include_str!("../../fixtures/plantuml/styledgraph.puml"),
    },
    Example {
        name: "plantuml-mindmap",
        source: include_str!("../../fixtures/plantuml/mindmap.puml"),
    },
    Example {
        name: "plantuml-wbs",
        source: include_str!("../../fixtures/plantuml/wbs.puml"),
    },
    Example {
        name: "plantuml-json",
        source: include_str!("../../fixtures/plantuml/json.puml"),
    },
    Example {
        name: "plantuml-yaml",
        source: include_str!("../../fixtures/plantuml/yaml.puml"),
    },
    Example {
        name: "plantuml-state",
        source: include_str!("../../fixtures/plantuml/state.puml"),
    },
    Example {
        name: "plantuml-activity",
        source: include_str!("../../fixtures/plantuml/activity.puml"),
    },
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
    fn should_find_chen_and_icon_free_archimate_with_honest_families() {
        // given
        for (name, family, kind) in [
            ("plantuml-chen", "chen", "chen-relationship"),
            ("plantuml-archimate", "description", "archimate"),
        ] {
            // when
            let source = super::source(name).unwrap();
            let output = diagram_engine::render(
                source,
                &diagram_engine::RenderOptions {
                    language: diagram_engine::Language::PlantUml,
                    ..Default::default()
                },
            )
            .unwrap();
            // then
            assert_eq!(output.family, family);
            assert!(output.svg.contains(&format!("data-kind=\"{kind}\"")));
        }
    }
    #[test]
    fn should_find_four_native_inventory_examples_with_real_family_shapes() {
        // given
        for (name, family, kind) in [
            ("plantuml-files", "files", "files-note"),
            ("plantuml-packetdiag", "packetdiag", "packet-fragment"),
            ("plantuml-board", "board", "board-stage"),
            ("plantuml-hcl", "hcl", "data-link"),
        ] {
            // when
            let source = super::source(name).unwrap();
            let output = diagram_engine::render(
                source,
                &diagram_engine::RenderOptions {
                    language: diagram_engine::Language::PlantUml,
                    ..Default::default()
                },
            )
            .unwrap();
            // then
            assert_eq!(output.family, family);
            assert!(output.svg.contains(&format!("data-kind=\"{kind}\"")));
        }
    }
    #[test]
    fn should_find_native_chart_example_with_actual_default_range_bars_and_legend() {
        // given
        let source = super::source("plantuml-chart").unwrap();
        // when
        let output = diagram_engine::render(
            source,
            &diagram_engine::RenderOptions {
                language: diagram_engine::Language::PlantUml,
                ..Default::default()
            },
        )
        .unwrap();
        // then
        assert_eq!(output.family, "chart");
        assert_eq!(output.svg.matches("data-kind=\"chart-bar\"").count(), 3);
        assert_eq!(output.svg.matches("data-kind=\"chart-legend\"").count(), 1);
        assert_eq!(output.svg.matches("data-kind=\"chart-tick\"").count(), 6);
    }

    #[test]
    fn should_find_native_nwdiag_example_with_shared_router_and_three_buses() {
        // given
        let name = "plantuml-nwdiag";
        // when
        let source = super::source(name).unwrap();
        let output = diagram_engine::render(
            source,
            &diagram_engine::RenderOptions {
                language: diagram_engine::Language::PlantUml,
                ..Default::default()
            },
        )
        .unwrap();
        // then
        assert_eq!(output.family, "nwdiag");
        assert_eq!(
            output.svg.matches("data-kind=\"nwdiag-network\"").count(),
            3
        );
        assert_eq!(output.svg.matches("data-kind=\"nwdiag-server\"").count(), 5);
        assert_eq!(
            output
                .svg
                .matches("data-kind=\"nwdiag-membership\"")
                .count(),
            7
        );
        assert!(output.svg.contains("Storage zone"));
    }
    #[test]
    fn should_find_native_salt_example_with_nested_controls_and_spans() {
        // given
        let name = "plantuml-salt";
        // when
        let source = super::source(name).unwrap();
        let output = diagram_engine::render(
            source,
            &diagram_engine::RenderOptions {
                language: diagram_engine::Language::PlantUml,
                ..Default::default()
            },
        )
        .unwrap();
        // then
        assert_eq!(output.family, "salt");
        for kind in [
            "salt-table",
            "salt-field",
            "salt-button",
            "salt-checkbox-on",
            "salt-checkbox-off",
            "salt-radio-on",
            "salt-radio-off",
        ] {
            assert!(output.svg.contains(kind), "{kind}");
        }
    }
    #[test]
    fn should_find_native_gantt_example_with_aliases_bars_dependencies_and_milestones() {
        // given
        let name = "plantuml-gantt";
        // when
        let source = super::source(name).unwrap();
        let output = diagram_engine::render(
            source,
            &diagram_engine::RenderOptions {
                language: diagram_engine::Language::PlantUml,
                ..Default::default()
            },
        )
        .unwrap();
        // then
        assert_eq!(output.family, "gantt");
        for kind in ["gantt-bar", "gantt-milestone", "gantt-dependency"] {
            assert!(output.svg.contains(kind));
        }
    }
    #[test]
    fn should_find_substantive_native_timing_example() {
        // given
        let name = "plantuml-timing";
        // when
        let source = super::source(name).unwrap();
        let output = diagram_engine::render(
            source,
            &diagram_engine::RenderOptions {
                language: diagram_engine::Language::PlantUml,
                ..Default::default()
            },
        )
        .unwrap();
        // then
        assert_eq!(output.family, "timing");
        for kind in [
            "timing-robust",
            "timing-concise",
            "timing-constraint",
            "timing-message",
        ] {
            assert!(output.svg.contains(kind));
        }
    }
    #[test]
    fn should_find_native_control_tree_and_data_examples() {
        // given
        for (name, family) in [
            ("plantuml-styled-sequence", "sequence"),
            ("plantuml-styled-graph", "class"),
            ("plantuml-state", "state"),
            ("plantuml-activity", "activity"),
            ("plantuml-mindmap", "mindmap"),
            ("plantuml-wbs", "wbs"),
            ("plantuml-json", "json"),
            ("plantuml-yaml", "yaml"),
        ] {
            // when
            let source = super::source(name).unwrap();
            let result = diagram_engine::render(
                source,
                &diagram_engine::RenderOptions {
                    language: diagram_engine::Language::PlantUml,
                    ..Default::default()
                },
            )
            .unwrap();
            // then
            assert_eq!(result.family, family);
        }
    }

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
    #[test]
    fn should_find_eager_constant_sequence_and_class_document_examples() {
        // given
        for name in ["plantuml-constant-sequence", "plantuml-constant-graph"] {
            // when
            let input = super::source(name).unwrap();
            // then
            let output = diagram_engine::render(
                input,
                &diagram_engine::RenderOptions {
                    language: diagram_engine::Language::PlantUml,
                    ..Default::default()
                },
            )
            .unwrap();
            assert!(output.svg.contains("Ready"));
            assert!(input.contains("!define"));
        }
    }
}
