//! Original measured Chen geometry over the shared checked graph service.
use super::{
    budget::{Budget, invalid},
    chen::{Chen, ChenNode, Link, Participation},
    scene::{Dash, Decoration, Element, Paint, Point, Rect, Scene, Shape, text_at},
    text::{Context, Label},
};
use crate::RenderError;
use dugong::{
    EdgeLabel, GraphLabel, NodeLabel,
    graphlib::{Graph, GraphOptions},
};

type LayoutGraph = Graph<NodeLabel, EdgeLabel, GraphLabel>;
struct Measured {
    label: Label,
    width: f64,
    height: f64,
}
struct Measurements {
    nodes: Vec<Measured>,
    links: Vec<Label>,
}
struct Drawing<'a> {
    model: &'a Chen,
    measured: &'a Measurements,
    graph: &'a LayoutGraph,
}

pub(super) fn layout(model: &Chen, context: &mut Context<'_>) -> Result<Scene, RenderError> {
    let nodes = measure_nodes(model, context)?;
    let labels = model
        .links
        .iter()
        .map(|link| context.label(link.cardinality()))
        .collect::<Result<Vec<_>, _>>()?;
    let measured = Measurements {
        nodes,
        links: labels,
    };
    let mut graph = build_graph(model, &measured, context.budget)?;
    super::graph::layout_checked(&mut graph, context.budget)?;
    let mut scene = Scene::new();
    let drawing = Drawing {
        model,
        measured: &measured,
        graph: &graph,
    };
    drawing.draw_links(&mut scene, context.budget)?;
    drawing.draw_nodes(&mut scene, context.budget)?;
    Ok(scene)
}

fn measure_nodes(model: &Chen, context: &mut Context<'_>) -> Result<Vec<Measured>, RenderError> {
    let mut result = Vec::new();
    for node in &model.nodes {
        let label = context.label(&node.display)?;
        let (width, height) = match node.kind {
            ChenNode::Entity => ((label.width + 28.).max(70.), (label.height + 24.).max(45.)),
            ChenNode::Relationship => {
                let width = (label.width + 2. * label.height + 40.).max(90.);
                (width, width / 2.)
            }
            ChenNode::Attribute(_) => (
                (label.width + 24.) * std::f64::consts::SQRT_2,
                (label.height + 16.) * std::f64::consts::SQRT_2,
            ),
        };
        result.push(Measured {
            label,
            width,
            height,
        });
    }
    Ok(result)
}

fn build_graph(
    model: &Chen,
    measured: &Measurements,
    budget: &mut Budget,
) -> Result<LayoutGraph, RenderError> {
    let mut graph = Graph::new(GraphOptions {
        directed: true,
        multigraph: true,
        compound: false,
    });
    graph.set_graph(GraphLabel {
        nodesep: 60.,
        ranksep: 80.,
        marginx: 32.,
        marginy: 32.,
        ..Default::default()
    });
    for (id, node) in measured.nodes.iter().enumerate() {
        budget.charge(1)?;
        graph.set_node(
            format!("c{id}"),
            NodeLabel {
                width: node.width,
                height: node.height,
                ..Default::default()
            },
        );
    }
    for (id, link) in model.links.iter().enumerate() {
        budget.charge(1)?;
        let (from, to) = link.endpoints();
        graph
            .try_set_edge_named(
                format!("c{from}"),
                format!("c{to}"),
                Some(format!("cl{id}")),
                Some(EdgeLabel {
                    width: measured.links[id].width + 24.,
                    height: measured.links[id].height + 16.,
                    ..Default::default()
                }),
            )
            .map_err(|error| invalid(1, format!("invalid Chen graph edge {id}: {error}")))?;
    }
    Ok(graph)
}

fn bounds(graph: &LayoutGraph, id: usize) -> Result<Rect, RenderError> {
    let node = graph
        .node(&format!("c{id}"))
        .ok_or_else(|| invalid(1, format!("layout omitted Chen node {id}")))?;
    let x = node
        .x
        .ok_or_else(|| invalid(1, format!("layout omitted Chen node {id} x")))?;
    let y = node
        .y
        .ok_or_else(|| invalid(1, format!("layout omitted Chen node {id} y")))?;
    Ok(Rect {
        x: x - node.width / 2.,
        y: y - node.height / 2.,
        width: node.width,
        height: node.height,
    })
}

impl Drawing<'_> {
    fn draw_nodes(&self, scene: &mut Scene, budget: &mut Budget) -> Result<(), RenderError> {
        for (id, node) in self.model.nodes.iter().enumerate() {
            budget.charge(1)?;
            let rect = bounds(self.graph, id)?;
            let kind = match node.kind {
                ChenNode::Entity => "chen-entity",
                ChenNode::Relationship => "chen-relationship",
                ChenNode::Attribute(_) => "chen-attribute",
            };
            let mut element = Element::new(format!("c{id}"), kind);
            budget.text(node.key.len())?;
            element.name = node.key.clone();
            if let ChenNode::Attribute(owner) = node.kind {
                element.owner = format!("c{}", owner.id());
            }
            element.shapes.push(match node.kind {
                ChenNode::Entity => Shape::Rect(rect, 0., Paint::Node),
                ChenNode::Attribute(_) => Shape::Ellipse(rect, Paint::Node),
                ChenNode::Relationship => diamond(rect),
            });
            element
                .shapes
                .push(text_at(rect, self.measured.nodes[id].label.clone()));
            scene.push(element, budget)?;
        }
        Ok(())
    }
}

fn diamond(rect: Rect) -> Shape {
    Shape::Polygon(
        vec![
            Point::new(rect.x, rect.y + rect.height / 2.),
            Point::new(rect.x + rect.width / 2., rect.y),
            Point::new(rect.x + rect.width, rect.y + rect.height / 2.),
            Point::new(rect.x + rect.width / 2., rect.y + rect.height),
        ],
        Paint::Node,
    )
}

impl Drawing<'_> {
    fn draw_links(&self, scene: &mut Scene, budget: &mut Budget) -> Result<(), RenderError> {
        for id in 0..self.model.links.len() {
            budget.charge(1)?;
            scene.push(self.link_element(id, budget)?, budget)?;
        }
        Ok(())
    }

    fn edge(&self, id: usize) -> Result<&EdgeLabel, RenderError> {
        let link = &self.model.links[id];
        let (from, to) = link.endpoints();
        self.graph
            .edge(
                &format!("c{from}"),
                &format!("c{to}"),
                Some(&format!("cl{id}")),
            )
            .ok_or_else(|| invalid(1, format!("layout omitted Chen link {id}")))
    }

    fn route(&self, id: usize, budget: &mut Budget) -> Result<Vec<Point>, RenderError> {
        let (from, to) = self.model.links[id].endpoints();
        let edge = self.edge(id)?;
        budget.charge(edge.points.len())?;
        if edge.points.len() < 2 {
            return Err(invalid(1, format!("Chen link {id} has no route")));
        }
        let mut points = edge
            .points
            .iter()
            .map(|p| Point::new(p.x, p.y))
            .collect::<Vec<_>>();
        points[0] = intersect(
            self.model.nodes[from].kind,
            bounds(self.graph, from)?,
            points[0],
        );
        let end = points.len() - 1;
        points[end] = intersect(
            self.model.nodes[to].kind,
            bounds(self.graph, to)?,
            points[end],
        );
        Ok(points)
    }

    fn link_element(&self, id: usize, budget: &mut Budget) -> Result<Element, RenderError> {
        let link = &self.model.links[id];
        let (from, to) = link.endpoints();
        let kind = match link {
            Link::Ownership { .. } => "chen-ownership",
            _ => "chen-association",
        };
        let mut element = Element::new(format!("cl{id}"), kind);
        element.owner = format!("c{from}");
        element.target = format!("c{to}");
        let dash = match link {
            Link::Association {
                participation: Participation::Bold,
                ..
            } => Dash::Bold,
            _ => Dash::Solid,
        };
        element
            .shapes
            .push(Shape::Line(self.route(id, budget)?, dash));
        if !link.cardinality().is_empty() {
            self.add_cardinality(id, &mut element)?;
        }
        Ok(element)
    }

    fn add_cardinality(&self, id: usize, element: &mut Element) -> Result<(), RenderError> {
        let edge = self.edge(id)?;
        let x = edge
            .x
            .ok_or_else(|| invalid(1, format!("Chen link {id} missing label x")))?;
        let y = edge
            .y
            .ok_or_else(|| invalid(1, format!("Chen link {id} missing label y")))?;
        let label = &self.measured.links[id];
        let rect = Rect {
            x: x - label.width / 2. - 3.,
            y: y - label.height / 2. - 2.,
            width: label.width + 6.,
            height: label.height + 4.,
        };
        element.shapes.push(Shape::Rect(rect, 0., Paint::Paper));
        element.shapes.push(Shape::Text(
            Point::new(x - label.width / 2., y - label.height / 2.),
            label.clone(),
            Decoration::Plain,
        ));
        Ok(())
    }
}

fn intersect(kind: ChenNode, rect: Rect, point: Point) -> Point {
    let cx = rect.x + rect.width / 2.;
    let cy = rect.y + rect.height / 2.;
    let dx = (point.x - cx) / (rect.width / 2.);
    let dy = (point.y - cy) / (rect.height / 2.);
    let scale = match kind {
        ChenNode::Entity => dx.abs().max(dy.abs()),
        ChenNode::Relationship => dx.abs() + dy.abs(),
        ChenNode::Attribute(_) => dx.hypot(dy),
    };
    if scale > 0. {
        return Point::new(cx + (point.x - cx) / scale, cy + (point.y - cy) / scale);
    }
    Point::new(cx, rect.y)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RenderOptions;
    use merman_render::environment::RenderEnvironment;

    fn enclosure(element: &Element) -> Rect {
        match &element.shapes[0] {
            Shape::Rect(rect, _, _) | Shape::Ellipse(rect, _) => *rect,
            Shape::Polygon(points, _) => Rect {
                x: points[0].x,
                y: points[1].y,
                width: points[2].x - points[0].x,
                height: points[3].y - points[1].y,
            },
            _ => panic!("expected Chen enclosure"),
        }
    }
    fn normalized(kind: &str, rect: Rect, point: Point) -> f64 {
        let dx = (point.x - rect.x - rect.width / 2.).abs() / (rect.width / 2.);
        let dy = (point.y - rect.y - rect.height / 2.).abs() / (rect.height / 2.);
        match kind {
            "chen-entity" => dx.max(dy),
            "chen-relationship" => dx + dy,
            _ => dx.hypot(dy),
        }
    }

    #[test]
    fn should_contain_measured_text_in_real_shapes_and_intersect_link_boundaries() {
        // given
        for size in [6, 14, 96] {
            let source = "@startchen\nentity \"Long customer name\\nsecond row\" as C {\n\"postal address\\nsecond row\" as address\n}\nrelationship \"Long relationship\\nsecond row\" as R {\nname\n}\nC -1- R\nC =(0,N)= R\nR -- C\nC -- C\nentity Isolated {\n}\n@endchen";
            let mut budget = Budget::new(&RenderOptions::default());
            let (_, lines) = super::super::source::prepare_document(source, &mut budget).unwrap();
            let model = super::super::chen::parse(&lines, &mut budget).unwrap();
            let session = RenderEnvironment::deterministic().begin_session().unwrap();
            let mut context = Context::new(&session, &mut budget);
            let mut fonts = super::super::style::Preamble::default().fonts;
            fonts[0].size = size;
            context = context.with_fonts(fonts);
            // when
            let scene = layout(&model, &mut context).unwrap();
            // then
            for element in &scene.elements {
                if matches!(
                    element.kind,
                    "chen-entity" | "chen-attribute" | "chen-relationship"
                ) {
                    let rect = enclosure(element);
                    for shape in &element.shapes {
                        if let Shape::Text(point, label, _) = shape {
                            for (x, y) in [
                                (point.x, point.y),
                                (point.x + label.width, point.y),
                                (point.x, point.y + label.height),
                                (point.x + label.width, point.y + label.height),
                            ] {
                                assert!(
                                    normalized(element.kind, rect, Point::new(x, y)) <= 1.,
                                    "{} text exceeds true shape at {size}px",
                                    element.name
                                );
                            }
                        }
                    }
                } else {
                    let Shape::Line(points, _) = &element.shapes[0] else {
                        panic!()
                    };
                    for (name, point) in [
                        (&element.owner, points[0]),
                        (&element.target, *points.last().unwrap()),
                    ] {
                        let owner = scene.elements.iter().find(|n| &n.id == name).unwrap();
                        assert!(
                            (normalized(owner.kind, enclosure(owner), point) - 1.).abs() < 1e-7,
                            "link {} misses {} boundary",
                            element.id,
                            name
                        );
                    }
                    for shape in &element.shapes {
                        if let Shape::Text(point, label, _) = shape {
                            let center =
                                Point::new(point.x + label.width / 2., point.y + label.height / 2.);
                            let on_route = points.windows(2).any(|segment| {
                                let (a, b) = (segment[0], segment[1]);
                                let (dx, dy) = (b.x - a.x, b.y - a.y);
                                let length = dx * dx + dy * dy;
                                if length == 0. {
                                    return false;
                                }
                                let t = ((center.x - a.x) * dx + (center.y - a.y) * dy) / length;
                                (0. ..=1.).contains(&t)
                                    && (center.x - a.x - t * dx).hypot(center.y - a.y - t * dy)
                                        < 1e-7
                            });
                            assert!(
                                on_route,
                                "cardinality is not centered on association {}",
                                element.id
                            );
                            for owner in scene.elements.iter().filter(|e| {
                                matches!(
                                    e.kind,
                                    "chen-entity" | "chen-attribute" | "chen-relationship"
                                )
                            }) {
                                let rect = enclosure(owner);
                                assert!(
                                    point.x + label.width <= rect.x
                                        || point.x >= rect.x + rect.width
                                        || point.y + label.height <= rect.y
                                        || point.y >= rect.y + rect.height,
                                    "cardinality overlaps {}",
                                    owner.name
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn should_project_diagonal_routes_to_true_ellipse_and_diamond() {
        // given
        let rect = Rect {
            x: 0.,
            y: 0.,
            width: 120.,
            height: 60.,
        };
        for (kind, name) in [
            (ChenNode::Relationship, "chen-relationship"),
            (
                ChenNode::Attribute(super::super::chen::AttributeOwner::Entity(0)),
                "chen-attribute",
            ),
        ] {
            // when
            let point = intersect(kind, rect, Point::new(120., 60.));
            // then
            assert!((normalized(name, rect, point) - 1.).abs() < 1e-8);
            assert!(point.x < 120. && point.y < 60.);
        }
    }
}
