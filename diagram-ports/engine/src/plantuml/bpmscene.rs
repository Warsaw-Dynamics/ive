//! Sparse native BPM branch composition; no Java grid-cleaner loop or activity parser.
use super::{
    bpm::{Bpm, EdgeKind, InstructionKind, MergeKind, NodeKind, Owner},
    budget::{Budget, bound},
    scene::{Dash, Element, Paint, Point, Rect, Scene, Shape, text_at},
    text::{Context, Label},
};
use crate::RenderError;
#[derive(Clone, Copy, Default, Debug)]
struct Size {
    width: f64,
    height: f64,
}
#[derive(Clone, Copy)]
struct Placement {
    block: usize,
    x: f64,
    y: f64,
}
struct Measured {
    instructions: Vec<Size>,
    blocks: Vec<Size>,
    nodes: Vec<Size>,
    labels: Vec<Label>,
}
fn reserve<T>(values: &mut Vec<T>, count: usize, budget: &mut Budget) -> Result<(), RenderError> {
    bound("BPM layout entries", count, budget.limits.model_items)?;
    budget.charge(count)?;
    let bytes = count.saturating_mul(std::mem::size_of::<T>());
    bound(
        "BPM layout allocation bytes",
        bytes,
        budget
            .limits
            .model_items
            .saturating_mul(std::mem::size_of::<T>()),
    )?;
    values.try_reserve_exact(count).map_err(|error| {
        RenderError::Backend(format!("PlantUML BPM layout allocation failed: {error}"))
    })
}
fn filled<T: Clone>(count: usize, value: T, budget: &mut Budget) -> Result<Vec<T>, RenderError> {
    let mut values = Vec::new();
    reserve(&mut values, count, budget)?;
    values.resize(count, value);
    Ok(values)
}
fn push<T>(values: &mut Vec<T>, value: T, budget: &mut Budget) -> Result<(), RenderError> {
    bound(
        "BPM layout entries",
        values.len() + 1,
        budget.limits.model_items,
    )?;
    budget.charge(1)?;
    if values.len() == values.capacity() {
        let capacity = values.capacity().saturating_mul(2).max(4);
        bound(
            "BPM layout capacity peak",
            capacity.saturating_add(values.capacity()),
            budget.limits.model_items.saturating_mul(3).max(4),
        )?;
        budget.charge(values.len())?;
        values
            .try_reserve_exact(capacity - values.len())
            .map_err(|error| {
                RenderError::Backend(format!("PlantUML BPM layout growth failed: {error}"))
            })?;
    }
    values.push(value);
    Ok(())
}
fn measure(model: &Bpm, context: &mut Context<'_>) -> Result<Measured, RenderError> {
    let blank = Label {
        lines: std::sync::Arc::from([]),
        width: 0.,
        height: 0.,
    };
    let mut measured = Measured {
        instructions: filled(model.instructions.len(), Size::default(), context.budget)?,
        blocks: filled(model.blocks.len(), Size::default(), context.budget)?,
        nodes: filled(model.nodes.len(), Size::default(), context.budget)?,
        labels: filled(model.nodes.len(), blank, context.budget)?,
    };
    for (id, node) in model.nodes.iter().enumerate() {
        context.budget.charge(1)?;
        let size = if node.kind == NodeKind::Event {
            let label = context.label(&node.label)?;
            let size = Size {
                width: (label.width + 28.).max(80.),
                height: (label.height + 24.).max(40.),
            };
            measured.labels[id] = label;
            size
        } else {
            Size {
                width: 20.,
                height: 20.,
            }
        };
        measured.nodes[id] = size;
    }
    // Child blocks are reduced before their branch instruction and enclosing block.
    for (id, block) in model.blocks.iter().enumerate() {
        let mut size = Size::default();
        for &instruction in &block.instructions {
            context.budget.charge(1)?;
            let tile = match &model.instructions[instruction].kind {
                InstructionKind::Event(node) => measured.nodes[*node],
                InstructionKind::Branch { arms, .. } => {
                    let mut width = 0f64;
                    let mut height = 0.;
                    for (i, &arm) in arms.iter().enumerate() {
                        context.budget.charge(1)?;
                        let a = measured.blocks[arm];
                        width = width.max(a.width);
                        height += a.height + if i == 0 { 0. } else { 32. };
                    }
                    Size {
                        width: width + 120.,
                        height: height.max(20.),
                    }
                }
            };
            for value in [tile.width, tile.height] {
                if !value.is_finite() || !(0. ..=1_000_000.).contains(&value) {
                    return Err(super::budget::invalid(1, "excessive BPM tile geometry"));
                }
            }
            measured.instructions[instruction] = tile;
            size.width += tile.width + if size.width == 0. { 0. } else { 40. };
            size.height = size.height.max(tile.height);
        }
        measured.blocks[id] = size;
    }
    Ok(measured)
}
fn positions(
    model: &Bpm,
    measured: &Measured,
    budget: &mut Budget,
) -> Result<Vec<Rect>, RenderError> {
    let mut rectangles = filled(
        model.nodes.len(),
        Rect {
            x: 0.,
            y: 0.,
            width: 0.,
            height: 0.,
        },
        budget,
    )?;
    let center = 30. + measured.blocks[model.root].height.max(20.) / 2.;
    rectangles[0] = Rect {
        x: 30.,
        y: center - 10.,
        width: 20.,
        height: 20.,
    };
    let mut pending = Vec::new();
    push(
        &mut pending,
        Placement {
            block: model.root,
            x: 90.,
            y: center,
        },
        budget,
    )?;
    while let Some(placement) = pending.pop() {
        let mut x = placement.x;
        let y = placement.y;
        for &id in &model.blocks[placement.block].instructions {
            budget.charge(1)?;
            let size = measured.instructions[id];
            match &model.instructions[id].kind {
                InstructionKind::Event(node) => {
                    rectangles[*node] = Rect {
                        x,
                        y: y - size.height / 2.,
                        width: size.width,
                        height: size.height,
                    }
                }
                InstructionKind::Branch { entry, exit, arms } => {
                    rectangles[*entry] = Rect {
                        x,
                        y: y - 10.,
                        width: 20.,
                        height: 20.,
                    };
                    rectangles[*exit] = Rect {
                        x: x + size.width - 20.,
                        y: y - 10.,
                        width: 20.,
                        height: 20.,
                    };
                    let mut top = y - size.height / 2.;
                    for &arm in arms {
                        budget.charge(1)?;
                        let a = measured.blocks[arm];
                        push(
                            &mut pending,
                            Placement {
                                block: arm,
                                x: x + 60.,
                                y: top + a.height / 2.,
                            },
                            budget,
                        )?;
                        top += a.height + 32.;
                    }
                }
            }
            x += size.width + 40.;
        }
    }
    Ok(rectangles)
}
fn owner(value: Owner) -> String {
    match value {
        Owner::Document => "bpm-root".into(),
        Owner::Branch(id) => format!("bpm-branch-{id}"),
    }
}
pub(super) fn layout(model: &Bpm, context: &mut Context<'_>) -> Result<Scene, RenderError> {
    let measured = measure(model, context)?;
    let rectangles = positions(model, &measured, context.budget)?;
    let mut scene = Scene::new();
    context.budget.item()?;
    scene.push(Element::new("bpm-root", "bpm-document"), context.budget)?;
    for (id, instruction) in model.instructions.iter().enumerate() {
        context.budget.charge(1)?;
        if let InstructionKind::Branch { entry, .. } = instruction.kind {
            context.budget.item()?;
            let mut element = Element::new(format!("bpm-branch-{id}"), "bpm-branch");
            element.owner = owner(model.nodes[entry].owner);
            element.target = format!("bytes:{}..{}", instruction.span.start, instruction.span.end);
            scene.push(element, context.budget)?;
        }
    }
    for (id, node) in model.nodes.iter().enumerate() {
        context.budget.item()?;
        context.budget.charge(12)?;
        let kind = match node.kind {
            NodeKind::Start => "bpm-start",
            NodeKind::Event => "bpm-event",
            NodeKind::Merge(MergeKind::Split) => "bpm-split",
            NodeKind::Merge(MergeKind::Join) => "bpm-merge",
        };
        let mut element = Element::new(format!("bpm-n{id}"), kind);
        element.name = node.label.clone();
        element.owner = owner(node.owner);
        element.target = format!("bytes:{}..{}", node.span.start, node.span.end);
        let r = rectangles[id];
        match node.kind {
            NodeKind::Start => element.shapes.push(Shape::Ellipse(r, Paint::Ink)),
            NodeKind::Event => {
                element.shapes.push(Shape::Rect(r, 0., Paint::Node));
                element.shapes.push(text_at(r, measured.labels[id].clone()));
            }
            NodeKind::Merge(_) => element.shapes.push(Shape::Polygon(
                vec![
                    Point::new(r.x + r.width / 2., r.y),
                    Point::new(r.x + r.width, r.y + r.height / 2.),
                    Point::new(r.x + r.width / 2., r.y + r.height),
                    Point::new(r.x, r.y + r.height / 2.),
                ],
                Paint::Node,
            )),
        }
        scene.push(element, context.budget)?;
    }
    for (id, edge) in model.edges.iter().enumerate() {
        context.budget.item()?;
        context.budget.charge(20)?;
        let (a, b) = (rectangles[edge.from], rectangles[edge.to]);
        let from = Point::new(a.x + a.width, a.y + a.height / 2.);
        let to = Point::new(b.x, b.y + b.height / 2.);
        let points = match edge.kind {
            EdgeKind::Next => vec![from, to],
            EdgeKind::ArmEnter => vec![
                from,
                Point::new(from.x + 16., from.y),
                Point::new(from.x + 16., to.y),
                to,
            ],
            EdgeKind::ArmExit => vec![
                from,
                Point::new(to.x - 16., from.y),
                Point::new(to.x - 16., to.y),
                to,
            ],
        };
        let mut element = Element::new(format!("bpm-e{id}"), "bpm-edge");
        element.name = format!("bpm-n{}", edge.from);
        element.target = format!("bpm-n{}", edge.to);
        element.owner = owner(edge.owner);
        element.shapes.push(Shape::Line(points, Dash::Solid));
        element.shapes.push(Shape::Polygon(
            vec![
                to,
                Point::new(to.x - 6., to.y - 3.),
                Point::new(to.x - 6., to.y + 3.),
            ],
            Paint::Ink,
        ));
        scene.push(element, context.budget)?;
    }
    Ok(scene)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RenderOptions, plantuml::parser};
    use merman_render::environment::RenderEnvironment;
    fn draw(body: &str) -> Scene {
        let mut budget = Budget::new(&RenderOptions::default());
        let model =
            parser::test_support::body((body, "bpm"), &mut budget, parser::bpm::parse).unwrap();
        let session = RenderEnvironment::deterministic().begin_session().unwrap();
        layout(&model, &mut Context::new(&session, &mut budget)).unwrap()
    }
    fn rectangle(element: &Element) -> Rect {
        match element.shapes[0] {
            Shape::Rect(r, ..) => r,
            _ => panic!("event rectangle"),
        }
    }
    #[test]
    fn should_stack_ordered_arms_without_overlapping_events_and_keep_continue_to_the_right() {
        // given
        let scene = draw(
            ":Before;\nnew branch\n:Top;\nelse\n:Middle;\nelse\n:Bottom;\nend branch\n:After;",
        );
        // when
        let event = |name| {
            rectangle(
                scene
                    .elements
                    .iter()
                    .find(|e| e.kind == "bpm-event" && e.name == name)
                    .unwrap(),
            )
        };
        let (top, middle, bottom, after) = (
            event("Top"),
            event("Middle"),
            event("Bottom"),
            event("After"),
        );
        // then
        assert!(top.y + top.height < middle.y && middle.y + middle.height < bottom.y);
        assert_eq!(top.x, middle.x);
        assert_eq!(middle.x, bottom.x);
        assert!(after.x > top.x + top.width);
        let start = scene
            .elements
            .iter()
            .find(|e| e.kind == "bpm-start")
            .unwrap();
        assert!(matches!(start.shapes[0], Shape::Ellipse(..)));
        for e in scene
            .elements
            .iter()
            .filter(|e| matches!(e.kind, "bpm-split" | "bpm-merge"))
        {
            assert!(matches!(&e.shapes[0],Shape::Polygon(points,_) if points.len()==4));
        }
    }
    #[test]
    fn should_route_nested_split_and_join_edges_outside_all_event_boxes_and_contain_labels() {
        // given
        let scene = draw(
            ":Before;\nnew branch\n:Top **bold**;\nnew branch\n:Wide first event;\nelse\n:Small;\nend branch\nelse\n:Bottom\\nsecond line;\nend branch\n:After;",
        );
        // when
        let events = scene
            .elements
            .iter()
            .filter(|e| e.kind == "bpm-event")
            .collect::<Vec<_>>();
        let boxes = events.iter().map(|e| rectangle(e)).collect::<Vec<_>>();
        // then
        for element in &scene.elements {
            for shape in &element.shapes {
                if let Shape::Line(vertices, _) = shape {
                    assert!(vertices.len() <= 4);
                    for pair in vertices.windows(2) {
                        let (a, b) = (pair[0], pair[1]);
                        assert!(a.x == b.x || a.y == b.y);
                        for r in &boxes {
                            let crosses = if a.y == b.y {
                                a.y > r.y
                                    && a.y < r.y + r.height
                                    && a.x.min(b.x) < r.x + r.width
                                    && a.x.max(b.x) > r.x
                            } else {
                                a.x > r.x
                                    && a.x < r.x + r.width
                                    && a.y.min(b.y) < r.y + r.height
                                    && a.y.max(b.y) > r.y
                            };
                            assert!(!crosses, "{} crosses {r:?}", element.id);
                        }
                    }
                }
                if let Shape::Text(p, label, _) = shape {
                    assert!(
                        p.x >= scene.min_x
                            && p.y >= scene.min_y
                            && p.x + label.width <= scene.width
                            && p.y + label.height <= scene.height
                    );
                }
            }
        }
        for (element, r) in events.iter().zip(boxes) {
            for shape in &element.shapes {
                if let Shape::Text(p, label, _) = shape {
                    assert!(
                        p.x >= r.x
                            && p.y >= r.y
                            && p.x + label.width <= r.x + r.width
                            && p.y + label.height <= r.y + r.height
                    );
                }
            }
        }
    }
    #[test]
    fn should_bound_layout_work_before_allocating_sparse_positions() {
        // given
        let mut config = RenderOptions::default();
        let mut budget = Budget::new(&config);
        let model =
            parser::test_support::body((":A;\n:B;", "bpm"), &mut budget, parser::bpm::parse)
                .unwrap();
        let session = RenderEnvironment::deterministic().begin_session().unwrap();
        config.limits.layout_work = 1;
        let mut bounded = Budget::new(&config);
        // when
        let result = layout(&model, &mut Context::new(&session, &mut bounded));
        // then
        assert!(result.is_err());
    }
}
