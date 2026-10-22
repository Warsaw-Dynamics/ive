//! PlantUML MIT 1.2026.9: sequencediagram command event semantics.
//! Lane geometry is original native code; exact source traces are in provenance/.
use super::{
    arrow::{self, Arrow, End},
    budget::{Budget, invalid, unsupported},
    lexer,
    scene::{Dash, Decoration, Element, Paint, Point, Rect, Scene, Shape, text_at},
    source::Line,
    text::{Context, Label, validate_label},
};
use crate::RenderError;

#[derive(Clone, Copy, Debug)]
pub(super) enum ParticipantKind {
    Participant,
    Actor,
    Database,
}
#[derive(Debug)]
pub(super) struct Participant {
    pub name: String,
    pub label: String,
    pub kind: ParticipantKind,
}
#[derive(Clone, Copy, Debug)]
enum Side {
    Left,
    Right,
    Over,
}
#[derive(Debug)]
enum Event {
    Message {
        from: usize,
        to: usize,
        arrow: Arrow,
        label: String,
    },
    Note {
        participant: usize,
        side: Side,
        label: String,
    },
    Open {
        kind: String,
        label: String,
    },
    Else(String),
    Close,
    Activate(usize),
    Deactivate(usize),
    Create(usize),
    Destroy(usize),
}
#[derive(Debug)]
pub(super) struct Sequence {
    pub participants: Vec<Participant>,
    events: Vec<Event>,
}
enum Life {
    Alive,
    Dead,
}
struct Parser {
    model: Sequence,
    active: Vec<usize>,
    life: Vec<Life>,
    frames: Vec<String>,
}

pub(super) fn parse(lines: &[Line], budget: &mut Budget) -> Result<Sequence, RenderError> {
    let mut parser = Parser {
        model: Sequence {
            participants: Vec::new(),
            events: Vec::new(),
        },
        active: Vec::new(),
        life: Vec::new(),
        frames: Vec::new(),
    };
    let mut index = 0;
    while index < lines.len() {
        budget.charge(1)?;
        let line = &lines[index];
        let (head, label) = lexer::split_label(line);
        let tokens = lexer::tokens(
            &Line {
                number: line.number,
                text: head.into(),
            },
            budget,
        )?;
        if tokens.is_empty() {
            return Err(invalid(line.number, "empty command"));
        }
        if tokens[0].text == "note" {
            let label = collect_note(lines, &mut index, label)?;
            parser.note(&tokens, &label, budget)?;
        } else {
            parser.command(line, &tokens, label, budget)?;
        }
        index += 1;
    }
    if !parser.frames.is_empty() || parser.active.iter().any(|n| *n > 0) {
        return Err(invalid(
            lines.last().unwrap().number,
            "unclosed group or activation",
        ));
    }
    if parser.model.participants.is_empty() {
        return Err(invalid(1, "sequence has no participants"));
    }
    Ok(parser.model)
}

fn collect_note(lines: &[Line], index: &mut usize, single: &str) -> Result<String, RenderError> {
    if lines[*index].text.contains(':') {
        return Ok(single.into());
    }
    let start = *index;
    let mut body = Vec::new();
    while *index + 1 < lines.len() {
        *index += 1;
        if lines[*index].text == "end note" {
            return Ok(body.join("\n"));
        }
        body.push(lines[*index].text.as_str());
    }
    Err(invalid(lines[start].number, "unterminated note"))
}

impl Parser {
    fn participant(&mut self, name: &str, budget: &mut Budget) -> Result<usize, RenderError> {
        budget.charge(self.model.participants.len())?;
        if let Some(index) = self.model.participants.iter().position(|p| p.name == name) {
            return Ok(index);
        }
        budget.item()?;
        let index = self.model.participants.len();
        self.model.participants.push(Participant {
            name: name.into(),
            label: name.into(),
            kind: ParticipantKind::Participant,
        });
        self.active.push(0);
        self.life.push(Life::Alive);
        Ok(index)
    }

    fn push(&mut self, event: Event, budget: &mut Budget) -> Result<(), RenderError> {
        budget.item()?;
        self.model.events.push(event);
        Ok(())
    }

    fn command(
        &mut self,
        line: &Line,
        tokens: &[lexer::Token],
        label: &str,
        budget: &mut Budget,
    ) -> Result<(), RenderError> {
        match tokens[0].text.as_str() {
            "participant" | "actor" | "database" => self.declare(line, tokens, budget),
            "activate" | "deactivate" | "create" | "destroy" => {
                self.lifecycle(line, tokens, budget)
            }
            "group" | "alt" | "opt" | "loop" | "par" | "break" | "critical" => {
                self.open(line, budget)
            }
            "else" => {
                validate_label(line.text[4..].trim(), line.number)?;
                if self.frames.last().is_none_or(|f| f != "alt" && f != "par") {
                    return Err(invalid(line.number, "else outside alt/par frame"));
                }
                self.push(Event::Else(line.text[4..].trim().into()), budget)
            }
            "end" if tokens.len() == 1 => {
                self.frames
                    .pop()
                    .ok_or_else(|| invalid(line.number, "end without group"))?;
                self.push(Event::Close, budget)
            }
            _ => self.message(line, tokens, label, budget),
        }
    }

    fn declare(
        &mut self,
        line: &Line,
        tokens: &[lexer::Token],
        budget: &mut Budget,
    ) -> Result<(), RenderError> {
        let (name, label) = lexer::name(&tokens[1..], line.number)?;
        validate_label(&label, line.number)?;
        budget.charge(self.model.participants.len())?;
        if self.model.participants.iter().any(|p| p.name == name) {
            return Err(invalid(
                line.number,
                format!("duplicate participant {name:?}"),
            ));
        }
        let id = self.participant(&name, budget)?;
        self.model.participants[id] = Participant {
            name,
            label,
            kind: match tokens[0].text.as_str() {
                "actor" => ParticipantKind::Actor,
                "database" => ParticipantKind::Database,
                _ => ParticipantKind::Participant,
            },
        };
        Ok(())
    }

    fn lifecycle(
        &mut self,
        line: &Line,
        tokens: &[lexer::Token],
        budget: &mut Budget,
    ) -> Result<(), RenderError> {
        if tokens.len() != 2 {
            return Err(unsupported(line.number, &line.text));
        }
        let existed = self
            .model
            .participants
            .iter()
            .any(|p| p.name == tokens[1].text);
        let id = self.participant(&tokens[1].text, budget)?;
        if matches!(self.life[id], Life::Dead) {
            return Err(invalid(line.number, "participant already destroyed"));
        }
        let event = match tokens[0].text.as_str() {
            "activate" => {
                self.active[id] += 1;
                budget.depth(self.active[id])?;
                Event::Activate(id)
            }
            "deactivate" => {
                self.active[id] = self.active[id]
                    .checked_sub(1)
                    .ok_or_else(|| invalid(line.number, "deactivate without activate"))?;
                Event::Deactivate(id)
            }
            "create" if !existed => Event::Create(id),
            "create" => return Err(invalid(line.number, "create requires a new participant")),
            _ => {
                if self.active[id] > 0 {
                    return Err(invalid(line.number, "destroy with unclosed activation"));
                }
                self.life[id] = Life::Dead;
                Event::Destroy(id)
            }
        };
        self.push(event, budget)
    }

    fn open(&mut self, line: &Line, budget: &mut Budget) -> Result<(), RenderError> {
        let (kind, label) = line.text.split_once(' ').unwrap_or((&line.text, ""));
        validate_label(label, line.number)?;
        self.frames.push(kind.into());
        budget.depth(self.frames.len())?;
        self.push(
            Event::Open {
                kind: kind.into(),
                label: label.into(),
            },
            budget,
        )
    }

    fn note(
        &mut self,
        tokens: &[lexer::Token],
        label: &str,
        budget: &mut Budget,
    ) -> Result<(), RenderError> {
        let (side, name) = match tokens {
            [_, side, of, name]
                if of.text == "of" && matches!(side.text.as_str(), "left" | "right") =>
            {
                (
                    if side.text == "left" {
                        Side::Left
                    } else {
                        Side::Right
                    },
                    &name.text,
                )
            }
            [_, over, name] if over.text == "over" => (Side::Over, &name.text),
            _ => {
                return Err(unsupported(
                    1,
                    "note positioning (expected left/right of or over one participant)",
                ));
            }
        };
        validate_label(label, 1)?;
        let participant = self.participant(name, budget)?;
        self.push(
            Event::Note {
                participant,
                side,
                label: label.into(),
            },
            budget,
        )
    }

    fn message(
        &mut self,
        line: &Line,
        tokens: &[lexer::Token],
        label: &str,
        budget: &mut Budget,
    ) -> Result<(), RenderError> {
        let [left, middle, right] = tokens else {
            return Err(unsupported(line.number, &line.text));
        };
        for endpoint in [left, right] {
            if !endpoint.quoted {
                lexer::validate_identifier(&endpoint.text, line.number)?;
            }
        }
        let mut arrow = arrow::parse(&middle.text, line.number)?;
        if !matches!(arrow.start, End::None | End::Open | End::Filled)
            || !matches!(arrow.end, End::None | End::Open | End::Filled)
        {
            return Err(unsupported(line.number, "sequence extremity"));
        }
        let reverse = arrow.start != End::None && arrow.end == End::None;
        if arrow.start != End::None && arrow.end != End::None {
            return Err(unsupported(line.number, "bidirectional sequence arrow"));
        }
        if reverse {
            arrow.end = arrow.start;
            arrow.start = End::None;
        }
        if middle.text.contains("--") {
            arrow.dash = Dash::Dashed;
        }
        validate_label(label, line.number)?;
        let a = self.participant(&left.text, budget)?;
        let b = self.participant(&right.text, budget)?;
        if matches!(self.life[a], Life::Dead) || matches!(self.life[b], Life::Dead) {
            return Err(invalid(
                line.number,
                "message references a destroyed participant",
            ));
        }
        let (from, to) = if reverse { (b, a) } else { (a, b) };
        self.push(
            Event::Message {
                from,
                to,
                arrow,
                label: label.into(),
            },
            budget,
        )
    }
}

pub(super) fn layout(model: &Sequence, context: &mut Context<'_>) -> Result<Scene, RenderError> {
    let heads = model
        .participants
        .iter()
        .map(|p| context.label(&p.label))
        .collect::<Result<Vec<_>, _>>()?;
    let labels = model
        .events
        .iter()
        .map(|event| context.label(event_label(event)))
        .collect::<Result<Vec<_>, _>>()?;
    let widest = heads
        .iter()
        .chain(&labels)
        .map(|label| label.width)
        .fold(80., f64::max);
    let step = widest + 110.;
    context.budget.charge(model.events.len())?;
    let margin = frame_depth(model) as f64 * 8.;
    let xs: Vec<f64> = (0..heads.len())
        .map(|i| margin + step * (i + 1) as f64)
        .collect();
    let mut scene = Scene::new();
    let mut rows = Vec::new();
    let mut y = heads.iter().map(|h| h.height).fold(35., f64::max) + 110.;
    for label in &labels {
        context.budget.charge(1)?;
        rows.push(y);
        y += label.height + 36.;
    }
    draw_lanes(model, &heads, &xs, &rows, y, &mut scene, context)?;
    draw_events(model, &labels, &xs, &rows, &mut scene, context)?;
    Ok(scene)
}

fn event_label(event: &Event) -> &str {
    match event {
        Event::Message { label, .. }
        | Event::Note { label, .. }
        | Event::Open { label, .. }
        | Event::Else(label) => label,
        _ => "",
    }
}

fn frame_depth(model: &Sequence) -> usize {
    model
        .events
        .iter()
        .fold((0usize, 0usize), |(depth, maximum), event| {
            let depth = match event {
                Event::Open { .. } => depth + 1,
                Event::Close => depth.saturating_sub(1),
                _ => depth,
            };
            (depth, maximum.max(depth))
        })
        .1
}

fn draw_lanes(
    model: &Sequence,
    heads: &[Label],
    xs: &[f64],
    rows: &[f64],
    end: f64,
    scene: &mut Scene,
    context: &mut Context<'_>,
) -> Result<(), RenderError> {
    for (id, participant) in model.participants.iter().enumerate() {
        context.budget.charge(model.events.len())?;
        let birth = model
            .events
            .iter()
            .position(|e| matches!(e, Event::Create(p) if *p == id))
            .map_or(24., |i| rows[i]);
        let death = model
            .events
            .iter()
            .position(|e| matches!(e, Event::Destroy(p) if *p == id))
            .map_or(end, |i| rows[i]);
        let height = heads[id].height
            + if matches!(participant.kind, ParticipantKind::Actor) {
                65.
            } else {
                24.
            };
        let rect = Rect {
            x: xs[id] - heads[id].width.max(60.) / 2. - 12.,
            y: birth,
            width: heads[id].width.max(60.) + 24.,
            height,
        };
        let mut lane = Element::new(format!("lane-{id}"), "lifeline");
        lane.owner = format!("p{id}");
        lane.shapes.push(Shape::Line(
            vec![
                Point::new(xs[id], birth + height),
                Point::new(xs[id], death),
            ],
            Dash::Dashed,
        ));
        scene.push(lane, context.budget)?;
        let mut head = Element::new(format!("p{id}"), "participant");
        head.name = participant.name.clone();
        head.shapes = super::shapes::node(
            rect,
            match participant.kind {
                ParticipantKind::Actor => "actor",
                ParticipantKind::Database => "database",
                _ => "participant",
            },
        );
        let label_rect = if matches!(participant.kind, ParticipantKind::Actor) {
            Rect {
                y: rect.y + 58.,
                height: rect.height - 58.,
                ..rect
            }
        } else {
            rect
        };
        head.shapes.push(text_at(label_rect, heads[id].clone()));
        scene.push(head, context.budget)?;
    }
    Ok(())
}

fn draw_events(
    model: &Sequence,
    labels: &[Label],
    xs: &[f64],
    rows: &[f64],
    scene: &mut Scene,
    context: &mut Context<'_>,
) -> Result<(), RenderError> {
    let mut frames: Vec<(usize, String)> = Vec::new();
    context.budget.charge(model.events.len())?;
    let maximum_depth = frame_depth(model);
    let frame_margin = labels.iter().map(|label| label.width).fold(80., f64::max) + 60.;
    let mut activations: Vec<Vec<f64>> = (0..xs.len()).map(|_| Vec::new()).collect();
    for (index, event) in model.events.iter().enumerate() {
        let y = rows[index];
        let mut element = Element::new(format!("event-{index}"), "event");
        match event {
            Event::Message {
                from, to, arrow, ..
            } => {
                element.kind = "message";
                element.owner = format!("p{from}");
                element.target = format!("p{to}");
                let points = if from == to {
                    vec![
                        Point::new(xs[*from], y),
                        Point::new(xs[*from] + 45., y),
                        Point::new(xs[*from] + 45., y + 20.),
                        Point::new(xs[*from], y + 20.),
                    ]
                } else {
                    vec![Point::new(xs[*from], y), Point::new(xs[*to], y)]
                };
                element.shapes = arrow::draw(points, *arrow);
                element.shapes.push(Shape::Text(
                    Point::new(xs[*from].min(xs[*to]) + 10., y - labels[index].height - 4.),
                    labels[index].clone(),
                    Decoration::Plain,
                ));
            }
            Event::Note {
                participant, side, ..
            } => {
                element.kind = "note";
                element.owner = format!("p{participant}");
                let width = labels[index].width + 24.;
                let x = match side {
                    Side::Left => xs[*participant] - width - 24.,
                    Side::Right => xs[*participant] + 24.,
                    Side::Over => xs[*participant] - width / 2.,
                };
                let rect = Rect {
                    x,
                    y: y - labels[index].height / 2.,
                    width,
                    height: labels[index].height + 16.,
                };
                element.shapes = super::shapes::node(rect, "note");
                element.shapes.push(text_at(rect, labels[index].clone()));
            }
            Event::Open { kind, .. } => {
                frames.push((index, kind.clone()));
                continue;
            }
            Event::Else(_) => {
                element.kind = "frame-else";
                let padding = (maximum_depth + 1 - frames.len()) as f64 * 8.;
                element.shapes.push(Shape::Line(
                    vec![
                        Point::new(xs[0] - frame_margin - padding, y),
                        Point::new(xs[xs.len() - 1] + frame_margin + padding, y),
                    ],
                    Dash::Dashed,
                ));
                element.shapes.push(Shape::Text(
                    Point::new(xs[0] - 50., y + 5.),
                    labels[index].clone(),
                    Decoration::Plain,
                ));
            }
            Event::Close => {
                let (start, kind) = frames
                    .pop()
                    .ok_or_else(|| invalid(1, "unbalanced frame model"))?;
                element.kind = "frame";
                element.name = kind.clone();
                let padding = (maximum_depth - frames.len()) as f64 * 8.;
                element.shapes.push(Shape::Rect(
                    Rect {
                        x: xs[0] - frame_margin - padding,
                        y: rows[start] - 24.,
                        width: xs[xs.len() - 1] - xs[0] + frame_margin * 2. + padding * 2.,
                        height: y - rows[start] + 40.,
                    },
                    0.,
                    Paint::None,
                ));
                element.shapes.push(Shape::Text(
                    Point::new(xs[0] - 55., rows[start] - 20.),
                    labels[start].clone(),
                    Decoration::Plain,
                ));
                element.shapes.push(Shape::Text(
                    Point::new(xs[0] - 55., rows[start] - 40.),
                    context.label(&kind)?,
                    Decoration::Plain,
                ));
            }
            Event::Activate(id) => {
                activations[*id].push(y);
                continue;
            }
            Event::Deactivate(id) => {
                let start = activations[*id]
                    .pop()
                    .ok_or_else(|| invalid(1, "unbalanced activation model"))?;
                element.kind = "activation";
                element.owner = format!("p{id}");
                element.shapes.push(Shape::Rect(
                    Rect {
                        x: xs[*id] - 5. + activations[*id].len() as f64 * 5.,
                        y: start,
                        width: 10.,
                        height: y - start,
                    },
                    0.,
                    Paint::Paper,
                ));
            }
            Event::Create(_) => continue,
            Event::Destroy(id) => {
                element.kind = "destroy";
                element.owner = format!("p{id}");
                element.shapes.push(Shape::Line(
                    vec![
                        Point::new(xs[*id] - 7., y - 7.),
                        Point::new(xs[*id] + 7., y + 7.),
                    ],
                    Dash::Solid,
                ));
                element.shapes.push(Shape::Line(
                    vec![
                        Point::new(xs[*id] + 7., y - 7.),
                        Point::new(xs[*id] - 7., y + 7.),
                    ],
                    Dash::Solid,
                ));
            }
        }
        scene.push(element, context.budget)?;
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::RenderOptions;
    use merman_render::environment::RenderEnvironment;

    #[test]
    fn should_preserve_lane_order_reverse_message_and_balanced_events() {
        // given
        let source = "@startuml\nparticipant Alpha as A\nA <- B : Response\nactivate A\nA -> A : Self\ndeactivate A\n@enduml";
        let mut budget = Budget::new(&RenderOptions::default());
        let lines = super::super::source::prepare(source, &mut budget).unwrap();
        // when
        let model = parse(&lines, &mut budget).unwrap();
        // then
        assert_eq!(
            model
                .participants
                .iter()
                .map(|p| p.name.as_str())
                .collect::<Vec<_>>(),
            ["A", "B"]
        );
        assert!(matches!(
            model.events[0],
            Event::Message { from: 1, to: 0, .. }
        ));
        assert!(matches!(model.events[1], Event::Activate(0)));
        assert!(matches!(
            model.events[2],
            Event::Message { from: 0, to: 0, .. }
        ));
        assert!(matches!(model.events[3], Event::Deactivate(0)));
    }

    #[test]
    fn should_contain_nested_frames_and_end_lifelines_at_destruction() {
        // given
        let source = "@startuml\nparticipant A\ngroup outer\nloop inner\ncreate B\nA -> B : Create\nactivate B\nB -> B : Work\nnote right of B : Inspect current work\ndeactivate B\ndestroy B\nend\nend\n@enduml";
        let mut budget = Budget::new(&RenderOptions::default());
        let lines = super::super::source::prepare(source, &mut budget).unwrap();
        let model = parse(&lines, &mut budget).unwrap();
        let session = RenderEnvironment::deterministic().begin_session().unwrap();
        // when
        let scene = layout(&model, &mut Context::new(&session, &mut budget)).unwrap();
        let frames = scene
            .elements
            .iter()
            .filter(|e| e.kind == "frame")
            .map(|e| match e.shapes[0] {
                Shape::Rect(r, _, _) => r,
                _ => panic!("frame is not a rectangle"),
            })
            .collect::<Vec<_>>();
        let lane = scene.elements.iter().find(|e| e.id == "lane-1").unwrap();
        let destruction = scene.elements.iter().find(|e| e.kind == "destroy").unwrap();
        // then
        assert!(frames[1].x < frames[0].x);
        assert!(frames[1].x + frames[1].width > frames[0].x + frames[0].width);
        assert!(frames[1].y < frames[0].y);
        assert!(frames[1].y + frames[1].height > frames[0].y + frames[0].height);
        let note = scene.elements.iter().find(|e| e.kind == "note").unwrap();
        let Shape::Polygon(points, _) = &note.shapes[0] else {
            panic!("note outline")
        };
        assert!(
            points
                .iter()
                .all(|p| p.x >= frames[0].x && p.x <= frames[0].x + frames[0].width),
            "note exceeds its interaction frame"
        );
        let Shape::Line(points, _) = &lane.shapes[0] else {
            panic!("lifeline")
        };
        let Shape::Line(cross, _) = &destruction.shapes[0] else {
            panic!("destruction")
        };
        assert!(points[0].y > 24.);
        assert_eq!(points[1].y, (cross[0].y + cross[1].y) / 2.);
    }
}
