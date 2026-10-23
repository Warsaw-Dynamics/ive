//! Development-only immutable parser-migration baseline exporter.
use diagram_engine::{Language, RenderOptions, render};
use std::{fs, io::Write, path::Path};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = RenderOptions {
        language: Language::PlantUml,
        ..RenderOptions::default()
    };
    let mut output = std::io::stdout().lock();
    for path in std::env::args().skip(1) {
        if path.len() > 4096 || fs::metadata(&path)?.len() > options.limits.source_bytes as u64 {
            return Err(format!("baseline input exceeds path/source bounds: {path:?}").into());
        }
        let source = fs::read_to_string(Path::new(&path))?;
        let rendered = render(&source, &options)?;
        serde_json::to_writer(
            &mut output,
            &serde_json::json!({"path": path, "family": rendered.family, "svg": rendered.svg}),
        )?;
        output.write_all(b"\n")?;
    }
    Ok(())
}
