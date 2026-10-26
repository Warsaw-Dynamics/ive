use diagram_engine::{Language, RenderOptions, render};

fn draw(body: &str) -> String {
    render(
        &format!("@startuml\n{body}\n@enduml"),
        &RenderOptions {
            language: Language::PlantUml,
            ..Default::default()
        },
    )
    .unwrap()
    .svg
}

#[test]
fn should_preserve_ordered_concurrent_regions_and_scoped_roles() {
    // given
    let body = "state S {\n[*] --> A\nA --> [*]\n--\n[*] --> B\nB --> [*]\n}";
    // when
    let svg = draw(body);
    let xml = roxmltree::Document::parse(&svg).unwrap();
    // then
    for (kind, count) in [
        ("state-region", 2),
        ("state-initial", 2),
        ("state-final", 2),
        ("state-transition", 4),
    ] {
        assert_eq!(
            xml.descendants()
                .filter(|n| n.attribute("data-kind") == Some(kind))
                .count(),
            count,
            "{kind}"
        );
    }
    assert_eq!(svg, draw(body));
    assert!(usvg::Tree::from_str(&svg, &usvg::Options::default()).is_ok());
}

#[test]
fn should_keep_named_and_scope_history_entities_distinct() {
    // given
    let body =
        "state S {\nstate h <<history>>\nstate d <<history*>>\nh --> [H]\nd --> [H*]\n[H] --> h\n}";
    // when
    let svg = draw(body);
    let xml = roxmltree::Document::parse(&svg).unwrap();
    // then
    for kind in ["state-history", "state-deep-history"] {
        assert_eq!(
            xml.descendants()
                .filter(|n| n.attribute("data-kind") == Some(kind))
                .count(),
            2
        );
    }
    assert_eq!(
        xml.descendants()
            .filter(|n| n.attribute("data-kind") == Some("state-transition"))
            .count(),
        3
    );
}

#[test]
fn should_draw_actual_choice_junction_and_explicit_fork_join_edges() {
    // given
    let body = "state Choose <<choice>>\nstate Junction <<junction>>\nstate Split <<fork>>\nstate Join <<join>>\n[*] --> Split\nSplit --> A : left\nSplit --> B : right\nA --> Join\nB --> Join\nJoin --> Choose\nChoose --> Junction : yes\nJunction --> [*]";
    // when
    let svg = draw(body);
    let xml = roxmltree::Document::parse(&svg).unwrap();
    // then
    for (kind, count) in [
        ("state-choice", 1),
        ("state-junction", 1),
        ("state-fork-join", 2),
        ("state-transition", 8),
    ] {
        assert_eq!(
            xml.descendants()
                .filter(|n| n.attribute("data-kind") == Some(kind))
                .count(),
            count,
            "{kind}"
        );
    }
}

#[test]
fn should_route_named_composite_history_and_primary_region_escape_without_losing_regions() {
    // given
    for body in [
        "state A\nA --> Missing[H] : restore\nMissing[H*] --> B : resume",
        "state Outside\nstate S {\nA --> Outside\n--\nstate B\n}",
        "state S {\nstate A\n--\nstate N {\nstate B\n}\nstate C\nN[H] --> C\n}",
        "state S {\nstate N {\nstate A\n}\n}\nstate Outside\nOutside --> S.N[H] : restore",
    ] {
        // when
        let svg = draw(body);
        // then
        assert!(usvg::Tree::from_str(&svg, &usvg::Options::default()).is_ok());
        assert!(svg.contains("state-transition"));
        if body.contains("--\n") {
            assert_eq!(svg.matches("data-kind=\"state-region\"").count(), 2);
        }
    }
}

#[test]
fn should_reject_cross_region_references_without_creating_shadow_nodes() {
    // given
    for body in [
        "state S {\nstate A\n--\nstate B\nA --> B\n}",
        "state S {\nstate A\nA --> B\n--\nstate B\n}",
        "state S {\n--\nstate B\n}\nB --> C",
        "state Outside\nstate S {\nstate A\n--\nB --> Outside\n}",
        "state S {\nstate A\n--\nstate A\n}",
    ] {
        // when
        let result = render(
            &format!("@startuml\n{body}\n@enduml"),
            &RenderOptions {
                language: Language::PlantUml,
                ..Default::default()
            },
        );
        // then
        assert!(result.is_err(), "accepted {body}");
    }
}

#[test]
fn should_preserve_mixed_empty_nested_regions_and_permissive_pseudostate_degrees() {
    // given
    for body in [
        "state S {\n--\n||\n}",
        "state S {\nstate N {\nA --> B\n||\nC --> D\n}\n--\nE --> F\n}",
        "state F <<fork>>\nF --> F\nstate J <<join>>\nJ --> A\nstate C <<choice>>\nC --> C\nstate X <<junction>>\nX --> X",
    ] {
        // when
        let svg = draw(body);
        // then
        assert_eq!(svg, draw(body));
        assert!(usvg::Tree::from_str(&svg, &usvg::Options::default()).is_ok());
    }
}

#[test]
fn should_render_exact_original_region_history_and_pseudostate_topologies() {
    // given
    for (source, edges, regions) in [
        (
            include_str!("../../fixtures/plantuml/stateregions.puml"),
            6,
            3,
        ),
        (
            include_str!("../../fixtures/plantuml/statehistory.puml"),
            2,
            0,
        ),
        (
            include_str!("../../fixtures/plantuml/statepseudos.puml"),
            8,
            0,
        ),
        (
            include_str!("../../fixtures/plantuml/stateadvanced.puml"),
            8,
            2,
        ),
    ] {
        // when
        let result = render(
            source,
            &RenderOptions {
                language: Language::PlantUml,
                ..Default::default()
            },
        )
        .unwrap();
        let xml = roxmltree::Document::parse(&result.svg).unwrap();
        // then
        assert_eq!(result.family, "state");
        assert_eq!(
            xml.descendants()
                .filter(|n| n.attribute("data-kind") == Some("state-transition"))
                .count(),
            edges
        );
        assert_eq!(
            xml.descendants()
                .filter(|n| n.attribute("data-kind") == Some("state-region"))
                .count(),
            regions
        );
        assert!(usvg::Tree::from_str(&result.svg, &usvg::Options::default()).is_ok());
    }
}

#[test]
fn should_preserve_named_start_end_identity_and_implicit_history_family() {
    // given
    for body in [
        "state I <<start>>\nstate F <<end>>\n[*] --> I\nI --> F\nF --> [*]",
        "[H] --> A\nA --> [H*]",
        "A --> S[H]\nS[H*] --> B",
    ] {
        // when
        let svg = draw(body);
        // then
        assert!(svg.contains("state-transition"));
        if body.starts_with("state I") {
            assert_eq!(svg.matches("data-kind=\"state-initial\"").count(), 2);
            assert_eq!(svg.matches("data-kind=\"state-final\"").count(), 2);
        }
    }
}

#[test]
fn should_bound_extended_state_source_scopes_work_and_output() {
    // given
    let source = include_str!("../../fixtures/plantuml/stateadvanced.puml");
    for resource in ["source", "items", "depth", "work", "svg"] {
        let mut options = RenderOptions {
            language: Language::PlantUml,
            ..Default::default()
        };
        match resource {
            "source" => options.limits.source_bytes = 16,
            "items" => options.limits.model_items = 3,
            "depth" => options.limits.nesting_depth = 0,
            "work" => options.limits.layout_work = 30,
            _ => options.limits.svg_bytes = 128,
        }
        // when
        let result = render(source, &options);
        // then
        assert!(result.is_err(), "unbounded {resource}");
    }
}
