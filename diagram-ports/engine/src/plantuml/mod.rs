//! Native subset translated from the authorized PlantUML MIT 1.2026.9 source artifact.
//! Copyright 2009-2024 Arnaud Roques; credited link/declaration contribution: Hisashi Miyashita.
//! Credited state/transition contribution: Serge Wenger.
//! MIT grant retained in provenance/LICENSE-PlantUML-MIT.txt.
//! Exact behavioral source traces and copyright notices: provenance/plantuml-mit-1.2026.9.json.
mod activity;
mod activitytile;
mod arrow;
mod board;
mod boardscene;
mod budget;
mod chen;
mod chenscene;
mod chrome;
mod creole;
mod data;
mod datascene;
mod files;
mod filesscene;
mod gantt;
mod ganttscene;
mod graph;
mod hcl;
mod lexer;
mod nwdiag;
mod nwscene;
mod origin;
mod packet;
mod packetscene;
mod parser;
mod preprocess;
mod salt;
mod saltscene;
mod scene;
mod sequence;
mod shapes;
mod source;
mod state;
mod stategraph;
mod style;
mod svg;
mod text;
mod timing;
mod timingscene;
mod tree;
mod treescene;
mod uml;
mod yaml;

use crate::{Language, RenderError, RenderOptions, RenderedDiagram, limits, resources};
use merman_render::environment::RenderEnvironment;
use parser::header::Selection;

pub(crate) fn render(
    source: &str,
    options: &RenderOptions,
) -> Result<RenderedDiagram, RenderError> {
    limits::validate_source(source, &options.limits)?;
    let policy = limits::resource_policy(&options.limits)?;
    let mut budget = budget::Budget::new(options);
    budget.checkpoint()?;
    let expanded = preprocess::prepare(source, &mut budget)?;
    let mut prepared = parser::document::parse(&expanded, &mut budget)?;
    let wrapper = prepared.wrapper;
    let (preamble, body) = parser::preamble::parse(&mut prepared, &mut budget)?;
    let first_body = expanded.text[..body.start]
        .bytes()
        .filter(|&byte| byte == b'\n')
        .count()
        + 1;
    let start = prepared
        .lines
        .partition_point(|line| line.number < first_body);
    let lines = &prepared.lines[start..];
    let selection = if wrapper == source::Wrapper::Uml {
        parser::header::select(&prepared.body, lines, &mut budget)?
    } else {
        Selection::Sequence
    };
    let sequence_model = if wrapper == source::Wrapper::Uml && selection == Selection::Sequence {
        Some(parser::sequence::parse(prepared.body, &mut budget)?)
    } else {
        None
    };
    let result = (|| {
        let session = RenderEnvironment::deterministic()
            .without_math_renderer()
            .with_resource_policy(policy)
            .with_text_measurement_policy(options.text_measurement.clone())
            .begin_session_with_control(budget.control.clone())
            .map_err(|e| RenderError::Backend(e.to_string()))?;
        let mut context = text::Context::new(&session, &mut budget)
            .with_fonts(preamble.fonts.clone())
            .with_origins(&expanded);
        let (family, scene) = if wrapper == source::Wrapper::Chen {
            let model = chen::parse(lines, context.budget)?;
            ("chen", chenscene::layout(&model, &mut context)?)
        } else if wrapper == source::Wrapper::Files {
            let model = files::parse(lines, context.budget)?;
            ("files", filesscene::layout(&model, &mut context)?)
        } else if wrapper == source::Wrapper::Packet {
            let model = packet::parse(lines, context.budget)?;
            ("packetdiag", packetscene::layout(&model, &mut context)?)
        } else if wrapper == source::Wrapper::Board {
            let model = board::parse(lines, context.budget)?;
            ("board", boardscene::layout(&model, &mut context)?)
        } else if wrapper == source::Wrapper::Hcl {
            let document = hcl::parse(lines, context.budget)?;
            let model = hcl::lower(document, context.budget)?;
            ("hcl", datascene::layout(&model, &mut context)?)
        } else if wrapper == source::Wrapper::Nwdiag {
            let model = nwdiag::parse(lines, context.budget)?;
            ("nwdiag", nwscene::layout(&model, &mut context)?)
        } else if wrapper == source::Wrapper::Salt {
            let model = salt::parse(lines, context.budget)?;
            ("salt", saltscene::layout(&model, &mut context)?)
        } else if wrapper == source::Wrapper::Gantt {
            let model = gantt::parse(lines, context.budget)?;
            ("gantt", ganttscene::layout(&model, &mut context)?)
        } else if wrapper == source::Wrapper::Mindmap {
            let model = tree::parse_mindmap(lines, context.budget)?;
            ("mindmap", treescene::mindmap(&model, &mut context)?)
        } else if wrapper == source::Wrapper::Wbs {
            let model = tree::parse_wbs(lines, context.budget)?;
            ("wbs", treescene::wbs(&model, &mut context)?)
        } else if matches!(wrapper, source::Wrapper::Json | source::Wrapper::Yaml) {
            let (family, model) = if wrapper == source::Wrapper::Json {
                ("json", data::parse_json(lines, context.budget)?)
            } else {
                ("yaml", yaml::parse(lines, context.budget)?)
            };
            (family, datascene::layout(&model, &mut context)?)
        } else if selection == Selection::Activity {
            let model = activity::parse(lines, context.budget)?;
            ("activity", activitytile::layout(&model, &mut context)?)
        } else if selection == Selection::State {
            let model = state::parse(lines, context.budget)?;
            ("state", stategraph::layout(&model, &mut context)?)
        } else if selection == Selection::Uml {
            let model = uml::parse(lines, context.budget)?;
            let family = model.family;
            (family, graph::layout(&model, &mut context)?)
        } else if selection == Selection::Timing {
            let model = timing::parse(lines, context.budget)?;
            ("timing", timingscene::layout(&model, &mut context)?)
        } else {
            let model = sequence_model.as_ref().ok_or_else(|| {
                RenderError::Backend("PlantUML sequence selection has no parsed model".into())
            })?;
            ("sequence", sequence::layout(model, &mut context)?)
        };
        let scene = chrome::compose(scene, &preamble, &mut context)?;
        let raw = svg::serialize(&scene, &mut budget)?;
        let svg = merman::svg::SvgPipeline::resvg_safe()
            .with_postprocessor(resources::ResourceAdmission::new(budget.control.clone()))
            .process_resvg_compatible(&raw, &session)
            .map_err(|e| RenderError::UnsafeSvg(e.to_string()))?
            .into_string();
        budget.checkpoint()?;
        crate::svg::validate(&svg, &options.limits)?;
        Ok(RenderedDiagram{svg,language:Language::PlantUml,backend:"native PlantUML",upstream:"PlantUML MIT 1.2026.9",family:family.into(),diagnostics:vec!["Native subset: sequence, UML graphs including icon-free ArchiMate description rectangles, Chen entity/relationship/attribute graphs, scoped states, structured activities, integer timing, dated Gantt, Salt tables/controls, sparse nwdiag buses/shared servers, mindmap, WBS, ordered files/notes, inclusive packetdiag, staged boards and ordered JSON/narrow YAML/HCL; unsupported syntax fails explicitly. Native/Dagre geometry differs from PlantUML; text baselines use the default descent approximation, not native glyph bounds.".into()],measurement:session.text_measurement_report(),layout_work_units:budget.work})
    })();
    result.map_err(|error| expanded.map_error(error, &mut budget))
}
