use diagram_engine::{Language, RenderOptions, render};
fn options() -> RenderOptions {
    RenderOptions {
        language: Language::PlantUml,
        ..Default::default()
    }
}

#[test]
fn should_keep_same_label_text_with_distinct_explicit_colors_in_the_shared_cache() {
    // given
    let body = "* A\n---\n* B\nA -> B #112233 : signal\nA <- B #445566 : signal";
    // when
    let svg = output(body);
    // then
    for color in ["#112233", "#445566"] {
        assert!(
            svg.contains(&format!("stroke=\"{color}\""))
                && svg.contains(&format!("fill=\"{color}\"")),
            "{color}: {svg}"
        );
    }
}
fn source(body: &str) -> String {
    format!("@startwire\n{body}\n@endwire")
}
fn output(body: &str) -> String {
    let result = render(&source(body), &options()).unwrap();
    assert_eq!(result.family, "wire");
    usvg::Tree::from_str(&result.svg, &usvg::Options::default()).unwrap();
    result.svg
}
#[test]
fn should_render_original_columns_with_real_horizontal_wire_and_forced_sizes() {
    // given
    let body = "* Input [100x50]\n---\n* Output [100x50]\nInput -> Output : signal";
    // when
    let svg = output(body);
    let doc = roxmltree::Document::parse(&svg).unwrap();
    let blocks = doc
        .descendants()
        .filter(|n| n.attribute("data-kind") == Some("wire-block"))
        .collect::<Vec<_>>();
    // then
    assert_eq!(blocks.len(), 2);
    for block in blocks {
        let r = block.children().find(|n| n.has_tag_name("rect")).unwrap();
        assert_eq!(r.attribute("width"), Some("100"));
        assert_eq!(r.attribute("height"), Some("50"));
    }
    assert!(svg.contains("signal"));
    assert_eq!(
        doc.descendants()
            .filter(|n| n.attribute("data-kind") == Some("wire-link"))
            .count(),
        1
    );
    assert_eq!(svg, output(body));
}
#[test]
fn should_keep_nested_dotted_identity_and_both_vertical_arrowheads() {
    // given
    let body = "* Parent\n    * Same [100x50]\n    * Next [100x50]\nParent.Same <--> Parent.Next : down\n* Other\n\t* Same [100x50]";
    // when
    let svg = output(body);
    let doc = roxmltree::Document::parse(&svg).unwrap();
    // then
    for key in ["Parent.Same", "Parent.Next", "Other.Same"] {
        assert!(svg.contains(&format!("data-name=\"{key}\"")));
    }
    let link = doc
        .descendants()
        .find(|n| n.attribute("data-kind") == Some("wire-link"))
        .unwrap();
    assert_eq!(
        link.children()
            .filter(|n| n.has_tag_name("polygon"))
            .count(),
        2
    );
    assert!(svg.contains("down"));
}
#[test]
fn should_reject_wire_unmodeled_operations_endpoints_and_indentation() {
    // given
    for body in [
        "* A\nA => A",
        "* A\nspot A",
        "* A\nmove(10,20)",
        "* A\ngoto(10,20)",
        "* A\nprint text",
        "* A\nA(50%,0) -> A",
        "* A\nA -> Missing",
        "* A\n* A",
        "        * Missing_parent",
        "* A [0x20]",
        "* A [100001x20]",
    ] {
        // when
        let result = render(&source(body), &options());
        // then
        assert!(result.is_err(), "accepted {body}");
    }
}

#[test]
fn should_keep_orientation_separate_from_all_four_arrow_directions() {
    // given
    for (arrow, heads) in [
        ("-", 0),
        ("->", 1),
        ("<-", 1),
        ("<->", 2),
        ("--", 0),
        ("-->", 1),
        ("<--", 1),
        ("<-->", 2),
    ] {
        let body = if arrow.contains("--") {
            format!("* A [100x100]\n* B [100x100]\nA {arrow} B")
        } else {
            format!("* A [100x100]\n---\n* B [100x100]\nA {arrow} B")
        };
        // when
        let svg = output(&body);
        let doc = roxmltree::Document::parse(&svg).unwrap();
        let link = doc
            .descendants()
            .find(|n| n.attribute("data-kind") == Some("wire-link"))
            .unwrap();
        // then
        assert_eq!(
            link.children()
                .filter(|n| n.has_tag_name("polygon"))
                .count(),
            heads,
            "{arrow}"
        );
        assert_eq!(
            link.attribute("data-name"),
            Some(if arrow.contains("--") {
                "vertical"
            } else {
                "horizontal"
            })
        );
    }
}

#[test]
fn should_preserve_wire_raw_label_ownership_and_apply_color_to_line_and_text() {
    // given
    let body = "title Measured wire\n* First_Block\n---\n* Other_Block\nFirst_Block -> Other_Block #112233 : O'Brien /' literal !include https://host/path";
    // when
    let svg = output(body);
    // then
    assert!(svg.contains("First Block") && svg.contains("Other Block"));
    assert!(
        svg.contains("O&apos;Brien /&apos; literal !include https://host/path"),
        "{svg}"
    );
    assert!(svg.contains("stroke=\"#112233\"") && svg.contains("fill=\"#112233\""));
}

#[test]
fn should_report_forced_parent_and_one_axis_route_rejections_at_real_declarations() {
    // given
    for (body, line, detail) in [
        (
            "* Parent [40x30]\n    * Child [100x50]",
            2,
            "smaller than measured contents",
        ),
        ("* A\n* B\nA -> B", 4, "destination extent"),
    ] {
        // when
        let error = render(&source(body), &options()).unwrap_err().to_string();
        // then
        assert!(
            error.contains(&format!("line {line}, column 1")) && error.contains(detail),
            "{error}"
        );
    }
}

#[test]
fn should_bound_wire_source_model_depth_work_output_and_cancel_without_partial_scene() {
    // given
    let input = source("* A\n---\n* B\nA -> B : measured signal");
    for resource in ["source", "model", "depth", "work", "output", "cancel"] {
        let mut config = options();
        let text = if resource == "depth" {
            source("* A\n    * B")
        } else {
            input.clone()
        };
        match resource {
            "source" => config.limits.source_bytes = 10,
            "model" => config.limits.model_items = 3,
            "depth" => config.limits.nesting_depth = 1,
            "work" => config.limits.layout_work = 100,
            "output" => config.limits.svg_bytes = 1536,
            _ => config.control.cancel(),
        };
        // when
        let result = render(&text, &config);
        // then
        assert!(result.is_err(), "{resource}");
    }
}
