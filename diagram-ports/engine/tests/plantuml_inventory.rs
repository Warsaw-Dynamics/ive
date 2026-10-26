use diagram_engine::{Language, RenderOptions, render};

fn options() -> RenderOptions {
    RenderOptions {
        language: Language::PlantUml,
        ..Default::default()
    }
}
fn source(family: &str, body: &str) -> String {
    format!("@start{family}\n{body}\n@end{family}")
}
fn svg(family: &str, body: &str) -> String {
    let output = render(&source(family, body), &options()).unwrap();
    assert_eq!(output.family, family);
    usvg::Tree::from_str(&output.svg, &usvg::Options::default()).unwrap();
    assert_eq!(
        output.svg,
        render(&source(family, body), &options()).unwrap().svg
    );
    output.svg
}
fn names(svg: &str, kind: &str) -> Vec<String> {
    roxmltree::Document::parse(svg)
        .unwrap()
        .descendants()
        .filter(|n| n.attribute("data-kind") == Some(kind))
        .map(|n| n.attribute("data-name").unwrap_or("").to_owned())
        .collect()
}

#[test]
fn should_keep_files_ordered_folder_reuse_and_duplicate_identity() {
    // given
    let body = "/b/x\n/a/z\n/b/y\n/b/x\n/a\n/empty/";
    // when
    let output = svg("files", body);
    // then
    assert_eq!(names(&output, "files-folder"), ["b", "a", "empty"]);
    assert_eq!(names(&output, "files-file"), ["x", "y", "x", "z", "a"]);
    assert!(!output.contains("data-kind=\"edge\""));
}

#[test]
fn should_keep_files_note_anchors_and_literal_names() {
    // given
    let body = "/../世界/a&b\n<note>\nFirst\nSecond\n</note>\n<note>\nAgain\n</note>\n/empty/\n<note>\nRoot\n</note>";
    // when
    let output = svg("files", body);
    let xml = roxmltree::Document::parse(&output).unwrap();
    let owners: Vec<_> = xml
        .descendants()
        .filter(|n| n.attribute("data-kind") == Some("files-note"))
        .map(|n| n.attribute("data-owner").unwrap_or(""))
        .collect();
    // then
    assert_eq!(owners, ["f2", "f2", "f0"]);
    assert_eq!(names(&output, "files-folder"), ["..", "世界", "empty"]);
    assert!(output.contains("a&amp;b"));
}

#[test]
fn should_reject_invalid_files_paths_notes_and_mixed_syntax() {
    // given
    for body in [
        "/",
        "//",
        "/a//b",
        "/a\nunknown",
        "/a\n<note>\nopen",
        "/a\ntitle: late",
        "/a\n!include x",
    ] {
        // when
        let result = render(&source("files", body), &options());
        // then
        assert!(result.is_err(), "accepted {body:?}");
    }
}

#[test]
fn should_conserve_packet_fragments_across_three_rows() {
    // given
    let body = "packetdiag {\ncolwidth=8;\n0-2: Prefix\n3-22: Payload\n}";
    // when
    let output = svg("packetdiag", body);
    let xml = roxmltree::Document::parse(&output).unwrap();
    let fragments: Vec<_> = xml
        .descendants()
        .filter(|n| n.attribute("data-kind") == Some("packet-fragment"))
        .map(|n| {
            (
                n.attribute("data-owner").unwrap(),
                n.attribute("data-name").unwrap(),
            )
        })
        .collect();
    // then
    assert_eq!(
        fragments,
        [
            ("p0", "0:0:3"),
            ("p1", "0:3:5"),
            ("p1", "1:0:8"),
            ("p1", "2:0:7")
        ]
    );
}

#[test]
fn should_place_packet_ranges_by_width_and_auto_after_declared_end() {
    // given
    let body = "colwidth=16\n0-3: Four\n8: Eight [len=8]\n* Next [len=4,height=2]";
    // when
    let output = svg("packetdiag", body);
    // then
    assert_eq!(
        names(&output, "packet-field"),
        ["0-3:Four", "8-15:Eight", "16-19:Next"]
    );
    assert_eq!(
        names(&output, "packet-fragment"),
        ["0:0:4", "0:4:8", "0:12:4"]
    );
}

#[test]
fn should_apply_packet_rtl_ticks_height_and_colwidth_one() {
    // given
    let body = "colwidth=8\nscale_interval=2\nscale_direction=rtl\nnode_height=50\nsame_height=true\n0-3: A\n4-7: B [height=2]";
    // when
    let output = svg("packetdiag", body);
    // then
    assert_eq!(
        names(&output, "packet-tick"),
        ["8", "7", "6", "5", "4", "3", "2", "1", "0"]
    );
    assert!(svg("packetdiag", "colwidth=1\n0: X").contains("packet-fragment"));
}

#[test]
fn should_reject_packet_invalid_ranges_attributes_and_braces() {
    // given
    for body in [
        "5-2: bad",
        "0: X [len=0]",
        "0: X [rotate=90]",
        "0: X [unknown=1]",
        "0: X [height=-1]",
        "99999999: X",
        "colwidth=0\n0: X",
        "node_height=0\n0: X",
        "scale_interval=0\n0: X",
        "packetdiag {\n0: X",
        "0: X\n}",
        "0: X [len=2 junk]",
        "0-1: X [len=4]",
    ] {
        // when
        let result = render(&source("packetdiag", body), &options());
        // then
        assert!(result.is_err(), "accepted {body:?}");
    }
}

#[test]
fn should_keep_board_stage_parent_and_activity_columns() {
    // given
    let body = "Checkout\n+ Draft\n++ Validate\n+ Submit\n+++ Ship\nAudit\n+ Record";
    // when
    let output = svg("board", body);
    let xml = roxmltree::Document::parse(&output).unwrap();
    let cards: Vec<_> = xml
        .descendants()
        .filter(|n| n.attribute("data-kind") == Some("board-card"))
        .map(|n| {
            (
                n.attribute("data-name").unwrap(),
                n.attribute("data-owner").unwrap_or(""),
            )
        })
        .collect();
    // then
    assert_eq!(
        cards,
        [
            ("Checkout", ""),
            ("Draft", "b0"),
            ("Validate", "b1"),
            ("Submit", "b0"),
            ("Ship", "b3"),
            ("Audit", ""),
            ("Record", "b5")
        ]
    );
    assert_eq!(names(&output, "board-stage").len(), 3);
}

#[test]
fn should_reject_orphan_board_release_and_denied_commands() {
    // given
    for body in [
        "+ Missing",
        "Root\nskinparam unknown value",
        "Root\ntitle: late",
        "Root\n!include x",
        "Root\nnewpage",
        "Root\n<style>",
    ] {
        // when
        let result = render(&source("board", body), &options());
        // then
        assert!(result.is_err(), "accepted {body:?}");
    }
}

#[test]
fn should_unwrap_hcl_and_keep_duplicate_fields_as_strings_and_owned_rows() {
    // given
    let body = "service \"billing\" {\nname = \"ledger\"\nname: \"again\"\ncount=3\nenabled=true\nnone=null\nzones=[\"north\",\"south\"]\ntags={tier=\"core\"}\n}";
    // when
    let output = svg("hcl", body);
    // then
    assert_eq!(
        names(&output, "data-row-string"),
        ["name", "name", "count", "enabled", "none", "0", "1", "tier"]
    );
    assert!(!output.contains("service &quot;billing&quot;"));
    let xml = roxmltree::Document::parse(&output).unwrap();
    let edges: Vec<_> = xml
        .descendants()
        .filter(|n| n.attribute("data-kind") == Some("data-link"))
        .map(|n| n.attribute("data-owner").unwrap())
        .collect();
    assert_eq!(edges, ["d0r5", "d0r6"]);
}

#[test]
fn should_keep_hcl_blocks_literal_escapes_comments_and_empty_containers() {
    // given
    let body = "# ignored\nservice \"世界\" { x=\"a\\nb & # https://example.test/a\" a=[] b={} }\nother { x=false }";
    // when
    let output = svg("hcl", body);
    // then
    assert!(output.contains("anb &amp; # https://example.test/a"));
    assert_eq!(names(&output, "data-object"), ["", "", "", ""]);
    assert!(output.contains("service &quot;世界&quot;"));
}

#[test]
fn should_reject_hcl_data_loss_evaluation_and_malformed_input() {
    // given
    for body in [
        "a{x=[1]}",
        "a{x=[[\"x\"]]}",
        "a{x=f(\"x\")}",
        "a{x=1+2}",
        "a{x=\"${x}\"}",
        "a{x=<<EOF}",
        "a{x=\"open}",
        "a{x={}",
        "a{x=1}\na{x=2}",
    ] {
        // when
        let result = render(&source("hcl", body), &options());
        // then
        assert!(result.is_err(), "accepted {body:?}");
    }
}

#[test]
fn should_preserve_leading_blank_files_note_rows() {
    // given
    let body = "/a\n<note>\n\ntext\n\n</note>";
    // when
    let output = svg("files", body);
    // then
    assert_eq!(names(&output, "files-note"), [" text "]);
}

#[test]
fn should_keep_hcl_command_named_blocks_and_reject_escaped_physical_newlines() {
    // given
    let body = "title \"named block\" { x=1 }\nfooter { y=2 }";
    // when
    let output = svg("hcl", body);
    // then
    assert!(output.contains("title &quot;named block&quot;"));
    assert!(output.contains("footer"));
    assert!(render(&source("hcl", "a{x=\"line\\\nbreak\"}"), &options()).is_err());
}

#[test]
fn should_enforce_inventory_limits_and_cancellation() {
    // given
    for (family, body) in [
        ("files", "/a/b/c"),
        ("packetdiag", "colwidth=1\n0-9999999: Huge"),
        ("board", "Root\n+++ Deep"),
        ("hcl", "a{x={y={z=1}}}"),
    ] {
        // when
        let mut small = options();
        small.limits.model_items = 3;
        let limited = render(&source(family, body), &small);
        let cancelled = options();
        cancelled.control.cancel();
        let stopped = render(&source(family, body), &cancelled);
        // then
        assert!(limited.is_err(), "unbounded {family}");
        assert!(stopped.is_err(), "uncancelled {family}");
    }
}

#[test]
fn should_measure_shared_display_markup_and_reject_actual_resource_labels() {
    // given
    for (family, body) in [
        ("files", "/a\n<note>\n**Emphasis**\n</note>"),
        ("packetdiag", "0-3: **Emphasis**"),
        ("board", "Root\n+ **Emphasis**"),
    ] {
        // when
        let output = svg(family, body);
        // then
        assert!(
            output.contains("font-weight=\"700\""),
            "literalized markup in {family}"
        );
        assert!(
            render(
                &source(
                    family,
                    &body.replace("**Emphasis**", "<img:https://example.test/a.png>")
                ),
                &options()
            )
            .is_err(),
            "resource in {family}"
        );
    }
    assert!(svg("files", "/a/<img:literal>").contains("&lt;img:literal&gt;"));
    assert!(
        svg("hcl", "a{x=\"<img:https://example.test/a.png>\"}")
            .contains("&lt;img:https://example.test/a.png&gt;")
    );
}

#[test]
fn should_stop_each_inventory_family_after_the_first_cancelled_measurement() {
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
    for (family, body) in [
        ("files", "/a/b"),
        ("packetdiag", "0-3: First\n4-7: Next"),
        ("board", "Root\n+ Child"),
        ("hcl", "a{x=\"one\" y=\"two\"}"),
    ] {
        let mut config = options();
        let host = Arc::new(Cancel {
            control: config.control.clone(),
            calls: AtomicUsize::new(0),
        });
        config.text_measurement = TextMeasurementPolicy::host_display(
            TextMeasurementProfileIdentity::new(
                MeasurementProfileId::new("inventory-cancel").unwrap(),
                "1",
            )
            .unwrap(),
            host.clone(),
            [TextMeasurementPhase::Layout],
        );
        // when
        let error = render(&source(family, body), &config)
            .unwrap_err()
            .to_string();
        // then
        assert!(error.contains("cancel"), "{family}: {error}");
        assert_eq!(host.calls.load(Ordering::Relaxed), 1);
    }
}

#[test]
fn should_count_synthetic_hcl_multiblock_root_in_value_depth() {
    // given
    let mut config = options();
    config.limits.nesting_depth = 2;
    // when
    let single = render(&source("hcl", "a{x=1}"), &config);
    let multiple = render(&source("hcl", "a{x=1}\nb{y=2}"), &config);
    // then
    assert!(single.is_ok());
    assert!(
        multiple.is_err(),
        "synthetic root did not add a value depth"
    );
    config.limits.nesting_depth = 3;
    assert!(render(&source("hcl", "a{x=1}\nb{y=2}"), &config).is_ok());
}

#[test]
fn should_reject_repeated_packet_delimiters_and_unconsumed_attribute_suffixes() {
    // given
    for body in [
        "0::: X",
        "colwidth=8;;\n0: X",
        "0: X [height=1] extra",
        "0: X [height=1,height=2]",
        "packetdiag {\n0: X\n}\n0: Extra",
    ] {
        // when
        let result = render(&source("packetdiag", body), &options());
        // then
        assert!(result.is_err(), "accepted malformed packet input {body:?}");
    }
}

#[test]
fn should_admit_exact_original_inventory_reference_sources() {
    // given
    for (family, input) in [
        (
            "files",
            include_str!("../../fixtures/plantuml/filesbase.puml"),
        ),
        (
            "packetdiag",
            include_str!("../../fixtures/plantuml/packetbase.puml"),
        ),
        (
            "board",
            include_str!("../../fixtures/plantuml/boardbase.puml"),
        ),
        ("hcl", include_str!("../../fixtures/plantuml/hclbase.puml")),
    ] {
        // when
        let output = render(input, &options()).unwrap();
        // then
        assert_eq!(output.family, family);
        usvg::Tree::from_str(&output.svg, &usvg::Options::default()).unwrap();
    }
}

#[test]
fn should_expand_prefix_literals_in_all_inventory_families() {
    // given
    for (family, body) in [
        ("files", "/app/NAME"),
        ("packetdiag", "0-3: NAME"),
        ("board", "Root\n+ NAME"),
        ("hcl", "service {name=\"NAME\"}"),
    ] {
        let input = source(family, &format!("!define NAME Ready\n{body}"));
        // when
        let output = render(&input, &options()).unwrap();
        // then
        assert!(output.svg.contains("Ready"), "{family} literal missing");
    }
}

#[test]
fn should_preserve_significant_file_component_spaces_without_normalization() {
    // given
    let body = "/dir / a \n/dir /x ";
    // when
    let output = svg("files", body);
    // then
    assert_eq!(names(&output, "files-folder"), ["dir "]);
    assert_eq!(names(&output, "files-file"), [" a ", "x "]);
    assert!(render(&source("packetdiag", "*: X"), &options()).is_err());
}

#[test]
fn should_reject_hcl_title_statement_before_a_separate_block() {
    // given
    let input = source("hcl", "title Unsupported chrome\nservice { x = 1 }");
    // when
    let error = render(&input, &options()).unwrap_err().to_string();
    // then
    assert!(error.contains("line 2, column 1"), "{error}");
    assert!(error.contains("title Unsupported chrome"), "{error}");
    assert!(error.contains("block header"), "{error}");
}

#[test]
fn should_reject_hcl_style_statement_before_a_separate_block() {
    // given
    let input = source("hcl", "skinparam defaultFontSize 96\nservice { x = 1 }");
    // when
    let error = render(&input, &options()).unwrap_err().to_string();
    // then
    assert!(error.contains("line 2, column 1"), "{error}");
    assert!(error.contains("skinparam defaultFontSize 96"), "{error}");
    assert!(error.contains("block header"), "{error}");
}

#[test]
fn should_reject_hcl_combined_prefix_statements_without_lowering_them_away() {
    // given
    let input = source(
        "hcl",
        "title Unsupported chrome\nskinparam defaultFontSize 96\nservice { x = 1 }",
    );
    // when
    let error = render(&input, &options()).unwrap_err().to_string();
    // then
    assert!(error.contains("line 2, column 1"), "{error}");
    assert!(error.contains("title Unsupported chrome"), "{error}");
    assert!(error.contains("physical line"), "{error}");
}

#[test]
fn should_reject_hcl_header_token_continuation_after_any_complete_block() {
    // given
    for header in [
        "title Unsupported chrome",
        "ordinary incomplete header",
        "service billing",
    ] {
        let input = source(
            "hcl",
            &format!("first {{ x = 1 }}\n{header}\nnext {{ y = 2 }}"),
        );
        // when
        let error = render(&input, &options()).unwrap_err().to_string();
        // then
        assert!(error.contains("line 3, column 1"), "{error}");
        assert!(error.contains(header), "{error}");
    }
}

#[test]
fn should_keep_hcl_command_names_split_opening_braces_and_nested_value_newlines() {
    // given
    let body = "title \"named block\"\n{\nx =\n\"one\"\nnested =\n{\ny =\n\"two\"\n}\nitems =\n[\n\"north\",\n{ z = \"three\" }\n]\n}\nfooter\n# comment before the opening brace\n{\ny = 2\n}";
    // when
    let output = svg("hcl", body);
    // then
    assert!(output.contains("title &quot;named block&quot;"));
    assert!(output.contains("footer"));
    assert_eq!(names(&output, "data-row-string"), ["x", "y", "0", "z", "y"]);
    assert_eq!(names(&output, "data-link").len(), 5);
}
