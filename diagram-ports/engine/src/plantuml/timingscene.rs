//! Native bounded linear timing projection and measured signals/dimension reservations.
//! Waveform/ribbon semantics: MIT timingdiagram graphic sources, Arnaud Roques 2009-2024.
use super::{
    budget::{Budget, invalid},
    scene::{Dash, Decoration, Element, Paint, Point, Scene, Shape},
    text::{Context, Label},
    timing::{Endpoint, Initial, Timing, Track, TrackKind, search_cost},
};
use crate::RenderError;

const GAP: f64 = 8.;

struct TrackLayout {
    title: Label,
    states: Vec<Label>,
    active: Vec<usize>,
    levels: Vec<f64>,
    top: f64,
    signal_top: f64,
    bottom: f64,
    ribbon_height: f64,
}
struct ConnectionLabel {
    label: Label,
    y: f64,
}
struct Layout {
    tracks: Vec<TrackLayout>,
    constraints: Vec<ConnectionLabel>,
    messages: Vec<ConnectionLabel>,
    ticks: Vec<(i64, Label)>,
    axis: Axis,
    axis_y: f64,
    initial_width: f64,
    tail: f64,
}
struct Axis {
    min: i64,
    max: i64,
    left: f64,
    scale: f64,
}
impl Axis {
    fn x(&self, time: i64) -> f64 {
        self.left + (time - self.min) as f64 * self.scale
    }
}

pub(super) fn layout(model: &Timing, context: &mut Context<'_>) -> Result<Scene, RenderError> {
    let measured = measure(model, context)?;
    let mut scene = Scene::new();
    paint_axis(&measured, &mut scene, context.budget)?;
    for (id, track) in model.tracks.iter().enumerate() {
        paint_track((id, track), &measured, &mut scene, context.budget)?;
    }
    paint_constraints(model, &measured, &mut scene, context.budget)?;
    paint_messages(model, &measured, &mut scene, context.budget)?;
    Ok(scene)
}

fn measure(model: &Timing, context: &mut Context<'_>) -> Result<Layout, RenderError> {
    let mut tracks = Vec::new();
    for track in &model.tracks {
        tracks.push(measure_track(track, context)?);
    }
    let mut constraints = Vec::new();
    for constraint in &model.constraints {
        context.budget.item()?;
        constraints.push(ConnectionLabel {
            label: context.label(&constraint.label)?,
            y: 0.,
        });
    }
    let mut messages = Vec::new();
    for message in &model.messages {
        context.budget.item()?;
        messages.push(ConnectionLabel {
            label: context.label(&message.label)?,
            y: 0.,
        });
    }
    let mut ticks = Vec::new();
    for time in select_ticks(
        (model.marks[0], *model.marks.last().unwrap()),
        context.budget,
    )? {
        context.budget.item()?;
        ticks.push((time, context.label(&time.to_string())?));
    }
    let axis = Axis {
        min: model.marks[0],
        max: *model.marks.last().unwrap(),
        left: 0.,
        scale: 0.,
    };
    let mut measured = Layout {
        tracks,
        constraints,
        messages,
        ticks,
        axis,
        axis_y: 0.,
        initial_width: 40.,
        tail: 40.,
    };
    reserve_width(model, &mut measured, context.budget)?;
    reserve_height(model, &mut measured, context.budget)?;
    Ok(measured)
}

fn measure_track(track: &Track, context: &mut Context<'_>) -> Result<TrackLayout, RenderError> {
    context.budget.item()?;
    let title = context.label(&track.display)?;
    let mut states = Vec::new();
    for state in &track.states {
        states.push(context.label(state)?);
    }
    let mut active = std::collections::BTreeSet::new();
    for state in track
        .changes
        .iter()
        .map(|change| change.state)
        .chain(match track.initial {
            Initial::Absent => None,
            Initial::State(state) => Some(state),
        })
    {
        context.budget.charge(search_cost(active.len()))?;
        if !active.contains(&state) {
            context.budget.item()?;
            active.insert(state);
        }
    }
    context.budget.charge(active.len())?;
    let active: Vec<_> = active.into_iter().collect();
    let ribbon_height = active
        .iter()
        .fold(0_f64, |height, &id| height.max(states[id].height))
        + 2. * GAP;
    context.budget.charge(states.len())?;
    let levels = vec![0.; states.len()];
    Ok(TrackLayout {
        title,
        states,
        active,
        levels,
        top: 0.,
        signal_top: 0.,
        bottom: 0.,
        ribbon_height,
    })
}

fn reserve_width(
    model: &Timing,
    measured: &mut Layout,
    budget: &mut Budget,
) -> Result<(), RenderError> {
    let span = (measured.axis.max - measured.axis.min).max(1) as f64;
    let mut left: f64 = 0.;
    let mut scale = 640. / span;
    for (track, layout) in model.tracks.iter().zip(&measured.tracks) {
        budget.charge(layout.active.len() + track.changes.len() + 1)?;
        left = left.max(layout.title.width);
        if track.kind == TrackKind::Robust {
            for &state in &layout.active {
                left = left.max(layout.states[state].width);
            }
        }
        if let Initial::State(state) = track.initial {
            measured.initial_width = measured
                .initial_width
                .max(layout.states[state].width + 4. * GAP);
        }
        for (index, change) in track.changes.iter().enumerate() {
            if track.kind == TrackKind::Concise {
                if let Some(next) = track.changes.get(index + 1) {
                    scale = scale.max(
                        (layout.states[change.state].width + 4. * GAP)
                            / (next.time - change.time) as f64,
                    );
                } else {
                    measured.tail = measured
                        .tail
                        .max(layout.states[change.state].width + 4. * GAP);
                }
            }
        }
    }
    for (constraint, label) in model.constraints.iter().zip(&measured.constraints) {
        budget.charge(1)?;
        scale =
            scale.max((label.label.width + 4. * GAP) / (constraint.end - constraint.start) as f64);
    }
    for pair in measured.ticks.windows(2) {
        budget.charge(1)?;
        scale = scale.max(
            ((pair[0].1.width + pair[1].1.width) / 2. + 2. * GAP) / (pair[1].0 - pair[0].0) as f64,
        );
    }
    for label in &measured.messages {
        budget.charge(1)?;
        scale = scale.max((label.label.width + 4. * GAP) / span);
        if measured.axis.min == measured.axis.max {
            measured.tail = measured.tail.max(label.label.width + 4. * GAP);
        }
    }
    let tick_half = measured
        .ticks
        .first()
        .map_or(0., |(_, label)| label.width / 2.);
    measured.axis.left = left + measured.initial_width + tick_half + 6. * GAP;
    measured.axis.scale = scale;
    if measured.axis.x(measured.axis.max) + measured.tail > 999_000. {
        return Err(invalid(
            1,
            "timing measured interval width exceeds coordinate limit",
        ));
    }
    Ok(())
}

fn reserve_height(
    model: &Timing,
    measured: &mut Layout,
    budget: &mut Budget,
) -> Result<(), RenderError> {
    budget.charge(model.tracks.len().saturating_mul(2))?;
    let mut constraint_rows = vec![Vec::new(); model.tracks.len()];
    let mut message_rows = vec![Vec::new(); model.tracks.len()];
    for (id, constraint) in model.constraints.iter().enumerate() {
        budget.charge(1)?;
        constraint_rows[constraint.track].push(id);
    }
    for (id, message) in model.messages.iter().enumerate() {
        budget.charge(1)?;
        message_rows[message.from.track.min(message.to.track)].push(id);
    }
    let mut y = 3. * GAP;
    for (id, layout) in measured.tracks.iter_mut().enumerate() {
        budget.charge(layout.active.len() + 1)?;
        layout.top = y;
        y += layout.title.height + 2. * GAP;
        for &constraint in &constraint_rows[id] {
            measured.constraints[constraint].y = y;
            y += measured.constraints[constraint].label.height + 3. * GAP;
        }
        layout.signal_top = y;
        if model.tracks[id].kind == TrackKind::Robust {
            for &state in &layout.active {
                layout.levels[state] = y + layout.states[state].height / 2.;
                y += layout.states[state].height + 2. * GAP;
            }
        } else {
            for &state in &layout.active {
                layout.levels[state] = y + layout.ribbon_height / 2.;
            }
            y += layout.ribbon_height + GAP;
        }
        layout.bottom = y;
        y += 2. * GAP;
        for &message in &message_rows[id] {
            measured.messages[message].y = y;
            y += measured.messages[message].label.height + 3. * GAP;
        }
        y += 2. * GAP;
        if y > 999_000. {
            return Err(invalid(
                1,
                "timing measured track height exceeds coordinate limit",
            ));
        }
    }
    measured.axis_y = y;
    Ok(())
}

fn select_ticks(range: (i64, i64), budget: &mut Budget) -> Result<Vec<i64>, RenderError> {
    let span = range.1 - range.0;
    if span == 0 {
        budget.item()?;
        return Ok(vec![range.0]);
    }
    let target = (span + 7) / 8;
    let mut power = 1_i64;
    while power * 10 < target {
        budget.charge(1)?;
        power *= 10;
    }
    let step = [power, power * 2, power * 5, power * 10]
        .into_iter()
        .find(|&value| value >= target)
        .unwrap();
    let mut result = Vec::new();
    budget.item()?;
    result.push(range.0);
    let mut time = (range.0.div_euclid(step) + 1) * step;
    while time < range.1 {
        budget.charge(1)?;
        if time - range.0 >= step / 2 && range.1 - time >= step / 2 {
            budget.item()?;
            result.push(time);
        }
        time += step;
    }
    budget.item()?;
    result.push(range.1);
    Ok(result)
}

fn paint_axis(layout: &Layout, scene: &mut Scene, budget: &mut Budget) -> Result<(), RenderError> {
    let mut axis = element("timing-axis", "timing-axis", budget)?;
    line(
        &mut axis,
        &[
            Point::new(layout.axis.left, layout.axis_y),
            Point::new(layout.axis.x(layout.axis.max) + layout.tail, layout.axis_y),
        ],
        budget,
    )?;
    scene.push(axis, budget)?;
    for (id, (time, label)) in layout.ticks.iter().enumerate() {
        let mut tick = element(format!("tick{id}"), "timing-tick", budget)?;
        tick.name = time.to_string();
        let x = layout.axis.x(*time);
        line(
            &mut tick,
            &[
                Point::new(x, layout.axis_y - 4.),
                Point::new(x, layout.axis_y + 4.),
            ],
            budget,
        )?;
        text(
            &mut tick,
            Point::new(x - label.width / 2., layout.axis_y + GAP),
            label,
            budget,
        )?;
        scene.push(tick, budget)?;
    }
    Ok(())
}

fn paint_track(
    entry: (usize, &Track),
    layout: &Layout,
    scene: &mut Scene,
    budget: &mut Budget,
) -> Result<(), RenderError> {
    let (id, track) = entry;
    let measured = &layout.tracks[id];
    let kind = match track.kind {
        TrackKind::Robust => "timing-robust",
        TrackKind::Concise => "timing-concise",
    };
    let mut signal = element(format!("track{id}"), kind, budget)?;
    budget.text(track.code.len())?;
    signal.name = track.code.clone();
    text(
        &mut signal,
        Point::new(3. * GAP, measured.top),
        &measured.title,
        budget,
    )?;
    if track.kind == TrackKind::Robust {
        for &state in &measured.active {
            text(
                &mut signal,
                Point::new(
                    layout.axis.left
                        - layout.initial_width
                        - measured.states[state].width
                        - 2. * GAP,
                    measured.levels[state] - measured.states[state].height / 2.,
                ),
                &measured.states[state],
                budget,
            )?;
        }
    }
    paint_intervals(
        track,
        measured,
        &layout.axis,
        (layout.initial_width, layout.tail),
        &mut signal,
        budget,
    )?;
    scene.push(signal, budget)?;
    if let Initial::State(state) = track.initial {
        let mut initial = element(format!("initial{id}"), "timing-change", budget)?;
        budget.text(track.code.len() + track.states[state].len() + 8)?;
        initial.owner = track.code.clone();
        initial.name = format!("initial:{}", track.states[state]);
        scene.push(initial, budget)?;
    }
    for (index, change) in track.changes.iter().enumerate() {
        let mut event = element(format!("change{id}_{index}"), "timing-change", budget)?;
        budget.text(track.code.len() + track.states[change.state].len() + 13)?;
        event.owner = track.code.clone();
        event.name = format!("{}:{}", change.time, track.states[change.state]);
        scene.push(event, budget)?;
    }
    Ok(())
}

fn paint_intervals(
    track: &Track,
    measured: &TrackLayout,
    axis: &Axis,
    margins: (f64, f64),
    signal: &mut Element,
    budget: &mut Budget,
) -> Result<(), RenderError> {
    let mut previous = match track.initial {
        Initial::Absent => None,
        Initial::State(state) => Some((axis.left - margins.0, state)),
    };
    for change in &track.changes {
        budget.charge(1)?;
        let x = axis.x(change.time);
        if let Some((start, state)) = previous {
            interval((start, x), state, track.kind, measured, signal, budget)?;
            if track.kind == TrackKind::Robust {
                line(
                    signal,
                    &[
                        Point::new(x, measured.levels[state]),
                        Point::new(x, measured.levels[change.state]),
                    ],
                    budget,
                )?;
            }
        }
        previous = Some((x, change.state));
    }
    if let Some((start, state)) = previous {
        interval(
            (start, axis.x(axis.max) + margins.1),
            state,
            track.kind,
            measured,
            signal,
            budget,
        )?;
    }
    Ok(())
}

fn interval(
    range: (f64, f64),
    state: usize,
    kind: TrackKind,
    measured: &TrackLayout,
    signal: &mut Element,
    budget: &mut Budget,
) -> Result<(), RenderError> {
    let (left, right) = range;
    let y = measured.levels[state];
    if kind == TrackKind::Robust {
        return line(signal, &[Point::new(left, y), Point::new(right, y)], budget);
    }
    budget.item()?;
    let height = measured.ribbon_height / 2.;
    budget.charge(6)?;
    let tip = GAP.min((right - left) / 4.);
    signal.shapes.push(Shape::Polygon(
        vec![
            Point::new(left, y),
            Point::new(left + tip, y - height),
            Point::new(right - tip, y - height),
            Point::new(right, y),
            Point::new(right - tip, y + height),
            Point::new(left + tip, y + height),
        ],
        Paint::Node,
    ));
    text(
        signal,
        Point::new(
            (left + right - measured.states[state].width) / 2.,
            y - measured.states[state].height / 2.,
        ),
        &measured.states[state],
        budget,
    )
}

fn paint_constraints(
    model: &Timing,
    layout: &Layout,
    scene: &mut Scene,
    budget: &mut Budget,
) -> Result<(), RenderError> {
    for (id, constraint) in model.constraints.iter().enumerate() {
        let measured = &layout.constraints[id];
        let y = measured.y + measured.label.height + GAP;
        let a = Point::new(layout.axis.x(constraint.start), y);
        let b = Point::new(layout.axis.x(constraint.end), y);
        let mut item = element(format!("constraint{id}"), "timing-constraint", budget)?;
        budget.text(model.tracks[constraint.track].code.len() + 24)?;
        item.owner = model.tracks[constraint.track].code.clone();
        item.name = format!("{}:{}", constraint.start, constraint.end);
        text(
            &mut item,
            Point::new((a.x + b.x - measured.label.width) / 2., measured.y),
            &measured.label,
            budget,
        )?;
        line(&mut item, &[a, b], budget)?;
        arrow(&mut item, b, a, budget)?;
        arrow(&mut item, a, b, budget)?;
        for point in [a, b] {
            line(
                &mut item,
                &[
                    Point::new(point.x, y + GAP),
                    Point::new(point.x, layout.tracks[constraint.track].signal_top),
                ],
                budget,
            )?;
        }
        scene.push(item, budget)?;
    }
    Ok(())
}

fn project(
    endpoint: Endpoint,
    model: &Timing,
    layout: &Layout,
    budget: &mut Budget,
) -> Result<Point, RenderError> {
    let track = &model.tracks[endpoint.track];
    budget.charge(search_cost(track.changes.len()))?;
    let state = track
        .state_at(endpoint.time)
        .ok_or_else(|| invalid(1, "timing endpoint lacks state after validation"))?;
    Ok(Point::new(
        layout.axis.x(endpoint.time),
        layout.tracks[endpoint.track].levels[state],
    ))
}

fn paint_messages(
    model: &Timing,
    layout: &Layout,
    scene: &mut Scene,
    budget: &mut Budget,
) -> Result<(), RenderError> {
    for (id, message) in model.messages.iter().enumerate() {
        let label = &layout.messages[id];
        let a = project(message.from, model, layout, budget)?;
        let b = project(message.to, model, layout, budget)?;
        let route = label.y + label.label.height + GAP;
        let mut item = element(format!("message{id}"), "timing-message", budget)?;
        budget.text(
            model.tracks[message.from.track].code.len()
                + model.tracks[message.to.track].code.len()
                + 24,
        )?;
        item.owner = model.tracks[message.from.track].code.clone();
        item.target = model.tracks[message.to.track].code.clone();
        item.name = format!("{}:{}", message.from.time, message.to.time);
        let x = ((a.x + b.x - label.label.width) / 2.).clamp(
            layout.axis.left,
            layout.axis.x(layout.axis.max) + layout.tail - label.label.width,
        );
        text(&mut item, Point::new(x, label.y), &label.label, budget)?;
        line(
            &mut item,
            &[a, Point::new(a.x, route), Point::new(b.x, route), b],
            budget,
        )?;
        arrow(&mut item, Point::new(b.x, route), b, budget)?;
        scene.push(item, budget)?;
    }
    Ok(())
}

fn element(
    id: impl Into<String>,
    kind: &'static str,
    budget: &mut Budget,
) -> Result<Element, RenderError> {
    budget.item()?;
    Ok(Element::new(id, kind))
}
fn text(
    element: &mut Element,
    at: Point,
    label: &Label,
    budget: &mut Budget,
) -> Result<(), RenderError> {
    budget.item()?;
    element
        .shapes
        .push(Shape::Text(at, label.clone(), Decoration::Plain));
    Ok(())
}
fn line(element: &mut Element, points: &[Point], budget: &mut Budget) -> Result<(), RenderError> {
    budget.item()?;
    budget.charge(points.len())?;
    element
        .shapes
        .push(Shape::Line(points.to_vec(), Dash::Solid));
    Ok(())
}
fn arrow(
    element: &mut Element,
    from: Point,
    to: Point,
    budget: &mut Budget,
) -> Result<(), RenderError> {
    budget.item()?;
    let length = (to.x - from.x).hypot(to.y - from.y);
    if length == 0. {
        return Err(invalid(1, "zero-length timing arrow projection"));
    }
    let dx = (to.x - from.x) / length;
    let dy = (to.y - from.y) / length;
    budget.charge(3)?;
    element.shapes.push(Shape::Polygon(
        vec![
            to,
            Point::new(to.x - 7. * dx + 3. * dy, to.y - 7. * dy - 3. * dx),
            Point::new(to.x - 7. * dx - 3. * dy, to.y - 7. * dy + 3. * dx),
        ],
        Paint::Ink,
    ));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RenderOptions;
    use merman_render::environment::RenderEnvironment;

    fn inspect(source: &str, verify: impl FnOnce(Timing, Layout, Scene)) {
        let mut budget = Budget::new(&RenderOptions::default());
        let (preamble, model) = crate::plantuml::parser::test_support::styled(
            source,
            &mut budget,
            crate::plantuml::parser::timing::parse,
        )
        .unwrap();
        let session = RenderEnvironment::deterministic().begin_session().unwrap();
        let mut context = Context::new(&session, &mut budget).with_fonts(preamble.fonts);
        let measured = measure(&model, &mut context).unwrap();
        let scene = layout(&model, &mut context).unwrap();
        verify(model, measured, scene);
    }

    #[test]
    fn should_share_one_monotonic_time_projection_across_tracks() {
        // given
        let source = "@startuml\nrobust V\nconcise C\n@-10\nV is Off\nC is Waiting\n@0\nV is On\nC is Running\n@10\nV is Off\nC is Done\n@20\n@enduml";
        // when
        inspect(source, |model, measured, scene| {
            // then
            assert!(
                ((measured.axis.x(0) - measured.axis.x(-10))
                    - (measured.axis.x(10) - measured.axis.x(0)))
                .abs()
                    < 0.000_001
            );
            for pair in model.marks.windows(2) {
                assert!(measured.axis.x(pair[0]) < measured.axis.x(pair[1]));
            }
            let robust = scene
                .elements
                .iter()
                .find(|element| element.kind == "timing-robust")
                .unwrap();
            let concise = scene
                .elements
                .iter()
                .find(|element| element.kind == "timing-concise")
                .unwrap();
            let vertical = robust
                .shapes
                .iter()
                .filter_map(|shape| match shape {
                    Shape::Line(points, _) if points[0].x == points[1].x => Some(points[0].x),
                    _ => None,
                })
                .collect::<Vec<_>>();
            assert_eq!(vertical, [measured.axis.x(0), measured.axis.x(10)]);
            let ribbons = concise
                .shapes
                .iter()
                .filter_map(|shape| match shape {
                    Shape::Polygon(points, _) => Some(points),
                    _ => None,
                })
                .collect::<Vec<_>>();
            assert_eq!(ribbons.len(), 3);
            assert_eq!(ribbons[0][3].x, ribbons[1][0].x);
            assert_eq!(ribbons[1][3].x, ribbons[2][0].x);
            assert_eq!(ribbons[1][0].x, vertical[0]);
            assert_eq!(ribbons[2][0].x, vertical[1]);
        });
    }

    #[test]
    fn should_reserve_measured_constraint_caption_height_and_ribbon_labels() {
        // given
        for size in [6, 96] {
            let source = format!(
                "@startuml\nskinparam defaultFontSize {size}\nrobust \"Long valve title\" as V\nconcise \"Long controller title\" as C\n@0\nV is Closed\nC is \"Wide initial label\"\n@1\nV is Open\nC is \"Wide second label\"\n@2\nV@0 <-> @1 : **Wide** constraint\\nsecond row\nV@0 <-> @2 : second dimension\n@enduml"
            );
            // when
            inspect(&source, |model, measured, scene| {
                // then
                let constraints = &measured.constraints;
                assert!(
                    constraints[0].y + constraints[0].label.height + 3. * GAP <= constraints[1].y
                );
                assert!(
                    constraints[1].y + constraints[1].label.height + 3. * GAP
                        <= measured.tracks[0].signal_top
                );
                assert!(measured.tracks[0].bottom < measured.tracks[1].top);
                let concise = scene
                    .elements
                    .iter()
                    .find(|element| element.kind == "timing-concise")
                    .unwrap();
                let mut ribbon = None;
                for shape in &concise.shapes {
                    match shape {
                        Shape::Polygon(points, _) => ribbon = Some(points),
                        Shape::Text(at, label, _) if ribbon.is_some() => {
                            let points = ribbon.unwrap();
                            assert!(at.x >= points[1].x && at.x + label.width <= points[2].x);
                            assert!(at.y > points[1].y && at.y + label.height < points[4].y);
                        }
                        _ => {}
                    }
                }
                assert_eq!(model.constraints.len(), 2);
                for element in &scene.elements {
                    for shape in &element.shapes {
                        if let Shape::Text(at, label, _) = shape {
                            assert!(at.x >= scene.min_x && at.y >= scene.min_y);
                            assert!(
                                at.x + label.width <= scene.width
                                    && at.y + label.height <= scene.height
                            );
                        }
                    }
                }
            });
        }
    }

    #[test]
    fn should_attach_message_endpoints_to_their_tracks_and_right_hand_states() {
        // given
        let source = "@startuml\nrobust V\nconcise C\n@0\nV is Closed\nC is Waiting\n@6\nV is Open\n@15\nV@6 -> C@8 : confirm\\nsecond row\nC@0 -> V@0 : return\n@20\n@enduml";
        // when
        inspect(source, |model, measured, scene| {
            // then
            for (id, message) in model.messages.iter().enumerate() {
                let element = scene
                    .elements
                    .iter()
                    .find(|element| element.id == format!("message{id}"))
                    .unwrap();
                let points = element
                    .shapes
                    .iter()
                    .find_map(|shape| match shape {
                        Shape::Line(points, _) => Some(points),
                        _ => None,
                    })
                    .unwrap();
                assert_eq!(points[0].x, measured.axis.x(message.from.time));
                assert_eq!(points[3].x, measured.axis.x(message.to.time));
                assert_eq!(
                    points[0].y,
                    measured.tracks[message.from.track].levels[model.tracks[message.from.track]
                        .state_at(message.from.time)
                        .unwrap()]
                );
                assert_eq!(
                    points[3].y,
                    measured.tracks[message.to.track].levels[model.tracks[message.to.track]
                        .state_at(message.to.time)
                        .unwrap()]
                );
                assert!(measured.messages[id].y > measured.tracks[0].bottom);
                assert!(
                    measured.messages[id].y + measured.messages[id].label.height + 2. * GAP
                        < measured.tracks[1].top
                );
            }
            assert!(
                measured.messages[0].y + measured.messages[0].label.height + 2. * GAP
                    <= measured.messages[1].y
            );
        });
    }

    #[test]
    fn should_keep_initial_signal_before_axis_without_a_synthetic_zero_event() {
        // given
        let source = "@startuml\nrobust V\nV is Initial\nconcise C\nC is Before\n@10\nV is After\nC is Ready\n@20\n@enduml";
        // when
        inspect(source, |model, measured, scene| {
            // then
            assert_eq!(model.marks, [10, 20]);
            assert_eq!(model.tracks[0].changes[0].time, 10);
            for kind in ["timing-robust", "timing-concise"] {
                let element = scene
                    .elements
                    .iter()
                    .find(|element| element.kind == kind)
                    .unwrap();
                let start = element
                    .shapes
                    .iter()
                    .find_map(|shape| match shape {
                        Shape::Line(points, _) | Shape::Polygon(points, _) => Some(points[0].x),
                        _ => None,
                    })
                    .unwrap();
                assert_eq!(start, measured.axis.left - measured.initial_width);
            }
        });
    }

    #[test]
    fn should_bound_generated_ticks_before_allocation_and_check_cancellation() {
        // given
        let mut options = RenderOptions::default();
        options.limits.model_items = 2;
        let mut budget = Budget::new(&options);
        // when
        let error = select_ticks((-1_000_000_000, 1_000_000_000), &mut budget).unwrap_err();
        // then
        assert!(
            error
                .to_string()
                .contains("model items value 3 exceeds limit 2")
        );
        options.control.cancel();
        assert!(select_ticks((0, 100), &mut Budget::new(&options)).is_err());
        let mut budget = Budget::new(&RenderOptions::default());
        let ticks = select_ticks((-999_999_991, 10), &mut budget).unwrap();
        assert!(ticks.len() <= 12);
        assert!(budget.work < 30);
    }
}
