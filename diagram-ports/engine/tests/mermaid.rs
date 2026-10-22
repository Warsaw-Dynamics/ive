use diagram_engine::{Language, RenderError, RenderOptions, render};

fn text_content(svg: &str) -> String {
    let document = roxmltree::Document::parse(svg).expect("valid SVG XML");
    assert_eq!(document.root_element().tag_name().name(), "svg");
    assert!(document.descendants().any(|node| {
        node.is_element()
            && matches!(
                node.tag_name().name(),
                "path" | "rect" | "circle" | "polygon"
            )
    }));
    document
        .descendants()
        .filter(|node| node.is_text() && node.ancestors().any(|parent| parent.has_tag_name("text")))
        .filter_map(|node| node.text())
        .collect()
}

#[test]
fn should_render_metadata_as_literal_label_text() {
    // given
    let source = "flowchart LR\n A[\"metadata: loaded\"] --> B[Done]";
    // when
    let diagram = render(source, &RenderOptions::default()).expect("metadata label renders");
    // then
    assert!(text_content(&diagram.svg).contains("metadata: loaded"));
}

#[test]
fn should_ignore_url_in_comment() {
    // given
    let source = "flowchart LR\n %% See https://example.invalid/docs\n A[Start] --> B[Done]";
    // when
    let diagram = render(source, &RenderOptions::default()).expect("URL comment is inert");
    // then
    assert!(text_content(&diagram.svg).contains("Start"));
    assert!(text_content(&diagram.svg).contains("Done"));
}

#[test]
fn should_render_url_as_literal_label_text() {
    // given
    let source = "flowchart LR\n A[\"https://example.invalid/docs\"] --> B[Done]";
    // when
    let diagram = render(source, &RenderOptions::default()).expect("URL label renders");
    // then
    assert!(text_content(&diagram.svg).contains("https://example.invalid/docs"));
}

#[test]
fn should_render_image_metadata_tokens_as_literal_label_text() {
    // given
    let source = "flowchart LR\n A[\"img: example.png; file: README; data: loaded\"] --> B[Done]";
    // when
    let diagram = render(source, &RenderOptions::default()).expect("resource words in text render");
    // then
    assert!(text_content(&diagram.svg).contains("img: example.png"));
    assert!(text_content(&diagram.svg).contains("file: README"));
    assert!(text_content(&diagram.svg).contains("data: loaded"));
}

#[test]
fn should_reject_external_navigation_link() {
    // given
    let source =
        "flowchart LR\n A[Docs] --> B[Done]\n click A href \"https://example.invalid/docs\"";
    // when
    let result = render(source, &RenderOptions::default());
    // then
    assert!(matches!(result, Err(RenderError::ExternalResource(_))));
}

#[test]
fn should_reject_external_css_paint_resource() {
    // given
    let source = "flowchart LR\n A[Start] --> B[Done]\n style A fill:url(https://example.invalid/paint.svg#red)";
    // when
    let result = render(source, &RenderOptions::default());
    // then
    assert!(
        matches!(result, Err(RenderError::ExternalResource(_))),
        "{result:?}"
    );
}

#[test]
fn should_render_both_flowchart_labels() {
    // given
    let source = "flowchart LR\n A[First node] --> B[Second node]";
    // when
    let diagram = render(source, &RenderOptions::default()).expect("flowchart renders");
    // then
    let text = text_content(&diagram.svg);
    assert!(text.contains("First node"));
    assert!(text.contains("Second node"));
    assert_eq!(diagram.language, Language::Mermaid);
}

#[test]
fn should_render_sequence_message() {
    // given
    let source = "sequenceDiagram\n Alice->>Bob: Send greeting";
    // when
    let diagram = render(source, &RenderOptions::default()).expect("sequence renders");
    // then
    assert!(text_content(&diagram.svg).contains("Send greeting"));
}

#[test]
fn should_render_class_members() {
    // given
    let source = "classDiagram\n class Account {\n +String name\n +save()\n }";
    // when
    let diagram = render(source, &RenderOptions::default()).expect("class renders");
    // then
    let text = text_content(&diagram.svg);
    assert!(text.contains("name"));
    assert!(text.contains("save()"));
}

#[test]
fn should_reject_invalid_source() {
    // given
    let source = "flowchart LR\n A[unfinished";
    // when
    let result = render(source, &RenderOptions::default());
    // then
    assert!(matches!(result, Err(RenderError::InvalidSource(_))));
}

#[test]
fn should_preserve_xml_metacharacters_and_unicode() {
    // given
    let source = "flowchart LR\n A[\"Żółw & kot <test>\"] --> B[\"東京 🦀\"]";
    // when
    let diagram = render(source, &RenderOptions::default()).expect("Unicode labels render");
    // then
    let text = text_content(&diagram.svg);
    assert!(text.contains("Żółw & kot"));
    assert!(text.contains("東京 🦀"));
    assert!(!diagram.svg.contains("<test>"));
}

#[test]
fn should_enforce_source_limit_before_parsing() {
    // given
    let options = RenderOptions {
        limits: diagram_engine::RenderLimits {
            source_bytes: 8,
            ..Default::default()
        },
        ..Default::default()
    };
    // when
    let result = render("flowchart LR\n A-->B", &options);
    // then
    assert!(matches!(
        result,
        Err(RenderError::SourceLimit { limit: 8, .. })
    ));
}

#[test]
fn should_reject_external_image_resources() {
    // given
    let source =
        "flowchart LR\n A@{ img: \"https://example.invalid/private.png\", label: \"Remote\" }";
    // when
    let result = render(source, &RenderOptions::default());
    // then
    assert!(matches!(result, Err(RenderError::ExternalResource(_))));
}

#[test]
fn should_reject_unavailable_requested_layout() {
    // given
    let source = "---\nconfig:\n  layout: elk\n---\nflowchart LR\n A-->B";
    // when
    let result = render(source, &RenderOptions::default());
    // then
    assert!(matches!(result, Err(RenderError::UnsupportedLayout(_))));
}

#[test]
fn should_report_plantuml_as_unsupported() {
    // given
    let options = RenderOptions {
        language: Language::PlantUml,
        ..Default::default()
    };
    // when
    let result = render("@startuml\nAlice -> Bob\n@enduml", &options);
    // then
    assert_eq!(
        result.unwrap_err(),
        RenderError::UnsupportedLanguage(Language::PlantUml)
    );
}

#[test]
fn should_observe_cancellation() {
    // given
    let options = RenderOptions::default();
    options.control.cancel();
    // when
    let result = render("flowchart LR\n A-->B", &options);
    // then
    assert!(matches!(result, Err(RenderError::Cancelled(_))));
}

#[test]
fn should_enforce_svg_output_budget() {
    // given
    let options = RenderOptions {
        limits: diagram_engine::RenderLimits {
            svg_bytes: 128,
            ..Default::default()
        },
        ..Default::default()
    };
    // when
    let result = render(
        "flowchart LR\n A[Large output] --> B[More output]",
        &options,
    );
    // then
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("max_svg_bytes"));
}

#[test]
fn should_observe_expired_deadline() {
    // given
    let options = RenderOptions {
        limits: diagram_engine::RenderLimits {
            timeout: std::time::Duration::from_nanos(1),
            ..Default::default()
        },
        ..Default::default()
    };
    // when
    let result = render("flowchart LR\n A-->B", &options);
    // then
    assert!(matches!(result, Err(RenderError::Cancelled(_))));
}

#[test]
fn should_reject_relative_image_resources_before_layout() {
    // given
    let source = "flowchart LR\n A@{ img : \"private.png\", label: \"Local\" }";
    // when
    let result = render(source, &RenderOptions::default());
    // then
    assert!(matches!(result, Err(RenderError::ExternalResource(_))));
}

#[test]
fn should_preserve_plain_xml_metacharacters() {
    // given
    let source = "flowchart LR\n A[\"1 < 2 & 3 > 2\"] --> B[Done]";
    // when
    let diagram = render(source, &RenderOptions::default()).expect("metacharacters render");
    // then
    assert!(text_content(&diagram.svg).contains("1 < 2 & 3 > 2"));
}

#[test]
fn should_produce_deterministic_svg_across_requests() {
    // given
    let source = "flowchart LR\n A[First] --> B[Second]";
    let options = RenderOptions::default();
    // when
    let first = render(source, &options).unwrap();
    let second = render(source, &options).unwrap();
    // then
    assert_eq!(first.svg, second.svg);
}
