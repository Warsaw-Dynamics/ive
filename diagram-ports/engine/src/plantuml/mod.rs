//! Native subset translated from the authorized PlantUML MIT 1.2026.9 source artifact.
//! Copyright 2009-2024 Arnaud Roques; credited link/declaration contribution: Hisashi Miyashita.
//! MIT grant retained in provenance/LICENSE-PlantUML-MIT.txt.
//! Exact behavioral source traces and copyright notices: provenance/plantuml-mit-1.2026.9.json.
mod arrow;
mod budget;
mod graph;
mod lexer;
mod scene;
mod sequence;
mod shapes;
mod source;
mod svg;
mod text;
mod uml;

use crate::{Language, RenderError, RenderOptions, RenderedDiagram, limits, resources};
use merman_render::environment::RenderEnvironment;

pub(crate) fn render(
    source: &str,
    options: &RenderOptions,
) -> Result<RenderedDiagram, RenderError> {
    limits::validate_source(source, &options.limits)?;
    let policy = limits::resource_policy(&options.limits)?;
    let mut budget = budget::Budget::new(options);
    budget.checkpoint()?;
    let lines = source::prepare(source, &mut budget)?;
    let session = RenderEnvironment::deterministic()
        .without_math_renderer()
        .with_resource_policy(policy)
        .with_text_measurement_policy(options.text_measurement.clone())
        .begin_session_with_control(budget.control.clone())
        .map_err(|e| RenderError::Backend(e.to_string()))?;
    let (family, scene) = if uml::is_graph(&lines) {
        let model = uml::parse(&lines, &mut budget)?;
        let family = model.family;
        (
            family,
            graph::layout(&model, &mut text::Context::new(&session, &mut budget))?,
        )
    } else {
        let model = sequence::parse(&lines, &mut budget)?;
        (
            "sequence",
            sequence::layout(&model, &mut text::Context::new(&session, &mut budget))?,
        )
    };
    let raw = svg::serialize(&scene, &mut budget)?;
    let svg = merman::svg::SvgPipeline::resvg_safe()
        .with_postprocessor(resources::ResourceAdmission::new(budget.control.clone()))
        .process_resvg_compatible(&raw, &session)
        .map_err(|e| RenderError::UnsafeSvg(e.to_string()))?
        .into_string();
    budget.checkpoint()?;
    crate::svg::validate(&svg, &options.limits)?;
    Ok(RenderedDiagram{svg,language:Language::PlantUml,backend:"native PlantUML",upstream:"PlantUML MIT 1.2026.9",family:family.into(),diagnostics:vec!["Native subset: sequence and UML graphs; unsupported syntax fails explicitly. Dagre geometry differs from Graphviz; text baselines use the PlantUML default descent approximation, not native glyph bounds.".into()],measurement:session.text_measurement_report(),layout_work_units:budget.work})
}
