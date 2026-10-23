use diagram_engine::{Language, RenderOptions, render};

fn options() -> RenderOptions {
    RenderOptions {
        language: Language::PlantUml,
        ..Default::default()
    }
}

fn source(body: &str) -> String {
    format!("@startuml\n{body}\n@enduml")
}

fn svg(body: &str) -> String {
    let result = render(&source(body), &options()).unwrap();
    assert_eq!(result.family, "timing");
    usvg::Tree::from_str(&result.svg, &usvg::Options::default()).unwrap();
    result.svg
}

#[test]
fn should_draw_distinct_robust_steps_and_concise_ribbons() {
    // given
    let body = "robust \"Valve\" as V\nconcise C\n@0\nV is Closed\nC is Waiting\n@6\nV is Open\nC is Running\n@15\nV is Closed\nC is Done\nV@6 <-> @+9 : open interval\nV@6 -> C@8 : confirm\n@20";
    // when
    let result = svg(body);
    let xml = roxmltree::Document::parse(&result).unwrap();
    // then
    for (kind, count) in [
        ("timing-robust", 1),
        ("timing-concise", 1),
        ("timing-change", 6),
        ("timing-constraint", 1),
        ("timing-message", 1),
    ] {
        assert_eq!(
            xml.descendants()
                .filter(|n| n.attribute("data-kind") == Some(kind))
                .count(),
            count,
            "{kind}"
        );
    }
    let robust = xml
        .descendants()
        .find(|n| n.attribute("data-kind") == Some("timing-robust"))
        .unwrap();
    assert!(robust.descendants().any(|n| n.has_tag_name("polyline")));
    let concise = xml
        .descendants()
        .find(|n| n.attribute("data-kind") == Some("timing-concise"))
        .unwrap();
    assert_eq!(
        concise
            .descendants()
            .filter(|n| n.has_tag_name("polygon"))
            .count(),
        3
    );
}

#[test]
fn should_keep_player_cursor_separate_from_time_cursor() {
    // given
    let body = "robust V\nconcise C\nV is Initial\nV is Replaced\n@-5\nC is Waiting\n@V\n+5 is Open\n@C\n+10 is Running\nV is Closed\n@20";
    // when
    let result = svg(body);
    let xml = roxmltree::Document::parse(&result).unwrap();
    let names: Vec<_> = xml
        .descendants()
        .filter(|n| n.attribute("data-kind") == Some("timing-change"))
        .map(|n| {
            (
                n.attribute("data-owner").unwrap(),
                n.attribute("data-name").unwrap(),
            )
        })
        .collect();
    // then
    assert_eq!(
        names,
        [
            ("V", "initial:Replaced"),
            ("V", "0:Open"),
            ("V", "10:Closed"),
            ("C", "-5:Waiting"),
            ("C", "10:Running")
        ]
    );
}

#[test]
fn should_keep_first_change_at_equal_numeric_time() {
    // given
    let body = "robust V\n@10\nV is First\n@-2\nV is Earlier\n@10\nV is Discarded\n@+0\nV is AlsoDiscarded\n@20";
    // when
    let result = svg(body);
    let xml = roxmltree::Document::parse(&result).unwrap();
    // then
    let names: Vec<_> = xml
        .descendants()
        .filter(|n| n.attribute("data-kind") == Some("timing-change"))
        .map(|n| n.attribute("data-name").unwrap())
        .collect();
    assert_eq!(names, ["-2:Earlier", "10:First"]);
}

#[test]
fn should_resolve_constraint_end_relative_to_constraint_start() {
    // given
    let body = "robust V\n@0\nV is A\n@15\nV@6 <-> @+9 : interval\nV is B\n@30";
    // when
    let result = svg(body);
    let xml = roxmltree::Document::parse(&result).unwrap();
    // then
    let constraint = xml
        .descendants()
        .find(|n| n.attribute("data-kind") == Some("timing-constraint"))
        .unwrap();
    assert_eq!(constraint.attribute("data-owner"), Some("V"));
    assert_eq!(constraint.attribute("data-name"), Some("6:15"));
    assert!(
        xml.descendants()
            .any(|n| n.attribute("data-name") == Some("15:B"))
    );
}

#[test]
fn should_resolve_message_offsets_from_current_time() {
    // given
    let body =
        "robust V\nconcise C\n@0\nV is A\nC is B\n@15\nV@6 -> C@+9 : signal\nV -> C : now\n@30";
    // when
    let result = svg(body);
    let xml = roxmltree::Document::parse(&result).unwrap();
    // then
    let messages: Vec<_> = xml
        .descendants()
        .filter(|n| n.attribute("data-kind") == Some("timing-message"))
        .collect();
    assert_eq!(messages[0].attribute("data-name"), Some("6:24"));
    assert_eq!(messages[1].attribute("data-name"), Some("15:15"));
    assert_eq!(messages[0].attribute("data-owner"), Some("V"));
    assert_eq!(messages[0].attribute("data-target"), Some("C"));
}

#[test]
fn should_bound_sparse_ticks_and_render_deterministically() {
    // given
    let body = "robust V\n@-1000000000\nV is A\n@1000000000\nV is B";
    // when
    let a = svg(body);
    let b = svg(body);
    let xml = roxmltree::Document::parse(&a).unwrap();
    // then
    assert_eq!(a, b);
    assert!(
        xml.descendants()
            .filter(|n| n.attribute("data-kind") == Some("timing-tick"))
            .count()
            <= 12
    );
}

#[test]
fn should_reject_unsupported_timing_syntax_with_offending_text() {
    // given
    for row in [
        "@1.25",
        "@0 as :origin",
        "@:origin+5",
        "@1000000001",
        "@+-1000000001",
        "clock CLK with period 10",
        "analog A",
        "binary B",
        "compact robust R",
        "mode compact",
        "hide time-axis",
        "ticks on multiple 5",
        "use date format yyyy",
        "V is {hidden}",
        "V is \"{?}\"",
        "V is A #red",
        "V is A : comment",
        "robust V",
        "note top of V : note",
        "V@0 -[#red]> V@1 : x",
        "@missing",
        "unknown is A",
    ] {
        let input = source(&format!("robust V\n@0\nV is A\n@10\n{row}"));
        // when
        let error = render(&input, &options()).unwrap_err().to_string();
        // then
        assert!(error.replace("\\\"", "\"").contains(row), "{row}: {error}");
    }
}

#[test]
fn should_reject_connections_outside_explicit_axis_or_without_signal() {
    // given
    for row in [
        "V@0 -> C@11 : outside",
        "V@-1 <-> @4 : outside",
        "V@0 -> C@0 : undefined",
        "V@0 -> Missing@1 : unknown",
    ] {
        let input = source(&format!(
            "robust V\nconcise C\n@0\nV is A\n@5\nC is B\n@10\n{row}"
        ));
        // when
        let result = render(&input, &options());
        // then
        assert!(result.is_err(), "accepted {row}");
    }
}

#[test]
fn should_apply_shared_fonts_chrome_and_constants_to_timing() {
    // given
    let body = "!define READY \"**Ready**\"\nskinparam defaultFontName Liberation Sans\nskinparam defaultFontSize 30\ntitle Timing title\nfooter Timing footer\nrobust V\n@0\nV is READY\n@5";
    // when
    let result = svg(body);
    // then
    assert!(result.contains("Timing title") && result.contains("Timing footer"));
    assert!(result.contains("font-size=\"30\"") && result.contains("Liberation Sans"));
}

#[test]
fn should_treat_arrows_and_resource_looking_text_inside_quoted_states_as_data() {
    // given
    let body = "robust startPlayer\n@0\nstartPlayer is \"literal -> and <-> https://example.test/x\"\n@startPlayer\n+5 is \"next: /tmp/data\"\n@10";
    // when
    let result = svg(body);
    // then
    assert!(result.contains("literal -&gt; and &lt;-&gt; https://example.test/x"));
    assert!(result.contains("next: /tmp/data"));
}

#[test]
fn should_accept_time_before_declaration_without_changing_family_precedence() {
    // given
    let body = "@-5\nrobust V\nV is A\n@+10\nV is B";
    // when
    let result = svg(body);
    // then
    assert!(result.contains("-5:A") && result.contains("5:B"));
    for body in [
        "participant V\nrobust R\n@0\nV is A",
        "class V\nrobust R\n@0\nR is A",
        "state V\nrobust R\n@0\nR is A",
        "start\nrobust R\n@0\nR is A\nstop",
    ] {
        assert!(render(&source(body), &options()).is_err());
    }
}

#[test]
fn should_obey_custom_limits_and_cancellation_for_timing() {
    // given
    let input = source("robust V\n@0\nV is A\n@5\nV is B\n@10");
    let mut configurations = Vec::new();
    let mut source_limit = options();
    source_limit.limits.source_bytes = 32;
    configurations.push(source_limit);
    let mut model_limit = options();
    model_limit.limits.model_items = 8;
    configurations.push(model_limit);
    let mut work_limit = options();
    work_limit.limits.layout_work = 100;
    configurations.push(work_limit);
    let mut svg_limit = options();
    svg_limit.limits.svg_bytes = 1024;
    configurations.push(svg_limit);
    let mut depth_limit = options();
    depth_limit.limits.nesting_depth = 0;
    configurations.push(depth_limit);
    let cancelled = options();
    cancelled.control.cancel();
    configurations.push(cancelled);
    // when
    let errors: Vec<_> = configurations
        .iter()
        .map(|config| render(&input, config).unwrap_err().to_string())
        .collect();
    // then
    assert!(errors[0].contains("32-byte"));
    assert!(errors[1].contains("model items"));
    assert!(errors[2].contains("work units"));
    assert!(errors[3].contains("SVG bytes"));
    assert!(errors[4].contains("MaxModelNestingDepth"), "{}", errors[4]);
    assert!(errors[5].contains("cancel"));
}

#[test]
fn should_preserve_grammar_words_in_connection_captions() {
    // given
    let body = "robust V\nconcise C\n@0\nV is A\nC is B\n@10\nV@0 <-> @5 : interval is bounded -> here\nV@2 -> C@8 : result is A <-> B";
    // when
    let result = svg(body);
    // then
    assert!(result.contains("interval is bounded -&gt; here"));
    assert!(result.contains("result is A &lt;-&gt; B"));
}

#[test]
fn should_reserve_message_caption_width_with_only_one_time_mark() {
    // given
    let body = "robust V\nconcise C\n@0\nV is A\nC is B\nV -> C : long caption needs actual measured width even without an axis span";
    // when
    let result = svg(body);
    // then
    assert!(result.contains("long caption needs actual measured width"));
}

#[test]
fn should_accept_whitespace_between_timing_grammar_tokens() {
    // given
    let body = "robust\t\"Valve\" as V\nconcise   C\n@0\nV  is\tClosed\nC\tis  Waiting\n@V\n+5\tis  Open\n@10";
    // when
    let result = svg(body);
    // then
    assert!(result.contains("5:Open") && result.contains("0:Waiting"));
}

#[test]
fn should_admit_exact_original_timing_reference_fixtures() {
    // given
    for (input, changes, constraints, messages) in [
        (
            include_str!("../../fixtures/plantuml/timingbase.puml"),
            3,
            0,
            0,
        ),
        (
            include_str!("../../fixtures/plantuml/timingdual.puml"),
            6,
            1,
            1,
        ),
        (
            include_str!("../../fixtures/plantuml/timinglane.puml"),
            6,
            0,
            0,
        ),
    ] {
        // when
        let output = render(input, &options()).unwrap();
        let xml = roxmltree::Document::parse(&output.svg).unwrap();
        // then
        assert_eq!(output.family, "timing");
        assert_eq!(output.svg, render(input, &options()).unwrap().svg);
        for (kind, count) in [
            ("timing-change", changes),
            ("timing-constraint", constraints),
            ("timing-message", messages),
        ] {
            assert_eq!(
                xml.descendants()
                    .filter(|node| node.attribute("data-kind") == Some(kind))
                    .count(),
                count
            );
        }
        usvg::Tree::from_str(&output.svg, &usvg::Options::default()).unwrap();
    }
}

#[test]
fn should_cancel_timing_during_actual_host_measurement() {
    use diagram_engine::{
        HostMeasurementResult, HostTextMeasurement, HostTextMeasurementRequest, HostTextMeasurer,
        MeasurementProfileId, TextMeasurementPhase, TextMeasurementPolicy,
        TextMeasurementProfileIdentity, TextMetrics,
    };
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    struct Canceller {
        control: diagram_engine::RenderControl,
        calls: Arc<AtomicUsize>,
    }
    impl HostTextMeasurer for Canceller {
        fn measure(&self, _: HostTextMeasurementRequest<'_>) -> HostMeasurementResult {
            self.calls.fetch_add(1, Ordering::Relaxed);
            self.control.cancel();
            Ok(Some(HostTextMeasurement::Metrics(TextMetrics {
                width: 30.,
                height: 20.,
                line_count: 1,
            })))
        }
    }
    // given
    let mut options = options();
    let calls = Arc::new(AtomicUsize::new(0));
    options.text_measurement = TextMeasurementPolicy::host_display(
        TextMeasurementProfileIdentity::new(
            MeasurementProfileId::new("timing-cancellation").unwrap(),
            "1",
        )
        .unwrap(),
        Arc::new(Canceller {
            control: options.control.clone(),
            calls: calls.clone(),
        }),
        [TextMeasurementPhase::Layout],
    );
    // when
    let error = render(
        include_str!("../../fixtures/plantuml/timingdual.puml"),
        &options,
    )
    .unwrap_err();
    // then
    assert!(error.to_string().contains("cancel"));
    assert_eq!(calls.load(Ordering::Relaxed), 1);
}

#[test]
fn should_keep_ordinary_special_looking_state_names_literal() {
    // given
    let body = "robust V\n@0\nV is hidden\n@1\nV is \"-\"\n@2\nV is \"...\"\n@3\nV is \"?\"\n@4";
    // when
    let result = svg(body);
    // then
    for state in ["0:hidden", "1:-", "2:...", "3:?"] {
        assert!(result.contains(state));
    }
}

#[test]
fn should_validate_discarded_equal_time_state_markup_before_ignoring_its_change() {
    // given
    let input = source("robust V\n@0\nV is First\nV is \"<img:forbidden.png>\"\n@10");
    // when
    let result = render(&input, &options());
    // then
    assert!(
        result.is_err(),
        "unsupported equal-time state markup was silently ignored"
    );
}

#[test]
fn should_reject_empty_track_display_at_its_declaration() {
    // given
    let input = source("robust \"\" as V\n@0\nV is A\n@5");
    // when
    let result = render(&input, &options());
    // then
    assert!(result.is_err(), "empty declared display was accepted");
}

#[test]
fn should_bound_repeated_timing_owner_metadata_before_retention() {
    // given
    let code = "A".repeat(40_000);
    let mut body = format!("robust \"Track\" as {code}\n");
    for time in 0..100 {
        body.push_str(&format!("{time} is A\n"));
    }
    // when
    let result = render(&source(&body), &options());
    // then
    let error = match result {
        Err(error) => error.to_string(),
        Ok(_) => panic!("repeated owner metadata escaped the shared text budget"),
    };
    assert!(error.contains("expanded model text bytes"), "{error}");
}
