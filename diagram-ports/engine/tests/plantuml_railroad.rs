use diagram_engine::{Language, RenderOptions, render};

fn options() -> RenderOptions {
    RenderOptions {
        language: Language::PlantUml,
        ..Default::default()
    }
}

fn checked(family: &str, body: &str) -> String {
    let source = format!("@start{family}\n{body}\n@end{family}");
    let result = render(&source, &options()).unwrap();
    assert_eq!(result.family, family);
    assert_eq!(result.svg, render(&source, &options()).unwrap().svg);
    usvg::Tree::from_str(&result.svg, &usvg::Options::default()).unwrap();
    result.svg
}

#[test]
fn should_render_ordered_ebnf_rules_with_distinct_terminal_reference_and_repeat_rails() {
    // given
    let source =
        "digit = \"digit\" | digit; next = missing, [ 'a;b,c' ], { digit }, { 'x' }-; empty = ;";
    // when
    let svg = checked("ebnf", source);
    let xml = roxmltree::Document::parse(&svg).unwrap();
    // then
    for kind in [
        "railroad-rule",
        "railroad-terminal",
        "railroad-reference",
        "railroad-choice",
        "railroad-bypass",
        "railroad-return",
        "railroad-empty",
    ] {
        assert!(
            xml.descendants()
                .any(|n| n.attribute("data-kind") == Some(kind)),
            "{kind}"
        );
    }
    assert_eq!(
        xml.descendants()
            .filter(|n| n.attribute("data-kind") == Some("railroad-rule"))
            .count(),
        3
    );
}

#[test]
fn should_render_regex_choice_classes_and_symbolic_repeats() {
    // given
    let source = r"INV-(A|B)[^a-c\]]+x{2,9}y?z*";
    // when
    let svg = checked("regex", source);
    // then
    for kind in [
        "railroad-choice",
        "railroad-class",
        "railroad-return",
        "railroad-bypass",
        "railroad-repeat-bound",
    ] {
        assert!(svg.contains(kind), "{kind}");
    }
}

#[test]
fn should_distinguish_repeat_return_and_zero_bypass_topology() {
    // given
    let cases = [("a+", 1, 0), ("a*", 1, 1), ("a?", 0, 1), ("a{2,9}", 1, 0)];
    // when
    let outputs =
        cases.map(|(source, returns, bypasses)| (checked("regex", source), returns, bypasses));
    // then
    for (svg, returns, bypasses) in outputs {
        let xml = roxmltree::Document::parse(&svg).unwrap();
        for (kind, count) in [("railroad-return", returns), ("railroad-bypass", bypasses)] {
            assert_eq!(
                xml.descendants()
                    .filter(|n| n.attribute("data-kind") == Some(kind))
                    .count(),
                count
            );
        }
    }
}

#[test]
fn should_reject_whole_unsupported_ebnf_constructs_and_regex_assertions_modifiers() {
    // given
    let cases = [
        ("ebnf", "a = b - c;"),
        ("ebnf", "a = 3 * b;"),
        ("ebnf", "a = (* visible *) b;"),
        ("ebnf", "a = ? special ?;"),
        ("regex", "a(?=b)"),
        ("regex", "a\\1"),
        ("regex", "(?i)a"),
        ("regex", "a+?"),
        ("regex", "a*+"),
        ("regex", "a{0}"),
        ("regex", "a{0,0}"),
        ("regex", "a^"),
        ("regex", "a\\p{L}"),
        ("regex", "a\nb"),
    ];
    // when
    let results = cases
        .map(|(family, body)| render(&format!("@start{family}\n{body}\n@end{family}"), &options()));
    // then
    for ((family, body), result) in cases.into_iter().zip(results) {
        assert!(
            matches!(
                result,
                Err(diagram_engine::RenderError::UnsupportedFeature(_))
            ),
            "{family}: {body}: {result:?}"
        );
    }
}

#[test]
fn should_reject_incomplete_or_malformed_grammar_instead_of_rendering_prefixes() {
    // given
    let cases = [
        ("ebnf", ""),
        ("ebnf", "a = b"),
        ("ebnf", "a = 'oops;"),
        ("ebnf", "a = [ b );"),
        ("ebnf", "a = b; garbage"),
        ("regex", ""),
        ("regex", "(ab"),
        ("regex", "[z-a]"),
        ("regex", "[]"),
        ("regex", "a{9,2}"),
        ("regex", "a{1,,3}"),
        ("regex", "a**"),
        ("regex", "a\\"),
    ];
    // when
    let results = cases
        .map(|(family, body)| render(&format!("@start{family}\n{body}\n@end{family}"), &options()));
    // then
    for ((family, body), result) in cases.into_iter().zip(results) {
        assert!(result.is_err(), "{family}: {body}");
    }
}

#[test]
fn should_preserve_regex_literal_quotes_slashes_spaces_and_ebnf_literal_comment_spelling() {
    // given
    let cases = [
        ("regex", "a 'b' // c"),
        ("regex", "title"),
        ("regex", "!include inert"),
        ("ebnf", "title = \"// ' (* inert *)\";"),
        ("ebnf", "a = 'https://example.invalid/a';"),
    ];
    // when
    let outputs = cases.map(|(family, body)| checked(family, body));
    // then
    for output in outputs {
        assert!(output.contains("railroad-terminal"));
    }
}

#[test]
fn should_keep_constant_generated_regex_directives_and_wrappers_as_literal_characters() {
    // given
    let sources = [
        "@startregex\n!define X @endregex\nX\n@endregex",
        "@startregex\n!define X !include\nX inert\n@endregex",
    ];
    // when
    let outputs = sources.map(|source| render(source, &options()));
    // then
    for result in outputs {
        assert!(result.is_ok(), "{result:?}");
    }
}

#[test]
fn should_admit_exact_depth64_and_reject_the_next_opener_in_both_families() {
    // given
    let nested = |family: &str, depth| {
        let expr = format!("{}a{}", "(".repeat(depth), ")".repeat(depth));
        if family == "ebnf" {
            format!("rule = {expr};")
        } else {
            expr
        }
    };
    // when
    for family in ["ebnf", "regex"] {
        let accepted = checked(family, &nested(family, 64));
        let source = format!("@start{family}\n{}\n@end{family}", nested(family, 65));
        let rejected = render(&source, &options()).unwrap_err();
        // then
        assert!(accepted.contains("railroad-reference") || accepted.contains("railroad-terminal"));
        assert!(
            rejected
                .to_string()
                .contains("nesting depth value 65 exceeds limit 64"),
            "{rejected}"
        );
    }
}

#[test]
fn should_observe_custom_source_model_work_output_depth_and_cancellation_limits() {
    // given
    let source = "@startregex\n(a|b)+c*\n@endregex";
    let mut controls = Vec::new();
    let mut source_bound = options();
    source_bound.limits.source_bytes = source.len() - 1;
    controls.push(source_bound);
    let mut items = options();
    items.limits.model_items = 2;
    controls.push(items);
    let mut work = options();
    work.limits.layout_work = 50;
    controls.push(work);
    let mut output = options();
    output.limits.svg_bytes = 512;
    controls.push(output);
    let mut depth = options();
    depth.limits.nesting_depth = 1;
    controls.push(depth);
    let cancelled = options();
    cancelled.control.cancel();
    controls.push(cancelled);
    // when
    let results = controls
        .iter()
        .enumerate()
        .map(|(i, o)| {
            if i == 4 {
                render("@startregex\n((a))\n@endregex", o)
            } else {
                render(source, o)
            }
        })
        .collect::<Vec<_>>();
    // then
    assert!(results.iter().all(Result::is_err), "{results:?}");
    let mut exact = options();
    exact.limits.source_bytes = source.len();
    assert!(render(source, &exact).is_ok());
}

#[test]
fn should_admit_bounded_flat_literals_and_multiline_rule_sets_with_tight_source_quota() {
    // given
    let cases = [
        ("regex", "x".repeat(200)),
        (
            "ebnf",
            (0..50)
                .map(|i| format!("r{i} = 'x', [missing];\n"))
                .collect::<String>(),
        ),
    ];
    // when
    let outputs = cases.map(|(family, body)| {
        let source = format!("@start{family}\n{body}\n@end{family}");
        let mut options = options();
        options.limits.source_bytes = source.len();
        render(&source, &options)
    });
    // then
    for output in outputs {
        assert!(output.is_ok(), "{output:?}");
    }
}

#[test]
fn should_bound_display_terminals_and_numeric_metadata_before_retaining_them() {
    // given
    let accepted = format!("r = '{}' ;", "x".repeat(16_384));
    let rejected = format!("r = '{}' ;", "x".repeat(16_385));
    // when
    checked("ebnf", &accepted);
    let terminal = render(&format!("@startebnf\n{rejected}\n@endebnf"), &options()).unwrap_err();
    let numeric = render("@startregex\na{00000000}\n@endregex", &options()).unwrap_err();
    // then
    assert!(
        terminal
            .to_string()
            .contains("label bytes value 16385 exceeds limit 16384"),
        "{terminal}"
    );
    assert!(
        numeric
            .to_string()
            .contains("regex repeat numeric bytes value 8 exceeds limit 7"),
        "{numeric}"
    );
}

#[test]
fn should_observe_cancellation_from_the_host_measurement_callback() {
    use diagram_engine::{
        HostMeasurementResult, HostTextMeasurementRequest, HostTextMeasurer, MeasurementProfileId,
        TextMeasurementPhase, TextMeasurementPolicy, TextMeasurementProfileIdentity,
    };
    struct Cancel(diagram_engine::RenderControl);
    impl HostTextMeasurer for Cancel {
        fn measure(&self, _request: HostTextMeasurementRequest<'_>) -> HostMeasurementResult {
            self.0.cancel();
            Ok(None)
        }
    }
    // given
    let mut config = options();
    config.text_measurement = TextMeasurementPolicy::host_display(
        TextMeasurementProfileIdentity::new(
            MeasurementProfileId::new("railroad-cancel").unwrap(),
            "1",
        )
        .unwrap(),
        std::sync::Arc::new(Cancel(config.control.clone())),
        [TextMeasurementPhase::Layout],
    );
    // when
    let result = render("@startregex\n(a|b)+\n@endregex", &config);
    // then
    assert!(
        matches!(result, Err(diagram_engine::RenderError::Cancelled(_))),
        "{result:?}"
    );
}
