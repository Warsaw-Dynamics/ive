//! Native subset translated from the authorized PlantUML MIT 1.2026.9 source artifact.
//! Copyright 2009-2024 Arnaud Roques; credited link/declaration contribution: Hisashi Miyashita.
//! Credited state/transition contribution: Serge Wenger.
//! MIT grant retained in provenance/LICENSE-PlantUML-MIT.txt.
//! Exact behavioral source traces and copyright notices: provenance/plantuml-mit-1.2026.9.json.
mod activity;
mod activitytile;
mod arrow;
mod budget;
mod chrome;
mod creole;
mod data;
mod datascene;
mod graph;
mod lexer;
mod origin;
mod preprocess;
mod scene;
mod sequence;
mod shapes;
mod source;
mod state;
mod stategraph;
mod style;
mod svg;
mod text;
mod tree;
mod treescene;
mod uml;
mod yaml;

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
    let expanded = preprocess::prepare(source, &mut budget)?;
    let result = (|| {
        let (wrapper, lines) = source::prepare_document(&expanded.text, &mut budget)?;
        let preamble = style::prepare(&lines, wrapper, &mut budget)?;
        expanded.check_preamble(&lines, preamble.start, &mut budget)?;
        let lines = &lines[preamble.start..];
        let session = RenderEnvironment::deterministic()
            .without_math_renderer()
            .with_resource_policy(policy)
            .with_text_measurement_policy(options.text_measurement.clone())
            .begin_session_with_control(budget.control.clone())
            .map_err(|e| RenderError::Backend(e.to_string()))?;
        let selection = select(lines);
        let mut context = text::Context::new(&session, &mut budget)
            .with_fonts(preamble.fonts.clone())
            .with_origins(&expanded);
        let (family, scene) = if wrapper == source::Wrapper::Mindmap {
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
        } else {
            let model = sequence::parse(lines, context.budget)?;
            ("sequence", sequence::layout(&model, &mut context)?)
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
        Ok(RenderedDiagram{svg,language:Language::PlantUml,backend:"native PlantUML",upstream:"PlantUML MIT 1.2026.9",family:family.into(),diagnostics:vec!["Native subset: sequence, UML graphs, scoped states, structured activities, mindmap, WBS and ordered JSON/narrow YAML; unsupported syntax fails explicitly. Native/Dagre geometry differs from PlantUML; text baselines use the default descent approximation, not native glyph bounds.".into()],measurement:session.text_measurement_report(),layout_work_units:budget.work})
    })();
    result.map_err(|error| expanded.map_error(error, &mut budget))
}

#[derive(PartialEq, Eq)]
enum Selection {
    Sequence,
    Uml,
    State,
    Activity,
}

fn select(lines: &[source::Line]) -> Selection {
    let mut candidate = Selection::Sequence;
    for line in lines {
        if has_identifier_arrow(line) {
            continue;
        }
        let word = line.text.split_whitespace().next().unwrap_or("");
        if matches!(
            word,
            "class"
                | "abstract"
                | "object"
                | "interface"
                | "enum"
                | "annotation"
                | "component"
                | "node"
                | "rectangle"
                | "package"
                | "namespace"
                | "usecase"
        ) {
            return Selection::Uml;
        }
        if word == "state" {
            return Selection::State;
        }
        if line.text.starts_with(':')
            || matches!(
                word.trim_end_matches(';'),
                "start" | "stop" | "repeat" | "fork" | "partition"
            )
            || line.text.starts_with("if(")
            || line.text.starts_with("if (")
            || line.text.starts_with("while(")
            || line.text.starts_with("while (")
        {
            return Selection::Activity;
        }
        if word == "note" {
            return if uml::is_graph(std::slice::from_ref(line)) {
                Selection::Uml
            } else {
                Selection::Sequence
            };
        }
        if matches!(
            word,
            "participant"
                | "activate"
                | "deactivate"
                | "alt"
                | "loop"
                | "opt"
                | "create"
                | "destroy"
        ) {
            return Selection::Sequence;
        }
        let (head, _) = lexer::split_label(line);
        if head.starts_with("[*]") || head.ends_with("[*]") {
            return Selection::State;
        }
        if head != line.text
            && !head.contains(['-', '<', '>'])
            && !matches!(word, "actor" | "database")
        {
            candidate = Selection::State;
        }
    }
    if candidate == Selection::State {
        return candidate;
    }
    if uml::is_graph(lines) {
        Selection::Uml
    } else {
        Selection::Sequence
    }
}

fn has_identifier_arrow(line: &source::Line) -> bool {
    let text = line.text.trim_start();
    let word = text.split_whitespace().next().unwrap_or("");
    !word.starts_with('[')
        && !lexer::split_label(line).0.ends_with("[*]")
        && lexer::arrow_at(text[word.len()..].trim_start()).is_some()
}
