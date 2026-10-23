use diagram_engine::{Language, RenderOptions, render};

fn svg(body: &str) -> String {
    render(
        &format!("@startuml\n{body}\n@enduml"),
        &RenderOptions {
            language: Language::PlantUml,
            ..Default::default()
        },
    )
    .unwrap()
    .svg
}

#[test]
fn should_reject_preprocessing_inside_raw_action_and_note_bodies() {
    // given
    for body in [
        ":before\n!include ive-review-nonexistent-include.puml\nafter;",
        ":before\n  !define VALUE replacement\nafter;",
        "note over A\n  !include ive-review-nonexistent-include.puml\nend note",
        "note over A\n!if true\ntext\n!endif\nend note",
    ] {
        // when
        let result = render(
            &format!("@startuml\n{body}\n@enduml"),
            &RenderOptions {
                language: Language::PlantUml,
                ..Default::default()
            },
        );
        // then
        let error = result
            .expect_err("raw body admitted preprocessing")
            .to_string();
        assert!(error.contains("line 3"), "{error}");
        assert!(error.contains('!'), "{error}");
    }
}

#[test]
fn should_reject_ancestor_state_references_without_synthesizing_local_copies() {
    // given
    for body in [
        "state A\nstate Outer {\nstate B\nA --> B\n}",
        "state Outer {\nstate Inner {\nstate B\nB --> A\n}\n}\nstate A",
        "state A\nstate Outer {\nstate A\nstate B\nA --> B\n}",
        "state Outer {\nstate A\n}\nstate A",
    ] {
        // when
        let result = render(
            &format!("@startuml\n{body}\n@enduml"),
            &RenderOptions {
                language: Language::PlantUml,
                ..Default::default()
            },
        );
        // then
        assert!(result.is_err(), "invented a local ancestor copy: {body}");
    }
}

#[test]
fn should_admit_sequence_participants_named_like_state_and_activity_commands() {
    // given
    for name in [
        "start",
        "stop",
        "state",
        "repeat",
        "fork",
        "partition",
        "class",
        "if",
        "while",
    ] {
        for declaration in [
            String::new(),
            format!("actor {name}\n"),
            format!("database {name}\n"),
        ] {
            let body = format!("{declaration}{name} -> Bob : hello");
            // when
            let result = render(
                &format!("@startuml\n{body}\n@enduml"),
                &RenderOptions {
                    language: Language::PlantUml,
                    ..Default::default()
                },
            )
            .unwrap_or_else(|error| panic!("{body}: {error}"));
            // then
            assert_eq!(result.family, "sequence", "{body}");
            assert!(result.svg.contains("hello"));
        }
    }
}

#[test]
fn should_attach_descriptions_to_punctuated_quoted_state_displays() {
    // given
    for display in ["well-known", "left < right", "a -> b"] {
        let body = format!("state \"{display}\" as S : description");
        // when
        let result = svg(&body);
        let xml = roxmltree::Document::parse(&result).unwrap();
        let labels: Vec<_> = xml
            .descendants()
            .filter(|n| n.has_tag_name("text"))
            .filter_map(|n| n.text())
            .collect();
        // then
        assert_eq!(labels, [display, "description"]);
    }
}

#[test]
fn should_preserve_raw_note_rows_after_keyword_named_sequence_messages() {
    // given
    let body =
        "state -> Bob : hello\nnote over Bob\n  state value  \n\n  start ! literal\nend note";
    // when
    let result = svg(body);
    let xml = roxmltree::Document::parse(&result).unwrap();
    // then
    let labels: Vec<_> = xml
        .descendants()
        .filter(|n| n.has_tag_name("text"))
        .filter_map(|n| n.text())
        .collect();
    assert!(labels.contains(&"  state value  "));
    assert!(labels.contains(&"  start ! literal"));
}

#[test]
fn should_keep_implicit_state_family_for_pseudostate_targets() {
    // given
    let body = "A --> [*] : finished";
    // when
    let result = svg(body);
    // then
    assert!(result.contains("state-final"));
    assert!(result.contains("finished"));
}

#[test]
fn should_reject_state_arrows_split_between_operator_primitives() {
    // given
    for arrow in [
        "- >", "-- >", "- ->", "- - >", "< -", "<- -", "< --", "-\t>", "<\t-",
    ] {
        let source = format!("@startuml\nstate A\nA {arrow} B\n@enduml");
        // when
        let result = render(
            &source,
            &RenderOptions {
                language: Language::PlantUml,
                ..Default::default()
            },
        );
        // then
        assert!(result.is_err(), "accepted split arrow {arrow:?}");
    }
}

#[test]
fn should_render_contiguous_state_arrows_across_removed_inline_comments() {
    // given
    for arrow in [
        "->",
        "-->",
        "<--",
        "-/'Ω'/>",
        "-/'gap'/-/'gap'/>",
        "</'gap'/-/'gap'/-",
    ] {
        // when
        let result = svg(&format!("state A\nA {arrow} B : literal - >"));
        let xml = roxmltree::Document::parse(&result).unwrap();
        // then
        assert_eq!(
            xml.descendants()
                .filter(|node| node.attribute("data-kind") == Some("state-transition"))
                .count(),
            1,
            "{arrow}"
        );
        assert!(
            xml.descendants()
                .any(|node| node.text() == Some("literal - >")),
            "{arrow}"
        );
    }
}

#[test]
fn should_reuse_initial_and_final_per_scope() {
    // given
    let body = "state One {\n[*] --> A\n[*] --> B\nA --> [*]\nB --> [*]\n}\nstate Two {\n[*] --> C\nC --> [*]\n}\n[*] --> One\nOne --> [*]";
    // when
    let result = svg(body);
    let xml = roxmltree::Document::parse(&result).unwrap();
    // then
    assert_eq!(
        xml.descendants()
            .filter(|n| n.attribute("data-kind") == Some("state-initial"))
            .count(),
        3
    );
    assert_eq!(
        xml.descendants()
            .filter(|n| n.attribute("data-kind") == Some("state-final"))
            .count(),
        3
    );
    assert_eq!(
        xml.descendants()
            .filter(|n| n.attribute("data-kind") == Some("state-transition"))
            .count(),
        8
    );
}

#[test]
fn should_bind_reverse_pseudostates_by_semantic_role() {
    // given
    let body = "state A\nA <-- [*]\n[*] <-- A";
    // when
    let result = svg(body);
    let xml = roxmltree::Document::parse(&result).unwrap();
    let initial = xml
        .descendants()
        .find(|n| n.attribute("data-kind") == Some("state-initial"))
        .unwrap()
        .attribute("id")
        .unwrap();
    let final_id = xml
        .descendants()
        .find(|n| n.attribute("data-kind") == Some("state-final"))
        .unwrap()
        .attribute("id")
        .unwrap();
    let edges: Vec<_> = xml
        .descendants()
        .filter(|n| n.attribute("data-kind") == Some("state-transition"))
        .collect();
    // then
    assert_eq!(edges[0].attribute("data-owner"), Some(initial));
    assert_eq!(edges[1].attribute("data-target"), Some(final_id));
}

#[test]
fn should_attach_ordered_state_descriptions_and_resolve_forward_composites() {
    // given
    let body = "[*] --> Root\nstate \"Root title\" as Root {\nRoot : entry / open\nstate Child : first\nChild : exit / close\nChild --> [*] : done\n}\nRoot : exit / finish";
    // when
    let result = svg(body);
    let xml = roxmltree::Document::parse(&result).unwrap();
    let root = xml
        .descendants()
        .find(|n| n.attribute("data-name") == Some("Root"))
        .unwrap();
    let child = xml
        .descendants()
        .find(|n| n.attribute("data-name") == Some("Child"))
        .unwrap();
    // then
    assert_eq!(root.attribute("data-kind"), Some("state-composite"));
    assert_eq!(child.attribute("data-owner"), root.attribute("id"));
    let labels: Vec<_> = root
        .descendants()
        .filter(|n| n.has_tag_name("text"))
        .filter_map(|n| n.text())
        .collect();
    assert_eq!(labels, ["Root title", "entry / open", "exit / finish"]);
}

#[test]
fn should_converge_only_live_activity_branches() {
    // given
    let body = "start\nif (Valid?) then (yes)\n:Persist;\nelse (no)\nstop\nendif\n:Reply;\nend";
    // when
    let result = svg(body);
    let xml = roxmltree::Document::parse(&result).unwrap();
    let stop = xml
        .descendants()
        .find(|n| n.attribute("data-kind") == Some("activity-stop"))
        .unwrap()
        .attribute("id")
        .unwrap();
    // then
    assert!(
        xml.descendants()
            .any(|n| n.attribute("data-kind") == Some("activity-merge"))
    );
    assert!(
        xml.descendants()
            .filter(|n| n.attribute("data-kind") == Some("activity-flow"))
            .all(|n| n.attribute("data-owner") != Some(stop))
    );
    assert!(
        xml.descendants()
            .any(|n| n.attribute("data-kind") == Some("activity-end"))
    );
}

#[test]
fn should_preserve_elseif_loop_fork_and_nested_partition_topology() {
    // given
    let body = "start\npartition Outer {\ngroup Inner {\nif (a?) then (a)\n:A;\nelseif (b?) then (b)\n:B;\nelse (other)\n:C;\nendif\nwhile (more?) is (yes)\n:Work;\nendwhile (no)\nrepeat\n:Try;\nrepeat while (retry?) is (again) not (done)\nfork\n:Left;\nfork again\n:Right;\nend fork\n}\n}\nstop";
    // when
    let result = svg(body);
    let xml = roxmltree::Document::parse(&result).unwrap();
    // then
    for kind in [
        "activity-if",
        "activity-while",
        "activity-repeat",
        "activity-fork",
        "activity-join",
        "activity-partition",
        "activity-group",
    ] {
        assert!(
            xml.descendants()
                .any(|n| n.attribute("data-kind") == Some(kind)),
            "missing {kind}"
        );
    }
    for guard in ["a", "b", "other", "yes", "no", "again", "done"] {
        assert!(
            xml.descendants()
                .any(|n| n.attribute("data-kind") == Some("activity-flow")
                    && n.attribute("data-name") == Some(guard)),
            "missing guard {guard}"
        );
    }
}

#[test]
fn should_keep_implicit_false_path_and_dead_branch_exit_contracts() {
    // given
    for (body, merges) in [
        ("if (ready?)\nstop\nendif\n:Next;", 1),
        ("if (ready?)\nstop\nelse\nend\nendif\n:Next;", 0),
    ] {
        // when
        let result = svg(body);
        let xml = roxmltree::Document::parse(&result).unwrap();
        // then
        assert_eq!(
            xml.descendants()
                .filter(|n| n.attribute("data-kind") == Some("activity-merge"))
                .count(),
            merges
        );
        for terminal in xml.descendants().filter(|n| {
            matches!(
                n.attribute("data-kind"),
                Some("activity-stop" | "activity-end")
            )
        }) {
            assert!(
                xml.descendants()
                    .filter(|n| n.attribute("data-kind") == Some("activity-flow"))
                    .all(|n| n.attribute("data-owner") != terminal.attribute("id"))
            );
        }
    }
}

#[test]
fn should_select_families_without_reading_payload_as_commands() {
    // given
    for (body, family) in [
        ("class A {\nstart\nloop\nstate B\n}", "class"),
        ("actor A\ndatabase D\nA -> D : [*] start", "sequence"),
        (
            "A -> B : hi\nnote over B\nstate C\nstart\nend note",
            "sequence",
        ),
        (":state A\n[*] --> B;\nstop", "activity"),
        ("class A\nnote \"hello\" as N\nN .. A", "class"),
    ] {
        // when
        let result = render(
            &format!("@startuml\n{body}\n@enduml"),
            &RenderOptions {
                language: Language::PlantUml,
                ..Default::default()
            },
        )
        .unwrap();
        // then
        assert_eq!(result.family, family, "{body}");
    }
}

#[test]
fn should_reject_unsupported_control_syntax_and_mismatched_stacks() {
    // given
    for body in [
        "state S <<choice>>",
        "state S {\n--\nstate B\n}",
        "state A\nA --> [H]",
        "state A\nA --> [ * ]",
        "state A\nA ..> B",
        "state A\nA -right-> B",
        "state A\nA --> B :",
        "start\n|lane|\n:Task;",
        "start\nswitch (x)\nendswitch",
        "start\nbreak",
        "start\nkill",
        "start\ndetach",
        "start\nbackward :x;",
        "start\nif (x)\nwhile (y)\nendif",
        "start\nfork\nif (x)\nfork again\nend fork",
        "start\nelse (no)",
        "start\nif (x)\nelse\nelseif (y)\nendif",
        "start\npartition P {\nif (x)\n}",
        "start\nunknown command",
        "start\n-> unsupported lane arrow;\n:A;",
    ] {
        // when
        let result = render(
            &format!("@startuml\n{body}\n@enduml"),
            &RenderOptions {
                language: Language::PlantUml,
                ..Default::default()
            },
        );
        // then
        assert!(result.is_err(), "accepted {body}");
    }
}

#[test]
fn should_keep_state_description_order_and_hyphenated_literal_text() {
    // given
    let body = "S : first-line\nstate S : second-line\nS : third <value>";
    // when
    let result = svg(body);
    let xml = roxmltree::Document::parse(&result).unwrap();
    let labels: Vec<_> = xml
        .descendants()
        .filter(|n| n.has_tag_name("text"))
        .filter_map(|n| n.text())
        .collect();
    // then
    assert_eq!(labels, ["S", "first-line", "second-line", "third <value>"]);
}

#[test]
fn should_preserve_text_commands_inside_class_members() {
    // given
    let body = "class C {\n:member\nnote over B\nlast\n}";
    // when
    let result = svg(body);
    // then
    assert!(result.contains(":member") && result.contains("note over B"));
}

#[test]
fn should_admit_exact_original_state_and_activity_fixtures_deterministically() {
    // given
    for (source, family) in [
        (include_str!("../../fixtures/plantuml/state.puml"), "state"),
        (
            include_str!("../../fixtures/plantuml/activity.puml"),
            "activity",
        ),
    ] {
        let options = RenderOptions {
            language: Language::PlantUml,
            ..Default::default()
        };
        // when
        let rendered = render(source, &options).unwrap();
        let again = render(source, &options).unwrap();
        // then
        assert_eq!(rendered.family, family);
        assert_eq!(rendered.svg, again.svg);
        assert!(usvg::Tree::from_str(&rendered.svg, &usvg::Options::default()).is_ok());
        let xml = roxmltree::Document::parse(&rendered.svg).unwrap();
        if family == "state" {
            assert_eq!(
                xml.descendants()
                    .filter(|n| n.attribute("data-kind") == Some("state-initial"))
                    .count(),
                3
            );
            let sealed = xml
                .descendants()
                .find(|n| n.attribute("data-name") == Some("Sealed"))
                .unwrap();
            let checked = xml
                .descendants()
                .find(|n| n.attribute("data-name") == Some("Checked"))
                .unwrap();
            let inspect = xml
                .descendants()
                .find(|n| {
                    n.attribute("data-kind") == Some("state-transition")
                        && n.attribute("data-name") == Some("inspect")
                })
                .unwrap();
            assert_eq!(inspect.attribute("data-owner"), sealed.attribute("id"));
            assert_eq!(inspect.attribute("data-target"), checked.attribute("id"));
            assert_eq!(
                sealed.attribute("data-owner"),
                checked.attribute("data-owner")
            );
        } else {
            for (guard, action) in [("yes", "Persist"), ("no", "Reject")] {
                let target = xml
                    .descendants()
                    .find(|n| {
                        n.attribute("data-kind") == Some("activity-action")
                            && n.attribute("data-name") == Some(action)
                    })
                    .unwrap();
                assert!(
                    xml.descendants()
                        .any(|n| n.attribute("data-kind") == Some("activity-flow")
                            && n.attribute("data-name") == Some(guard)
                            && n.attribute("data-target") == target.attribute("id"))
                );
            }
        }
    }
}

#[test]
fn should_enforce_tiny_state_and_activity_limits() {
    // given
    for source in [
        include_str!("../../fixtures/plantuml/state.puml"),
        include_str!("../../fixtures/plantuml/activity.puml"),
    ] {
        for limit in ["model", "depth", "work", "svg"] {
            let mut options = RenderOptions {
                language: Language::PlantUml,
                ..Default::default()
            };
            match limit {
                "model" => options.limits.model_items = 1,
                "depth" => options.limits.nesting_depth = 1,
                "work" => options.limits.layout_work = 1,
                _ => options.limits.svg_bytes = 10,
            }
            let nested = if limit == "depth" && source.contains("start\n") {
                "@startuml\nif (a)\nif (b)\n:A;\nendif\nendif\n@enduml"
            } else {
                source
            };
            // when
            let result = render(nested, &options);
            // then
            assert!(result.is_err(), "ignored {limit}: {source}");
        }
    }
}

#[test]
fn should_distinguish_synchronizing_join_from_merge_and_keep_dead_lanes_dead() {
    // given
    for (closer, kind) in [
        ("end fork {all}", "activity-join"),
        ("end merge {either}", "activity-merge"),
    ] {
        // when
        let result = svg(&format!("fork\n:Live;\nfork again\nstop\n{closer}\n:Next;"));
        let xml = roxmltree::Document::parse(&result).unwrap();
        let join = xml
            .descendants()
            .find(|n| n.attribute("data-kind") == Some(kind))
            .unwrap();
        // then
        assert_eq!(
            xml.descendants()
                .filter(|n| n.attribute("data-kind") == Some("activity-flow")
                    && n.attribute("data-target") == join.attribute("id"))
                .count(),
            1
        );
        let specification = if kind == "activity-join" {
            "{all}"
        } else {
            "{either}"
        };
        assert!(
            join.descendants()
                .any(|n| n.has_tag_name("text") && n.text() == Some(specification))
        );
    }
}

#[test]
fn should_reject_nonliteral_state_alias_forms() {
    // given
    for body in [
        "state A as B",
        "state \"A\" as \"B\"",
        "state \"A B\" {\nstate C\n}",
        "state Outer.Inner",
        "state A\nA --> unknown.Inner",
    ] {
        // when
        let result = render(
            &format!("@startuml\n{body}\n@enduml"),
            &RenderOptions {
                language: Language::PlantUml,
                ..Default::default()
            },
        );
        // then
        assert!(
            result.is_err(),
            "accepted unsupported alias/qualified declaration: {body}"
        );
    }
}

#[test]
fn should_preserve_action_opening_line_whitespace_and_comment_like_text() {
    // given
    let body = ":first /'literal'/  \n  next  \nlast;\nstop";
    // when
    let result = svg(body);
    let xml = roxmltree::Document::parse(&result).unwrap();
    let labels: Vec<_> = xml
        .descendants()
        .filter(|n| n.has_tag_name("text"))
        .map(|n| n.text().unwrap_or(""))
        .collect();
    // then
    assert_eq!(labels, ["first /'literal'/  ", "  next  ", "last"]);
}

#[test]
fn should_cancel_state_and_activity_inside_layout_measurement() {
    use diagram_engine::{
        HostMeasurementResult, HostTextMeasurementRequest, HostTextMeasurer, MeasurementProfileId,
        RenderControl, RenderError, TextMeasurementPhase, TextMeasurementPolicy,
        TextMeasurementProfileIdentity,
    };
    use std::sync::Arc;
    struct Canceller(RenderControl);
    impl HostTextMeasurer for Canceller {
        fn measure(&self, _: HostTextMeasurementRequest<'_>) -> HostMeasurementResult {
            self.0.cancel();
            Ok(None)
        }
    }
    // given
    for source in [
        include_str!("../../fixtures/plantuml/state.puml"),
        include_str!("../../fixtures/plantuml/activity.puml"),
    ] {
        let mut options = RenderOptions {
            language: Language::PlantUml,
            ..Default::default()
        };
        options.text_measurement = TextMeasurementPolicy::host_display(
            TextMeasurementProfileIdentity::new(
                MeasurementProfileId::new("control-cancel").unwrap(),
                "1",
            )
            .unwrap(),
            Arc::new(Canceller(options.control.clone())),
            [TextMeasurementPhase::Layout],
        );
        // when
        let result = render(source, &options);
        // then
        assert!(matches!(result, Err(RenderError::Cancelled(_))));
    }
}

#[test]
fn should_retain_interior_semicolons_and_raw_multiline_action_and_note_rows() {
    // given
    for (body, expected) in [
        (
            ":first; second\n\n  indented\n' literal\nlast;\nstop",
            vec!["first; second", "", "  indented", "' literal", "last"],
        ),
        (
            "A -> B : hi\nnote over B\nfirst\n\n  indented\n' literal\nend note",
            vec!["first", "", "  indented", "' literal"],
        ),
    ] {
        // when
        let result = svg(body);
        let xml = roxmltree::Document::parse(&result).unwrap();
        let group = xml
            .descendants()
            .find(|n| matches!(n.attribute("data-kind"), Some("activity-action" | "note")))
            .unwrap();
        let labels: Vec<_> = group
            .descendants()
            .filter(|n| n.has_tag_name("text"))
            .map(|n| n.text().unwrap_or(""))
            .collect();
        // then
        assert_eq!(labels, expected);
    }
}
