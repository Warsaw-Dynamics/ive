use diagram_engine::{RenderOptions, render};

macro_rules! family_case {
    ($test:ident, $file:literal, $label:literal) => {
        #[test]
        fn $test() {
            // given
            let source = include_str!(concat!("../../fixtures/mermaid/", $file, ".mmd"));
            // when
            let diagram = render(source, &RenderOptions::default()).expect($file);
            // then
            verify_diagram(&diagram.svg, $label);
        }
    };
}

fn verify_diagram(svg: &str, expected_label: &str) {
    let document = roxmltree::Document::parse(svg).expect("well-formed SVG");
    let root = document.root_element();
    assert_eq!(root.tag_name().name(), "svg");
    let view_box: Vec<f64> = root
        .attribute("viewBox")
        .expect("viewBox")
        .split_whitespace()
        .map(|value| value.parse().expect("numeric viewBox"))
        .collect();
    assert_eq!(view_box.len(), 4);
    assert!(view_box.iter().all(|value| value.is_finite()));
    assert!(view_box[2] > 0.0 && view_box[3] > 0.0);
    let text = document
        .descendants()
        .filter(|node| node.is_text() && node.ancestors().any(|parent| parent.has_tag_name("text")))
        .filter_map(|node| node.text())
        .collect::<Vec<_>>()
        .join(" ");
    let normalized_text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(
        normalized_text
            .to_lowercase()
            .contains(&expected_label.to_lowercase()),
        "missing {expected_label:?} in {normalized_text}"
    );
    assert!(document.descendants().any(|node| node.has_tag_name("text")));
    let native = usvg::Tree::from_str(svg, &usvg::Options::default()).expect("native SVG parser");
    assert!(native.size().width() > 0.0 && native.size().height() > 0.0);
}

family_case!(should_render_flowchart, "flowchart", "Read changes");
family_case!(should_render_sequence, "sequence", "Hello John");
family_case!(should_render_class, "class", "beakColor");
family_case!(should_render_agentflow, "agentflow", "Gather changes");
family_case!(should_render_architecture, "architecture", "Database");
family_case!(should_render_block, "block", "A wide one");
family_case!(should_render_c4, "c4", "Banking Customer");
family_case!(should_render_cynefin, "cynefin", "Investigate root cause");
family_case!(should_render_er, "er", "CUSTOMER");
family_case!(should_render_eventmodeling, "eventmodeling", "CartUI");
family_case!(should_render_gantt, "gantt", "Another task");
family_case!(should_render_git, "git", "develop");
family_case!(should_render_info, "info", "v12.1.0");
family_case!(should_render_ishikawa, "ishikawa", "Blurry Photo");
family_case!(should_render_journey, "journey", "Make tea");
family_case!(should_render_kanban, "kanban", "Task Description");
family_case!(should_render_mindmap, "mindmap", "Origins");
family_case!(should_render_packet, "packet", "Source Port");
family_case!(should_render_pie, "pie", "Dogs");
family_case!(should_render_quadrant, "quadrant", "Campaign A");
family_case!(should_render_radar, "radar", "Science");
family_case!(should_render_railroad, "railroad", "digit");
family_case!(should_render_railroad_ebnf, "railroad-ebnf", "digit");
family_case!(should_render_railroad_abnf, "railroad-abnf", "digit");
family_case!(should_render_railroad_peg, "railroad-peg", "Digit");
family_case!(should_render_requirement, "requirement", "test_req");
family_case!(should_render_sankey, "sankey", "Biomass");
family_case!(should_render_state, "state", "Moving");
family_case!(should_render_swimlane, "swimlane", "Browse catalogue");
family_case!(should_render_timeline, "timeline", "LinkedIn");
family_case!(should_render_tree, "tree", "index.ts");
family_case!(should_render_treemap, "treemap", "Item A1");
family_case!(should_render_usecase, "usecase", "Place order");
family_case!(should_render_venn, "venn", "Frontend");
family_case!(should_render_wardley, "wardley", "Cup of Tea");
family_case!(should_render_xychart, "xychart", "Sales Revenue");
family_case!(should_render_zenuml, "zenuml", "Hello John");
