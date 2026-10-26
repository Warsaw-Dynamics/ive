use diagram_engine::{Language, RenderOptions, render};
fn options() -> RenderOptions {
    RenderOptions {
        language: Language::PlantUml,
        ..Default::default()
    }
}
fn checked(body: &str) -> String {
    let source = format!("@startflow\n{body}\n@endflow");
    let result = render(&source, &options()).unwrap();
    assert_eq!(result.family, "flow");
    assert_eq!(result.svg, render(&source, &options()).unwrap().svg);
    usvg::Tree::from_str(&result.svg, &usvg::Options::default()).unwrap();
    result.svg
}
fn rect(node: roxmltree::Node<'_, '_>) -> (f64, f64, f64, f64) {
    let r = node.children().find(|n| n.has_tag_name("rect")).unwrap();
    let n = |key| r.attribute(key).unwrap().parse::<f64>().unwrap();
    (n("x"), n("y"), n("width"), n("height"))
}
#[test]
fn should_preserve_default_south_and_ordered_cardinal_placement() {
    // given
    let body = "Read \"Read\"\nCheck \"Check\"\ne Store \"Store\"\nn North \"North\"\ne East \"East\"\nn High \"High\"\nw Left \"Left\"";
    // when
    let svg = checked(body);
    let xml = roxmltree::Document::parse(&svg).unwrap();
    let tile = |name| {
        rect(
            xml.descendants()
                .find(|n| {
                    n.attribute("data-kind") == Some("flow-tile")
                        && n.attribute("data-name") == Some(name)
                })
                .unwrap(),
        )
    };
    // then
    let center = |r: (f64, f64, f64, f64)| (r.0 + r.2 / 2., r.1 + r.3 / 2.);
    let (read, check, store) = (
        center(tile("Read")),
        center(tile("Check")),
        center(tile("Store")),
    );
    assert_eq!(read.0, check.0);
    assert!(check.1 > read.1);
    assert!(store.0 > check.0);
    assert_eq!(store.1, check.1);
}
#[test]
fn should_keep_last_tile_after_explicit_link_and_draw_red_circle_destination_markers() {
    // given
    let body = "A \"A\"\ne B \"B\"\nw A\ns C \"C\"";
    // when
    let svg = checked(body);
    let xml = roxmltree::Document::parse(&svg).unwrap();
    let tile = |name| {
        xml.descendants()
            .find(|n| {
                n.attribute("data-kind") == Some("flow-tile")
                    && n.attribute("data-name") == Some(name)
            })
            .unwrap()
    };
    let (a, b, c) = (tile("A"), tile("B"), tile("C"));
    let (b_box, c_box) = (rect(b), rect(c));
    // then
    assert_eq!(b_box.0 + b_box.2 / 2., c_box.0 + c_box.2 / 2.);
    assert!(c_box.1 > b_box.1);
    let edges = xml
        .descendants()
        .filter(|n| n.attribute("data-kind") == Some("flow-path"))
        .collect::<Vec<_>>();
    assert_eq!(edges.len(), 3);
    for (from, to) in [(a, b), (b, a), (b, c)] {
        assert!(
            edges
                .iter()
                .any(|n| n.attribute("data-owner") == from.attribute("id")
                    && n.attribute("data-target") == to.attribute("id"))
        );
    }
    for edge in edges {
        assert!(
            edge.descendants()
                .any(|n| n.has_tag_name("ellipse") && n.attribute("fill") == Some("#a80036"))
        );
        assert!(!edge.descendants().any(|n| n.has_tag_name("polygon")));
        assert!(
            edge.descendants()
                .any(|n| n.attribute("stroke") == Some("#a80036"))
        );
    }
}

#[test]
fn should_ignore_root_direction_and_keep_direction_words_as_contextual_tile_ids() {
    // given
    let body = "n n \"Root\"\ne e \"East\"\ns s \"South\"\nw w \"West\"";
    // when
    let svg = checked(body);
    let xml = roxmltree::Document::parse(&svg).unwrap();
    // then
    let tiles = xml
        .descendants()
        .filter(|n| n.attribute("data-kind") == Some("flow-tile"))
        .collect::<Vec<_>>();
    assert_eq!(tiles.len(), 4);
    assert!(
        tiles[0]
            .attribute("data-target")
            .unwrap()
            .contains("declared:n")
    );
    assert_eq!(tiles[0].attribute("data-name"), Some("n"));
}

#[test]
fn should_admit_clear_nonadjacent_west_side_path_without_advancing_the_cursor() {
    // given
    let body = "A \"A\"\ns D \"D\"\ne E \"E\"\ne F \"F\"\nn C \"C\"\nw A\nn G \"G\"";
    // when
    let svg = checked(body);
    let xml = roxmltree::Document::parse(&svg).unwrap();
    // then
    let tile = |name| {
        xml.descendants()
            .find(|n| {
                n.attribute("data-kind") == Some("flow-tile")
                    && n.attribute("data-name") == Some(name)
            })
            .unwrap()
    };
    let (a, c, g) = (tile("A"), tile("C"), tile("G"));
    assert!(
        xml.descendants()
            .any(|n| n.attribute("data-kind") == Some("flow-path")
                && n.attribute("data-owner") == c.attribute("id")
                && n.attribute("data-target") == a.attribute("id"))
    );
    assert!(
        xml.descendants()
            .any(|n| n.attribute("data-owner") == c.attribute("id")
                && n.attribute("data-target") == g.attribute("id"))
    );
}

#[test]
fn should_reject_duplicate_unknown_colliding_self_obstructed_and_unrouteable_forms_whole() {
    // given
    let bodies = [
        "A \"A\"\ne A \"duplicate\"",
        "Missing",
        "A \"A\"\ne Missing",
        "A \"A\"\ne B \"B\"\nw C \"collision\"",
        "A \"A\"\ne A",
        "A \"A\"\ne B \"B\"\ns C \"C\"\nw A",
        "A \"A\"\ne B \"B\"\ne C \"C\"\nw A",
        "A \"A\"\ns B \"B\"\nno B",
    ];
    // when
    let results = bodies.map(|body| render(&format!("@startflow\n{body}\n@endflow"), &options()));
    // then
    for (body, result) in bodies.into_iter().zip(results) {
        assert!(
            matches!(
                result,
                Err(diagram_engine::RenderError::UnsupportedFeature(_))
            ),
            "{body}: {result:?}"
        );
        if body.contains("Missing") {
            assert!(result.unwrap_err().to_string().contains("\"Missing\""));
        }
    }
}

#[test]
fn should_reject_malformed_quotes_gaps_suffixes_wrappers_and_unverified_commands() {
    // given
    let sources = [
        "@startflow\nA\"gap missing\"\n@endflow",
        "@startflow\nA \"unterminated\n@endflow",
        "@startflow\nA \"a\" junk\n@endflow",
        "@startflow\nA \"a\"\nstart\n@endflow",
        "@startflow\nA \"a\"\n@endbpm",
        "@startflow\n@endflow",
        "@startflow\nA \"tab\\tstop\"\n@endflow",
    ];
    // when
    let results = sources.map(|source| render(source, &options()));
    // then
    assert!(results.iter().all(Result::is_err), "{results:?}");
}

#[test]
fn should_keep_quoted_punctuation_and_constants_as_label_data_without_escape_reinterpretation() {
    // given
    let source = "@startflow\n!define X Ready\nA \"X ' // ; @endflow !include a\\b\"\ne B \"embedded \"quote\"\"\n@endflow";
    // when
    let svg = render(source, &options()).unwrap().svg;
    // then
    assert!(svg.contains("Ready"));
    assert!(svg.contains("flow-tile"));
    assert!(svg.contains("embedded"));
}

#[test]
fn should_preserve_long_syntax_ids_and_bound_display_labels_separately() {
    // given
    let alias = "a".repeat(16_385);
    let large = "x".repeat(16_385);
    // when
    let admitted = checked(&format!("{alias} \"Short\""));
    let rejected = render(&format!("@startflow\nA \"{large}\"\n@endflow"), &options()).unwrap_err();
    // then
    assert!(admitted.contains("Short"));
    assert!(
        rejected
            .to_string()
            .contains("label bytes value 16385 exceeds limit 16384"),
        "{rejected}"
    );
}

#[test]
fn should_bound_sparse_grid_model_work_output_source_and_cancellation() {
    // given
    let source = "@startflow\nA \"A\"\ne B \"B\"\ns C \"C\"\n@endflow";
    let mut configs = Vec::new();
    let mut items = options();
    items.limits.model_items = 2;
    configs.push(items);
    let mut work = options();
    work.limits.layout_work = 80;
    configs.push(work);
    let mut output = options();
    output.limits.svg_bytes = 512;
    configs.push(output);
    let mut source_bound = options();
    source_bound.limits.source_bytes = source.len() - 1;
    configs.push(source_bound);
    let cancelled = options();
    cancelled.control.cancel();
    configs.push(cancelled);
    // when
    let results = configs
        .iter()
        .map(|config| render(source, config))
        .collect::<Vec<_>>();
    let body = (0..1000)
        .map(|i| format!("n t{i:04} \"Tile\"\n"))
        .collect::<String>();
    let large = format!("@startflow\n{body}@endflow");
    let mut exact = options();
    exact.limits.source_bytes = large.len();
    exact.limits.nesting_depth = 1;
    let admitted = render(&large, &exact);
    // then
    assert!(results.iter().all(Result::is_err), "{results:?}");
    assert!(admitted.is_ok(), "{admitted:?}");
}

#[test]
fn should_measure_and_paint_serif14_and_observe_callback_cancellation() {
    use diagram_engine::{
        HostMeasurementResult, HostTextMeasurementRequest, HostTextMeasurer, MeasurementProfileId,
        TextMeasurementPhase, TextMeasurementPolicy, TextMeasurementProfileIdentity,
    };
    struct Cancel(diagram_engine::RenderControl);
    impl HostTextMeasurer for Cancel {
        fn measure(&self, request: HostTextMeasurementRequest<'_>) -> HostMeasurementResult {
            assert_eq!(request.style.font_family.as_deref(), Some("serif"));
            assert_eq!(request.style.font_size, 14.);
            self.0.cancel();
            Ok(None)
        }
    }
    // given
    let mut config = options();
    config.text_measurement = TextMeasurementPolicy::host_display(
        TextMeasurementProfileIdentity::new(
            MeasurementProfileId::new("flow-serif-cancel").unwrap(),
            "1",
        )
        .unwrap(),
        std::sync::Arc::new(Cancel(config.control.clone())),
        [TextMeasurementPhase::Layout],
    );
    // when
    let result = render("@startflow\nA \"Measure\"\n@endflow", &config);
    let svg = checked("A \"Paint\"");
    // then
    assert!(
        matches!(result, Err(diagram_engine::RenderError::Cancelled(_))),
        "{result:?}"
    );
    assert!(svg.contains("font-family=\"serif\""));
}
