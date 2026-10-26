use diagram_engine::{Language, RenderOptions, render};

fn options() -> RenderOptions {
    RenderOptions {
        language: Language::PlantUml,
        ..Default::default()
    }
}
fn input(body: &str) -> String {
    format!("@startgantt\n{body}\n@endgantt")
}
fn svg(body: &str) -> String {
    let result = render(&input(body), &options()).unwrap();
    assert_eq!(result.family, "gantt");
    usvg::Tree::from_str(&result.svg, &usvg::Options::default()).unwrap();
    result.svg
}
const ALIASES: &str = "Project starts 2026-11-02\n[Draft brief] as [draft] lasts 2 days\n[Build package] as [build] lasts 3 days\n[build] starts at [draft]'s end\n[Review checkpoint] as [review] happens at [build]'s end\n[Publish checkpoint] happens 1 day after [build]'s end";

#[test]
fn should_resolve_aliases_in_source_order_with_distinct_displays() {
    // given
    let body = ALIASES;
    // when
    let output = svg(body);
    let xml = roxmltree::Document::parse(&output).unwrap();
    let tasks: Vec<_> = xml
        .descendants()
        .filter(|n| {
            matches!(
                n.attribute("data-kind"),
                Some("gantt-bar" | "gantt-milestone")
            )
        })
        .collect();
    // then
    assert_eq!(tasks.len(), 4);
    assert_eq!(
        tasks
            .iter()
            .map(|n| n.attribute("data-owner").unwrap())
            .collect::<Vec<_>>(),
        ["draft", "build", "review", "Publish checkpoint"]
    );
    for display in [
        "Draft brief",
        "Build package",
        "Review checkpoint",
        "Publish checkpoint",
    ] {
        assert!(output.contains(display));
    }
}

#[test]
fn should_use_exclusive_successor_and_last_occupied_milestone_dates() {
    // given
    let body = ALIASES;
    // when
    let output = svg(body);
    let xml = roxmltree::Document::parse(&output).unwrap();
    // then
    for (owner, date) in [
        ("draft", "2026-11-02/2026-11-04"),
        ("build", "2026-11-04/2026-11-07"),
        ("review", "2026-11-06"),
        ("Publish checkpoint", "2026-11-07"),
    ] {
        let task = xml
            .descendants()
            .find(|n| {
                n.attribute("data-owner") == Some(owner)
                    && matches!(
                        n.attribute("data-kind"),
                        Some("gantt-bar" | "gantt-milestone")
                    )
            })
            .unwrap();
        assert_eq!(task.attribute("data-name"), Some(date));
    }
    let edge = xml
        .descendants()
        .find(|n| n.attribute("data-kind") == Some("gantt-dependency"))
        .unwrap();
    assert_eq!(edge.attribute("data-owner"), Some("draft"));
    assert_eq!(edge.attribute("data-target"), Some("build"));
    assert_eq!(edge.attribute("data-name"), Some("end:0"));
}

#[test]
fn should_draw_duration_widths_diamonds_and_owned_routes() {
    // given
    let body = ALIASES;
    // when
    let output = svg(body);
    let xml = roxmltree::Document::parse(&output).unwrap();
    let bars: Vec<_> = xml
        .descendants()
        .filter(|n| n.attribute("data-kind") == Some("gantt-bar"))
        .map(|n| n.descendants().find(|s| s.has_tag_name("rect")).unwrap())
        .collect();
    let number = |node: roxmltree::Node<'_, '_>, key: &str| {
        node.attribute(key).unwrap().parse::<f64>().unwrap()
    };
    // then
    assert!((number(bars[1], "width") / number(bars[0], "width") - 1.5).abs() < 0.0001);
    assert!(
        (number(bars[0], "x") + number(bars[0], "width") - number(bars[1], "x")).abs() < 0.0001
    );
    for diamond in xml
        .descendants()
        .filter(|n| n.attribute("data-kind") == Some("gantt-milestone"))
    {
        assert!(diamond.descendants().any(|n| n.has_tag_name("polygon")));
    }
    assert!(
        xml.descendants()
            .filter(|n| n.attribute("data-kind") == Some("gantt-dependency"))
            .all(|n| n.descendants().any(|s| s.has_tag_name("polyline")))
    );
}

#[test]
fn should_cross_leap_days_and_keep_seven_days_open() {
    // given
    for (date, end) in [("2028-02-28", "2028-03-02"), ("2026-10-23", "2026-10-26")] {
        // when
        let output = svg(&format!(
            "Project starts {date}\n[A] requires 3 days\n[B] starts at [A]'s end"
        ));
        // then
        assert!(output.contains(&format!("{date}/{end}")));
        assert!(output.contains(&format!("{end}/")));
    }
}

#[test]
fn should_apply_start_end_and_before_after_offsets() {
    // given
    let body = "Project starts 2026-11-02\n[A] lasts 3 days\n[B] starts 1 day before [A]'s end\n[C] starts 2 days after [A]'s start\n[M] happens 1 day before [A]'s end\n[N] happens 1 day after [A]'s start";
    // when
    let output = svg(body);
    let xml = roxmltree::Document::parse(&output).unwrap();
    // then
    for (id, dates) in [
        ("B", "2026-11-04/2026-11-05"),
        ("C", "2026-11-04/2026-11-05"),
        ("M", "2026-11-03"),
        ("N", "2026-11-03"),
    ] {
        assert!(
            xml.descendants()
                .any(|n| n.attribute("data-owner") == Some(id)
                    && n.attribute("data-name") == Some(dates))
        );
    }
}

#[test]
fn should_reject_missing_late_conflicting_unknown_and_cyclic_schedules() {
    // given
    for (body, reason) in [
        ("[A] lasts 2 days", "Project starts"),
        (
            "Project starts 2026-11-02\n[A] lasts 2 days\nProject starts 2026-11-03",
            "project",
        ),
        (
            "Project starts 2026-11-02\n[A] lasts 2 days\n[A] lasts 3 days",
            "duration",
        ),
        (
            "Project starts 2026-11-02\n[A] starts 2026-11-03\n[A] starts 2026-11-04",
            "start",
        ),
        (
            "Project starts 2026-11-02\n[B] starts at [missing]'s end",
            "missing",
        ),
        (
            "Project starts 2026-11-02\n[Display] as [A] lasts 2 days\n[Other] as [A] starts 2026-11-03",
            "alias",
        ),
        (
            "Project starts 2026-11-02\n[A] lasts 2 days\n[B] lasts 3 days\n[B] starts at [A]'s end\n[A] starts at [B]'s end",
            "cycl",
        ),
        (
            "Project starts 2026-11-02\n[A] lasts 2 days\n[B] starts at [A]'s end\n[A] starts 2026-11-03",
            "frozen",
        ),
        (
            "Project starts 2026-11-02\n[M] happens 2026-11-03\n[B] starts at [M]'s end",
            "milestone",
        ),
    ] {
        // when
        let error = render(&input(body), &options()).unwrap_err().to_string();
        // then
        assert!(error.contains(reason), "{body}: {error}");
    }
}

#[test]
fn should_reject_invalid_dates_and_unimplemented_rules_with_offending_input() {
    // given
    for row in [
        "Project starts 2026-02-29",
        "Project starts 2028-02-30",
        "Project starts 0000-01-01",
        "Project starts 99999-01-01",
        "saturday are closed",
        "[A] lasts 0 days",
        "[A] lasts -1 days",
        "[A] lasts 1.5 days",
        "[A] lasts 9999999999999999999999999999999 days",
        "[A] lasts 2 weeks",
        "[A] lasts 3 hours",
        "[A] on {Bob} lasts 2 days",
        "[A] is 50% complete",
        "printscale weekly",
        "today is 2026-11-03",
        "[A] is colored in red",
        "[A] lasts 2 days and starts 2026-11-03",
    ] {
        let body = format!("Project starts 2026-11-02\n{row}");
        // when
        let error = render(&input(&body), &options()).unwrap_err().to_string();
        // then
        assert!(error.contains(row), "{row}: {error}");
    }
}

#[test]
fn should_bound_sparse_calendar_ticks_and_reject_excessive_spans() {
    // given
    let body = "Project starts 2026-01-01\n[A] lasts 7000 days";
    // when
    let a = svg(body);
    let b = svg(body);
    let xml = roxmltree::Document::parse(&a).unwrap();
    // then
    assert_eq!(a, b);
    assert!(
        xml.descendants()
            .filter(|n| n.attribute("data-kind") == Some("gantt-tick"))
            .count()
            <= 14
    );
    assert!(
        render(
            &input("Project starts 2026-01-01\n[A] starts 2099-01-01"),
            &options()
        )
        .is_err()
    );
}

#[test]
fn should_apply_shared_styles_chrome_constants_and_literal_task_text() {
    // given
    let body = "!define DAYS 2\nskinparam defaultFontName Liberation Sans\nskinparam defaultFontSize 30\ntitle Schedule title\nfooter Schedule footer\nProject starts 2026-11-02\n[**Deploy** https://example.test/x /tmp/data] as [A] lasts DAYS days";
    // when
    let output = svg(body);
    // then
    for text in [
        "Schedule title",
        "Schedule footer",
        "Deploy",
        "https://example.test/x /tmp/data",
        "font-size=\"30\"",
        "Liberation Sans",
    ] {
        assert!(output.contains(text), "{text}");
    }
}

#[test]
fn should_obey_custom_gantt_limits_and_cancellation() {
    // given
    let body = input(ALIASES);
    let mut configs = Vec::new();
    let mut model = options();
    model.limits.model_items = 5;
    configs.push(model);
    let mut work = options();
    work.limits.layout_work = 100;
    configs.push(work);
    let mut bytes = options();
    bytes.limits.svg_bytes = 4096;
    configs.push(bytes);
    let cancelled = options();
    cancelled.control.cancel();
    configs.push(cancelled);
    let mut source = options();
    source.limits.source_bytes = 32;
    configs.push(source);
    let mut depth = options();
    depth.limits.nesting_depth = 0;
    configs.push(depth);
    // when
    let errors: Vec<_> = configs
        .iter()
        .map(|c| render(&body, c).unwrap_err().to_string())
        .collect();
    // then
    for (error, kind) in errors.iter().zip([
        "model items",
        "work units",
        "SVG bytes",
        "cancel",
        "32-byte",
        "MaxModelNestingDepth",
    ]) {
        assert!(error.contains(kind), "{error}");
    }
}

#[test]
fn should_admit_exact_original_gantt_reference_fixtures() {
    // given
    for input in [
        include_str!("../../fixtures/plantuml/ganttbase.puml"),
        include_str!("../../fixtures/plantuml/ganttalias.puml"),
        include_str!("../../fixtures/plantuml/ganttleap.puml"),
    ] {
        // when
        let output = render(input, &options()).unwrap();
        // then
        assert_eq!(output.family, "gantt");
        assert_eq!(output.svg, render(input, &options()).unwrap().svg);
        usvg::Tree::from_str(&output.svg, &usvg::Options::default()).unwrap();
    }
}

#[test]
fn should_validate_real_project_dates_before_constructing_tasks() {
    // given
    for date in [
        "2026-02-29",
        "1900-02-29",
        "2028-02-30",
        "2026-04-31",
        "0000-01-01",
        "2026-13-01",
        "2026-00-10",
        "2026-11-00",
        "2026-1-01",
        "2026/11/02",
    ] {
        // when
        let error = render(
            &input(&format!("Project starts {date}\n[A] lasts 1 day")),
            &options(),
        )
        .unwrap_err()
        .to_string();
        // then
        assert!(error.contains(date), "{error}");
    }
    assert!(
        render(
            &input("Project starts 2000-02-29\n[A] lasts 1 day"),
            &options()
        )
        .is_ok()
    );
}

#[test]
fn should_reject_incomplete_predecessors_wrong_wrappers_and_late_preambles() {
    // given
    for source in [
        "@startgantt\nProject starts 2026-11-02\n[A] starts 2026-11-03\n[B] starts at [A]'s end\n@endgantt",
        "@startgantt\nProject starts 2026-11-02\n[A] lasts 2 days\n@enduml",
        "@startgantt\nProject starts 2026-11-02\n[A] lasts 2 days\n@endgantt\n[A] lasts 3 days",
        "@startproject\nProject starts 2026-11-02\n[A] lasts 2 days\n@endproject",
        "@startgantt\nProject starts 2026-11-02\n[A] lasts 2 days\nskinparam defaultFontSize 30\n@endgantt",
        "@startgantt\nProject starts 2026-11-02\n[A] lasts 2 days\n[A] starts 2026-11-01\n@endgantt",
    ] {
        // when
        let result = render(source, &options());
        // then
        assert!(result.is_err(), "accepted {source}");
    }
}

#[test]
fn should_consume_only_admitted_start_complement_grammar() {
    // given
    for row in [
        "[B] starts on [A]'s end",
        "[B] starts at 1 day after [A]'s end",
        "[B] starts at at [A]'s end",
    ] {
        let source = input(&format!(
            "Project starts 2026-11-02\n[A] lasts 2 days\n{row}"
        ));
        // when
        let result = render(&source, &options());
        // then
        assert!(result.is_err(), "accepted {row}");
    }
}

#[test]
fn should_bound_authored_identity_and_display_labels_before_measurement() {
    // given
    for name in ["x".repeat(16385), "x".repeat(16000)] {
        let mut config = options();
        if name.len() == 16000 {
            config.limits.model_items = 5;
        }
        let body = format!("Project starts 2026-11-02\n[{name}] lasts 2 days");
        // when
        let result = render(&input(&body), &config);
        // then
        assert!(result.is_err());
    }
}

#[test]
fn should_cancel_gantt_at_the_first_real_host_measurement() {
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
            MeasurementProfileId::new("gantt-cancellation").unwrap(),
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
    let result = render(&input(ALIASES), &options);
    // then
    assert!(result.unwrap_err().to_string().contains("cancel"));
    assert_eq!(calls.load(Ordering::Relaxed), 1);
}
#[test]
fn should_reject_unterminated_trailing_block_comment_after_gantt_wrapper() {
    // given
    let source = "@startgantt\nProject starts 2025-01-01\n[Task] lasts 1 day\n@endgantt\n";
    // when
    let invalid = render(&format!("{source}/' unterminated"), &options());
    let valid = render(&format!("{source}/' closed '/"), &options());
    // then
    assert!(matches!(
        invalid,
        Err(diagram_engine::RenderError::InvalidSource(_))
    ));
    assert!(valid.is_ok(), "{valid:?}");
}
