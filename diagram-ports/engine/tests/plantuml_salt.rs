use diagram_engine::{Language, RenderOptions, render};

fn options() -> RenderOptions {
    RenderOptions {
        language: Language::PlantUml,
        ..Default::default()
    }
}
fn input(body: &str) -> String {
    format!("@startsalt\n{body}\n@endsalt")
}
fn svg(body: &str) -> String {
    let output = render(&input(body), &options()).unwrap();
    assert_eq!(output.family, "salt");
    usvg::Tree::from_str(&output.svg, &usvg::Options::default()).unwrap();
    output.svg
}

#[test]
fn should_keep_widget_kinds_and_checked_dispatch_distinct() {
    // given
    let body = "{\nText | \"value   \" | [Apply]\n[X] | [] | [ ] Off\n(X) Selected | () | ( ) Other\n[x]\n}";
    // when
    let output = svg(body);
    let xml = roxmltree::Document::parse(&output).unwrap();
    let kinds: Vec<_> = xml
        .descendants()
        .filter_map(|n| n.attribute("data-kind"))
        .filter(|k| k.starts_with("salt-") && *k != "salt-table" && *k != "salt-cell")
        .collect();
    // then
    assert_eq!(
        kinds,
        [
            "salt-text",
            "salt-field",
            "salt-button",
            "salt-checkbox-on",
            "salt-checkbox-off",
            "salt-checkbox-off",
            "salt-radio-on",
            "salt-radio-off",
            "salt-radio-off",
            "salt-button"
        ]
    );
    for (kind, shape, count) in [
        ("salt-checkbox-on", "polygon", 1),
        ("salt-radio-on", "ellipse", 2),
        ("salt-radio-off", "ellipse", 1),
        ("salt-field", "polyline", 3),
    ] {
        let widget = xml
            .descendants()
            .find(|n| n.attribute("data-kind") == Some(kind))
            .unwrap();
        assert_eq!(
            widget
                .descendants()
                .filter(|n| n.has_tag_name(shape))
                .count(),
            count
        );
    }
}

#[test]
fn should_own_nested_tables_and_same_row_spans_without_painting_stars() {
    // given
    let body = "{#\nAccount | \"alice   \"\n{+\n[X] Remember\n[ ] Newsletter\n(X) Standard\n( ) Express\n} | {\n[Cancel] | [Continue]\nWide status | *\n}\n}";
    // when
    let output = svg(body);
    let xml = roxmltree::Document::parse(&output).unwrap();
    let tables: Vec<_> = xml
        .descendants()
        .filter(|n| n.attribute("data-kind") == Some("salt-table"))
        .collect();
    let cells: Vec<_> = xml
        .descendants()
        .filter(|n| n.attribute("data-kind") == Some("salt-cell"))
        .collect();
    // then
    assert_eq!(tables.len(), 3);
    assert_eq!(cells.len(), 11);
    let root = tables[0].attribute("id").unwrap();
    assert_eq!(
        cells
            .iter()
            .filter(|c| c.attribute("data-owner") == Some(root))
            .count(),
        4
    );
    assert!(
        cells
            .iter()
            .any(|c| c.attribute("data-name") == Some("1:0:1:2"))
    );
    assert!(
        !xml.descendants()
            .any(|n| n.has_tag_name("text") && n.text() == Some("*"))
    );
    assert_eq!(
        tables
            .iter()
            .map(|t| t.attribute("data-name").unwrap())
            .collect::<Vec<_>>(),
        ["grid", "outer", "none"]
    );
}

#[test]
fn should_use_closing_brace_terminator_for_parent_column_and_row() {
    // given
    let body = "{\n{ A } | B\n{ C }\nD\n}";
    // when
    let output = svg(body);
    let xml = roxmltree::Document::parse(&output).unwrap();
    let root = xml
        .descendants()
        .find(|n| n.attribute("data-kind") == Some("salt-table"))
        .unwrap()
        .attribute("id")
        .unwrap();
    // then
    assert_eq!(
        xml.descendants()
            .filter(|n| n.attribute("data-kind") == Some("salt-cell")
                && n.attribute("data-owner") == Some(root))
            .map(|n| n.attribute("data-name").unwrap())
            .collect::<Vec<_>>(),
        ["0:0:1:1", "0:1:1:1", "1:0:1:1", "2:0:1:1"]
    );
}

#[test]
fn should_preserve_salt_raw_apostrophes_escaped_delimiters_and_pua() {
    // given
    let body = "{+\n' literal /' retained | \"value\"\nLabel \\| pipe | Brace \\{literal\\}\n\u{e000}\u{e001}\u{e002} | https://example.test/a\n}";
    // when
    let output = svg(body);
    // then
    for label in [
        "literal /",
        "retained",
        "Label | pipe",
        "Brace {literal}",
        "\u{e000}\u{e001}\u{e002}",
        "https://example.test/a",
    ] {
        assert!(output.contains(label), "{label}: {output}");
    }
}

#[test]
fn should_reserve_field_padding_and_real_colspan_width() {
    // given
    let plain = svg("{\n\"a\"\n}");
    let padded = svg("{\n\"a            \"\n}");
    let span = svg("{#\nA | B\n\"this is a wide field             \" | *\n}");
    let width = |source: &str| {
        roxmltree::Document::parse(source)
            .unwrap()
            .root_element()
            .attribute("width")
            .unwrap()
            .parse::<f64>()
            .unwrap()
    };
    // when / then
    assert!(width(&padded) >= width(&plain) + 80.);
    assert!(width(&span) > width(&padded));
}

#[test]
fn should_draw_only_selected_table_borders() {
    // given
    for (opener, count) in [("{", 0), ("{+", 1), ("{#", 5)] {
        // when
        let output = svg(&format!("{opener}\nA | B\nC | D\n}}"));
        let xml = roxmltree::Document::parse(&output).unwrap();
        let table = xml
            .descendants()
            .find(|n| n.attribute("data-kind") == Some("salt-table"))
            .unwrap();
        // then
        assert_eq!(
            table
                .descendants()
                .filter(|n| n.has_tag_name("rect") || n.has_tag_name("polyline"))
                .count(),
            count,
            "{opener}"
        );
    }
}

#[test]
fn should_reject_orphan_spans_and_unconsumed_or_unsupported_structure() {
    // given
    for body in [
        "{\n* | A\n}",
        "{\nA\n*\n}",
        "{ A",
        "{ A }}",
        "{ A } trailing",
        "{T#\nA\n}",
        "{^ Title\nA\n}",
        "{S\nA\n}",
        "{* A }",
        "{/ A }",
        "{\nN= | [Top]\n}",
        "{\n^Choice^\n}",
        "{\n<&image>\n}",
        "{\n<<dictionary>>\n}",
        "{\n[[https://example.test]]\n}",
        "{\n<img:foo>\n}",
    ] {
        // when
        let result = render(&input(body), &options());
        // then
        assert!(result.is_err(), "accepted {body:?}");
    }
}

#[test]
fn should_apply_shared_styles_chrome_and_literal_constants_to_salt() {
    // given
    let body = "!define NAME Ada\nskinparam defaultFontName Liberation Sans\nskinparam defaultFontSize 24\nskinparam defaultFontColor #123456\ntitle Controls\n{+\nName | \"NAME   \"\n[**Save**] | [X] Ready\n}";
    // when
    let output = svg(body);
    // then
    for value in [
        "Ada",
        "Controls",
        "#123456",
        "font-size=\"24\"",
        "font-weight=\"700\"",
        "Liberation Sans",
    ] {
        assert!(output.contains(value), "{value}");
    }
}

#[test]
fn should_bound_source_model_depth_work_output_and_cancel_before_measurement() {
    // given
    let source = input("{#\nA | \"B   \"\n{ [X] C } | [Done]\n}");
    for resource in ["source", "model", "depth", "work", "svg", "cancel"] {
        let mut config = options();
        match resource {
            "source" => config.limits.source_bytes = 16,
            "model" => config.limits.model_items = 2,
            "depth" => config.limits.nesting_depth = 1,
            "work" => config.limits.layout_work = 10,
            "svg" => config.limits.svg_bytes = 4096,
            _ => config.control.cancel(),
        }
        let source = if resource == "svg" {
            input(&format!(
                "{{#\n{}\n}}",
                (0..8).map(|_| "[A]").collect::<Vec<_>>().join("\n")
            ))
        } else {
            source.clone()
        };
        // when
        let error = render(&source, &config).unwrap_err().to_string();
        // then
        let reason = match resource {
            "source" => "16-byte",
            "model" => "model items",
            "depth" => "nesting depth",
            "work" => "work units",
            "svg" => "SVG bytes",
            _ => "cancel",
        };
        assert!(error.contains(reason), "{resource}: {error}");
    }
}

#[test]
fn should_render_exact_reference_fixtures_deterministically() {
    // given
    for source in [
        include_str!("../../fixtures/plantuml/saltbase.puml"),
        include_str!("../../fixtures/plantuml/saltwidgets.puml"),
        include_str!("../../fixtures/plantuml/saltliteral.puml"),
    ] {
        // when
        let a = render(source, &options()).unwrap();
        let b = render(source, &options()).unwrap();
        // then
        assert_eq!(a.family, "salt");
        assert_eq!(a.svg, b.svg);
        let tree = usvg::Tree::from_str(&a.svg, &usvg::Options::default()).unwrap();
        assert!(tree.size().width() > 0. && tree.size().height() > 0.);
    }
}

#[test]
fn should_preserve_escaped_leading_braces_and_split_unescaped_quoted_delimiters() {
    // given
    let body = "{\n\\{literal\\} | \"a\\|b\" | [c\\|d]\n}";
    // when
    let output = svg(body);
    // then
    assert!(output.contains("{literal}"));
    assert!(output.contains("a|b"));
    assert!(output.contains("c|d"));
    assert!(render(&input("{\n\"a|b\"\n}"), &options()).is_err());
}

#[test]
fn should_reject_separate_tree_markers_and_line_widgets_until_drawn() {
    // given
    for body in [
        "{\nT#\n+ Root\n}",
        "{\nT\nRoot\n}",
        "{\n----\n}",
        "{\n==section==\n}",
        "{\n....\n}",
        "{\n~~~~\n}",
    ] {
        // when
        let result = render(&input(body), &options());
        // then
        assert!(result.is_err(), "accepted structural widget {body:?}");
    }
}

#[test]
fn should_keep_ordinary_punctuation_and_resource_looking_data_literal() {
    // given
    let body = "{\n^ power | ? | C: notes\nN= label | https://example.test | /tmp/icon.png\nT literal | + label | ! literal\n}";
    // when
    let output = svg(body);
    // then
    for text in [
        "^ power",
        "C: notes",
        "N= label",
        "https://example.test",
        "/tmp/icon.png",
        "T literal",
        "+ label",
        "! literal",
    ] {
        assert!(output.contains(text), "{text}");
    }
}

#[test]
fn should_reject_wrong_wrappers_late_definitions_and_interleaved_style_requests() {
    // given
    for source in [
        "@startwire\n{ A }\n@endwire",
        "@STARTSALT\n{ A }\n@ENDSALT",
        "@startsalt target.svg\n{ A }\n@endsalt",
        "@startsalt\n{ A }\n@enduml",
        "@startsalt\n{ A }\n@endsalt\nextra",
        "@startsalt\n{\n!define X value\nX\n}\n@endsalt",
        "@startsalt\n{\nA\nskinparam defaultFontSize 30\n}\n@endsalt",
        "@startsalt\n{\nA\n<style>\n}\n@endsalt",
    ] {
        // when
        let result = render(source, &options());
        // then
        assert!(result.is_err(), "accepted {source}");
    }
}

#[test]
fn should_stop_salt_after_an_inflight_measurement_cancels() {
    use diagram_engine::{
        HostMeasurementResult, HostTextMeasurementRequest, HostTextMeasurer, MeasurementProfileId,
        RenderControl, TextMeasurementPhase, TextMeasurementPolicy, TextMeasurementProfileIdentity,
    };
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    struct Cancel {
        control: RenderControl,
        calls: AtomicUsize,
    }
    impl HostTextMeasurer for Cancel {
        fn measure(&self, _: HostTextMeasurementRequest<'_>) -> HostMeasurementResult {
            self.calls.fetch_add(1, Ordering::Relaxed);
            self.control.cancel();
            Ok(None)
        }
    }
    // given
    let mut config = options();
    let host = Arc::new(Cancel {
        control: config.control.clone(),
        calls: AtomicUsize::new(0),
    });
    config.text_measurement = TextMeasurementPolicy::host_display(
        TextMeasurementProfileIdentity::new(MeasurementProfileId::new("salt-cancel").unwrap(), "1")
            .unwrap(),
        host.clone(),
        [TextMeasurementPhase::Layout],
    );
    // when
    let error = render(&input("{\nFirst | Second\n}"), &config)
        .unwrap_err()
        .to_string();
    // then
    assert!(error.contains("cancel"), "{error}");
    assert_eq!(host.calls.load(Ordering::Relaxed), 1);
}

#[test]
fn should_reject_every_admitted_chrome_command_form_inside_tables() {
    // given
    let mut commands = vec![
        "title: Late title".to_owned(),
        "title : Late title".to_owned(),
        "legend: Late legend".to_owned(),
        "header: Late header".to_owned(),
        "footer: Late footer".to_owned(),
        "TiTlE: Late title".to_owned(),
    ];
    for alignment in ["left", "center", "right"] {
        for region in ["header", "footer"] {
            for separator in [" ", ": ", " : "] {
                commands.push(format!("{alignment} {region}{separator}Late chrome"));
            }
        }
    }
    commands.push("RiGhT\tHeAdEr:\tLate chrome".into());
    for command in &commands {
        let output = svg(&format!("{command}\n{{A}}"));
        assert!(
            output.contains("id=\"document-"),
            "missing prefix chrome for {command}"
        );
    }
    // when
    let accepted: Vec<_> = commands
        .iter()
        .filter(
            |command| match render(&input(&format!("{{\nA\n{command}\n}}")), &options()) {
                Ok(_) => true,
                Err(error) => {
                    assert!(
                        error
                            .to_string()
                            .contains("Salt styles/chrome must precede"),
                        "{command}: {error}"
                    );
                    false
                }
            },
        )
        .collect();
    // then
    assert!(accepted.is_empty(), "accepted late commands: {accepted:?}");
}

#[test]
fn should_preserve_literal_controls_and_noncommand_text_near_chrome_keywords() {
    // given
    let body = "{\n\"title: field\" | [left header: button]\n[X] right footer: checkbox | (X) center header: radio\nsubtitle: ordinary | left title: ordinary\nright headerboard: ordinary | titlecase ordinary\n}";
    // when
    let output = svg(body);
    let xml = roxmltree::Document::parse(&output).unwrap();
    // then
    for text in [
        "title: field",
        "left header: button",
        "right footer: checkbox",
        "center header: radio",
        "subtitle: ordinary",
        "left title: ordinary",
        "right headerboard: ordinary",
        "titlecase ordinary",
    ] {
        assert!(output.contains(text), "missing {text}");
    }
    for kind in [
        "salt-field",
        "salt-button",
        "salt-checkbox-on",
        "salt-radio-on",
    ] {
        assert_eq!(
            xml.descendants()
                .filter(|n| n.attribute("data-kind") == Some(kind))
                .count(),
            1,
            "{kind}"
        );
    }
}

#[test]
fn should_treat_inline_t_words_as_text_and_reserve_only_exact_tree_markers() {
    // given
    let mut rejected = Vec::new();
    // when
    for text in ["Text", "Textile", "T#literal"] {
        match render(&input(&format!("{{{text}}}")), &options()) {
            Ok(output) => assert!(output.svg.contains(text), "missing {text}"),
            Err(error) => rejected.push(format!("{text}: {error}")),
        }
    }
    for marker in ["T", "T#", "T+", "T-", "T!", "T^"] {
        let result = render(&input(&format!("{{{marker}\nRoot\n}}")), &options());
        assert!(result.is_err(), "accepted tree marker {marker}");
    }
    // then
    assert!(
        rejected.is_empty(),
        "rejected ordinary inline text: {rejected:?}"
    );
}
