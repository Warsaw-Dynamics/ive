//! Native measured tree layout; WBS fork/trunk topology follows authorized wbs sources.
use super::{
    scene::{Dash, Element, Paint, Point, Rect, Scene, Shape, text_at},
    text::{Context, Label},
    tree::{Mindmap, Node, Outline, Parent, Side, Wbs},
};
use crate::RenderError;

struct Measured {
    labels: Vec<Label>,
    sizes: Vec<Rect>,
    children: Vec<Vec<usize>>,
}

fn measure(nodes: &[Node], context: &mut Context<'_>) -> Result<Measured, RenderError> {
    let mut labels = Vec::new();
    let mut sizes = Vec::new();
    let mut children = vec![Vec::new(); nodes.len()];
    for (id, node) in nodes.iter().enumerate() {
        context.budget.charge(1)?;
        let label = context.tree_label(&node.label)?;
        let (width, height) = match node.outline {
            Outline::Pseudo => (0., 0.),
            Outline::Boxless => (label.width + 8., label.height + 4.),
            Outline::Box => (label.width + 24., label.height + 16.),
        };
        sizes.push(Rect {
            x: 0.,
            y: 0.,
            width,
            height,
        });
        labels.push(label);
        if let Parent::Node(parent) = node.parent {
            children[parent].push(id);
        }
    }
    Ok(Measured {
        labels,
        sizes,
        children,
    })
}

pub(super) fn mindmap(model: &Mindmap, context: &mut Context<'_>) -> Result<Scene, RenderError> {
    let measured = measure(&model.nodes, context)?;
    let mut heights: Vec<_> = measured.sizes.iter().map(|r| r.height).collect();
    for id in (0..model.nodes.len()).rev() {
        context.budget.charge(measured.children[id].len() + 1)?;
        let mut sums = [0., 0.];
        for &child in &measured.children[id] {
            let side = usize::from(model.nodes[child].side == Side::Right);
            if sums[side] > 0. {
                sums[side] += 20.;
            }
            sums[side] += heights[child];
        }
        heights[id] = heights[id].max(sums[0]).max(sums[1]);
    }
    let mut bounds = measured.sizes.clone();
    let mut top = 24.;
    for &root in &model.roots {
        bounds[root].x = -bounds[root].width / 2.;
        bounds[root].y = top + (heights[root] - bounds[root].height) / 2.;
        let mut pending = vec![root];
        while let Some(parent) = pending.pop() {
            context.budget.charge(measured.children[parent].len() + 1)?;
            for side in [Side::Left, Side::Right] {
                let children: Vec<_> = measured.children[parent]
                    .iter()
                    .copied()
                    .filter(|&id| model.nodes[id].side == side)
                    .collect();
                let total = children.iter().map(|&id| heights[id] + 20.).sum::<f64>()
                    - if children.is_empty() { 0. } else { 20. };
                let mut y = bounds[parent].y + bounds[parent].height / 2. - total / 2.;
                for child in children {
                    bounds[child].x = match side {
                        Side::Left => bounds[parent].x - 64. - bounds[child].width,
                        Side::Right => bounds[parent].x + bounds[parent].width + 64.,
                    };
                    bounds[child].y = y + (heights[child] - bounds[child].height) / 2.;
                    y += heights[child] + 20.;
                    pending.push(child);
                }
            }
        }
        top += heights[root] + 64.;
    }
    let mut scene = Scene::new();
    for (id, node) in model.nodes.iter().enumerate() {
        if let Parent::Node(parent) = node.parent {
            let a = bounds[parent];
            let b = bounds[id];
            let (x1, x2) = if node.side == Side::Right {
                (a.x + a.width, b.x)
            } else {
                (a.x, b.x + b.width)
            };
            let mid = (x1 + x2) / 2.;
            push_link(
                &mut scene,
                Link {
                    id: format!("me{id}"),
                    kind: "mindmap-branch",
                    owner: format!("m{parent}"),
                    target: format!("m{id}"),
                    points: vec![
                        Point::new(x1, a.y + a.height / 2.),
                        Point::new(mid, a.y + a.height / 2.),
                        Point::new(mid, b.y + b.height / 2.),
                        Point::new(x2, b.y + b.height / 2.),
                    ],
                },
                context,
            )?;
        }
    }
    paint_nodes(
        &model.nodes,
        &measured,
        &bounds,
        "mindmap",
        &mut scene,
        context,
    )?;
    Ok(scene)
}

#[derive(Clone, Copy, Default)]
struct Extent {
    left: f64,
    right: f64,
    height: f64,
}

pub(super) fn wbs(model: &Wbs, context: &mut Context<'_>) -> Result<Scene, RenderError> {
    let measured = measure(&model.nodes, context)?;
    let mut extents = vec![Extent::default(); model.nodes.len()];
    for id in (1..model.nodes.len()).rev() {
        context.budget.charge(measured.children[id].len() + 1)?;
        let size = measured.sizes[id];
        let mut extent = Extent {
            left: size.width / 2.,
            right: size.width / 2.,
            height: size.height,
        };
        let mut height = [0., 0.];
        for &child in &measured.children[id] {
            let side = usize::from(model.nodes[child].side == Side::Right);
            height[side] += extents[child].height + 24.;
            let width = extents[child].left + extents[child].right + 24.;
            if side == 0 {
                extent.left = extent.left.max(width);
            } else {
                extent.right = extent.right.max(width);
            }
        }
        extent.height += height[0].max(height[1]);
        extents[id] = extent;
    }
    let mut first = Vec::new();
    for &child in &measured.children[0] {
        context.budget.charge(first.len() + 1)?;
        if model.nodes[child].side == Side::Left {
            first.insert(0, child);
        } else {
            first.push(child);
        }
    }
    let mut bounds = measured.sizes.clone();
    let mut x = 24.;
    for &child in &first {
        let center = x + extents[child].left;
        bounds[child].x = center - bounds[child].width / 2.;
        bounds[child].y = 24. + bounds[0].height + 64.;
        x += extents[child].left + extents[child].right + 40.;
    }
    let root_center = if let (Some(&a), Some(&b)) = (first.first(), first.last()) {
        (bounds[a].x + bounds[a].width / 2. + bounds[b].x + bounds[b].width / 2.) / 2.
    } else {
        bounds[0].width / 2. + 24.
    };
    bounds[0].x = root_center - bounds[0].width / 2.;
    bounds[0].y = 24.;
    let mut scene = Scene::new();
    if !first.is_empty() {
        let bus = bounds[0].y + bounds[0].height + 32.;
        let a = first[0];
        let b = first[first.len() - 1];
        push_link(
            &mut scene,
            Link {
                id: "wbus".into(),
                kind: "wbs-bus",
                owner: "w0".into(),
                target: String::new(),
                points: vec![
                    Point::new(bounds[a].x + bounds[a].width / 2., bus),
                    Point::new(bounds[b].x + bounds[b].width / 2., bus),
                ],
            },
            context,
        )?;
        push_link(
            &mut scene,
            Link {
                id: "wroot".into(),
                kind: "wbs-trunk",
                owner: "w0".into(),
                target: String::new(),
                points: vec![
                    Point::new(root_center, bounds[0].y + bounds[0].height),
                    Point::new(root_center, bus),
                ],
            },
            context,
        )?;
        for &child in &first {
            let x = bounds[child].x + bounds[child].width / 2.;
            push_link(
                &mut scene,
                Link {
                    id: format!("we{child}"),
                    kind: "wbs-branch",
                    owner: "w0".into(),
                    target: format!("w{child}"),
                    points: vec![Point::new(x, bus), Point::new(x, bounds[child].y)],
                },
                context,
            )?;
        }
    }
    let mut pending = first;
    while let Some(parent) = pending.pop() {
        context.budget.charge(measured.children[parent].len() + 1)?;
        let rect = bounds[parent];
        let trunk = rect.x + rect.width / 2.;
        let bottom = rect.y + rect.height;
        let mut y = [bottom, bottom];
        let mut last_y = bottom;
        for &child in &measured.children[parent] {
            let side = usize::from(model.nodes[child].side == Side::Right);
            y[side] += 24.;
            let extent = extents[child];
            let center = if side == 0 {
                trunk - 24. - extent.right
            } else {
                trunk + 24. + extent.left
            };
            bounds[child].x = center - bounds[child].width / 2.;
            bounds[child].y = y[side];
            let mid = y[side] + bounds[child].height / 2.;
            let end = if side == 0 {
                bounds[child].x + bounds[child].width
            } else {
                bounds[child].x
            };
            last_y = last_y.max(mid);
            push_link(
                &mut scene,
                Link {
                    id: format!("we{child}"),
                    kind: "wbs-branch",
                    owner: format!("w{parent}"),
                    target: format!("w{child}"),
                    points: vec![Point::new(trunk, mid), Point::new(end, mid)],
                },
                context,
            )?;
            y[side] += extent.height;
            pending.push(child);
        }
        if last_y > bottom {
            push_link(
                &mut scene,
                Link {
                    id: format!("wt{parent}"),
                    kind: "wbs-trunk",
                    owner: format!("w{parent}"),
                    target: String::new(),
                    points: vec![Point::new(trunk, bottom), Point::new(trunk, last_y)],
                },
                context,
            )?;
        }
    }
    paint_nodes(&model.nodes, &measured, &bounds, "wbs", &mut scene, context)?;
    Ok(scene)
}

struct Link {
    id: String,
    kind: &'static str,
    owner: String,
    target: String,
    points: Vec<Point>,
}
fn push_link(scene: &mut Scene, link: Link, context: &mut Context<'_>) -> Result<(), RenderError> {
    let mut element = Element::new(link.id, link.kind);
    element.owner = link.owner;
    element.target = link.target;
    element.shapes.push(Shape::Line(link.points, Dash::Solid));
    scene.push(element, context.budget)
}

fn paint_nodes(
    nodes: &[Node],
    measured: &Measured,
    bounds: &[Rect],
    family: &str,
    scene: &mut Scene,
    context: &mut Context<'_>,
) -> Result<(), RenderError> {
    let prefix = if family == "wbs" { "w" } else { "m" };
    for (id, node) in nodes.iter().enumerate() {
        context.budget.charge(node.depth + 1)?;
        let kind = match (family, node.outline) {
            ("wbs", Outline::Pseudo) => "wbs-pseudo",
            ("wbs", _) => "wbs-node",
            _ => "mindmap-node",
        };
        let mut element = Element::new(format!("{prefix}{id}"), kind);
        element.name = if node.alias.is_empty() {
            node.label.clone()
        } else {
            node.alias.clone()
        };
        if let Parent::Node(parent) = node.parent {
            element.owner = format!("{prefix}{parent}");
        }
        if node.outline == Outline::Box {
            element
                .shapes
                .push(Shape::Rect(bounds[id], 5., Paint::Node));
        }
        if node.outline != Outline::Pseudo {
            element
                .shapes
                .push(text_at(bounds[id], measured.labels[id].clone()));
        }
        scene.push(element, context.budget)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::{budget::Budget, source::Line, tree};
    use super::*;
    use crate::RenderOptions;
    use merman_render::environment::RenderEnvironment;

    fn scene(source: &str, family: &str) -> Scene {
        let lines: Vec<_> = source
            .lines()
            .enumerate()
            .map(|(i, s)| Line {
                number: i + 2,
                text: s.into(),
            })
            .collect();
        let mut budget = Budget::new(&RenderOptions::default());
        let session = RenderEnvironment::deterministic().begin_session().unwrap();
        if family == "mindmap" {
            let model = tree::parse_mindmap(&lines, &mut budget).unwrap();
            mindmap(&model, &mut Context::new(&session, &mut budget)).unwrap()
        } else {
            let model = tree::parse_wbs(&lines, &mut budget).unwrap();
            wbs(&model, &mut Context::new(&session, &mut budget)).unwrap()
        }
    }
    fn rect(scene: &Scene, id: &str) -> Rect {
        scene
            .elements
            .iter()
            .find(|e| e.id == id)
            .unwrap()
            .shapes
            .iter()
            .find_map(|s| {
                if let Shape::Rect(r, _, _) = s {
                    Some(*r)
                } else {
                    None
                }
            })
            .unwrap()
    }
    fn check_nonoverlap_and_labels(scene: &Scene) {
        let boxes: Vec<_> = scene
            .elements
            .iter()
            .filter(|e| e.kind.ends_with("-node"))
            .filter_map(|e| {
                e.shapes
                    .iter()
                    .find_map(|s| {
                        if let Shape::Rect(r, _, _) = s {
                            Some(*r)
                        } else {
                            None
                        }
                    })
                    .map(|r| (e, r))
            })
            .collect();
        for (i, (element, a)) in boxes.iter().enumerate() {
            for (_, b) in &boxes[i + 1..] {
                assert!(
                    a.x + a.width <= b.x
                        || b.x + b.width <= a.x
                        || a.y + a.height <= b.y
                        || b.y + b.height <= a.y,
                    "overlap {a:?} {b:?}"
                );
            }
            for shape in &element.shapes {
                if let Shape::Text(p, label, _) = shape {
                    assert!(
                        p.x >= a.x
                            && p.y >= a.y
                            && p.x + label.width <= a.x + a.width + 1e-6
                            && p.y + label.height <= a.y + a.height + 1e-6
                    );
                }
            }
        }
    }

    #[test]
    fn should_reserve_nonoverlapping_measured_mindmap_subtrees_on_actual_sides() {
        // given
        let source = "+ Root\n-- Wide left branch\n--- Very long\\nmultiline\\nleft child\n++ Right\n+++ Long\\nright\\nchild\n++ Other\n+++ Other child\n+ Second map\n-- Second left";
        // when
        let scene = scene(source, "mindmap");
        // then
        check_nonoverlap_and_labels(&scene);
        let root = rect(&scene, "m0");
        let left = rect(&scene, "m1");
        let right = rect(&scene, "m3");
        assert!(left.x + left.width < root.x);
        assert!(right.x > root.x + root.width);
        assert!(rect(&scene, "m7").y > rect(&scene, "m6").y + rect(&scene, "m6").height);
        for e in scene.elements.iter().filter(|e| e.kind == "mindmap-branch") {
            let target = rect(&scene, &e.target);
            let Shape::Line(points, _) = &e.shapes[0] else {
                panic!()
            };
            assert_eq!(points.last().unwrap().y, target.y + target.height / 2.);
        }
    }

    #[test]
    fn should_draw_wbs_root_fork_and_attach_descendants_to_parent_trunks() {
        // given
        let source = "* Project\n** First\n*** Right child\n--- Left child\n**** Deep\\nwide child\n** Second\n*** Other\n-- Prepended\n---< Left descendant";
        // when
        let scene = scene(source, "wbs");
        // then
        check_nonoverlap_and_labels(&scene);
        let root = rect(&scene, "w0");
        let first = rect(&scene, "w1");
        let second = rect(&scene, "w5");
        let prepended = rect(&scene, "w7");
        assert!(root.y + root.height < first.y && root.y + root.height < second.y);
        assert!(prepended.x < first.x && first.x < second.x);
        let trunk = first.x + first.width / 2.;
        assert!(rect(&scene, "w2").x > trunk);
        assert!(rect(&scene, "w3").x + rect(&scene, "w3").width < trunk);
        let bus = scene.elements.iter().find(|e| e.kind == "wbs-bus").unwrap();
        let Shape::Line(points, _) = &bus.shapes[0] else {
            panic!()
        };
        assert_eq!(points[0].y, points[1].y);
        assert!(points[0].y > root.y + root.height && points[0].y < first.y);
        for e in scene
            .elements
            .iter()
            .filter(|e| e.kind == "wbs-branch" && e.owner == "w1")
        {
            let Shape::Line(points, _) = &e.shapes[0] else {
                panic!()
            };
            assert_eq!(points[0].x, trunk);
            assert_eq!(points[0].y, points[1].y);
        }
    }
}
