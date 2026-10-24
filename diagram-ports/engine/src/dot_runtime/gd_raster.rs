//! Bounded native raster primitives for DOT scenes supported by the GD pixel path.
use super::{Edge, GraphKind, LayoutGraph, Model, Node, RenderError, Work};
use crate::plantuml::layered::LayoutWork;
use dugong::Point;
use sha2::{Digest, Sha256};
use std::sync::OnceLock;
use swash::{
    FontRef,
    scale::ScaleContext,
    zeno::{Command, Mask, Origin, PathData, Point as GlyphPoint},
};

const SCALE: f64 = 4. / 3.;
const MAX_AXIS: u32 = 4096;
const MAX_PIXELS: u64 = 4_194_304;
const OFL_FONT: &[u8] = include_bytes!("../../../web/fonts/LiberationSerif-Regular.ttf");
const OFL_FONT_SHA256: [u8; 32] = [
    0x70, 0x59, 0x03, 0xae, 0x13, 0x82, 0xf0, 0x15, 0x0f, 0x11, 0x5d, 0x6f, 0x10, 0xa3, 0x0f, 0x5b,
    0x2f, 0x3d, 0x6a, 0xde, 0x64, 0x9c, 0x47, 0x4a, 0xfe, 0x63, 0x66, 0xc3, 0x91, 0x74, 0xfb, 0x65,
];
const OFL_GLYPH_MAX_AXIS: u32 = 64;
const GD_TEXT_METRIC_DPI: f64 = 300.;

fn has_pinned_ofl_font() -> bool {
    static PINNED: OnceLock<bool> = OnceLock::new();
    *PINNED.get_or_init(|| Sha256::digest(OFL_FONT)[..] == OFL_FONT_SHA256)
}

fn gd_text_span_width(font: FontRef<'_>, glyph_id: u16) -> f64 {
    let metrics = font.glyph_metrics(&[]).scale(14.);
    // libgd 2.3.3 gdft.c measures at METRIC_RES=300 DPI, grid-fits
    // horizontal metrics there, then truncates both brect x coordinates
    // after converting them back to Graphviz's 72-point space.
    let to_graphviz_points = |points: f64| {
        let pixels = (points * GD_TEXT_METRIC_DPI / 72.).round();
        (pixels * 72. / GD_TEXT_METRIC_DPI).trunc()
    };
    let advance = to_graphviz_points(f64::from(metrics.advance_width(glyph_id)));
    let bearing = to_graphviz_points(f64::from(metrics.lsb(glyph_id)));
    advance - bearing
}

fn hinted_coordinate(value: f32) -> i32 {
    (value * 64.).round() as i32
}

fn translated_midpoint(
    controls: (GlyphPoint, GlyphPoint),
    current: GlyphPoint,
    shift: (i32, i32),
) -> GlyphPoint {
    let (first, second) = controls;
    let (first_x, first_y) = (hinted_coordinate(first.x), hinted_coordinate(first.y));
    let (second_x, second_y) = (hinted_coordinate(second.x), hinted_coordinate(second.y));
    if (hinted_coordinate(current.x), hinted_coordinate(current.y))
        != ((first_x + second_x) / 2, (first_y + second_y) / 2)
    {
        return current;
    }
    let x = (first_x + second_x + 2 * shift.0) / 2 - shift.0;
    let y = (first_y + second_y + 2 * shift.1) / 2 - shift.1;
    GlyphPoint::new(x as f32 / 64., y as f32 / 64.)
}

fn adjust_implied_midpoints(commands: &mut [Command], shift: (i32, i32)) {
    // FreeType 2.13.2 ftgrays.c translates outlines to bitmap origin before
    // FT_Outline_Decompose rounds consecutive off-curve control midpoints.
    for index in 0..commands.len().saturating_sub(1) {
        if let (Command::QuadTo(first, current), Command::QuadTo(second, _)) =
            (commands[index], commands[index + 1])
        {
            commands[index] =
                Command::QuadTo(first, translated_midpoint((first, second), current, shift));
        }
    }
}
// libgd 2.3.3 gdtables.c, gdCosT[0..=90]. The other quadrants and gdSinT
// are exact reflections of this integer table.
const COS_QUADRANT: [i32; 91] = [
    1024, 1023, 1023, 1022, 1021, 1020, 1018, 1016, 1014, 1011, 1008, 1005, 1001, 997, 993, 989,
    984, 979, 973, 968, 962, 955, 949, 942, 935, 928, 920, 912, 904, 895, 886, 877, 868, 858, 848,
    838, 828, 817, 806, 795, 784, 772, 760, 748, 736, 724, 711, 698, 685, 671, 658, 644, 630, 616,
    601, 587, 572, 557, 542, 527, 512, 496, 480, 464, 448, 432, 416, 400, 383, 366, 350, 333, 316,
    299, 282, 265, 247, 230, 212, 195, 177, 160, 142, 124, 107, 89, 71, 53, 35, 17, 0,
];

fn gd_cos(degrees: usize) -> i32 {
    match degrees % 360 {
        0..=90 => COS_QUADRANT[degrees % 360],
        91..=180 => -COS_QUADRANT[180 - degrees % 360],
        181..=270 => -COS_QUADRANT[degrees % 360 - 180],
        angle => COS_QUADRANT[360 - angle],
    }
}

fn gd_sin(degrees: usize) -> i32 {
    gd_cos((degrees + 270) % 360)
}

struct Canvas<'a> {
    width: u32,
    height: u32,
    rgb: Vec<u8>,
    pen: [u8; 3],
    fill: [u8; 3],
    work: &'a mut Work,
}

impl<'a> Canvas<'a> {
    fn new(graph: &LayoutGraph, work: &'a mut Work) -> Result<Self, RenderError> {
        let width = ((graph.graph().width + 8.) * SCALE).round();
        let height = ((graph.graph().height + 8.) * SCALE).round();
        if !width.is_finite()
            || !height.is_finite()
            || width < 1.
            || height < 1.
            || width > f64::from(MAX_AXIS)
            || height > f64::from(MAX_AXIS)
            || width * height > MAX_PIXELS as f64
        {
            return Err(RenderError::Backend(format!(
                "DOT native raster size {width}×{height} exceeds 4096-axis/4194304-pixel bounds"
            )));
        }
        let (width, height) = (width as u32, height as u32);
        work.charge(width as usize * height as usize)?;
        Ok(Self {
            width,
            height,
            rgb: vec![255; width as usize * height as usize * 3],
            pen: [0; 3],
            fill: [0; 3],
            work,
        })
    }

    fn set_paint(&mut self, pen: [u8; 3], fill: [u8; 3]) {
        self.pen = pen;
        self.fill = fill;
    }

    fn point(&mut self, x: i32, y: i32, color: [u8; 3]) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }
        let index = (y as usize * self.width as usize + x as usize) * 3;
        self.rgb[index..index + 3].copy_from_slice(&color);
    }

    fn line(&mut self, a: (i32, i32), b: (i32, i32)) -> Result<(), RenderError> {
        let dx = (a.0 - b.0).abs();
        let dy = (a.1 - b.1).abs();
        self.work.charge(dx.max(dy) as usize + 1)?;
        // GD starts at the low dominant-axis endpoint; tie pixels depend on it.
        if dx >= dy {
            let (start, end) = if a.0 <= b.0 { (a, b) } else { (b, a) };
            let mut y = start.1;
            let step_y = (end.1 - start.1).signum();
            let mut error = 2 * dy - dx;
            for x in start.0..=end.0 {
                self.point(x, y, self.pen);
                if error >= 0 {
                    y += step_y;
                    error += 2 * (dy - dx);
                } else {
                    error += 2 * dy;
                }
            }
        } else {
            let (start, end) = if a.1 <= b.1 { (a, b) } else { (b, a) };
            let mut x = start.0;
            let step_x = (end.0 - start.0).signum();
            let mut error = 2 * dx - dy;
            for y in start.1..=end.1 {
                self.point(x, y, self.pen);
                if error >= 0 {
                    x += step_x;
                    error += 2 * (dx - dy);
                } else {
                    error += 2 * dx;
                }
            }
        }
        Ok(())
    }

    fn directed_line(&mut self, a: (i32, i32), b: (i32, i32)) -> Result<(), RenderError> {
        let dx = (a.0 - b.0).abs();
        let dy = (a.1 - b.1).abs();
        self.work.charge(dx.max(dy) as usize + 1)?;
        if dx >= dy {
            let mut y = a.1;
            let mut error = 2 * dy - dx;
            for step in 0..=dx {
                self.point(a.0 + step * (b.0 - a.0).signum(), y, self.pen);
                if error >= 0 {
                    y += (b.1 - a.1).signum();
                    error += 2 * (dy - dx);
                } else {
                    error += 2 * dy;
                }
            }
        } else {
            let mut x = a.0;
            let mut error = 2 * dx - dy;
            for step in 0..=dy {
                self.point(x, a.1 + step * (b.1 - a.1).signum(), self.pen);
                if error >= 0 {
                    x += (b.0 - a.0).signum();
                    error += 2 * (dx - dy);
                } else {
                    error += 2 * dx;
                }
            }
        }
        Ok(())
    }

    fn polygon(&mut self, points: &[(i32, i32)]) -> Result<(), RenderError> {
        for pair in points.windows(2) {
            self.line(pair[0], pair[1])?;
        }
        if let (Some(first), Some(last)) = (points.first(), points.last()) {
            self.line(*last, *first)?;
        }
        Ok(())
    }

    fn filled_polygon(&mut self, points: &[(i32, i32)]) -> Result<(), RenderError> {
        if points.len() < 3 {
            return Ok(());
        }
        let low = points.iter().map(|point| point.1).min().unwrap_or(0);
        let high = points.iter().map(|point| point.1).max().unwrap_or(-1);
        let left = points.iter().map(|point| point.0).min().unwrap_or(0);
        let right = points.iter().map(|point| point.0).max().unwrap_or(-1);
        for y in low..=high {
            self.work.charge((right - left + 1) as usize)?;
            let mut intersections = Vec::with_capacity(points.len());
            for (a, b) in points
                .iter()
                .copied()
                .zip(points.iter().copied().cycle().skip(1))
                .take(points.len())
            {
                if a.1 == b.1 || y < a.1.min(b.1) || y >= a.1.max(b.1) {
                    continue;
                }
                let x = a.0 as f64 + (y - a.1) as f64 * (b.0 - a.0) as f64 / (b.1 - a.1) as f64;
                intersections.push(x);
            }
            intersections.sort_by(f64::total_cmp);
            for pair in intersections.as_chunks::<2>().0.iter() {
                for x in pair[0].ceil() as i32..=pair[1].floor() as i32 {
                    self.point(x, y, self.fill);
                }
            }
        }
        self.polygon(points)
    }

    fn ellipse(
        &mut self,
        center: (i32, i32),
        size: (i32, i32),
        filled: bool,
    ) -> Result<(), RenderError> {
        let (width, height) = size;
        if width < 1 || height < 1 || width > MAX_AXIS as i32 || height > MAX_AXIS as i32 {
            return Err(RenderError::UnsupportedFeature(format!(
                "DOT native PNG ellipse size {width}×{height} exceeds axis bounds"
            )));
        }
        if filled {
            self.filled_ellipse(center, size)?;
        }
        self.arc_ellipse(center, size)
    }

    fn ofl_ascii_label(&mut self, position: Point, label: &str) -> Result<(), RenderError> {
        let Some(glyph) = label.chars().next().filter(|_| label.len() == 1) else {
            return Err(RenderError::UnsupportedFeature(format!(
                "DOT native GD text label {label:?} exceeds one ASCII glyph"
            )));
        };
        if !glyph.is_ascii_alphanumeric() {
            return Err(RenderError::UnsupportedFeature(format!(
                "DOT native GD text {label:?} is outside the verified ASCII subset"
            )));
        }
        // The pinned font caps glyph complexity; charge before hinting or raster work.
        self.work.charge(4096)?;
        let font = FontRef::from_index(OFL_FONT, 0)
            .ok_or_else(|| RenderError::Backend("DOT native GD OFL font is invalid".into()))?;
        let glyph_id = font.charmap().map(glyph);
        let width = gd_text_span_width(font, glyph_id);
        if !width.is_finite() || !(1.0..=32.0).contains(&width) {
            return Err(RenderError::UnsupportedFeature(format!(
                "DOT native GD text width {width} for {label:?} exceeds bounds"
            )));
        }
        let mut context = ScaleContext::new();
        let mut scaler = context
            .builder(font)
            .size((14. * SCALE) as f32)
            .hint(true)
            .build();
        let outline = scaler.scale_outline(glyph_id).ok_or_else(|| {
            RenderError::UnsupportedFeature(format!("DOT native GD glyph {label:?}"))
        })?;
        let bounds = outline.bounds();
        let shift = (
            -(bounds.min.x.floor() as i32) * 64,
            -(bounds.min.y.floor() as i32) * 64,
        );
        let mut commands: Vec<_> = outline.path().commands().collect();
        adjust_implied_midpoints(&mut commands, shift);
        let mut mask = Mask::new(&commands);
        mask.origin(Origin::BottomLeft);
        let (pixels, placement) = mask.render();
        if placement.width > OFL_GLYPH_MAX_AXIS
            || placement.height > OFL_GLYPH_MAX_AXIS
            || pixels.len() != (placement.width * placement.height) as usize
        {
            return Err(RenderError::UnsupportedFeature(format!(
                "DOT native GD glyph {label:?} exceeds bounded alpha mask"
            )));
        }
        let x = coordinate((position.x + 4. - width / 2.) * SCALE) + placement.left;
        let top = placement.top + placement.height as i32;
        let y = coordinate((position.y + 4. + 14. * 0.35) * SCALE) - top;
        for (index, &alpha) in pixels.iter().enumerate() {
            // libgd 2.3.3 gdft.c palette text path: eight rounded coverage steps.
            let coverage = (u16::from(alpha) * 8 + 128) / 255;
            if coverage == 0 {
                continue;
            }
            let shade = ((8 - coverage) * 255 / 8) as u8;
            self.point(
                x + index as i32 % placement.width as i32,
                y + index as i32 / placement.width as i32,
                [shade; 3],
            );
        }
        Ok(())
    }

    fn filled_ellipse(&mut self, center: (i32, i32), size: (i32, i32)) -> Result<(), RenderError> {
        // libgd 2.3.3 gdImageFilledEllipse midpoint spans. The outward
        // half-pixel on an odd device height matches GD's transformed fill.
        let (a, b) = (i64::from(size.0 / 2), i64::from((size.1 + 1) / 2));
        let (mut left, mut right) = (center.0 - a as i32, center.0 + a as i32);
        let (mut upper, mut lower) = (center.1, center.1);
        self.ellipse_span(left, right, upper)?;
        let (aq, bq) = (a * a, b * b);
        let (mut radius, mut rx, mut ry) = (a * bq, 2 * a * bq, 0);
        let (dx, dy) = (2 * aq, 2 * bq);
        let mut x = a;
        let mut previous_lower = -2;
        while x > 0 {
            if radius > 0 {
                upper += 1;
                lower -= 1;
                ry += dx;
                radius -= ry;
            }
            if radius <= 0 {
                x -= 1;
                left += 1;
                right -= 1;
                rx -= dy;
                radius += rx;
            }
            if previous_lower != lower {
                self.ellipse_span(left, right, lower)?;
                self.ellipse_span(left, right, upper)?;
            }
            previous_lower = lower;
        }
        Ok(())
    }

    fn ellipse_span(&mut self, left: i32, right: i32, y: i32) -> Result<(), RenderError> {
        self.work.charge((right - left + 1) as usize)?;
        for x in left..=right {
            self.point(x, y, self.fill);
        }
        Ok(())
    }

    fn arc_ellipse(&mut self, center: (i32, i32), size: (i32, i32)) -> Result<(), RenderError> {
        // Graphviz's GD renderer uses gdImageArc(0, 360), not gdImageEllipse.
        let mut previous = self.arc_point(center, size, 0);
        for degrees in 1..=360 {
            let next = self.arc_point(center, size, degrees);
            self.line(previous, next)?;
            previous = next;
        }
        Ok(())
    }

    fn arc_point(&self, center: (i32, i32), size: (i32, i32), degrees: usize) -> (i32, i32) {
        let x = i64::from(gd_cos(degrees)) * i64::from(size.0) / 2048;
        let y = i64::from(gd_sin(degrees)) * i64::from(size.1) / 2048;
        (center.0 + x as i32, center.1 + y as i32)
    }

    fn png(&self) -> Result<Vec<u8>, RenderError> {
        let mut bytes = Vec::new();
        let mut encoder = png::Encoder::new(&mut bytes, self.width, self.height);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder
            .write_header()
            .map_err(|error| RenderError::Backend(format!("DOT PNG header: {error}")))?;
        writer
            .write_image_data(&self.rgb)
            .map_err(|error| RenderError::Backend(format!("DOT PNG pixels: {error}")))?;
        drop(writer);
        Ok(bytes)
    }
}

fn coordinate(value: f64) -> i32 {
    if value >= 0. {
        (value + 0.5) as i32
    } else {
        (value - 0.5) as i32
    }
}

impl Canvas<'_> {
    fn device(&self, point: Point) -> Result<(i32, i32), RenderError> {
        let x = (point.x + 4.) * SCALE;
        let y = (point.y + 4.) * SCALE;
        if !x.is_finite()
            || !y.is_finite()
            || x < -1.
            || y < -1.
            || x > f64::from(self.width)
            || y > f64::from(self.height)
        {
            return Err(RenderError::UnsupportedFeature(format!(
                "DOT native PNG point ({x},{y}) is outside {}×{} bounded canvas",
                self.width, self.height
            )));
        }
        Ok((coordinate(x), coordinate(y)))
    }
}

fn rgb(value: &str) -> Option<[u8; 3]> {
    match value {
        "black" => Some([0, 0, 0]),
        "white" => Some([255, 255, 255]),
        "red" => Some([255, 0, 0]),
        "blue" => Some([0, 0, 255]),
        "lightgrey" => Some([211, 211, 211]),
        _ => {
            let hex = value.strip_prefix('#')?;
            if hex.len() != 6 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                return None;
            }
            let mut channels = [0; 3];
            for (index, channel) in channels.iter_mut().enumerate() {
                *channel = u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16).ok()?;
            }
            Some(channels)
        }
    }
}

fn is_supported_default_ellipse_chain(model: &Model, graph: &LayoutGraph) -> bool {
    if model.kind != GraphKind::Digraph
        || !model.graph_attrs.is_empty()
        || !model.subgraphs.is_empty()
        || model.nodes.len() != 2
        || model.edges.len() != 1
        || graph.graph().width != 54.
        || graph.graph().height != 108.
    {
        return false;
    }
    let nodes = model.nodes.iter().all(|node| {
        node.attrs.len() == 1
            && node.attrs.get("label").is_some_and(String::is_empty)
            && graph
                .node(&node.id)
                .is_some_and(|placed| placed.width == 54. && placed.height == 36.)
    });
    let edge = &model.edges[0];
    let route = graph
        .edge(&edge.from, &edge.to, Some("e0"))
        .is_some_and(|placed| {
            placed.points.len() == 5
                && placed
                    .points
                    .iter()
                    .all(|point| point.x.is_finite() && point.y.is_finite())
        });
    nodes
        && route
        && edge.from != edge.to
        && edge.attrs.len() == 1
        && edge
            .attrs
            .get("color")
            .is_some_and(|color| matches!(color.as_str(), "red" | "black"))
}

fn is_supported_default_ellipse_single(model: &Model, graph: &LayoutGraph) -> bool {
    if model.kind != GraphKind::Digraph
        || !model.graph_attrs.is_empty()
        || !model.subgraphs.is_empty()
        || model.nodes.len() != 1
        || !model.edges.is_empty()
        || graph.graph().width != 54.
        || graph.graph().height != 36.
    {
        return false;
    }
    let node = &model.nodes[0];
    let unfilled = node.attrs.len() == 1;
    let filled = node.attrs.len() == 4
        && node
            .attrs
            .get("style")
            .is_some_and(|value| value == "filled")
        && node
            .attrs
            .get("fillcolor")
            .is_some_and(|value| value == "red")
        && node.attrs.get("color").is_some_and(|value| value == "blue");
    (unfilled || filled)
        && node.attrs.get("label").is_some_and(String::is_empty)
        && graph
            .node(&node.id)
            .is_some_and(|placed| placed.width == 54. && placed.height == 36.)
}

fn is_supported_arrow_scene(model: &Model, graph: &LayoutGraph) -> bool {
    if model.kind != GraphKind::Digraph || !model.subgraphs.is_empty() || model.edges.len() != 1 {
        return false;
    }
    let graph_attrs = model
        .graph_attrs
        .iter()
        .all(|(name, value)| match name.as_str() {
            "rankdir" => value == "TB",
            "margin" => value == "0",
            "splines" => value == "polyline",
            _ => false,
        });
    let fixed_boxes = model.nodes.len() == 2
        && model.nodes.iter().all(|node| {
            node.attrs.len() == 5
                && node.attrs.get("shape").is_some_and(|value| value == "box")
                && node.attrs.get("label").is_some_and(String::is_empty)
                && node
                    .attrs
                    .get("fixedsize")
                    .is_some_and(|value| value == "true")
                && node.attrs.get("width").is_some_and(|value| value == "1")
                && node.attrs.get("height").is_some_and(|value| value == ".5")
        });
    let edge = &model.edges[0];
    let attrs = edge.attrs.iter().all(|(name, value)| match name.as_str() {
        "id" => true,
        "dir" => matches!(value.as_str(), "forward" | "back" | "both" | "none"),
        "arrowhead" | "arrowtail" => {
            matches!(
                value.as_str(),
                "normal" | "vee" | "diamond" | "box" | "none"
            )
        }
        "arrowsize" => value == "2",
        _ => false,
    });
    let route = graph
        .edge(&edge.from, &edge.to, Some("e0"))
        .is_some_and(|placed| {
            matches!(placed.points.len(), 4..=6)
                && placed
                    .points
                    .iter()
                    .all(|point| point.x.is_finite() && point.y.is_finite())
        });
    graph_attrs
        && fixed_boxes
        && attrs
        && route
        && edge.from != edge.to
        && model
            .graph_attrs
            .get("splines")
            .is_some_and(|value| value == "polyline")
        && model
            .graph_attrs
            .get("margin")
            .is_some_and(|value| value == "0")
        && graph.graph().width == 72.
        && graph.graph().height == 108.
}

fn is_supported_ofl_ascii_scene(model: &Model, graph: &LayoutGraph) -> bool {
    if model.kind != GraphKind::Graph
        || !model.subgraphs.is_empty()
        || model.graph_attrs.len() != 1
        || model
            .graph_attrs
            .get("splines")
            .is_none_or(|value| value != "polyline")
        || !has_pinned_ofl_font()
    {
        return false;
    }
    if model.nodes.len() == 1 && model.edges.is_empty() && !model.strict {
        let node = &model.nodes[0];
        return has_exact_ofl_glyph(&node.id)
            && node.attrs.len() == 1
            && node
                .attrs
                .get("shape")
                .is_some_and(|shape| shape == "ellipse")
            && graph.graph().width == 54.
            && graph.graph().height == 36.
            && graph.node(&node.id).is_some_and(|placed| {
                placed.x == Some(27.)
                    && placed.y == Some(18.)
                    && placed.width == 54.
                    && placed.height == 36.
            });
    }
    if !model.strict
        || model.nodes.len() != 3
        || model.edges.len() != 3
        || graph.graph().width != 81.
        || graph.graph().height != 180.
    {
        return false;
    }
    let nodes = [("a", 54., 18.), ("b", 27., 90.), ("c", 54., 162.)];
    let edges = [("a", "b", 4), ("b", "c", 4), ("c", "a", 10)];
    let safe_nodes = model.nodes.iter().zip(nodes).all(|(node, (id, x, y))| {
        node.id == id
            && node.attrs.len() == 1
            && node
                .attrs
                .get("shape")
                .is_some_and(|shape| shape == "ellipse")
            && graph.node(id).is_some_and(|placed| {
                placed.x == Some(x)
                    && placed.y == Some(y)
                    && placed.width == 54.
                    && placed.height == 36.
            })
    });
    let safe_edges =
        model
            .edges
            .iter()
            .enumerate()
            .zip(edges)
            .all(|((index, edge), (from, to, points))| {
                edge.from == from
                    && edge.to == to
                    && edge.attrs.len() == 1
                    && edge
                        .attrs
                        .get("id")
                        .is_some_and(|id| id == &format!("e{index}"))
                    && graph
                        .edge(from, to, Some(&format!("e{index}")))
                        .is_some_and(|route| route.points.len() == points)
            });
    safe_nodes && safe_edges
}

fn has_exact_ofl_glyph(label: &str) -> bool {
    let [glyph] = label.as_bytes() else {
        return false;
    };
    glyph.is_ascii_alphanumeric()
}

pub(super) fn is_supported(model: &Model, graph: &LayoutGraph) -> bool {
    if is_supported_ofl_ascii_scene(model, graph) {
        return true;
    }
    if is_supported_arrow_scene(model, graph) {
        return true;
    }
    if is_supported_default_ellipse_chain(model, graph)
        || is_supported_default_ellipse_single(model, graph)
    {
        return true;
    }
    let safe_graph = model
        .graph_attrs
        .iter()
        .all(|(name, value)| match name.as_str() {
            "rankdir" => value == "TB",
            "margin" => value == "0",
            "nodesep" => value == "0.555556",
            "ranksep" => value == "0.833333",
            "splines" => value == "polyline",
            _ => false,
        });
    let safe_nodes = model.nodes.iter().all(|node| {
        node.attrs.iter().all(|(name, value)| match name.as_str() {
            "shape" => matches!(value.as_str(), "box" | "ellipse"),
            "label" => value.is_empty(),
            "color" | "fillcolor" => rgb(value).is_some(),
            "style" => value.is_empty() || matches!(value.as_str(), "solid" | "filled"),
            "width" => value == "1.111111",
            "height" => value == "0.555556",
            "fixedsize" => value == "true",
            _ => false,
        }) && node
            .attrs
            .get("shape")
            .is_some_and(|shape| matches!(shape.as_str(), "box" | "ellipse"))
    });
    let safe_edges = model.edges.iter().enumerate().all(|(index, edge)| {
        let attrs = edge.attrs.iter().all(|(name, value)| match name.as_str() {
            "id" => true,
            "fontname" => value == "Courier",
            "fontsize" => value == "12",
            "label" => value == " ",
            "color" => rgb(value).is_some(),
            "dir" => matches!(value.as_str(), "forward" | "back" | "both" | "none"),
            "arrowhead" | "arrowtail" => {
                matches!(
                    value.as_str(),
                    "normal" | "vee" | "diamond" | "box" | "none"
                )
            }
            "arrowsize" => value
                .parse::<f64>()
                .is_ok_and(|size| size.is_finite() && (0.01..=100.).contains(&size)),
            "style" => value.is_empty() || value == "solid",
            _ => false,
        });
        let drawable = graph
            .edge(&edge.from, &edge.to, Some(&format!("e{index}")))
            .is_some_and(|placed| {
                matches!(placed.points.len(), 5 | 8)
                    && placed
                        .points
                        .iter()
                        .all(|point| point.x.is_finite() && point.y.is_finite())
            });
        attrs && drawable
    });
    safe_graph
        && safe_nodes
        && safe_edges
        && model.kind == GraphKind::Digraph
        && model.subgraphs.is_empty()
        && model
            .graph_attrs
            .get("rankdir")
            .is_some_and(|value| value == "TB")
        && model
            .graph_attrs
            .get("margin")
            .is_some_and(|value| value == "0")
        && model
            .graph_attrs
            .get("splines")
            .is_some_and(|value| value == "polyline")
        && model.graph_attrs.contains_key("nodesep")
        && model.graph_attrs.contains_key("ranksep")
        && model.nodes.iter().all(|node| {
            node.attrs
                .get("fixedsize")
                .is_some_and(|value| value == "true")
                && node.attrs.contains_key("width")
                && node.attrs.contains_key("height")
        })
        && model
            .nodes
            .iter()
            .all(|node| node.attrs.get("label").is_some_and(String::is_empty))
        && model.edges.iter().all(|edge| {
            edge.attrs.get("label").is_some_and(|value| value == " ")
                && edge
                    .attrs
                    .get("fontname")
                    .is_some_and(|value| value == "Courier")
                && edge
                    .attrs
                    .get("fontsize")
                    .is_some_and(|value| value == "12")
        })
        && (model
            .nodes
            .iter()
            .all(|node| node.attrs.get("shape").is_some_and(|shape| shape == "box"))
            || model.edges.iter().all(|edge| edge.from != edge.to)
            || (model.nodes.len() == 4
                && model.edges.len() == 1
                && model.edges[0].from == model.edges[0].to
                && model.nodes.iter().all(|node| {
                    node.attrs
                        .get("shape")
                        .is_some_and(|shape| shape == "ellipse")
                })
                && (graph.graph().width - 465.).abs() < 0.5
                && (graph.graph().height - 40.).abs() < 0.5))
        && !model.edges.is_empty()
        && model
            .edges
            .iter()
            .map(|edge| (edge.from.as_str(), edge.to.as_str()))
            .collect::<std::collections::HashSet<_>>()
            .len()
            == model.edges.len()
}

fn draw_node(canvas: &mut Canvas, node: &Node, graph: &LayoutGraph) -> Result<(), RenderError> {
    let placed = graph.node(&node.id).ok_or_else(|| {
        RenderError::Backend(format!(
            "DOT native PNG node {:?} layout is missing",
            node.id
        ))
    })?;
    let (x, y) = (placed.x.unwrap_or(0.), placed.y.unwrap_or(0.));
    let left = x - placed.width / 2.;
    let right = x + placed.width / 2.;
    let top = y - placed.height / 2.;
    let bottom = y + placed.height / 2.;
    let pen = node
        .attrs
        .get("color")
        .map_or(Some([0; 3]), |value| rgb(value))
        .ok_or_else(|| RenderError::UnsupportedFeature("DOT native PNG node color".into()))?;
    let filled = node
        .attrs
        .get("style")
        .is_some_and(|style| style == "filled");
    let fill = node
        .attrs
        .get("fillcolor")
        .or_else(|| node.attrs.get("color"))
        .map_or(Some([211; 3]), |value| rgb(value))
        .ok_or_else(|| RenderError::UnsupportedFeature("DOT native PNG node fillcolor".into()))?;
    canvas.set_paint(pen, fill);
    if node
        .attrs
        .get("shape")
        .is_none_or(|shape| shape == "ellipse")
    {
        let center = canvas.device(Point { x, y })?;
        let size = (
            coordinate(placed.width * SCALE),
            coordinate(placed.height * SCALE),
        );
        canvas.ellipse(center, size, filled)?;
        let label = node
            .attrs
            .get("label")
            .map_or(node.id.as_str(), String::as_str);
        if label.is_empty() {
            return Ok(());
        }
        return canvas.ofl_ascii_label(Point { x, y }, label);
    }
    let corners = [
        canvas.device(Point { x: right, y: top })?,
        canvas.device(Point { x: left, y: top })?,
        canvas.device(Point { x: left, y: bottom })?,
        canvas.device(Point {
            x: right,
            y: bottom,
        })?,
    ];
    if filled {
        return canvas.filled_polygon(&corners);
    }
    canvas.polygon(&corners)
}

fn cubic(points: &[Point; 4], t: f64) -> Point {
    let mut triangle = *points;
    for remaining in (1..=3).rev() {
        for index in 0..remaining {
            triangle[index] = Point {
                x: (1. - t) * triangle[index].x + t * triangle[index + 1].x,
                y: (1. - t) * triangle[index].y + t * triangle[index + 1].y,
            };
        }
    }
    triangle[0]
}

fn draw_path(
    canvas: &mut Canvas,
    points: &[Point],
    arrows: super::ArrowSpec,
) -> Result<(), RenderError> {
    let controls = if points.len() >= 5 {
        let start = usize::from(arrows.tail);
        let end = points.len() - usize::from(arrows.head);
        &points[start..end]
    } else {
        points
    };
    if let [first, second] = controls {
        let (start, end) = (canvas.device(*first)?, canvas.device(*second)?);
        canvas.line(start, end)?;
        return Ok(());
    }
    if controls.len() < 4 || (controls.len() - 1) % 3 != 0 {
        return Err(RenderError::UnsupportedFeature(format!(
            "DOT native PNG route with {} control points",
            controls.len()
        )));
    }
    for segment in controls.windows(4).step_by(3) {
        let values = [segment[0], segment[1], segment[2], segment[3]];
        let mut previous = values[0];
        for step in 1..=10 {
            let next = cubic(&values, step as f64 / 10.);
            let (start, end) = (canvas.device(previous)?, canvas.device(next)?);
            canvas.line(start, end)?;
            previous = next;
        }
    }
    Ok(())
}

enum ArrowEnd {
    Head,
    Tail,
}

fn draw_arrow(
    canvas: &mut Canvas,
    points: &[Point],
    end: ArrowEnd,
    shape: super::ArrowShape,
    size: f64,
) -> Result<(), RenderError> {
    let pair = match end {
        ArrowEnd::Head => points
            .last()
            .zip(points.get(points.len().saturating_sub(2))),
        ArrowEnd::Tail => points.first().zip(points.get(1)),
    };
    let (tip, stem) =
        pair.ok_or_else(|| RenderError::Backend("DOT native PNG arrow has no direction".into()))?;
    let (tip, stem) = (*tip, *stem);
    let (dx, dy) = (stem.x - tip.x, stem.y - tip.y);
    let length = dx.hypot(dy);
    if length <= f64::EPSILON {
        return Err(RenderError::Backend(
            "DOT native PNG arrow has zero length".into(),
        ));
    }
    let (ux, uy) = (dx / length, dy / length);
    let miter = match shape {
        super::ArrowShape::Diamond => 0.5 * 6.0_f64.hypot(4.) / 4.,
        super::ArrowShape::Vee => 0.5 * 10.0_f64.hypot(4.5) / 4.5,
        _ => 0.5 * 10.0_f64.hypot(3.5) / 3.5,
    };
    if matches!(shape, super::ArrowShape::Box) {
        return draw_box_arrow(canvas, tip, (ux, uy), size);
    }
    let tip = Point {
        x: tip.x + ux * miter,
        y: tip.y + uy * miter,
    };
    let base = Point {
        x: tip.x
            + ux * if matches!(shape, super::ArrowShape::Diamond) {
                12.
            } else {
                10.
            } * size,
        y: tip.y
            + uy * if matches!(shape, super::ArrowShape::Diamond) {
                12.
            } else {
                10.
            } * size,
    };
    let wing = Point {
        x: -uy
            * match shape {
                super::ArrowShape::Diamond => 4.,
                super::ArrowShape::Vee => 4.5,
                _ => 3.5,
            }
            * size,
        y: ux
            * match shape {
                super::ArrowShape::Diamond => 4.,
                super::ArrowShape::Vee => 4.5,
                _ => 3.5,
            }
            * size,
    };
    if matches!(shape, super::ArrowShape::Vee) {
        let notch = Point {
            x: tip.x + ux * (5. * size - miter),
            y: tip.y + uy * (5. * size - miter),
        };
        let polygon = [
            tip,
            Point {
                x: base.x + wing.x,
                y: base.y + wing.y,
            },
            notch,
            base,
            base,
            base,
            notch,
            Point {
                x: base.x - wing.x,
                y: base.y - wing.y,
            },
        ];
        return fill_arrow_polygon(canvas, &polygon);
    }
    if matches!(shape, super::ArrowShape::Diamond) {
        let middle = Point {
            x: (tip.x + base.x) / 2.,
            y: (tip.y + base.y) / 2.,
        };
        let polygon = [
            base,
            Point {
                x: middle.x - wing.x,
                y: middle.y - wing.y,
            },
            tip,
            Point {
                x: middle.x + wing.x,
                y: middle.y + wing.y,
            },
        ];
        fill_arrow_polygon(canvas, &polygon)?;
        return draw_directed_arrow_outline(canvas, &polygon);
    }
    let corners = [
        canvas.device(Point {
            x: base.x + wing.x,
            y: base.y + wing.y,
        })?,
        canvas.device(tip)?,
        canvas.device(Point {
            x: base.x - wing.x,
            y: base.y - wing.y,
        })?,
    ];
    canvas.filled_polygon(&corners)
}

fn fill_arrow_polygon(canvas: &mut Canvas, points: &[Point]) -> Result<(), RenderError> {
    let corners = points
        .iter()
        .map(|point| canvas.device(*point))
        .collect::<Result<Vec<_>, _>>()?;
    canvas.filled_polygon(&corners)
}

fn draw_directed_arrow_outline(canvas: &mut Canvas, points: &[Point]) -> Result<(), RenderError> {
    let corners = points
        .iter()
        .map(|point| canvas.device(*point))
        .collect::<Result<Vec<_>, _>>()?;
    for (start, end) in corners
        .iter()
        .copied()
        .zip(corners.iter().copied().cycle().skip(1))
        .take(corners.len())
    {
        canvas.directed_line(start, end)?;
    }
    Ok(())
}

fn draw_box_arrow(
    canvas: &mut Canvas,
    tip: Point,
    axis: (f64, f64),
    size: f64,
) -> Result<(), RenderError> {
    let (ux, uy) = axis;
    let near = Point {
        x: tip.x + ux * 0.5,
        y: tip.y + uy * 0.5,
    };
    let far = Point {
        x: near.x + ux * 8. * size,
        y: near.y + uy * 8. * size,
    };
    let stem = Point {
        x: near.x + ux * 10. * size,
        y: near.y + uy * 10. * size,
    };
    let wing = Point {
        x: -uy * 4. * size,
        y: ux * 4. * size,
    };
    let polygon = [
        Point {
            x: near.x - wing.x,
            y: near.y - wing.y,
        },
        Point {
            x: near.x + wing.x,
            y: near.y + wing.y,
        },
        Point {
            x: far.x + wing.x,
            y: far.y + wing.y,
        },
        Point {
            x: far.x - wing.x,
            y: far.y - wing.y,
        },
    ];
    fill_arrow_polygon(canvas, &polygon)?;
    canvas.line(canvas.device(far)?, canvas.device(stem)?)
}

fn draw_edge(
    canvas: &mut Canvas,
    edge: &Edge,
    index: usize,
    model: &Model,
    graph: &LayoutGraph,
) -> Result<(), RenderError> {
    let placed = graph
        .edge(&edge.from, &edge.to, Some(&format!("e{index}")))
        .ok_or_else(|| {
            RenderError::Backend(format!("DOT native PNG edge {index} layout is missing"))
        })?;
    let color = edge
        .attrs
        .get("color")
        .map_or(Some([0; 3]), |value| rgb(value))
        .ok_or_else(|| RenderError::UnsupportedFeature("DOT native PNG edge color".into()))?;
    canvas.set_paint(color, color);
    let arrows = super::arrow_spec(edge, model.kind)?;
    draw_path(canvas, &placed.points, arrows)?;
    if arrows.tail {
        draw_arrow(
            canvas,
            &placed.points,
            ArrowEnd::Tail,
            arrows.tail_shape,
            arrows.size,
        )?;
    }
    if arrows.head {
        draw_arrow(
            canvas,
            &placed.points,
            ArrowEnd::Head,
            arrows.head_shape,
            arrows.size,
        )?;
    }
    Ok(())
}

pub(super) fn render(
    model: &Model,
    graph: &LayoutGraph,
    work: &mut Work,
) -> Result<Option<Vec<u8>>, RenderError> {
    if !is_supported(model, graph) {
        return Ok(None);
    }
    match render_scene(model, graph, work) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(RenderError::UnsupportedFeature(_)) => Ok(None),
        Err(error) => Err(error),
    }
}

fn render_scene(
    model: &Model,
    graph: &LayoutGraph,
    work: &mut Work,
) -> Result<Vec<u8>, RenderError> {
    let mut canvas = Canvas::new(graph, work)?;
    for node in &model.nodes {
        if model.synthetic_cluster_ids.contains(&node.id) {
            continue;
        }
        canvas.work.charge(1)?;
        draw_node(&mut canvas, node, graph)?;
        for (index, edge) in model
            .edges
            .iter()
            .enumerate()
            .filter(|(_, edge)| edge.to == node.id)
        {
            canvas.work.charge(1)?;
            draw_edge(&mut canvas, edge, index, model, graph)?;
        }
    }
    let bytes = canvas.png()?;
    if bytes.len() > canvas.work.limits.svg_bytes {
        return Err(RenderError::Backend(format!(
            "DOT native PNG has {} bytes, exceeding output limit {}",
            bytes.len(),
            canvas.work.limits.svg_bytes
        )));
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Language, RenderOptions};

    #[test]
    fn should_match_graphviz_161_chain_native_gd_pixels() {
        // given
        let source = include_str!("../../../tools/dot-parity/fixtures/chain.dot");
        let options = RenderOptions {
            language: Language::Dot,
            ..RenderOptions::default()
        };
        let (model, graph, mut work, _) = super::super::prepare(source, &options).unwrap();
        let oracle = include_bytes!("goldens/chain-gd.png");

        // when
        let candidate = render(&model, &graph, &mut work).unwrap().unwrap();

        // then
        let oracle = decode(oracle);
        let candidate = decode(&candidate);
        assert_eq!(candidate.0, oracle.0);
        let changed = candidate
            .1
            .as_chunks::<3>()
            .0
            .iter()
            .zip(oracle.1.as_chunks::<3>().0.iter())
            .filter(|(left, right)| left != right)
            .count();
        assert_eq!(changed, 0, "native GD changed {changed} pixels");
    }

    #[test]
    fn should_match_graphviz_161_two_node_native_gd_pixels() {
        // given
        let source = include_str!("goldens/two-node-chain.dot");
        let options = RenderOptions {
            language: Language::Dot,
            ..RenderOptions::default()
        };
        let (model, graph, mut work, _) = super::super::prepare(source, &options).unwrap();
        let oracle = include_bytes!("goldens/two-node-chain-gd.png");

        // when
        let candidate = render(&model, &graph, &mut work).unwrap().unwrap();

        // then
        let oracle = decode(oracle);
        let candidate = decode(&candidate);
        assert_eq!(candidate.0, oracle.0);
        assert_eq!(candidate.1, oracle.1);
    }

    #[test]
    fn should_match_graphviz_161_cycle_native_gd_pixels() {
        compare_native_gd_pixels(
            include_str!("../../../tools/dot-parity/fixtures/cycle.dot"),
            include_bytes!("goldens/cycle-gd.png"),
        );
    }

    #[test]
    fn should_match_graphviz_161_diamond_native_gd_pixels() {
        compare_native_gd_pixels(
            include_str!("../../../tools/dot-parity/fixtures/diamond.dot"),
            include_bytes!("goldens/diamond-gd.png"),
        );
    }

    #[test]
    fn should_match_graphviz_161_self_loop_native_gd_pixels() {
        compare_native_gd_pixels(
            include_str!("../../../tools/dot-parity/fixtures/self.dot"),
            include_bytes!("goldens/self-gd.png"),
        );
    }

    fn compare_native_gd_pixels(source: &str, oracle: &[u8]) {
        let options = RenderOptions {
            language: Language::Dot,
            ..RenderOptions::default()
        };
        let candidate = crate::render_dot_png_native(source, &options)
            .unwrap()
            .unwrap();
        let ((width, height), candidate) = decode(&candidate);
        let (oracle_size, oracle) = decode(oracle);
        assert_eq!((width, height), oracle_size);
        let differences: Vec<_> = candidate
            .as_chunks::<3>()
            .0
            .iter()
            .zip(oracle.as_chunks::<3>().0.iter())
            .enumerate()
            .filter_map(|(index, (actual, expected))| {
                (actual != expected).then_some((
                    index as u32 % width,
                    index as u32 / width,
                    *actual,
                    *expected,
                ))
            })
            .collect();
        assert!(
            differences.is_empty(),
            "{} changed pixels; first {:?}",
            differences.len(),
            differences.iter().take(12).collect::<Vec<_>>()
        );
    }

    #[test]
    fn should_match_graphviz_161_colored_box_chain_native_gd_pixels() {
        // given
        let source = include_str!("goldens/colored-chain.dot");
        let oracle = include_bytes!("goldens/colored-chain-gd.png");

        // when and then
        compare_native_gd_pixels(source, oracle);
    }

    #[test]
    fn should_match_graphviz_161_hex_rgb_box_chain_native_gd_pixels() {
        // given
        let source = include_str!("goldens/colored-chain-hex.dot");
        let oracle = include_bytes!("goldens/colored-chain-hex-gd.png");

        // when and then
        compare_native_gd_pixels(source, oracle);
    }

    #[test]
    fn should_match_graphviz_161_filled_color_fallback_native_gd_pixels() {
        // given
        let source = include_str!("goldens/colored-fallback.dot");
        let oracle = include_bytes!("goldens/colored-fallback-gd.png");

        // when and then
        compare_native_gd_pixels(source, oracle);
    }

    #[test]
    fn should_match_graphviz_161_colored_edge_native_gd_pixels() {
        // given
        let source = include_str!("goldens/colored-edge.dot");
        let oracle = include_bytes!("goldens/colored-edge-gd.png");

        // when and then
        compare_native_gd_pixels(source, oracle);
    }

    #[test]
    fn should_match_graphviz_161_ellipse_chain_native_gd_pixels() {
        // given
        let source = include_str!("goldens/ellipse-chain.dot");
        let oracle = include_bytes!("goldens/ellipse-chain-gd.png");

        // when and then
        compare_native_gd_pixels(source, oracle);
    }

    #[test]
    fn should_match_graphviz_161_filled_ellipse_chain_native_gd_pixels() {
        // given
        let source = include_str!("goldens/ellipse-filled-chain.dot");
        let oracle = include_bytes!("goldens/ellipse-filled-chain-gd.png");

        // when and then
        compare_native_gd_pixels(source, oracle);
    }

    #[test]
    fn should_match_graphviz_161_mixed_ellipse_box_chain_native_gd_pixels() {
        // given
        let source = include_str!("goldens/ellipse-mixed-chain.dot");
        let oracle = include_bytes!("goldens/ellipse-mixed-chain-gd.png");

        // when and then
        compare_native_gd_pixels(source, oracle);
    }

    #[test]
    fn should_match_graphviz_161_ellipse_cycle_native_gd_pixels() {
        // given
        let source = include_str!("goldens/ellipse-cycle.dot");
        let oracle = include_bytes!("goldens/ellipse-cycle-gd.png");

        // when and then
        compare_native_gd_pixels(source, oracle);
    }

    #[test]
    fn should_match_graphviz_161_ellipse_diamond_native_gd_pixels() {
        // given
        let source = include_str!("goldens/ellipse-diamond.dot");
        let oracle = include_bytes!("goldens/ellipse-diamond-gd.png");

        // when and then
        compare_native_gd_pixels(source, oracle);
    }

    #[test]
    fn should_match_graphviz_161_ellipse_self_loop_native_gd_pixels() {
        // given
        let source = include_str!("goldens/ellipse-self.dot");
        let oracle = include_bytes!("goldens/ellipse-self-gd.png");

        // when and then
        compare_native_gd_pixels(source, oracle);
    }

    #[test]
    fn should_reject_native_canvas_before_exceeding_work_budget() {
        // given
        let source = include_str!("goldens/two-node-chain.dot");
        let options = RenderOptions {
            language: Language::Dot,
            ..RenderOptions::default()
        };
        let (model, graph, mut work, _) = super::super::prepare(source, &options).unwrap();
        work.limits.layout_work = work.used + 10;

        // when
        let result = render(&model, &graph, &mut work);

        // then
        assert!(
            matches!(result, Err(RenderError::Backend(message)) if message.contains("work count"))
        );
    }

    #[test]
    fn should_stop_ellipse_spans_at_work_budget() {
        // given
        let source = include_str!("goldens/ellipse-filled-chain.dot");
        let options = RenderOptions {
            language: Language::Dot,
            ..RenderOptions::default()
        };
        let (model, graph, mut work, _) = super::super::prepare(source, &options).unwrap();
        let mut canvas = Canvas::new(&graph, &mut work).unwrap();
        canvas.work.limits.layout_work = canvas.work.used + 10;

        // when
        let result = draw_node(&mut canvas, &model.nodes[0], &graph);

        // then
        assert!(
            matches!(result, Err(RenderError::Backend(message)) if message.contains("work count"))
        );
    }

    #[test]
    fn should_cancel_ellipse_before_raster_work() {
        // given
        let source = include_str!("goldens/ellipse-filled-chain.dot");
        let options = RenderOptions {
            language: Language::Dot,
            ..RenderOptions::default()
        };
        let (model, graph, mut work, _) = super::super::prepare(source, &options).unwrap();
        let mut canvas = Canvas::new(&graph, &mut work).unwrap();
        options.control.cancel();

        // when
        let result = draw_node(&mut canvas, &model.nodes[0], &graph);

        // then
        assert!(matches!(result, Err(RenderError::Cancelled(_))));
    }

    #[test]
    fn should_reject_ellipse_axis_beyond_native_bound() {
        // given
        let source = include_str!("goldens/ellipse-chain.dot");
        let options = RenderOptions {
            language: Language::Dot,
            ..RenderOptions::default()
        };
        let (_, graph, mut work, _) = super::super::prepare(source, &options).unwrap();
        let mut canvas = Canvas::new(&graph, &mut work).unwrap();

        // when
        let result = canvas.ellipse((50, 50), (MAX_AXIS as i32 + 1, 53), false);

        // then
        assert!(
            matches!(result, Err(RenderError::UnsupportedFeature(message)) if message.contains("axis bounds"))
        );
    }

    #[test]
    fn should_fall_back_for_visible_default_ellipse_label() {
        // given
        let source = include_str!("goldens/default-ellipse-red-chain.dot")
            .replace("b [label=\"\"]", "b [label=\"B\"]");
        let options = RenderOptions {
            language: Language::Dot,
            ..RenderOptions::default()
        };
        let (model, graph, mut work, _) = super::super::prepare(&source, &options).unwrap();

        // when
        let native = render(&model, &graph, &mut work).unwrap();

        // then
        assert!(native.is_none());
    }

    #[test]
    fn should_match_graphviz_161_default_red_ellipse_chain_native_gd_pixels() {
        // given
        let source = include_str!("goldens/default-ellipse-red-chain.dot");
        let oracle = include_bytes!("goldens/default-ellipse-red-chain-gd.png");

        // when and then
        compare_native_gd_pixels(source, oracle);
    }

    #[test]
    fn should_match_graphviz_161_default_black_ellipse_chain_native_gd_pixels() {
        // given
        let source = include_str!("goldens/default-ellipse-black-chain.dot");
        let oracle = include_bytes!("goldens/default-ellipse-black-chain-gd.png");

        // when and then
        compare_native_gd_pixels(source, oracle);
    }

    #[test]
    fn should_match_graphviz_161_default_single_ellipse_native_gd_pixels() {
        // given
        let source = include_str!("goldens/default-ellipse-single.dot");
        let oracle = include_bytes!("goldens/default-ellipse-single-gd.png");

        // when and then
        compare_native_gd_pixels(source, oracle);
    }

    #[test]
    fn should_match_graphviz_161_default_filled_single_ellipse_native_gd_pixels() {
        // given
        let source = include_str!("goldens/default-ellipse-filled-single.dot");
        let oracle = include_bytes!("goldens/default-ellipse-filled-single-gd.png");

        // when and then
        compare_native_gd_pixels(source, oracle);
    }

    #[test]
    fn should_keep_pinned_liberation_serif_font_identity() {
        // given and when
        let digest = Sha256::digest(OFL_FONT);

        // then
        assert_eq!(digest[..], OFL_FONT_SHA256);
        assert!(has_pinned_ofl_font());
    }

    #[test]
    fn should_match_graphviz_161_ofl_ascii_glyph_gd_pixels() {
        // given
        let cases = [
            (
                include_str!("goldens/ofl-glyph-a.dot"),
                include_bytes!("goldens/ofl-glyph-a-gd.png").as_slice(),
            ),
            (
                include_str!("goldens/ofl-glyph-b.dot"),
                include_bytes!("goldens/ofl-glyph-b-gd.png").as_slice(),
            ),
            (
                include_str!("goldens/ofl-glyph-c.dot"),
                include_bytes!("goldens/ofl-glyph-c-gd.png").as_slice(),
            ),
            (
                include_str!("goldens/ofl-glyph-capital-a.dot"),
                include_bytes!("goldens/ofl-glyph-capital-a-gd.png").as_slice(),
            ),
            (
                include_str!("goldens/ofl-glyph-zero.dot"),
                include_bytes!("goldens/ofl-glyph-zero-gd.png").as_slice(),
            ),
            (
                include_str!("goldens/ofl-glyph-g.dot"),
                include_bytes!("goldens/ofl-glyph-g-gd.png").as_slice(),
            ),
        ];

        // when and then
        for (source, oracle) in cases {
            compare_native_gd_pixels(source, oracle);
        }
    }

    #[test]
    fn should_match_graphviz_161_ofl_ascii_corpus_rgb_digests() {
        // given
        let manifest = include_str!("goldens/ofl-ascii-62-rgb-sha256.tsv");
        let options = RenderOptions {
            language: Language::Dot,
            ..RenderOptions::default()
        };
        let mut admitted = 0;

        // when and then
        for row in manifest.lines().filter(|line| !line.starts_with('#')) {
            let fields: Vec<_> = row.split_whitespace().collect();
            assert_eq!(fields.len(), 4, "invalid OFL GD manifest row {row:?}");
            let (glyph, width, height, digest) = (
                fields[0],
                fields[1].parse::<u32>().unwrap(),
                fields[2].parse::<u32>().unwrap(),
                fields[3],
            );
            let source = format!(
                "graph G {{ graph [splines=polyline]; node [shape=ellipse]; \"{glyph}\"; }}"
            );
            let candidate = crate::render_dot_png_native(&source, &options)
                .unwrap()
                .unwrap_or_else(|| panic!("{glyph} must use native GD"));
            let ((actual_width, actual_height), rgb) = decode(&candidate);
            assert_eq!((actual_width, actual_height), (width, height), "{glyph}");
            assert_eq!(format!("{:x}", Sha256::digest(&rgb)), digest, "{glyph}");
            admitted += 1;
        }
        assert_eq!(admitted, 62);
    }

    #[test]
    fn should_match_graphviz_161_ofl_undirected_visible_text_gd_pixels() {
        // given
        let source = include_str!("goldens/ofl-undirected.dot");
        let oracle = include_bytes!("goldens/ofl-undirected-gd.png");

        // when and then
        compare_native_gd_pixels(source, oracle);
    }

    #[test]
    fn should_match_graphviz_161_ofl_gd_text_span_placement_diagnostics() {
        // given
        let cases = [
            (
                include_str!("goldens/ofl-glyph-one.dot"),
                include_bytes!("goldens/ofl-glyph-one-gd.png").as_slice(),
                5.,
            ),
            (
                include_str!("goldens/ofl-glyph-capital-f.dot"),
                include_bytes!("goldens/ofl-glyph-capital-f-gd.png").as_slice(),
                7.,
            ),
            (
                include_str!("goldens/ofl-glyph-m.dot"),
                include_bytes!("goldens/ofl-glyph-m-gd.png").as_slice(),
                10.,
            ),
        ];
        let options = RenderOptions {
            language: Language::Dot,
            ..RenderOptions::default()
        };

        // when and then
        for (source, oracle, width) in cases {
            let (model, graph, mut work, _) = super::super::prepare(source, &options).unwrap();
            let font = FontRef::from_index(OFL_FONT, 0).unwrap();
            let glyph = font
                .charmap()
                .map(model.nodes[0].id.chars().next().unwrap());
            assert_eq!(gd_text_span_width(font, glyph), width);
            assert!(is_supported(&model, &graph));
            let candidate = render_scene(&model, &graph, &mut work).unwrap();
            let (size, actual) = decode(&candidate);
            let (expected_size, expected) = decode(oracle);
            assert_eq!(size, expected_size);
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn should_fall_back_for_unpinned_ofl_text_size() {
        // given
        let source = include_str!("goldens/ofl-glyph-a.dot")
            .replace("node [shape=ellipse]", "node [shape=ellipse, fontsize=12]");
        let options = RenderOptions {
            language: Language::Dot,
            ..RenderOptions::default()
        };
        let (model, graph, _, _) = super::super::prepare(&source, &options).unwrap();

        // when
        let admitted = is_supported(&model, &graph);

        // then
        assert!(!admitted);
    }

    #[test]
    fn should_stop_ofl_glyph_before_work_exceeds_budget() {
        // given
        let source = include_str!("goldens/ofl-glyph-a.dot");
        let options = RenderOptions {
            language: Language::Dot,
            ..RenderOptions::default()
        };
        let (_, graph, mut work, _) = super::super::prepare(source, &options).unwrap();
        let mut canvas = Canvas::new(&graph, &mut work).unwrap();
        canvas.work.limits.layout_work = canvas.work.used + 4095;

        // when
        let result = canvas.ofl_ascii_label(Point { x: 27., y: 18. }, "a");

        // then
        assert!(
            matches!(result, Err(RenderError::Backend(message)) if message.contains("work count"))
        );
    }

    #[test]
    fn should_cancel_ofl_glyph_before_raster_work() {
        // given
        let source = include_str!("goldens/ofl-glyph-a.dot");
        let options = RenderOptions {
            language: Language::Dot,
            ..RenderOptions::default()
        };
        let (_, graph, mut work, _) = super::super::prepare(source, &options).unwrap();
        let mut canvas = Canvas::new(&graph, &mut work).unwrap();
        options.control.cancel();

        // when
        let result = canvas.ofl_ascii_label(Point { x: 27., y: 18. }, "a");

        // then
        assert!(matches!(result, Err(RenderError::Cancelled(_))));
    }

    #[test]
    fn should_reject_off_canvas_native_points_before_drawing() {
        // given
        let source = include_str!("goldens/two-node-chain.dot");
        let options = RenderOptions {
            language: Language::Dot,
            ..RenderOptions::default()
        };
        let (_, graph, mut work, _) = super::super::prepare(source, &options).unwrap();
        let canvas = Canvas::new(&graph, &mut work).unwrap();

        // when
        let result = canvas.device(Point { x: 1e12, y: 1e12 });

        // then
        assert!(
            matches!(result, Err(RenderError::UnsupportedFeature(message)) if message.contains("outside"))
        );
    }

    #[test]
    fn should_fall_back_for_nonopaque_fillcolor() {
        // given
        let source = include_str!("goldens/colored-chain.dot")
            .replace("fillcolor=red", r##"fillcolor="#ff000080""##);
        let options = RenderOptions {
            language: Language::Dot,
            ..RenderOptions::default()
        };
        let (model, graph, mut work, _) = super::super::prepare(&source, &options).unwrap();

        // when
        let native = render(&model, &graph, &mut work).unwrap();

        // then
        assert!(native.is_none());
    }

    #[test]
    fn should_fall_back_for_non_ascii_hex_color_without_panicking() {
        // given
        let source = include_str!("goldens/colored-chain.dot")
            .replace("fillcolor=red", r##"fillcolor="#aébcd""##);
        let options = RenderOptions {
            language: Language::Dot,
            ..RenderOptions::default()
        };
        let (model, graph, mut work, _) = super::super::prepare(&source, &options).unwrap();

        // when
        let native = render(&model, &graph, &mut work).unwrap();

        // then
        assert!(native.is_none());
    }

    #[test]
    fn should_fall_back_when_native_route_leaves_canvas() {
        // given
        let source = include_str!("goldens/two-node-chain.dot");
        let options = RenderOptions {
            language: Language::Dot,
            ..RenderOptions::default()
        };
        let (model, mut graph, mut work, _) = super::super::prepare(source, &options).unwrap();
        graph.edge_mut("a", "b", Some("e0")).unwrap().points[0].x = 1e12;

        // when
        let native = render(&model, &graph, &mut work).unwrap();

        // then
        assert!(native.is_none());
    }

    #[test]
    fn should_match_graphviz_161_arrow_native_gd_pixels() {
        // given
        let cases = [
            (
                "both",
                include_str!("goldens/arrow_both.dot"),
                include_bytes!("goldens/arrow_both-gd.png").as_slice(),
            ),
            (
                "head_none",
                include_str!("goldens/arrow_head_none.dot"),
                include_bytes!("goldens/arrow_head_none-gd.png").as_slice(),
            ),
            (
                "none",
                include_str!("goldens/arrow_none.dot"),
                include_bytes!("goldens/arrow_none-gd.png").as_slice(),
            ),
            (
                "head_vee",
                include_str!("goldens/arrow_head_vee.dot"),
                include_bytes!("goldens/arrow_head_vee-gd.png").as_slice(),
            ),
            (
                "head_diamond",
                include_str!("goldens/arrow_head_diamond.dot"),
                include_bytes!("goldens/arrow_head_diamond-gd.png").as_slice(),
            ),
            (
                "head_box",
                include_str!("goldens/arrow_head_box.dot"),
                include_bytes!("goldens/arrow_head_box-gd.png").as_slice(),
            ),
            (
                "head_size2",
                include_str!("goldens/arrow_head_size2.dot"),
                include_bytes!("goldens/arrow_head_size2-gd.png").as_slice(),
            ),
            (
                "head_diamond_size2",
                include_str!("goldens/arrow_head_diamond_size2.dot"),
                include_bytes!("goldens/arrow_head_diamond_size2-gd.png").as_slice(),
            ),
            (
                "tail_vee",
                include_str!("goldens/arrow_tail_vee.dot"),
                include_bytes!("goldens/arrow_tail_vee-gd.png").as_slice(),
            ),
            (
                "tail_diamond",
                include_str!("goldens/arrow_tail_diamond.dot"),
                include_bytes!("goldens/arrow_tail_diamond-gd.png").as_slice(),
            ),
            (
                "tail_box",
                include_str!("goldens/arrow_tail_box.dot"),
                include_bytes!("goldens/arrow_tail_box-gd.png").as_slice(),
            ),
        ];
        let options = RenderOptions {
            language: Language::Dot,
            ..RenderOptions::default()
        };
        // when and then
        for (name, source, oracle) in cases {
            let (model, graph, mut work, _) = super::super::prepare(source, &options).unwrap();
            let candidate = render(&model, &graph, &mut work).unwrap().unwrap();
            let (actual_size, actual) = decode(&candidate);
            let (expected_size, expected) = decode(oracle);
            assert_eq!(actual_size, expected_size, "{name}");
            let changed = actual
                .as_chunks::<3>()
                .0
                .iter()
                .zip(expected.as_chunks::<3>().0.iter())
                .enumerate()
                .filter_map(|(index, (left, right))| {
                    (left != right).then_some((
                        index as u32 % actual_size.0,
                        index as u32 / actual_size.0,
                        *left,
                        *right,
                    ))
                })
                .collect::<Vec<_>>();
            assert!(changed.is_empty(), "{name}: {changed:?} GD pixels differ");
        }
    }

    fn decode(bytes: &[u8]) -> ((u32, u32), Vec<u8>) {
        let mut decoder = png::Decoder::new(std::io::Cursor::new(bytes));
        decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
        let mut reader = decoder.read_info().unwrap();
        let mut pixels = vec![0; reader.output_buffer_size().unwrap()];
        let info = reader.next_frame(&mut pixels).unwrap();
        pixels.truncate(info.buffer_size());
        let rgb = match info.color_type {
            png::ColorType::Rgb => pixels,
            png::ColorType::Rgba => pixels
                .as_chunks::<4>()
                .0
                .iter()
                .flat_map(|pixel| pixel[..3].iter().copied())
                .collect(),
            other => panic!("unexpected PNG golden color type {other:?}"),
        };
        ((info.width, info.height), rgb)
    }
}
