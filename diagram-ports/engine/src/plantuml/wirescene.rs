//! Native bounded wire cursor placement and straight one-axis routes, from MIT WBlock/WLink*.
//! Copyright 2009–2024 Arnaud Roques. Measured native gutters intentionally differ.
use super::{
    budget::{Budget, bound},
    scene::{Dash, Decoration, Element, Paint, Point, Rect, Scene, Shape},
    text::{Context, Label},
    wire::{self, Anchor, Direction, Orientation, Placement, Size, Wire, WireColor},
};
use crate::RenderError;
struct Measures {
    blocks: Vec<Label>,
    links: Vec<Label>,
    h_gap: f64,
    v_gap: f64,
    h_steps: Vec<f64>,
    v_steps: Vec<f64>,
}
struct Geometry {
    rects: Vec<Rect>,
    width: f64,
    height: f64,
}
pub(super) fn layout(model: &Wire, context: &mut Context<'_>) -> Result<Scene, RenderError> {
    let count = model
        .blocks
        .len()
        .checked_add(model.links.len())
        .ok_or_else(wire::overflow)?;
    bound(
        "wire layout retained bytes",
        count.saturating_mul(2048),
        context
            .budget
            .limits
            .source_bytes
            .saturating_mul(256)
            .saturating_add(context.budget.limits.model_items.saturating_mul(2048)),
    )?;
    let pairs = model
        .blocks
        .len()
        .checked_mul(model.links.len())
        .ok_or_else(wire::overflow)?;
    context.budget.charge(pairs.saturating_mul(4))?;
    let measured = measure(model, context)?;
    let geometry = place(model, &measured, context)?;
    let mut scene = Scene::new();
    draw_blocks((&mut scene, model), (&measured, &geometry), context)?;
    for id in 0..model.links.len() {
        context.budget.charge(1)?;
        draw_link((&mut scene, model), (&measured, &geometry, id), context)?;
    }
    scene.width = scene.width.max(geometry.width + 48.);
    scene.height = scene.height.max(geometry.height + 48.);
    Ok(scene)
}
fn store<T>(out: &mut Vec<T>, value: T, budget: &mut Budget) -> Result<(), RenderError> {
    super::parser::context::push(out, value, budget)
        .map_err(|failure| failure.into_document(&super::origin::EMPTY, budget))
}
fn measure(model: &Wire, context: &mut Context<'_>) -> Result<Measures, RenderError> {
    let mut blocks = Vec::new();
    let mut links = Vec::new();
    let mut h_gap = 20_f64;
    let mut v_gap = 20_f64;
    let mut h_steps = Vec::new();
    let mut v_steps = Vec::new();
    for block in &model.blocks {
        context.budget.charge(block.local.len())?;
        let label = context.label(&block.local.replace('_', " "))?;
        store(&mut blocks, label, context.budget)?;
        store(&mut h_steps, 15_f64, context.budget)?;
        store(&mut v_steps, 15_f64, context.budget)?;
    }
    for link in &model.links {
        let label = match link.color {
            WireColor::Default => context.label(&link.label)?,
            WireColor::Rgb(color) => context.colored_label(&link.label, color)?,
        };
        if !link.label.is_empty() {
            h_gap = h_gap.max(label.width + 24.);
            v_gap = v_gap.max(label.height + 24.);
            if link.style.orientation == Orientation::Horizontal {
                h_steps[link.from] = h_steps[link.from].max(label.height + 12.);
            } else {
                v_steps[link.from] = v_steps[link.from].max(label.width + 12.);
            }
        }
        store(&mut links, label, context.budget)?;
    }
    Ok(Measures {
        blocks,
        links,
        h_gap,
        v_gap,
        h_steps,
        v_steps,
    })
}
fn place(model: &Wire, m: &Measures, context: &mut Context<'_>) -> Result<Geometry, RenderError> {
    let mut sizes = Vec::new();
    let mut locals = Vec::new();
    let mut horizontal = Vec::new();
    let mut vertical = Vec::new();
    for _ in &model.blocks {
        store(&mut sizes, (0_f64, 0_f64), context.budget)?;
        store(&mut locals, Point::new(0., 0.), context.budget)?;
        store(&mut horizontal, 0usize, context.budget)?;
        store(&mut vertical, 0usize, context.budget)?;
    }
    for link in &model.links {
        context.budget.charge(1)?;
        if link.style.orientation == Orientation::Horizontal {
            horizontal[link.from] += 1;
        } else {
            vertical[link.from] += 1;
        }
    }
    for id in (0..model.blocks.len()).rev() {
        context.budget.charge(1)?;
        let block = &model.blocks[id];
        let label = &m.blocks[id];
        let header = if id == 0 { 0. } else { label.height };
        let mut x = 10_f64;
        let mut y = header + 20.;
        let mut right = 10_f64;
        let mut bottom = header + 10.;
        for operation in &block.placements {
            context.budget.charge(1)?;
            match operation {
                Placement::Column => {
                    x = right + m.h_gap;
                    y = header + 20.;
                }
                Placement::Child(child) => {
                    locals[*child] = Point::new(x, y);
                    right = right.max(x + sizes[*child].0);
                    bottom = bottom.max(y + sizes[*child].1);
                    y += sizes[*child].1 + m.v_gap;
                }
            }
        }
        let leaf = block.children.is_empty();
        let width = (right + 10.)
            .max(label.width + 20.)
            .max(if leaf { 100. } else { 0. })
            .max(if vertical[id] > 0 {
                10. + (vertical[id] - 1) as f64 * m.v_steps[id]
            } else {
                0.
            });
        let height = (bottom + 10.)
            .max(header + 20.)
            .max(if leaf { 100. } else { 0. })
            .max(if horizontal[id] > 0 {
                10. + (horizontal[id] - 1) as f64 * m.h_steps[id]
            } else {
                0.
            });
        let size = match block.size {
            Size::Auto => (width, height),
            Size::Forced(w, h) => {
                let minimum_w = if leaf { label.width + 20. } else { width };
                let minimum_h = if leaf { label.height + 10. } else { height };
                if (w as f64) < minimum_w || (h as f64) < minimum_h {
                    return Err(fail(
                        context,
                        (
                            block.span.clone(),
                            block.key.len().saturating_mul(4).saturating_add(256),
                        ),
                        || {
                            format!(
                                "wire block {:?} forced {w}x{h} is smaller than measured contents {minimum_w}x{minimum_h}",
                                block.key
                            )
                        },
                    ));
                }
                (w as f64, h as f64)
            }
        };
        if !size.0.is_finite() || !size.1.is_finite() || size.0 > 999000. || size.1 > 999000. {
            return Err(fail(
                context,
                (
                    block.span.clone(),
                    block.key.len().saturating_mul(4).saturating_add(256),
                ),
                || {
                    format!(
                        "wire block {:?} measured dimensions exceed coordinate limit",
                        block.key
                    )
                },
            ));
        }
        sizes[id] = size;
    }
    let mut rects = Vec::<Rect>::new();
    for (id, block) in model.blocks.iter().enumerate() {
        context.budget.charge(1)?;
        let origin = if id == 0 {
            Point::new(24., 24.)
        } else {
            Point::new(
                rects[block.parent].x + locals[id].x,
                rects[block.parent].y + locals[id].y,
            )
        };
        if origin.x + sizes[id].0 > 999000. || origin.y + sizes[id].1 > 999000. {
            return Err(fail(
                context,
                (
                    block.span.clone(),
                    block.key.len().saturating_mul(4).saturating_add(256),
                ),
                || {
                    format!(
                        "wire block {:?} cumulative placement exceeds coordinate limit",
                        block.key
                    )
                },
            ));
        }
        store(
            &mut rects,
            Rect {
                x: origin.x,
                y: origin.y,
                width: sizes[id].0,
                height: sizes[id].1,
            },
            context.budget,
        )?;
    }
    Ok(Geometry {
        rects,
        width: sizes[0].0,
        height: sizes[0].1,
    })
}
fn fail(
    context: &mut Context<'_>,
    site: (std::ops::Range<usize>, usize),
    message: impl FnOnce() -> String,
) -> RenderError {
    if let Err(error) = context.budget.text(site.1) {
        return error;
    }
    context.locate_failure(wire::failure(site.0, message()))
}
fn draw_blocks(
    target: (&mut Scene, &Wire),
    view: (&Measures, &Geometry),
    context: &mut Context<'_>,
) -> Result<(), RenderError> {
    let (scene, model) = target;
    let (m, g) = view;
    for (id, block) in model.blocks.iter().enumerate().skip(1) {
        let rect = g.rects[id];
        let mut element = Element::new(format!("w{id}"), "wire-block");
        element.name = block.key.clone();
        if block.parent > 0 {
            element.owner = format!("w{}", block.parent);
        }
        element.shapes.push(Shape::Rect(
            rect,
            0.,
            match block.color {
                WireColor::Default => {
                    if block.children.is_empty() {
                        Paint::Paper
                    } else {
                        Paint::Container
                    }
                }
                WireColor::Rgb(color) => Paint::Color(color),
            },
        ));
        element.shapes.push(Shape::Text(
            Point::new(rect.x + 5., rect.y + 5.),
            m.blocks[id].clone(),
            Decoration::Plain,
        ));
        scene.push(element, context.budget)?;
    }
    Ok(())
}
fn endpoints(
    model: &Wire,
    view: (&Measures, &Geometry, usize),
    context: &mut Context<'_>,
) -> Result<(Point, Point), RenderError> {
    let (m, g, id) = view;
    let link = &model.links[id];
    let source = g.rects[link.from];
    let target = g.rects[link.to];
    let Anchor::Ordered(slot) = link.anchor;
    let (start, end, inside) = if link.style.orientation == Orientation::Horizontal {
        let y = source.y + 5. + slot as f64 * m.h_steps[link.from];
        (
            Point::new(source.x + source.width, y),
            Point::new(target.x, y),
            y + 5. <= source.y + source.height && y >= target.y && y <= target.y + target.height,
        )
    } else {
        let x = source.x + 5. + slot as f64 * m.v_steps[link.from];
        (
            Point::new(x, source.y + source.height),
            Point::new(x, target.y),
            x + 5. <= source.x + source.width && x >= target.x && x <= target.x + target.width,
        )
    };
    let length = if link.style.orientation == Orientation::Horizontal {
        end.x - start.x
    } else {
        end.y - start.y
    };
    if !inside || length < 12. {
        return Err(fail(
            context,
            (
                link.span.clone(),
                (model.blocks[link.from].key.len() + model.blocks[link.to].key.len())
                    .saturating_mul(4)
                    .saturating_add(256),
            ),
            || {
                format!(
                    "wire route {} to {} misses the destination extent, exhausts attachment slots, or has nonpositive clearance",
                    model.blocks[link.from].key, model.blocks[link.to].key
                )
            },
        ));
    }
    Ok((start, end))
}
fn draw_link(
    target: (&mut Scene, &Wire),
    view: (&Measures, &Geometry, usize),
    context: &mut Context<'_>,
) -> Result<(), RenderError> {
    let (scene, model) = target;
    let (m, g, id) = view;
    let link = &model.links[id];
    let (start, end) = endpoints(model, view, context)?;
    let label = &m.links[id];
    let _ = link.style.kind;
    let mut element = Element::new(format!("wl{id}"), "wire-link");
    element.owner = format!("w{}", link.from);
    element.target = format!("w{}", link.to);
    element.name = if link.style.orientation == Orientation::Horizontal {
        "horizontal"
    } else {
        "vertical"
    }
    .into();
    element.shapes.push(Shape::ColoredLine(
        vec![start, end],
        Dash::Solid,
        link.color.value(),
    ));
    if matches!(link.style.direction, Direction::Forward | Direction::Both) {
        element.shapes.push(head(end, start, link.color.value()));
    }
    if matches!(link.style.direction, Direction::Reverse | Direction::Both) {
        element.shapes.push(head(start, end, link.color.value()));
    }
    let point = if link.style.orientation == Orientation::Horizontal {
        Point::new(
            (start.x + end.x - label.width) / 2.,
            start.y - label.height / 2.,
        )
    } else {
        Point::new(start.x + 8., (start.y + end.y - label.height) / 2.)
    };
    let label_rect = Rect {
        x: point.x,
        y: point.y,
        width: label.width,
        height: label.height,
    };
    for (node, rect) in g.rects.iter().enumerate().skip(1) {
        if model.blocks[node].children.is_empty()
            && node != link.from
            && node != link.to
            && (crosses(*rect, start, end)
                || (!link.label.is_empty() && overlaps(*rect, label_rect)))
        {
            return Err(fail(
                context,
                (
                    link.span.clone(),
                    model.blocks[node]
                        .key
                        .len()
                        .saturating_mul(4)
                        .saturating_add(256),
                ),
                || {
                    format!(
                        "wire route or label intersects unrelated block {:?}",
                        model.blocks[node].key
                    )
                },
            ));
        }
    }
    if !link.label.is_empty() {
        element
            .shapes
            .push(Shape::Rect(label_rect, 0., Paint::Paper));
        element
            .shapes
            .push(Shape::Text(point, label.clone(), Decoration::Plain));
    }
    scene.push(element, context.budget)
}
fn overlaps(a: Rect, b: Rect) -> bool {
    a.x < b.x + b.width && a.x + a.width > b.x && a.y < b.y + b.height && a.y + a.height > b.y
}
fn crosses(rect: Rect, start: Point, end: Point) -> bool {
    if start.x == end.x {
        start.x > rect.x
            && start.x < rect.x + rect.width
            && start.y < rect.y + rect.height
            && end.y > rect.y
    } else {
        start.y > rect.y
            && start.y < rect.y + rect.height
            && start.x < rect.x + rect.width
            && end.x > rect.x
    }
}
fn head(tip: Point, previous: Point, color: super::style::Color) -> Shape {
    let dx = if tip.x == previous.x {
        0.
    } else {
        (tip.x - previous.x).signum()
    };
    let dy = if tip.y == previous.y {
        0.
    } else {
        (tip.y - previous.y).signum()
    };
    Shape::Polygon(
        vec![
            tip,
            Point::new(tip.x - dx * 7. - dy * 4., tip.y - dy * 7. + dx * 4.),
            Point::new(tip.x - dx * 7. + dy * 4., tip.y - dy * 7. - dx * 4.),
        ],
        Paint::Color(color),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        RenderOptions,
        plantuml::parser::{test_support, wire::parse},
    };
    use merman_render::environment::RenderEnvironment;
    fn inspect(body: &str, verify: impl FnOnce(Wire, Measures, Geometry, Scene)) {
        let mut budget = Budget::new(&RenderOptions::default());
        let source = format!("@startwire\n{body}\n@endwire");
        let (preamble, model) = test_support::styled(&source, &mut budget, parse).unwrap();
        let session = RenderEnvironment::deterministic().begin_session().unwrap();
        let mut context = Context::new(&session, &mut budget).with_fonts(preamble.fonts);
        let m = measure(&model, &mut context).unwrap();
        let g = place(&model, &m, &mut context).unwrap();
        let scene = layout(&model, &mut context).unwrap();
        verify(model, m, g, scene);
    }
    #[test]
    fn should_draw_axis_aligned_arrowheads_with_the_tip_on_each_wire_endpoint() {
        // given
        for (tip, previous, horizontal) in [
            (Point::new(20., 10.), Point::new(0., 10.), true),
            (Point::new(0., 10.), Point::new(20., 10.), true),
            (Point::new(10., 20.), Point::new(10., 0.), false),
        ] {
            // when
            let Shape::Polygon(points, _) = head(tip, previous, super::super::style::Color(0))
            else {
                panic!()
            };
            // then
            assert_eq!(points[0].x, tip.x);
            assert_eq!(points[0].y, tip.y);
            if horizontal {
                assert_eq!(points[1].x, points[2].x);
                assert_eq!(points[1].y + points[2].y, 2. * tip.y);
            } else {
                assert_eq!(points[1].y, points[2].y);
                assert_eq!(points[1].x + points[2].x, 2. * tip.x);
            }
        }
    }
    #[test]
    fn should_size_children_once_and_keep_parent_labels_columns_and_forced_leaf_dimensions() {
        // given
        for font in [6, 14, 96] {
            let body = format!(
                "skinparam defaultFontSize {font}\n* Parent_block\n    * Left_child\n    ---\n    * Right_child\n* F [100x150]"
            );
            // when
            inspect(&body, |model, m, g, scene| {
                // then
                let parent = g.rects[1];
                for id in [2, 3] {
                    let child = g.rects[id];
                    assert!(child.x > parent.x && child.y >= parent.y + m.blocks[1].height + 10.);
                    assert!(
                        child.x + child.width <= parent.x + parent.width
                            && child.y + child.height <= parent.y + parent.height
                    );
                    assert_eq!(model.blocks[id].parent, 1);
                }
                assert!(g.rects[3].x >= g.rects[2].x + g.rects[2].width + 20.);
                assert_eq!(g.rects[4].width, 100.);
                assert_eq!(g.rects[4].height, 150.);
                for e in &scene.elements {
                    for shape in &e.shapes {
                        if let Shape::Text(p, l, _) = shape {
                            assert!(
                                p.x >= scene.min_x
                                    && p.y >= scene.min_y
                                    && p.x + l.width <= scene.width
                                    && p.y + l.height <= scene.height
                            );
                        }
                    }
                }
            });
        }
    }
    #[test]
    fn should_keep_separate_source_slots_and_reserve_long_horizontal_and_vertical_labels() {
        // given
        let body = "* A [300x300]\n* C [300x300]\n---\n* B [300x300]\nA -> B : first long horizontal label\nA --> C : down\nA <- B : second long horizontal label\nA <-- C : down again";
        // when
        inspect(body, |_, m, g, scene| {
            // then
            let links = scene
                .elements
                .iter()
                .filter(|e| e.kind == "wire-link")
                .collect::<Vec<_>>();
            let lines = links
                .iter()
                .map(|e| match &e.shapes[0] {
                    Shape::ColoredLine(points, _, _) => points,
                    _ => panic!(),
                })
                .collect::<Vec<_>>();
            assert_eq!(lines[0][0].y + m.h_steps[1], lines[2][0].y);
            assert_eq!(lines[1][0].x + m.v_steps[1], lines[3][0].x);
            assert_eq!(lines[0][0].y, lines[0][1].y);
            assert_eq!(lines[1][0].x, lines[1][1].x);
            for e in links {
                for shape in &e.shapes {
                    if let Shape::Text(p, label, _) = shape {
                        let rect = Rect {
                            x: p.x,
                            y: p.y,
                            width: label.width,
                            height: label.height,
                        };
                        assert!(
                            !overlaps(rect, g.rects[1])
                                && !overlaps(rect, g.rects[2])
                                && !overlaps(rect, g.rects[3])
                        );
                    }
                }
            }
        });
    }

    #[test]
    fn should_keep_measured_attachment_spacing_local_to_each_source_block() {
        // given
        let body = "* A [500x500]\n* C [500x500]\n---\n* B [500x500]\n* D [500x500]\nA -> B : multi\\nline\\nlabel\nC -> D : plain\nC <- D : plain";
        // when
        inspect(body, |_, m, _, scene| {
            // then
            let links = scene
                .elements
                .iter()
                .filter(|e| e.kind == "wire-link")
                .collect::<Vec<_>>();
            let Shape::ColoredLine(a, _, _) = &links[1].shapes[0] else {
                panic!()
            };
            let Shape::ColoredLine(b, _, _) = &links[2].shapes[0] else {
                panic!()
            };
            assert!((b[0].y - a[0].y - (m.links[1].height + 12.).max(15.)).abs() < 1e-6);
        });
    }

    #[test]
    fn should_cancel_iterative_placement_and_reject_route_product_before_measurement() {
        // given
        let mut budget = Budget::new(&RenderOptions::default());
        let body = (0..100)
            .map(|id| format!("* A{id}"))
            .collect::<Vec<_>>()
            .join("\n");
        let model = test_support::body((&body, "wire"), &mut budget, parse).unwrap();
        let session = RenderEnvironment::deterministic().begin_session().unwrap();
        let mut context = Context::new(&session, &mut budget);
        let m = measure(&model, &mut context).unwrap();
        context.budget.control.cancel_after_checkpoints(10);
        // when
        let result = place(&model, &m, &mut context);
        // then
        assert!(matches!(result, Err(RenderError::Cancelled(_))));
        let mut budget = Budget::new(&RenderOptions::default());
        let model =
            test_support::body(("* A\n---\n* B\nA -> B", "wire"), &mut budget, parse).unwrap();
        budget.limits.layout_work = budget.work + 1;
        let mut context = Context::new(&session, &mut budget);
        assert!(matches!(
            layout(&model, &mut context),
            Err(RenderError::Backend(_))
        ));
    }
}
