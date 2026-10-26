//! Original checked, recursive Dagre adapter for typed state ownership (not UML entities).
use super::{
    arrow,
    budget::{Budget, invalid},
    scene::{Dash, Decoration, Element, Paint, Point, Rect, Scene, Shape},
    state::{Axis, Kind, Scope, State},
    text::{Context, Label},
};
use crate::RenderError;
use dugong::{
    EdgeLabel, GraphLabel, NodeLabel,
    graphlib::{Graph, GraphOptions},
};

struct Work<'a> {
    budget: &'a mut Budget,
    failure: Vec<RenderError>,
}
impl dugong::WorkControl for Work<'_> {
    fn charge(&mut self, units: usize) -> Result<(), dugong::WorkError> {
        self.budget.charge(units).map_err(|error| {
            self.failure.push(error);
            dugong::WorkError::Interrupted
        })
    }
}
struct BoxNode {
    id: usize,
    rect: Rect,
    title: Label,
    body: Vec<Label>,
    children: Group,
}
struct Group {
    nodes: Vec<BoxNode>,
    edges: Vec<(usize, Vec<Point>, Point)>,
    width: f64,
    height: f64,
    regions: Vec<(usize, Rect)>,
}

pub(super) fn layout(model: &State, context: &mut Context<'_>) -> Result<Scene, RenderError> {
    if advanced(model) {
        admit_extension(model, context.budget)?;
    }
    let labels = model
        .transitions
        .iter()
        .map(|r| context.label(&r.label))
        .collect::<Result<Vec<_>, _>>()?;
    let root = compose(model, (Scope::Root, &labels), context)?;
    let mut scene = Scene::new();
    draw_group(
        &root,
        (model, &labels, Point::new(24., 24.)),
        (&mut scene, context),
    )?;
    draw_boundaries(model, (&root, &labels), (&mut scene, context))?;
    Ok(scene)
}

fn compose(
    model: &State,
    input: (Scope, &[Label]),
    context: &mut Context<'_>,
) -> Result<Group, RenderError> {
    let (scope, labels) = input;
    context
        .budget
        .charge(model.nodes.len() + model.transitions.len())?;
    let mut nodes = Vec::new();
    for (id, node) in model
        .nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| n.scope == scope)
    {
        let title = context.label(match node.kind {
            Kind::History => "H",
            Kind::DeepHistory => "H*",
            Kind::State | Kind::Composite => &node.label,
            _ => "",
        })?;
        let body = node
            .descriptions
            .iter()
            .map(|s| context.label(s))
            .collect::<Result<Vec<_>, _>>()?;
        let children = if node.kind == Kind::Composite {
            compose_regions(model, (id, labels), context)?
        } else {
            Group {
                nodes: vec![],
                edges: vec![],
                width: 0.,
                height: 0.,
                regions: vec![],
            }
        };
        let caption_width = body.iter().map(|l| l.width).fold(title.width, f64::max) + 32.;
        let caption_height = title.height + body.iter().map(|l| l.height + 4.).sum::<f64>() + 24.;
        let (width, height) = match node.kind {
            Kind::Initial | Kind::Final | Kind::Choice => (24., 24.),
            Kind::Junction => (12., 12.),
            Kind::ForkJoin => (80., 8.),
            Kind::History | Kind::DeepHistory => {
                let size = title.width.max(title.height).max(22.) + 12.;
                (size, size)
            }
            _ => (
                caption_width.max(children.width + 24.).max(80.),
                caption_height + children.height,
            ),
        };
        nodes.push(BoxNode {
            id,
            rect: Rect {
                x: 0.,
                y: 0.,
                width,
                height,
            },
            title,
            body,
            children,
        });
    }
    arrange(model, (scope, nodes, labels), context.budget)
}

fn compose_regions(
    model: &State,
    input: (usize, &[Label]),
    context: &mut Context<'_>,
) -> Result<Group, RenderError> {
    let (owner, labels) = input;
    context.budget.charge(model.regions.len())?;
    let regions = model
        .regions
        .iter()
        .enumerate()
        .filter(|(_, r)| r.owner == owner)
        .collect::<Vec<_>>();
    if regions.is_empty() {
        return compose(model, (Scope::State(owner), labels), context);
    }
    let mut group = Group {
        nodes: vec![],
        edges: vec![],
        width: 0.,
        height: 0.,
        regions: vec![],
    };
    for (id, region) in regions {
        context.budget.charge(1)?;
        let mut child = compose(model, (region.scope, labels), context)?;
        let rect = Rect {
            x: if region.axis == Axis::SideBySide {
                group.width
            } else {
                0.
            },
            y: if region.axis == Axis::Stacked {
                group.height
            } else {
                0.
            },
            width: child.width.max(80.),
            height: child.height.max(80.),
        };
        for node in &mut child.nodes {
            node.rect.x += rect.x;
            node.rect.y += rect.y;
        }
        for (_, points, label) in &mut child.edges {
            for p in points {
                p.x += rect.x;
                p.y += rect.y;
            }
            label.x += rect.x;
            label.y += rect.y;
        }
        group.width = group.width.max(rect.x + rect.width);
        group.height = group.height.max(rect.y + rect.height);
        group.nodes.extend(child.nodes);
        group.edges.extend(child.edges);
        group.regions.push((id, rect));
    }
    Ok(group)
}

fn arrange(
    model: &State,
    input: (Scope, Vec<BoxNode>, &[Label]),
    budget: &mut Budget,
) -> Result<Group, RenderError> {
    let (scope, mut nodes, labels) = input;
    let mut marginx = 24_f64;
    let mut marginy = 24_f64;
    if advanced(model) {
        context_boundary_margins(model, labels, (&mut marginx, &mut marginy), budget)?;
    }
    let mut graph = Graph::new(GraphOptions {
        directed: true,
        multigraph: true,
        compound: false,
    });
    graph.set_graph(GraphLabel {
        nodesep: 60.,
        ranksep: 70.,
        marginx,
        marginy,
        ..Default::default()
    });
    for node in &nodes {
        budget.charge(1)?;
        graph.set_node(
            format!("s{}", node.id),
            NodeLabel {
                width: node.rect.width,
                height: node.rect.height,
                ..Default::default()
            },
        );
    }
    let mut edges = Vec::new();
    for (id, edge) in model.transitions.iter().enumerate() {
        if model.nodes[edge.from].scope != scope {
            continue;
        }
        if model.nodes[edge.to].scope != scope {
            continue;
        }
        budget.charge(1)?;
        graph
            .try_set_edge_named(
                format!("s{}", edge.from),
                format!("s{}", edge.to),
                Some(format!("t{id}")),
                Some(EdgeLabel {
                    width: labels[id].width + 12.,
                    height: labels[id].height + 12.,
                    ..Default::default()
                }),
            )
            .map_err(|e| invalid(1, format!("state graph edge: {e}")))?;
    }
    if let crate::GraphLayout::Dot(runner) = budget.graph_layout.clone() {
        super::dot::layout(&mut graph, runner.as_ref(), budget)?;
    } else {
        let mut work = Work {
            budget,
            failure: vec![],
        };
        let result = dugong::layout_controlled(&mut graph, &mut work);
        if let Some(error) = work.failure.pop() {
            return Err(error);
        }
        result.map_err(|e| invalid(1, format!("state graph layout: {e}")))?;
    }
    let mut width: f64 = 0.;
    let mut height: f64 = 0.;
    for node in &mut nodes {
        budget.charge(1)?;
        let placed = graph
            .node(&format!("s{}", node.id))
            .ok_or_else(|| invalid(1, "missing state placement"))?;
        node.rect.x = placed.x.ok_or_else(|| invalid(1, "missing state x"))? - node.rect.width / 2.;
        node.rect.y =
            placed.y.ok_or_else(|| invalid(1, "missing state y"))? - node.rect.height / 2.;
        width = width.max(node.rect.x + node.rect.width + 24.);
        height = height.max(node.rect.y + node.rect.height + 24.);
    }
    for (id, edge) in model
        .transitions
        .iter()
        .enumerate()
        .filter(|(_, e)| model.nodes[e.from].scope == scope && model.nodes[e.to].scope == scope)
    {
        budget.charge(1)?;
        let placed = graph
            .edge(
                &format!("s{}", edge.from),
                &format!("s{}", edge.to),
                Some(&format!("t{id}")),
            )
            .ok_or_else(|| invalid(1, "missing state route"))?;
        let mut points = placed
            .points
            .iter()
            .map(|p| Point::new(p.x, p.y))
            .collect::<Vec<_>>();
        if advanced(model) && points.len() > 1 {
            budget.charge(nodes.len().saturating_mul(2))?;
            let from = nodes.iter().find(|node| node.id == edge.from).unwrap();
            let to = nodes.iter().find(|node| node.id == edge.to).unwrap();
            points[0] = border(from.rect, model.nodes[edge.from].kind, points[1]);
            let end = points.len() - 1;
            points[end] = border(to.rect, model.nodes[edge.to].kind, points[end - 1]);
        }
        let point = Point::new(
            placed.x.unwrap_or(0.) - labels[id].width / 2.,
            placed.y.unwrap_or(0.) - labels[id].height / 2.,
        );
        width = width.max(point.x + labels[id].width + 24.);
        height = height.max(point.y + labels[id].height + 24.);
        edges.push((id, points, point));
    }
    Ok(Group {
        nodes,
        edges,
        width,
        height,
        regions: vec![],
    })
}

fn draw_group(
    group: &Group,
    input: (&State, &[Label], Point),
    output: (&mut Scene, &mut Context<'_>),
) -> Result<(), RenderError> {
    let (model, labels, offset) = input;
    let (scene, context) = output;
    for node in &group.nodes {
        let data = &model.nodes[node.id];
        let rect = Rect {
            x: node.rect.x + offset.x,
            y: node.rect.y + offset.y,
            ..node.rect
        };
        let mut element = Element::new(
            format!("s{}", node.id),
            match data.kind {
                Kind::State => "state",
                Kind::Composite => "state-composite",
                Kind::Initial => "state-initial",
                Kind::Final => "state-final",
                Kind::History => "state-history",
                Kind::DeepHistory => "state-deep-history",
                Kind::Choice => "state-choice",
                Kind::Junction => "state-junction",
                Kind::ForkJoin => "state-fork-join",
            },
        );
        element.name = data.name.clone();
        if let Scope::State(owner) = data.scope {
            element.owner = format!("s{owner}");
        }
        if let Scope::Region(id) = data.scope {
            element.owner = format!("r{id}");
        }
        element.shapes = node_shapes(node, (rect, data.kind));
        scene.push(element, context.budget)?;
        let caption_height =
            node.title.height + node.body.iter().map(|l| l.height + 4.).sum::<f64>() + 24.;
        draw_group(
            &node.children,
            (
                model,
                labels,
                Point::new(rect.x + 12., rect.y + caption_height),
            ),
            (scene, context),
        )?;
    }
    for (id, rect) in &group.regions {
        let region = &model.regions[*id];
        let mut element = Element::new(format!("r{id}"), "state-region");
        element.owner = format!("s{}", region.owner);
        element.name = region.ordinal.to_string();
        if region.ordinal > 0 {
            let start = Point::new(rect.x + offset.x, rect.y + offset.y);
            let end = if region.axis == Axis::Stacked {
                Point::new(offset.x + group.width, start.y)
            } else {
                Point::new(start.x, offset.y + group.height)
            };
            element
                .shapes
                .push(Shape::Line(vec![start, end], Dash::Dashed));
        }
        scene.push(element, context.budget)?;
    }
    for (id, points, label_at) in &group.edges {
        let edge = &model.transitions[*id];
        let mut element = Element::new(format!("t{id}"), "state-transition");
        element.owner = format!("s{}", edge.from);
        element.target = format!("s{}", edge.to);
        element.name = edge.label.clone();
        element.shapes = arrow::draw(
            points
                .iter()
                .map(|p| Point::new(p.x + offset.x, p.y + offset.y))
                .collect(),
            arrow::Arrow {
                start: arrow::End::None,
                end: arrow::End::Filled,
                dash: Dash::Solid,
            },
        );
        if !edge.label.is_empty() {
            element.shapes.push(Shape::Text(
                Point::new(label_at.x + offset.x, label_at.y + offset.y),
                labels[*id].clone(),
                Decoration::Plain,
            ));
        }
        scene.push(element, context.budget)?;
    }
    Ok(())
}

fn node_shapes(node: &BoxNode, input: (Rect, Kind)) -> Vec<Shape> {
    let (rect, kind) = input;
    if kind == Kind::Initial {
        return vec![Shape::Ellipse(rect, Paint::Ink)];
    }
    if kind == Kind::Final {
        return vec![
            Shape::Ellipse(rect, Paint::Paper),
            Shape::Ellipse(
                Rect {
                    x: rect.x + 5.,
                    y: rect.y + 5.,
                    width: 14.,
                    height: 14.,
                },
                Paint::Ink,
            ),
        ];
    }
    if matches!(kind, Kind::History | Kind::DeepHistory) {
        return vec![
            Shape::Ellipse(rect, Paint::Paper),
            Shape::Text(
                Point::new(
                    rect.x + (rect.width - node.title.width) / 2.,
                    rect.y + (rect.height - node.title.height) / 2.,
                ),
                node.title.clone(),
                Decoration::Plain,
            ),
        ];
    }
    if kind == Kind::Junction {
        return vec![Shape::Ellipse(rect, Paint::Ink)];
    }
    if kind == Kind::ForkJoin {
        return vec![Shape::Rect(rect, 0., Paint::Ink)];
    }
    if kind == Kind::Choice {
        return vec![Shape::Polygon(
            vec![
                Point::new(rect.x + rect.width / 2., rect.y),
                Point::new(rect.x + rect.width, rect.y + rect.height / 2.),
                Point::new(rect.x + rect.width / 2., rect.y + rect.height),
                Point::new(rect.x, rect.y + rect.height / 2.),
            ],
            Paint::Node,
        )];
    }
    let mut shapes = vec![
        Shape::Rect(
            rect,
            10.,
            if kind == Kind::Composite {
                Paint::Container
            } else {
                Paint::Node
            },
        ),
        Shape::Text(
            Point::new(rect.x + (rect.width - node.title.width) / 2., rect.y + 10.),
            node.title.clone(),
            Decoration::Plain,
        ),
    ];
    let mut y = rect.y + node.title.height + 16.;
    if !node.body.is_empty() || kind == Kind::Composite {
        shapes.push(Shape::Line(
            vec![Point::new(rect.x, y), Point::new(rect.x + rect.width, y)],
            Dash::Solid,
        ));
    }
    for label in &node.body {
        shapes.push(Shape::Text(
            Point::new(rect.x + 16., y + 4.),
            label.clone(),
            Decoration::Plain,
        ));
        y += label.height + 4.;
    }
    shapes
}

fn advanced(model: &State) -> bool {
    !model.regions.is_empty()
        || model
            .nodes
            .iter()
            .any(|n| !matches!(n.kind, Kind::State | Kind::Composite) && n.name != "[*]")
}

fn admit_extension(model: &State, budget: &mut Budget) -> Result<(), RenderError> {
    let count = model
        .nodes
        .len()
        .saturating_add(model.regions.len())
        .saturating_add(model.transitions.len());
    super::budget::bound(
        "state extended layout retained bytes",
        count.saturating_mul(2048),
        budget
            .limits
            .source_bytes
            .saturating_mul(256)
            .saturating_add(budget.limits.model_items.saturating_mul(2048)),
    )?;
    budget.charge(count)
}

fn context_boundary_margins(
    model: &State,
    labels: &[Label],
    margins: (&mut f64, &mut f64),
    budget: &mut Budget,
) -> Result<(), RenderError> {
    let (x, y) = margins;
    for (id, edge) in model.transitions.iter().enumerate() {
        budget.charge(1)?;
        if model.nodes[edge.from].scope != model.nodes[edge.to].scope {
            *x = x.max(labels[id].width + 32.);
            *y = y.max(labels[id].height + 24.);
        }
    }
    Ok(())
}

fn border(rect: Rect, kind: Kind, toward: Point) -> Point {
    let center = Point::new(rect.x + rect.width / 2., rect.y + rect.height / 2.);
    let dx = toward.x - center.x;
    let dy = toward.y - center.y;
    if dx == 0. && dy == 0. {
        return Point::new(center.x, rect.y);
    }
    let x = dx / (rect.width / 2.);
    let y = dy / (rect.height / 2.);
    let scale = match kind {
        Kind::Initial | Kind::Final | Kind::History | Kind::DeepHistory | Kind::Junction => {
            1. / (x * x + y * y).sqrt()
        }
        Kind::Choice => 1. / (x.abs() + y.abs()),
        _ => 1. / x.abs().max(y.abs()),
    };
    Point::new(center.x + dx * scale, center.y + dy * scale)
}

struct World {
    nodes: Vec<Rect>,
    scopes: Vec<(Scope, Rect)>,
}

fn draw_boundaries(
    model: &State,
    input: (&Group, &[Label]),
    output: (&mut Scene, &mut Context<'_>),
) -> Result<(), RenderError> {
    let (root, labels) = input;
    let (scene, context) = output;
    if !advanced(model) {
        return Ok(());
    }
    let empty = Rect {
        x: 0.,
        y: 0.,
        width: 0.,
        height: 0.,
    };
    let mut world = World {
        nodes: vec![empty; model.nodes.len()],
        scopes: vec![(
            Scope::Root,
            Rect {
                x: 24.,
                y: 24.,
                width: root.width,
                height: root.height,
            },
        )],
    };
    locate(
        root,
        (model, Point::new(24., 24.)),
        (&mut world, context.budget),
    )?;
    for (id, edge) in model
        .transitions
        .iter()
        .enumerate()
        .filter(|(_, e)| model.nodes[e.from].scope != model.nodes[e.to].scope)
    {
        context.budget.charge(1)?;
        let scope = common_scope(
            model,
            (model.nodes[edge.from].scope, model.nodes[edge.to].scope),
            context.budget,
        )?;
        context.budget.charge(world.scopes.len())?;
        let rect = world
            .scopes
            .iter()
            .rev()
            .find(|(s, _)| *s == scope)
            .unwrap()
            .1;
        let mut route = Vec::new();
        for gutter in [rect.x + 8., rect.x + rect.width - 8.] {
            let source = world.nodes[edge.from];
            let target = world.nodes[edge.to];
            let a = Point::new(gutter, source.y + source.height / 2.);
            let b = Point::new(gutter, target.y + target.height / 2.);
            let points = vec![
                border(source, model.nodes[edge.from].kind, a),
                a,
                b,
                border(target, model.nodes[edge.to].kind, b),
            ];
            if clear_route(
                (&points, &world),
                (model, edge.from, edge.to),
                context.budget,
            )? {
                route = points;
                break;
            }
        }
        if route.is_empty() {
            return Err(
                context.locate_failure(super::parser::error::ParseFailure::unsupported(
                    edge.span.clone(),
                    "state boundary route intersects an unrelated state or concurrent region",
                )),
            );
        }
        let mut element = Element::new(format!("t{id}"), "state-transition");
        element.owner = format!("s{}", edge.from);
        element.target = format!("s{}", edge.to);
        element.name = edge.label.clone();
        element.shapes = arrow::draw(
            route.clone(),
            arrow::Arrow {
                start: arrow::End::None,
                end: arrow::End::Filled,
                dash: Dash::Solid,
            },
        );
        if !edge.label.is_empty() {
            let point = Point::new(
                route[1].x + 6.,
                (route[1].y + route[2].y - labels[id].height) / 2.,
            );
            context.budget.charge(world.nodes.len())?;
            let mut blocked = false;
            for (node, box_rect) in world.nodes.iter().enumerate() {
                if owns(
                    model,
                    Scope::State(node),
                    model.nodes[edge.from].scope,
                    context.budget,
                )? || owns(
                    model,
                    Scope::State(node),
                    model.nodes[edge.to].scope,
                    context.budget,
                )? {
                    continue;
                }
                blocked |= overlaps(
                    Rect {
                        x: point.x,
                        y: point.y,
                        width: labels[id].width,
                        height: labels[id].height,
                    },
                    *box_rect,
                );
            }
            if blocked {
                return Err(context.locate_failure(
                    super::parser::error::ParseFailure::unsupported(
                        edge.span.clone(),
                        "state boundary transition label overlaps a state",
                    ),
                ));
            }
            element
                .shapes
                .push(Shape::Text(point, labels[id].clone(), Decoration::Plain));
        }
        scene.push(element, context.budget)?;
    }
    Ok(())
}

fn locate(
    group: &Group,
    input: (&State, Point),
    output: (&mut World, &mut Budget),
) -> Result<(), RenderError> {
    let (model, offset) = input;
    let (world, budget) = output;
    for node in &group.nodes {
        budget.charge(1)?;
        let rect = Rect {
            x: node.rect.x + offset.x,
            y: node.rect.y + offset.y,
            ..node.rect
        };
        world.nodes[node.id] = rect;
        let caption =
            node.title.height + node.body.iter().map(|l| l.height + 4.).sum::<f64>() + 24.;
        let child = Point::new(rect.x + 12., rect.y + caption);
        if model.nodes[node.id].kind == Kind::Composite {
            world.scopes.push((
                Scope::State(node.id),
                Rect {
                    x: child.x,
                    y: child.y,
                    width: node.children.width,
                    height: node.children.height,
                },
            ));
        }
        locate(&node.children, (model, child), (world, budget))?;
    }
    for (id, rect) in &group.regions {
        budget.charge(1)?;
        world.scopes.push((
            model.regions[*id].scope,
            Rect {
                x: rect.x + offset.x,
                y: rect.y + offset.y,
                ..*rect
            },
        ));
    }
    Ok(())
}

fn common_scope(
    model: &State,
    input: (Scope, Scope),
    budget: &mut Budget,
) -> Result<Scope, RenderError> {
    let (mut left, mut right) = input;
    let mut ancestors = vec![Scope::Root];
    while left != Scope::Root {
        budget.charge(1)?;
        ancestors.push(left);
        left = super::state::parent_scope(model, left);
    }
    loop {
        budget.charge(ancestors.len())?;
        if ancestors.contains(&right) {
            return Ok(right);
        }
        right = super::state::parent_scope(model, right);
    }
}

fn owns(
    model: &State,
    scope: Scope,
    mut child: Scope,
    budget: &mut Budget,
) -> Result<bool, RenderError> {
    loop {
        budget.charge(1)?;
        if child == scope {
            return Ok(true);
        }
        if child == Scope::Root {
            return Ok(false);
        }
        child = super::state::parent_scope(model, child);
    }
}

fn clear_route(
    input: (&[Point], &World),
    endpoints: (&State, usize, usize),
    budget: &mut Budget,
) -> Result<bool, RenderError> {
    let (points, world) = input;
    let (model, from, to) = endpoints;
    for (id, rect) in world.nodes.iter().enumerate() {
        budget.charge(1)?;
        if id == from
            || id == to
            || owns(model, Scope::State(id), model.nodes[from].scope, budget)?
            || owns(model, Scope::State(id), model.nodes[to].scope, budget)?
        {
            continue;
        }
        if points.windows(2).any(|p| crosses((p[0], p[1]), *rect)) {
            return Ok(false);
        }
    }
    for (scope, rect) in &world.scopes {
        if !matches!(scope, Scope::Region(_)) {
            continue;
        }
        if owns(model, *scope, model.nodes[from].scope, budget)?
            || owns(model, *scope, model.nodes[to].scope, budget)?
        {
            continue;
        }
        if points.windows(2).any(|p| crosses((p[0], p[1]), *rect)) {
            return Ok(false);
        }
    }
    Ok(true)
}

fn crosses(segment: (Point, Point), rect: Rect) -> bool {
    let (a, b) = segment;
    if a.x == b.x {
        a.x > rect.x
            && a.x < rect.x + rect.width
            && a.y.min(b.y) < rect.y + rect.height
            && a.y.max(b.y) > rect.y
    } else {
        a.y > rect.y
            && a.y < rect.y + rect.height
            && a.x.min(b.x) < rect.x + rect.width
            && a.x.max(b.x) > rect.x
    }
}

fn overlaps(a: Rect, b: Rect) -> bool {
    a.x < b.x + b.width && a.x + a.width > b.x && a.y < b.y + b.height && a.y + a.height > b.y
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RenderOptions;
    use merman_render::environment::RenderEnvironment;

    fn extension(body: &str, budget: &mut Budget) -> State {
        let document =
            super::super::preprocess::prepare(&format!("@startuml\n{body}\n@enduml"), budget)
                .unwrap();
        let mut parsed = super::super::parser::document::parse(&document, budget).unwrap();
        super::super::parser::preamble::parse(&mut parsed, budget).unwrap();
        super::super::parser::state::parse(parsed.body, budget).unwrap()
    }

    #[test]
    fn should_clip_real_routes_to_advanced_circle_and_diamond_borders() {
        // given
        let mut budget = Budget::new(&RenderOptions::default());
        let model = extension(
            "state C <<choice>>\nstate J <<junction>>\nstate H <<history>>\nC --> J\nC --> H\nJ --> H",
            &mut budget,
        );
        let session = RenderEnvironment::deterministic().begin_session().unwrap();
        let mut context = Context::new(&session, &mut budget);
        let labels = model
            .transitions
            .iter()
            .map(|e| context.label(&e.label).unwrap())
            .collect::<Vec<_>>();
        // when
        let group = compose(&model, (Scope::Root, &labels), &mut context).unwrap();
        // then
        for (id, points, _) in &group.edges {
            let edge = &model.transitions[*id];
            for (node, point) in [(edge.from, points[0]), (edge.to, *points.last().unwrap())] {
                let rect = group.nodes.iter().find(|n| n.id == node).unwrap().rect;
                let x = (point.x - rect.x - rect.width / 2.) / (rect.width / 2.);
                let y = (point.y - rect.y - rect.height / 2.) / (rect.height / 2.);
                let equation = if model.nodes[node].kind == Kind::Choice {
                    x.abs() + y.abs()
                } else {
                    x * x + y * y
                };
                assert!((equation - 1.).abs() < 1e-9, "node{node}: {point:?}");
            }
        }
    }

    #[test]
    fn should_compose_measured_regions_on_first_axis_and_contain_large_font_history() {
        // given
        for (first, second, axis) in [("--", "||", Axis::Stacked), ("||", "--", Axis::SideBySide)] {
            for size in [6, 14, 96] {
                let mut budget = Budget::new(&RenderOptions::default());
                let model = extension(
                    &format!(
                        "state S {{\nstate A\nA --> [H]\n{first}\nstate B\nB --> [H*]\n{second}\n}}\n"
                    ),
                    &mut budget,
                );
                let session = RenderEnvironment::deterministic().begin_session().unwrap();
                let mut fonts = super::super::style::Preamble::default().fonts;
                for font in &mut fonts {
                    font.size = size;
                }
                let mut context = Context::new(&session, &mut budget).with_fonts(fonts);
                let labels = model
                    .transitions
                    .iter()
                    .map(|e| context.label(&e.label).unwrap())
                    .collect::<Vec<_>>();
                // when
                let group = compose(&model, (Scope::Root, &labels), &mut context).unwrap();
                // then
                let child = &group.nodes[0].children;
                assert_eq!(child.regions.len(), 3);
                for pair in child.regions.windows(2) {
                    if axis == Axis::Stacked {
                        assert!(pair[1].1.y >= pair[0].1.y + pair[0].1.height);
                        assert_eq!(pair[1].1.x, 0.);
                    } else {
                        assert!(pair[1].1.x >= pair[0].1.x + pair[0].1.width);
                        assert_eq!(pair[1].1.y, 0.);
                    }
                }
                for node in &child.nodes {
                    let region = child
                        .regions
                        .iter()
                        .find(|(id, _)| model.regions[*id].scope == model.nodes[node.id].scope)
                        .unwrap()
                        .1;
                    assert!(node.rect.x >= region.x && node.rect.y >= region.y);
                    assert!(
                        node.rect.x + node.rect.width <= region.x + region.width
                            && node.rect.y + node.rect.height <= region.y + region.height
                    );
                    if matches!(model.nodes[node.id].kind, Kind::History | Kind::DeepHistory) {
                        assert!(
                            node.title.width <= node.rect.width
                                && node.title.height <= node.rect.height
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn should_cancel_during_independent_region_layout() {
        // given
        let mut budget = Budget::new(&RenderOptions::default());
        let model = extension("state S {\nA --> B\n--\nC --> D\n}", &mut budget);
        let session = RenderEnvironment::deterministic().begin_session().unwrap();
        let mut context = Context::new(&session, &mut budget);
        let labels = model
            .transitions
            .iter()
            .map(|edge| context.label(&edge.label).unwrap())
            .collect::<Vec<_>>();
        let group = compose(&model, (Scope::Root, &labels), &mut context).unwrap();
        let children = group.nodes.into_iter().next().unwrap().children;
        let nodes = children
            .nodes
            .into_iter()
            .filter(|node| model.nodes[node.id].scope == Scope::Region(1))
            .collect::<Vec<_>>();
        assert_eq!(nodes.len(), 2);
        context.budget.control.cancel_after_checkpoints(12);
        // when
        let result = arrange(&model, (Scope::Region(1), nodes, &labels), context.budget);
        // then
        assert!(matches!(result, Err(RenderError::Cancelled(_))));
    }

    #[test]
    fn should_contain_nested_state_descriptions_children_and_pseudostates() {
        // given
        let source = "@startuml\nstate Outer {\nOuter : entry / prepare\\nsecond row\nstate Inner {\nInner : exit / finish\n[*] --> A\nA --> [*]\n}\n[*] --> Inner\n}\n[*] --> Outer\n@enduml";
        let mut budget = Budget::new(&RenderOptions::default());
        let document = super::super::preprocess::prepare(source, &mut budget).unwrap();
        let mut parsed = super::super::parser::document::parse(&document, &mut budget).unwrap();
        super::super::parser::preamble::parse(&mut parsed, &mut budget).unwrap();
        let model = super::super::parser::state::parse(parsed.body, &mut budget).unwrap();
        let session = RenderEnvironment::deterministic().begin_session().unwrap();
        // when
        let scene = layout(&model, &mut Context::new(&session, &mut budget)).unwrap();
        // then
        for node in scene
            .elements
            .iter()
            .filter(|e| e.kind != "state-transition")
        {
            let own = match node.shapes[0] {
                Shape::Rect(r, _, _) | Shape::Ellipse(r, _) => r,
                _ => panic!("state boundary"),
            };
            if let Some(parent) = scene.elements.iter().find(|e| e.id == node.owner) {
                let Shape::Rect(bounds, _, _) = parent.shapes[0] else {
                    panic!("composite boundary");
                };
                assert!(
                    own.x >= bounds.x
                        && own.y >= bounds.y
                        && own.x + own.width <= bounds.x + bounds.width
                        && own.y + own.height <= bounds.y + bounds.height
                );
            }
            for shape in &node.shapes {
                if let Shape::Text(point, label, _) = shape {
                    assert!(
                        point.x >= own.x
                            && point.y >= own.y
                            && point.x + label.width <= own.x + own.width
                            && point.y + label.height <= own.y + own.height
                    );
                }
            }
        }
    }

    #[test]
    fn should_interrupt_inside_checked_state_graph_layout() {
        // given
        let mut options = RenderOptions::default();
        options.limits.layout_work = 5;
        let mut budget = Budget::new(&options);
        let model = State {
            nodes: vec![],
            transitions: vec![],
            regions: vec![],
        };
        let label = Label {
            lines: Vec::new().into(),
            width: 0.,
            height: 0.,
        };
        let nodes = vec![BoxNode {
            id: 0,
            rect: Rect {
                x: 0.,
                y: 0.,
                width: 40.,
                height: 40.,
            },
            title: label,
            body: vec![],
            children: Group {
                nodes: vec![],
                edges: vec![],
                width: 0.,
                height: 0.,
                regions: vec![],
            },
        }];
        // when
        let result = arrange(&model, (Scope::Root, nodes, &[]), &mut budget);
        // then
        assert!(result.is_err());
        assert!(
            budget.work > 1,
            "interruption happened before graph construction finished"
        );
    }
}
