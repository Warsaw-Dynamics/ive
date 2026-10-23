use super::*;

#[cfg(target_os = "linux")]
#[test]
fn should_keep_native_creole_style_transitions_separated_in_actual_painted_glyphs() {
    use crate::{
        native_text::{NativeText, NativeTextResources},
        worker::{MeasureQuery, MeasureReply, Measurer},
    };
    use diagram_engine::{
        HostMeasurementResult, HostTextMeasurement, HostTextMeasurementRequest, HostTextMeasurer,
        Language, MeasurementProfileId, RenderOptions, TextMeasurementPhase, TextMeasurementPolicy,
        TextMeasurementProfileIdentity,
    };
    struct Bridge(Arc<NativeText>);
    impl HostTextMeasurer for Bridge {
        fn measure(&self, request: HostTextMeasurementRequest<'_>) -> HostMeasurementResult {
            let reply = self.0.measure(&MeasureQuery {
                operation: request.operation.external_code(),
                text: request.text.into(),
                style: request.style.clone(),
            });
            eprintln!(
                "measured {:?} {:?}: {:?}",
                request.text, request.style, reply
            );
            Ok(match reply {
                MeasureReply::Metrics(metrics) => Some(HostTextMeasurement::Metrics(metrics)),
                MeasureReply::Length(width) => Some(HostTextMeasurement::Length(width)),
                MeasureReply::Fallback => None,
            })
        }
    }
    // given
    let platform = gpui_platform::current_platform(true);
    let resources =
        NativeTextResources::new(Arc::new(gpui::TextSystem::new(platform.text_system())));
    let native = resources.start_render();
    let config = RenderOptions {
        language: Language::PlantUml,
        text_measurement: TextMeasurementPolicy::host_display(
            TextMeasurementProfileIdentity::new(
                MeasurementProfileId::new("native-creole-spacing").unwrap(),
                "1",
            )
            .unwrap(),
            Arc::new(Bridge(native.clone())),
            [TextMeasurementPhase::Layout],
        ),
        ..Default::default()
    };
    let source = crate::examples::source("plantuml-creole").unwrap().replace(
        "@endcreole",
        "**heavy //both// restored** plain\n@endcreole",
    );
    // when
    let output = diagram_engine::render(&source, &config).unwrap();
    let tree = usvg::Tree::from_str(&output.svg, &crate::svg_fonts::options()).unwrap();
    // then
    eprintln!("resolved {:?}", native.report());
    let report = native.report();
    let family = report[0].rsplit_once(" → ").unwrap().1;
    let mut runs = Vec::new();
    let mut pending = vec![tree.root()];
    while let Some(group) = pending.pop() {
        for node in group.children() {
            match node {
                usvg::Node::Group(group) => pending.push(group),
                usvg::Node::Text(text) => {
                    let value = text.chunks().iter().map(|c| c.text()).collect::<String>();
                    if value.trim().is_empty() {
                        continue;
                    }
                    let ink = text.abs_stroke_bounding_box();
                    let fonts: std::collections::BTreeSet<_> = text
                        .layouted()
                        .iter()
                        .flat_map(|s| &s.positioned_glyphs)
                        .filter_map(|g| tree.fontdb().face(g.font))
                        .map(|face| {
                            format!(
                                "{:?} weight={:?} style={:?}",
                                face.families, face.weight, face.style
                            )
                        })
                        .collect();
                    eprintln!("painted {value:?} {ink:?} {fonts:?}");
                    for face in text
                        .layouted()
                        .iter()
                        .flat_map(|span| &span.positioned_glyphs)
                        .filter_map(|glyph| tree.fontdb().face(glyph.font))
                    {
                        assert!(
                            face.families.iter().any(|(name, _)| name == family),
                            "native family {family} differs from actual painted {fonts:?}"
                        );
                        if matches!(value.as_str(), "Invoice" | "heavy " | "both" | " restored") {
                            assert_eq!(face.weight.0, 700, "actual bold face lost for {value:?}");
                        }
                        if matches!(value.as_str(), "ready" | "both") {
                            assert_eq!(
                                face.style,
                                usvg::fontdb::Style::Italic,
                                "actual italic face lost for {value:?}"
                            );
                        }
                        if value == " plain" {
                            assert_eq!(face.weight.0, 400);
                            assert_eq!(face.style, usvg::fontdb::Style::Normal);
                        }
                    }
                    runs.push((value, ink));
                }
                _ => {}
            }
        }
    }
    for (left, right) in [
        ("Invoice", "ready"),
        ("Paid", "old"),
        ("heavy ", "both"),
        ("both", " restored"),
        (" restored", " plain"),
    ] {
        let a = runs.iter().find(|(text, _)| text == left).unwrap().1;
        let b = runs.iter().find(|(text, _)| text == right).unwrap().1;
        assert!(
            a.right() + 1. <= b.left(),
            "{left} ink right={} overlaps {right} ink left={} despite authored space",
            a.right(),
            b.left()
        );
    }
}
