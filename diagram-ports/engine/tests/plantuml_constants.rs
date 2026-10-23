use diagram_engine::{Language, RenderOptions, render};

fn options() -> RenderOptions {
    RenderOptions {
        language: Language::PlantUml,
        ..Default::default()
    }
}

fn labels(source: &str) -> Vec<String> {
    let output = render(source, &options()).unwrap();
    let xml = roxmltree::Document::parse(&output.svg).unwrap();
    xml.descendants()
        .filter(|node| node.has_tag_name("text"))
        .map(|node| node.text().unwrap_or("").to_owned())
        .collect()
}

#[test]
fn should_freeze_prior_values_without_rescanning_insertions() {
    // given
    let source = "@startuml\n!define FIRST SECOND\n!define SECOND Ready\n!define NEXT SECOND now\nA -> B : FIRST | SECOND | NEXT\n@enduml";
    // when
    let text = labels(source);
    // then
    assert!(text.contains(&"SECOND | Ready | Ready now".into()));
}

#[test]
fn should_keep_quotes_and_mutual_looking_values_literal() {
    // given
    let source = "@startuml\n!define X Y\n!define Y X\n!define QUOTED \"Ready\"\nA -> B : X | Y | QUOTED\n@enduml";
    // when
    let text = labels(source);
    // then
    assert!(text.contains(&"Y | Y | \"Ready\"".into()));
}

#[test]
fn should_expand_case_sensitive_boundaries_inside_quotes_and_after_backslash_n() {
    // given
    let source = "@startuml\n!define X Ready\nparticipant \"X\" as A\nA -> B : X X1 aX X_a x (X)\\nX\n@enduml";
    // when
    let text = labels(source);
    // then
    assert!(text.contains(&"Ready X1 aX X_a x (Ready)".into()));
    assert!(
        text.iter()
            .filter(|value| value.as_str() == "Ready")
            .count()
            >= 2
    );
}

#[test]
fn should_reject_shadowing_and_late_or_outside_definitions() {
    // given
    for source in [
        "@startuml\n!define X one\n!define X two\nA -> B\n@enduml",
        "@startuml\nA -> B\n!define X one\n@enduml",
        "!define X one\n@startuml\nA -> B\n@enduml",
    ] {
        // when
        let error = render(source, &options()).unwrap_err().to_string();
        // then
        assert!(error.contains("definition"), "{error}");
    }
}

#[test]
fn should_map_expanded_style_failure_to_use_and_definition() {
    // given
    let source =
        "@startuml\n!define SIZE enormous\n  skinparam defaultFontSize SIZE\nA -> B\n@enduml";
    // when
    let error = render(source, &options()).unwrap_err().to_string();
    // then
    assert!(error.contains("line 3, column 29"), "{error}");
    assert!(
        error.contains("SIZE") && error.contains("defined at line 2"),
        "{error}"
    );
}

#[test]
fn should_preserve_data_and_raw_note_payloads_after_expansion() {
    // given
    let json = "@startjson\n!define VALUE ready\n{\"label\":\"VALUE **literal** \\\\n\"}\n@endjson";
    let note = "@startuml\n!define VALUE ready\nparticipant A\nnote over A\n  VALUE  \n\nend note\n@enduml";
    // when
    let data = labels(json);
    let raw = labels(note);
    // then
    assert!(data.contains(&"ready **literal** \\n".into()));
    assert!(raw.contains(&"  ready  ".into()));
}

#[test]
fn should_preserve_inert_resource_and_dollar_percent_text() {
    // given
    let source = "@startuml\n!define TEXT !include https://example.test/x $5 50% %literal /tmp/file\nA -> B : TEXT\n@enduml";
    // when
    let text = labels(source);
    // then
    assert!(text.contains(&"!include https://example.test/x $5 50% %literal /tmp/file".into()));
}

#[test]
fn should_deny_generated_capabilities_and_active_tim_grammar() {
    // given
    for body in [
        "!define CMD !include /tmp/file\nCMD\nA -> B",
        "!define CMD @enduml\nCMD\nA -> B",
        "!define X(x) X(x)\nA -> B",
        "!function f()\n!include /tmp/file\n!endfunction\nA -> B",
        "!define X %getenv(\"HOME\")\nA -> B : X",
        "!define X $name()\nA -> B : X",
        "!define X value\nA -> B : ##X##",
    ] {
        // when
        let result = render(&format!("@startuml\n{body}\n@enduml"), &options());
        // then
        assert!(result.is_err(), "accepted {body}");
    }
}

#[test]
fn should_reject_ambiguous_non_ascii_adjacent_constant_boundaries() {
    // given
    for label in ["éX", "Xé", "😀X", "X\u{0301}", "X٢"] {
        // when
        let error = render(
            &format!("@startuml\n!define X value\nA -> B : {label}\n@enduml"),
            &options(),
        )
        .unwrap_err()
        .to_string();
        // then
        assert!(error.contains("non-ASCII constant boundary"), "{error}");
    }
}

#[test]
fn should_reuse_options_without_retaining_constants() {
    // given
    let config = options();
    let source = "@startuml\n!define X Ready\nA -> B : X\n@enduml";
    // when
    let first = render(source, &config).unwrap();
    let second = render(source, &config).unwrap();
    let plain = render("@startuml\nA -> B : X\n@enduml", &config).unwrap();
    // then
    assert_eq!(first.svg, second.svg);
    assert!(plain.svg.contains(">X</text>"));
}

#[test]
fn should_map_generated_label_resource_error_to_its_use_and_definition() {
    // given
    let source =
        "@startuml\n!define IMAGE <img:https://example.test/image>\nA -> B : IMAGE\n@enduml";
    // when
    let error = render(source, &options()).unwrap_err().to_string();
    // then
    assert!(error.contains("used at line 3, column 10"), "{error}");
    assert!(
        error.contains("IMAGE") && error.contains("defined at line 2"),
        "{error}"
    );
}

#[test]
fn should_preserve_unrelated_unicode_and_blank_line_admission() {
    // given
    let source = format!(
        "@startuml\n!define X Ready\n{}A -> B : X 世界 😀\n@enduml",
        "\n".repeat(50_000)
    );
    // when
    let text = labels(&source);
    // then
    assert!(text.contains(&"Ready 世界 😀".into()));
}

#[test]
fn should_reject_definitions_inside_style_sheets() {
    // given
    for opening in [
        "<style>",
        "!define SHEET <style>\nSHEET",
        "!define SHEET <style>\nSHEET /' inert '/",
    ] {
        let source = format!(
            "@startuml\n{opening}\nroot {{\n!define X blue\nFontColor #112233\n}}\n</style>\nA -> B\n@enduml"
        );
        // when
        let result = render(&source, &options());
        // then
        assert!(
            result.is_err(),
            "definition was swallowed from a style rule"
        );
    }
}

#[test]
fn should_report_bounded_candidates_for_identical_expanded_late_labels() {
    // given
    let source = "@startuml\n!define IMAGE <img:https://example.test/image>\nA -> B : IMAGE\nB -> A : IMAGE\n@enduml";
    // when
    let error = render(source, &options()).unwrap_err().to_string();
    // then
    assert!(error.contains("candidate expansion uses"), "{error}");
    assert!(
        error.contains("used at line 3, column 10") && error.contains("used at line 4, column 10"),
        "{error}"
    );
    assert!(error.contains("defined at line 2"), "{error}");
    assert!(error.contains("label-local line 1"), "{error}");
    assert!(!error.contains("PlantUML at line 1"), "{error}");
}

#[test]
fn should_identify_inert_duplicate_label_candidates_and_omitted_occurrences() {
    // given
    let source = format!(
        "@startuml\n!define IMAGE <img:x>\nA -> B : <img:x>\n{}@enduml",
        "A -> B : IMAGE\n".repeat(10)
    );
    // when
    let error = render(&source, &options()).unwrap_err().to_string();
    // then
    assert!(
        error.contains("direct candidate at line 3, column 10"),
        "{error}"
    );
    assert!(error.contains("3 textual occurrences omitted"), "{error}");
}

#[test]
fn should_reject_invalid_literal_controls_even_when_unused() {
    // given
    for literal in [
        "bad\0value",
        "bad\rvalue",
        "bad\u{000c}value",
        "continuation\\",
    ] {
        // when
        let error = render(
            &format!("@startuml\n!define X {literal}\nA -> B\n@enduml"),
            &options(),
        )
        .unwrap_err()
        .to_string();
        // then
        assert!(error.contains("literal"), "{error}");
    }
}

#[test]
fn should_preserve_hash_tree_syntax_and_inert_literal_hashes() {
    // given
    let tree = "@startmindmap\n!define LABEL Ready\n# Root\n## LABEL\n@endmindmap";
    let source = "@startuml\n!define HASH #\nA -> B : hashes HASH# literal##text\n@enduml";
    // when
    let tree_text = labels(tree);
    let sequence_text = labels(source);
    // then
    assert!(tree_text.contains(&"Ready".into()));
    assert!(sequence_text.contains(&"hashes ## literal##text".into()));
}

#[test]
fn should_preserve_inert_comments_without_preprocessor_execution() {
    // given
    let source = "@startuml\n' %getenv(\"HOME\")\n/'\n!define X hidden\n'/\n!define X Ready\nA -> B : X\n@enduml";
    // when
    let text = labels(source);
    // then
    assert!(text.contains(&"Ready".into()));
}

#[test]
fn should_keep_case_insensitive_uml_wrappers_and_unknown_names() {
    // given
    let source = "@STARTUML\n!define X Ready\nA -> B : X Unknown $X %X $other %other\n@ENDUML";
    // when
    let text = labels(source);
    // then
    assert!(text.contains(&"Ready Unknown $Ready %Ready $other %other".into()));
}

#[test]
fn should_preserve_inert_inline_comments_around_expanded_source() {
    // given
    let source = "@startuml\n!define X Ready\nA -> B : X /' %getenv(\"HOME\") '/\n@enduml";
    // when
    let text = labels(source);
    // then
    assert!(text.contains(&"Ready".into()));
}

#[test]
fn should_expand_raw_note_and_action_rows_before_comment_normalization() {
    // given
    for body in [
        "participant A\nnote over A\n' X\nend note",
        "start\n:opening\n' X\nending;\nstop",
    ] {
        let source = format!("@startuml\n!define X Ready\n{body}\n@enduml");
        // when
        let text = labels(&source);
        // then
        assert!(text.contains(&"' Ready".into()), "{text:?}");
    }
}

#[test]
fn should_ignore_block_commented_wrappers_before_definitions() {
    // given
    let source = "@startuml\n/'\n@enduml\n'/\n!define X Ready\nA -> B : X\n@enduml";
    // when
    let text = labels(source);
    // then
    assert!(text.contains(&"Ready".into()));
}

#[test]
fn should_recognize_comment_normalized_opening_wrappers() {
    // given
    let source = "@startuml /' wrapper comment '/\n!define X Ready\nA -> B : X\n@enduml";
    // when
    let text = labels(source);
    // then
    assert!(text.contains(&"Ready".into()));
}

#[test]
fn should_expand_wrapper_looking_raw_note_rows() {
    // given
    let source =
        "@startuml\n!define X Ready\nparticipant A\nnote over A\n@enduml X\nend note\n@enduml";
    // when
    let text = labels(source);
    // then
    assert!(text.contains(&"@enduml Ready".into()), "{text:?}");
}

#[test]
fn should_keep_comment_looking_action_openings_raw() {
    // given
    let source =
        "@startuml\n!define X Ready\nstart\n:opening /' literal\n' X\nending;\nstop\n@enduml";
    // when
    let text = labels(source);
    // then
    assert!(text.contains(&"' Ready".into()), "{text:?}");
    assert!(text.contains(&"opening /' literal".into()), "{text:?}");
}

#[test]
fn should_keep_raw_ownership_until_note_and_action_terminators() {
    // given
    for body in [
        "participant A\nnote over A\n@startuml X\n@enduml\n' X\nend note\nA -> B : X",
        "start\n:opening /' literal\n@enduml X\n' X\nending;  \n:next X;\nstop",
        "start\n:one /' literal X;  \n/'\n@enduml\n'/\n:next X;\nstop",
    ] {
        let source = format!("@startuml\n!define X Ready\n{body}\n@enduml");
        // when
        let text = labels(&source);
        // then
        assert!(text.iter().any(|label| label.contains("Ready")), "{text:?}");
        assert!(text.iter().all(|label| !label.contains('X')), "{text:?}");
    }
}

#[test]
fn should_preserve_literal_data_wrappers_and_strict_generated_admission() {
    // given
    let data = "@startyaml\n!define X Ready\nlabel: '@enduml X'\n@endyaml";
    // when
    let text = labels(data);
    // then
    assert!(text.contains(&"@enduml Ready".into()), "{text:?}");
    for body in [
        "!define END @enduml\nEND\nA -> B",
        "!define START @startuml\nSTART\nA -> B",
        "!define IMAGE <img:x>\nparticipant A\nnote over A\nIMAGE\nend note",
        "!define X Ready\nA -> B : X\n@enduml\n!define Y later",
    ] {
        assert!(render(&format!("@startuml\n{body}\n@enduml"), &options()).is_err());
    }
}

#[test]
fn should_reject_generated_wrapper_after_expanded_action_terminator() {
    // given
    let source = "@startuml\n!define FIN ;\n!define END @enduml\nstart\n:hello FIN\nstop\nEND\n";
    // when
    let error = render(source, &options()).unwrap_err().to_string();
    // then
    assert!(error.contains("generated directive or wrapper"), "{error}");
    assert!(error.contains("used at line 7, column 1"), "{error}");
    assert!(error.contains("defined at line 3, column 1"), "{error}");
}

#[test]
fn should_close_expanded_opening_and_continuation_actions_before_normal_syntax() {
    // given
    for action in [":hello FIN", ":opening /' literal\n' X\nending FIN"] {
        let source = format!(
            "@startuml\n!define FIN ;\n!define X Ready\nstart\n{action}\n' %getenv(\"HOME\")\n:next X;\nstop\n@enduml"
        );
        // when
        let text = labels(&source);
        // then
        assert!(text.contains(&"next Ready".into()), "{text:?}");
    }
}

#[test]
fn should_close_expanded_note_terminators_before_normal_syntax() {
    // given
    let prefix = "@startuml\n!define CLOSE end note\n!define END @enduml\n!define X Ready\nparticipant A\nnote over A\n@enduml X\nCLOSE\n";
    // when
    let text = labels(&format!("{prefix}' %getenv(\"HOME\")\nA -> B : X\n@enduml"));
    let error = render(&format!("{prefix}END"), &options())
        .unwrap_err()
        .to_string();
    // then
    assert!(text.contains(&"@enduml Ready".into()), "{text:?}");
    assert!(text.contains(&"Ready".into()), "{text:?}");
    assert!(error.contains("generated directive or wrapper"), "{error}");
}

#[test]
fn should_keep_note_rows_raw_when_expansion_removes_authored_terminator() {
    // given
    let source = "@startuml\n!define OPEN note over A\n!define CLOSE end note\n!define note body\n!define X Ready\nparticipant A\nOPEN\nend note\n' X\nCLOSE\n@enduml";
    // when
    let text = labels(source);
    // then
    assert!(text.contains(&"end body".into()), "{text:?}");
    assert!(text.contains(&"' Ready".into()), "{text:?}");
}
