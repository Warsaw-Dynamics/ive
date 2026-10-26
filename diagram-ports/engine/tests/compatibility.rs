use diagram_engine::{
    HostMeasurementResult, HostTextMeasurementRequest, HostTextMeasurer, MeasurementProfileId,
    RenderOptions, TextMeasurementPhase, TextMeasurementPolicy, TextMeasurementProfileIdentity,
    TextStyle, render,
};
use merman::{
    Engine, ParseOptions, RenderSemanticModel, diagrams::mindmap::MindmapDiagramRenderModel,
};
use std::sync::{Arc, Mutex};

const MINDMAP: &str = "mindmap\n  Root\n    BranchA\n      LeafA\n    BranchB\n      LeafB\n";

fn parse_mindmap(source: &str) -> MindmapDiagramRenderModel {
    let parsed = Engine::new()
        .parse_diagram_for_render_model_sync(source, ParseOptions::strict())
        .unwrap()
        .unwrap();
    let RenderSemanticModel::Mindmap(model) = parsed.into_parts().1 else {
        panic!("expected typed mindmap")
    };
    model
}

fn depth_source(body: &str) -> String {
    format!("---\nconfig:\n  mindmap:\n    colorBy: depth\n---\n{body}")
}

#[test]
fn should_color_typed_nodes_and_edges_by_structural_depth() {
    // given
    let source = depth_source(MINDMAP);
    // when
    let model = parse_mindmap(&source);
    // then
    assert_eq!(
        model
            .nodes
            .iter()
            .map(|node| node.section)
            .collect::<Vec<_>>(),
        [None, Some(0), Some(1), Some(0), Some(1)]
    );
    assert_eq!(
        model
            .edges
            .iter()
            .map(|edge| edge.section)
            .collect::<Vec<_>>(),
        [Some(0), Some(1), Some(0), Some(1)]
    );
    assert!(model.edges.iter().all(|edge| {
        edge.classes
            .contains(&format!("section-edge-{}", edge.section.unwrap()))
    }));
}

#[test]
fn should_preserve_default_and_explicit_branch_coloring() {
    // given
    let explicit = format!("---\nconfig:\n  mindmap:\n    colorBy: branch\n---\n{MINDMAP}");
    // when
    let models = [parse_mindmap(MINDMAP), parse_mindmap(&explicit)];
    // then
    for model in models {
        assert_eq!(
            model
                .nodes
                .iter()
                .map(|node| node.section)
                .collect::<Vec<_>>(),
            [None, Some(0), Some(0), Some(1), Some(1)]
        );
    }
}

#[test]
fn should_ignore_indentation_width_and_preserve_mindmap_decorations() {
    // given
    let source = depth_source(
        "mindmap\n    Root\n          BranchA\n            LeafA\n                DeepA\n              DeepB\n          BranchB\n              LeafB\n                DeepC\n                  :::custom\n                  ::icon(mdi:fire)\n",
    );
    // when
    let model = parse_mindmap(&source);
    // then
    assert_eq!(
        model
            .nodes
            .iter()
            .map(|node| node.section)
            .collect::<Vec<_>>(),
        [
            None,
            Some(0),
            Some(1),
            Some(2),
            Some(2),
            Some(0),
            Some(1),
            Some(2)
        ]
    );
    let last = model.nodes.last().unwrap();
    assert!(
        last.css_classes
            .split_whitespace()
            .any(|class| class == "custom")
    );
    assert_eq!(last.icon.as_deref(), Some("mdi:fire"));
}

#[test]
fn should_cycle_depth_colors_after_eleven_non_root_levels() {
    // given
    let mut body = "mindmap\nRoot\n".to_owned();
    for depth in 1..=13 {
        body.push_str(&format!("{}Node {depth}\n", "  ".repeat(depth)));
    }
    // when
    let model = parse_mindmap(&depth_source(&body));
    // then
    assert_eq!(
        model
            .nodes
            .iter()
            .skip(1)
            .map(|node| node.section)
            .collect::<Vec<_>>(),
        [
            Some(0),
            Some(1),
            Some(2),
            Some(3),
            Some(4),
            Some(5),
            Some(6),
            Some(7),
            Some(8),
            Some(9),
            Some(10),
            Some(0),
            Some(1)
        ]
    );
}

#[test]
fn should_preserve_depth_sections_when_inserting_a_preceding_branch() {
    // given
    let source = depth_source(MINDMAP);
    let inserted = source.replace("    BranchA", "    Added\n    BranchA");
    // when
    let before = parse_mindmap(&source);
    let after = parse_mindmap(&inserted);
    // then
    for node in before.nodes {
        assert_eq!(
            after
                .nodes
                .iter()
                .find(|candidate| candidate.label == node.label)
                .unwrap()
                .section,
            node.section
        );
    }
}

#[test]
fn should_render_depth_sections_through_the_public_engine() {
    // given
    let source = depth_source(MINDMAP);
    // when
    let rendered = render(&source, &RenderOptions::default()).unwrap();
    // then
    let document = roxmltree::Document::parse(&rendered.svg).unwrap();
    let classes = document
        .descendants()
        .filter(|node| node.has_tag_name("g"))
        .filter_map(|node| node.attribute("class"))
        .filter(|classes| {
            classes
                .split_whitespace()
                .any(|class| class == "mindmap-node")
        })
        .collect::<Vec<_>>();
    assert_eq!(
        classes,
        [
            "node mindmap-node section-root section--1",
            "node mindmap-node section-0",
            "node mindmap-node section-1",
            "node mindmap-node section-0",
            "node mindmap-node section-1"
        ]
    );
    usvg::Tree::from_str(&rendered.svg, &usvg::Options::default()).unwrap();
}

#[derive(Default)]
struct RequestedFonts(Mutex<Vec<(String, TextStyle)>>);

impl HostTextMeasurer for RequestedFonts {
    fn measure(&self, request: HostTextMeasurementRequest<'_>) -> HostMeasurementResult {
        self.0
            .lock()
            .unwrap()
            .push((request.text.into(), request.style.clone()));
        Ok(None)
    }
}

fn render_with_fonts(source: &str) -> (String, Vec<(String, TextStyle)>) {
    let fonts = Arc::new(RequestedFonts::default());
    let options = RenderOptions {
        text_measurement: TextMeasurementPolicy::host_display(
            TextMeasurementProfileIdentity::new(
                MeasurementProfileId::new("test-font-requests").unwrap(),
                "1",
            )
            .unwrap(),
            fonts.clone(),
            TextMeasurementPhase::ALL,
        ),
        ..Default::default()
    };
    let rendered = render(source, &options).unwrap();
    let requests = fonts.0.lock().unwrap().clone();
    (rendered.svg, requests)
}

fn label_style(svg: &str, text: &str) -> String {
    let document = roxmltree::Document::parse(svg).unwrap();
    document
        .descendants()
        .find(|node| {
            node.has_tag_name("g")
                && node
                    .attribute("class")
                    .is_some_and(|classes| classes.split_whitespace().any(|class| class == "label"))
                && node
                    .descendants()
                    .filter(|child| child.is_text())
                    .filter_map(|child| child.text())
                    .collect::<String>()
                    == text
        })
        .and_then(|node| node.attribute("style"))
        .unwrap()
        .to_owned()
}

fn font_value(css: &str) -> &str {
    css.split("font-family:")
        .nth(1)
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .trim()
}

fn font_tokens(value: &str) -> Vec<String> {
    let mut parser = cssparser::Parser::new(value);
    let mut tokens = Vec::new();
    while let Ok(token) = parser.next() {
        tokens.push(match token {
            cssparser::Token::Ident(value) => format!("ident:{value}"),
            cssparser::Token::QuotedString(value) => format!("string:{value}"),
            cssparser::Token::Comma => ",".into(),
            token => panic!("unexpected font token: {token:?}"),
        });
    }
    tokens
}

#[test]
fn should_measure_and_paint_svg_usecase_labels_with_the_effective_global_font() {
    // given
    let source = "%%{init: {\"fontFamily\":\"Liberation Serif\",\"themeVariables\":{\"fontFamily\":\"Liberation Serif\",\"fontSize\":\"20px\"},\"usecase\":{\"usecaseFontSize\":28,\"usecaseFontFamily\":\"monospace\"}}}%%\nusecase-beta\nactor User\nCheckout(\"Place order\")\nUser --> Checkout";
    // when
    let (svg, requests) = render_with_fonts(source);
    // then
    let labels = requests
        .iter()
        .filter(|(text, _)| text == "Place order")
        .collect::<Vec<_>>();
    assert!(!labels.is_empty());
    assert!(labels.iter().all(|(_, style)| style.font_size == 20.0
        && style.font_family.as_deref() == Some("Liberation Serif")));
    let style = label_style(&svg, "Place order");
    assert!(
        style.contains("font-family:Liberation Serif") && style.contains("font-size:20px"),
        "{style}"
    );
}

#[test]
fn should_preserve_usecase_node_font_overrides_in_measurement_and_paint() {
    // given
    let source = "usecase-beta\nCheckout(\"Custom label\")\nstyle Checkout font-size:18px,font-family:Liberation Mono";
    // when
    let (svg, requests) = render_with_fonts(source);
    // then
    let labels = requests
        .iter()
        .filter(|(text, _)| text == "Custom label")
        .collect::<Vec<_>>();
    assert!(!labels.is_empty());
    assert!(labels.iter().all(|(_, style)| style.font_size == 18.0
        && style.font_family.as_deref() == Some("Liberation Mono")));
    assert!(svg.contains("font-family:Liberation Mono;font-size:18px"));
}

#[test]
fn should_paint_tree_text_with_the_same_non_default_family_requested_for_layout() {
    // given
    let source = "%%{init: {\"fontFamily\":\"monospace\",\"themeVariables\":{\"fontFamily\":\"Liberation Serif\"}}}%%\ntreeView-beta\n    root/\n        file.rs";
    // when
    let (svg, requests) = render_with_fonts(source);
    // then
    assert!(!requests.is_empty());
    assert!(
        requests
            .iter()
            .all(|(_, style)| style.font_family.as_deref() == Some("Liberation Serif"))
    );
    let document = roxmltree::Document::parse(&svg).unwrap();
    let css = document
        .descendants()
        .filter(|node| node.has_tag_name("style"))
        .filter_map(|node| node.text())
        .collect::<String>();
    assert_eq!(
        font_tokens(font_value(&css)),
        ["ident:Liberation", "ident:Serif"]
    );
    assert!(
        requests
            .iter()
            .any(|(_, style)| style.font_weight.as_deref() == Some("bold"))
    );
    assert!(
        requests
            .iter()
            .any(|(_, style)| style.font_weight.is_none())
    );
}

#[test]
fn should_preserve_quoted_commas_and_generic_fallbacks_in_effective_font_lists() {
    // given
    let family = "\"Made, Up\", \"Liberation Serif\", serif";
    let config = serde_json::json!({"fontFamily":family,"themeVariables":{"fontFamily":family}});
    for body in [
        "treeView-beta\n    root/\n        file.rs",
        "usecase-beta\nCheckout(\"Label\")",
    ] {
        let source = format!("%%{{init: {config}}}%%\n{body}");
        // when
        let (svg, requests) = render_with_fonts(&source);
        // then
        assert!(!requests.is_empty());
        for (_, style) in &requests {
            assert_eq!(
                font_tokens(style.font_family.as_deref().unwrap()),
                [
                    "string:Made, Up",
                    ",",
                    "string:Liberation Serif",
                    ",",
                    "ident:serif"
                ]
            );
        }
        let document = roxmltree::Document::parse(&svg).unwrap();
        let css = if body.starts_with("tree") {
            document
                .descendants()
                .find(|node| node.has_tag_name("style"))
                .unwrap()
                .text()
                .unwrap()
                .to_owned()
        } else {
            label_style(&svg, "Label")
        };
        assert_eq!(
            font_tokens(font_value(&css)),
            [
                "string:Made, Up",
                ",",
                "string:Liberation Serif",
                ",",
                "ident:serif"
            ]
        );
        usvg::Tree::from_str(&svg, &usvg::Options::default()).unwrap();
    }
}

#[test]
fn should_filter_font_css_injection_before_measurement_and_painting() {
    // given
    for family in [
        "serif; } #injected { fill: red }",
        "url(https://example.invalid/font)",
        "</style><script>injected</script>",
    ] {
        let config =
            serde_json::json!({"fontFamily":family,"themeVariables":{"fontFamily":family}});
        let source = format!("%%{{init: {config}}}%%\ntreeView-beta\n    root/\n        file.rs");
        // when
        let (svg, requests) = render_with_fonts(&source);
        // then
        assert!(!svg.contains("injected") && !svg.contains("example.invalid"));
        assert!(!requests.is_empty());
        for (_, style) in requests {
            assert_eq!(
                font_tokens(style.font_family.as_deref().unwrap()),
                [
                    "string:trebuchet ms",
                    ",",
                    "ident:verdana",
                    ",",
                    "ident:arial",
                    ",",
                    "ident:sans-serif"
                ]
            );
        }
        let document = roxmltree::Document::parse(&svg).unwrap();
        let css = document
            .descendants()
            .find(|node| node.has_tag_name("style"))
            .unwrap()
            .text()
            .unwrap();
        assert_eq!(
            font_tokens(font_value(css)),
            [
                "string:trebuchet ms",
                ",",
                "ident:verdana",
                ",",
                "ident:arial",
                ",",
                "ident:sans-serif"
            ]
        );
    }
}
