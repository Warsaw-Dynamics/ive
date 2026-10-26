use diagram_engine::{Language, RenderOptions, render};

fn options() -> RenderOptions {
    RenderOptions {
        language: Language::PlantUml,
        ..Default::default()
    }
}
fn source(body: &str) -> String {
    format!("@startchart\n{body}\n@endchart")
}
fn output(body: &str) -> String {
    let output = render(&source(body), &options()).unwrap();
    assert_eq!(output.family, "chart");
    usvg::Tree::from_str(&output.svg, &usvg::Options::default()).unwrap();
    output.svg
}

#[test]
fn should_render_original_chart_with_default_range_and_real_bar_heights() {
    // given
    let body = "x-axis \"Week\" [\"Mon\", \"Tue\", \"Wed\"]\nbar \"Orders\" [3, 5, 4] labels\nlegend right";
    // when
    let svg = output(body);
    let document = roxmltree::Document::parse(&svg).unwrap();
    let heights: Vec<f64> = document
        .descendants()
        .filter(|n| n.attribute("data-kind") == Some("chart-bar"))
        .map(|n| {
            n.children()
                .find(|n| n.has_tag_name("rect"))
                .unwrap()
                .attribute("height")
                .unwrap()
                .parse()
                .unwrap()
        })
        .collect();
    // then
    assert_eq!(heights, [9., 15., 12.]);
    assert!(svg.contains("100"));
    assert!(svg.contains("Week") && svg.contains("Orders"));
    assert_eq!(svg, output(body));
}

#[test]
fn should_give_chart_legend_priority_before_axes_and_keep_series_line_colors() {
    // given
    let body = "x-axis [\"a\",\"b\",\"c\"]\nline \"one\" [0,50,100] #112233\nline \"two\" [100,50,0] #445566";
    // when
    let before = output(&format!("legend right\n{body}"));
    let after = output(&format!("{body}\nlegend right"));
    // then
    assert_eq!(before, after);
    for color in ["#112233", "#445566"] {
        assert!(
            before.matches(&format!("stroke=\"{color}\"")).count() >= 2,
            "{before}"
        );
    }
}

#[test]
fn should_reject_unsupported_or_malformed_chart_commands_and_mismatched_data() {
    // given
    for tail in [
        "bar [1]",
        "bar [NaN,2]",
        "bar [Infinity,2]",
        "bar [1,2] labels junk",
        "line [1,2] y2 labels",
        "y-axis 10 --> 10",
        "y-axis 10 --> 0",
        "bar [1,2]\nstackMode stacked",
        "bar [1,2]\nannotation \"bad\" at (a,1)",
        "bar [1,2]\nx-axis [\"a\",\"b\"] grid",
    ] {
        // when
        let result = render(
            &source(&format!("x-axis [\"a\",\"b\"]\n{tail}")),
            &options(),
        );
        // then
        assert!(result.is_err(), "accepted {tail}");
    }
}

#[test]
fn should_preserve_chart_display_literals_chrome_and_original_expansion_locations() {
    // given
    let body = "title Measured chart\nheader Native\nlegend top\nx-axis \"Week\" [\"https://host/a\",\"O'Brien\"]\nbar \"Orders\" [3,4] labels";
    // when
    let svg = output(body);
    let error = render(
        "@startchart\n!define BAD NaN\nx-axis [\"a\"]\nbar [BAD]\n@endchart",
        &options(),
    )
    .unwrap_err()
    .to_string();
    // then
    assert!(svg.contains("O&apos;Brien") && svg.contains("https://host/a"));
    assert!(svg.contains("Measured chart") && svg.contains("Native"));
    assert!(
        error.contains("line 4") && error.contains("defined at line 2"),
        "{error}"
    );
    assert!(error.contains("NaN"), "{error}");
}

#[test]
fn should_bound_chart_source_samples_model_work_output_and_cancellation() {
    // given
    let input = source("x-axis [\"a\",\"b\",\"c\"]\nbar [-10,0,150] labels\nlegend bottom");
    for resource in ["source", "model", "work", "output", "cancel"] {
        let mut config = options();
        match resource {
            "source" => config.limits.source_bytes = 10,
            "model" => config.limits.model_items = 4,
            "work" => config.limits.layout_work = 100,
            "output" => config.limits.svg_bytes = 100,
            _ => config.control.cancel(),
        };
        // when
        let result = render(&input, &config);
        // then
        assert!(result.is_err(), "{resource}");
    }
    let bad = source("x-axis [\"a\",\"b\"]\nline [1000000000001,2]");
    assert!(
        render(&bad, &options())
            .unwrap_err()
            .to_string()
            .contains("1000000000001")
    );
}

#[test]
fn should_locate_mismatched_series_at_its_declaration_with_actual_counts() {
    // given
    let input = "@startchart\nx-axis [\"a\",\"b\"]\nbar \"Orders\" [1]\n@endchart";
    // when
    let error = render(input, &options()).unwrap_err().to_string();
    // then
    assert!(error.contains("line 3, column 1"), "{error}");
    assert!(
        error.contains("Orders") && error.contains("1 samples for 2 categories"),
        "{error}"
    );
}
