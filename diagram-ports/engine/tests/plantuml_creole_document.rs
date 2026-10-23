use diagram_engine::{Language, RenderOptions, render};

fn options() -> RenderOptions {
    RenderOptions {
        language: Language::PlantUml,
        ..Default::default()
    }
}

#[test]
fn should_render_a_standalone_document_with_real_nested_styles_and_restored_runs() {
    // given
    let source = "@startcreole\n**Heavy //both// restored** plain\n__Paid__ --old--\n@endcreole";
    // when
    let result = render(source, &options()).unwrap();
    let xml = roxmltree::Document::parse(&result.svg).unwrap();
    let runs: Vec<_> = xml
        .descendants()
        .filter(|n| n.has_tag_name("text"))
        .collect();
    // then
    assert_eq!(result.family, "creole");
    assert!(runs.iter().any(|n| n.text() == Some("both")
        && n.attribute("font-weight") == Some("700")
        && n.attribute("font-style") == Some("italic")));
    assert!(runs.iter().any(|n| n.text() == Some(" restored")
        && n.attribute("font-weight") == Some("700")
        && n.attribute("font-style") == Some("normal")));
    assert!(
        runs.iter()
            .any(|n| n.text() == Some(" plain") && n.attribute("font-weight") == Some("400"))
    );
    assert!(
        runs.iter()
            .any(|n| n.attribute("text-decoration") == Some("underline"))
    );
    assert!(
        runs.iter()
            .any(|n| n.attribute("text-decoration") == Some("line-through"))
    );
    assert!(
        runs.iter()
            .all(|n| n.attribute("font-family") == Some("serif")
                && n.attribute("font-size") == Some("14"))
    );
    usvg::Tree::from_str(&result.svg, &usvg::Options::default()).unwrap();
}

#[test]
fn should_preserve_physical_lines_and_each_blank_paragraph_height() {
    // given
    let short = "@startcreole\nFirst\nSecond\n@endcreole";
    let blank = "@startcreole\nFirst\n\n\nSecond\n@endcreole";
    // when
    let first = render(short, &options()).unwrap();
    let second = render(blank, &options()).unwrap();
    let ys = |svg: &str| {
        let xml = roxmltree::Document::parse(svg).unwrap();
        xml.descendants()
            .filter(|n| n.has_tag_name("text") && matches!(n.text(), Some("First" | "Second")))
            .map(|n| n.attribute("y").unwrap().parse::<f64>().unwrap())
            .collect::<Vec<_>>()
    };
    // then
    let a = ys(&first.svg);
    let b = ys(&second.svg);
    assert_eq!(a.len(), 2);
    assert_eq!(b.len(), 2);
    assert!(b[1] - b[0] >= 3. * (a[1] - a[0]));
    let height = |svg: &str| {
        usvg::Tree::from_str(svg, &usvg::Options::default())
            .unwrap()
            .size()
            .height()
    };
    assert!(height(&second.svg) > height(&first.svg));
}

#[test]
fn should_reject_unsupported_document_blocks_html_resources_and_embedded_syntax() {
    // given
    let bodies = [
        "| table |",
        "* list",
        "# list",
        "= heading",
        "----",
        "~~wave~~",
        "\"\"mono\"\"",
        "<code>\nraw\n</code>",
        "{{uml\nA -> B\n}}",
        "<html>raw</html>",
        "<div>raw</div>",
        "<br>",
        "<svg><path/></svg>",
        "[[https://example.test link]]",
        "<img:test>",
        "<math>x</math>",
        "<latex>x</latex>",
        "<font:serif>x</font>",
        "<size:20>x</size>",
        "<style>\nroot {\n}\n</style>",
        "**<img:test>**",
        "!define X value\nX",
        "!include secret",
        "**cross //wrong** end//",
        "<b>open",
        "a\\tbad",
    ];
    // when / then
    for body in bodies {
        let result = render(&format!("@startcreole\n{body}\n@endcreole"), &options());
        assert!(result.is_err(), "admitted {body:?}");
    }
    for source in [
        "@startCREOLE\ntext\n@endCREOLE",
        "@startcreole output.svg\ntext\n@endcreole",
        "@startcreole\ntext\n@enduml",
        "@startcreole\ntext\n@endcreole\ntrailing",
    ] {
        assert!(render(source, &options()).is_err(), "admitted {source:?}");
    }
}

#[test]
fn should_decode_escapes_once_and_keep_raw_spaces_comments_angles_and_pua() {
    // given
    let source = "\u{feff}@startcreole\r\n  ~**literal~** \\nline \\\\n \\~ <literal> \u{e000}\r\n' apostrophe /' untouched\r\n~<img:literal>\r\n@endcreole\r\n";
    // when
    let result = render(source, &options()).unwrap();
    let xml = roxmltree::Document::parse(&result.svg).unwrap();
    let texts: Vec<_> = xml
        .descendants()
        .filter(|n| n.has_tag_name("text"))
        .map(|n| n.text().unwrap_or(""))
        .collect();
    // then
    assert!(texts.contains(&"  **literal** "));
    assert!(texts.contains(&"line \\n ~ <literal> \u{e000}"));
    assert!(texts.contains(&"' apostrophe /' untouched"));
    assert!(texts.contains(&"<img:literal>"));
    assert_eq!(result.svg, render(source, &options()).unwrap().svg);
}

#[test]
fn should_bound_document_source_paragraph_lines_styles_items_work_and_svg() {
    // given
    let source = "@startcreole\n<b><i><u>nested</u></i></b>\n\nnext\n@endcreole";
    let mut bytes = options();
    bytes.limits.source_bytes = source.len() - 1;
    let mut items = options();
    items.limits.model_items = 2;
    let mut depth = options();
    depth.limits.nesting_depth = 2;
    let mut work = options();
    work.limits.layout_work = 20;
    let mut svg = options();
    svg.limits.svg_bytes = 512;
    let cancelled = options();
    cancelled.control.cancel();
    // when / then
    for configured in [bytes, items, depth, work, svg, cancelled] {
        assert!(render(source, &configured).is_err());
    }
    for text in [
        "x".repeat(16 * 1024 + 1),
        "part\n".repeat(4000),
        "x\n\n".repeat(5000),
        "**x** ".repeat(1400),
    ] {
        assert!(render(&format!("@startcreole\n{text}\n@endcreole"), &options()).is_err());
    }
    let mut shallow = options();
    shallow.limits.nesting_depth = 1;
    let flat = format!("@startcreole\n{}@endcreole", "line\n\n".repeat(80));
    assert!(render(&flat, &shallow).is_ok());
}

#[derive(Clone, Debug)]
struct Query {
    text: String,
    width: f64,
    weight: String,
    slant: String,
}
struct Recorder(std::sync::Arc<std::sync::Mutex<Vec<Query>>>);
impl diagram_engine::HostTextMeasurer for Recorder {
    fn measure(
        &self,
        request: diagram_engine::HostTextMeasurementRequest<'_>,
    ) -> diagram_engine::HostMeasurementResult {
        assert_eq!(request.style.font_family.as_deref(), Some("serif"));
        assert_eq!(request.style.font_size, 14.);
        let weight = request.style.font_weight.clone().unwrap();
        let width = request.text.chars().count() as f64 * if weight == "700" { 12. } else { 8. };
        self.0.lock().unwrap().push(Query {
            text: request.text.into(),
            width,
            weight,
            slant: request.style.font_style.clone().unwrap(),
        });
        Ok(Some(diagram_engine::HostTextMeasurement::Metrics(
            diagram_engine::TextMetrics {
                width,
                height: 23.,
                line_count: 1,
            },
        )))
    }
}
fn recorded() -> (RenderOptions, std::sync::Arc<std::sync::Mutex<Vec<Query>>>) {
    let calls = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let mut config = options();
    config.text_measurement = diagram_engine::TextMeasurementPolicy::host_display(
        diagram_engine::TextMeasurementProfileIdentity::new(
            diagram_engine::MeasurementProfileId::new("creole-document-contract").unwrap(),
            "1",
        )
        .unwrap(),
        std::sync::Arc::new(Recorder(calls.clone())),
        [diagram_engine::TextMeasurementPhase::Layout],
    );
    (config, calls)
}

#[test]
fn should_measure_the_exact_painted_segments_cache_complete_styles_and_contain_every_run() {
    // given
    let (config, calls) = recorded();
    let source = "@startcreole\n**bold //both//** plain\n\n**bold //both//** plain\n\n__under --cut--__\n@endcreole";
    // when
    let result = render(source, &config).unwrap();
    let xml = roxmltree::Document::parse(&result.svg).unwrap();
    let tree = usvg::Tree::from_str(&result.svg, &usvg::Options::default()).unwrap();
    let queries = calls.lock().unwrap();
    // then
    assert_eq!(queries.len(), 5);
    assert!(
        queries
            .iter()
            .any(|q| q.text == "both" && q.weight == "700" && q.slant == "italic")
    );
    assert!(
        queries
            .iter()
            .any(|q| q.text == " plain" && q.weight == "400" && q.slant == "normal")
    );
    let mut painted = 0;
    for node in xml
        .descendants()
        .filter(|n| n.has_tag_name("text") && n.text().is_some_and(|s| !s.is_empty()))
    {
        let query = queries
            .iter()
            .find(|q| Some(q.text.as_str()) == node.text())
            .unwrap();
        let x: f64 = node.attribute("x").unwrap().parse().unwrap();
        let y: f64 = node.attribute("y").unwrap().parse().unwrap();
        assert!(x >= 0. && x + query.width <= f64::from(tree.size().width()) + 0.01);
        assert!(y - 23. >= 0. && y + 4. <= f64::from(tree.size().height()) + 0.01);
        assert_eq!(node.attribute("font-weight"), Some(query.weight.as_str()));
        assert_eq!(node.attribute("font-style"), Some(query.slant.as_str()));
        assert_eq!(node.attribute("fill"), Some("#000000"));
        painted += 1;
    }
    assert_eq!(painted, 8);
    assert!(
        xml.descendants()
            .any(|n| n.attribute("text-decoration") == Some("underline line-through"))
    );
}

#[test]
fn should_enforce_aggregate_document_measurement_limit_before_the_4097th_callback() {
    // given
    let (mut config, calls) = recorded();
    config.limits.model_items = 100_000;
    config.limits.layout_work = 100_000_000;
    let mut body = String::from("@startcreole\n");
    for index in 0..4097 {
        body.push_str(&format!("line{index}\n\n"));
    }
    body.push_str("@endcreole");
    // when
    let failure = render(&body, &config).unwrap_err();
    // then
    assert!(
        failure
            .to_string()
            .contains("measurement requests value 4097"),
        "{failure}"
    );
    assert_eq!(calls.lock().unwrap().len(), 4096);
}

#[test]
fn should_stop_document_measurement_after_a_host_callback_cancels_the_job() {
    struct Canceller(
        diagram_engine::RenderControl,
        std::sync::Arc<std::sync::atomic::AtomicUsize>,
    );
    impl diagram_engine::HostTextMeasurer for Canceller {
        fn measure(
            &self,
            _: diagram_engine::HostTextMeasurementRequest<'_>,
        ) -> diagram_engine::HostMeasurementResult {
            self.1.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            self.0.cancel();
            Ok(None)
        }
    }
    // given
    let mut config = options();
    let calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    config.text_measurement = diagram_engine::TextMeasurementPolicy::host_display(
        diagram_engine::TextMeasurementProfileIdentity::new(
            diagram_engine::MeasurementProfileId::new("creole-document-cancel").unwrap(),
            "1",
        )
        .unwrap(),
        std::sync::Arc::new(Canceller(config.control.clone(), calls.clone())),
        [diagram_engine::TextMeasurementPhase::Layout],
    );
    // when
    let failure = render(
        "@startcreole\n**first** //later//\n\nlast\n@endcreole",
        &config,
    )
    .unwrap_err();
    // then
    assert!(matches!(failure, diagram_engine::RenderError::Cancelled(_)));
    assert_eq!(calls.load(std::sync::atomic::Ordering::Relaxed), 1);
}
