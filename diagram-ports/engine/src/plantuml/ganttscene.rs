//! Original measured calendar/bar layout, with MIT Gantt duration/diamond semantics.
//! Copyright 2009-2024 Arnaud Roques; exact source traces in provenance.
use super::{
    budget::{Budget, invalid},
    gantt::{CivilDay, Endpoint, Gantt, Kind},
    scene::{Dash, Decoration, Element, Paint, Point, Rect, Scene, Shape},
    text::{Context, Label},
};
use crate::RenderError;

struct Row {
    label: Label,
    top: f64,
    height: f64,
}
struct Tick {
    day: CivilDay,
    label: Label,
}
struct Layout {
    rows: Vec<Row>,
    ticks: Vec<Tick>,
    left: f64,
    scale: f64,
    top: f64,
    bottom: f64,
    min: CivilDay,
    max: CivilDay,
}
impl Layout {
    fn x(&self, day: CivilDay) -> f64 {
        self.left + (day.0 - self.min.0) as f64 * self.scale
    }
    fn center(&self, id: usize) -> f64 {
        self.rows[id].top + self.rows[id].height / 2.
    }
}

pub(super) fn layout(model: &Gantt, context: &mut Context<'_>) -> Result<Scene, RenderError> {
    let measured = measure(model, context)?;
    let mut scene = Scene::new();
    paint_axis(&measured, &mut scene, context.budget)?;
    for (id, task) in model.tasks.iter().enumerate() {
        context.budget.charge(1)?;
        context
            .budget
            .text(task.code.len().saturating_mul(2) + 64)?;
        let row = &measured.rows[id];
        let mut label = Element::new(format!("gantt-label-{id}"), "gantt-label");
        label.owner = task.code.clone();
        label.shapes.push(Shape::Text(
            Point::new(24., row.top + (row.height - row.label.height) / 2.),
            row.label.clone(),
            Decoration::Plain,
        ));
        scene.push(label, context.budget)?;
        paint_task((id, model), &measured, &mut scene, context.budget)?;
    }
    paint_dependencies(model, &measured, &mut scene, context.budget)?;
    Ok(scene)
}

fn measure(model: &Gantt, context: &mut Context<'_>) -> Result<Layout, RenderError> {
    let min = model.project_start;
    let mut max = min;
    let mut rows = Vec::new();
    let mut left: f64 = 0.;
    for task in &model.tasks {
        context.budget.item()?;
        context.budget.charge(64)?;
        max = max.max(task.end);
        let label = context.label(&task.display)?;
        left = left.max(label.width);
        rows.push(Row {
            height: label.height.max(24.) + 16.,
            label,
            top: 0.,
        });
    }
    let span = max.0 - min.0;
    let step = ((span + 11) / 12).max(1);
    let count = (span - 1) / step + 1;
    context.budget.charge(count as usize * 64)?;
    let mut ticks = Vec::new();
    for index in 0..count {
        context.budget.item()?;
        context.budget.text(32)?;
        let day = min.shift(index * step, 1)?;
        ticks.push(Tick {
            day,
            label: context.label(&tick_text(day))?,
        });
    }
    let mut scale = if span <= 31 { 40. } else { 640. / span as f64 };
    let mut tick_height: f64 = 0.;
    for tick in &ticks {
        context.budget.charge(1)?;
        scale = scale.max((tick.label.width + 24.) / step as f64);
        tick_height = tick_height.max(tick.label.height);
    }
    left += 64. + ticks.first().map_or(0., |tick| tick.label.width / 2.);
    let top = 24. + tick_height + 16.;
    let mut bottom = top;
    for row in &mut rows {
        context.budget.charge(1)?;
        row.top = bottom + 16.;
        bottom = row.top + row.height + 24.;
    }
    let right =
        left + span as f64 * scale + ticks.last().map_or(24., |tick| tick.label.width / 2. + 24.);
    if !right.is_finite() || right > 999_900. || bottom > 999_900. {
        return Err(invalid(
            1,
            format!("excessive Gantt measured geometry {right} by {bottom}"),
        ));
    }
    Ok(Layout {
        rows,
        ticks,
        left,
        scale,
        top,
        bottom,
        min,
        max,
    })
}

fn tick_text(day: CivilDay) -> String {
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    const DAYS: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
    let (year, month, date) = day.parts();
    format!(
        "{} {year}\n{} {date:02}",
        MONTHS[(month - 1) as usize],
        DAYS[(day.0 % 7) as usize]
    )
}
fn paint_axis(
    measured: &Layout,
    scene: &mut Scene,
    budget: &mut Budget,
) -> Result<(), RenderError> {
    budget.charge(4)?;
    let mut axis = Element::new("gantt-axis", "gantt-axis");
    axis.shapes.push(Shape::Line(
        vec![
            Point::new(measured.left, measured.top),
            Point::new(measured.x(measured.max), measured.top),
        ],
        Dash::Solid,
    ));
    scene.push(axis, budget)?;
    for (index, tick) in measured.ticks.iter().enumerate() {
        budget.charge(8)?;
        budget.text(32)?;
        let x = measured.x(tick.day) + measured.scale / 2.;
        let mut element = Element::new(format!("gantt-tick-{index}"), "gantt-tick");
        element.name = tick.day.iso();
        element.shapes.push(Shape::Text(
            Point::new(x - tick.label.width / 2., 24.),
            tick.label.clone(),
            Decoration::Plain,
        ));
        element.shapes.push(Shape::Line(
            vec![Point::new(x, measured.top), Point::new(x, measured.bottom)],
            Dash::Dashed,
        ));
        scene.push(element, budget)?;
    }
    Ok(())
}
fn paint_task(
    target: (usize, &Gantt),
    measured: &Layout,
    scene: &mut Scene,
    budget: &mut Budget,
) -> Result<(), RenderError> {
    let (id, model) = target;
    let task = &model.tasks[id];
    budget.charge(16)?;
    let kind = match task.kind {
        Kind::Bar => "gantt-bar",
        Kind::Milestone => "gantt-milestone",
    };
    let mut element = Element::new(format!("gantt-task-{id}"), kind);
    element.owner = task.code.clone();
    element.name = match task.kind {
        Kind::Bar => format!("{}/{}", task.start.iso(), task.end.iso()),
        Kind::Milestone => task.start.iso(),
    };
    let y = measured.center(id);
    let x = measured.x(task.start);
    match task.kind {
        Kind::Bar => element.shapes.push(Shape::Rect(
            Rect {
                x,
                y: y - 10.,
                width: measured.x(task.end) - x,
                height: 20.,
            },
            2.,
            Paint::Node,
        )),
        Kind::Milestone => {
            let x = x + measured.scale / 2.;
            element.shapes.push(Shape::Polygon(
                vec![
                    Point::new(x, y - 10.),
                    Point::new(x + 10., y),
                    Point::new(x, y + 10.),
                    Point::new(x - 10., y),
                ],
                Paint::Node,
            ));
        }
    }
    scene.push(element, budget)
}
fn paint_dependencies(
    model: &Gantt,
    measured: &Layout,
    scene: &mut Scene,
    budget: &mut Budget,
) -> Result<(), RenderError> {
    for (index, edge) in model.dependencies.iter().enumerate() {
        budget.charge(32)?;
        let (from, to) = (&model.tasks[edge.from], &model.tasks[edge.to]);
        budget.text(from.code.len() + to.code.len() + 64)?;
        let kind = match edge.kind {
            Kind::Bar => "gantt-dependency",
            Kind::Milestone => "gantt-reference",
        };
        let mut element = Element::new(format!("gantt-edge-{index}"), kind);
        element.owner = from.code.clone();
        element.target = to.code.clone();
        element.name = format!("{}:{}", edge.endpoint.name(), edge.offset);
        if edge.kind == Kind::Bar {
            let boundary = match edge.endpoint {
                Endpoint::Start => from.start,
                Endpoint::End => from.end,
            };
            let (x, y) = (measured.x(boundary), measured.center(edge.from));
            let (tx, ty) = (measured.x(to.start), measured.center(edge.to));
            let route_y = measured.rows[edge.from].top + measured.rows[edge.from].height + 12.;
            element.shapes.push(Shape::Line(
                vec![
                    Point::new(x, y),
                    Point::new(x, route_y),
                    Point::new(tx - 12., route_y),
                    Point::new(tx - 12., ty),
                    Point::new(tx, ty),
                ],
                Dash::Solid,
            ));
            element.shapes.push(Shape::Polygon(
                vec![
                    Point::new(tx, ty),
                    Point::new(tx - 6., ty - 4.),
                    Point::new(tx - 6., ty + 4.),
                ],
                Paint::Ink,
            ));
        }
        scene.push(element, budget)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RenderOptions;
    use merman_render::environment::RenderEnvironment;

    fn inspect(source: &str, verify: impl FnOnce(Gantt, Layout, Scene)) {
        let mut budget = Budget::new(&RenderOptions::default());
        let (preamble, model) = crate::plantuml::parser::test_support::styled(
            source,
            &mut budget,
            crate::plantuml::parser::gantt::parse,
        )
        .unwrap();
        let session = RenderEnvironment::deterministic().begin_session().unwrap();
        let mut context = Context::new(&session, &mut budget).with_fonts(preamble.fonts);
        let measured = measure(&model, &mut context).unwrap();
        let scene = layout(&model, &mut context).unwrap();
        verify(model, measured, scene);
    }
    #[test]
    fn should_project_duration_bars_and_civil_day_diamonds_on_one_axis() {
        // given
        let source = include_str!("../../../fixtures/plantuml/ganttalias.puml");
        // when
        inspect(source, |model, measured, scene| {
            // then
            for (id, task) in model.tasks.iter().enumerate() {
                let shape = &scene
                    .elements
                    .iter()
                    .find(|e| e.id == format!("gantt-task-{id}"))
                    .unwrap()
                    .shapes[0];
                match shape {
                    Shape::Rect(rect, _, _) => {
                        assert_eq!(rect.x, measured.x(task.start));
                        assert!(
                            (rect.width - (task.end.0 - task.start.0) as f64 * measured.scale)
                                .abs()
                                < 0.000001
                        );
                    }
                    Shape::Polygon(points, _) => {
                        assert_eq!(points.len(), 4);
                        assert_eq!(points[0].x, measured.x(task.start) + measured.scale / 2.);
                        assert_eq!(points[0].x, points[2].x);
                        assert_eq!(points[1].y, points[3].y);
                    }
                    _ => panic!("wrong task shape"),
                }
            }
            let edge = scene
                .elements
                .iter()
                .find(|e| e.kind == "gantt-dependency")
                .unwrap();
            let Shape::Line(points, _) = &edge.shapes[0] else {
                panic!("missing route")
            };
            assert_eq!(points[0].x, measured.x(model.tasks[0].end));
            assert_eq!(points[0].y, measured.center(0));
            assert_eq!(points.last().unwrap().x, measured.x(model.tasks[1].start));
            assert_eq!(points.last().unwrap().y, measured.center(1));
            assert_eq!(edge.owner, "draft");
            assert_eq!(edge.target, "build");
            assert!(
                scene
                    .elements
                    .iter()
                    .filter(|e| e.kind == "gantt-reference")
                    .all(|e| e.shapes.is_empty())
            );
        });
    }
    #[test]
    fn should_reserve_large_multiline_labels_ticks_and_rows_without_overlap() {
        // given
        for size in [6, 96] {
            let source = format!(
                "@startgantt\nskinparam defaultFontSize {size}\nProject starts 2028-02-28\n[A very long display label\\nwith another row] as [A] lasts 3 days\n[Next long display] as [B] starts at [A]'s end\n[M] happens at [A]'s end\n@endgantt"
            );
            // when
            inspect(&source, |_, measured, scene| {
                // then
                for row in &measured.rows {
                    assert!(row.label.width + 24. < measured.left - 12.);
                    assert!(row.top > measured.top);
                    assert!(row.label.height < row.height);
                }
                for pair in measured.rows.windows(2) {
                    assert!(pair[0].top + pair[0].height < pair[1].top);
                }
                for pair in measured.ticks.windows(2) {
                    assert!(
                        measured.x(pair[0].day) + pair[0].label.width / 2. + 8.
                            < measured.x(pair[1].day) - pair[1].label.width / 2.
                    );
                }
                for element in scene.elements {
                    for shape in element.shapes {
                        if let Shape::Text(p, label, _) = shape {
                            assert!(p.x >= scene.min_x && p.y >= scene.min_y);
                            assert!(
                                p.x + label.width < scene.width
                                    && p.y + label.height < scene.height
                            );
                        }
                    }
                }
            });
        }
    }
}
