use super::*;

#[cfg(target_os = "linux")]
#[test]
fn should_contain_actual_native_tree_and_data_text_within_owning_boxes() {
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
            let query = MeasureQuery {
                operation: request.operation.external_code(),
                text: request.text.into(),
                style: request.style.clone(),
            };
            eprintln!("native query text={:?} style={:?}", query.text, query.style);
            Ok(match self.0.measure(&query) {
                MeasureReply::Metrics(m) => Some(HostTextMeasurement::Metrics(m)),
                MeasureReply::Length(w) => Some(HostTextMeasurement::Length(w)),
                MeasureReply::Fallback => None,
            })
        }
    }
    // given
    let platform = gpui_platform::current_platform(true);
    let fonts = Arc::new(gpui::TextSystem::new(platform.text_system()));
    let resources = NativeTextResources::new(fonts);
    for source in [
        "@startjson\n{\"short\":\"x\",\"labels\":[\"it's <literal> & Ω\",\"https://example.test/a:b # text\",\"長い日本語 Ω café 😀\"]}\n@endjson",
        "@startyaml\nnotes: |\n  &literal *literal !customtag\n  quoted: text\nnext: value\n@endyaml",
        "@startmindmap\n* Root\n** A very long ordinary tree label\n*** Unicode Ω café\n@endmindmap",
        "@startwbs\n* Root\n** A very long ordinary tree label\n*** Unicode Ω café\n@endwbs",
    ] {
        let native = resources.start_render();
        let mut config = RenderOptions {
            language: Language::PlantUml,
            ..Default::default()
        };
        config.text_measurement = TextMeasurementPolicy::host_display(
            TextMeasurementProfileIdentity::new(
                MeasurementProfileId::new("native-paint-contract").unwrap(),
                "1",
            )
            .unwrap(),
            Arc::new(Bridge(native.clone())),
            [TextMeasurementPhase::Layout],
        );
        // when
        let output = diagram_engine::render(source, &config).unwrap();
        let xml = roxmltree::Document::parse(&output.svg).unwrap();
        let options = crate::svg_fonts::options();
        let family = options
            .fontdb
            .query(&usvg::fontdb::Query {
                families: &[usvg::fontdb::Family::SansSerif],
                ..Default::default()
            })
            .and_then(|id| options.fontdb.face(id))
            .map(|face| &face.families);
        eprintln!(
            "native resolved={:?}; SVG generic resolved={family:?}",
            native.report()
        );
        let tree = usvg::Tree::from_str(&output.svg, &options).unwrap();
        let mut inspected = 0;
        // then
        for row in xml.descendants().filter(|n| {
            n.attribute("data-kind").is_some_and(|kind| {
                kind.starts_with("data-row-") || matches!(kind, "mindmap-node" | "wbs-node")
            })
        }) {
            let id = row.attribute("id").unwrap();
            let owner_id = if row.attribute("data-kind").unwrap().starts_with("data-row-") {
                row.attribute("data-owner").unwrap()
            } else {
                id
            };
            let usvg::Node::Group(owner) = tree.node_by_id(owner_id).unwrap() else {
                panic!("owner must be group")
            };
            let bounds = owner
                .children()
                .iter()
                .find_map(|node| {
                    if let usvg::Node::Path(path) = node {
                        Some(path.abs_bounding_box())
                    } else {
                        None
                    }
                })
                .expect("owning box must paint a path");
            let usvg::Node::Group(group) = tree.node_by_id(id).unwrap() else {
                panic!("row must be group")
            };
            let mut pending = vec![group.as_ref()];
            while let Some(group) = pending.pop() {
                for child in group.children() {
                    match child {
                        usvg::Node::Group(group) => pending.push(group),
                        usvg::Node::Text(text) => {
                            inspected += 1;
                            let painted = text.abs_stroke_bounding_box();
                            let mut db = options.fontdb.clone();
                            let selected = (options.font_resolver.select_font)(
                                text.chunks()[0].spans()[0].font(),
                                &mut db,
                            )
                            .and_then(|id| db.face(id))
                            .map(|face| &face.families);
                            eprintln!("actual selected paint font={selected:?}");
                            let actual_fonts: std::collections::BTreeSet<_> = text
                                .layouted()
                                .iter()
                                .flat_map(|span| &span.positioned_glyphs)
                                .filter_map(|glyph| tree.fontdb().face(glyph.font))
                                .map(|face| format!("{:?}", face.families))
                                .collect();
                            eprintln!("actual glyph fonts={actual_fonts:?}");
                            eprintln!(
                                "SVG text {:?} spans {:?} paint={painted:?}",
                                text.chunks().iter().map(|c| c.text()).collect::<Vec<_>>(),
                                text.chunks()
                                    .iter()
                                    .flat_map(|c| c.spans())
                                    .map(|s| (
                                        s.font().families(),
                                        s.font_size(),
                                        s.font().weight()
                                    ))
                                    .collect::<Vec<_>>()
                            );
                            assert!(
                                painted.left() >= bounds.left() - 1.
                                    && painted.top() >= bounds.top() - 1.
                                    && painted.right() <= bounds.right() + 1.
                                    && painted.bottom() <= bounds.bottom() + 1.,
                                "text paint {painted:?} escapes {id} compartment {bounds:?}"
                            );
                        }
                        _ => {}
                    }
                }
            }
        }
        assert!(inspected >= 2, "no actual text inspected");
    }
}

#[test]
fn should_contain_builtin_tree_text_without_changing_its_connector_clip() {
    // given
    let source = crate::examples::source("tree").unwrap();
    let output = diagram_engine::render(source, &Default::default()).unwrap();
    // when
    let display = DisplaySvg::parse(&output.svg, &AtomicBool::new(false)).unwrap();
    // then
    let mut right = 0_f32;
    let mut groups = vec![display.tree.root()];
    while let Some(group) = groups.pop() {
        for node in group.children() {
            match node {
                usvg::Node::Group(group) => groups.push(group),
                usvg::Node::Text(text) => right = right.max(text.abs_stroke_bounding_box().right()),
                _ => {}
            }
        }
    }
    assert!(
        right > display.tree.size().width(),
        "builtin must reproduce clipped text"
    );
    assert!(
        display.canvas.right() >= right,
        "text paint ends at {right}, canvas ends at {}",
        display.canvas.right()
    );
    assert_eq!(
        display.canvas.left(),
        0.,
        "do not reveal the left connector"
    );
    let document = roxmltree::Document::parse(&output.svg).unwrap();
    let viewbox = document.root_element().attribute("viewBox").unwrap();
    let values: Vec<f32> = viewbox
        .split_whitespace()
        .map(|value| value.parse().unwrap())
        .collect();
    let (width, height) = display.dimensions();
    let expanded = output.svg.replace(
        &format!("viewBox=\"{viewbox}\""),
        &format!("viewBox=\"{} {} {width} {height}\"", values[0], values[1]),
    );
    let raster = (width as u32 * 4, height as u32 * 4);
    assert_eq!(
        display
            .render(raster, &AtomicBool::new(false))
            .unwrap()
            .as_bytes(0),
        pinned(&expanded, raster).as_bytes(0),
        "same tree and a test-only equivalent root canvas must match every color channel"
    );
}

fn svg(body: &str) -> String {
    format!(r#"<svg xmlns="http://www.w3.org/2000/svg" width="60" height="60">{body}</svg>"#)
}

fn parse(source: &str) -> DisplaySvg {
    DisplaySvg::parse(source, &AtomicBool::new(false)).unwrap()
}

fn pinned(source: &str, dimensions: (u32, u32)) -> std::sync::Arc<gpui::RenderImage> {
    let renderer = gpui::SvgRenderer::new(std::sync::Arc::new(()));
    let tree = renderer.parse_svg(source.as_bytes()).unwrap();
    renderer
        .render_parsed(
            &tree,
            gpui::SvgSize::ExactSize(gpui::size(
                gpui::DevicePixels(dimensions.0 as i32),
                gpui::DevicePixels(dimensions.1 as i32),
            )),
        )
        .unwrap()
}

#[test]
fn should_preserve_root_clipping_when_leftward_text_would_expose_nontext() {
    // given
    let source = svg(
        r#"<path d="M-25 40H-5" stroke="red" stroke-width="8"/><text x="-30" y="20">leftward</text>"#,
    );
    // when
    let display = parse(&source);
    let image = display.render((60, 60), &AtomicBool::new(false)).unwrap();
    // then
    assert_eq!(display.dimensions(), (60., 60.));
    assert!(display.diagnostic().unwrap().contains("non-text"));
    assert_eq!(image.as_bytes(0), pinned(&source, (60, 60)).as_bytes(0));
}

#[test]
fn should_preserve_explicit_effects_and_clips_without_normalizing() {
    // given
    let effects = [
        (
            r#"<clipPath id="e"><rect x="0" y="0" width="20" height="60"/></clipPath>"#,
            "clip-path",
        ),
        (
            r#"<mask id="e"><rect width="20" height="60" fill="white"/></mask>"#,
            "mask",
        ),
        (
            r#"<filter id="e"><feGaussianBlur stdDeviation="3"/></filter>"#,
            "filter",
        ),
    ];
    // when
    for (definition, attribute) in effects {
        let source = svg(&format!(
            r#"<defs>{definition}</defs><g {attribute}="url(#e)"><text x="-20" y="20" fill="blue">outside canvas text</text></g>"#
        ));
        let display = parse(&source);
        // then
        assert_eq!(display.dimensions(), (60., 60.));
        assert!(display.diagnostic().unwrap().contains("effect"));
        assert_eq!(
            display
                .render((60, 60), &AtomicBool::new(false))
                .unwrap()
                .as_bytes(0),
            pinned(&source, (60, 60)).as_bytes(0)
        );
    }
}

#[test]
fn should_match_pinned_font_selection_and_full_color_pixels() {
    // given
    for family in [
        "Liberation Sans",
        "DejaVu Sans",
        "sans-serif",
        "serif",
        "monospace",
        "Missing Font Family",
    ] {
        let source = format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="400" height="80"><defs><linearGradient id="g"><stop stop-color="#8040e0"/><stop offset="1" stop-color="#30d060"/></linearGradient></defs><rect width="400" height="80" fill="url(#g)" opacity="0.4"/><text x="20" y="32" font-family="{family}" font-size="16" fill="#f03090" opacity="0.7">office ffi x́ A 世界 🐈</text><text x="20" y="64" font-family="{family}" font-size="16" font-style="italic" stroke="#30a0e0" stroke-width="0.5" fill="#ff8000">jfy package.json</text></svg>"##
        );
        // when
        let display = parse(&source);
        let actual = display.render((400, 80), &AtomicBool::new(false)).unwrap();
        // then
        assert_eq!(display.dimensions(), (400., 80.));
        assert_eq!(
            actual.as_bytes(0),
            pinned(&source, (400, 80)).as_bytes(0),
            "font {family}"
        );
    }
}

#[test]
fn should_unpremultiply_and_swap_all_color_channels() {
    // given
    let source = svg(
        r##"<rect width="20" height="20" fill="#804020"/><rect x="20" width="20" height="20" fill="#804020" opacity="0.5"/>"##,
    );
    // when
    let image = parse(&source)
        .render((60, 60), &AtomicBool::new(false))
        .unwrap();
    let bytes = image.as_bytes(0).unwrap();
    // then
    assert_eq!(&bytes[0..4], &[32, 64, 128, 255]);
    assert_eq!(&bytes[20 * 4..21 * 4], &[31, 63, 127, 128]);
    assert_eq!(&bytes[40 * 4..41 * 4], &[0, 0, 0, 0]);
}

#[test]
fn should_contain_transformed_stroked_and_fallback_text_paint() {
    // given: fixed large oracle canvas, independent of the bounds algorithm.
    let bodies = [
        r#"<g transform="translate(-20 -10) scale(1.5 2)"><text y="15" font-style="italic">jfy</text></g>"#,
        r#"<g transform="translate(10 10) rotate(-40)"><text y="10" font-family="DejaVu Sans">office ffi</text></g>"#,
        r#"<g transform="skewX(30)"><g transform="translate(-30 55) scale(1.2 .8)"><text font-size="25" stroke="red" stroke-width="3">x́<tspan dy="20">jfy</tspan></text></g></g>"#,
        r#"<g transform="translate(45 40) scale(2)"><text font-family="Missing Font Family" font-size="20">A 世界 🐈</text></g>"#,
    ];
    // when
    for body in bodies {
        let display = parse(&svg(body));
        let mut oracle = resvg::tiny_skia::Pixmap::new(1000, 1000).unwrap();
        resvg::render(
            &display.tree,
            usvg::Transform::from_translate(200., 200.),
            &mut oracle.as_mut(),
        );
        let (width, height) = display.dimensions();
        let image = display
            .render((width as u32, height as u32), &AtomicBool::new(false))
            .unwrap();
        let mut painted = 0;
        // then
        for (index, pixel) in oracle.data().as_chunks::<4>().0.iter().enumerate() {
            if pixel[3] == 0 {
                continue;
            }
            painted += 1;
            let x = (index % 1000) as f32 - 200.;
            let y = (index / 1000) as f32 - 200.;
            assert!(
                x >= display.canvas.left()
                    && x < display.canvas.right()
                    && y >= display.canvas.top()
                    && y < display.canvas.bottom(),
                "paint {x},{y} outside {:?}: {body}",
                display.canvas
            );
        }
        assert!(painted > 100);
        assert!(
            image
                .as_bytes(0)
                .unwrap()
                .as_chunks::<4>()
                .0
                .iter()
                .filter(|pixel| pixel[3] > 0)
                .count()
                > 100
        );
    }
}

#[test]
fn should_ignore_invisible_and_transparent_paint_but_include_visible_strokes() {
    // given
    for attribute in [
        "visibility=\"hidden\"",
        "opacity=\"0\"",
        "fill=\"none\"",
        "fill-opacity=\"0\"",
        "fill=\"none\" stroke=\"red\" stroke-opacity=\"0\"",
    ] {
        let source = svg(&format!(
            r#"<text x="-50" y="20" {attribute}>invisible overflow</text>"#
        ));
        // when
        let display = parse(&source);
        // then
        assert_eq!(display.dimensions(), (60., 60.), "{attribute}");
    }
    let display = parse(&svg(
        r#"<text x="-10" y="20" fill="none" stroke="blue" stroke-width="6">stroke extends across both edges</text>"#,
    ));
    assert!(display.canvas.left() < 0. && display.canvas.right() > 60.);
}

#[test]
fn should_reject_cancelled_low_budget_and_extreme_bounds_before_raster_allocation() {
    // given
    let source = svg("<text y=\"20\">text</text>");
    // when / then
    assert!(DisplaySvg::parse(&source, &AtomicBool::new(true)).is_err());
    for controls in [
        Controls {
            items: 1,
            ..Controls::default()
        },
        Controls {
            depth: 1,
            ..Controls::default()
        },
        Controls {
            points: 1,
            ..Controls::default()
        },
    ] {
        assert!(DisplaySvg::parse_controlled(&source, &AtomicBool::new(false), controls).is_err());
    }
    for transform in [
        "translate(1e30)",
        "matrix(NaN 0 0 1 0 0)",
        "scale(1e30)",
        "matrix(0 0 0 0 0 0)",
    ] {
        assert!(
            DisplaySvg::parse(
                &svg(&format!(
                    r#"<g transform="{transform}"><text y="20">extreme</text></g>"#
                )),
                &AtomicBool::new(false)
            )
            .is_err(),
            "{transform}"
        );
    }
    let display = parse(&source);
    for size in [(0, 1), (4097, 1), (4096, 4096)] {
        assert!(display.render(size, &AtomicBool::new(false)).is_err());
    }
    assert!(display.render((60, 60), &AtomicBool::new(true)).is_err());
}

#[test]
fn should_keep_expanded_logical_size_independent_of_raster_cap() {
    // given
    let source = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1000" height="10000"><text x="990" y="9000">overflow text</text></svg>"#;
    let display = parse(source);
    // when
    let layout = crate::viewport::layout(
        display.dimensions(),
        crate::viewport::Viewport {
            size: (800., 600.),
            device_scale: 2.,
        },
        crate::viewport::Zoom::Percent(200),
    )
    .unwrap();
    let image = display
        .render(layout.raster_size, &AtomicBool::new(false))
        .unwrap();
    // then
    assert!(layout.logical_size.0 > 2000.);
    assert_eq!(layout.logical_size.1, 20000.);
    assert!(image.as_bytes(0).unwrap().len() <= 4_194_304 * 4);
}

#[test]
fn should_preserve_fractional_original_sides_when_only_text_on_other_sides_expands() {
    // given
    let source = r#"<svg xmlns="http://www.w3.org/2000/svg" width="60.25" height="60.5"><text x="-20" y="20">left</text></svg>"#;
    // when
    let display = parse(source);
    // then
    assert!(display.canvas.left() < 0.);
    assert_eq!(display.canvas.right(), 60.25);
    assert_eq!(display.canvas.bottom(), 60.5);
}

#[test]
fn should_keep_percent_viewbox_alignment_and_negative_scene_origins() {
    // given
    let sources = [
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="100%" viewBox="-20 -30 120 80"><text x="0" y="0" fill="red">view</text></svg>"#,
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="180" height="100" viewBox="-20 -30 120 80" preserveAspectRatio="xMaxYMin meet"><text x="0" y="0" fill="red">view</text></svg>"#,
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="180" height="100" viewBox="-20 -30 120 80" preserveAspectRatio="none"><text x="0" y="0" fill="red">view</text></svg>"#,
    ];
    // when
    for source in sources {
        let display = parse(source);
        let raster = (display.dimensions().0 as u32, display.dimensions().1 as u32);
        // then
        assert_eq!(
            display
                .render(raster, &AtomicBool::new(false))
                .unwrap()
                .as_bytes(0),
            pinned(source, raster).as_bytes(0)
        );
        assert!(display.diagnostic().is_none());
    }
}

#[test]
fn should_reject_excessive_bitmap_glyph_decode_area() {
    // given: declared embedded bitmap dimensions, without allocating its pixels.
    let size = usvg::Size::from_wh(65536., 65536.).unwrap();
    // when / then
    assert!(validate_bitmap(size).is_err());
}

#[test]
#[cfg(target_os = "linux")]
fn should_bound_aggregate_bitmap_glyph_work_before_rasterization() {
    // given: each installed Noto bitmap fits, but eight exceed the smaller job budget.
    assert!(
        crate::svg_fonts::options()
            .fontdb
            .query(&usvg::fontdb::Query {
                families: &[usvg::fontdb::Family::Name("Noto Color Emoji")],
                ..Default::default()
            })
            .is_some(),
        "this Linux integration test requires installed Noto Color Emoji; portable bitmap limits are tested separately"
    );
    let single = svg(r#"<text y="20" font-family="Noto Color Emoji">🐈</text>"#);
    let repeated = svg(r#"<text y="20" font-family="Noto Color Emoji">🐈🐈🐈🐈🐈🐈🐈🐈</text>"#);
    let controls = Controls {
        bitmap_pixels: 100_000,
        ..Controls::default()
    };
    // when
    let one = DisplaySvg::parse_controlled(&single, &AtomicBool::new(false), controls);
    let many = DisplaySvg::parse_controlled(&repeated, &AtomicBool::new(false), controls);
    // then
    assert!(one.is_ok());
    assert!(many.err().unwrap().contains("decoded bitmap pixels"));
}

#[test]
fn should_ignore_transparent_nontext_when_admitting_leftward_expansion() {
    // given
    for style in [
        "opacity=\"0\"",
        "visibility=\"hidden\"",
        "stroke-opacity=\"0\"",
    ] {
        let source = svg(&format!(
            r#"<path d="M-25 40H-5" fill="none" stroke="red" {style}/><text x="-30" y="20">left</text>"#
        ));
        // when
        let display = parse(&source);
        // then
        assert!(display.canvas.left() < 0.);
        assert!(matches!(display.containment, Containment::Expanded));
    }
}

#[test]
fn should_preserve_pattern_paint_without_normalizing() {
    // given
    let source = svg(
        r#"<defs><pattern id="p" patternUnits="userSpaceOnUse" width="10" height="10"><rect width="5" height="10" fill="red"/></pattern></defs><text x="-20" y="20" fill="url(#p)">pattern text</text>"#,
    );
    // when
    let display = parse(&source);
    // then
    assert_eq!(display.dimensions(), (60., 60.));
    assert!(display.diagnostic().unwrap().contains("pattern"));
    assert_eq!(
        display
            .render((60, 60), &AtomicBool::new(false))
            .unwrap()
            .as_bytes(0),
        pinned(&source, (60, 60)).as_bytes(0)
    );
}

#[test]
fn should_charge_paths_inside_svg_images_in_the_same_traversal() {
    // given: SVG glyphs use this same nested-image branch in usvg's flattened tree.
    let inner =
        svg(r#"<g transform="translate(5 5) rotate(10)"><path d="M0 0H20V20H0Z" fill="red"/></g>"#);
    let encoded: String = inner.bytes().map(|byte| format!("%{byte:02X}")).collect();
    let source = svg(&format!(
        r#"<image href="data:image/svg+xml,{encoded}" width="60" height="60"/>"#
    ));
    // when
    let display = parse(&source);
    let limited = DisplaySvg::parse_controlled(
        &source,
        &AtomicBool::new(false),
        Controls {
            points: 1,
            ..Controls::default()
        },
    );
    // then
    assert!(limited.err().unwrap().contains("path points"));
    assert_eq!(
        display
            .render((60, 60), &AtomicBool::new(false))
            .unwrap()
            .as_bytes(0),
        pinned(&source, (60, 60)).as_bytes(0)
    );
}

fn tiny_png_uri() -> String {
    let mut data = std::io::Cursor::new(Vec::new());
    image::RgbaImage::from_pixel(1, 1, image::Rgba([240, 30, 80, 255]))
        .write_to(&mut data, image::ImageFormat::Png)
        .unwrap();
    let encoded: String = data
        .into_inner()
        .iter()
        .map(|byte| format!("%{byte:02X}"))
        .collect();
    format!("data:image/png,{encoded}")
}

#[test]
fn should_reject_repeated_legal_bitmaps_before_decoding_or_painting() {
    // given
    let item = format!(
        r#"<image href="{}" width="60" height="60"/>"#,
        tiny_png_uri()
    );
    let single = parse(&svg(&item));
    let many = parse(&svg(&item.repeat(70)));
    let control = Controls {
        bitmap_pixels: 2,
        ..Controls::default()
    };
    // when
    let decode =
        DisplaySvg::parse_controlled(&svg(&item.repeat(3)), &AtomicBool::new(false), control);
    let paint = raster_guard::check(
        &many.tree,
        Transform::from_scale(1024. / 60., 1024. / 60.),
        (1024, 1024),
        &AtomicBool::new(false),
    );
    // then
    assert!(decode.err().unwrap().contains("decoded bitmap pixels"));
    assert!(
        raster_guard::check(
            &single.tree,
            Transform::from_scale(1024. / 60., 1024. / 60.),
            (1024, 1024),
            &AtomicBool::new(false)
        )
        .is_ok()
    );
    assert!(paint.err().unwrap().contains("paint work"));
}

#[test]
fn should_reject_filter_image_with_cancelling_pattern_shear_before_rasterization() {
    // given
    let source = r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
      <defs>
        <pattern id="p" patternUnits="userSpaceOnUse" width="100" height="100"
                 patternTransform="matrix(1 0 -50 1 0 0)">
          <rect width="1" height="1"/>
        </pattern>
        <rect id="art" width="1" height="1" fill="url(#p)"/>
        <filter id="f" filterUnits="userSpaceOnUse" x="0" y="0" width="1" height="1">
          <feImage href="#art" x="0" y="0" width="1" height="1"/>
        </filter>
      </defs>
      <g transform="matrix(1 0 50 1 0 0)">
        <rect width="1" height="1" filter="url(#f)"/>
      </g>
    </svg>"##;
    let display = parse(source);
    // when
    // Never render this source: resvg's scale-only feImage context would request
    // a 250100-by-100 pattern tile, despite the cancelling parent/pattern shears.
    let result = raster_guard::check(
        &display.tree,
        Transform::identity(),
        (100, 100),
        &AtomicBool::new(false),
    );
    // then
    let error = result.expect_err("filter image must be rejected before any raster allocation");
    assert!(error.contains("feImage"), "{error}");
    assert!(error.contains("unsupported bounded raster work"), "{error}");
}

#[test]
fn should_reject_oversize_or_cumulative_effect_surfaces_before_allocation() {
    // given
    let cases = [
        svg(
            r#"<defs><filter id="f"><feConvolveMatrix order="3" kernelMatrix="0 1 0 1 1 1 0 1 0"/></filter></defs><rect width="60" height="60" filter="url(#f)"/>"#,
        ),
        svg(
            r#"<defs><pattern id="p" patternUnits="userSpaceOnUse" width="100000" height="100000"><rect width="10" height="10"/></pattern></defs><rect width="60" height="60" fill="url(#p)"/>"#,
        ),
        svg(
            r#"<defs><filter id="f" filterUnits="userSpaceOnUse" x="-100000" y="-100000" width="200000" height="200000"><feGaussianBlur stdDeviation="2"/></filter></defs><rect width="60" height="60" filter="url(#f)"/>"#,
        ),
        svg(&format!(
            "{}<rect width=\"60\" height=\"60\"/>{}",
            "<g opacity=\"0.9\">".repeat(70),
            "</g>".repeat(70)
        )),
    ];
    // when / then
    for source in cases {
        let display = parse(&source);
        assert!(
            raster_guard::check(
                &display.tree,
                Transform::from_scale(1024. / 60., 1024. / 60.),
                (1024, 1024),
                &AtomicBool::new(false)
            )
            .is_err()
        );
    }
}
