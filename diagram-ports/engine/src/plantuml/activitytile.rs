//! Original bottom-up activity tiles with explicit entry/live/dead exit ports.
//! Control-flow semantics follow the authorized MIT sources in the provenance manifest.
use super::{
    activity::{Block, Branch, Condition, GroupKind, Instruction, Join},
    arrow,
    budget::{Budget, invalid},
    scene::{Dash, Decoration, Element, Paint, Point, Rect, Scene, Shape, text_at},
    text::Context,
};
use crate::RenderError;

#[derive(Clone, Copy, Debug)]
struct Port {
    id: usize,
    point: Point,
}
#[derive(Clone, Copy, Debug)]
enum Exit {
    Live(Port),
    Dead,
}
struct Tile {
    elements: Vec<Element>,
    width: f64,
    height: f64,
    input: Port,
    output: Exit,
}
struct Builder<'a, 'b> {
    context: &'a mut Context<'b>,
    next_id: usize,
}
struct Flow<'a> {
    from: Port,
    to: Port,
    label: &'a str,
    via: Vec<Point>,
    label_at: Point,
}

pub(super) fn layout(block: &Block, context: &mut Context<'_>) -> Result<Scene, RenderError> {
    let mut builder = Builder {
        context,
        next_id: 0,
    };
    let mut tile = builder.block(block)?;
    shift(&mut tile, Point::new(24., 24.), builder.context.budget)?;
    builder.context.budget.charge(tile.elements.len())?;
    // Container fills must precede incoming flows from any enclosing composition.
    let (containers, content): (Vec<_>, Vec<_>) = tile
        .elements
        .into_iter()
        .partition(|element| matches!(element.kind, "activity-group" | "activity-partition"));
    let mut scene = Scene::new();
    for element in containers.into_iter().chain(content) {
        scene.push(element, builder.context.budget)?;
    }
    Ok(scene)
}

impl Builder<'_, '_> {
    fn id(&mut self) -> Result<usize, RenderError> {
        self.context.budget.charge(1)?;
        let id = self.next_id;
        self.next_id += 1;
        Ok(id)
    }

    fn primitive(&mut self, kind: &'static str, text: &str) -> Result<Tile, RenderError> {
        let id = self.id()?;
        let label = self.context.label(text)?;
        let (width, height) = match kind {
            "activity-start" | "activity-stop" | "activity-end" => (24., 24.),
            "activity-port" => (1., 1.),
            "activity-fork" | "activity-join" => (
                (label.width + 32.).max(80.),
                8. + if text.is_empty() {
                    0.
                } else {
                    label.height + 10.
                },
            ),
            "activity-if" | "activity-while" | "activity-repeat" | "activity-merge" => {
                ((label.width * 2. + 40.).max(40.), label.height * 2. + 24.)
            }
            _ => ((label.width + 32.).max(80.), label.height + 24.),
        };
        let rect = Rect {
            x: 0.,
            y: 0.,
            width,
            height,
        };
        let mut element = Element::new(format!("a{id}"), kind);
        element.name = text.into();
        element.shapes = primitive_shapes(rect, kind);
        if !text.is_empty() {
            element.shapes.push(text_at(rect, label));
        }
        Ok(Tile {
            elements: vec![element],
            width,
            height,
            input: Port {
                id,
                point: Point::new(width / 2., 0.),
            },
            output: if matches!(kind, "activity-stop" | "activity-end") {
                Exit::Dead
            } else {
                Exit::Live(Port {
                    id,
                    point: Point::new(width / 2., height),
                })
            },
        })
    }

    fn block(&mut self, block: &Block) -> Result<Tile, RenderError> {
        self.context.budget.charge(block.len() + 1)?;
        if block.is_empty() {
            return self.primitive("activity-port", "");
        }
        let tiles = block
            .iter()
            .map(|instruction| self.instruction(instruction))
            .collect::<Result<Vec<_>, _>>()?;
        let width = tiles.iter().map(|t| t.width).fold(0., f64::max);
        let mut elements = Vec::new();
        let mut y = 0.;
        let mut input = tiles[0].input;
        let mut output = Exit::Dead;
        for (index, mut tile) in tiles.into_iter().enumerate() {
            let x = (width - tile.width) / 2.;
            shift(&mut tile, Point::new(x, y), self.context.budget)?;
            if index == 0 {
                input = tile.input;
            }
            if let Exit::Live(from) = output {
                self.connect(&mut elements, (from, tile.input, ""))?;
            }
            y += tile.height + 48.;
            output = tile.output;
            elements.extend(tile.elements);
        }
        Ok(Tile {
            elements,
            width,
            height: y - 48.,
            input,
            output,
        })
    }

    fn instruction(&mut self, instruction: &Instruction) -> Result<Tile, RenderError> {
        self.context.budget.checkpoint()?;
        match instruction {
            Instruction::Start => self.primitive("activity-start", ""),
            Instruction::Stop => self.primitive("activity-stop", ""),
            Instruction::End => self.primitive("activity-end", ""),
            Instruction::Action(text) => self.primitive("activity-action", text),
            Instruction::If {
                branches,
                otherwise,
                guard,
            } => self.conditional(branches, (otherwise, guard)),
            Instruction::While {
                condition,
                body,
                out,
            } => self.while_loop(condition, (body, out)),
            Instruction::Repeat {
                entry,
                body,
                condition,
                out,
            } => self.repeat_loop((entry, body), (condition, out)),
            Instruction::Fork { lanes, join, label } => self.fork(lanes, (*join, label)),
            Instruction::Group { name, kind, body } => self.group(body, (name, *kind)),
        }
    }

    fn connect(
        &mut self,
        elements: &mut Vec<Element>,
        input: (Port, Port, &str),
    ) -> Result<(), RenderError> {
        let (from, to, label) = input;
        let middle = (from.point.y + to.point.y) / 2.;
        self.flow(
            elements,
            Flow {
                from,
                to,
                label,
                via: vec![
                    Point::new(from.point.x, middle),
                    Point::new(to.point.x, middle),
                ],
                label_at: Point::new(from.point.x + 6., from.point.y + 8.),
            },
        )
    }

    fn flow(&mut self, elements: &mut Vec<Element>, flow: Flow<'_>) -> Result<(), RenderError> {
        let id = self.id()?;
        let mut element = Element::new(format!("f{id}"), "activity-flow");
        element.owner = format!("a{}", flow.from.id);
        element.target = format!("a{}", flow.to.id);
        self.context.budget.text(flow.label.len())?;
        element.name = flow.label.into();
        let mut points = vec![flow.from.point];
        points.extend(flow.via);
        points.push(flow.to.point);
        element.shapes = arrow::draw(
            points,
            arrow::Arrow {
                start: arrow::End::None,
                end: arrow::End::Filled,
                dash: Dash::Solid,
            },
        );
        if !flow.label.is_empty() {
            element.shapes.push(Shape::Text(
                flow.label_at,
                self.context.label(flow.label)?,
                Decoration::Plain,
            ));
        }
        elements.push(element);
        Ok(())
    }

    fn conditional(
        &mut self,
        branches: &[Branch],
        otherwise: (&Block, &str),
    ) -> Result<Tile, RenderError> {
        let mut lanes = Vec::new();
        let mut decisions = Vec::new();
        let mut widths = Vec::new();
        let guard = self.context.label(otherwise.1)?;
        let mut heading: f64 = guard.height + 40.;
        for branch in branches {
            let decision = self.primitive("activity-if", &branch.condition.test)?;
            let body = self.block(&branch.body)?;
            let label = self.context.label(&branch.condition.guard)?;
            heading = heading.max(decision.height + label.height + 40.);
            widths.push(body.width.max(decision.width + label.width * 2. + 24.));
            decisions.push(decision);
            lanes.push(body);
        }
        let other = self.block(otherwise.0)?;
        widths.push(other.width.max(guard.width * 2. + 40.));
        lanes.push(other);
        let width = widths.iter().sum::<f64>() + 70. * (widths.len() - 1) as f64;
        let mut elements = Vec::new();
        let mut x = 0.;
        for ((lane, lane_width), decision) in
            lanes.iter_mut().zip(&widths).zip(decisions.iter_mut())
        {
            shift(
                decision,
                Point::new(x + (lane_width - decision.width) / 2., 0.),
                self.context.budget,
            )?;
            shift(
                lane,
                Point::new(x + (lane_width - lane.width) / 2., heading),
                self.context.budget,
            )?;
            x += lane_width + 70.;
        }
        let last = lanes.last_mut().unwrap();
        shift(
            last,
            Point::new(x + (widths.last().unwrap() - last.width) / 2., heading),
            self.context.budget,
        )?;
        for (index, decision) in decisions.iter().enumerate() {
            self.connect(
                &mut elements,
                (
                    live(decision.output)?,
                    lanes[index].input,
                    &branches[index].condition.guard,
                ),
            )?;
            let to = if let Some(next) = decisions.get(index + 1) {
                side(next, Side::Left)
            } else {
                lanes[index + 1].input
            };
            let from = side(decision, Side::Right);
            self.flow(
                &mut elements,
                Flow {
                    from,
                    to,
                    label: if index + 1 == decisions.len() {
                        otherwise.1
                    } else {
                        ""
                    },
                    via: vec![Point::new(to.point.x, from.point.y)],
                    label_at: Point::new(from.point.x + 8., from.point.y - guard.height - 8.),
                },
            )?;
        }
        let input = decisions[0].input;
        let bottom = heading + lanes.iter().map(|t| t.height).fold(0., f64::max) + 48.;
        let (output, height) =
            self.merge_lanes(&mut elements, (&lanes, width, bottom), (Join::Merge, ""))?;
        elements.extend(decisions.into_iter().flat_map(|t| t.elements));
        elements.extend(lanes.into_iter().flat_map(|t| t.elements));
        Ok(Tile {
            elements,
            width,
            height,
            input,
            output,
        })
    }

    fn merge_lanes(
        &mut self,
        elements: &mut Vec<Element>,
        lanes: (&[Tile], f64, f64),
        join: (Join, &str),
    ) -> Result<(Exit, f64), RenderError> {
        let (lanes, width, y) = lanes;
        if lanes.iter().all(|lane| matches!(lane.output, Exit::Dead)) {
            return Ok((Exit::Dead, y));
        }
        let mut tile = self.primitive(
            match join.0 {
                Join::Synchronize => "activity-join",
                Join::Merge => "activity-merge",
            },
            join.1,
        )?;
        if matches!(join.0, Join::Synchronize) {
            tile.width = width;
            tile.input.point.x = width / 2.;
            if let Exit::Live(port) = &mut tile.output {
                port.point.x = width / 2.;
                port.point.y = 8.;
            }
            if let Shape::Rect(rect, _, _) = &mut tile.elements[0].shapes[0] {
                rect.width = width;
            }
            for shape in &mut tile.elements[0].shapes {
                if let Shape::Text(point, _, _) = shape {
                    point.x = width / 2. + 10.;
                    point.y = 18.;
                }
            }
        }
        center(&mut tile, (width, y), self.context.budget)?;
        for lane in lanes {
            if let Exit::Live(port) = lane.output {
                let to = if matches!(join.0, Join::Synchronize) {
                    Port {
                        point: Point::new(port.point.x, tile.input.point.y),
                        ..tile.input
                    }
                } else {
                    tile.input
                };
                self.connect(elements, (port, to, ""))?;
            }
        }
        let result = (tile.output, y + tile.height);
        elements.extend(tile.elements);
        Ok(result)
    }

    fn while_loop(
        &mut self,
        condition: &Condition,
        input: (&Block, &str),
    ) -> Result<Tile, RenderError> {
        let mut test = self.primitive("activity-while", &condition.test)?;
        let mut body = self.block(input.0)?;
        let yes = self.context.label(&condition.guard)?;
        let no = self.context.label(input.1)?;
        let gutter = yes.width.max(no.width) + 48.;
        let width = body.width.max(test.width) + gutter * 2.;
        let body_y = test.height + yes.height + 40.;
        center(&mut test, (width, 0.), self.context.budget)?;
        center(&mut body, (width, body_y), self.context.budget)?;
        let mut exit = self.primitive("activity-port", "")?;
        shift(
            &mut exit,
            Point::new(width / 2., body_y + body.height + 48.),
            self.context.budget,
        )?;
        let mut elements = Vec::new();
        self.connect(
            &mut elements,
            (live(test.output)?, body.input, &condition.guard),
        )?;
        let from = side(&test, Side::Right);
        self.flow(
            &mut elements,
            Flow {
                from,
                to: exit.input,
                label: input.1,
                via: vec![
                    Point::new(width - 12., from.point.y),
                    Point::new(width - 12., exit.input.point.y),
                ],
                label_at: Point::new(from.point.x + 8., from.point.y - no.height - 8.),
            },
        )?;
        if let Exit::Live(from) = body.output {
            let to = side(&test, Side::Left);
            self.flow(
                &mut elements,
                Flow {
                    from,
                    to,
                    label: "",
                    via: vec![
                        Point::new(from.point.x, exit.input.point.y - 20.),
                        Point::new(12., exit.input.point.y - 20.),
                        Point::new(12., to.point.y),
                    ],
                    label_at: Point::new(0., 0.),
                },
            )?;
        }
        let result = Tile {
            width,
            height: exit.input.point.y + 1.,
            input: test.input,
            output: exit.output,
            elements: vec![],
        };
        elements.extend(test.elements);
        elements.extend(body.elements);
        elements.extend(exit.elements);
        Ok(Tile { elements, ..result })
    }

    fn repeat_loop(
        &mut self,
        input: (&str, &Block),
        test_input: (&Condition, &str),
    ) -> Result<Tile, RenderError> {
        let (condition, out) = test_input;
        let mut entry = self.primitive(
            if input.0.is_empty() {
                "activity-merge"
            } else {
                "activity-action"
            },
            input.0,
        )?;
        let mut body = self.block(input.1)?;
        let mut test = self.primitive("activity-repeat", &condition.test)?;
        let yes = self.context.label(&condition.guard)?;
        let no = self.context.label(out)?;
        let gutter = yes.width.max(no.width) + 48.;
        let width = entry.width.max(body.width).max(test.width) + gutter * 2.;
        let body_y = entry.height + 48.;
        let test_y = body_y + body.height + 48.;
        center(&mut entry, (width, 0.), self.context.budget)?;
        center(&mut body, (width, body_y), self.context.budget)?;
        center(&mut test, (width, test_y), self.context.budget)?;
        let mut exit = self.primitive("activity-port", "")?;
        shift(
            &mut exit,
            Point::new(width / 2., test_y + test.height + no.height + 40.),
            self.context.budget,
        )?;
        let mut elements = Vec::new();
        self.connect(&mut elements, (live(entry.output)?, body.input, ""))?;
        let output = if let Exit::Live(from) = body.output {
            self.connect(&mut elements, (from, test.input, ""))?;
            let from = side(&test, Side::Left);
            let to = side(&entry, Side::Left);
            self.flow(
                &mut elements,
                Flow {
                    from,
                    to,
                    label: &condition.guard,
                    via: vec![Point::new(12., from.point.y), Point::new(12., to.point.y)],
                    label_at: Point::new(18., from.point.y - yes.height - 8.),
                },
            )?;
            self.connect(&mut elements, (live(test.output)?, exit.input, out))?;
            exit.output
        } else {
            Exit::Dead
        };
        let result = Tile {
            width,
            height: exit.input.point.y + 1.,
            input: entry.input,
            output,
            elements: vec![],
        };
        elements.extend(entry.elements);
        elements.extend(body.elements);
        elements.extend(test.elements);
        elements.extend(exit.elements);
        Ok(Tile { elements, ..result })
    }

    fn fork(&mut self, blocks: &[Block], join: (Join, &str)) -> Result<Tile, RenderError> {
        let mut lanes = blocks
            .iter()
            .map(|block| self.block(block))
            .collect::<Result<Vec<_>, _>>()?;
        let label = self.context.label(join.1)?;
        let occupied = lanes.iter().map(|l| l.width).sum::<f64>() + 60. * (lanes.len() - 1) as f64;
        let width = occupied.max(label.width * 2. + 80.);
        let mut split = self.primitive("activity-fork", "")?;
        split.width = width;
        split.input.point.x = width / 2.;
        split.output = Exit::Live(Port {
            id: split.input.id,
            point: Point::new(width / 2., 8.),
        });
        split.elements[0].shapes = primitive_shapes(
            Rect {
                x: 0.,
                y: 0.,
                width,
                height: 8.,
            },
            "activity-fork",
        );
        let mut elements = Vec::new();
        let mut x = (width - occupied) / 2.;
        for lane in &mut lanes {
            shift(lane, Point::new(x, 56.), self.context.budget)?;
            self.connect(
                &mut elements,
                (
                    Port {
                        id: split.input.id,
                        point: Point::new(lane.input.point.x, 8.),
                    },
                    lane.input,
                    "",
                ),
            )?;
            x += lane.width + 60.;
        }
        let bottom = 104. + lanes.iter().map(|l| l.height).fold(0., f64::max);
        let (output, height) = self.merge_lanes(&mut elements, (&lanes, width, bottom), join)?;
        let input = split.input;
        elements.extend(split.elements);
        elements.extend(lanes.into_iter().flat_map(|t| t.elements));
        Ok(Tile {
            elements,
            width,
            height,
            input,
            output,
        })
    }

    fn group(&mut self, block: &Block, input: (&str, GroupKind)) -> Result<Tile, RenderError> {
        let mut body = self.block(block)?;
        let title = self.context.label(input.0)?;
        let width = (body.width + 48.).max(title.width + 64.);
        let height = body.height + title.height + 60.;
        let id = self.id()?;
        let mut element = Element::new(
            format!("a{id}"),
            match input.1 {
                GroupKind::Partition => "activity-partition",
                GroupKind::Group => "activity-group",
            },
        );
        element.name = input.0.into();
        element.shapes = vec![
            Shape::Rect(
                Rect {
                    x: 0.,
                    y: 0.,
                    width,
                    height,
                },
                6.,
                Paint::Container,
            ),
            Shape::Text(Point::new(16., 10.), title.clone(), Decoration::Plain),
            Shape::Line(
                vec![
                    Point::new(0., title.height + 20.),
                    Point::new(width, title.height + 20.),
                ],
                Dash::Solid,
            ),
        ];
        center(&mut body, (width, title.height + 36.), self.context.budget)?;
        for child in &mut body.elements {
            if child.kind != "activity-flow" && child.owner.is_empty() {
                child.owner = format!("a{id}");
            }
        }
        let mut elements = vec![element];
        let entry = Port {
            id,
            point: Point::new(width - 12., 0.),
        };
        self.flow(
            &mut elements,
            Flow {
                from: entry,
                to: body.input,
                label: "",
                via: vec![
                    Point::new(entry.point.x, body.input.point.y - 12.),
                    Point::new(body.input.point.x, body.input.point.y - 12.),
                ],
                label_at: entry.point,
            },
        )?;
        elements.extend(body.elements);
        Ok(Tile {
            elements,
            width,
            height,
            input: entry,
            output: body.output,
        })
    }
}

fn live(exit: Exit) -> Result<Port, RenderError> {
    match exit {
        Exit::Live(port) => Ok(port),
        Exit::Dead => Err(invalid(1, "activity tile requires a live port")),
    }
}
enum Side {
    Left,
    Right,
}
fn side(tile: &Tile, side: Side) -> Port {
    Port {
        id: tile.input.id,
        point: Point::new(
            tile.input.point.x
                + match side {
                    Side::Left => -tile.width / 2.,
                    Side::Right => tile.width / 2.,
                },
            tile.input.point.y + tile.height / 2.,
        ),
    }
}

fn center(tile: &mut Tile, position: (f64, f64), budget: &mut Budget) -> Result<(), RenderError> {
    let x = (position.0 - tile.width) / 2.;
    shift(tile, Point::new(x, position.1), budget)
}

fn shift(tile: &mut Tile, offset: Point, budget: &mut Budget) -> Result<(), RenderError> {
    budget.charge(tile.elements.len() + 1)?;
    move_point(&mut tile.input.point, offset);
    if let Exit::Live(port) = &mut tile.output {
        move_point(&mut port.point, offset);
    }
    for element in &mut tile.elements {
        budget.charge(element.shapes.len())?;
        for shape in &mut element.shapes {
            match shape {
                Shape::Rect(rect, _, _) | Shape::Ellipse(rect, _) => {
                    rect.x += offset.x;
                    rect.y += offset.y;
                }
                Shape::Text(point, _, _) => move_point(point, offset),
                Shape::Line(points, _)
                | Shape::ColoredLine(points, _, _)
                | Shape::Polygon(points, _) => {
                    for point in points {
                        move_point(point, offset);
                    }
                }
            }
        }
    }
    Ok(())
}
fn move_point(point: &mut Point, offset: Point) {
    point.x += offset.x;
    point.y += offset.y;
}

fn primitive_shapes(rect: Rect, kind: &str) -> Vec<Shape> {
    match kind {
        "activity-port" => vec![],
        "activity-start" => vec![Shape::Ellipse(rect, Paint::Ink)],
        "activity-stop" => vec![
            Shape::Ellipse(rect, Paint::Paper),
            Shape::Ellipse(
                Rect {
                    x: 5.,
                    y: 5.,
                    width: 14.,
                    height: 14.,
                },
                Paint::Ink,
            ),
        ],
        "activity-end" => vec![
            Shape::Ellipse(rect, Paint::Paper),
            Shape::Line(vec![Point::new(5., 5.), Point::new(19., 19.)], Dash::Solid),
            Shape::Line(vec![Point::new(5., 19.), Point::new(19., 5.)], Dash::Solid),
        ],
        "activity-fork" | "activity-join" => {
            vec![Shape::Rect(Rect { height: 8., ..rect }, 0., Paint::Ink)]
        }
        "activity-if" | "activity-while" | "activity-repeat" | "activity-merge" => {
            vec![Shape::Polygon(
                vec![
                    Point::new(rect.width / 2., 0.),
                    Point::new(rect.width, rect.height / 2.),
                    Point::new(rect.width / 2., rect.height),
                    Point::new(0., rect.height / 2.),
                ],
                Paint::Node,
            )]
        }
        _ => vec![Shape::Rect(rect, 10., Paint::Node)],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RenderOptions;
    use merman_render::environment::RenderEnvironment;

    fn draw(body: &str) -> Scene {
        let mut budget = Budget::new(&RenderOptions::default());
        let model = crate::plantuml::parser::test_support::literal(
            &format!("@startuml\n{body}\n@enduml"),
            &mut budget,
            crate::plantuml::parser::activity::parse,
        )
        .unwrap();
        let session = RenderEnvironment::deterministic().begin_session().unwrap();
        layout(&model, &mut Context::new(&session, &mut budget)).unwrap()
    }

    fn box_of(element: &Element) -> Rect {
        match element.shapes[0] {
            Shape::Rect(rect, _, _) => rect,
            _ => panic!("expected rectangle for {}", element.kind),
        }
    }

    #[test]
    fn should_paint_incoming_group_flow_above_every_opaque_container() {
        // given
        for source in [
            "start\npartition P {\n:Inside;\n}\nstop",
            "start\npartition P {\ngroup Q {\n:Inside;\n}\n}\nstop",
            "if (x) then (yes)\npartition P {\n:Inside;\n}\nendif\nstop",
        ] {
            // when
            let scene = draw(source);
            let action = scene.elements.iter().find(|e| e.name == "Inside").unwrap();
            let (index, flow) = scene
                .elements
                .iter()
                .enumerate()
                .find(|(_, e)| e.kind == "activity-flow" && e.target == action.id)
                .unwrap();
            let Shape::Polygon(head, Paint::Ink) = &flow.shapes[1] else {
                panic!("missing incoming arrowhead");
            };
            let bounds = box_of(action);
            // then
            assert_eq!(head[0].y, bounds.y);
            assert_eq!(head[0].x, bounds.x + bounds.width / 2.);
            for (container_index, container) in scene
                .elements
                .iter()
                .enumerate()
                .filter(|(_, e)| matches!(e.kind, "activity-partition" | "activity-group"))
            {
                assert_inside(&flow.shapes[1], box_of(container));
                assert!(
                    container_index < index,
                    "opaque {} hides incoming arrowhead",
                    container.kind
                );
            }
        }
    }

    #[test]
    fn should_connect_labeled_join_bar_directly_to_following_instruction() {
        // given
        let source = "start\nfork\n:Left;\nfork again\n:Right;\nend fork {all}\n:After;\nstop";
        // when
        let scene = draw(source);
        let join = scene
            .elements
            .iter()
            .find(|e| e.kind == "activity-join")
            .unwrap();
        let after = scene.elements.iter().find(|e| e.name == "After").unwrap();
        let flow = scene
            .elements
            .iter()
            .find(|e| e.owner == join.id && e.target == after.id)
            .unwrap();
        let Shape::Line(points, _) = &flow.shapes[0] else {
            panic!("missing join output path");
        };
        let bar = box_of(join);
        // then
        assert_eq!(points[0].x, bar.x + bar.width / 2.);
        assert_eq!(
            points[0].y,
            bar.y + bar.height,
            "output path is detached from join bar"
        );
        assert_eq!(points.last().unwrap().y, box_of(after).y);
        let Shape::Text(label_at, label, _) = &join.shapes[1] else {
            panic!("missing join label");
        };
        assert!(
            label_at.x > points[0].x || label_at.x + label.width < points[0].x,
            "label overlaps output stem"
        );
    }

    #[test]
    fn should_route_nested_group_inputs_clear_of_long_captions() {
        // given
        let source = "start\npartition \"A long outer partition caption across the centerline\" {\ngroup \"A long inner group caption across the centerline\" {\n:Inside;\n}\n}\nstop";
        // when
        let scene = draw(source);
        // then
        for container in scene
            .elements
            .iter()
            .filter(|e| matches!(e.kind, "activity-partition" | "activity-group"))
        {
            let Shape::Text(origin, label, _) = &container.shapes[1] else {
                panic!("caption");
            };
            for flow in scene.elements.iter().filter(|e| e.kind == "activity-flow") {
                let Shape::Line(points, _) = &flow.shapes[0] else {
                    panic!("flow");
                };
                for segment in points.windows(2) {
                    let x0 = segment[0].x.min(segment[1].x);
                    let x1 = segment[0].x.max(segment[1].x);
                    let y0 = segment[0].y.min(segment[1].y);
                    let y1 = segment[0].y.max(segment[1].y);
                    assert!(
                        x1 < origin.x
                            || x0 > origin.x + label.width + 24.
                            || y1 < origin.y
                            || y0 > origin.y + label.height,
                        "flow {} crosses caption or its routing clearance: {}",
                        flow.id,
                        container.name
                    );
                }
            }
        }
    }

    #[test]
    fn should_separate_unequal_fork_lanes_and_span_join() {
        // given
        let wide = "A very wide lane action with substantially more text than the others";
        let source = format!("fork\n:{wide};\nfork again\n:B;\nfork again\n:C;\nend fork");
        // when
        let scene = draw(&source);
        let actions: Vec<_> = scene
            .elements
            .iter()
            .filter(|e| e.kind == "activity-action")
            .map(box_of)
            .collect();
        let join = box_of(
            scene
                .elements
                .iter()
                .find(|e| e.kind == "activity-join")
                .unwrap(),
        );
        // then
        assert!(actions[0].x + actions[0].width < actions[1].x);
        assert!(actions[1].x + actions[1].width < actions[2].x);
        assert!(join.x <= actions[0].x + actions[0].width / 2.);
        assert!(join.x + join.width >= actions[2].x + actions[2].width / 2.);
    }

    #[test]
    fn should_route_while_false_exit_and_live_back_edge() {
        // given
        let source = "while (more?) is (yes)\n:Work;\nendwhile (no)\n:Next;";
        // when
        let scene = draw(source);
        let test = scene
            .elements
            .iter()
            .find(|e| e.kind == "activity-while")
            .unwrap();
        let work = scene.elements.iter().find(|e| e.name == "Work").unwrap();
        let flows: Vec<_> = scene
            .elements
            .iter()
            .filter(|e| e.kind == "activity-flow")
            .collect();
        // then
        assert!(
            flows
                .iter()
                .any(|e| e.owner == test.id && e.target == work.id && e.name == "yes")
        );
        let back = flows
            .iter()
            .find(|e| e.owner == work.id && e.target == test.id)
            .unwrap();
        let Shape::Line(points, _) = &back.shapes[0] else {
            panic!("back route");
        };
        assert!(points.first().unwrap().y > points.last().unwrap().y);
        assert!(
            flows
                .iter()
                .any(|e| e.owner == test.id && e.target != work.id && e.name == "no")
        );
    }

    #[test]
    fn should_route_repeat_positive_back_and_negative_out() {
        // given
        let source = "repeat :Entry;\n:Work;\nrepeat while (retry?) is (again) not (done)\n:Next;";
        // when
        let scene = draw(source);
        let test = scene
            .elements
            .iter()
            .find(|e| e.kind == "activity-repeat")
            .unwrap();
        let entry = scene.elements.iter().find(|e| e.name == "Entry").unwrap();
        // then
        assert!(scene.elements.iter().any(|e| e.kind == "activity-flow"
            && e.owner == test.id
            && e.target == entry.id
            && e.name == "again"));
        assert!(scene.elements.iter().any(|e| e.kind == "activity-flow"
            && e.owner == test.id
            && e.target != entry.id
            && e.name == "done"));
    }

    #[test]
    fn should_contain_nested_partition_shapes_and_all_labels() {
        // given
        let source = "partition Outer {\ngroup Inner {\nwhile (more?) is (a very long positive guard)\n:Work;\nendwhile (a very long negative guard)\n}\n}";
        // when
        let scene = draw(source);
        // then
        for element in &scene.elements {
            if let Some(parent) = scene.elements.iter().find(|e| {
                e.id == element.owner && matches!(e.kind, "activity-partition" | "activity-group")
            }) {
                let bounds = box_of(parent);
                for shape in &element.shapes {
                    assert_inside(shape, bounds);
                }
            }
        }
    }

    fn assert_inside(shape: &Shape, bounds: Rect) {
        let points = match shape {
            Shape::Rect(r, _, _) | Shape::Ellipse(r, _) => vec![
                Point::new(r.x, r.y),
                Point::new(r.x + r.width, r.y + r.height),
            ],
            Shape::Text(p, label, _) => vec![*p, Point::new(p.x + label.width, p.y + label.height)],
            Shape::Line(points, _)
            | Shape::ColoredLine(points, _, _)
            | Shape::Polygon(points, _) => points.clone(),
        };
        for p in points {
            assert!(
                p.x >= bounds.x
                    && p.x <= bounds.x + bounds.width
                    && p.y >= bounds.y
                    && p.y <= bounds.y + bounds.height,
                "point {p:?} outside {bounds:?}"
            );
        }
    }

    #[test]
    fn should_stop_activity_composition_when_work_is_exhausted() {
        // given
        let mut budget = Budget::new(&RenderOptions::default());
        let model = crate::plantuml::parser::test_support::literal(
            "@startuml\nstart\nif (x)\n:A;\nelse\n:B;\nendif\nstop\n@enduml",
            &mut budget,
            crate::plantuml::parser::activity::parse,
        )
        .unwrap();
        budget.limits.layout_work = budget.work + 10;
        let session = RenderEnvironment::deterministic().begin_session().unwrap();
        // when
        let result = layout(&model, &mut Context::new(&session, &mut budget));
        // then
        assert!(result.is_err());
    }
}
