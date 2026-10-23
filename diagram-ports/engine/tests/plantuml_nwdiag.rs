use diagram_engine::{Language, RenderOptions, render};

fn options() -> RenderOptions {
    RenderOptions {
        language: Language::PlantUml,
        ..Default::default()
    }
}
fn input(body: &str) -> String {
    format!("@startnwdiag\nnwdiag {{\n{body}\n}}\n@endnwdiag")
}
fn svg(body: &str) -> String {
    let output = render(&input(body), &options()).unwrap();
    assert_eq!(output.family, "nwdiag");
    usvg::Tree::from_str(&output.svg, &usvg::Options::default()).unwrap();
    output.svg
}

#[test]
fn should_render_one_global_server_and_exact_owned_memberships() {
    // given
    let body = "network edge {\naddress = \"198.51.100.0/24\";\nrouter [address=\"198.51.100.1\",description=\"Edge router\"];\nweb;\n}\nnetwork core {\naddress=\"10.23.0.0/24\";\nrouter [address=\"10.23.0.1\"];\nstore;\n}";
    // when
    let output = svg(body);
    let xml = roxmltree::Document::parse(&output).unwrap();
    let nodes: Vec<_> = xml
        .descendants()
        .filter(|n| n.attribute("data-kind") == Some("nwdiag-server"))
        .collect();
    let edges: Vec<_> = xml
        .descendants()
        .filter(|n| n.attribute("data-kind") == Some("nwdiag-membership"))
        .map(|n| {
            (
                n.attribute("data-owner").unwrap(),
                n.attribute("data-target").unwrap(),
            )
        })
        .collect();
    // then
    assert_eq!(nodes.len(), 3);
    assert_eq!(
        nodes
            .iter()
            .map(|n| n.attribute("id").unwrap())
            .collect::<Vec<_>>(),
        ["nwdiag-server-0", "nwdiag-server-1", "nwdiag-server-2"]
    );
    assert_eq!(
        edges,
        [
            ("nwdiag-network-0", "nwdiag-server-0"),
            ("nwdiag-network-0", "nwdiag-server-1"),
            ("nwdiag-network-1", "nwdiag-server-0"),
            ("nwdiag-network-1", "nwdiag-server-2")
        ]
    );
    assert_eq!(
        xml.descendants()
            .filter(|n| n.has_tag_name("text") && n.text() == Some("Edge router"))
            .count(),
        1
    );
    for text in [
        "198.51.100.0/24",
        "198.51.100.1",
        "10.23.0.0/24",
        "10.23.0.1",
    ] {
        assert!(output.contains(text));
    }
}

#[test]
fn should_update_nonempty_membership_addresses_and_preserve_empty_updates() {
    // given
    let body = "network edge {\nr [address=old];\nr;\nr [address=new];\nr [address=\"\"];\n}\nnetwork core {\nr [address=other];\n}";
    // when
    let output = svg(body);
    let xml = roxmltree::Document::parse(&output).unwrap();
    // then
    assert_eq!(
        xml.descendants()
            .filter(|n| n.attribute("data-kind") == Some("nwdiag-membership"))
            .count(),
        2
    );
    assert_eq!(
        xml.descendants()
            .filter(|n| n.attribute("data-kind") == Some("nwdiag-server"))
            .count(),
        1
    );
    let addresses: Vec<_> = xml
        .descendants()
        .filter(|n| n.attribute("data-kind") == Some("nwdiag-interface"))
        .map(|n| n.attribute("data-name").unwrap())
        .collect();
    assert_eq!(addresses, ["new", "other"]);
}

#[test]
fn should_keep_top_level_group_references_separate_from_network_memberships() {
    // given
    let body = "network edge {\na;\nb;\n}\ngroup internal {\ndescription=\"Storage zone\";\ncolor=\"#CCEEFF\";\nb;\n}";
    // when
    let output = svg(body);
    let xml = roxmltree::Document::parse(&output).unwrap();
    // then
    assert_eq!(
        xml.descendants()
            .filter(|n| n.attribute("data-kind") == Some("nwdiag-membership"))
            .count(),
        2
    );
    assert_eq!(
        xml.descendants()
            .filter(|n| n.attribute("data-kind") == Some("nwdiag-group-member"))
            .map(|n| (
                n.attribute("data-owner").unwrap(),
                n.attribute("data-target").unwrap()
            ))
            .collect::<Vec<_>>(),
        [("nwdiag-group-0", "nwdiag-server-1")]
    );
    assert!(output.contains("Storage zone"));
    assert!(output.contains("#cceeff"));
}

#[test]
fn should_paint_only_description_as_group_header_and_support_network_groups() {
    // given
    let body = "network net {\ngroup hidden_name {\na;\n}\nb;\n}";
    // when
    let output = svg(body);
    let xml = roxmltree::Document::parse(&output).unwrap();
    // then
    assert!(
        !xml.descendants()
            .any(|n| n.has_tag_name("text") && n.text() == Some("hidden_name"))
    );
    assert_eq!(
        xml.descendants()
            .filter(|n| n.attribute("data-kind") == Some("nwdiag-membership"))
            .count(),
        2
    );
}

#[test]
fn should_reject_unconsumed_malformed_duplicate_or_unsupported_properties() {
    // given
    for statement in [
        "a [address=x garbage]",
        "a [address=x address=y]",
        "a [address=x,]",
        "a [,address=x]",
        "a [address=x,,description=y]",
        "a [address=x,address=y]",
        "a [unknown=x]",
        "a [shape=cloud]",
        "a [width=full]",
        "a [color=bogus]",
        "a [address=\"x\"trailing]",
        "a [] trailing",
        "a [address=\"unterminated]",
        "width=42",
        "unknown=value",
        "a -- b",
    ] {
        // when
        let result = render(
            &input(&format!("network net {{\n{statement}\n}}")),
            &options(),
        );
        // then
        assert!(result.is_err(), "accepted {statement:?}");
    }
}

#[test]
fn should_reject_nested_or_duplicate_blocks_standalone_servers_and_extra_group_membership() {
    // given
    for body in [
        "network n { network x { a; } }",
        "group g { group h { a; } }",
        "group g { network n { a; } }",
        "network n { a; } network n { b; }",
        "network n { a; } group g { a; a; }",
        "network n { a; } group g { a; } group h { a; }",
        "a;",
        "group g { a; }",
        "network n { a; } }",
        "network n { a;",
        "network n-x { a; } group g-x { a; }",
    ] {
        // when
        let result = render(&input(body), &options());
        // then
        assert!(result.is_err(), "accepted {body:?}");
    }
}

#[test]
fn should_apply_prefix_styles_constants_and_chrome_without_rejecting_literal_keywords() {
    // given
    let source = "@startnwdiag\n!define NAME **Edge router**\nskinparam defaultFontName Liberation Sans\nskinparam defaultFontSize 24\nskinparam defaultFontColor #123456\ntitle: Network map\nnwdiag {\nnetwork n {\ndescription=\"title: literal bus\";\na [description=\"NAME\",address=\"https://example.test\",color=\"#ddeeff\"];\n}\ngroup g {\ndescription=\"right header: literal group\";\na;\n}\n}\n@endnwdiag";
    // when
    let output = render(source, &options()).unwrap().svg;
    // then
    for value in [
        "Network map",
        "Edge router",
        "#123456",
        "#ddeeff",
        "font-size=\"24\"",
        "font-weight=\"700\"",
        "title: literal bus",
        "right header: literal group",
        "https://example.test",
    ] {
        assert!(output.contains(value), "{value}");
    }
}

#[test]
fn should_reject_late_chrome_and_wrong_or_incomplete_wrappers() {
    // given
    for command in [
        "title: Late",
        "right header: Late",
        "skinparam defaultFontSize 30",
        "<style>",
    ] {
        // when
        let result = render(
            &input(&format!("network n {{\na;\n{command}\n}}")),
            &options(),
        );
        // then
        assert!(result.is_err(), "accepted {command}");
    }
    for source in [
        "@startnwdiag\nnetwork n {\na;\n}\n@endnwdiag",
        "@STARTNWDIAG\nnwdiag {\nnetwork n {\na;\n}\n}\n@ENDNWDIAG",
        "@startnwdiag file.svg\nnwdiag {}\n@endnwdiag",
        "@startnwdiag\nnwdiag {\nnetwork n {\na;\n}\n}\n@enduml",
    ] {
        assert!(render(source, &options()).is_err(), "accepted {source}");
    }
}

#[test]
fn should_bound_source_model_depth_work_output_and_cancellation() {
    // given
    for resource in ["source", "model", "depth", "work", "svg", "cancel"] {
        let mut config = options();
        match resource {
            "source" => config.limits.source_bytes = 16,
            "model" => config.limits.model_items = 2,
            "depth" => config.limits.nesting_depth = 1,
            "work" => config.limits.layout_work = 10,
            "svg" => config.limits.svg_bytes = 8192,
            _ => config.control.cancel(),
        }
        let body = format!(
            "network n {{\n{}\n}}",
            (0..10)
                .map(|i| format!("server{i} [address=address{i}];"))
                .collect::<Vec<_>>()
                .join("\n")
        );
        // when
        let error = render(&input(&body), &config).unwrap_err().to_string();
        // then
        let reason = match resource {
            "source" => "16-byte",
            "model" => "model items",
            "depth" => "nesting depth",
            "work" => "work units",
            "svg" => "SVG bytes",
            _ => "cancel",
        };
        assert!(error.contains(reason), "{resource}: {error}");
    }
}

#[test]
fn should_preserve_punctuated_global_identifiers_and_deterministic_sparse_buses() {
    // given
    let source = input(
        "network edge-net.1 {\nserver-a.1 [address=a];\n}\nnetwork middle {\nother;\n}\nnetwork last {\nserver-a.1 [address=b];\n}",
    );
    // when
    let a = render(&source, &options()).unwrap();
    let b = render(&source, &options()).unwrap();
    let xml = roxmltree::Document::parse(&a.svg).unwrap();
    // then
    assert_eq!(a.svg, b.svg);
    assert_eq!(
        xml.descendants()
            .filter(|n| n.attribute("data-kind") == Some("nwdiag-membership"))
            .count(),
        3
    );
    assert_eq!(
        xml.descendants()
            .filter(|n| n.attribute("data-kind") == Some("nwdiag-server"))
            .count(),
        2
    );
    assert!(a.svg.contains("server-a.1"));
}

#[test]
fn should_stop_nwdiag_after_an_inflight_measurement_cancels() {
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
    let mut config = options();
    let host = Arc::new(Cancel {
        control: config.control.clone(),
        calls: AtomicUsize::new(0),
    });
    config.text_measurement = TextMeasurementPolicy::host_display(
        TextMeasurementProfileIdentity::new(
            MeasurementProfileId::new("nwdiag-cancel").unwrap(),
            "1",
        )
        .unwrap(),
        host.clone(),
        [TextMeasurementPhase::Layout],
    );
    // when
    let error = render(&input("network n {\na;\nb;\n}"), &config)
        .unwrap_err()
        .to_string();
    // then
    assert!(error.contains("cancel"), "{error}");
    assert_eq!(host.calls.load(Ordering::Relaxed), 1);
}

#[test]
fn should_reject_every_shared_late_chrome_form_but_preserve_quoted_values() {
    // given
    let commands = [
        "title: Caption",
        "TiTlE : Caption",
        "right header: Caption",
        "left footer Caption",
        "center header : Caption",
        "legend: Caption",
    ];
    for command in commands {
        let source = format!("@startnwdiag\n{command}\nnwdiag {{network n {{a;}}}}\n@endnwdiag");
        assert!(render(&source, &options()).is_ok(), "prefix {command}");
        // when
        let error = render(
            &input(&format!("network n {{\na;\n{command}\n}}")),
            &options(),
        )
        .unwrap_err()
        .to_string();
        let output = svg(&format!("network n {{\na [description=\"{command}\"];\n}}"));
        // then
        assert!(
            error.contains("nwdiag styles/chrome must precede"),
            "{error}"
        );
        assert!(output.contains(command));
    }
}

#[test]
fn should_reject_unclosed_comments_and_preserve_inert_comment_and_property_text() {
    // given
    let source = "@startnwdiag\nnwdiag {\nnetwork n {\n' %getenv(SECRET) is an inert comment\na [description=\"it's /' literal https://example.test\"];\n}\n}\n@endnwdiag";
    // when
    let output = render(source, &options()).unwrap();
    let error = render(&format!("{source} /' unterminated"), &options())
        .unwrap_err()
        .to_string();
    // then
    assert!(output.svg.contains("literal https://example.test"));
    assert!(
        error.contains("unterminated nwdiag block comment"),
        "{error}"
    );
}

#[test]
fn should_render_exact_reference_fixtures_deterministically() {
    // given
    for source in [
        include_str!("../../fixtures/plantuml/nwdiagbase.puml"),
        include_str!("../../fixtures/plantuml/nwdiagshared.puml"),
        include_str!("../../fixtures/plantuml/nwdiagrepeated.puml"),
    ] {
        // when
        let a = render(source, &options()).unwrap();
        let b = render(source, &options()).unwrap();
        // then
        assert_eq!(a.family, "nwdiag");
        assert_eq!(a.svg, b.svg);
        let tree = usvg::Tree::from_str(&a.svg, &usvg::Options::default()).unwrap();
        assert!(tree.size().width() > 0. && tree.size().height() > 0.);
    }
}

#[test]
fn should_split_server_and_interface_comma_space_labels_as_nwdiag_lines() {
    // given
    let body = "network n {\na [description=\"First, Second\",address=\"one, two\"];\n}";
    // when
    let output = svg(body);
    let xml = roxmltree::Document::parse(&output).unwrap();
    let text = xml
        .descendants()
        .filter(|n| n.has_tag_name("text"))
        .filter_map(|n| n.text())
        .collect::<Vec<_>>();
    // then
    for line in ["First", "Second", "one", "two"] {
        assert!(text.contains(&line), "missing {line}: {text:?}");
    }
}

#[test]
fn should_distinguish_network_and_group_names_from_complete_block_declarations() {
    // given
    let body = "network n {\nnetwork [address=one];\ngroup [address=two];\n}\ngroup g {\nnetwork;\ngroup;\n}";
    // when
    let output = svg(body);
    let xml = roxmltree::Document::parse(&output).unwrap();
    // then
    assert_eq!(
        xml.descendants()
            .filter(|n| n.attribute("data-kind") == Some("nwdiag-server"))
            .map(|n| n.attribute("data-name").unwrap())
            .collect::<Vec<_>>(),
        ["network", "group"]
    );
}
