use diagram_engine::{Language, RenderOptions, render};

fn options() -> RenderOptions {
    RenderOptions {
        language: Language::PlantUml,
        ..Default::default()
    }
}
fn draw(source: &str, family: &str) -> String {
    let output = render(source, &options()).unwrap();
    assert_eq!(output.family, family);
    usvg::Tree::from_str(&output.svg, &usvg::Options::default()).unwrap();
    output.svg
}

#[test]
fn should_draw_chen_nodes_with_qualified_ownership_and_cardinalities() {
    // given
    let source = "@startchen\nentity \"Customer\" as C {\nname\naddress {\ncity\n}\n}\nentity O {\nname\n}\nrelationship R {\nsince\n}\nC -1- R\nR =N= O\n@endchen";
    // when
    let svg = draw(source, "chen");
    let xml = roxmltree::Document::parse(&svg).unwrap();
    let nodes: Vec<_> = xml
        .descendants()
        .filter(|n| {
            n.attribute("data-kind").is_some_and(|k| {
                matches!(k, "chen-entity" | "chen-relationship" | "chen-attribute")
            })
        })
        .collect();
    // then
    assert_eq!(nodes.len(), 8);
    for (key, tag) in [
        ("C", "rect"),
        ("R", "polygon"),
        ("C/name", "ellipse"),
        ("O/name", "ellipse"),
        ("C/address/city", "ellipse"),
    ] {
        let node = nodes
            .iter()
            .find(|n| n.attribute("data-name") == Some(key))
            .unwrap();
        assert!(
            node.children().any(|n| n.has_tag_name(tag)),
            "missing {key} {tag}"
        );
    }
    assert_eq!(
        xml.descendants()
            .filter(|n| n.attribute("data-kind") == Some("chen-ownership"))
            .count(),
        5
    );
    let links: Vec<_> = xml
        .descendants()
        .filter(|n| n.attribute("data-kind") == Some("chen-association"))
        .collect();
    assert_eq!(links.len(), 2);
    assert!(
        links
            .iter()
            .all(|n| n.descendants().all(|c| !c.has_tag_name("polygon")))
    );
    assert!(
        links[1]
            .descendants()
            .any(|n| n.attribute("stroke-width") == Some("2"))
    );
    assert!(svg.contains(">1</text>") && svg.contains(">N</text>"));
}

#[test]
fn should_keep_parallel_cyclic_disconnected_chen_associations() {
    // given
    let source = "@startchen\nentity A {\n}\nrelationship B {\n}\nentity C {\n}\nA -1- B\nA -(0,N)- B\nB -- A\n@endchen";
    // when
    let svg = draw(source, "chen");
    // then
    assert_eq!(svg.matches("data-kind=\"chen-association\"").count(), 3);
    assert!(svg.contains("(0,N)"));
    assert!(!svg.contains("NaN"));
}

#[test]
fn should_reject_invalid_chen_ownership_endpoints_and_unsupported_semantics() {
    // given
    for body in [
        "orphan",
        "entity A",
        "entity A {\nx\nx\n}",
        "entity A {\nx",
        "}",
        "entity A {\n}\nA -1- Missing",
        "entity A {\nentity B {\n}\n}",
        "entity A {\n}\nA =1- A",
        "entity A <<weak>> {\n}",
        "relationship R <<identifying>> {\n}",
        "entity A {\nx <<key>>\n}",
        "entity A {\nx <<multi>>\n}",
        "entity A {\nx <<derived>>\n}",
        "entity A {\n}\nA ->- B",
        "entity A {\n}\nA ->- d {B,C}",
        "entity A {\n}\nA ->- o {B,C}",
        "entity A {\n}\nA ->- U {B,C}",
        "entity A {\n}\nA - 1 - A",
        "entity A {\n}\nA -- A #123456",
        "entity A {\n}\n@endchen\n/' unclosed",
    ] {
        // when
        let result = render(&format!("@startchen\n{body}\n@endchen"), &options());
        // then
        assert!(result.is_err(), "accepted {body:?}");
    }
}

#[test]
fn should_extend_description_with_colored_icon_free_archimate() {
    // given
    let source = "@startuml\narchimate #LightBlue \"Order service\" as Orders\narchimate #LightGreen \"Customer record\" as Record\nOrders --> Record : reads\n@enduml";
    // when
    let svg = draw(source, "description");
    let xml = roxmltree::Document::parse(&svg).unwrap();
    let nodes: Vec<_> = xml
        .descendants()
        .filter(|n| n.attribute("data-kind") == Some("archimate"))
        .collect();
    // then
    assert_eq!(nodes.len(), 2);
    assert_eq!(nodes[0].attribute("data-name"), Some("Orders"));
    assert!(
        nodes[0]
            .descendants()
            .any(|n| n.has_tag_name("rect") && n.attribute("fill") == Some("#add8e6"))
    );
    assert!(
        nodes[1]
            .descendants()
            .any(|n| n.has_tag_name("rect") && n.attribute("fill") == Some("#90ee90"))
    );
    let edge = xml
        .descendants()
        .find(|n| n.attribute("data-kind") == Some("relation"))
        .unwrap();
    assert_eq!(edge.attribute("data-owner"), Some("n0"));
    assert_eq!(edge.attribute("data-target"), Some("n1"));
    assert!(edge.descendants().any(|n| n.has_tag_name("polygon")));
}

#[test]
fn should_link_an_archimate_declaration_with_arrow_looking_quoted_display() {
    // given
    let source = "@startuml\narchimate #LightBlue \"Order --> service\" as archimate\nrectangle \"Target --> record\" as B\narchimate \"source role\" --> \"target role\" B : reads --> literal\n@enduml";
    // when
    let svg = draw(source, "description");
    let xml = roxmltree::Document::parse(&svg).unwrap();
    let node = xml
        .descendants()
        .find(|n| n.attribute("data-kind") == Some("archimate"))
        .unwrap();
    let edge = xml
        .descendants()
        .find(|n| n.attribute("data-kind") == Some("relation"))
        .unwrap();
    // then
    assert_eq!(node.attribute("data-name"), Some("archimate"));
    assert!(
        node.descendants()
            .any(|n| n.has_tag_name("rect") && n.attribute("fill") == Some("#add8e6"))
    );
    assert_eq!(edge.attribute("data-owner"), Some("n0"));
    assert_eq!(edge.attribute("data-target"), Some("n1"));
    for text in [
        "Order --> service",
        "Target --> record",
        "source role",
        "target role",
        "reads --> literal",
    ] {
        assert!(
            xml.descendants()
                .any(|n| n.has_tag_name("text") && n.text() == Some(text)),
            "missing {text}"
        );
    }
}

#[test]
fn should_reject_archimate_icons_colors_groups_and_invented_wrapper() {
    // given
    for body in [
        "archimate #NoSuchColor A",
        "archimate #url(x) A",
        "archimate #12345 A",
        "archimate #123456 A <<business>>",
        "archimate #123456 \"A\" <<business>> as A",
        "archimate #123456 A <<business>> as \"A\"",
        "archimate #123456 A {\nrectangle B\n}",
        "archimate #123456 A [\nbody\n]",
        "archimate #123456 A$bad",
        "!include <archimate>",
        "$archimate(A)",
    ] {
        // when
        let result = render(&format!("@startuml\n{body}\n@enduml"), &options());
        // then
        assert!(result.is_err(), "accepted {body:?}");
    }
    assert!(
        render(
            "@startarchimate\narchimate #123456 A\n@endarchimate",
            &options()
        )
        .is_err()
    );
}

#[test]
fn should_bound_models_output_and_cancel_both_extensions() {
    // given
    for source in [
        "@startchen\nentity A {\nx\n}\n@endchen",
        "@startuml\narchimate #123456 A\narchimate #654321 B\nA --> B\n@enduml",
    ] {
        for limit in ["model", "output", "work", "cancel"] {
            let mut config = options();
            match limit {
                "model" => config.limits.model_items = 1,
                "output" => config.limits.svg_bytes = 100,
                "work" => config.limits.layout_work = 1,
                _ => config.control.cancel(),
            }
            // when
            let result = render(source, &config);
            // then
            assert!(result.is_err(), "accepted {limit}: {source}");
        }
    }
}

#[test]
fn should_reject_archimate_empty_or_unsupported_quoted_alias_and_quoted_color() {
    // given
    for declaration in [
        "archimate #123456 \"\"",
        "archimate #123456 \"bad alias!\"",
        "archimate \"#123456\" A",
    ] {
        // when
        let result = render(&format!("@startuml\n{declaration}\n@enduml"), &options());
        // then
        assert!(result.is_err(), "accepted {declaration:?}");
    }
}

#[test]
fn should_stop_both_extensions_after_first_cancelled_host_measurement() {
    use diagram_engine::{
        HostMeasurementResult, HostTextMeasurementRequest, HostTextMeasurer, MeasurementProfileId,
        RenderControl, TextMeasurementPhase, TextMeasurementPolicy, TextMeasurementProfileIdentity,
    };
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    struct Cancel {
        control: RenderControl,
        calls: AtomicUsize,
    }
    impl HostTextMeasurer for Cancel {
        fn measure(&self, _: HostTextMeasurementRequest<'_>) -> HostMeasurementResult {
            self.calls.fetch_add(1, Ordering::Relaxed);
            self.control.cancel();
            Ok(None)
        }
    }
    // given
    for source in [
        "@startchen\nentity A {\nx\n}\n@endchen",
        "@startuml\narchimate #123456 A\narchimate #654321 B\nA --> B\n@enduml",
    ] {
        let mut config = options();
        let host = Arc::new(Cancel {
            control: config.control.clone(),
            calls: AtomicUsize::new(0),
        });
        config.text_measurement = TextMeasurementPolicy::host_display(
            TextMeasurementProfileIdentity::new(
                MeasurementProfileId::new("chen-archimate-cancel").unwrap(),
                "1",
            )
            .unwrap(),
            host.clone(),
            [TextMeasurementPhase::Layout],
        );
        // when
        let error = render(source, &config).unwrap_err();
        // then
        assert!(error.to_string().contains("cancel"), "{error}");
        assert_eq!(host.calls.load(Ordering::Relaxed), 1);
    }
}

#[test]
fn should_render_exact_fresh_original_sources_without_family_inflation() {
    // given
    for (source, family, kind, count) in [
        (
            include_str!("../../fixtures/plantuml/chenbase.puml"),
            "chen",
            "chen-attribute",
            2,
        ),
        (
            include_str!("../../fixtures/plantuml/archimatebase.puml"),
            "description",
            "archimate",
            2,
        ),
    ] {
        // when
        let svg = draw(source, family);
        // then
        assert_eq!(svg.matches(&format!("data-kind=\"{kind}\"")).count(), count);
    }
}

#[test]
fn should_preserve_description_precedence_and_reject_mixed_class_archimate() {
    // given
    let source = "@startuml\npackage P {\narchimate #112233 \"Service\" as S\nrectangle D\nS --> D\n}\n@enduml";
    // when
    let svg = draw(source, "description");
    let xml = roxmltree::Document::parse(&svg).unwrap();
    // then
    let node = xml
        .descendants()
        .find(|n| n.attribute("data-kind") == Some("archimate"))
        .unwrap();
    assert_eq!(node.attribute("data-owner"), Some("n0"));
    assert_eq!(node.attribute("data-name"), Some("P.S"));
    assert_eq!(
        draw(
            "@startuml\narchimate -> Other : ordinary endpoint\n@enduml",
            "sequence"
        )
        .matches("data-kind=\"message\"")
        .count(),
        1
    );
    for body in [
        "class C\narchimate #123456 A",
        "archimate #123456 A\nclass C",
    ] {
        assert!(render(&format!("@startuml\n{body}\n@enduml"), &options()).is_err());
    }
}

#[test]
fn should_reject_svg_output_at_the_bounded_writer_for_both_extensions() {
    // given
    let chen = format!(
        "@startchen\n{}\n@endchen",
        (0..40)
            .map(|id| format!("entity N{id} {{\n}}\n"))
            .collect::<String>()
    );
    let archimate = format!(
        "@startuml\n{}\n@enduml",
        (0..40)
            .map(|id| format!("archimate #123456 N{id}\n"))
            .collect::<String>()
    );
    for source in [chen, archimate] {
        let mut config = options();
        config.limits.svg_bytes = 8192;
        // when
        let error = render(&source, &config).unwrap_err();
        // then
        assert!(error.to_string().contains("SVG bytes"), "{error}");
    }
}
