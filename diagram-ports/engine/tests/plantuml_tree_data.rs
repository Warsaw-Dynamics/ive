use diagram_engine::{Language, RenderOptions, render};

fn options() -> RenderOptions {
    RenderOptions {
        language: Language::PlantUml,
        ..Default::default()
    }
}

fn checked(family: &str, body: &str) -> String {
    let source = format!("@start{family}\n{body}\n@end{family}");
    let result = render(&source, &options()).unwrap();
    assert_eq!(result.family, family);
    assert_eq!(result.svg, render(&source, &options()).unwrap().svg);
    let tree = usvg::Tree::from_str(&result.svg, &usvg::Options::default()).unwrap();
    assert!(tree.size().width().is_finite() && tree.size().width() > 0.);
    assert!(tree.size().height().is_finite() && tree.size().height() > 0.);
    result.svg
}

#[test]
fn should_use_independent_mindmap_side_stacks_and_stack_new_roots() {
    // given
    let body = "+ Root\n-- Left\n++ Right\n--- Left child\n+++ Right child\n+ Second\n++ Child";
    // when
    let result = checked("mindmap", body);
    let xml = roxmltree::Document::parse(&result).unwrap();
    let nodes: Vec<_> = xml
        .descendants()
        .filter(|n| n.attribute("data-kind") == Some("mindmap-node"))
        .collect();
    // then
    assert_eq!(nodes.len(), 7);
    assert_eq!(nodes[3].attribute("data-owner"), Some("m1"));
    assert_eq!(nodes[4].attribute("data-owner"), Some("m2"));
    assert_eq!(nodes[6].attribute("data-owner"), Some("m5"));
}

#[test]
fn should_keep_long_root_prefixes_and_aliases_distinct_from_display_labels() {
    // given
    let prefix = "*".repeat(16_385);
    let alias = "a".repeat(16_385);
    let cases = [
        ("mindmap", format!("{prefix} Root")),
        ("wbs", format!("{prefix} Root")),
        ("wbs", format!("*({alias}) Root")),
        ("wbs", format!("* \"Root\" as {alias}")),
    ];
    // when
    let outputs = cases
        .iter()
        .map(|(family, body)| checked(family, body))
        .collect::<Vec<_>>();
    // then
    for output in outputs {
        assert!(output.contains("Root"));
    }
    let label = "x".repeat(16_385);
    for (family, body) in [
        ("mindmap", format!("* {label}")),
        ("wbs", format!("* \"{label}\" as code")),
    ] {
        let source = format!("@start{family}\n{body}\n@end{family}");
        let error = render(&source, &options()).unwrap_err();
        assert!(
            matches!(error, diagram_engine::RenderError::Backend(ref message) if message == "PlantUML label bytes value 16385 exceeds limit 16384"),
            "{error}"
        );
    }
}

#[test]
fn should_normalize_mindmap_first_child_depth_without_guessing_visual_indent() {
    // given
    let body = "*** Root\n*** Child\n***** Grandchild\nleft side\n*** Left child";
    // when
    let result = checked("mindmap", body);
    let xml = roxmltree::Document::parse(&result).unwrap();
    let nodes: Vec<_> = xml
        .descendants()
        .filter(|n| n.attribute("data-kind") == Some("mindmap-node"))
        .collect();
    // then
    assert_eq!(nodes[1].attribute("data-owner"), Some("m0"));
    assert_eq!(nodes[2].attribute("data-owner"), Some("m1"));
    assert_eq!(nodes[3].attribute("data-owner"), Some("m0"));
}

#[test]
fn should_keep_wbs_global_parentage_direction_alias_and_pseudo_topology() {
    // given
    let body = "* Project\n**(design) Design\n---< Left\n+++> Right\n****_\n*****_ Leaf\\nsecond line\n** Rollout";
    // when
    let result = checked("wbs", body);
    let xml = roxmltree::Document::parse(&result).unwrap();
    let nodes: Vec<_> = xml
        .descendants()
        .filter(|n| matches!(n.attribute("data-kind"), Some("wbs-node" | "wbs-pseudo")))
        .collect();
    // then
    assert_eq!(nodes.len(), 7);
    assert_eq!(nodes[3].attribute("data-owner"), Some("w1"));
    assert_eq!(nodes[4].attribute("data-owner"), Some("w3"));
    assert_eq!(nodes[5].attribute("data-owner"), Some("w4"));
    assert_eq!(nodes[4].attribute("data-kind"), Some("wbs-pseudo"));
    assert!(nodes[4].children().all(|n| !n.is_element()));
    assert!(
        xml.descendants()
            .any(|n| n.attribute("data-kind") == Some("wbs-bus"))
    );
}

#[test]
fn should_preserve_json_duplicate_rows_types_and_container_row_connections() {
    // given
    let body =
        r#"{"z":1,"a":{"nested":null},"z":3,"flag":true,"ports":[80,443],"empty":{},"array":[]}"#;
    // when
    let result = checked("json", body);
    let xml = roxmltree::Document::parse(&result).unwrap();
    let rows: Vec<_> = xml
        .descendants()
        .filter(|n| {
            n.attribute("data-owner") == Some("d0")
                && n.attribute("data-kind")
                    .is_some_and(|k| k.starts_with("data-row-"))
        })
        .collect();
    // then
    assert_eq!(
        rows.iter()
            .map(|n| n.attribute("data-name").unwrap())
            .collect::<Vec<_>>(),
        ["z", "a", "z", "flag", "ports", "empty", "array"]
    );
    assert_eq!(rows[0].attribute("data-kind"), Some("data-row-number"));
    assert_eq!(rows[3].attribute("data-kind"), Some("data-row-boolean"));
    let links: Vec<_> = xml
        .descendants()
        .filter(|n| n.attribute("data-kind") == Some("data-link"))
        .collect();
    assert_eq!(links.len(), 4);
    assert_eq!(links[0].attribute("data-owner"), rows[1].attribute("id"));
    assert_eq!(links[1].attribute("data-owner"), rows[4].attribute("id"));
}

#[test]
fn should_keep_yaml_string_types_duplicates_quotes_flow_and_literal_blocks() {
    // given
    let body = "z: first\na: 2\nz: last\n\"a:b\": \"v:c # text\"\nflag: true\nnil: null\nobj: {}\nseq: []\ntext: |\n  first\n  second: value\nnext: 'it''s'\nescape: \"x\\ny\"";
    // when
    let result = checked("yaml", body);
    let xml = roxmltree::Document::parse(&result).unwrap();
    let rows: Vec<_> = xml
        .descendants()
        .filter(|n| {
            n.attribute("data-owner") == Some("d0")
                && n.attribute("data-kind")
                    .is_some_and(|k| k.starts_with("data-row-"))
        })
        .collect();
    let labels: Vec<_> = xml
        .descendants()
        .filter(|n| n.has_tag_name("text"))
        .filter_map(|n| n.text())
        .collect();
    // then
    assert_eq!(
        rows.iter()
            .map(|n| n.attribute("data-name").unwrap())
            .collect::<Vec<_>>(),
        [
            "z", "a", "a:b", "flag", "nil", "obj", "seq", "text", "next", "escape"
        ]
    );
    assert_eq!(rows[1].attribute("data-kind"), Some("data-row-string"));
    for text in [
        "last",
        "2",
        "v:c # text",
        "true",
        "null",
        "{}",
        "first",
        "second: value",
        "it''s",
        "x\\ny",
    ] {
        assert!(labels.contains(&text), "{text:?}: {labels:?}");
    }
}

#[test]
fn should_preserve_data_literal_markup_apostrophes_unicode_and_resource_looking_text() {
    // given
    let body = r#"{"x":"it's <img:x> **bold** /' */ https://example.test/a:b # !include","u":"\uD83D\uDE00 & < >","slash":"a\\nb"}"#;
    // when
    let result = checked("json", body);
    let xml = roxmltree::Document::parse(&result).unwrap();
    let labels: Vec<_> = xml
        .descendants()
        .filter(|n| n.has_tag_name("text"))
        .filter_map(|n| n.text())
        .collect();
    // then
    assert!(labels.contains(&"it's <img:x> **bold** /' */ https://example.test/a:b # !include"));
    assert!(labels.contains(&"😀 & < >"));
    assert!(labels.contains(&"a\\nb"));
}

#[test]
fn should_reject_invalid_tree_structure_styles_crosslinks_and_data_syntax() {
    // given
    let cases = [
        ("mindmap", "* R\n*** C\n**** Bad"),
        ("mindmap", "* R\n** C\n**** Jump"),
        ("mindmap", "* R\n**[#red] Color"),
        ("mindmap", "* R\n**:multiline\nend;"),
        ("wbs", "* R\n* second"),
        ("wbs", "* R\n*** jump"),
        ("wbs", "* R\n**(a) A\n**(b) B\na --> b : label"),
        ("json", "{\"x\":1,}"),
        ("json", "{\"x\":01}"),
        ("json", "[1] trailing"),
        ("json", "{\"x\":1e999999}"),
        ("json", "{\"x\":\"\\q\"}"),
        ("json", "// comment\n{}"),
        ("json", "#comment\n{}"),
        ("yaml", "a: >\n  folded"),
        ("yaml", "- name: |\n  text"),
        ("yaml", "---\na: value"),
        ("yaml", "a: &a [x,x]\nb: &b [*a,*a]\nc: &c [*b,*b]"),
        ("yaml", "<<: *x"),
        ("yaml", "x: !!str value"),
    ];
    for (family, body) in cases {
        // when
        let result = render(&format!("@start{family}\n{body}\n@end{family}"), &options());
        // then
        assert!(result.is_err(), "admitted {family}: {body}");
    }
}

#[test]
fn should_enforce_source_model_depth_work_output_limits_for_new_families() {
    // given
    for (family, body) in [
        ("mindmap", "* Root\n** Child\n*** Grandchild"),
        ("wbs", "* Root\n** Child\n*** Grandchild"),
        ("json", "{\"a\":{\"b\":[1,2,3]}}"),
        ("yaml", "a:\n  b:\n    - one\n    - two"),
    ] {
        for limit in 0..5 {
            let mut config = options();
            match limit {
                0 => config.limits.source_bytes = 8,
                1 => config.limits.model_items = 1,
                2 => config.limits.nesting_depth = 1,
                3 => config.limits.layout_work = 8,
                _ => config.limits.svg_bytes = 256,
            }
            // when
            let result = render(&format!("@start{family}\n{body}\n@end{family}"), &config);
            // then
            assert!(result.is_err(), "ignored limit {limit} in {family}");
        }
    }
}

#[test]
fn should_reject_active_preprocessor_lines_inside_yaml_literal_blocks() {
    // given
    for directive in [
        "!include missing-task3c.puml",
        "!includeurl https://invalid.example/task3c",
        "!define VALUE rewritten",
        "!if true",
        "!$x = 1",
    ] {
        let source = format!("@startyaml\ntext: |\n  {directive}\na: next\n@endyaml");
        // when
        let result = render(&source, &options());
        // then
        let error = result
            .expect_err("active preprocessing painted as literal YAML")
            .to_string();
        assert!(error.contains("line 3"), "{error}");
    }
}

#[test]
fn should_keep_inert_yaml_capability_looking_literals_and_original_block_stopping_rule() {
    // given
    let body = "quoted: \"&anchor *alias !!tag <<: !include inert\"\nblock: |\n  &literal *literal !customtag\n  \"!include quoted\"\nunindented plain literal\n  deep: literal\nnext: value";
    // when
    let result = checked("yaml", body);
    let xml = roxmltree::Document::parse(&result).unwrap();
    let labels: Vec<_> = xml
        .descendants()
        .filter(|n| n.has_tag_name("text"))
        .filter_map(|n| n.text())
        .collect();
    // then
    assert!(labels.contains(&"&anchor *alias !!tag <<: !include inert"));
    assert!(labels.contains(&"&literal *literal !customtag"));
    assert!(labels.contains(&"\"!include quoted\""));
    assert!(labels.contains(&"unindented plain literal"));
    assert!(labels.contains(&"deep: literal"));
}

#[test]
fn should_fail_yaml_alias_bomb_at_first_token_before_value_construction() {
    // given
    let mut config = options();
    config.limits.model_items = 1;
    let source = "@startyaml\na: &a [x,x,x,x]\nb: &b [*a,*a,*a,*a]\nc: &c [*b,*b,*b,*b]\n@endyaml";
    // when
    let error = render(source, &config).unwrap_err();
    // then
    assert!(matches!(
        error,
        diagram_engine::RenderError::UnsupportedFeature(_)
    ));
    assert!(error.to_string().contains("line 2"));
}

#[test]
fn should_keep_root_scalar_and_empty_container_policies_and_control_character_text() {
    // given
    for body in [
        "null",
        "true",
        "-1.25e2",
        "\"text\"",
        "{}",
        "[]",
        r#"{"\u0000":"\b\f\t\n\u0001"}"#,
    ] {
        // when
        let result = checked("json", body);
        let xml = roxmltree::Document::parse(&result).unwrap();
        // then
        assert!(xml.descendants().any(|n| {
            n.attribute("data-owner") == Some("d0")
                && n.attribute("data-kind")
                    .is_some_and(|k| k.starts_with("data-row-"))
        }));
    }
}

#[test]
fn should_admit_explicit_mindmap_roots_and_org_mode_smart_prefixes() {
    // given
    for body in [
        "0 Root\n** Child\n*** Grandchild",
        "* Root\n\t* Child\n\t\t* Grandchild",
        "* Root\n    ** Child\n*** Grandchild",
    ] {
        // when
        let result = checked("mindmap", body);
        let xml = roxmltree::Document::parse(&result).unwrap();
        // then
        let nodes: Vec<_> = xml
            .descendants()
            .filter(|n| n.attribute("data-kind") == Some("mindmap-node"))
            .collect();
        assert_eq!(nodes[2].attribute("data-owner"), Some("m1"));
    }
}

#[test]
fn should_cancel_new_families_during_real_layout_measurement() {
    use diagram_engine::{
        HostMeasurementResult, HostTextMeasurementRequest, HostTextMeasurer, MeasurementProfileId,
        RenderControl, RenderError, TextMeasurementPhase, TextMeasurementPolicy,
        TextMeasurementProfileIdentity,
    };
    use std::sync::Arc;
    struct Canceller(RenderControl);
    impl HostTextMeasurer for Canceller {
        fn measure(&self, _: HostTextMeasurementRequest<'_>) -> HostMeasurementResult {
            self.0.cancel();
            Ok(None)
        }
    }
    // given
    for (family, body) in [
        ("mindmap", "* Root\n** Child"),
        ("wbs", "* Root\n** Child"),
        ("json", "{\"x\":[1]}"),
        ("yaml", "x:\n  - one"),
    ] {
        let mut config = options();
        config.text_measurement = TextMeasurementPolicy::host_display(
            TextMeasurementProfileIdentity::new(
                MeasurementProfileId::new("tree-data-cancel").unwrap(),
                "1",
            )
            .unwrap(),
            Arc::new(Canceller(config.control.clone())),
            [TextMeasurementPhase::Layout],
        );
        // when
        let result = render(&format!("@start{family}\n{body}\n@end{family}"), &config);
        // then
        assert!(matches!(result, Err(RenderError::Cancelled(_))));
    }
}

#[test]
fn should_admit_exact_fresh_original_tree_and_data_fixtures() {
    // given
    for (family, source) in [
        (
            "mindmap",
            include_str!("../../fixtures/plantuml/mindmap.puml"),
        ),
        ("wbs", include_str!("../../fixtures/plantuml/wbs.puml")),
        ("json", include_str!("../../fixtures/plantuml/json.puml")),
        ("yaml", include_str!("../../fixtures/plantuml/yaml.puml")),
    ] {
        // when
        let result = render(source, &options()).unwrap();
        // then
        assert_eq!(result.family, family);
        assert_eq!(result.svg, render(source, &options()).unwrap().svg);
        usvg::Tree::from_str(&result.svg, &usvg::Options::default()).unwrap();
    }
}

#[test]
fn should_reject_spaced_yaml_merge_keys_but_keep_escaped_quote_flow_literals() {
    // given
    let source = "@startyaml\n<< : literal\n@endyaml";
    // when
    let result = render(source, &options());
    // then
    assert!(result.is_err(), "unquoted merge request admitted");
    let svg = checked("yaml", r#"items: ["escaped\" *literal", "&quoted"]"#);
    let xml = roxmltree::Document::parse(&svg).unwrap();
    let labels: Vec<_> = xml
        .descendants()
        .filter(|n| n.has_tag_name("text"))
        .filter_map(|n| n.text())
        .collect();
    assert!(labels.contains(&"escaped\" *literal"));
}

#[test]
fn should_enforce_flow_scalar_depth_at_exact_limit_and_one_below() {
    // given
    for (body, depth) in [
        ("a: [one]", 3),
        ("a: []", 3),
        ("nested:\n  a: [one]", 4),
        ("nested:\n  a: []", 4),
    ] {
        let source = format!("@startyaml\n{body}\n@endyaml");
        let mut exact = options();
        exact.limits.nesting_depth = depth;
        let mut below = options();
        below.limits.nesting_depth = depth - 1;
        // when
        let admitted = render(&source, &exact).unwrap();
        let rejected = render(&source, &below);
        // then
        assert_eq!(admitted.family, "yaml");
        usvg::Tree::from_str(&admitted.svg, &usvg::Options::default()).unwrap();
        let error = rejected
            .expect_err("flow scalar escaped caller nesting limit")
            .to_string();
        assert!(
            error.contains(&format!(
                "nesting depth value {depth} exceeds limit {}",
                depth - 1
            )),
            "{error}"
        );
    }
}

#[test]
fn should_render_long_whitespace_quote_and_escape_flow_fields_with_bounded_work() {
    // given
    for size in [4_000, 8_000, 16_000] {
        let spaces = " ".repeat(size);
        for body in [
            format!("a: [{spaces}]"),
            format!("a: [{spaces}\" quoted \",{spaces}\\\"literal]"),
        ] {
            let source = format!("@startyaml\n{body}\n@endyaml");
            let mut config = options();
            config.limits.layout_work = 20 * source.len() + 1_024;
            // when
            let output = render(&source, &config).unwrap();
            // then
            assert_eq!(output.family, "yaml");
            usvg::Tree::from_str(&output.svg, &usvg::Options::default()).unwrap();
            assert!(output.layout_work_units <= config.limits.layout_work);
            let xml = roxmltree::Document::parse(&output.svg).unwrap();
            let labels: Vec<_> = xml
                .descendants()
                .filter(|n| n.has_tag_name("text"))
                .filter_map(|n| n.text())
                .collect();
            if body.contains("quoted") {
                assert!(labels.contains(&" quoted "));
                assert!(labels.contains(&"\"literal"));
            }
            eprintln!(
                "render flow source_bytes={} work={} limit={}",
                source.len(),
                output.layout_work_units,
                config.limits.layout_work
            );
        }
    }
}
