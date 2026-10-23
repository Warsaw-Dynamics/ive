//! Development-only immutable parser-migration baseline exporter.
use diagram_engine::{Language, RenderOptions, TextMeasurementPolicy, render};
use std::{fs, io::Write, path::Path};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = RenderOptions {
        language: Language::PlantUml,
        ..RenderOptions::default()
    };
    let mut output = std::io::stdout().lock();
    serde_json::to_writer(
        &mut output,
        &serde_json::json!({"configuration": configuration(&options)?}),
    )?;
    output.write_all(b"\n")?;
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

fn configuration(options: &RenderOptions) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    if options.language != Language::PlantUml
        || options.text_measurement.routes() != TextMeasurementPolicy::deterministic().routes()
    {
        return Err("baseline exporter requires PlantUml with deterministic measurement".into());
    }
    let limits = &options.limits;
    Ok(serde_json::json!({
        "language": "PlantUml", "measurement": "default deterministic",
        "sourceBytes": limits.source_bytes, "svgBytes": limits.svg_bytes,
        "modelItems": limits.model_items, "nestingDepth": limits.nesting_depth,
        "layoutWork": limits.layout_work, "timeoutMilliseconds": limits.timeout.as_millis(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_report_actual_defaults_and_detect_drift_against_the_immutable_baseline() {
        // given
        let options = RenderOptions {
            language: Language::PlantUml,
            ..RenderOptions::default()
        };
        let baseline: serde_json::Value =
            serde_json::from_str(include_str!("../../fixtures/plantuml/parser-baseline.json"))
                .unwrap();
        // when
        let reported = configuration(&options).unwrap();
        let changed = RenderOptions {
            limits: diagram_engine::RenderLimits {
                source_bytes: options.limits.source_bytes + 1,
                ..options.limits
            },
            ..options
        };
        // then
        assert_eq!(reported, baseline["configuration"]);
        assert_eq!(
            configuration(&changed).unwrap()["sourceBytes"],
            changed.limits.source_bytes
        );
        assert_ne!(configuration(&changed).unwrap(), baseline["configuration"]);
    }
}
