use diagram_viewer::{
    cli::{self, Comparison, Mode},
    worker,
};
use std::path::Path;
fn main() {
    if let Err(error) = run() {
        eprintln!("diagram-viewer: {error}");
        std::process::exit(1);
    }
}
fn run() -> Result<(), String> {
    let cli = cli::parse(std::env::args_os().skip(1))?;
    match cli.mode {
        Mode::Worker => worker::main(),
        Mode::Help => {
            println!("{}", cli::HELP);
            Ok(())
        }
        Mode::Export {
            source,
            output,
            comparison,
        } => {
            if cli.language == "dot" {
                return export_dot(&source, &output, comparison);
            }
            let source_text = worker::read_source(&source)?;
            let mut job = worker::Job::headless(source_text);
            job.language = cli.language;
            let rendered = worker::run(
                &std::env::current_exe()
                    .map_err(|error| format!("cannot locate diagram-viewer executable: {error}"))?,
                job,
            )?;
            diagram_viewer::export::write(&rendered.svg, &output, comparison)
        }
        Mode::View {
            source,
            background,
            capture,
        } => diagram_viewer::app::run(
            source,
            diagram_viewer::app::Settings {
                language: cli.language,
                background,
                capture,
            },
        ),
    }
}

fn export_dot(source: &Path, output: &Path, comparison: Comparison) -> Result<(), String> {
    let source_text = worker::read_source(source)?;
    let options = diagram_engine::RenderOptions {
        language: diagram_engine::Language::Dot,
        graph_layout: diagram_engine::GraphLayout::Layered,
        ..Default::default()
    };
    let format = output.extension().and_then(|extension| extension.to_str());
    if format.is_some_and(|extension| extension.eq_ignore_ascii_case("json")) {
        let geometry = diagram_engine::render_dot_json(&source_text, &options)
            .map_err(|error| error.to_string())?;
        return std::fs::write(output, geometry)
            .map_err(|error| format!("cannot write DOT geometry {}: {error}", output.display()));
    }
    if format.is_some_and(|extension| extension.eq_ignore_ascii_case("png"))
        && let Some(png) = diagram_engine::render_dot_png_native(&source_text, &options)
            .map_err(|error| error.to_string())?
    {
        return diagram_viewer::export::write_native_dot_png(&png, output, comparison);
    }
    let rendered =
        diagram_engine::render(&source_text, &options).map_err(|error| error.to_string())?;
    diagram_viewer::export::write_dot(&rendered.svg, output, comparison)
}

#[cfg(test)]
mod tests {
    use super::export_dot;
    use diagram_viewer::cli::Comparison;

    #[test]
    fn should_export_dot_svg_png_and_json_directly_from_engine() {
        // given
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("chain.dot");
        let svg = directory.path().join("chain.svg");
        let png = directory.path().join("chain.png");
        let json = directory.path().join("chain.json");
        std::fs::write(&source, "digraph G { a -> b }").unwrap();
        // when
        export_dot(&source, &svg, Comparison::Disabled).unwrap();
        export_dot(&source, &png, Comparison::Disabled).unwrap();
        export_dot(&source, &json, Comparison::Disabled).unwrap();
        // then
        assert!(std::fs::read_to_string(svg).unwrap().contains("<svg"));
        assert!(
            std::fs::read(png)
                .unwrap()
                .starts_with(b"\x89PNG\r\n\x1a\n")
        );
        let geometry: serde_json::Value =
            serde_json::from_slice(&std::fs::read(json).unwrap()).unwrap();
        assert_eq!(geometry["objects"].as_array().unwrap().len(), 2);
    }
}
