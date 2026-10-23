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
    let view_bytes = std::mem::size_of::<parser::BodySource<'_>>();
    let adapters = prepared.body.adapters.saturating_add(view_bytes);
    budget::bound(
        "family parser view retained bytes",
        adapters,
        budget.limits.source_bytes.saturating_mul(256),
    )?;
    budget.charge(view_bytes)?;
    prepared.body.adapters = adapters;
    let sequence_model = if wrapper == source::Wrapper::Uml && selection == Selection::Sequence {
        Some(parser::sequence::parse(
            prepared.body.reborrow(),
            &mut budget,
        )?)
    } else {
        None
    };
    let state_model = if wrapper == source::Wrapper::Uml && selection == Selection::State {
        Some(parser::state::parse(prepared.body.reborrow(), &mut budget)?)
    } else {
        None
    };
    let files_model = if wrapper == source::Wrapper::Files {
        Some(parser::files::parse(prepared.body.reborrow(), &mut budget)?)
    } else {
        None
    };
    let board_model = if wrapper == source::Wrapper::Board {
        Some(parser::board::parse(prepared.body.reborrow(), &mut budget)?)
    } else {
        None
    };
    let mindmap_model = if wrapper == source::Wrapper::Mindmap {
        Some(parser::tree::parse_mindmap(
            prepared.body.reborrow(),
            &mut budget,
        )?)
    } else {
        None
    };
    let wbs_model = if wrapper == source::Wrapper::Wbs {
        Some(parser::tree::parse_wbs(
            prepared.body.reborrow(),
            &mut budget,
        )?)
    } else {
        None
    };
    let uml_model = if wrapper == source::Wrapper::Uml && selection == Selection::Uml {
        Some(parser::uml::parse(prepared.body.reborrow(), &mut budget)?)
    } else {
        None
    };
    let activity_model = if wrapper == source::Wrapper::Uml && selection == Selection::Activity {
        Some(parser::activity::parse(
            prepared.body.reborrow(),
            &mut budget,
        )?)
    } else {
        None
    };
    let chen_model = if wrapper == source::Wrapper::Chen {
        Some(parser::chen::parse(prepared.body.reborrow(), &mut budget)?)
    } else {
        None
    };

    let json_model = if wrapper == source::Wrapper::Json {
        Some(parser::json::parse(prepared.body.reborrow(), &mut budget)?)
    } else {
        None
    };
    let hcl_document = if wrapper == source::Wrapper::Hcl {
        Some(parser::hcl::parse(prepared.body.reborrow(), &mut budget)?)
    } else {
        None
    };
    let hcl_model = hcl_document
        .map(|document| hcl::lower(document, &mut budget))
        .transpose()?;

    let yaml_model = if wrapper == source::Wrapper::Yaml {
        Some(parser::yaml::parse(prepared.body.reborrow(), &mut budget)?)
    } else {
        None
    };
    let timing_model = if wrapper == source::Wrapper::Uml && selection == Selection::Timing {
        Some(parser::timing::parse(
            prepared.body.reborrow(),
            &mut budget,
        )?)
    } else {
        None
    };
    let gantt_model = if wrapper == source::Wrapper::Gantt {
        Some(parser::gantt::parse(prepared.body.reborrow(), &mut budget)?)
    } else {
        None
    };
    let salt_model = if wrapper == source::Wrapper::Salt {
        Some(parser::salt::parse(prepared.body.reborrow(), &mut budget)?)
    } else {
        None
    };
    let nwdiag_model = if wrapper == source::Wrapper::Nwdiag {
        Some(parser::nwdiag::parse(
            prepared.body.reborrow(),
            &mut budget,
        )?)
    } else {
        None
    };
    let packet_model = if wrapper == source::Wrapper::Packet {
        Some(parser::packet::parse(
            prepared.body.reborrow(),
            &mut budget,
        )?)
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
            let model = chen_model.as_ref().ok_or_else(|| {
                RenderError::Backend("PlantUML Chen selection has no parsed model".into())
            })?;
            ("chen", chenscene::layout(model, &mut context)?)
        } else if wrapper == source::Wrapper::Files {
            let model = files_model.as_ref().ok_or_else(|| {
                RenderError::Backend("PlantUML files selection has no parsed model".into())
            })?;
            ("files", filesscene::layout(model, &mut context)?)
        } else if wrapper == source::Wrapper::Packet {
            let model = packet_model.as_ref().ok_or_else(|| {
                RenderError::Backend("PlantUML packetdiag selection has no parsed model".into())
            })?;
            ("packetdiag", packetscene::layout(model, &mut context)?)
        } else if wrapper == source::Wrapper::Board {
            let model = board_model.as_ref().ok_or_else(|| {
                RenderError::Backend("PlantUML board selection has no parsed model".into())
            })?;
            ("board", boardscene::layout(model, &mut context)?)
        } else if wrapper == source::Wrapper::Hcl {
            let model = hcl_model.as_ref().ok_or_else(|| {
                RenderError::Backend("PlantUML HCL selection has no parsed model".into())
            })?;
            ("hcl", datascene::layout(model, &mut context)?)
        } else if wrapper == source::Wrapper::Nwdiag {
            let model = nwdiag_model.as_ref().ok_or_else(|| {
                RenderError::Backend("PlantUML nwdiag selection has no parsed model".into())
            })?;
            ("nwdiag", nwscene::layout(model, &mut context)?)
        } else if wrapper == source::Wrapper::Salt {
            let model = salt_model.as_ref().ok_or_else(|| {
                RenderError::Backend("PlantUML Salt selection has no parsed model".into())
            })?;
            ("salt", saltscene::layout(model, &mut context)?)
        } else if wrapper == source::Wrapper::Gantt {
            let model = gantt_model.as_ref().ok_or_else(|| {
                RenderError::Backend("Gantt selection has no parsed model".into())
            })?;
            ("gantt", ganttscene::layout(model, &mut context)?)
        } else if wrapper == source::Wrapper::Mindmap {
            let model = mindmap_model.as_ref().ok_or_else(|| {
                RenderError::Backend("PlantUML mindmap selection has no parsed model".into())
            })?;
            ("mindmap", treescene::mindmap(model, &mut context)?)
        } else if wrapper == source::Wrapper::Wbs {
            let model = wbs_model.as_ref().ok_or_else(|| {
                RenderError::Backend("PlantUML WBS selection has no parsed model".into())
            })?;
            ("wbs", treescene::wbs(model, &mut context)?)
        } else if wrapper == source::Wrapper::Json {
            let model = json_model.as_ref().ok_or_else(|| {
                RenderError::Backend("PlantUML JSON selection has no parsed model".into())
            })?;
            ("json", datascene::layout(model, &mut context)?)
        } else if wrapper == source::Wrapper::Yaml {
            let model = yaml_model.as_ref().ok_or_else(|| {
                RenderError::Backend("PlantUML YAML selection has no parsed model".into())
            })?;
            ("yaml", datascene::layout(model, &mut context)?)
        } else if selection == Selection::Activity {
            let model = activity_model.as_ref().ok_or_else(|| {
                RenderError::Backend("PlantUML activity selection has no parsed model".into())
            })?;
            ("activity", activitytile::layout(model, &mut context)?)
        } else if selection == Selection::State {
            let model = state_model.as_ref().ok_or_else(|| {
                RenderError::Backend("PlantUML state selection has no parsed model".into())
            })?;
            ("state", stategraph::layout(model, &mut context)?)
        } else if selection == Selection::Uml {
            let model = uml_model.as_ref().ok_or_else(|| {
                RenderError::Backend("PlantUML UML selection has no parsed model".into())
            })?;
            let family = model.family;
            (family, graph::layout(model, &mut context)?)
        } else if selection == Selection::Timing {
            let model = timing_model.as_ref().ok_or_else(|| {
                RenderError::Backend("timing selection has no parsed model".into())
            })?;
            ("timing", timingscene::layout(model, &mut context)?)
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
