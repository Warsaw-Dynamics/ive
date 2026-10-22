use diagram_engine::{Language, RenderError, RenderOptions, render};

fn options() -> RenderOptions {
    RenderOptions {
        language: Language::PlantUml,
        ..Default::default()
    }
}

fn diagram(body: &str) -> String {
    format!("@startuml\n{body}\n@enduml")
}

#[test]
fn should_bound_expanded_namespace_text_before_retaining_keys() {
    // given
    let mut body = String::new();
    for i in 0..20 {
        body.push_str(&format!("namespace n{i}{} {{\n", "x".repeat(80)));
    }
    for i in 0..100 {
        body.push_str(&format!("class Leaf{i}\n"));
    }
    body.push_str(&"}\n".repeat(20));
    let source = diagram(&body);
    let mut config = options();
    config.limits.source_bytes = source.len();
    // when
    let actual = render(&source, &config);
    // then
    assert!(
        actual
            .unwrap_err()
            .to_string()
            .contains("expanded model text")
    );
}

#[test]
fn should_keep_ambiguous_actor_database_declarations_in_sequence_context() {
    // given
    let source = diagram("actor User\ndatabase Store\nUser -> Store : Read");
    // when
    let actual = render(&source, &options()).unwrap();
    // then
    assert_eq!(actual.family, "sequence");
    assert!(actual.svg.contains("data-kind=\"lifeline\""));
}

#[test]
fn should_keep_sequence_keywords_as_class_member_text() {
    // given
    let source = diagram("class Control {\nloop\nactivate\n}");
    // when
    let actual = render(&source, &options()).unwrap();
    // then
    assert_eq!(actual.family, "class");
    assert!(actual.svg.contains(">loop</text>"));
}

#[test]
fn should_reject_unimplemented_quoted_escape_instead_of_changing_label() {
    // given
    let source = diagram("participant \"A\\tB\" as A\nA -> B : Hello");
    // when
    let actual = render(&source, &options());
    // then
    assert!(matches!(actual, Err(RenderError::UnsupportedFeature(_))));
}

#[test]
fn should_draw_named_graph_notes_with_authored_links() {
    // given
    let source = diagram("class Account\nnote \"Check\\nidentity\" as N\nN .. Account : review");
    // when
    let result = render(&source, &options()).unwrap();
    let xml = roxmltree::Document::parse(&result.svg).unwrap();
    let note = xml
        .descendants()
        .find(|n| n.attribute("data-kind") == Some("note"))
        .unwrap();
    // then
    assert_eq!(result.family, "class");
    assert!(note.descendants().any(|n| n.has_tag_name("polygon")));
    assert!(note.descendants().any(|n| n.text() == Some("identity")));
    assert!(
        xml.descendants()
            .any(|n| n.attribute("data-kind") == Some("relation")
                && n.attribute("data-owner") == Some("n1")
                && n.attribute("data-target") == Some("n0"))
    );
}

#[test]
fn should_report_actual_host_success_and_invalid_answer_fallback() {
    use diagram_engine::{
        HostMeasurementResult, HostTextMeasurement, HostTextMeasurementRequest, HostTextMeasurer,
        MeasurementProfileId, TextMeasurementPhase, TextMeasurementPolicy,
        TextMeasurementProfileIdentity, TextMeasurementSource, TextMetrics,
    };
    struct PartialHost;
    impl HostTextMeasurer for PartialHost {
        fn measure(&self, request: HostTextMeasurementRequest<'_>) -> HostMeasurementResult {
            Ok(Some(HostTextMeasurement::Metrics(TextMetrics {
                width: if request.text == "native" {
                    55.
                } else {
                    f64::NAN
                },
                height: 20.,
                line_count: 1,
            })))
        }
    }
    // given
    let mut config = options();
    config.text_measurement = TextMeasurementPolicy::host_display(
        TextMeasurementProfileIdentity::new(
            MeasurementProfileId::new("partial-native-test").unwrap(),
            "1",
        )
        .unwrap(),
        std::sync::Arc::new(PartialHost),
        [TextMeasurementPhase::Layout],
    );
    // when
    let actual = render(&diagram("A -> B : native"), &config).unwrap();
    // then
    assert!(
        actual
            .measurement
            .entries()
            .iter()
            .any(|e| e.provenance().source == TextMeasurementSource::Host && e.count() > 0)
    );
    assert!(
        actual
            .measurement
            .entries()
            .iter()
            .any(|e| e.provenance().fallback_reason.is_some() && e.count() > 0)
    );
    assert!(
        actual
            .diagnostics
            .iter()
            .any(|d| d.contains("baseline") && d.contains("approximation"))
    );
}

#[test]
fn should_render_five_authorized_reference_sources() {
    // given
    let cases = [
        (
            include_str!("../../fixtures/plantuml/sequence.puml"),
            "sequence",
            &[
                "Browser",
                "Service",
                "Checkout",
                "Submit basket",
                "Check stock",
                "Accepted",
            ][..],
        ),
        (
            include_str!("../../fixtures/plantuml/class.puml"),
            "class",
            &[
                "Account",
                "Premium",
                "+owner : String",
                "+debit(amount)",
                "many",
                "records",
            ][..],
        ),
        (
            include_str!("../../fixtures/plantuml/object.puml"),
            "object",
            &[
                "basket : Cart",
                "line : Item",
                "total = 42",
                "sku = \"A7\"",
                "contains",
            ][..],
        ),
        (
            include_str!("../../fixtures/plantuml/deployment.puml"),
            "description",
            &[
                "Cluster",
                "Worker pool",
                "Gateway",
                "Worker",
                "Store",
                "dispatch",
                "save",
            ][..],
        ),
        (
            include_str!("../../fixtures/plantuml/usecase.puml"),
            "description",
            &["Shopper", "Store", "Place order", "Pay invoice", "includes"][..],
        ),
    ];
    for (source, family, labels) in cases {
        // when
        let actual = render(source, &options()).unwrap();
        let xml = roxmltree::Document::parse(&actual.svg).unwrap();
        let painted: Vec<_> = xml
            .descendants()
            .filter(|n| n.is_text())
            .filter_map(|n| n.text())
            .collect();
        // then
        assert_eq!(actual.family, family);
        assert_eq!(actual.language, Language::PlantUml);
        for label in labels {
            assert!(painted.contains(label), "missing {label:?} from {family}");
        }
        assert!(actual.layout_work_units > 0);
        assert!(!actual.measurement.entries().is_empty());
        assert!(usvg::Tree::from_str(&actual.svg, &usvg::Options::default()).is_ok());
    }
}

#[test]
fn should_reject_unsupported_syntax_with_a_source_location() {
    // given
    let cases = [
        "newpage",
        "skinparam foo bar",
        "!include secret.puml",
        "!define X Y",
        "<style>\nclass { Color red }\n</style>",
        "A -> B : hello\nunknown command",
        "& A -> B : parallel",
        "state S",
        "start\n:read;\nstop",
        "class A\nA -right-> B",
        "class A\nA::port --> B",
    ];
    for body in cases {
        // when
        let error = render(&diagram(body), &options()).unwrap_err().to_string();
        // then
        assert!(
            error.contains("PlantUML") && error.contains("line"),
            "{error}"
        );
        assert!(!error.contains("Mermaid"), "{error}");
    }
}

#[test]
fn should_reject_malformed_wrappers_comments_and_stacks() {
    // given
    let cases = [
        "@startuml\nA -> B\n@enduml\n@startuml\nA -> B\n@enduml",
        "@startuml\nA -> B",
        "@startuml\n/' unterminated\n@enduml",
        "@startuml\n@enduml",
        "@startjson\n{}\n@endjson",
        "@startuml\nalt test\nA -> B\n@enduml",
        "@startuml\ndeactivate A\n@enduml",
        "@startuml\nclass A {\n@enduml",
    ];
    for source in cases {
        // when
        let actual = render(source, &options());
        // then
        assert!(actual.is_err(), "accepted {source:?}");
    }
}

#[test]
fn should_keep_quoted_apostrophes_urls_and_xml_text() {
    // given
    let source = "\u{feff}@startuml\r\n' comment\r\nparticipant \"O'Brien & Ω\" as A\r\nparticipant B\r\n/' ignored /' text '/\r\nA -> B : https://example.test?a=1&b=<value>\r\n@enduml";
    // when
    let result = render(source, &options()).unwrap();
    let document = roxmltree::Document::parse(&result.svg).unwrap();
    let text: Vec<_> = document
        .descendants()
        .filter(|n| n.is_text())
        .filter_map(|n| n.text())
        .collect();
    // then
    assert!(text.contains(&"O'Brien & Ω"));
    assert!(text.contains(&"https://example.test?a=1&b=<value>"));
    assert!(!result.svg.contains("href="));
}

#[test]
fn should_enforce_every_custom_resource_limit() {
    // given
    let source = diagram(
        "participant A\nparticipant B\ngroup outer\ngroup inner\nA -> B : long label\nend\nend",
    );
    for resource in ["source", "svg", "items", "depth", "work"] {
        let mut opts = options();
        match resource {
            "source" => opts.limits.source_bytes = 8,
            "svg" => opts.limits.svg_bytes = 32,
            "items" => opts.limits.model_items = 1,
            "depth" => opts.limits.nesting_depth = 1,
            _ => opts.limits.layout_work = 1,
        }
        // when
        let result = render(&source, &opts);
        // then
        assert!(result.is_err(), "unbounded {resource}");
    }
}

#[test]
fn should_validate_limits_and_cancellation_before_parsing() {
    // given
    let mut invalid = options();
    invalid.limits.nesting_depth = 0;
    let cancelled = options();
    cancelled.control.cancel();
    // when
    let first = render(&diagram("A -> B"), &invalid);
    let second = render(&diagram("A -> B"), &cancelled);
    // then
    assert!(matches!(first, Err(RenderError::InvalidOptions(_))));
    assert!(matches!(second, Err(RenderError::Cancelled(_))));
}

#[test]
fn should_preserve_sequence_direction_frames_and_lifecycle() {
    // given
    let source = diagram(
        "participant A\nparticipant B\nalt valid\nA -> B : request\nactivate B\nB -> B : self\nelse invalid\nA <-- B : reply\nend\ndeactivate B\ncreate C\nB -> C : create\ndestroy C",
    );
    // when
    let rendered = render(&source, &options()).unwrap();
    let xml = roxmltree::Document::parse(&rendered.svg).unwrap();
    let messages: Vec<_> = xml
        .descendants()
        .filter(|n| n.attribute("data-kind") == Some("message"))
        .collect();
    // then
    assert_eq!(messages.len(), 4);
    assert_eq!(messages[0].attribute("data-owner"), Some("p0"));
    assert_eq!(messages[0].attribute("data-target"), Some("p1"));
    assert_eq!(
        messages[1].attribute("data-owner"),
        messages[1].attribute("data-target")
    );
    assert_eq!(messages[2].attribute("data-owner"), Some("p1"));
    assert_eq!(messages[2].attribute("data-target"), Some("p0"));
    for kind in ["frame", "activation", "destroy"] {
        assert!(
            xml.descendants()
                .any(|n| n.attribute("data-kind") == Some(kind)),
            "{kind}"
        );
    }
}

#[test]
fn should_preserve_parallel_self_cyclic_and_disconnected_uml_edges() {
    // given
    let source = diagram(
        "class A\nclass B\nclass C\nA --> B : one\nA ..> B : two\nB --> A : back\nA --> A : self",
    );
    // when
    let rendered = render(&source, &options()).unwrap();
    let xml = roxmltree::Document::parse(&rendered.svg).unwrap();
    // then
    assert_eq!(
        xml.descendants()
            .filter(|n| n.attribute("data-kind") == Some("relation"))
            .count(),
        4
    );
    assert_eq!(
        xml.descendants()
            .filter(|n| n.attribute("data-kind") == Some("class"))
            .count(),
        3
    );
}

#[test]
fn should_preserve_scoped_aliases_and_genuine_description_shapes() {
    // given
    let source = diagram(
        "package Left {\nclass A\n}\npackage Right {\nclass A\n}\nLeft.A --> Right.A : cross",
    );
    // when
    let rendered = render(&source, &options()).unwrap();
    let xml = roxmltree::Document::parse(&rendered.svg).unwrap();
    let edge = xml
        .descendants()
        .find(|n| n.attribute("data-kind") == Some("relation"))
        .unwrap();
    // then
    assert_ne!(edge.attribute("data-owner"), edge.attribute("data-target"));
    assert_eq!(
        xml.descendants()
            .filter(|n| n.attribute("data-kind") == Some("package"))
            .count(),
        2
    );
}

#[test]
fn should_choose_sequence_when_actors_have_explicit_participant_context() {
    // given
    let source = diagram("actor Buyer\nparticipant Service\nBuyer -> Service : request");
    // when
    let result = render(&source, &options()).unwrap();
    // then
    assert_eq!(result.family, "sequence");
    assert!(result.svg.contains("data-kind=\"lifeline\""));
}

#[test]
fn should_reject_implicit_names_containing_unimplemented_syntax() {
    // given
    let bodies = [
        "A -> B++ : invalid",
        "A -> B! : invalid",
        "A -> B #red",
        "A -> B : <color:red>styled</color>",
        "A -> B : **bold**",
        "class A\nA --> B::port",
        "class A\nA --> B <<stereotype>>",
        "!if false\n!include secret\n!endif",
    ];
    for body in bodies {
        // when
        let actual = render(&diagram(body), &options());
        // then
        assert!(actual.is_err(), "silently accepted {body:?}");
    }
}

#[test]
fn should_reject_contradictory_family_declarations() {
    // given
    let source = diagram("class A\ncomponent B\nA --> B");
    // when
    let actual = render(&source, &options());
    // then
    assert!(actual.is_err(), "allowmixing is not implemented");
}

#[test]
fn should_observe_cancellation_after_a_host_measurement() {
    use diagram_engine::{
        HostMeasurementResult, HostTextMeasurementRequest, HostTextMeasurer, TextMeasurementPolicy,
        TextMeasurementProfileIdentity,
    };
    use std::sync::Arc;
    struct Canceller(diagram_engine::RenderControl);
    impl HostTextMeasurer for Canceller {
        fn measure(&self, _: HostTextMeasurementRequest<'_>) -> HostMeasurementResult {
            self.0.cancel();
            Ok(None)
        }
    }
    // given
    let mut opts = options();
    opts.text_measurement = TextMeasurementPolicy::host_display(
        TextMeasurementProfileIdentity::new(
            diagram_engine::MeasurementProfileId::new("cancel-test").unwrap(),
            "1",
        )
        .unwrap(),
        Arc::new(Canceller(opts.control.clone())),
        [diagram_engine::TextMeasurementPhase::Layout],
    );
    // when
    let actual = render(&diagram("A -> B : hello"), &opts);
    // then
    assert!(matches!(actual, Err(RenderError::Cancelled(_))));
}

#[test]
fn should_reject_label_markup_without_corrupting_literal_angle_text() {
    // given
    let unsupported = [
        "//italic//",
        "<u>underline</u>",
        "<back:red>paint</back>",
        "[[https://example.test link]]",
        "<$icon>",
    ];
    for label in unsupported {
        // when
        let actual = render(&diagram(&format!("A -> B : {label}")), &options());
        // then
        assert!(actual.is_err(), "silently ignored {label:?}");
    }
}

#[test]
fn should_draw_distinct_uml_extremities_and_underlined_object_headers() {
    // given
    let source = diagram("object A\nobject B\nA o-- B\nA *-- B\nA <|-- B\nA ..|> B");
    // when
    let rendered = render(&source, &options()).unwrap();
    let xml = roxmltree::Document::parse(&rendered.svg).unwrap();
    // then
    assert_eq!(
        xml.descendants()
            .filter(|n| n.has_tag_name("polygon"))
            .count(),
        4
    );
    assert!(
        xml.descendants()
            .any(|n| n.has_tag_name("text") && n.attribute("text-decoration") == Some("underline"))
    );
}

#[test]
fn should_keep_nested_container_labels_clear_of_children() {
    // given
    let source = include_str!("../../fixtures/plantuml/deployment.puml");
    // when
    let rendered = render(source, &options()).unwrap();
    let xml = roxmltree::Document::parse(&rendered.svg).unwrap();
    let parent = xml
        .descendants()
        .find(|n| n.attribute("data-name") == Some("Cluster.Worker pool"))
        .unwrap();
    let child = xml
        .descendants()
        .find(|n| n.attribute("data-name") == Some("Cluster.Worker pool.Worker"))
        .unwrap();
    let parent_rect = parent.children().find(|n| n.has_tag_name("rect")).unwrap();
    let child_rect = child.children().find(|n| n.has_tag_name("rect")).unwrap();
    let number =
        |n: roxmltree::Node<'_, '_>, name| n.attribute(name).unwrap().parse::<f64>().unwrap();
    let label = parent
        .descendants()
        .find(|n| n.has_tag_name("text"))
        .unwrap();
    // then
    assert!(number(child_rect, "x") >= number(parent_rect, "x"));
    assert!(
        number(child_rect, "y") > number(label, "y") + 4.,
        "container caption overlaps child"
    );
    assert!(
        number(child_rect, "x") + number(child_rect, "width")
            <= number(parent_rect, "x") + number(parent_rect, "width")
    );
    assert!(
        number(child_rect, "y") + number(child_rect, "height")
            <= number(parent_rect, "y") + number(parent_rect, "height")
    );
}

#[test]
fn should_keep_command_prefixes_in_sequence_identifiers() {
    // given
    let source = diagram("titleClient -> scaleServer : hello");
    // when
    let actual = render(&source, &options()).unwrap();
    // then
    assert_eq!(actual.family, "sequence");
    assert!(actual.svg.contains("titleClient"));
}

#[test]
fn should_measure_italic_paint_with_italic_style_and_separate_cache_entries() {
    use diagram_engine::{
        HostMeasurementResult, HostTextMeasurementRequest, HostTextMeasurer, MeasurementProfileId,
        TextMeasurementPhase, TextMeasurementPolicy, TextMeasurementProfileIdentity,
    };
    use std::sync::{Arc, Mutex};
    struct Recorder(Arc<Mutex<Vec<(String, String)>>>);
    impl HostTextMeasurer for Recorder {
        fn measure(&self, request: HostTextMeasurementRequest<'_>) -> HostMeasurementResult {
            self.0.lock().unwrap().push((
                request.text.into(),
                request.style.font_style.clone().unwrap_or_default(),
            ));
            Ok(None)
        }
    }
    // given
    let calls = Arc::new(Mutex::new(Vec::new()));
    let mut opts = options();
    opts.text_measurement = TextMeasurementPolicy::host_display(
        TextMeasurementProfileIdentity::new(MeasurementProfileId::new("style-test").unwrap(), "1")
            .unwrap(),
        Arc::new(Recorder(calls.clone())),
        [TextMeasurementPhase::Layout],
    );
    let source = diagram(
        "class \"Same\" as A\nabstract class \"Same\" as B {\n{abstract} run()\n{static} count\n}\nA --> B",
    );
    // when
    let rendered = render(&source, &opts).unwrap();
    let recorded = calls.lock().unwrap();
    // then
    assert!(
        recorded
            .iter()
            .any(|(text, style)| text == "Same" && style == "italic")
    );
    assert!(
        recorded
            .iter()
            .any(|(text, style)| text == "Same" && style != "italic")
    );
    assert!(
        recorded
            .iter()
            .any(|(text, style)| text == "run()" && style == "italic")
    );
    assert!(rendered.svg.contains("text-decoration=\"underline\""));
}
