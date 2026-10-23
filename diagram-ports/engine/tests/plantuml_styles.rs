use diagram_engine::{
    HostMeasurementResult, HostTextMeasurement, HostTextMeasurementRequest, HostTextMeasurer,
    Language, MeasurementProfileId, RenderOptions, TextMeasurementPhase, TextMeasurementPolicy,
    TextMeasurementProfileIdentity, TextMetrics, render,
};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, PartialEq)]
struct Query {
    text: String,
    family: String,
    size: f64,
    weight: String,
    slant: String,
}
struct Recorder(Arc<Mutex<Vec<Query>>>);
impl HostTextMeasurer for Recorder {
    fn measure(&self, request: HostTextMeasurementRequest<'_>) -> HostMeasurementResult {
        self.0.lock().unwrap().push(Query {
            text: request.text.into(),
            family: request.style.font_family.clone().unwrap_or_default(),
            size: request.style.font_size,
            weight: request.style.font_weight.clone().unwrap_or_default(),
            slant: request.style.font_style.clone().unwrap_or_default(),
        });
        Ok(Some(HostTextMeasurement::Metrics(TextMetrics {
            width: request.text.chars().count() as f64 * request.style.font_size,
            height: request.style.font_size * 1.2,
            line_count: 1,
        })))
    }
}
fn options() -> RenderOptions {
    RenderOptions {
        language: Language::PlantUml,
        ..Default::default()
    }
}
fn recorded() -> (RenderOptions, Arc<Mutex<Vec<Query>>>) {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let mut options = options();
    options.text_measurement = TextMeasurementPolicy::host_display(
        TextMeasurementProfileIdentity::new(
            MeasurementProfileId::new("style-contract").unwrap(),
            "1",
        )
        .unwrap(),
        Arc::new(Recorder(calls.clone())),
        [TextMeasurementPhase::Layout],
    );
    (options, calls)
}
fn source(body: &str) -> String {
    format!("@startuml\n{body}\n@enduml")
}

#[test]
fn should_measure_and_paint_identical_nested_runs_with_composable_base_styles() {
    // given
    let (options, calls) = recorded();
    let input = source(
        "skinparam defaultFontName Liberation Sans\nskinparam defaultFontSize 22\nskinparam defaultFontColor #123456\nobject \"**Same //Both//**\" as A {\n{static} **Same**\n}",
    );
    // when
    let result = render(&input, &options).unwrap();
    let xml = roxmltree::Document::parse(&result.svg).unwrap();
    let calls = calls.lock().unwrap();
    // then
    for node in xml.descendants().filter(|n| n.has_tag_name("text")) {
        let query = Query {
            text: node.text().unwrap_or("").into(),
            family: node.attribute("font-family").unwrap().into(),
            size: node.attribute("font-size").unwrap().parse().unwrap(),
            weight: node.attribute("font-weight").unwrap().into(),
            slant: node.attribute("font-style").unwrap().into(),
        };
        assert!(
            calls.contains(&query),
            "paint has no matching measurement: {query:?}"
        );
        assert_eq!(node.attribute("fill"), Some("#123456"));
        assert_eq!(node.attribute("text-decoration"), Some("underline"));
    }
    assert!(
        calls
            .iter()
            .any(|q| q.text == "Both" && q.size == 22. && q.weight == "700" && q.slant == "italic")
    );
}

#[test]
fn should_reserve_chrome_and_keep_default_element_size_separate() {
    // given
    let (options, _) = recorded();
    let input = source(
        "skinparam defaultFontSize 22\nleft header Wide wide wide wide wide wide wide wide\ntitle Top\\nSecond\nlegend Bottom\nright footer Foot\nA -> B : message",
    );
    // when
    let result = render(&input, &options).unwrap();
    let xml = roxmltree::Document::parse(&result.svg).unwrap();
    // then
    let text = |s| {
        xml.descendants()
            .find(|n| n.has_tag_name("text") && n.text() == Some(s))
            .unwrap()
    };
    assert_eq!(text("Top").attribute("font-size"), Some("14"));
    assert_eq!(text("Bottom").attribute("font-size"), Some("14"));
    assert_eq!(text("message").attribute("font-size"), Some("22"));
    let y = |s| text(s).attribute("y").unwrap().parse::<f64>().unwrap();
    assert!(
        y("Top") < y("Second")
            && y("Second") < y("message")
            && y("message") < y("Bottom")
            && y("Bottom") < y("Foot")
    );
    assert!(
        xml.root_element()
            .attribute("width")
            .unwrap()
            .parse::<f64>()
            .unwrap()
            >= 390.
    );
}

#[test]
fn should_follow_last_authored_matching_style_property_and_keep_color_cache_separate() {
    // given
    let input = source(
        "<style>\ndocument {\ntitle {\nFontColor #ff0000\n}\n}\nroot {\nFontColor #0000ff\n}\nelement {\nFontColor #00ff00\n}\n</style>\ntitle Same\nA -> B : Same",
    );
    // when
    let result = render(&input, &options()).unwrap();
    let xml = roxmltree::Document::parse(&result.svg).unwrap();
    // then
    let colors: Vec<_> = xml
        .descendants()
        .filter(|n| n.has_tag_name("text") && n.text() == Some("Same"))
        .map(|n| n.attribute("fill").unwrap())
        .collect();
    assert!(colors.contains(&"#0000ff") && colors.contains(&"#00ff00"));
}

#[test]
fn should_decode_breaks_once_in_quoted_unquoted_and_chrome_labels() {
    // given
    let input = source(
        r#"title "a\\n b\n c"
participant "a\\n b\n c" as A
A -> B : a\\n b\n c"#,
    );
    // when
    let result = render(&input, &options()).unwrap();
    let xml = roxmltree::Document::parse(&result.svg).unwrap();
    // then
    assert!(
        xml.descendants()
            .filter(|n| n.has_tag_name("text") && n.text() == Some("a\\n b"))
            .count()
            >= 3
    );
    assert!(
        xml.descendants()
            .filter(|n| n.has_tag_name("text") && n.text() == Some(" c"))
            .count()
            >= 3
    );
}

#[test]
fn should_preserve_escaped_markup_urls_angles_pua_and_lone_markers() {
    // given
    let input = source(
        "A -> B : ~**literal~** <angle> https://one/a https://two/b \u{e02a} left -- right ** // __ ~<b> ",
    );
    // when
    let result = render(&input, &options()).unwrap();
    let xml = roxmltree::Document::parse(&result.svg).unwrap();
    // then
    assert!(xml.descendants().any(|n| n.has_tag_name("text") && n.text().is_some_and(|s| s.contains("**literal** <angle> https://one/a https://two/b \u{e02a} left -- right ** // __ <b>"))));
}

#[test]
fn should_reject_crossed_unclosed_and_unsupported_markup() {
    // given
    for label in [
        "**a //b** c//",
        "<b>x",
        "<B>x</i>",
        "**<img:x>**",
        "~~wave~~",
        "[[link]]",
        "<color:#ff0000>x</color>",
    ] {
        // when
        let result = render(&source(&format!("A -> B : {label}")), &options());
        // then
        assert!(result.is_err(), "accepted {label}");
    }
}

#[test]
fn should_reject_unknown_style_declarations_values_and_unbalanced_rules() {
    // given
    for preamble in [
        "skinparam unknown 20",
        "skinparam defaultFontSize 97",
        "skinparam titleFontColor red",
        "skinparam defaultFontName url(x)",
        "skinparam defaultFontStyle bold",
        "skinparam titleFontStyle bold italic",
        "<style>\nroot {\nBogus 1\n}\n</style>",
        "<style>\ndocument {\nactor {\nFontSize 20\n}\n}\n</style>",
        "<style>\nroot {\nFontSize 20\n</style>",
        "skinparam titleFontSize 20\n<style>\nroot {\nFontSize 22\n}\n</style>",
        "title\nx\nend title",
    ] {
        // when
        let result = render(&source(&format!("{preamble}\nA -> B : x")), &options());
        // then
        assert!(result.is_err(), "accepted {preamble}");
    }
}

#[test]
fn should_style_data_preamble_without_interpreting_literal_payload() {
    // given
    for (wrapper, payload) in [
        ("json", r#"{"title":"**literal**\\n"}"#),
        (
            "yaml",
            "title: '**literal**\\n'\nblock: |\n  title body\n  <style>",
        ),
    ] {
        let input = format!(
            "@start{wrapper}\ntitle Data\n<style>\nelement {{\nFontSize 24\nFontColor #123456\n}}\n</style>\n{payload}\n@end{wrapper}"
        );
        // when
        let result = render(&input, &options()).unwrap();
        let xml = roxmltree::Document::parse(&result.svg).unwrap();
        // then
        assert!(xml.descendants().any(|n| n.has_tag_name("text")
            && n.text() == Some("**literal**\\n")
            && n.attribute("font-size") == Some("24")
            && n.attribute("fill") == Some("#123456")));
    }
}

#[test]
fn should_reject_unimplemented_block_markup_without_rejecting_lone_punctuation() {
    // given
    for label in ["* item", "# item", "= heading", "| a | b |", "----"] {
        // when
        let result = render(&source(&format!("A -> B : {label}")), &options());
        // then
        assert!(result.is_err(), "accepted block markup {label:?}");
    }
}

#[test]
fn should_keep_chrome_contained_and_separate_from_every_family_body() {
    // given
    for (wrapper, body) in [
        (
            "uml",
            "participant A\nA -> B : **request**\nnote over B : //note//",
        ),
        (
            "uml",
            "package \"**Outer**\" as P {\npackage \"Inner\" as Q {\nclass \"__Type__\" as A {\n{abstract} **member**\n}\n}\nclass B\nA --> \"//many//\" B : --relation--\n}",
        ),
        (
            "uml",
            "state S {\nstate \"**Child**\" as C\n[*] --> C : //start//\n}",
        ),
        (
            "uml",
            "start\npartition \"**Group**\" {\n:__action__;\n}\nstop",
        ),
        ("mindmap", "* **Root**\n** //Child//"),
        ("wbs", "* **Root**\n** //Child//"),
    ] {
        let (options, _) = recorded();
        let preamble = "skinparam defaultFontSize 24\nleft header A very wide header with many words\ntitle **Top**\\n//Second//\nlegend __Bottom__\nright footer --Foot--";
        let input = format!("@start{wrapper}\n{preamble}\n{body}\n@end{wrapper}");
        // when
        let result = render(&input, &options).unwrap();
        let xml = roxmltree::Document::parse(&result.svg).unwrap();
        let bounds: Vec<f64> = xml
            .root_element()
            .attribute("viewBox")
            .unwrap()
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();
        // then
        for node in xml.descendants().filter(|n| n.has_tag_name("text")) {
            let x: f64 = node.attribute("x").unwrap().parse().unwrap();
            let y: f64 = node.attribute("y").unwrap().parse().unwrap();
            let size: f64 = node.attribute("font-size").unwrap().parse().unwrap();
            let width = node.text().unwrap_or("").chars().count() as f64 * size;
            assert!(
                x >= bounds[0] && x + width <= bounds[0] + bounds[2] + 0.001,
                "horizontal containment {wrapper}: {node:?}"
            );
            assert!(
                y - size >= bounds[1] && y + size / 4.5 <= bounds[1] + bounds[3],
                "vertical containment {wrapper}"
            );
        }
        let title_bottom = xml
            .descendants()
            .filter(|n| {
                n.has_tag_name("text")
                    && n.parent().unwrap().attribute("id") == Some("document-title")
            })
            .map(|n| n.attribute("y").unwrap().parse::<f64>().unwrap() + 14. / 4.5)
            .fold(0., f64::max);
        let body_top = xml
            .descendants()
            .filter(|n| {
                n.has_tag_name("text")
                    && !n
                        .parent()
                        .unwrap()
                        .attribute("id")
                        .unwrap_or("")
                        .starts_with("document-")
            })
            .map(|n| n.attribute("y").unwrap().parse::<f64>().unwrap() - 24. * 1.2)
            .fold(f64::INFINITY, f64::min);
        assert!(title_bottom < body_top, "chrome overlaps {wrapper} body");
        usvg::Tree::from_str(&result.svg, &usvg::Options::default()).unwrap();
    }
}

#[test]
fn should_preserve_member_and_raw_note_commands_but_reject_interleaved_preambles() {
    // given
    for body in [
        "A -> B : message\ntitle Late",
        "class A\nskinparam defaultFontSize 20",
        "start\n<style>\nroot {\nFontSize 20\n}\n</style>\nstop",
    ] {
        // when
        let result = render(&source(body), &options());
        // then
        assert!(result.is_err(), "accepted interleaved {body}");
    }
    for body in [
        "class A {\ntitle\nlegend\nheader\n}",
        "A -> B : hi\nnote over B\ntitle raw\n  header raw\nend note",
    ] {
        // when
        let result = render(&source(body), &options()).unwrap();
        // then
        assert!(!result.svg.contains("document-title"));
        assert!(result.svg.contains("title"));
    }
}

#[test]
fn should_replace_chrome_regions_and_their_alignment() {
    // given
    let input =
        source("left header Old\nright header New\ntitle OldTitle\ntitle Final\nA -> B : message");
    // when
    let result = render(&input, &options()).unwrap();
    let xml = roxmltree::Document::parse(&result.svg).unwrap();
    // then
    assert!(!result.svg.contains("Old"));
    let header = xml
        .descendants()
        .find(|n| n.has_tag_name("text") && n.text() == Some("New"))
        .unwrap();
    assert!(header.attribute("x").unwrap().parse::<f64>().unwrap() > 24.);
    assert_eq!(
        xml.descendants()
            .filter(|n| n.attribute("id") == Some("document-header"))
            .count(),
        1
    );
}

#[test]
fn should_enforce_run_depth_and_all_public_limits_for_styled_input() {
    // given
    let input = source("title Top\nA -> B : **a //b __c --d--__//**");
    for limit in 0..6 {
        let mut config = options();
        match limit {
            0 => config.limits.source_bytes = 8,
            1 => config.limits.model_items = 5,
            2 => config.limits.nesting_depth = 3,
            3 => config.limits.layout_work = 100,
            4 => config.limits.svg_bytes = 400,
            _ => config.control.cancel(),
        }
        // when
        let result = render(&input, &config);
        // then
        assert!(result.is_err(), "ignored style limit {limit}");
    }
}

#[test]
fn should_keep_keyed_data_style_words_literal_and_reject_unsupported_data_preamble() {
    // given
    for key in [
        "skinparam",
        "title",
        "header",
        "title heading",
        "skinparam defaultFontName",
        "left header heading",
    ] {
        let input = format!("@startyaml\n{key}: literal\n@endyaml");
        // when
        let result = render(&input, &options()).unwrap();
        // then
        assert!(result.svg.contains(&format!(">{key}</text>")));
        assert!(!result.svg.contains("document-title"));
    }
    for command in [
        "skinparam defaultFontSize 20",
        "legend Extra",
        "left footer Extra",
        "center header Extra",
        "right header Extra",
    ] {
        // when
        let result = render(
            &format!("@startyaml\n{command}\nx: value\n@endyaml"),
            &options(),
        );
        // then
        assert!(result.is_err(), "accepted data command {command}");
    }
}

#[test]
fn should_reject_blank_font_families_and_alignment_markup() {
    // given
    for body in [
        "<style>\nroot {\nFontName \"  \"\n}\n</style>\nA -> B : x",
        "A -> B : <center>center",
        "A -> B : <left>left",
    ] {
        // when
        let result = render(&source(body), &options());
        // then
        assert!(result.is_err(), "accepted {body}");
    }
}

#[test]
fn should_cancel_between_styled_run_host_calls() {
    struct Canceller(diagram_engine::RenderControl, Arc<Mutex<Vec<String>>>);
    impl HostTextMeasurer for Canceller {
        fn measure(&self, request: HostTextMeasurementRequest<'_>) -> HostMeasurementResult {
            self.1.lock().unwrap().push(request.text.into());
            if request.text == "stop" {
                self.0.cancel();
            }
            Ok(None)
        }
    }
    // given
    let mut config = options();
    let calls = Arc::new(Mutex::new(Vec::new()));
    config.text_measurement = TextMeasurementPolicy::host_display(
        TextMeasurementProfileIdentity::new(
            MeasurementProfileId::new("cancel-styled-run").unwrap(),
            "1",
        )
        .unwrap(),
        Arc::new(Canceller(config.control.clone(), calls.clone())),
        [TextMeasurementPhase::Layout],
    );
    // when
    let result = render(&source("A -> B : before **stop**//later//"), &config);
    // then
    assert!(matches!(
        result,
        Err(diagram_engine::RenderError::Cancelled(_))
    ));
    let calls = calls.lock().unwrap();
    assert!(calls.iter().any(|text| text == "stop"));
    assert!(!calls.iter().any(|text| text == "later"));
}
