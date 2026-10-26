use diagram_engine::{Language, RenderOptions, render};
fn options() -> RenderOptions {
    RenderOptions {
        language: Language::PlantUml,
        ..Default::default()
    }
}
fn checked(body: &str) -> String {
    let source = format!("@startbpm\n{body}\n@endbpm");
    let result = render(&source, &options()).unwrap();
    assert_eq!(result.family, "bpm");
    assert_eq!(result.svg, render(&source, &options()).unwrap().svg);
    usvg::Tree::from_str(&result.svg, &usvg::Options::default()).unwrap();
    result.svg
}
#[test]
fn should_preserve_bpm_split_merge_and_continuation_instead_of_chaining_alternatives() {
    // given
    let body = ":Receive;\nnew branch\n:Check;\nelse\n:Retry;\nend branch\n:Archive;";
    // when
    let svg = checked(body);
    let xml = roxmltree::Document::parse(&svg).unwrap();
    let nodes = xml
        .descendants()
        .filter(|n| {
            matches!(
                n.attribute("data-kind"),
                Some("bpm-event" | "bpm-split" | "bpm-merge" | "bpm-start")
            )
        })
        .collect::<Vec<_>>();
    let id = |name| {
        nodes
            .iter()
            .find(|n| n.attribute("data-name") == Some(name))
            .unwrap()
            .attribute("id")
            .unwrap()
    };
    let split = nodes
        .iter()
        .find(|n| n.attribute("data-kind") == Some("bpm-split"))
        .unwrap()
        .attribute("id")
        .unwrap();
    let merge = nodes
        .iter()
        .find(|n| n.attribute("data-kind") == Some("bpm-merge"))
        .unwrap()
        .attribute("id")
        .unwrap();
    let edges = xml
        .descendants()
        .filter(|n| n.attribute("data-kind") == Some("bpm-edge"))
        .map(|n| {
            (
                n.attribute("data-name").unwrap(),
                n.attribute("data-target").unwrap(),
            )
        })
        .collect::<Vec<_>>();
    // then
    for edge in [
        (id("Receive"), split),
        (split, id("Check")),
        (split, id("Retry")),
        (id("Check"), merge),
        (id("Retry"), merge),
        (merge, id("Archive")),
    ] {
        assert!(edges.contains(&edge), "missing {edge:?}: {edges:?}");
    }
    assert!(!edges.contains(&(id("Check"), id("Retry"))));
    assert_eq!(edges.iter().filter(|e| e.0 == split).count(), 2);
    assert_eq!(edges.iter().filter(|e| e.1 == merge).count(), 2);
}
#[test]
fn should_keep_nested_bpm_branch_ownership_and_three_ordered_arms() {
    // given
    let body = "new branch\n:One;\nnew branch\n:A;\nelse\n:B;\nend branch\nelse\n:Two;\nelse\n:Three;\nend branch\n:Done;";
    // when
    let svg = checked(body);
    let xml = roxmltree::Document::parse(&svg).unwrap();
    // then
    for kind in ["bpm-split", "bpm-merge"] {
        assert_eq!(
            xml.descendants()
                .filter(|n| n.attribute("data-kind") == Some(kind))
                .count(),
            2
        );
    }
    let event = |name| {
        xml.descendants()
            .find(|n| {
                n.attribute("data-kind") == Some("bpm-event")
                    && n.attribute("data-name") == Some(name)
            })
            .unwrap()
    };
    assert_ne!(
        event("A").attribute("data-owner"),
        event("One").attribute("data-owner")
    );
    assert_eq!(
        event("One").attribute("data-owner"),
        event("Two").attribute("data-owner")
    );
}

#[test]
fn should_add_an_implicit_start_and_preserve_event_order_and_literal_punctuation() {
    // given
    let body = ":First;\n:literal ' slash // ; middle;\n:Last;";
    // when
    let svg = checked(body);
    let xml = roxmltree::Document::parse(&svg).unwrap();
    let start = xml
        .descendants()
        .find(|n| n.attribute("data-kind") == Some("bpm-start"))
        .unwrap();
    let events = xml
        .descendants()
        .filter(|n| n.attribute("data-kind") == Some("bpm-event"))
        .collect::<Vec<_>>();
    let edges = xml
        .descendants()
        .filter(|n| n.attribute("data-kind") == Some("bpm-edge"))
        .map(|n| {
            (
                n.attribute("data-name").unwrap(),
                n.attribute("data-target").unwrap(),
            )
        })
        .collect::<Vec<_>>();
    // then
    assert_eq!(
        events
            .iter()
            .map(|n| n.attribute("data-name").unwrap())
            .collect::<Vec<_>>(),
        ["First", "literal ' slash // ; middle", "Last"]
    );
    assert!(edges.contains(&(
        start.attribute("id").unwrap(),
        events[0].attribute("id").unwrap()
    )));
    for pair in events.windows(2) {
        assert!(edges.contains(&(
            pair[0].attribute("id").unwrap(),
            pair[1].attribute("id").unwrap()
        )));
    }
}

#[test]
fn should_reject_unbalanced_single_arm_empty_arm_and_unimplemented_commands_whole() {
    // given
    let invalid = [
        "else",
        "end branch",
        "new branch\n:A;",
        "new branch\n:A;\nend branch",
        "new branch\nelse\n:B;\nend branch",
        ":A;\ngoto x",
        ":A;\nresume x",
        ":A;\nx:<+>",
        "start\n:A;\nstop",
        "if (x) then (yes)\n:A;\nendif",
        ":missing semicolon",
        ":A;\nend",
        ":A;\nnew branch suffix\n:B;\nelse\n:C;\nend branch",
    ];
    // when
    let results = invalid.map(|body| render(&format!("@startbpm\n{body}\n@endbpm"), &options()));
    // then
    for (body, result) in invalid.into_iter().zip(results) {
        assert!(result.is_err(), "{body}");
    }
    for body in [
        "new branch\n:A;\nend branch",
        "new branch\nelse\n:B;\nend branch",
        "goto x",
        "resume x",
        "x:<+>",
        "start",
    ] {
        assert!(
            matches!(
                render(&format!("@startbpm\n{body}\n@endbpm"), &options()),
                Err(diagram_engine::RenderError::UnsupportedFeature(_))
            ),
            "{body}"
        );
    }
}

#[test]
fn should_admit_depth64_and_reject_the_next_branch_opener_before_retention() {
    // given
    let nested = |depth| {
        format!(
            "{}:Leaf;\n{}",
            "new branch\n".repeat(depth),
            "else\n:Other;\nend branch\n".repeat(depth)
        )
    };
    // when
    let accepted = checked(&nested(64));
    let error = render(&format!("@startbpm\n{}\n@endbpm", nested(65)), &options()).unwrap_err();
    // then
    assert!(accepted.contains("bpm-merge"));
    assert!(
        error
            .to_string()
            .contains("nesting depth value 65 exceeds limit 64"),
        "{error}"
    );
}

#[test]
fn should_bound_synthetic_nodes_edges_layout_work_output_and_cancellation() {
    // given
    let source = "@startbpm\n:A;\nnew branch\n:B;\nelse\n:C;\nend branch\n:D;\n@endbpm";
    let mut cases = Vec::new();
    let mut items = options();
    items.limits.model_items = 8;
    cases.push(items);
    let mut work = options();
    work.limits.layout_work = 100;
    cases.push(work);
    let mut depth = options();
    depth.limits.nesting_depth = 1;
    cases.push(depth);
    let mut output = options();
    output.limits.svg_bytes = 512;
    cases.push(output);
    let mut source_bound = options();
    source_bound.limits.source_bytes = source.len() - 1;
    cases.push(source_bound);
    let cancelled = options();
    cancelled.control.cancel();
    cases.push(cancelled);
    // when
    let results=cases.iter().enumerate().map(|(i,config)|if i==2 {render("@startbpm\nnew branch\nnew branch\n:A;\nelse\n:B;\nend branch\nelse\n:C;\nend branch\n@endbpm",config)}else{render(source,config)}).collect::<Vec<_>>();
    // then
    assert!(results.iter().all(Result::is_err), "{results:?}");
}

#[test]
fn should_admit_long_event_sets_with_exact_source_quota_and_bound_display_text() {
    // given
    let body = ":Event;\n".repeat(200);
    let source = format!("@startbpm\n{body}@endbpm");
    let mut exact = options();
    exact.limits.source_bytes = source.len();
    let large = format!("@startbpm\n:{};\n@endbpm", "x".repeat(16_385));
    // when
    let accepted = render(&source, &exact);
    let rejected = render(&large, &options()).unwrap_err();
    // then
    assert!(accepted.is_ok(), "{accepted:?}");
    assert!(
        rejected
            .to_string()
            .contains("label bytes value 16385 exceeds limit 16384"),
        "{rejected}"
    );
}

#[test]
fn should_measure_literal_event_data_once_and_preserve_expansion_error_origin() {
    // given
    let source = "@startbpm\n!define X Ready\n:X;\n@endbpm";
    let malformed = "@startbpm\n!define X new branch\nX\n:A;\nend branch\n@endbpm";
    // when
    let svg = render(source, &options()).unwrap().svg;
    let error = render(malformed, &options()).unwrap_err();
    // then
    assert!(svg.contains("Ready"));
    assert!(error.to_string().contains("definition"), "{error}");
}

#[test]
fn should_observe_cancellation_from_a_bpm_measurement_callback() {
    use diagram_engine::{
        HostMeasurementResult, HostTextMeasurementRequest, HostTextMeasurer, MeasurementProfileId,
        TextMeasurementPhase, TextMeasurementPolicy, TextMeasurementProfileIdentity,
    };
    struct Cancel(diagram_engine::RenderControl);
    impl HostTextMeasurer for Cancel {
        fn measure(&self, _request: HostTextMeasurementRequest<'_>) -> HostMeasurementResult {
            self.0.cancel();
            Ok(None)
        }
    }
    // given
    let mut config = options();
    config.text_measurement = TextMeasurementPolicy::host_display(
        TextMeasurementProfileIdentity::new(MeasurementProfileId::new("bpm-cancel").unwrap(), "1")
            .unwrap(),
        std::sync::Arc::new(Cancel(config.control.clone())),
        [TextMeasurementPhase::Layout],
    );
    // when
    let result = render("@startbpm\n:Measure;\n@endbpm", &config);
    // then
    assert!(
        matches!(result, Err(diagram_engine::RenderError::Cancelled(_))),
        "{result:?}"
    );
}
