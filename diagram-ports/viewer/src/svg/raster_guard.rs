use super::{charge, checkpoint, path_paint_bounds, validate_bitmap, validate_transform};
use std::sync::atomic::AtomicBool;
use usvg::{Group, Node, Rect, Transform};

const PIXEL_WORK: usize = 64 * 1024 * 1024;

#[derive(Clone, Copy)]
struct Context {
    transform: Transform,
    surface: (u32, u32),
    depth: usize,
}

enum Operation<'a> {
    Node(&'a Node),
    Clip(&'a usvg::ClipPath),
    Mask(&'a usvg::Mask),
    Pattern(&'a usvg::Pattern),
}

struct Guard<'a> {
    pending: Vec<(Operation<'a>, Context)>,
    items: usize,
    points: usize,
    decoded: usize,
    pixels: usize,
}

// resvg 0.46 clips ordinary/filter layers to 5×target axes, but not pattern tiles.
// Budget every occurrence, including retained layers and effect scratch buffers,
// before calling resvg. This is conservative admission, not a hard host deadline.
pub(super) fn check(
    tree: &usvg::Tree,
    transform: Transform,
    size: (u32, u32),
    cancelled: &AtomicBool,
) -> Result<(), String> {
    let mut guard = Guard {
        pending: Vec::new(),
        items: 100_000,
        points: 1_000_000,
        decoded: 4_194_304,
        pixels: PIXEL_WORK,
    };
    guard.surface((f64::from(size.0), f64::from(size.1)), 1)?;
    guard.children(
        tree.root(),
        Context {
            transform,
            surface: size,
            depth: 0,
        },
    )?;
    while let Some((operation, context)) = guard.pending.pop() {
        checkpoint(cancelled)?;
        guard.visit(operation, context)?;
    }
    checkpoint(cancelled)
}

impl<'a> Guard<'a> {
    fn push(&mut self, operation: Operation<'a>, context: Context) -> Result<(), String> {
        charge(&mut self.items, 1, "raster items")?;
        if context.depth >= 128 {
            return Err("diagram SVG raster effects exceed depth limit128".into());
        }
        self.pending.push((operation, context));
        Ok(())
    }

    fn children(&mut self, group: &'a Group, context: Context) -> Result<(), String> {
        let context = Context {
            depth: context.depth + 1,
            ..context
        };
        for node in group.children() {
            self.push(Operation::Node(node), context)?;
        }
        Ok(())
    }

    fn visit(&mut self, operation: Operation<'a>, context: Context) -> Result<(), String> {
        match operation {
            Operation::Node(node) => match node {
                Node::Group(group) => self.group(group, context),
                Node::Text(text) => self.group(text.flattened(), context),
                Node::Path(path) => self.path(path, context),
                Node::Image(image) => self.image(image, context),
            },
            Operation::Clip(clip) => self.clip(clip, context),
            Operation::Mask(mask) => self.mask(mask, context),
            Operation::Pattern(pattern) => self.pattern(pattern, context),
        }
    }

    fn group(&mut self, group: &'a Group, parent: Context) -> Result<(), String> {
        let mut context = Context {
            transform: parent.transform.pre_concat(group.transform()),
            ..parent
        };
        validate_transform(context.transform)?;
        if group.should_isolate() {
            let rect = transformed(group.layer_bounding_box().to_rect(), context.transform)?;
            // resvg expands non-filter layers by two device pixels on each side.
            let fringe = if group.filters().is_empty() { 4. } else { 0. };
            context.surface = self.surface(
                (
                    f64::from(rect.width().ceil()) + fringe,
                    f64::from(rect.height().ceil()) + fringe,
                ),
                1,
            )?;
        }
        if let Some(clip) = group.clip_path() {
            self.push(Operation::Clip(clip), context)?;
        }
        if let Some(mask) = group.mask() {
            self.push(Operation::Mask(mask), context)?;
        }
        for filter in group.filters() {
            self.filter(filter, context)?;
        }
        self.children(group, context)
    }

    fn path(&mut self, path: &'a usvg::Path, context: Context) -> Result<(), String> {
        charge(
            &mut self.points,
            path.data().points().len(),
            "raster path points",
        )?;
        if let Some(rect) = path_paint_bounds(path) {
            self.paint(rect, context)?;
            for paint in path
                .fill()
                .map(|fill| fill.paint())
                .into_iter()
                .chain(path.stroke().map(|stroke| stroke.paint()))
            {
                if let usvg::Paint::Pattern(pattern) = paint {
                    self.push(Operation::Pattern(pattern), context)?;
                }
            }
        }
        Ok(())
    }

    fn image(&mut self, image: &'a usvg::Image, context: Context) -> Result<(), String> {
        if image.is_visible() {
            match image.kind() {
                usvg::ImageKind::SVG(tree) => {
                    self.scratch(context.surface, 1)?;
                    self.children(tree.root(), context)?;
                }
                _ => {
                    validate_bitmap(image.size())?;
                    charge(
                        &mut self.decoded,
                        (f64::from(image.size().width()) * f64::from(image.size().height()))
                            as usize,
                        "raster decoded bitmap pixels",
                    )?;
                    self.paint(image.bounding_box(), context)?;
                }
            }
        }
        Ok(())
    }

    fn clip(&mut self, clip: &'a usvg::ClipPath, context: Context) -> Result<(), String> {
        self.scratch(context.surface, 4)?;
        if let Some(parent) = clip.clip_path() {
            self.push(
                Operation::Clip(parent),
                Context {
                    depth: context.depth + 1,
                    ..context
                },
            )?;
        }
        let context = Context {
            transform: context.transform.pre_concat(clip.transform()),
            ..context
        };
        validate_transform(context.transform)?;
        self.children(clip.root(), context)
    }

    fn mask(&mut self, mask: &'a usvg::Mask, context: Context) -> Result<(), String> {
        self.scratch(context.surface, 4)?;
        if let Some(parent) = mask.mask() {
            self.push(
                Operation::Mask(parent),
                Context {
                    depth: context.depth + 1,
                    ..context
                },
            )?;
        }
        self.children(mask.root(), context)
    }

    fn pattern(&mut self, pattern: &'a usvg::Pattern, context: Context) -> Result<(), String> {
        let transform = context.transform.pre_concat(pattern.transform());
        validate_transform(transform)?;
        let (sx, sy) = transform.get_scale();
        let surface = self.surface(
            (
                f64::from(pattern.rect().width()) * f64::from(sx),
                f64::from(pattern.rect().height()) * f64::from(sy),
            ),
            1,
        )?;
        self.children(
            pattern.root(),
            Context {
                transform: Transform::from_scale(sx, sy),
                surface,
                ..context
            },
        )
    }

    fn filter(&mut self, filter: &'a usvg::filter::Filter, context: Context) -> Result<(), String> {
        use usvg::filter::Kind;
        for primitive in filter.primitives() {
            charge(&mut self.items, 1, "filter primitives")?;
            self.scratch(context.surface, 16)?;
            match primitive.kind() {
                Kind::Blend(_)
                | Kind::ColorMatrix(_)
                | Kind::ComponentTransfer(_)
                | Kind::Composite(_)
                | Kind::DropShadow(_)
                | Kind::Flood(_)
                | Kind::GaussianBlur(_)
                | Kind::Offset(_) => {}
                Kind::Merge(merge) => {
                    self.scratch(context.surface, merge.inputs().len())?;
                }
                other => {
                    // feImage replaces the parent linear transform with scale-only
                    // in resvg; ordinary subtree accounting cannot bound its tiles.
                    return Err(format!(
                        "diagram SVG filter {} has unsupported bounded raster work",
                        filter_name(other)
                    ));
                }
            }
        }
        Ok(())
    }

    fn paint(&mut self, rect: Rect, context: Context) -> Result<(), String> {
        let rect = transformed(rect, context.transform)?;
        // Clipping can only reduce work; count the whole transformed rectangle plus AA fringe.
        let area = (f64::from(rect.width()).ceil() + 2.) * (f64::from(rect.height()).ceil() + 2.);
        self.pixels(area)
    }

    fn scratch(&mut self, size: (u32, u32), copies: usize) -> Result<(), String> {
        self.pixels(f64::from(size.0) * f64::from(size.1) * copies as f64)
    }

    fn surface(&mut self, size: (f64, f64), copies: usize) -> Result<(u32, u32), String> {
        let size = (size.0.ceil().max(1.), size.1.ceil().max(1.));
        if size.0.is_finite()
            && size.1.is_finite()
            && size.0 <= 4096.
            && size.1 <= 4096.
            && size.0 * size.1 <= 4_194_304.
        {
            self.pixels(size.0 * size.1 * copies as f64)?;
            return Ok((size.0 as u32, size.1 as u32));
        }
        Err(format!(
            "diagram SVG offscreen surface {size:?} exceeds 4096-axis/4194304-pixel bounds"
        ))
    }

    fn pixels(&mut self, pixels: f64) -> Result<(), String> {
        if pixels.is_finite() && pixels >= 0. && pixels <= self.pixels as f64 {
            self.pixels -= pixels.ceil() as usize;
            return Ok(());
        }
        Err(format!(
            "diagram SVG paint work {pixels} exceeds remaining {} of {PIXEL_WORK} pixel operations",
            self.pixels
        ))
    }
}

fn transformed(rect: Rect, transform: Transform) -> Result<Rect, String> {
    rect.transform(transform)
        .filter(|rect| {
            [rect.x(), rect.y(), rect.width(), rect.height()]
                .iter()
                .all(|value| value.is_finite())
        })
        .ok_or_else(|| format!("invalid diagram raster bounds {rect:?} under {transform:?}"))
}

fn filter_name(kind: &usvg::filter::Kind) -> &'static str {
    use usvg::filter::Kind;
    match kind {
        Kind::Image(_) => "feImage",
        Kind::ConvolveMatrix(_) => "feConvolveMatrix",
        Kind::DiffuseLighting(_) => "feDiffuseLighting",
        Kind::DisplacementMap(_) => "feDisplacementMap",
        Kind::Morphology(_) => "feMorphology",
        Kind::SpecularLighting(_) => "feSpecularLighting",
        Kind::Tile(_) => "feTile",
        Kind::Turbulence(_) => "feTurbulence",
        _ => "unknown",
    }
}
