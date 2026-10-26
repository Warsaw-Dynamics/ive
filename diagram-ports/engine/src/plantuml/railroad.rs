//! Measured native railroad tiles: MIT ETile semantics with bounded orthogonal rails.
use super::{
    budget::{Budget, bound},
    scene::{Dash, Element, Paint, Point, Rect, Scene, Shape, text_at},
    text::{Context, Label},
};
use crate::RenderError;

pub(super) enum Tile<'a> {
    Empty,
    Terminal(&'a str),
    Reference(&'a str),
    Class(&'a str),
    Sequence(usize, usize),
    Choice(usize, usize),
    Group(usize, &'a str),
    Optional(usize),
    Repeat(usize, usize, &'a str),
}
pub(super) trait Diagram {
    fn len(&self) -> usize;
    fn root_count(&self) -> usize;
    fn root(&self, index: usize) -> (&str, usize);
    fn tile(&self, id: usize) -> Tile<'_>;
    fn span(&self, id: usize) -> std::ops::Range<usize>;
    fn rule_span(&self, index: usize) -> std::ops::Range<usize>;
}
#[derive(Clone, Copy, Debug)]
pub(super) struct Geometry {
    pub width: f64,
    pub above: f64,
    pub below: f64,
    pub entry: Point,
    pub exit: Point,
}
struct Measured {
    geometry: Geometry,
    label: Label,
}
fn geometry(width: f64, above: f64, below: f64) -> Geometry {
    Geometry {
        width,
        above,
        below,
        entry: Point::new(0., above),
        exit: Point::new(width, above),
    }
}
fn store<T>(items: &mut Vec<T>, value: T, budget: &mut Budget) -> Result<(), RenderError> {
    bound(
        "railroad layout entries",
        items.len() + 1,
        budget.limits.model_items,
    )?;
    budget.charge(1)?;
    if items.len() == items.capacity() {
        let capacity = items.capacity().saturating_mul(2).max(4);
        bound(
            "railroad layout retained slots",
            capacity.saturating_add(items.capacity()),
            budget.limits.model_items.saturating_mul(3).max(4),
        )?;
        budget.charge(items.len())?;
        items
            .try_reserve_exact(capacity - items.len())
            .map_err(|error| {
                RenderError::Backend(format!(
                    "PlantUML railroad layout allocation failed: {error}"
                ))
            })?;
    }
    items.push(value);
    Ok(())
}
fn measure(
    diagram: &impl Diagram,
    context: &mut Context<'_>,
) -> Result<Vec<Measured>, RenderError> {
    let mut measured: Vec<Measured> = Vec::new();
    for id in 0..diagram.len() {
        context.budget.item()?;
        let tile = diagram.tile(id);
        let text = match tile {
            Tile::Terminal(s)
            | Tile::Reference(s)
            | Tile::Class(s)
            | Tile::Group(_, s)
            | Tile::Repeat(_, _, s) => s,
            _ => "",
        };
        let label = if text.is_empty() {
            Label {
                lines: std::sync::Arc::from([]),
                width: 0.,
                height: 0.,
            }
        } else {
            context.data_label(text)?
        };
        let g = match tile {
            Tile::Empty => geometry(30., 8., 8.),
            Tile::Terminal(_) | Tile::Reference(_) | Tile::Class(_) => geometry(
                (label.width + 20.).max(30.),
                (label.height + 16.).max(26.) / 2.,
                (label.height + 16.).max(26.) / 2.,
            ),
            Tile::Sequence(a, b) => {
                let a = measured[a].geometry;
                let b = measured[b].geometry;
                geometry(
                    a.width + 20. + b.width,
                    a.above.max(b.above),
                    a.below.max(b.below),
                )
            }
            Tile::Choice(a, b) => {
                let a = measured[a].geometry;
                let b = measured[b].geometry;
                geometry(
                    a.width.max(b.width) + 48.,
                    a.above,
                    a.below + 10. + b.above + b.below,
                )
            }
            Tile::Group(child, caption) => {
                let c = measured[child].geometry;
                if caption.is_empty() {
                    c
                } else {
                    geometry(
                        (c.width + 20.).max(label.width + 20.),
                        c.above + label.height + 16.,
                        c.below + 10.,
                    )
                }
            }
            Tile::Optional(child) => {
                let c = measured[child].geometry;
                geometry(c.width + 48., c.above + 24., c.below)
            }
            Tile::Repeat(child, min, _) => {
                let c = measured[child].geometry;
                geometry(
                    (c.width + 48.).max(label.width + 24.),
                    c.above + 28. + label.height,
                    c.below + if min == 0 { 24. } else { 0. },
                )
            }
        };
        for dimension in [g.width, g.above, g.below] {
            if !dimension.is_finite() || !(0. ..=1_000_000.).contains(&dimension) {
                return Err(super::budget::invalid(
                    1,
                    "excessive railroad tile geometry",
                ));
            }
        }
        store(
            &mut measured,
            Measured { geometry: g, label },
            context.budget,
        )?;
    }
    Ok(measured)
}

pub(super) fn layout(
    diagram: &impl Diagram,
    context: &mut Context<'_>,
) -> Result<Scene, RenderError> {
    let measured = measure(diagram, context)?;
    let mut scene = Scene::new();
    let mut top = 30.;
    for rule in 0..diagram.root_count() {
        let (name, root) = diagram.root(rule);
        let title = context.data_label(name)?;
        context.budget.item()?;
        let mut header = Element::new(format!("rr-rule-{rule}"), "railroad-rule");
        header.name = name.to_owned();
        let span = diagram.rule_span(rule);
        header.target = format!("bytes:{}..{}", span.start, span.end);
        if !name.is_empty() {
            header.shapes.push(text_at(
                Rect {
                    x: 30.,
                    y: top,
                    width: title.width,
                    height: title.height,
                },
                title.clone(),
            ));
            top += title.height + 16.;
        }
        scene.push(header, context.budget)?;
        let g = measured[root].geometry;
        let baseline = top + g.above;
        emit(
            &mut scene,
            context.budget,
            Mark {
                rule,
                id: root,
                kind: "railroad-start",
                x: 30.,
                y: baseline,
            },
            || {
                vec![Shape::Ellipse(
                    Rect {
                        x: 24.,
                        y: baseline - 6.,
                        width: 12.,
                        height: 12.,
                    },
                    Paint::Ink,
                )]
            },
        )?;
        emit(
            &mut scene,
            context.budget,
            Mark {
                rule,
                id: root,
                kind: "railroad-end",
                x: 60. + g.width,
                y: baseline,
            },
            || {
                vec![Shape::Ellipse(
                    Rect {
                        x: 54. + g.width,
                        y: baseline - 6.,
                        width: 12.,
                        height: 12.,
                    },
                    Paint::Ink,
                )]
            },
        )?;
        rail(
            &mut scene,
            context.budget,
            Mark {
                rule,
                id: root,
                kind: "railroad-entry",
                x: 0.,
                y: 0.,
            },
            &[(30., baseline), (45. + g.entry.x, top + g.entry.y)],
        )?;
        rail(
            &mut scene,
            context.budget,
            Mark {
                rule,
                id: root,
                kind: "railroad-exit",
                x: 0.,
                y: 0.,
            },
            &[(45. + g.exit.x, top + g.exit.y), (60. + g.width, baseline)],
        )?;
        let mut pending = Vec::new();
        store(&mut pending, (root, 45., baseline), context.budget)?;
        while let Some((id, x, y)) = pending.pop() {
            context.budget.charge(1)?;
            place(
                diagram,
                &measured,
                (id, x, y),
                rule,
                (&mut pending, &mut scene),
                context.budget,
            )?;
        }
        top += g.above + g.below + 40.;
    }
    Ok(scene)
}

#[derive(Clone, Copy)]
struct Mark {
    rule: usize,
    id: usize,
    kind: &'static str,
    x: f64,
    y: f64,
}
fn emit(
    scene: &mut Scene,
    budget: &mut Budget,
    mark: Mark,
    shapes: impl FnOnce() -> Vec<Shape>,
) -> Result<(), RenderError> {
    budget.item()?;
    budget.charge(8)?;
    let shapes = shapes();
    budget.charge(shapes.len() + 1)?;
    let mut element = Element::new(
        format!("rr-{}-{}-{}", mark.rule, mark.id, mark.kind),
        mark.kind,
    );
    element.owner = format!("rr-rule-{}", mark.rule);
    element.target = format!("rr-node-{}-{}", mark.rule, mark.id);
    element.name = format!("port:{},{}", mark.x, mark.y);
    element.shapes = shapes;
    scene.push(element, budget)
}
fn rail(
    scene: &mut Scene,
    budget: &mut Budget,
    mark: Mark,
    points: &[(f64, f64)],
) -> Result<(), RenderError> {
    // Every route has at most six vertices; account before allocating the line/arrow.
    bound("railroad route vertices", points.len(), 6)?;
    budget.item()?;
    budget.charge(points.len() + 8)?;
    let mut vertices = Vec::new();
    for &(x, y) in points {
        store(&mut vertices, Point::new(x, y), budget)?;
    }
    let mut shapes = vec![Shape::Line(vertices, Dash::Solid)];
    if let [.., (a, b), (x, y)] = points {
        if (x - a).abs() > 10. {
            let d = if x > a { 1. } else { -1. };
            let mid = (x + a) / 2.;
            shapes.push(Shape::Polygon(
                vec![
                    Point::new(mid + 3. * d, *y),
                    Point::new(mid - 3. * d, y - 3.),
                    Point::new(mid - 3. * d, y + 3.),
                ],
                Paint::Ink,
            ));
        } else if (y - b).abs() > 10. {
            let d = if y > b { 1. } else { -1. };
            let mid = (y + b) / 2.;
            shapes.push(Shape::Polygon(
                vec![
                    Point::new(*x, mid + 3. * d),
                    Point::new(x - 3., mid - 3. * d),
                    Point::new(x + 3., mid - 3. * d),
                ],
                Paint::Ink,
            ));
        }
    }
    emit(scene, budget, mark, || shapes)
}
fn place(
    diagram: &impl Diagram,
    measured: &[Measured],
    at: (usize, f64, f64),
    rule: usize,
    output: (&mut Vec<(usize, f64, f64)>, &mut Scene),
    budget: &mut Budget,
) -> Result<(), RenderError> {
    let (id, x, y) = at;
    let (pending, scene) = output;
    let m = &measured[id];
    let g = m.geometry;
    let span = diagram.span(id);
    budget.item()?;
    let mut anchor = Element::new(format!("rr-node-{rule}-{id}"), "railroad-node");
    anchor.owner = format!("rr-rule-{rule}");
    anchor.target = format!("bytes:{}..{}", span.start, span.end);
    scene.push(anchor, budget)?;
    let mark = Mark {
        rule,
        id,
        kind: "railroad-empty",
        x,
        y,
    };
    match diagram.tile(id) {
        Tile::Empty => rail(scene, budget, mark, &[(x, y), (x + g.width, y)]),
        Tile::Terminal(_) | Tile::Reference(_) | Tile::Class(_) => {
            let (kind, radius) = match diagram.tile(id) {
                Tile::Terminal(_) => ("railroad-terminal", 0.),
                Tile::Class(_) => ("railroad-class", 4.),
                _ => ("railroad-reference", 10.),
            };
            budget.charge(4)?;
            let rect = Rect {
                x,
                y: y - g.above,
                width: g.width,
                height: g.above + g.below,
            };
            emit(scene, budget, Mark { kind, ..mark }, || {
                vec![
                    Shape::Rect(rect, radius, Paint::Node),
                    text_at(rect, m.label.clone()),
                ]
            })
        }
        Tile::Sequence(a, b) => {
            let a_width = measured[a].geometry.width;
            rail(
                scene,
                budget,
                Mark {
                    kind: "railroad-concat",
                    ..mark
                },
                &[(x + a_width, y), (x + a_width + 20., y)],
            )?;
            store(pending, (b, x + a_width + 20., y), budget)?;
            store(pending, (a, x, y), budget)
        }
        Tile::Choice(a, b) => {
            let a_g = measured[a].geometry;
            let b_g = measured[b].geometry;
            let second = y + a_g.below + 10. + b_g.above;
            let lane = x + 12.;
            let end = x + g.width - 12.;
            rail(
                scene,
                budget,
                Mark {
                    kind: "railroad-choice",
                    ..mark
                },
                &[(x, y), (x + 24., y)],
            )?;
            rail(
                scene,
                budget,
                Mark {
                    kind: "railroad-choice-lower",
                    ..mark
                },
                &[(lane, y), (lane, second), (x + 24., second)],
            )?;
            rail(
                scene,
                budget,
                Mark {
                    kind: "railroad-choice-upper-exit",
                    ..mark
                },
                &[(x + 24. + a_g.width, y), (x + g.width, y)],
            )?;
            rail(
                scene,
                budget,
                Mark {
                    kind: "railroad-choice-lower-exit",
                    ..mark
                },
                &[(x + 24. + b_g.width, second), (end, second), (end, y)],
            )?;
            store(pending, (b, x + 24., second), budget)?;
            store(pending, (a, x + 24., y), budget)
        }
        Tile::Group(child, caption) => {
            if caption.is_empty() {
                return store(pending, (child, x, y), budget);
            }
            let c = measured[child].geometry;
            let inset = (g.width - c.width) / 2.;
            let rect = Rect {
                x,
                y: y - g.above,
                width: g.width,
                height: g.above + g.below,
            };
            budget.charge(9)?;
            emit(
                scene,
                budget,
                Mark {
                    kind: "railroad-group",
                    ..mark
                },
                || {
                    vec![
                        Shape::Line(
                            vec![
                                Point::new(x, rect.y),
                                Point::new(x + g.width, rect.y),
                                Point::new(x + g.width, y + g.below),
                                Point::new(x, y + g.below),
                                Point::new(x, rect.y),
                            ],
                            Dash::Dashed,
                        ),
                        text_at(
                            Rect {
                                x: x + 8.,
                                y: rect.y + 4.,
                                width: m.label.width,
                                height: m.label.height,
                            },
                            m.label.clone(),
                        ),
                    ]
                },
            )?;
            rail(
                scene,
                budget,
                Mark {
                    kind: "railroad-group-entry",
                    ..mark
                },
                &[(x, y), (x + inset, y)],
            )?;
            rail(
                scene,
                budget,
                Mark {
                    kind: "railroad-group-exit",
                    ..mark
                },
                &[(x + inset + c.width, y), (x + g.width, y)],
            )?;
            store(pending, (child, x + inset, y), budget)
        }
        Tile::Optional(child) => {
            let c = measured[child].geometry;
            let inset = 24.;
            let bypass = y - c.above - 24.;
            rail(
                scene,
                budget,
                Mark {
                    kind: "railroad-bypass",
                    ..mark
                },
                &[
                    (x, y),
                    (x + 12., y),
                    (x + 12., bypass),
                    (x + g.width - 12., bypass),
                    (x + g.width - 12., y),
                    (x + g.width, y),
                ],
            )?;
            rail(
                scene,
                budget,
                Mark {
                    kind: "railroad-optional-entry",
                    ..mark
                },
                &[(x, y), (x + inset, y)],
            )?;
            rail(
                scene,
                budget,
                Mark {
                    kind: "railroad-optional-exit",
                    ..mark
                },
                &[(x + inset + c.width, y), (x + g.width, y)],
            )?;
            store(pending, (child, x + inset, y), budget)
        }
        Tile::Repeat(child, min, caption) => {
            let c = measured[child].geometry;
            let inset = (g.width - c.width) / 2.;
            let back = y - c.above - 20.;
            rail(
                scene,
                budget,
                Mark {
                    kind: "railroad-return",
                    ..mark
                },
                &[
                    (x + g.width - 12., y),
                    (x + g.width - 12., back),
                    (x + 12., back),
                    (x + 12., y),
                ],
            )?;
            rail(
                scene,
                budget,
                Mark {
                    kind: "railroad-repeat-entry",
                    ..mark
                },
                &[(x, y), (x + inset, y)],
            )?;
            rail(
                scene,
                budget,
                Mark {
                    kind: "railroad-repeat-exit",
                    ..mark
                },
                &[(x + inset + c.width, y), (x + g.width, y)],
            )?;
            if min == 0 {
                let bypass = y + c.below + 24.;
                rail(
                    scene,
                    budget,
                    Mark {
                        kind: "railroad-bypass",
                        ..mark
                    },
                    &[
                        (x, y),
                        (x + 6., y),
                        (x + 6., bypass),
                        (x + g.width - 6., bypass),
                        (x + g.width - 6., y),
                        (x + g.width, y),
                    ],
                )?;
            }
            if !caption.is_empty() {
                budget.charge(1)?;
                emit(
                    scene,
                    budget,
                    Mark {
                        kind: "railroad-repeat-bound",
                        ..mark
                    },
                    || {
                        vec![text_at(
                            Rect {
                                x: x + (g.width - m.label.width) / 2.,
                                y: y - g.above,
                                width: m.label.width,
                                height: m.label.height,
                            },
                            m.label.clone(),
                        )]
                    },
                )?;
            }
            store(pending, (child, x + inset, y), budget)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RenderOptions, plantuml::parser};
    use merman_render::environment::RenderEnvironment;
    fn draw(source: &str) -> Scene {
        let mut budget = Budget::new(&RenderOptions::default());
        let model = parser::test_support::literal(
            &format!("@startregex\n{source}\n@endregex"),
            &mut budget,
            parser::regex::parse,
        )
        .unwrap();
        let session = RenderEnvironment::deterministic().begin_session().unwrap();
        layout(&model, &mut Context::new(&session, &mut budget)).unwrap()
    }
    fn rect(element: &Element) -> Rect {
        match element.shapes[0] {
            Shape::Rect(rect, ..) => rect,
            _ => panic!("expected leaf rectangle"),
        }
    }
    fn points(element: &Element) -> &[Point] {
        match &element.shapes[0] {
            Shape::Line(points, _) => points,
            _ => panic!("expected rail"),
        }
    }
    #[test]
    fn should_stack_ordered_branches_and_connect_both_ports_to_shared_rails() {
        // given
        let scene = draw("a|bb");
        // when
        let leaves = scene
            .elements
            .iter()
            .filter(|e| e.kind == "railroad-terminal")
            .collect::<Vec<_>>();
        let boxes = leaves.iter().map(|e| rect(e)).collect::<Vec<_>>();
        let branch = scene
            .elements
            .iter()
            .find(|e| e.kind == "railroad-choice-lower")
            .unwrap();
        let exit = scene
            .elements
            .iter()
            .find(|e| e.kind == "railroad-choice-lower-exit")
            .unwrap();
        // then
        assert_eq!(boxes.len(), 3);
        assert!(boxes[0].y + boxes[0].height < boxes[1].y);
        let p = points(branch).last().unwrap();
        assert_eq!(p.x, boxes[1].x);
        assert_eq!(p.y, boxes[1].y + boxes[1].height / 2.);
        let p = points(exit)[0];
        assert_eq!(p.x, boxes[2].x + boxes[2].width);
        assert_eq!(p.y, boxes[2].y + boxes[2].height / 2.);
        assert_eq!(boxes[1].y, boxes[2].y);
    }
    #[test]
    fn should_connect_repeat_return_and_optional_bypass_without_crossing_leaf_interiors() {
        // given
        let sources = ["a+", "a*", "a?", "(a|[b-d])+e{2,9}", "(?:a|b)*"];
        // when
        let scenes = sources.map(draw);
        // then
        for scene in scenes {
            let boxes = scene
                .elements
                .iter()
                .filter(|e| {
                    matches!(
                        e.kind,
                        "railroad-terminal" | "railroad-reference" | "railroad-class"
                    )
                })
                .map(rect)
                .collect::<Vec<_>>();
            for element in &scene.elements {
                for shape in &element.shapes {
                    if let Shape::Line(vertices, Dash::Solid) = shape {
                        for pair in vertices.windows(2) {
                            let (a, b) = (pair[0], pair[1]);
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
                                assert!(!crosses, "{} crosses leaf {r:?}", element.kind);
                            }
                        }
                    }
                    if let Shape::Text(p, label, _) = shape {
                        assert!(p.x >= scene.min_x && p.y >= scene.min_y);
                        assert!(
                            p.x + label.width <= scene.width && p.y + label.height <= scene.height
                        );
                    }
                }
            }
            for a in 0..boxes.len() {
                for b in a + 1..boxes.len() {
                    let (a, b) = (boxes[a], boxes[b]);
                    assert!(
                        a.x + a.width <= b.x
                            || b.x + b.width <= a.x
                            || a.y + a.height <= b.y
                            || b.y + b.height <= a.y
                    );
                }
            }
        }
    }
    #[test]
    fn should_bound_every_tile_and_expose_measured_entry_exit_ports() {
        // given
        let mut budget = Budget::new(&RenderOptions::default());
        let model = parser::test_support::literal(
            "@startregex\n(a|b)+c?\n@endregex",
            &mut budget,
            parser::regex::parse,
        )
        .unwrap();
        let session = RenderEnvironment::deterministic().begin_session().unwrap();
        // when
        let tiles = measure(&model, &mut Context::new(&session, &mut budget)).unwrap();
        // then
        for tile in tiles {
            let g = tile.geometry;
            assert_eq!(g.entry.x, 0.);
            assert_eq!(g.exit.x, g.width);
            assert_eq!(g.entry.y, g.above);
            assert_eq!(g.exit.y, g.above);
            assert!(g.above >= 0. && g.below >= 0.);
        }
    }
}
