use diagram_engine::{Language, RenderOptions, render};

fn options() -> RenderOptions {
    RenderOptions {
        language: Language::PlantUml,
        ..Default::default()
    }
}

fn checked(source: &str) -> String {
    let result = render(source, &options()).unwrap();
    assert_eq!(result.family, "git");
    let tree = usvg::Tree::from_str(&result.svg, &usvg::Options::default()).unwrap();
    assert!(tree.size().width().is_finite() && tree.size().width() > 0.);
    assert!(tree.size().height().is_finite() && tree.size().height() > 0.);
    result.svg
}

#[test]
fn should_render_original_linear_history_as_owned_commits_and_older_parent_edges() {
    // given
    let source = include_str!("../../fixtures/plantuml/gitlinear.puml");
    // when
    let svg = checked(source);
    let xml = roxmltree::Document::parse(&svg).unwrap();
    // then
    assert_eq!(
        xml.descendants()
            .filter(|n| n.attribute("data-kind") == Some("git-commit"))
            .count(),
        3
    );
    let edges: Vec<_> = xml
        .descendants()
        .filter(|n| n.attribute("data-kind") == Some("git-parent"))
        .map(|n| {
            (
                n.attribute("data-owner").unwrap(),
                n.attribute("data-target").unwrap(),
            )
        })
        .collect();
    assert_eq!(edges, [("git0", "git1"), ("git1", "git2")]);
    for label in ["c3", "b2", "a1"] {
        assert!(svg.contains(label));
    }
}

#[test]
fn should_render_both_observed_merge_parents_and_their_shared_older_base() {
    // given
    let source = include_str!("../../fixtures/plantuml/git.puml");
    // when
    let svg = checked(source);
    let xml = roxmltree::Document::parse(&svg).unwrap();
    // then
    assert_eq!(
        xml.descendants()
            .filter(|n| n.attribute("data-kind") == Some("git-commit"))
            .count(),
        4
    );
    let mut edges: Vec<_> = xml
        .descendants()
        .filter(|n| n.attribute("data-kind") == Some("git-parent"))
        .map(|n| {
            (
                n.attribute("data-owner").unwrap(),
                n.attribute("data-target").unwrap(),
            )
        })
        .collect();
    edges.sort_unstable();
    assert_eq!(
        edges,
        [
            ("git0", "git1"),
            ("git0", "git2"),
            ("git1", "git3"),
            ("git2", "git3")
        ]
    );
    for label in ["merge4", "side3", "main2", "base1"] {
        assert!(svg.contains(label));
    }
}

#[test]
fn should_preserve_literal_git_comments_without_uml_comment_or_resource_interpretation() {
    // given
    let comment = "https://example.test/a?x=1 <img:literal> ' /\\ * @enduml !include";
    let source =
        format!("\u{feff}@startgit\r\n   * tip ({comment})\r\n   |\r\n   * base\r\n@endgit\r\n");
    // when
    let svg = checked(&source);
    let xml = roxmltree::Document::parse(&svg).unwrap();
    // then
    let texts: Vec<_> = xml
        .descendants()
        .filter(|node| node.has_tag_name("text"))
        .map(|node| node.text().unwrap_or(""))
        .collect();
    assert_eq!(texts.iter().filter(|&&text| text == comment).count(), 1);
    assert!(texts.contains(&"tip") && texts.contains(&"base"));
    assert!(
        xml.descendants()
            .all(|node| node.attribute("href").is_none())
    );
}

#[test]
fn should_reject_every_unmodeled_or_ambiguous_git_construct_without_partial_output() {
    // given
    let bodies = [
        "|\n* a",
        "* a\n|",
        "* a\n|\n|\\\n| * b\n* c",
        "* same\n* same",
        "* a-b\n* ab",
        "* id (unterminated",
        "* id (nested (x))",
        "* (missing)",
        "* id (done) extra",
        "* id extra",
        "* * second",
        "* id * second",
        " * m\n/|\\\n* a",
        "*-. join",
        "* a\n_",
        "* a\n.",
        "\t* a",
        "commit a",
        "branch main",
        "checkout main",
        "gitGraph\ncommit a",
        "title concealed\n* a",
        "<style>\n* a\n</style>",
        "!define X a\n* X",
    ];
    // when / then
    for body in bodies {
        let source = format!("@startgit\n{body}\n@endgit");
        assert!(render(&source, &options()).is_err(), "admitted {body:?}");
    }
    for source in [
        "@startGIT\n* a\n@endGIT",
        "@startgit result.svg\n* a\n@endgit",
        "@startgit\n* a\n@endgit\n* b",
        "@startgit\n* a\n@enduml",
    ] {
        assert!(render(source, &options()).is_err(), "admitted {source:?}");
    }
}

#[test]
fn should_bound_git_source_models_work_depth_output_and_pre_cancelled_delivery() {
    // given
    let source = "@startgit\n* top\n|\n* base\n@endgit";
    let mut source_limit = options();
    source_limit.limits.source_bytes = source.len() - 1;
    let mut models = options();
    models.limits.model_items = 2;
    let mut work = options();
    work.limits.layout_work = 20;
    let mut depth = options();
    depth.limits.nesting_depth = 0;
    let mut output = options();
    output.limits.svg_bytes = 64;
    let cancelled = options();
    cancelled.control.cancel();
    // when / then
    for configured in [source_limit, models, work, depth, output, cancelled] {
        assert!(render(source, &configured).is_err());
    }
    let mut shallow = options();
    shallow.limits.nesting_depth = 1;
    assert!(render(source, &shallow).is_ok());
}

#[test]
fn should_admit_flat_git_history_at_depth_one_and_exact_source_quota() {
    // given
    let mut body = String::new();
    for index in 0..80 {
        body.push_str(&format!("* c{index}\n"));
        if index < 79 {
            body.push_str("|\n");
        }
    }
    let source = format!("@startgit\n{body}@endgit\n");
    let mut configured = options();
    configured.limits.nesting_depth = 1;
    configured.limits.source_bytes = source.len();
    // when
    let result = render(&source, &configured).unwrap();
    let xml = roxmltree::Document::parse(&result.svg).unwrap();
    // then
    assert_eq!(
        xml.descendants()
            .filter(|node| node.attribute("data-kind") == Some("git-commit"))
            .count(),
        80
    );
    assert_eq!(
        xml.descendants()
            .filter(|node| node.attribute("data-kind") == Some("git-parent"))
            .count(),
        79
    );
    configured.limits.source_bytes -= 1;
    assert!(matches!(
        render(&source, &configured),
        Err(diagram_engine::RenderError::SourceLimit { .. })
    ));
}

#[test]
fn should_keep_native_raw_grid_ancestry_when_original_sentinel_spaces_are_omitted() {
    // given
    let original_verified = include_str!("../../fixtures/plantuml/git.puml");
    let raw = original_verified
        .lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n");
    // when
    let verified_svg = checked(original_verified);
    let robust_svg = checked(&raw);
    // then
    assert_eq!(verified_svg, robust_svg);
}
