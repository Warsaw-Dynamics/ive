//! Developer-only in-process check of the public DOT layout-work limit.

use diagram_engine::{GraphLayout, Language, RenderError, RenderOptions, render_dot_json};

fn main() {
    let source = include_str!("fixtures/chain.dot");
    let mut options = RenderOptions {
        language: Language::Dot,
        graph_layout: GraphLayout::Layered,
        ..RenderOptions::default()
    };
    if let Err(error) = render_dot_json(source, &options) {
        eprintln!("valid DOT control case failed: {error}");
        std::process::exit(1);
    }
    options.limits.layout_work = 1;
    match render_dot_json(source, &options) {
        Err(RenderError::Backend(message)) if message.contains("DOT work count") => {
            println!("{message}");
        }
        other => {
            eprintln!("DOT layout_work=1 did not stop work: {other:?}");
            std::process::exit(1);
        }
    }
}
