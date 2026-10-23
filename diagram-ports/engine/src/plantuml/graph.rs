//! Checked native Dagre adapter. No PlantUML/Graphviz layout implementation is translated.
use super::{
    arrow,
    budget::{Budget, invalid},
    scene::{Dash, Decoration, Element, Paint, Point, Rect, Scene, Shape, text_at},
    text::{Context, Label},
    uml::{Direction, Kind, Member, Parent, Uml},
};
use crate::RenderError;
use dugong::{
    EdgeLabel, GraphLabel, NodeLabel, RankDir,
    graphlib::{Graph, GraphOptions},
};

enum Failure {
    None,
    Error(RenderError),
}

struct Work<'a> {
    budget: &'a mut Budget,
    failure: Failure,
}
impl dugong::WorkControl for Work<'_> {
    fn charge(&mut self, units: usize) -> Result<(), dugong::WorkError> {
        match self.budget.charge(units) {
            Ok(()) => Ok(()),
            Err(error) => {
                self.failure = Failure::Error(error);
                Err(dugong::WorkError::Interrupted)
            }
        }
    }
}
struct Measured {
    title: Label,
    members: Vec<(Label, Decoration)>,
    width: f64,
    height: f64,
}

struct RelationLabels {
    middle: Label,
    left: Label,
    right: Label,
}

fn measure_relations(
    model: &Uml,
    context: &mut Context<'_>,
) -> Result<Vec<RelationLabels>, RenderError> {
    model
        .relations
        .iter()
        .map(|relation| {
            Ok(RelationLabels {
                middle: context.label(&relation.label)?,
                left: context.label(&relation.left)?,
                right: context.label(&relation.right)?,
            })
        })
        .collect()
}

pub(super) fn layout(model: &Uml, context: &mut Context<'_>) -> Result<Scene, RenderError> {
    let measured = measure(model, context)?;
    let labels = measure_relations(model, context)?;
    let mut graph = build_graph(model, &measured, &labels, context.budget)?;
    let mut work = Work {
        budget: context.budget,
        failure: Failure::None,
    };
    let result = dugong::layout_controlled(&mut graph, &mut work);
    if let Failure::Error(error) = work.failure {
        return Err(error);
    }
    result.map_err(|e| RenderError::Backend(format!("PlantUML Dagre layout: {e}")))?;
    let mut scene = Scene::new();
    draw_nodes(model, &measured, &graph, &mut scene, context)?;
    draw_edges(model, &labels, &graph, &mut scene, context)?;
    Ok(scene)
}

fn measure(model: &Uml, context: &mut Context<'_>) -> Result<Vec<Measured>, RenderError> {
    let mut result = Vec::new();
    for entity in &model.entities {
        let title = context.styled_label(
            &entity.label,
            match entity.kind {
                Kind::Abstract => Decoration::Italic,
                Kind::Object => Decoration::Underlined,
                _ => Decoration::Plain,
            },
        )?;
        let mut members = Vec::new();
        let mut width = title.width + 36.;
        let mut height = title.height + 24.;
        for member in &entity.members {
            let (label, decoration) = match member {
                Member::Text(text, style) => (context.styled_label(text, *style)?, *style),
                Member::Separator => (context.label("")?, Decoration::Plain),
            };
            width = width.max(label.width + 28.);
            height += label.height + 6.;
            members.push((label, decoration));
        }
        if entity.kind == Kind::Actor {
            height += 58.;
        }
        if entity.kind == Kind::Database {
            height += 24.;
        }
        if matches!(entity.kind, Kind::Interface | Kind::Enum | Kind::Annotation) {
            height += 22.;
        }
        result.push(Measured {
            title,
            members,
            width: width.max(70.),
            height: height.max(45.),
        });
    }
    Ok(result)
}

fn build_graph(
    model: &Uml,
    nodes: &[Measured],
    labels: &[RelationLabels],
    budget: &mut Budget,
) -> Result<Graph<NodeLabel, EdgeLabel, GraphLabel>, RenderError> {
    let mut graph = Graph::new(GraphOptions {
        directed: true,
        multigraph: true,
        compound: true,
    });
    let ranksep = model
        .entities
        .iter()
        .zip(nodes)
        .filter(|(e, _)| e.container)
        .map(|(_, n)| 2. * (n.title.height + 40.))
        .fold(75., f64::max);
    graph.set_graph(GraphLabel {
        rankdir: match model.direction {
            Direction::Down => RankDir::TB,
            Direction::Right => RankDir::LR,
        },
        nodesep: ranksep,
        ranksep,
        marginx: 60.,
        marginy: 60.,
        ..Default::default()
    });
    for (i, node) in nodes.iter().enumerate() {
        budget.charge(1)?;
        graph.set_node(
            format!("n{i}"),
            NodeLabel {
                width: node.width,
                height: node.height,
                ..Default::default()
            },
        );
    }
    let mut parents = Vec::new();
    for (i, node) in model.entities.iter().enumerate() {
        if node.container {
            budget.charge(1)?;
            let caption = format!("caption-{i}");
            graph.set_node(
                &caption,
                NodeLabel {
                    width: nodes[i].title.width + 36.,
                    height: nodes[i].title.height + 40.,
                    ..Default::default()
                },
            );
            parents.push((
                graph.node_ix(&caption).unwrap(),
                graph.node_ix(&format!("n{i}")).unwrap(),
            ));
        }
        if let Parent::Node(parent) = node.parent {
            parents.push((
                graph.node_ix(&format!("n{i}")).unwrap(),
                graph
                    .node_ix(&format!("n{parent}"))
                    .ok_or_else(|| invalid(1, "invalid parent ID"))?,
            ));
        }
    }
    budget.charge(
        (nodes.len() + parents.len()).saturating_mul(nodes.len().max(2).ilog2() as usize + 16),
    )?;
    graph
        .try_set_unparented_parents_ix(&parents)
        .map_err(|e| invalid(1, format!("invalid compound graph: {e}")))?;
    for (i, relation) in model.relations.iter().enumerate() {
        budget.charge(1)?;
        let (width, height) = relation_space(&labels[i], model.direction);
        graph
            .try_set_edge_named(
                format!("n{}", relation.from),
                format!("n{}", relation.to),
                Some(format!("r{i}")),
                Some(EdgeLabel {
                    width,
                    height,
                    ..Default::default()
                }),
            )
            .map_err(|e| invalid(1, format!("invalid graph relation: {e}")))?;
    }
    Ok(graph)
}

fn relation_space(labels: &RelationLabels, direction: Direction) -> (f64, f64) {
    let sizes = [&labels.middle, &labels.left, &labels.right].map(|label| {
        if label.lines.iter().all(String::is_empty) {
            (0., 0.)
        } else {
            (label.width, label.height)
        }
    });
    match direction {
        Direction::Down => (
            sizes.iter().map(|s| s.0).fold(0., f64::max) + 32.,
            sizes.iter().map(|s| s.1).sum::<f64>() + 48.,
        ),
        Direction::Right => (
            sizes.iter().map(|s| s.0).sum::<f64>() + 48.,
            sizes.iter().map(|s| s.1).fold(0., f64::max) + 32.,
        ),
    }
}

fn bounds(graph: &Graph<NodeLabel, EdgeLabel, GraphLabel>, id: usize) -> Result<Rect, RenderError> {
    let node = graph
        .node(&format!("n{id}"))
        .ok_or_else(|| invalid(1, "layout omitted entity"))?;
    let x = node.x.ok_or_else(|| invalid(1, "layout omitted node x"))?;
    let y = node.y.ok_or_else(|| invalid(1, "layout omitted node y"))?;
    Ok(Rect {
        x: x - node.width / 2.,
        y: y - node.height / 2.,
        width: node.width,
        height: node.height,
    })
}

fn draw_nodes(
    model: &Uml,
    nodes: &[Measured],
    graph: &Graph<NodeLabel, EdgeLabel, GraphLabel>,
    scene: &mut Scene,
    context: &mut Context<'_>,
) -> Result<(), RenderError> {
    for (id, entity) in model.entities.iter().enumerate() {
        let rect = bounds(graph, id)?;
        let mut element = Element::new(format!("n{id}"), entity.kind.name());
        context.budget.text(entity.key.len())?;
        element.name = entity.key.clone();
        if let Parent::Node(parent) = entity.parent {
            element.owner = format!("n{parent}");
        }
        element.shapes = super::shapes::node(rect, entity.kind.name());
        let header_y = rect.y
            + if matches!(
                entity.kind,
                Kind::Package | Kind::Namespace | Kind::Database
            ) {
                18.
            } else if entity.kind == Kind::Actor {
                58.
            } else {
                8.
            };
        let title_rect = Rect {
            x: rect.x + 12.,
            y: header_y,
            width: rect.width - 24.,
            height: nodes[id].title.height + 8.,
        };
        let mut title = text_at(title_rect, nodes[id].title.clone());
        if let Shape::Text(_, _, ref mut decoration) = title {
            *decoration = match entity.kind {
                Kind::Object => Decoration::Underlined,
                Kind::Abstract => Decoration::Italic,
                _ => Decoration::Plain,
            };
        }
        element.shapes.push(title);
        let mut y = header_y + nodes[id].title.height + 18.;
        if matches!(entity.kind, Kind::Interface | Kind::Enum | Kind::Annotation) {
            let marker = context.label(&format!("«{}»", entity.kind.name()))?;
            element.shapes.push(Shape::Text(
                Point::new(rect.x + 12., y),
                marker,
                Decoration::Plain,
            ));
            y += 22.;
        }
        if !nodes[id].members.is_empty() {
            element.shapes.push(Shape::Line(
                vec![
                    Point::new(rect.x, y - 5.),
                    Point::new(rect.x + rect.width, y - 5.),
                ],
                Dash::Solid,
            ));
        }
        for (label, decoration) in &nodes[id].members {
            if label.lines.iter().all(String::is_empty) {
                element.shapes.push(Shape::Line(
                    vec![
                        Point::new(rect.x, y + 4.),
                        Point::new(rect.x + rect.width, y + 4.),
                    ],
                    Dash::Solid,
                ));
            } else {
                element.shapes.push(Shape::Text(
                    Point::new(rect.x + 12., y),
                    label.clone(),
                    *decoration,
                ));
            }
            y += label.height + 6.;
        }
        scene.push(element, context.budget)?;
    }
    Ok(())
}

fn draw_edges(
    model: &Uml,
    labels: &[RelationLabels],
    graph: &Graph<NodeLabel, EdgeLabel, GraphLabel>,
    scene: &mut Scene,
    context: &mut Context<'_>,
) -> Result<(), RenderError> {
    for (id, relation) in model.relations.iter().enumerate() {
        let edge = graph
            .edge(
                &format!("n{}", relation.from),
                &format!("n{}", relation.to),
                Some(&format!("r{id}")),
            )
            .ok_or_else(|| invalid(1, "layout omitted relation"))?;
        let mut points: Vec<_> = edge.points.iter().map(|p| Point::new(p.x, p.y)).collect();
        if points.len() < 2 {
            return Err(invalid(1, "layout relation has no route"));
        }
        points[0] = intersect(
            model.entities[relation.from].kind,
            bounds(graph, relation.from)?,
            points[0],
        );
        let end = points.len() - 1;
        points[end] = intersect(
            model.entities[relation.to].kind,
            bounds(graph, relation.to)?,
            points[end],
        );
        let first = points[0];
        let last = points[points.len() - 1];
        let near_first = points[1];
        let near_last = points[points.len() - 2];
        let middle = &labels[id].middle;
        let mut element = Element::new(format!("r{id}"), "relation");
        element.owner = format!("n{}", relation.from);
        element.target = format!("n{}", relation.to);
        element.shapes = arrow::draw(points, relation.arrow);
        if !relation.label.is_empty() {
            let x = edge.x.ok_or_else(|| invalid(1, "layout omitted label x"))?;
            let y = edge.y.ok_or_else(|| invalid(1, "layout omitted label y"))?;
            element.shapes.push(Shape::Rect(
                Rect {
                    x: x - middle.width / 2. - 3.,
                    y: y - middle.height / 2. - 2.,
                    width: middle.width + 6.,
                    height: middle.height + 4.,
                },
                0.,
                Paint::Paper,
            ));
            element.shapes.push(Shape::Text(
                Point::new(x - middle.width / 2., y - middle.height / 2.),
                middle.clone(),
                Decoration::Plain,
            ));
        }
        for (point, neighbor, label, owner) in [
            (first, near_first, &labels[id].left, relation.from),
            (last, near_last, &labels[id].right, relation.to),
        ] {
            if label.lines.iter().any(|line| !line.is_empty()) {
                let rect = endpoint_label_box((point, neighbor), label, bounds(graph, owner)?);
                element.shapes.push(Shape::Rect(rect, 0., Paint::Paper));
                element.shapes.push(text_at(rect, label.clone()));
            }
        }
        scene.push(element, context.budget)?;
    }
    Ok(())
}

fn endpoint_label_box(route: (Point, Point), label: &Label, owner: Rect) -> Rect {
    let (point, neighbor) = route;
    let dx = neighbor.x - point.x;
    let dy = neighbor.y - point.y;
    let length = dx.hypot(dy).max(1e-9);
    let (ux, uy) = (dx / length, dy / length);
    let along = ux.abs() * label.width / 2. + uy.abs() * label.height / 2. + 16.;
    let normal = uy.abs() * label.width / 2. + ux.abs() * label.height / 2. + 8.;
    let rect = Rect {
        x: point.x + ux * along - uy * normal - label.width / 2. - 3.,
        y: point.y + uy * along + ux * normal - label.height / 2. - 3.,
        width: label.width + 6.,
        height: label.height + 6.,
    };
    if rect.x + rect.width <= owner.x
        || rect.x >= owner.x + owner.width
        || rect.y + rect.height <= owner.y
        || rect.y >= owner.y + owner.height
    {
        return rect;
    }
    if dx.abs() > dy.abs() {
        return Rect {
            x: if dx < 0. {
                owner.x - rect.width - 16.
            } else {
                owner.x + owner.width + 16.
            },
            ..rect
        };
    }
    Rect {
        y: if dy < 0. {
            owner.y - rect.height - 16.
        } else {
            owner.y + owner.height + 16.
        },
        ..rect
    }
}

fn intersect(kind: Kind, rect: Rect, point: Point) -> Point {
    if kind != Kind::Usecase {
        return point;
    }
    let cx = rect.x + rect.width / 2.;
    let cy = rect.y + rect.height / 2.;
    let dx = point.x - cx;
    let dy = point.y - cy;
    let scale = ((dx / (rect.width / 2.)).powi(2) + (dy / (rect.height / 2.)).powi(2)).sqrt();
    if scale > 0. {
        return Point::new(cx + dx / scale, cy + dy / scale);
    }
    point
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RenderOptions;
    use merman_render::environment::RenderEnvironment;

    fn draw(source: &str) -> Scene {
        let mut budget = Budget::new(&RenderOptions::default());
        let lines = super::super::source::prepare(source, &mut budget).unwrap();
        let model = super::super::uml::parse(&lines, &mut budget).unwrap();
        let session = RenderEnvironment::deterministic().begin_session().unwrap();
        layout(&model, &mut Context::new(&session, &mut budget)).unwrap()
    }

    #[test]
    fn should_contain_long_multiline_compound_captions_in_both_directions() {
        // given
        for direction in ["top to bottom direction", "left to right direction"] {
            let source = format!(
                "@startuml\n{direction}\npackage \"Extremely long container caption that must stay within its box\\nA second line\\nA third line\" as P {{\nclass A\n}}\n@enduml"
            );
            // when
            let scene = draw(&source);
            let parent = scene.elements.iter().find(|e| e.id == "n0").unwrap();
            let Shape::Rect(box_rect, _, _) = parent.shapes[0] else {
                panic!("package box")
            };
            // then
            for shape in &parent.shapes {
                if let Shape::Text(point, label, _) = shape {
                    assert!(
                        point.x >= box_rect.x
                            && point.x + label.width <= box_rect.x + box_rect.width,
                        "caption exceeds compound width: {direction}"
                    );
                    assert!(
                        point.y >= box_rect.y
                            && point.y + label.height <= box_rect.y + box_rect.height,
                        "caption exceeds compound height: {direction}"
                    );
                }
            }
        }
    }

    #[test]
    fn should_place_endpoint_labels_outside_nodes_for_tb_lr_and_reverse_routes() {
        // given
        for direction in ["top to bottom direction", "left to right direction"] {
            for arrow in ["-->", "<--"] {
                let source = format!(
                    "@startuml\n{direction}\nclass A\nclass TargetName\nA \"one source\" {arrow} \"many targets\" TargetName : relation\n@enduml"
                );
                // when
                let scene = draw(&source);
                let boxes = scene
                    .elements
                    .iter()
                    .filter(|e| e.kind == "class")
                    .map(|e| match e.shapes[0] {
                        Shape::Rect(rect, _, _) => rect,
                        _ => panic!("class box"),
                    })
                    .collect::<Vec<_>>();
                let edge = scene
                    .elements
                    .iter()
                    .find(|e| e.kind == "relation")
                    .unwrap();
                // then
                for shape in &edge.shapes {
                    if let Shape::Text(point, label, _) = shape {
                        for rect in &boxes {
                            assert!(
                                point.x + label.width <= rect.x
                                    || point.x >= rect.x + rect.width
                                    || point.y + label.height <= rect.y
                                    || point.y >= rect.y + rect.height,
                                "edge label {:?} overlaps a node: {direction} {arrow}",
                                label.lines
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn should_intersect_usecase_ellipse_instead_of_its_bounding_rectangle() {
        // given
        let rect = Rect {
            x: 0.,
            y: 0.,
            width: 100.,
            height: 60.,
        };
        // when
        let point = intersect(Kind::Usecase, rect, Point::new(100., 60.));
        // then
        let ellipse = ((point.x - 50.) / 50.).powi(2) + ((point.y - 30.) / 30.).powi(2);
        assert!((ellipse - 1.).abs() < 1e-8);
        assert!(point.x < 100. && point.y < 60.);
    }

    #[test]
    fn should_keep_diagonal_endpoint_label_outside_a_wide_owning_node() {
        // given
        let owner = Rect {
            x: 0.,
            y: 0.,
            width: 500.,
            height: 100.,
        };
        let label = Label {
            font_family: "sans-serif",
            lines: vec!["wide role".into()],
            width: 160.,
            height: 20.,
            line_height: 20.,
        };
        // when
        let rect = endpoint_label_box(
            (Point::new(250., 100.), Point::new(200., 150.)),
            &label,
            owner,
        );
        // then
        assert!(
            rect.x + rect.width <= owner.x
                || rect.x >= owner.x + owner.width
                || rect.y >= owner.y + owner.height
                || rect.y + rect.height <= owner.y
        );
    }

    fn model(budget: &mut Budget) -> Uml {
        let lines = super::super::source::prepare("@startuml\npackage P {\nclass A\nclass B\nA --> B\nB --> A\nA --> A\nA --> B : second\n}\nclass Alone\n@enduml", budget).unwrap();
        super::super::uml::parse(&lines, budget).unwrap()
    }

    #[test]
    fn should_reject_parent_cycles_without_panicking() {
        // given
        let mut budget = Budget::new(&RenderOptions::default());
        let mut model = model(&mut budget);
        model.entities[0].parent = Parent::Node(1);
        let session = RenderEnvironment::deterministic().begin_session().unwrap();
        let mut context = Context::new(&session, &mut budget);
        let nodes = measure(&model, &mut context).unwrap();
        let labels = measure_relations(&model, &mut context).unwrap();
        // when
        let result = build_graph(&model, &nodes, &labels, context.budget);
        // then
        assert!(result.is_err());
    }

    #[test]
    fn should_interrupt_inside_dagre_after_checked_graph_construction() {
        // given
        let mut budget = Budget::new(&RenderOptions::default());
        let model = model(&mut budget);
        let session = RenderEnvironment::deterministic().begin_session().unwrap();
        let mut context = Context::new(&session, &mut budget);
        let nodes = measure(&model, &mut context).unwrap();
        let labels = measure_relations(&model, &mut context).unwrap();
        let mut graph = build_graph(&model, &nodes, &labels, context.budget).unwrap();
        context.budget.limits.layout_work = context.budget.work + 1;
        let mut work = Work {
            budget: context.budget,
            failure: Failure::None,
        };
        // when
        let result = dugong::layout_controlled(&mut graph, &mut work);
        // then
        assert!(result.is_err());
        assert!(matches!(
            work.failure,
            Failure::Error(RenderError::Backend(_))
        ));
    }
}
