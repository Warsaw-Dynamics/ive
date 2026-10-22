//! Adapter for Merman 0.9.2 (Mermaid 12.1.0); no upstream implementation is copied here.

use crate::{Language, RenderError, RenderOptions, RenderedDiagram, limits, svg};
use merman::svg::SvgPipeline;
use merman::{
    Engine, MermaidConfig, ParseOptions, RenderOutput, RenderTarget, Renderer, SvgEnvironment,
    SvgRequest,
};
use serde_json::{Value, json};

pub(crate) fn render(
    source: &str,
    options: &RenderOptions,
) -> Result<RenderedDiagram, RenderError> {
    limits::validate_source(source, &options.limits)?;
    let policy = limits::resource_policy(&options.limits)?;
    let control = options
        .control
        .child()
        .with_deadline(options.limits.timeout);
    control
        .checkpoint()
        .map_err(|error| RenderError::Cancelled(error.to_string()))?;
    reject_resources(source)?;
    let renderer = create_renderer().with_resource_policy(*policy.input_policy());
    let semantic = renderer
        .prepare_semantic(source, control.clone())
        .map_err(map_error)?
        .ok_or_else(|| RenderError::InvalidSource("no Mermaid diagram was detected".into()))?;
    validate_layout(semantic.metadata().config.as_value())?;
    let family = semantic.diagram_type().to_owned();
    if family == "flowchart-elk" {
        return Err(RenderError::UnsupportedLayout("flowchart-elk".into()));
    }
    let environment = SvgEnvironment::deterministic()
        .without_math_renderer()
        .with_resource_policy(policy)
        .with_text_measurement_policy(options.text_measurement.clone());
    let pipeline = SvgPipeline::resvg_safe().with_postprocessor(svg::NormalizeOutput::new(control));
    let request = SvgRequest {
        environment,
        pipeline: Some(pipeline),
        ..Default::default()
    };
    let output = semantic
        .render(RenderTarget::Svg(request))
        .map_err(map_error)?;
    finish(output, family, options)
}

fn create_renderer() -> Renderer {
    let config = MermaidConfig::from_value(json!({
        "layout": "dagre", "look": "classic", "theme": "default", "securityLevel": "strict",
        "htmlLabels": false, "flowchart": { "htmlLabels": false, "defaultRenderer": "dagre-wrapper" },
        "deterministicIds": true, "deterministicIDSeed": "diagram-ports",
        "startOnLoad": false, "suppressErrorRendering": true,
    }));
    Renderer::new()
        .with_engine(Engine::new().with_site_config(config))
        .with_runtime_policy(merman::runtime::RuntimePolicy::deterministic())
        .with_parse_options(ParseOptions::strict())
}

fn reject_resources(source: &str) -> Result<(), RenderError> {
    let lower = source.to_ascii_lowercase();
    for token in [
        "://",
        "file:",
        "data:",
        "javascript:",
        "<img",
        "<image",
        "@import",
        "href=",
        "src=",
    ] {
        if lower.contains(token) {
            return Err(RenderError::ExternalResource(token.into()));
        }
    }
    let compact: String = lower.split_whitespace().collect();
    if compact.contains("img:") {
        return Err(RenderError::ExternalResource(
            "image metadata (img:)".into(),
        ));
    }
    Ok(())
}

fn validate_layout(value: &Value) -> Result<(), RenderError> {
    match value {
        Value::Object(properties) => {
            for (key, value) in properties {
                if matches!(key.as_str(), "layout" | "defaultRenderer") {
                    let layout = value.as_str().unwrap_or("<non-string>");
                    if matches!(
                        layout,
                        "dagre" | "dagre-d3" | "dagre-wrapper" | "cose-bilkent" | "swimlane"
                    ) {
                        continue;
                    }
                    return Err(RenderError::UnsupportedLayout(layout.into()));
                }
                validate_layout(value)?;
            }
        }
        Value::Array(items) => {
            for item in items {
                validate_layout(item)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn finish(
    output: RenderOutput,
    family: String,
    options: &RenderOptions,
) -> Result<RenderedDiagram, RenderError> {
    let RenderOutput::Svg(Some(output)) = output else {
        return Err(RenderError::Backend("Mermaid produced no SVG".into()));
    };
    svg::validate(output.svg(), &options.limits)?;
    let (svg, evidence) = output.into_parts();
    Ok(RenderedDiagram {
        svg, language: Language::Mermaid, backend: "merman 0.9.2", upstream: "Mermaid 12.1.0", family,
        diagnostics: vec!["ELK and math are unavailable; graph layout defaults to Dagre. Font measurement provenance is reported separately.".into()],
        measurement: evidence.measurement().clone(), layout_work_units: evidence.layout_work_units(),
    })
}

fn map_error(error: merman::RenderError) -> RenderError {
    match error {
        merman::RenderError::Parse(error) => RenderError::InvalidSource(error.to_string()),
        merman::RenderError::NoDiagram => {
            RenderError::InvalidSource("no Mermaid diagram was detected".into())
        }
        merman::RenderError::Cancelled(error) => RenderError::Cancelled(error.to_string()),
        error => RenderError::Backend(error.to_string()),
    }
}
