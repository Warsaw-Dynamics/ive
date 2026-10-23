use gpui::RenderImage;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use usvg::{Rect, Transform};

mod raster_guard;

pub(crate) struct DisplaySvg {
    tree: usvg::Tree,
    canvas: usvg::Rect,
    containment: Containment,
}

enum Containment {
    Original,
    Expanded,
    Preserved(&'static str),
}

#[derive(Clone, Copy)]
struct Controls {
    items: usize,
    depth: usize,
    points: usize,
    bitmap_pixels: usize,
}

impl Default for Controls {
    fn default() -> Self {
        Self {
            items: 100_000,
            depth: 128,
            points: 1_000_000,
            bitmap_pixels: 4_194_304,
        }
    }
}

impl DisplaySvg {
    pub(crate) fn parse(source: &str, cancelled: &AtomicBool) -> Result<Self, String> {
        Self::parse_controlled(source, cancelled, Controls::default())
    }

    fn parse_controlled(
        source: &str,
        cancelled: &AtomicBool,
        controls: Controls,
    ) -> Result<Self, String> {
        preflight(source, cancelled, controls)?;
        let options = crate::svg_fonts::options();
        checkpoint(cancelled)?;
        let tree = usvg::Tree::from_str(source, &options)
            .map_err(|error| format!("cannot parse diagram SVG for display: {error}"))?;
        checkpoint(cancelled)?;
        let original = Rect::from_xywh(0., 0., tree.size().width(), tree.size().height())
            .ok_or("diagram SVG has an invalid viewport")?;
        validate_rect(original)?;
        let (canvas, containment) = inspect(&tree, cancelled, controls)?;
        Ok(Self {
            tree,
            canvas,
            containment,
        })
    }

    pub(crate) fn dimensions(&self) -> (f32, f32) {
        (self.canvas.width(), self.canvas.height())
    }

    pub(crate) fn diagnostic(&self) -> Option<String> {
        match self.containment {
            Containment::Original => None,
            Containment::Expanded => Some(format!(
                "native SVG paint containment: original {}×{}, display {}×{} at ({}, {})",
                self.tree.size().width(),
                self.tree.size().height(),
                self.canvas.width(),
                self.canvas.height(),
                self.canvas.left(),
                self.canvas.top(),
            )),
            Containment::Preserved(reason) => Some(format!(
                "native SVG paint containment skipped: {reason}; original viewport preserved"
            )),
        }
    }

    pub(crate) fn render(
        &self,
        size: (u32, u32),
        cancelled: &AtomicBool,
    ) -> Result<Arc<RenderImage>, String> {
        checkpoint(cancelled)?;
        if size.0 == 0
            || size.1 == 0
            || size.0 > 4096
            || size.1 > 4096
            || u64::from(size.0) * u64::from(size.1) > 4_194_304
        {
            return Err(format!(
                "diagram raster size {size:?} exceeds 4096-axis/4194304-pixel bounds or is empty"
            ));
        }
        let transform = Transform::from_scale(
            size.0 as f32 / self.canvas.width(),
            size.1 as f32 / self.canvas.height(),
        )
        .pre_translate(-self.canvas.left(), -self.canvas.top());
        raster_guard::check(&self.tree, transform, size, cancelled)?;
        let mut pixmap = resvg::tiny_skia::Pixmap::new(size.0, size.1)
            .ok_or_else(|| format!("cannot allocate diagram raster {size:?}"))?;
        resvg::render(&self.tree, transform, &mut pixmap.as_mut());
        checkpoint(cancelled)?;
        let mut buffer = image::RgbaImage::from_raw(size.0, size.1, pixmap.take())
            .ok_or("invalid diagram raster buffer")?;
        for row in buffer.chunks_exact_mut(size.0 as usize * 4) {
            checkpoint(cancelled)?;
            for pixel in row.as_chunks_mut::<4>().0 {
                gpui::swap_rgba_pa_to_bgra(pixel);
            }
        }
        Ok(Arc::new(RenderImage::new(vec![image::Frame::new(buffer)])))
    }
}

fn checkpoint(cancelled: &AtomicBool) -> Result<(), String> {
    if cancelled.load(Ordering::Acquire) {
        return Err("diagram SVG display rendering cancelled".into());
    }
    Ok(())
}

fn preflight(source: &str, cancelled: &AtomicBool, controls: Controls) -> Result<(), String> {
    checkpoint(cancelled)?;
    if source.len() > 16 * 1024 * 1024
        || controls.items == 0
        || controls.depth == 0
        || controls.points == 0
    {
        return Err(format!(
            "diagram SVG input length {} or display traversal controls are out of bounds",
            source.len()
        ));
    }
    let document = roxmltree::Document::parse_with_options(
        source,
        roxmltree::ParsingOptions {
            nodes_limit: controls.items.min(100_000) as u32,
            ..Default::default()
        },
    )
    .map_err(|error| format!("cannot inspect diagram SVG XML: {error}"))?;
    let mut stack = vec![document.root().children()];
    while let Some(children) = stack.last_mut() {
        checkpoint(cancelled)?;
        let Some(node) = children.next() else {
            stack.pop();
            continue;
        };
        if stack.len() >= controls.depth.min(128) {
            return Err(format!(
                "diagram SVG XML depth exceeds display limit {}",
                controls.depth
            ));
        }
        if let Some(value) = node.attribute("transform") {
            let transform = value
                .parse::<svgtypes::Transform>()
                .map_err(|error| format!("invalid diagram SVG transform {value:?}: {error}"))?;
            validate_transform(Transform::from_row(
                transform.a as f32,
                transform.b as f32,
                transform.c as f32,
                transform.d as f32,
                transform.e as f32,
                transform.f as f32,
            ))?;
        }
        stack.push(node.children());
    }
    Ok(())
}

fn validate_transform(transform: Transform) -> Result<(), String> {
    let values = [
        transform.sx,
        transform.ky,
        transform.kx,
        transform.sy,
        transform.tx,
        transform.ty,
    ];
    let determinant = f64::from(transform.sx) * f64::from(transform.sy)
        - f64::from(transform.kx) * f64::from(transform.ky);
    if values
        .iter()
        .all(|value| value.is_finite() && value.abs() <= 1_000_000.)
        && determinant != 0.
        && transform.invert().is_some()
    {
        return Ok(());
    }
    Err(format!(
        "diagram SVG transform {transform:?} must be finite, invertible and bounded by 1000000"
    ))
}

fn validate_rect(rect: Rect) -> Result<(), String> {
    if [
        rect.left(),
        rect.top(),
        rect.right(),
        rect.bottom(),
        rect.width(),
        rect.height(),
    ]
    .iter()
    .all(|value| value.is_finite() && value.abs() <= 1_000_000.)
    {
        return Ok(());
    }
    Err(format!(
        "diagram SVG paint bounds {rect:?} exceed finite 1000000-coordinate/axis limits"
    ))
}

#[derive(Clone, Copy)]
enum Content {
    Shape,
    Text,
}

struct Frame<'a> {
    children: std::slice::Iter<'a, usvg::Node>,
    transform: Transform,
    content: Content,
}

struct Bounds {
    original: Rect,
    text: Rect,
    shape_overflow: [bool; 4],
}

fn inspect(
    tree: &usvg::Tree,
    cancelled: &AtomicBool,
    controls: Controls,
) -> Result<(Rect, Containment), String> {
    let original = Rect::from_xywh(0., 0., tree.size().width(), tree.size().height())
        .ok_or("invalid diagram viewport")?;
    let mut bounds = Bounds {
        original,
        text: original,
        shape_overflow: [false; 4],
    };
    let mut remaining = controls;
    let mut effects = has_effects(tree);
    let mut stack = vec![Frame {
        children: tree.root().children().iter(),
        transform: Transform::identity(),
        content: Content::Shape,
    }];
    while let Some(frame) = stack.last_mut() {
        checkpoint(cancelled)?;
        let Some(node) = frame.children.next() else {
            stack.pop();
            continue;
        };
        charge(&mut remaining.items, 1, "items")?;
        let (transform, content) = (frame.transform, frame.content);
        match node {
            usvg::Node::Group(group) => {
                push_group(&mut stack, group, (transform, content), controls.depth)?
            }
            usvg::Node::Text(text) => push_group(
                &mut stack,
                text.flattened(),
                (transform, Content::Text),
                controls.depth,
            )?,
            usvg::Node::Path(path) => {
                charge(
                    &mut remaining.points,
                    path.data().points().len(),
                    "path points",
                )?;
                if let Some(rect) = path_paint_bounds(path) {
                    bounds.include(rect, transform, content)?;
                }
            }
            usvg::Node::Image(image) if image.is_visible() => match image.kind() {
                usvg::ImageKind::SVG(subtree) => {
                    effects |= has_effects(subtree);
                    if stack.len() >= controls.depth.min(128) {
                        return Err("diagram SVG image exceeds display depth limit".into());
                    }
                    stack.push(Frame {
                        children: subtree.root().children().iter(),
                        transform,
                        content,
                    });
                }
                _ => {
                    validate_bitmap(image.size())?;
                    charge(
                        &mut remaining.bitmap_pixels,
                        (f64::from(image.size().width()) * f64::from(image.size().height()))
                            as usize,
                        "decoded bitmap pixels",
                    )?;
                    bounds.include(image.bounding_box(), transform, content)?;
                }
            },
            _ => {}
        }
    }
    // Effects can change painted extents or intentional clipping; keep their original renderer semantics.
    if effects {
        return Ok((
            original,
            Containment::Preserved("explicit SVG effect/clip/pattern"),
        ));
    }
    bounds.finish()
}

fn has_effects(tree: &usvg::Tree) -> bool {
    [
        tree.clip_paths().len(),
        tree.masks().len(),
        tree.filters().len(),
        tree.patterns().len(),
    ]
    .iter()
    .any(|count| *count > 0)
}

fn push_group<'a>(
    stack: &mut Vec<Frame<'a>>,
    group: &'a usvg::Group,
    parent: (Transform, Content),
    depth: usize,
) -> Result<(), String> {
    let transform = parent.0.pre_concat(group.transform());
    validate_transform(transform)?;
    if stack.len() >= depth.min(128) {
        return Err(format!(
            "diagram SVG parsed tree exceeds display depth limit {depth}"
        ));
    }
    if group.opacity().get() > 0. {
        stack.push(Frame {
            children: group.children().iter(),
            transform,
            content: parent.1,
        });
    }
    Ok(())
}

fn charge(remaining: &mut usize, amount: usize, kind: &str) -> Result<(), String> {
    *remaining = remaining
        .checked_sub(amount)
        .ok_or_else(|| format!("diagram SVG display traversal exceeds {kind} budget"))?;
    Ok(())
}

fn validate_bitmap(size: usvg::Size) -> Result<(), String> {
    if size.width() > 4096.
        || size.height() > 4096.
        || f64::from(size.width()) * f64::from(size.height()) > 4_194_304.
    {
        return Err(format!(
            "diagram SVG bitmap dimensions {size:?} exceed 4096-axis/4194304-pixel decode bounds"
        ));
    }
    Ok(())
}

fn paint_visible(paint: &usvg::Paint) -> bool {
    match paint {
        usvg::Paint::Color(_) | usvg::Paint::Pattern(_) => true,
        usvg::Paint::LinearGradient(gradient) => gradient
            .stops()
            .iter()
            .any(|stop| stop.opacity().get() > 0.),
        usvg::Paint::RadialGradient(gradient) => gradient
            .stops()
            .iter()
            .any(|stop| stop.opacity().get() > 0.),
    }
}

fn path_paint_bounds(path: &usvg::Path) -> Option<Rect> {
    if path.is_visible() {
        if path
            .stroke()
            .is_some_and(|stroke| stroke.opacity().get() > 0. && paint_visible(stroke.paint()))
        {
            return Some(path.stroke_bounding_box());
        }
        if path
            .fill()
            .is_some_and(|fill| fill.opacity().get() > 0. && paint_visible(fill.paint()))
        {
            return Some(path.bounding_box());
        }
    }
    None
}

impl Bounds {
    fn include(
        &mut self,
        rect: Rect,
        transform: Transform,
        content: Content,
    ) -> Result<(), String> {
        validate_rect(rect)?;
        let rect = rect.transform(transform).ok_or_else(|| {
            format!("cannot transform diagram SVG paint bounds {rect:?} by {transform:?}")
        })?;
        validate_rect(rect)?;
        match content {
            Content::Text => {
                self.text = Rect::from_ltrb(
                    self.text.left().min(rect.left()),
                    self.text.top().min(rect.top()),
                    self.text.right().max(rect.right()),
                    self.text.bottom().max(rect.bottom()),
                )
                .ok_or("invalid diagram text bounds union")?
            }
            Content::Shape => {
                let overflow = [
                    rect.left() < self.original.left(),
                    rect.top() < self.original.top(),
                    rect.right() > self.original.right(),
                    rect.bottom() > self.original.bottom(),
                ];
                for (current, next) in self.shape_overflow.iter_mut().zip(overflow) {
                    *current |= next;
                }
            }
        }
        Ok(())
    }

    fn finish(self) -> Result<(Rect, Containment), String> {
        if self.text == self.original {
            return Ok((self.original, Containment::Original));
        }
        let canvas = Rect::from_ltrb(
            if self.text.left() < self.original.left() {
                self.text.left().floor()
            } else {
                self.original.left()
            },
            if self.text.top() < self.original.top() {
                self.text.top().floor()
            } else {
                self.original.top()
            },
            if self.text.right() > self.original.right() {
                self.text.right().ceil()
            } else {
                self.original.right()
            },
            if self.text.bottom() > self.original.bottom() {
                self.text.bottom().ceil()
            } else {
                self.original.bottom()
            },
        )
        .ok_or("invalid expanded diagram canvas")?;
        validate_rect(canvas)?;
        let expansion = [
            canvas.left() < self.original.left(),
            canvas.top() < self.original.top(),
            canvas.right() > self.original.right(),
            canvas.bottom() > self.original.bottom(),
        ];
        if self
            .shape_overflow
            .iter()
            .zip(expansion)
            .any(|(overflow, expanded)| *overflow && expanded)
        {
            return Ok((
                self.original,
                Containment::Preserved("expansion could reveal root-clipped non-text paint"),
            ));
        }
        Ok((canvas, Containment::Expanded))
    }
}

#[cfg(test)]
mod tests;

#[cfg(all(test, target_os = "linux"))]
mod creole_spacing_tests;
