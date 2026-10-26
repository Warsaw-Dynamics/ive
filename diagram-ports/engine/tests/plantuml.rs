use diagram_engine::{Language, RenderError, RenderOptions, render};

fn options() -> RenderOptions {
    RenderOptions {
        language: Language::PlantUml,
        ..Default::default()
    }
}

fn diagram(body: &str) -> String {
    format!("@startuml\n{body}\n@enduml")
}

#[test]
fn should_render_sequence_stereotypes_and_colored_participant_boxes() {
    // given
    let source = include_str!("cases/sequence_boxes.puml");
    // when
    let rendered = render(source, &options()).unwrap();
    let xml = roxmltree::Document::parse(&rendered.svg).unwrap();
    // then
    assert_eq!(rendered.family, "sequence");
    assert!(rendered.svg.contains("Application Web"));
    assert!(rendered.svg.contains("Serveur Play"));
    assert!(rendered.svg.contains("Contrôleur formulaire"));
    assert!(rendered.svg.contains("#add1b2"));
    assert_eq!(
        xml.descendants()
            .filter(|node| node.attribute("data-kind") == Some("sequence-box"))
            .count(),
        2
    );
}

#[test]
fn should_reject_unclosed_or_nested_sequence_boxes() {
    // given
    for body in [
        "box \"A\" #LightBlue\nparticipant A",
        "box \"A\" #LightBlue\nbox \"B\" #LightGreen\nparticipant A\nend box\nend box",
    ] {
        // when
        let result = render(&diagram(body), &options());
        // then
        assert!(result.is_err(), "accepted {body}");
    }
}

#[test]
fn should_reject_uml_quoted_tokens_beyond_raw_byte_limit() {
    // given
    let raw = "\\\"".repeat(8193);
    let mut accepted_over_limit = Vec::new();
    for (context, body) in [
        ("declaration", format!("class \"{raw}\" as A")),
        ("endpoint", format!("class A\nA --> \"{raw}\"")),
        ("left role", format!("class A\nclass B\nA \"{raw}\" --> B")),
        ("right role", format!("class A\nclass B\nA --> \"{raw}\" B")),
    ] {
        // when
        let result = render(&diagram(&body), &options());
        // then
        match result {
            Ok(_) => accepted_over_limit.push(context),
            Err(error) => assert!(
                matches!(error, RenderError::Backend(_))
                    && error.to_string().contains("label bytes"),
                "{context}: {error}"
            ),
        }
    }
    assert!(
        accepted_over_limit.is_empty(),
        "accepted 16,386 raw interior bytes in {accepted_over_limit:?}"
    );
}

#[test]
fn should_render_adjacent_quoted_sequence_command_primitives() {
    // given
    for body in [
        "participant\"A\"\nA -> B : hello",
        "participant A\nactivate\"A\"\nA -> B : hello\ndeactivate\"A\"",
        "\"participant\"\"Alpha\" as A\n\"activate\"\"A\"\nA -> B : hello\n\"deactivate\"\"A\"",
    ] {
        // when
        let rendered = render(&diagram(body), &options()).unwrap();
        // then
        assert_eq!(rendered.family, "sequence", "{body}");
        let xml = roxmltree::Document::parse(&rendered.svg).unwrap();
        let message = xml
            .descendants()
            .find(|node| node.attribute("data-kind") == Some("message"))
            .unwrap();
        assert_eq!(message.attribute("data-owner"), Some("p0"), "{body}");
        assert_eq!(message.attribute("data-target"), Some("p1"), "{body}");
        assert!(rendered.svg.contains("hello"), "{body}");
    }
}

#[test]
fn should_reject_reserved_first_primitives_as_sequence_message_endpoints() {
    // given
    for first in [
        "participant",
        "actor",
        "database",
        "activate",
        "deactivate",
        "create",
        "destroy",
        "note",
        "end",
    ] {
        for endpoint in [
            format!("{first}->B : hello"),
            format!("\"{first}\" -> B : hello"),
        ] {
            // when
            let result = render(&diagram(&endpoint), &options());
            // then
            assert!(
                matches!(
                    result,
                    Err(RenderError::InvalidSource(_) | RenderError::UnsupportedFeature(_))
                ),
                "accepted {endpoint}: {result:?}"
            );
        }
    }
}

#[test]
fn should_preserve_primitive_command_caption_and_note_ownership() {
    // given
    for (body, expected) in [
        (
            "participant A\n\"alt\" first\nA -> A : body\nend",
            "\"alt\"",
        ),
        (
            "participant A\ngroup\"joined\" title\nA -> A : body\nend",
            "group\"joined\"",
        ),
        (
            "participant A\nalt first\n\"else\" later\nA -> A : body\nend",
            "e\" later",
        ),
        (
            "participant A\n\"note\" over A\n  keep\n\n' ignored\nend note\nA -> A : body",
            "keep",
        ),
    ] {
        // when
        let rendered = render(&diagram(body), &options()).unwrap();
        // then
        assert_eq!(rendered.family, "sequence");
        let xml = roxmltree::Document::parse(&rendered.svg).unwrap();
        assert!(
            xml.descendants()
                .any(|node| node.has_tag_name("text") && node.text() == Some(expected)),
            "{body}"
        );
    }
}

#[test]
fn should_reject_malformed_quoted_sequence_frame_headers() {
    // given
    for command in [
        "group", "alt", "opt", "loop", "par", "break", "critical", "else",
    ] {
        for suffix in [
            "\"unterminated",
            "\"bad\\q\"",
            " \"quoted:unterminated",
            "::\"unterminated",
        ] {
            let header = format!("{command}{suffix}");
            let body = format!(
                "participant A\nalt outer\n{header}\nA -> A : body\nend\n{}",
                if command == "else" { "" } else { "end\n" }
            );
            // when
            let result = render(&diagram(&body), &options());
            // then
            assert!(
                matches!(
                    result,
                    Err(RenderError::InvalidSource(_) | RenderError::UnsupportedFeature(_))
                ),
                "accepted malformed header {header}"
            );
        }
    }
}

#[test]
fn should_keep_post_colon_sequence_frame_captions_literal() {
    // given
    for header in [
        "group: \"unterminated",
        "group\"joined\" : \"unterminated",
        "group \"valid:quoted\" : \"unterminated",
        "group \"escaped\\\"quote\" : \"unterminated",
        "else : \"unterminated",
    ] {
        let body = format!(
            "participant A\nalt outer\n{header}\nA -> A : body\nend\n{}",
            if header.starts_with("else") {
                ""
            } else {
                "end\n"
            }
        );
        // when
        let rendered = render(&diagram(&body), &options()).unwrap();
        // then
        let xml = roxmltree::Document::parse(&rendered.svg).unwrap();
        assert!(
            xml.descendants().any(|node| node.has_tag_name("text")
                && node
                    .text()
                    .is_some_and(|text| text.contains("\"unterminated"))),
            "{header}"
        );
    }
}

#[test]
fn should_reject_else_under_a_noncanonical_frame_spelling() {
    // given
    for head in ["\"alt\" first", "alt\"joined\" first"] {
        // when
        let result = render(
            &diagram(&format!(
                "participant A\n{head}\nelse later\nA -> A : body\nend"
            )),
            &options(),
        );
        // then
        assert!(
            matches!(result, Err(RenderError::InvalidSource(_))),
            "accepted {head}"
        );
    }
}

#[test]
fn should_render_implicit_state_descriptions_with_literal_arrow_text() {
    // given
    for label in [
        "waiting -> ready",
        "waiting --> ready",
        "waiting <- ready",
        "waiting ..> ready",
        "waiting o-- ready",
    ] {
        for declaration in ["state Idle\n", ""] {
            let source = diagram(&format!("{declaration}Idle : {label}"));
            // when
            let rendered = render(&source, &options()).unwrap();
            // then
            assert_eq!(rendered.family, "state", "{source}");
            let xml = roxmltree::Document::parse(&rendered.svg).unwrap();
            assert!(
                xml.descendants()
                    .any(|node| node.has_tag_name("text") && node.text() == Some(label)),
                "{source}"
            );
        }
    }
}

#[test]
fn should_render_keyword_prefixed_dotted_and_at_endpoints_as_implicit_sequence() {
    // given
    for endpoint in [
        "class.foo",
        "class@foo",
        "state.foo",
        "state@foo",
        "start.foo",
        "note@foo",
        "robust.foo",
        "concise@foo",
    ] {
        for declaration in [format!("participant {endpoint}\n"), String::new()] {
            let source = diagram(&format!("{declaration}{endpoint} -> B : ping"));
            // when
            let rendered = render(&source, &options()).unwrap();
            // then
            assert_eq!(rendered.family, "sequence", "{source}");
            assert!(rendered.svg.contains(endpoint), "{source}");
            assert!(rendered.svg.contains("ping"), "{source}");
        }
    }
}

#[test]
fn should_reject_substantive_suffixes_and_validate_every_endpoint_branch() {
    // given
    for body in [
        "class A : discarded member",
        "object A : discarded",
        "component A : discarded",
        "participant A : discarded",
        "A -> B : ok\nactivate B : discarded",
        "A -> B : ok\nactivate B/role",
        "create B/role",
        "A -> B : ok\nnote over B/role : discarded",
        "class A\nA --> \"1\" B/role",
        "class A\nA/role \"1\" --> B",
        "A -> B : ok\ngroup g\nend : discarded",
    ] {
        // when
        let result = render(&diagram(body), &options());
        // then
        assert!(
            result.is_err(),
            "accepted substantive suffix/endpoint: {body}"
        );
    }
}

#[test]
fn should_reject_rich_text_variants_in_every_label_bearing_path() {
    // given
    for markup in [
        "<s:red>strike</s>",
        "<STRIKE:red>x</STRIKE>",
        "<DEL>x</DEL>",
        "<U:blue>x</U>",
        "<W>x</W>",
        "<Plain>x</Plain>",
        "<FONT:serif>x</FONT>",
        "<sup>x</sup>",
        "<sub>x</sub>",
        "~~wave~~",
        "\"\"mono\"\"",
        "<color red>x</color>",
    ] {
        for body in [
            format!("A -> B : {markup}"),
            format!("participant \"{}\" as A", markup.replace('"', "\\\"")),
            format!("class A {{\n{markup}\n}}"),
            format!("class A\nnote \"{}\" as N", markup.replace('"', "\\\"")),
            format!("\"{}\" -> B : hi", markup.replace('"', "\\\"")),
            format!("class A\nA --> \"{}\"", markup.replace('"', "\\\"")),
        ] {
            // when
            let result = render(&diagram(&body), &options());
            // then
            assert!(result.is_err(), "silently painted rich text: {body}");
        }
    }
}

#[test]
fn should_reject_url_adjacent_markup_in_every_label_bearing_path() {
    // given
    for markup in [
        "<s:red>strike</s>",
        "<b:red>bold</b>",
        "<STRIKE:red>x</STRIKE>",
        "<DEL>x</DEL>",
        "<FoNt:serif>x</FoNt>",
        "<color red>x</color>",
        "<IMG:asset>",
        "<$sprite>",
        "<&icon>",
        "<math>x</math>",
        "[[target]]",
        "**<img:asset>**",
        "__[[target]]__",
        "--<math>x</math>--",
        "~~wave~~",
    ] {
        let label = format!("https://example.test/{markup}");
        for body in label_bodies(&label) {
            // when
            let result = render(&diagram(&body), &options());
            // then
            assert!(
                matches!(result, Err(RenderError::UnsupportedFeature(_))),
                "accepted URL-adjacent markup: {body}"
            );
        }
    }
}

fn label_bodies(label: &str) -> [String; 8] {
    let quoted = label.replace('"', "\\\"");
    [
        format!("A -> B : {label}"),
        format!("participant \"{quoted}\" as A"),
        format!("class A {{\n{label}\n}}"),
        format!("class A\nnote \"{quoted}\" as N"),
        format!("\"{quoted}\" -> B : hi"),
        format!("class A\nA --> \"{quoted}\""),
        format!("class A\nclass B\nA --> \"{quoted}\" B"),
        format!("A -> B : hi\nnote over B : {label}"),
    ]
}

#[test]
fn should_normalize_url_adjacent_escapes_without_changing_literal_paths() {
    // given
    for (label, expected) in [
        (
            "https://example.test/a//b//c?x=1&y=2#part",
            "https://example.test/a//b//c?x=1&y=2#part",
        ),
        (
            "http://example.test/a--b/file_name.ext",
            "http://example.test/a--b/file_name.ext",
        ),
        (
            "ftp://example.test/a//b/<value>",
            "ftp://example.test/a//b/<value>",
        ),
        ("file:///tmp/a//b", "file:///tmp/a//b"),
        (
            "https://example.test/~<s>literal~</s>",
            "https://example.test/<s>literal</s>",
        ),
        (
            "https://example.test/~<B>literal~</B>",
            "https://example.test/<B>literal</B>",
        ),
        (
            "https://example.test/~<$sprite>",
            "https://example.test/<$sprite>",
        ),
        (
            "https://example.test/~**plain~**/~#part",
            "https://example.test/**plain**/#part",
        ),
    ] {
        for body in label_bodies(label) {
            // when
            let result = render(&diagram(&body), &options()).unwrap();
            let xml = roxmltree::Document::parse(&result.svg).unwrap();
            // then
            assert!(
                xml.descendants()
                    .filter_map(|node| node.text())
                    .any(|text| text == expected),
                "changed literal URL: {body}"
            );
        }
    }
}

#[test]
fn should_preserve_literal_angles_urls_and_escaped_markup() {
    // given
    let source = diagram(
        "A -> B : <value> https://example.test/a//b 1 < 2 ~<s>literal~</s> ~**plain~** ~#hash ~.dot \\~literal",
    );
    // when
    let result = render(&source, &options()).unwrap();
    let xml = roxmltree::Document::parse(&result.svg).unwrap();
    // then
    assert!(xml.descendants().filter_map(|n| n.text()).any(|text| text.contains("<value> https://example.test/a//b 1 < 2 <s>literal</s> **plain** #hash .dot ~literal")));
}

#[test]
fn should_report_actual_multiline_note_statement_line() {
    // given
    let source = "@startuml\nparticipant A\nA -> B : hello\nnote beside B\nbody\nend note\n@enduml";
    // when
    let error = render(source, &options()).unwrap_err();
    // then
    assert!(error.to_string().contains("line 4,"), "{error}");
}

#[test]
fn should_keep_unpaired_punctuation_and_empty_object_slot_strings_literal() {
    // given
    let source = diagram("object A {\nempty = \"\"\n}\nobject B\nA --> B : left -- right");
    // when
    let result = render(&source, &options()).unwrap();
    let xml = roxmltree::Document::parse(&result.svg).unwrap();
    // then
    assert!(
        xml.descendants()
            .filter_map(|n| n.text())
            .any(|text| text == "empty = \"\"")
    );
    assert!(
        xml.descendants()
            .filter_map(|n| n.text())
            .any(|text| text == "left -- right")
    );
}

#[test]
fn should_keep_colons_in_quoted_multiline_note_targets() {
    // given
    let source = diagram(
        "A -> \"B:role\" : hello\nnote over \"B:role\"\nfirst note line\nsecond note line\nend note",
    );
    // when
    let result = render(&source, &options()).unwrap();
    // then
    assert!(result.svg.contains("first note line"));
    assert!(result.svg.contains("second note line"));
}

#[test]
fn should_bound_adversarial_comment_scanning_with_a_tiny_deadline() {
    // given
    let source = diagram(&format!("{}A{}", " ".repeat(65536), "'".repeat(65536)));
    let mut config = options();
    config.limits.timeout = std::time::Duration::from_millis(10);
    config.limits.layout_work = 200_000;
    let started = std::time::Instant::now();
    // when
    let result = render(&source, &config);
    // then
    assert!(result.is_err());
    assert!(
        started.elapsed() < std::time::Duration::from_millis(500),
        "cooperative scanner stalled for {:?}",
        started.elapsed()
    );
}

#[test]
fn should_bound_expanded_namespace_text_before_retaining_keys() {
    // given
    let mut body = String::new();
    for i in 0..20 {
        body.push_str(&format!("namespace n{i}{} {{\n", "x".repeat(80)));
    }
    for i in 0..100 {
        body.push_str(&format!("class Leaf{i}\n"));
    }
    body.push_str(&"}\n".repeat(20));
    let source = diagram(&body);
    let mut config = options();
    config.limits.source_bytes = source.len();
    // when
    let actual = render(&source, &config);
    // then
    assert!(
        actual
            .unwrap_err()
            .to_string()
            .contains("expanded model text")
    );
}

#[test]
fn should_keep_ambiguous_actor_database_declarations_in_sequence_context() {
    // given
    let source = diagram("actor User\ndatabase Store\nUser -> Store : Read");
    // when
    let actual = render(&source, &options()).unwrap();
    // then
    assert_eq!(actual.family, "sequence");
    assert!(actual.svg.contains("data-kind=\"lifeline\""));
}

#[test]
fn should_keep_sequence_keywords_as_class_member_text() {
    // given
    let source = diagram("class Control {\nloop\nactivate\n}");
    // when
    let actual = render(&source, &options()).unwrap();
    // then
    assert_eq!(actual.family, "class");
    assert!(actual.svg.contains(">loop</text>"));
}

#[test]
fn should_keep_existing_description_relations_from_an_archimate_alias() {
    // given
    for relation in [
        "archimate --> B",
        "archimate \"one\" --> \"many\" B : keeps --> literal text",
        "\"archimate\" --> B",
    ] {
        let source = diagram(&format!("rectangle archimate\nrectangle B\n{relation}"));
        // when
        let actual = render(&source, &options()).unwrap();
        let xml = roxmltree::Document::parse(&actual.svg).unwrap();
        let edge = xml
            .descendants()
            .find(|n| n.attribute("data-kind") == Some("relation"))
            .unwrap();
        // then
        assert_eq!(actual.family, "description");
        assert_eq!(edge.attribute("data-owner"), Some("n0"));
        assert_eq!(edge.attribute("data-target"), Some("n1"));
        assert_eq!(
            xml.descendants()
                .filter(|n| n.attribute("data-kind") == Some("rectangle"))
                .count(),
            2
        );
        assert!(edge.descendants().any(|n| n.has_tag_name("polygon")));
    }
}

#[test]
fn should_reject_unimplemented_quoted_escape_instead_of_changing_label() {
    // given
    let source = diagram("participant \"A\\tB\" as A\nA -> B : Hello");
    // when
    let actual = render(&source, &options());
    // then
    assert!(matches!(actual, Err(RenderError::UnsupportedFeature(_))));
}

#[test]
fn should_draw_named_graph_notes_with_authored_links() {
    // given
    let source = diagram("class Account\nnote \"Check\\nidentity\" as N\nN .. Account : review");
    // when
    let result = render(&source, &options()).unwrap();
    let xml = roxmltree::Document::parse(&result.svg).unwrap();
    let note = xml
        .descendants()
        .find(|n| n.attribute("data-kind") == Some("note"))
        .unwrap();
    // then
    assert_eq!(result.family, "class");
    assert!(note.descendants().any(|n| n.has_tag_name("polygon")));
    assert!(note.descendants().any(|n| n.text() == Some("identity")));
    assert!(
        xml.descendants()
            .any(|n| n.attribute("data-kind") == Some("relation")
                && n.attribute("data-owner") == Some("n1")
                && n.attribute("data-target") == Some("n0"))
    );
}

#[test]
fn should_report_actual_host_success_and_invalid_answer_fallback() {
    use diagram_engine::{
        HostMeasurementResult, HostTextMeasurement, HostTextMeasurementRequest, HostTextMeasurer,
        MeasurementProfileId, TextMeasurementPhase, TextMeasurementPolicy,
        TextMeasurementProfileIdentity, TextMeasurementSource, TextMetrics,
    };
    struct PartialHost;
    impl HostTextMeasurer for PartialHost {
        fn measure(&self, request: HostTextMeasurementRequest<'_>) -> HostMeasurementResult {
            Ok(Some(HostTextMeasurement::Metrics(TextMetrics {
                width: if request.text == "native" {
                    55.
                } else {
                    f64::NAN
                },
                height: 20.,
                line_count: 1,
            })))
        }
    }
    // given
    let mut config = options();
    config.text_measurement = TextMeasurementPolicy::host_display(
        TextMeasurementProfileIdentity::new(
            MeasurementProfileId::new("partial-native-test").unwrap(),
            "1",
        )
        .unwrap(),
        std::sync::Arc::new(PartialHost),
        [TextMeasurementPhase::Layout],
    );
    // when
    let actual = render(&diagram("A -> B : native"), &config).unwrap();
    // then
    assert!(
        actual
            .measurement
            .entries()
            .iter()
            .any(|e| e.provenance().source == TextMeasurementSource::Host && e.count() > 0)
    );
    assert!(
        actual
            .measurement
            .entries()
            .iter()
            .any(|e| e.provenance().fallback_reason.is_some() && e.count() > 0)
    );
    assert!(
        actual
            .diagnostics
            .iter()
            .any(|d| d.contains("baseline") && d.contains("approximation"))
    );
}

#[test]
fn should_render_five_authorized_reference_sources() {
    // given
    let cases = [
        (
            include_str!("../../fixtures/plantuml/sequence.puml"),
            "sequence",
            &[
                "Browser",
                "Service",
                "Checkout",
                "Submit basket",
                "Check stock",
                "Accepted",
            ][..],
        ),
        (
            include_str!("../../fixtures/plantuml/class.puml"),
            "class",
            &[
                "Account",
                "Premium",
                "+owner : String",
                "+debit(amount)",
                "many",
                "records",
            ][..],
        ),
        (
            include_str!("../../fixtures/plantuml/object.puml"),
            "object",
            &[
                "basket : Cart",
                "line : Item",
                "total = 42",
                "sku = \"A7\"",
                "contains",
            ][..],
        ),
        (
            include_str!("../../fixtures/plantuml/deployment.puml"),
            "description",
            &[
                "Cluster",
                "Worker pool",
                "Gateway",
                "Worker",
                "Store",
                "dispatch",
                "save",
            ][..],
        ),
        (
            include_str!("../../fixtures/plantuml/usecase.puml"),
            "description",
            &["Shopper", "Store", "Place order", "Pay invoice", "includes"][..],
        ),
    ];
    for (source, family, labels) in cases {
        // when
        let actual = render(source, &options()).unwrap();
        let xml = roxmltree::Document::parse(&actual.svg).unwrap();
        let painted: Vec<_> = xml
            .descendants()
            .filter(|n| n.is_text())
            .filter_map(|n| n.text())
            .collect();
        // then
        assert_eq!(actual.family, family);
        assert_eq!(actual.language, Language::PlantUml);
        for label in labels {
            assert!(painted.contains(label), "missing {label:?} from {family}");
        }
        assert!(actual.layout_work_units > 0);
        assert!(!actual.measurement.entries().is_empty());
        assert!(usvg::Tree::from_str(&actual.svg, &usvg::Options::default()).is_ok());
    }
}

#[test]
fn should_reject_unsupported_syntax_with_a_source_location() {
    // given
    let cases = [
        "newpage",
        "skinparam foo bar",
        "!include secret.puml",
        "!define X Y",
        "<style>\nclass { Color red }\n</style>",
        "A -> B : hello\nunknown command",
        "& A -> B : parallel",
        "state S <<custom>>",
        "start\nbreak\nstop",
        "class A\nA -right-> B",
        "class A\nA::port --> B",
    ];
    for body in cases {
        // when
        let error = render(&diagram(body), &options()).unwrap_err().to_string();
        // then
        assert!(
            error.contains("PlantUML") && error.contains("line"),
            "{error}"
        );
        assert!(!error.contains("Mermaid"), "{error}");
    }
}

#[test]
fn should_reject_malformed_wrappers_comments_and_stacks() {
    // given
    let cases = [
        "@startuml\nA -> B\n@enduml\n@startuml\nA -> B\n@enduml",
        "@startuml\nA -> B",
        "@startuml\n/' unterminated\n@enduml",
        "@startuml\n@enduml",
        "@startjson\n{}\n@endyaml",
        "@startuml\nalt test\nA -> B\n@enduml",
        "@startuml\ndeactivate A\n@enduml",
        "@startuml\nclass A {\n@enduml",
    ];
    for source in cases {
        // when
        let actual = render(source, &options());
        // then
        assert!(actual.is_err(), "accepted {source:?}");
    }
}

#[test]
fn should_keep_quoted_apostrophes_urls_and_xml_text() {
    // given
    let source = "\u{feff}@startuml\r\n' comment\r\nparticipant \"O'Brien & Ω\" as A\r\nparticipant B\r\n/' ignored /' text '/\r\nA -> B : https://example.test?a=1&b=<value>\r\n@enduml";
    // when
    let result = render(source, &options()).unwrap();
    let document = roxmltree::Document::parse(&result.svg).unwrap();
    let text: Vec<_> = document
        .descendants()
        .filter(|n| n.is_text())
        .filter_map(|n| n.text())
        .collect();
    // then
    assert!(text.contains(&"O'Brien & Ω"));
    assert!(text.contains(&"https://example.test?a=1&b=<value>"));
    assert!(!result.svg.contains("href="));
}

#[test]
fn should_enforce_every_custom_resource_limit() {
    // given
    let source = diagram(
        "participant A\nparticipant B\ngroup outer\ngroup inner\nA -> B : long label\nend\nend",
    );
    for resource in ["source", "svg", "items", "depth", "work"] {
        let mut opts = options();
        match resource {
            "source" => opts.limits.source_bytes = 8,
            "svg" => opts.limits.svg_bytes = 32,
            "items" => opts.limits.model_items = 1,
            "depth" => opts.limits.nesting_depth = 1,
            _ => opts.limits.layout_work = 1,
        }
        // when
        let result = render(&source, &opts);
        // then
        assert!(result.is_err(), "unbounded {resource}");
    }
}

#[test]
fn should_validate_limits_and_cancellation_before_parsing() {
    // given
    let mut invalid = options();
    invalid.limits.nesting_depth = 0;
    let cancelled = options();
    cancelled.control.cancel();
    // when
    let first = render(&diagram("A -> B"), &invalid);
    let second = render(&diagram("A -> B"), &cancelled);
    // then
    assert!(matches!(first, Err(RenderError::InvalidOptions(_))));
    assert!(matches!(second, Err(RenderError::Cancelled(_))));
}

#[test]
fn should_preserve_sequence_direction_frames_and_lifecycle() {
    // given
    let source = diagram(
        "participant A\nparticipant B\nalt valid\nA -> B : request\nactivate B\nB -> B : self\nelse invalid\nA <-- B : reply\nend\ndeactivate B\ncreate C\nB -> C : create\ndestroy C",
    );
    // when
    let rendered = render(&source, &options()).unwrap();
    let xml = roxmltree::Document::parse(&rendered.svg).unwrap();
    let messages: Vec<_> = xml
        .descendants()
        .filter(|n| n.attribute("data-kind") == Some("message"))
        .collect();
    // then
    assert_eq!(messages.len(), 4);
    assert_eq!(messages[0].attribute("data-owner"), Some("p0"));
    assert_eq!(messages[0].attribute("data-target"), Some("p1"));
    assert_eq!(
        messages[1].attribute("data-owner"),
        messages[1].attribute("data-target")
    );
    assert_eq!(messages[2].attribute("data-owner"), Some("p1"));
    assert_eq!(messages[2].attribute("data-target"), Some("p0"));
    for kind in ["frame", "activation", "destroy"] {
        assert!(
            xml.descendants()
                .any(|n| n.attribute("data-kind") == Some(kind)),
            "{kind}"
        );
    }
}

#[test]
fn should_preserve_parallel_self_cyclic_and_disconnected_uml_edges() {
    // given
    let source = diagram(
        "class A\nclass B\nclass C\nA --> B : one\nA ..> B : two\nB --> A : back\nA --> A : self",
    );
    // when
    let rendered = render(&source, &options()).unwrap();
    let xml = roxmltree::Document::parse(&rendered.svg).unwrap();
    // then
    assert_eq!(
        xml.descendants()
            .filter(|n| n.attribute("data-kind") == Some("relation"))
            .count(),
        4
    );
    assert_eq!(
        xml.descendants()
            .filter(|n| n.attribute("data-kind") == Some("class"))
            .count(),
        3
    );
}

#[test]
fn should_preserve_scoped_aliases_and_genuine_description_shapes() {
    // given
    let source = diagram(
        "package Left {\nclass A\n}\npackage Right {\nclass A\n}\nLeft.A --> Right.A : cross",
    );
    // when
    let rendered = render(&source, &options()).unwrap();
    let xml = roxmltree::Document::parse(&rendered.svg).unwrap();
    let edge = xml
        .descendants()
        .find(|n| n.attribute("data-kind") == Some("relation"))
        .unwrap();
    // then
    assert_ne!(edge.attribute("data-owner"), edge.attribute("data-target"));
    assert_eq!(
        xml.descendants()
            .filter(|n| n.attribute("data-kind") == Some("package"))
            .count(),
        2
    );
}

#[test]
fn should_choose_sequence_when_actors_have_explicit_participant_context() {
    // given
    let source = diagram("actor Buyer\nparticipant Service\nBuyer -> Service : request");
    // when
    let result = render(&source, &options()).unwrap();
    // then
    assert_eq!(result.family, "sequence");
    assert!(result.svg.contains("data-kind=\"lifeline\""));
}

#[test]
fn should_reject_implicit_names_containing_unimplemented_syntax() {
    // given
    let bodies = [
        "A -> B++ : invalid",
        "A -> B! : invalid",
        "A -> B #red",
        "A -> B : <color:red>styled</color>",
        "A -> B : **<img:asset>**",
        "class A\nA --> B::port",
        "class A\nA --> B <<stereotype>>",
        "!if false\n!include secret\n!endif",
    ];
    for body in bodies {
        // when
        let actual = render(&diagram(body), &options());
        // then
        assert!(actual.is_err(), "silently accepted {body:?}");
    }
}

#[test]
fn should_reject_contradictory_family_declarations() {
    // given
    let source = diagram("class A\ncomponent B\nA --> B");
    // when
    let actual = render(&source, &options());
    // then
    assert!(actual.is_err(), "allowmixing is not implemented");
}

#[test]
fn should_observe_cancellation_after_a_host_measurement() {
    use diagram_engine::{
        HostMeasurementResult, HostTextMeasurementRequest, HostTextMeasurer, TextMeasurementPolicy,
        TextMeasurementProfileIdentity,
    };
    use std::sync::Arc;
    struct Canceller(diagram_engine::RenderControl);
    impl HostTextMeasurer for Canceller {
        fn measure(&self, _: HostTextMeasurementRequest<'_>) -> HostMeasurementResult {
            self.0.cancel();
            Ok(None)
        }
    }
    // given
    let mut opts = options();
    opts.text_measurement = TextMeasurementPolicy::host_display(
        TextMeasurementProfileIdentity::new(
            diagram_engine::MeasurementProfileId::new("cancel-test").unwrap(),
            "1",
        )
        .unwrap(),
        Arc::new(Canceller(opts.control.clone())),
        [diagram_engine::TextMeasurementPhase::Layout],
    );
    // when
    let actual = render(&diagram("A -> B : hello"), &opts);
    // then
    assert!(matches!(actual, Err(RenderError::Cancelled(_))));
}

#[test]
fn should_reject_label_markup_without_corrupting_literal_angle_text() {
    // given
    let unsupported = [
        "//<img:asset>//",
        "<u:blue>underline</u>",
        "<back:red>paint</back>",
        "[[https://example.test link]]",
        "<$icon>",
    ];
    for label in unsupported {
        // when
        let actual = render(&diagram(&format!("A -> B : {label}")), &options());
        // then
        assert!(actual.is_err(), "silently ignored {label:?}");
    }
}

#[test]
fn should_draw_distinct_uml_extremities_and_underlined_object_headers() {
    // given
    let source = diagram("object A\nobject B\nA o-- B\nA *-- B\nA <|-- B\nA ..|> B");
    // when
    let rendered = render(&source, &options()).unwrap();
    let xml = roxmltree::Document::parse(&rendered.svg).unwrap();
    // then
    assert_eq!(
        xml.descendants()
            .filter(|n| n.has_tag_name("polygon"))
            .count(),
        4
    );
    assert!(
        xml.descendants()
            .any(|n| n.has_tag_name("text") && n.attribute("text-decoration") == Some("underline"))
    );
}

#[test]
fn should_keep_nested_container_labels_clear_of_children() {
    // given
    let source = include_str!("../../fixtures/plantuml/deployment.puml");
    // when
    let rendered = render(source, &options()).unwrap();
    let xml = roxmltree::Document::parse(&rendered.svg).unwrap();
    let parent = xml
        .descendants()
        .find(|n| n.attribute("data-name") == Some("Cluster.Worker pool"))
        .unwrap();
    let child = xml
        .descendants()
        .find(|n| n.attribute("data-name") == Some("Cluster.Worker pool.Worker"))
        .unwrap();
    let parent_rect = parent.children().find(|n| n.has_tag_name("rect")).unwrap();
    let child_rect = child.children().find(|n| n.has_tag_name("rect")).unwrap();
    let number =
        |n: roxmltree::Node<'_, '_>, name| n.attribute(name).unwrap().parse::<f64>().unwrap();
    let label = parent
        .descendants()
        .find(|n| n.has_tag_name("text"))
        .unwrap();
    // then
    assert!(number(child_rect, "x") >= number(parent_rect, "x"));
    assert!(
        number(child_rect, "y") > number(label, "y") + 4.,
        "container caption overlaps child"
    );
    assert!(
        number(child_rect, "x") + number(child_rect, "width")
            <= number(parent_rect, "x") + number(parent_rect, "width")
    );
    assert!(
        number(child_rect, "y") + number(child_rect, "height")
            <= number(parent_rect, "y") + number(parent_rect, "height")
    );
}

#[test]
fn should_keep_command_prefixes_in_sequence_identifiers() {
    // given
    let source = diagram("titleClient -> scaleServer : hello");
    // when
    let actual = render(&source, &options()).unwrap();
    // then
    assert_eq!(actual.family, "sequence");
    assert!(actual.svg.contains("titleClient"));
}

#[test]
fn should_measure_italic_paint_with_italic_style_and_separate_cache_entries() {
    use diagram_engine::{
        HostMeasurementResult, HostTextMeasurementRequest, HostTextMeasurer, MeasurementProfileId,
        TextMeasurementPhase, TextMeasurementPolicy, TextMeasurementProfileIdentity,
    };
    use std::sync::{Arc, Mutex};
    struct Recorder(Arc<Mutex<Vec<(String, String)>>>);
    impl HostTextMeasurer for Recorder {
        fn measure(&self, request: HostTextMeasurementRequest<'_>) -> HostMeasurementResult {
            self.0.lock().unwrap().push((
                request.text.into(),
                request.style.font_style.clone().unwrap_or_default(),
            ));
            Ok(None)
        }
    }
    // given
    let calls = Arc::new(Mutex::new(Vec::new()));
    let mut opts = options();
    opts.text_measurement = TextMeasurementPolicy::host_display(
        TextMeasurementProfileIdentity::new(MeasurementProfileId::new("style-test").unwrap(), "1")
            .unwrap(),
        Arc::new(Recorder(calls.clone())),
        [TextMeasurementPhase::Layout],
    );
    let source = diagram(
        "class \"Same\" as A\nabstract class \"Same\" as B {\n{abstract} run()\n{static} count\n}\nA --> B",
    );
    // when
    let rendered = render(&source, &opts).unwrap();
    let recorded = calls.lock().unwrap();
    // then
    assert!(
        recorded
            .iter()
            .any(|(text, style)| text == "Same" && style == "italic")
    );
    assert!(
        recorded
            .iter()
            .any(|(text, style)| text == "Same" && style != "italic")
    );
    assert!(
        recorded
            .iter()
            .any(|(text, style)| text == "run()" && style == "italic")
    );
    assert!(rendered.svg.contains("text-decoration=\"underline\""));
}
