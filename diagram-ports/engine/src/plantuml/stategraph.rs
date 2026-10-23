//! Original checked, recursive Dagre adapter for typed state ownership (not UML entities).
use super::{
    arrow,
    budget::{Budget, invalid},
    scene::{Dash, Decoration, Element, Paint, Point, Rect, Scene, Shape},
    state::{Kind, Scope, State},
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
}

pub(super) fn layout(model: &State, context: &mut Context<'_>) -> Result<Scene, RenderError> {
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
        let title = context.label(&node.label)?;
        let body = node
            .descriptions
            .iter()
            .map(|s| context.label(s))
            .collect::<Result<Vec<_>, _>>()?;
        let children = if node.kind == Kind::Composite {
            compose(model, (Scope::State(id), labels), context)?
        } else {
            Group {
                nodes: vec![],
                edges: vec![],
                width: 0.,
                height: 0.,
            }
        };
        let caption_width = body.iter().map(|l| l.width).fold(title.width, f64::max) + 32.;
        let caption_height = title.height + body.iter().map(|l| l.height + 4.).sum::<f64>() + 24.;
        let (width, height) = if matches!(node.kind, Kind::Initial | Kind::Final) {
            (24., 24.)
        } else {
            (
                caption_width.max(children.width + 24.).max(80.),
                caption_height + children.height,
            )
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

fn arrange(
    model: &State,
    input: (Scope, Vec<BoxNode>, &[Label]),
    budget: &mut Budget,
) -> Result<Group, RenderError> {
    let (scope, mut nodes, labels) = input;
    let mut graph = Graph::new(GraphOptions {
        directed: true,
        multigraph: true,
        compound: false,
    });
    graph.set_graph(GraphLabel {
        nodesep: 60.,
        ranksep: 70.,
        marginx: 24.,
        marginy: 24.,
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
            return Err(invalid(
                1,
                "state transition across nested ownership is not supported",
            ));
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
    let mut work = Work {
        budget,
        failure: vec![],
    };
    let result = dugong::layout_controlled(&mut graph, &mut work);
    if let Some(error) = work.failure.pop() {
        return Err(error);
    }
    result.map_err(|e| invalid(1, format!("state graph layout: {e}")))?;
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
        .filter(|(_, e)| model.nodes[e.from].scope == scope)
    {
        budget.charge(1)?;
        let placed = graph
            .edge(
                &format!("s{}", edge.from),
                &format!("s{}", edge.to),
                Some(&format!("t{id}")),
            )
            .ok_or_else(|| invalid(1, "missing state route"))?;
        let points = placed.points.iter().map(|p| Point::new(p.x, p.y)).collect();
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
            },
        );
        element.name = data.name.clone();
        if let Scope::State(owner) = data.scope {
            element.owner = format!("s{owner}");
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RenderOptions;
    use merman_render::environment::RenderEnvironment;

    #[test]
    fn should_contain_nested_state_descriptions_children_and_pseudostates() {
        // given
        let source = "@startuml\nstate Outer {\nOuter : entry / prepare\\nsecond row\nstate Inner {\nInner : exit / finish\n[*] --> A\nA --> [*]\n}\n[*] --> Inner\n}\n[*] --> Outer\n@enduml";
        let mut budget = Budget::new(&RenderOptions::default());
        let lines = super::super::source::prepare(source, &mut budget).unwrap();
        let model = super::super::state::parse(&lines, &mut budget).unwrap();
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
        };
        let label = Label {
            font_family: "sans-serif",
            lines: vec![],
            width: 0.,
            height: 0.,
            line_height: 18.,
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
