//! PlantUML MIT 1.2026.9: sequencediagram command event semantics.
//! Lane geometry is original native code; exact source traces are in provenance/.
use super::{
    arrow::{self, Arrow},
    budget::{Budget, invalid},
    scene::{Dash, Decoration, Element, Paint, Point, Rect, Scene, Shape, text_at},
    style::Color,
    text::{Context, Label},
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
    pub stereotype: Stereotype,
}
#[derive(Clone, Debug)]
pub(in crate::plantuml) enum Spot {
    None,
    Marker(char, Color),
}
#[derive(Clone, Debug)]
pub(in crate::plantuml) enum Stereotype {
    None,
    Label { text: String, spot: Spot },
}
impl Stereotype {
    fn text_len(&self) -> usize {
        match self {
            Self::None => 0,
            Self::Label { text, .. } => text.len(),
        }
    }
}
#[derive(Clone, Debug)]
pub(in crate::plantuml) struct BoxHeader {
    pub title: String,
    pub color: Color,
}
#[derive(Debug)]
struct ParticipantBox {
    title: String,
    color: Color,
    members: Vec<usize>,
}
#[derive(Clone, Copy, Debug)]
pub(in crate::plantuml) enum Side {
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
    boxes: Vec<ParticipantBox>,
    events: Vec<Event>,
}
enum Life {
    Alive,
    Dead,
}
pub(in crate::plantuml) struct Builder<'b> {
    pub budget: &'b mut Budget,
    model: Sequence,
    active: Vec<usize>,
    life: Vec<Life>,
    frames: Vec<FrameKind>,
    open_boxes: Vec<usize>,
    note: String,
    retained_bytes: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::plantuml) enum FrameKind {
    Group,
    Alt,
    Opt,
    Loop,
    Par,
    Break,
    Critical,
    Literal(String),
}
impl FrameKind {
    pub fn name(&self) -> &str {
        match self {
            Self::Group => "group",
            Self::Alt => "alt",
            Self::Opt => "opt",
            Self::Loop => "loop",
            Self::Par => "par",
            Self::Break => "break",
            Self::Critical => "critical",
            Self::Literal(value) => value,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub(in crate::plantuml) enum Lifecycle {
    Activate,
    Deactivate,
    Create,
    Destroy,
}
#[derive(Clone, Debug)]
pub(in crate::plantuml) struct Name {
    pub text: String,
    pub quoted: bool,
}
pub(in crate::plantuml) struct Declaration {
    pub name: String,
    pub label: String,
}
pub(in crate::plantuml) struct Message {
    pub left: Name,
    pub right: Name,
    pub arrow: Arrow,
    pub reverse: bool,
    pub label: String,
}
pub(in crate::plantuml) struct Anchor {
    pub side: Side,
    pub name: Name,
}

use super::parser::error::ParseFailure;
use std::ops::Range;

impl<'b> Builder<'b> {
    pub fn new(budget: &'b mut Budget, retained_bytes: usize) -> Self {
        Self {
            budget,
            model: Sequence {
                participants: Vec::new(),
                boxes: Vec::new(),
                events: Vec::new(),
            },
            active: Vec::new(),
            life: Vec::new(),
            frames: Vec::new(),
            open_boxes: Vec::new(),
            note: String::new(),
            retained_bytes,
        }
    }

    pub fn tick(&mut self) -> Result<(), ParseFailure> {
        self.budget.charge(1).map_err(ParseFailure::terminal)
    }

    pub fn single(&mut self, name: Name) -> Result<Declaration, ParseFailure> {
        self.budget
            .charge(name.text.len())
            .map_err(ParseFailure::terminal)?;
        Ok(Declaration {
            name: name.text.clone(),
            label: name.text,
        })
    }

    pub fn finish(self, span: Range<usize>) -> Result<Sequence, ParseFailure> {
        self.budget
            .charge(self.active.len())
            .map_err(ParseFailure::terminal)?;
        self.budget.checkpoint().map_err(ParseFailure::terminal)?;
        if self.active.iter().any(|n| *n > 0) {
            return Err(ParseFailure::syntax(span, "unclosed group or activation"));
        }
        if self.open_boxes.last().is_some() {
            return Err(ParseFailure::syntax(
                span,
                "unclosed sequence participant box",
            ));
        }
        if self.model.participants.is_empty() {
            return Err(ParseFailure::syntax(span, "sequence has no participants"));
        }
        Ok(self.model)
    }

    fn participant(&mut self, name: &str) -> Result<usize, ParseFailure> {
        self.budget
            .charge(self.model.participants.len())
            .map_err(ParseFailure::terminal)?;
        if let Some(index) = self.model.participants.iter().position(|p| p.name == name) {
            return Ok(index);
        }
        self.budget.item().map_err(ParseFailure::terminal)?;
        self.budget
            .text(name.len().saturating_mul(2))
            .map_err(ParseFailure::terminal)?;
        let index = self.model.participants.len();
        retain(
            (&mut self.model.participants, &mut self.retained_bytes),
            Participant {
                name: name.into(),
                label: name.into(),
                kind: ParticipantKind::Participant,
                stereotype: Stereotype::None,
            },
            self.budget,
        )?;
        retain((&mut self.active, &mut self.retained_bytes), 0, self.budget)?;
        retain(
            (&mut self.life, &mut self.retained_bytes),
            Life::Alive,
            self.budget,
        )?;
        Ok(index)
    }

    fn push(&mut self, event: Event) -> Result<(), ParseFailure> {
        self.budget.item().map_err(ParseFailure::terminal)?;
        retain(
            (&mut self.model.events, &mut self.retained_bytes),
            event,
            self.budget,
        )
    }

    pub fn declare(
        &mut self,
        declaration: (ParticipantKind, Declaration, Stereotype),
        span: Range<usize>,
    ) -> Result<(), ParseFailure> {
        let (kind, Declaration { name, label }, stereotype) = declaration;
        self.budget
            .charge(self.model.participants.len())
            .map_err(ParseFailure::terminal)?;
        if let Some(id) = self.model.participants.iter().position(|p| p.name == name) {
            if self.open_boxes.last().is_some()
                && label == name
                && matches!(stereotype, Stereotype::None)
            {
                return self.add_box_member(id, span);
            }
            return Err(ParseFailure::syntax(span, "duplicate sequence participant"));
        }
        let id = self.participant(&name)?;
        self.budget
            .text(label.len() + stereotype.text_len())
            .map_err(ParseFailure::terminal)?;
        self.model.participants[id] = Participant {
            name,
            label,
            kind,
            stereotype,
        };
        self.add_box_member(id, span)
    }

    pub fn open_box(&mut self, header: BoxHeader, span: Range<usize>) -> Result<(), ParseFailure> {
        if self.open_boxes.last().is_some() {
            return Err(ParseFailure::unsupported(
                span,
                "nested sequence participant box",
            ));
        }
        self.budget.item().map_err(ParseFailure::terminal)?;
        self.budget
            .text(header.title.len())
            .map_err(ParseFailure::terminal)?;
        let index = self.model.boxes.len();
        retain(
            (&mut self.model.boxes, &mut self.retained_bytes),
            ParticipantBox {
                title: header.title,
                color: header.color,
                members: Vec::new(),
            },
            self.budget,
        )?;
        retain(
            (&mut self.open_boxes, &mut self.retained_bytes),
            index,
            self.budget,
        )
    }

    pub fn close_box(&mut self, span: Range<usize>) -> Result<(), ParseFailure> {
        let index = self
            .open_boxes
            .pop()
            .ok_or_else(|| ParseFailure::syntax(span.clone(), "end box without box"))?;
        if self.model.boxes[index].members.is_empty() {
            return Err(ParseFailure::syntax(span, "empty sequence participant box"));
        }
        Ok(())
    }

    fn add_box_member(&mut self, id: usize, span: Range<usize>) -> Result<(), ParseFailure> {
        let Some(&index) = self.open_boxes.last() else {
            return Ok(());
        };
        if self
            .model
            .boxes
            .iter()
            .any(|group| group.members.contains(&id))
        {
            return Err(ParseFailure::syntax(
                span,
                "participant already belongs to a box",
            ));
        }
        retain(
            (
                &mut self.model.boxes[index].members,
                &mut self.retained_bytes,
            ),
            id,
            self.budget,
        )
    }

    pub fn lifecycle(
        &mut self,
        command: (Lifecycle, Name),
        span: Range<usize>,
    ) -> Result<(), ParseFailure> {
        let (kind, name) = command;
        self.budget
            .charge(self.model.participants.len())
            .map_err(ParseFailure::terminal)?;
        let existed = self.model.participants.iter().any(|p| p.name == name.text);
        let id = self.participant(&name.text)?;
        if matches!(self.life[id], Life::Dead) {
            return Err(ParseFailure::syntax(span, "participant already destroyed"));
        }
        let event = match kind {
            Lifecycle::Activate => {
                self.budget
                    .depth(self.active[id] + 1)
                    .map_err(ParseFailure::terminal)?;
                self.active[id] += 1;
                Event::Activate(id)
            }
            Lifecycle::Deactivate => {
                self.active[id] = self.active[id].checked_sub(1).ok_or_else(|| {
                    ParseFailure::syntax(span.clone(), "deactivate without activate")
                })?;
                Event::Deactivate(id)
            }
            Lifecycle::Create if existed => {
                return Err(ParseFailure::syntax(
                    span,
                    "create requires a new participant",
                ));
            }
            Lifecycle::Create => Event::Create(id),
            Lifecycle::Destroy => {
                if self.active[id] > 0 {
                    return Err(ParseFailure::syntax(
                        span,
                        "destroy with unclosed activation",
                    ));
                }
                self.life[id] = Life::Dead;
                Event::Destroy(id)
            }
        };
        self.push(event)
    }

    pub fn open(&mut self, frame: (FrameKind, String)) -> Result<(), ParseFailure> {
        let (kind, label) = frame;
        self.budget
            .depth(self.frames.len() + 1)
            .map_err(ParseFailure::terminal)?;
        self.tick()?;
        self.budget
            .text(kind.name().len())
            .map_err(ParseFailure::terminal)?;
        let display = kind.name().to_owned();
        retain(
            (&mut self.frames, &mut self.retained_bytes),
            kind,
            self.budget,
        )?;
        self.push(Event::Open {
            kind: display,
            label,
        })
    }

    pub fn branch(&mut self, label: String, span: Range<usize>) -> Result<(), ParseFailure> {
        if matches!(self.frames.last(), Some(FrameKind::Alt | FrameKind::Par)) {
            return self.push(Event::Else(label));
        }
        Err(ParseFailure::syntax(span, "else outside alt/par frame"))
    }

    pub fn close(&mut self) -> Result<(), ParseFailure> {
        self.tick()?;
        self.frames.pop();
        self.push(Event::Close)
    }

    pub fn append_note(&mut self, row: String) -> Result<(), ParseFailure> {
        self.append_note_part(&row)
    }

    pub fn separate_note(&mut self) -> Result<(), ParseFailure> {
        self.append_note_part("\n")
    }

    fn append_note_part(&mut self, part: &str) -> Result<(), ParseFailure> {
        if self.note.len().saturating_add(part.len()) > self.note.capacity() {
            self.budget
                .charge(self.note.len())
                .map_err(ParseFailure::terminal)?;
        }
        super::text::append_label(&mut self.note, part, self.budget).map_err(ParseFailure::terminal)
    }

    pub fn finish_note(&mut self, anchor: Anchor) -> Result<(), ParseFailure> {
        let label = std::mem::take(&mut self.note);
        self.note(anchor, label)
    }

    pub fn note(&mut self, anchor: Anchor, label: String) -> Result<(), ParseFailure> {
        let participant = self.participant(&anchor.name.text)?;
        self.push(Event::Note {
            participant,
            side: anchor.side,
            label,
        })
    }

    pub fn message(&mut self, message: Message, span: Range<usize>) -> Result<(), ParseFailure> {
        let Message {
            left,
            right,
            arrow,
            reverse,
            label,
        } = message;
        let a = self.participant(&left.text)?;
        let b = self.participant(&right.text)?;
        if matches!(self.life[a], Life::Dead) || matches!(self.life[b], Life::Dead) {
            return Err(ParseFailure::syntax(
                span,
                "message references a destroyed participant",
            ));
        }
        let (from, to) = if reverse { (b, a) } else { (a, b) };
        self.push(Event::Message {
            from,
            to,
            arrow,
            label,
        })
    }
}

fn retain<T>(
    target: (&mut Vec<T>, &mut usize),
    value: T,
    budget: &mut Budget,
) -> Result<(), ParseFailure> {
    let (values, retained) = target;
    budget.charge(1).map_err(ParseFailure::terminal)?;
    if values.len() == values.capacity() {
        let capacity = values.capacity().saturating_mul(2).max(4);
        let bytes = capacity.saturating_mul(std::mem::size_of::<T>());
        super::budget::bound(
            "sequence retained model arena bytes",
            retained.saturating_add(bytes),
            budget.limits.source_bytes.saturating_mul(256),
        )
        .map_err(ParseFailure::terminal)?;
        budget
            .charge(values.len())
            .map_err(ParseFailure::terminal)?;
        let old = values.capacity().saturating_mul(std::mem::size_of::<T>());
        values
            .try_reserve_exact(capacity - values.len())
            .map_err(|error| {
                ParseFailure::terminal(RenderError::Backend(format!(
                    "PlantUML sequence model allocation for capacity {capacity} failed: {error}"
                )))
            })?;
        *retained = retained
            .saturating_sub(old)
            .saturating_add(values.capacity().saturating_mul(std::mem::size_of::<T>()));
        super::budget::bound(
            "sequence actual model arena bytes",
            *retained,
            budget.limits.source_bytes.saturating_mul(256),
        )
        .map_err(ParseFailure::terminal)?;
    }
    values.push(value);
    Ok(())
}

const FRAME_GAP: f64 = 8.;

struct EventLabel {
    caption: Label,
    kind: Label,
}
enum StereotypeLabel {
    None,
    Present(Label),
}
struct HeadLabels {
    name: Label,
    stereotype: StereotypeLabel,
    spot_width: f64,
}
impl HeadLabels {
    fn width(&self) -> f64 {
        let stereo = match &self.stereotype {
            StereotypeLabel::None => 0.,
            StereotypeLabel::Present(label) => label.width + self.spot_width,
        };
        self.name.width.max(stereo)
    }

    fn height(&self) -> f64 {
        let stereo = match &self.stereotype {
            StereotypeLabel::None => 0.,
            StereotypeLabel::Present(label) => label.height + 4.,
        };
        self.name.height + stereo
    }
}
fn measure_head(
    participant: &Participant,
    context: &mut Context<'_>,
) -> Result<HeadLabels, RenderError> {
    let name = context.label(&participant.label)?;
    let (stereotype, spot_width) = match &participant.stereotype {
        Stereotype::None => (StereotypeLabel::None, 0.),
        Stereotype::Label { text, spot } => (
            StereotypeLabel::Present(
                context.styled_label(&format!("«{text}»"), Decoration::Italic)?,
            ),
            if matches!(spot, Spot::Marker(_, _)) {
                22.
            } else {
                0.
            },
        ),
    };
    Ok(HeadLabels {
        name,
        stereotype,
        spot_width,
    })
}
fn head_top(model: &Sequence) -> f64 {
    if model.boxes.is_empty() { 24. } else { 56. }
}
fn draw_boxes(
    model: &Sequence,
    heads: &[HeadLabels],
    titles: &[Label],
    xs: &[f64],
    bottom: f64,
    scene: &mut Scene,
    budget: &mut Budget,
) -> Result<(), RenderError> {
    for (index, group) in model.boxes.iter().enumerate() {
        budget.charge(group.members.len())?;
        let left = group
            .members
            .iter()
            .map(|id| xs[*id] - heads[*id].width().max(60.) / 2. - 24.)
            .fold(f64::INFINITY, f64::min);
        let right = group
            .members
            .iter()
            .map(|id| xs[*id] + heads[*id].width().max(60.) / 2. + 24.)
            .fold(f64::NEG_INFINITY, f64::max);
        let width = (right - left).max(titles[index].width + 24.);
        let rect = Rect {
            x: left,
            y: 12.,
            width,
            height: bottom - 12.,
        };
        let mut element = Element::new(format!("box-{index}"), "sequence-box");
        element.name = group.title.clone();
        element
            .shapes
            .push(Shape::Rect(rect, 0., Paint::Color(group.color)));
        element.shapes.push(Shape::Text(
            Point::new(left + 10., 18.),
            titles[index].clone(),
            Decoration::Plain,
        ));
        scene.push(element, budget)?;
    }
    Ok(())
}
impl EventLabel {
    fn width(&self) -> f64 {
        self.caption.width.max(self.kind.width)
    }
    fn header_height(&self) -> f64 {
        self.kind.height + self.caption.height + FRAME_GAP * 3.
    }
}

pub(super) fn layout(model: &Sequence, context: &mut Context<'_>) -> Result<Scene, RenderError> {
    let heads = model
        .participants
        .iter()
        .map(|p| measure_head(p, context))
        .collect::<Result<Vec<_>, _>>()?;
    let titles = model
        .boxes
        .iter()
        .map(|group| context.label(&group.title))
        .collect::<Result<Vec<_>, _>>()?;
    let labels = model
        .events
        .iter()
        .map(|event| {
            let kind = match event {
                Event::Open { kind, .. } => kind.as_str(),
                _ => "",
            };
            Ok(EventLabel {
                caption: context.label(event_label(event))?,
                kind: context.label(kind)?,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let widest = heads
        .iter()
        .map(HeadLabels::width)
        .chain(titles.iter().map(|label| label.width))
        .chain(labels.iter().map(EventLabel::width))
        .fold(80., f64::max);
    let step = widest + 110.;
    context.budget.charge(model.events.len())?;
    let margin = frame_depth(model) as f64 * 8.;
    let xs: Vec<f64> = (0..heads.len())
        .map(|i| margin + step * (i + 1) as f64)
        .collect();
    let mut scene = Scene::new();
    let (rows, bottom) = allocate_rows(model, (&heads, &labels), context.budget)?;
    draw_boxes(
        model,
        &heads,
        &titles,
        &xs,
        bottom,
        &mut scene,
        context.budget,
    )?;
    draw_lanes(model, &heads, &xs, &rows, bottom, &mut scene, context)?;
    draw_events(model, &labels, &xs, &rows, &mut scene, context)?;
    // Closing an activation determines its extent, not its foreground paint order.
    let count = scene.elements.len();
    context
        .budget
        .charge(count.saturating_mul(count.ilog2() as usize + 1))?;
    scene.elements.sort_by(|left, right| {
        let a = paint_layer(left);
        let b = paint_layer(right);
        a.0.cmp(&b.0).then_with(|| a.1.total_cmp(&b.1))
    });
    Ok(scene)
}

fn paint_layer(element: &Element) -> (u8, f64) {
    match element.kind {
        "sequence-box" => (0, 0.),
        "lifeline" => (1, 0.),
        "activation" => match element.shapes.first() {
            Some(Shape::Rect(rect, _, _)) => (2, rect.y),
            _ => (2, 0.),
        },
        _ => (3, 0.),
    }
}

fn allocate_rows(
    model: &Sequence,
    labels: (&[HeadLabels], &[EventLabel]),
    budget: &mut Budget,
) -> Result<(Vec<f64>, f64), RenderError> {
    let (heads, events) = labels;
    let top = head_top(model);
    let mut bottom = model
        .participants
        .iter()
        .zip(heads)
        .map(|(p, label)| top + head_height(p.kind, label))
        .fold(top, f64::max);
    let mut rows = Vec::with_capacity(events.len());
    for (event, label) in model.events.iter().zip(events) {
        budget.charge(1)?;
        let (above, below) = event_extents(event, label, heads);
        let anchor = bottom + 24. + above;
        rows.push(anchor);
        bottom = anchor + below;
    }
    Ok((rows, bottom + 36.))
}

fn head_height(kind: ParticipantKind, label: &HeadLabels) -> f64 {
    label.height()
        + if matches!(kind, ParticipantKind::Actor) {
            65.
        } else {
            24.
        }
}

fn event_extents(event: &Event, measured: &EventLabel, heads: &[HeadLabels]) -> (f64, f64) {
    let label = &measured.caption;
    match event {
        Event::Message { from, to, .. } => (label.height + 4., if from == to { 24. } else { 8. }),
        Event::Note { .. } => (label.height / 2., label.height / 2. + 16.),
        Event::Open { .. } => (0., measured.header_height()),
        Event::Else(_) => (0., label.height + 5.),
        Event::Close => (0., 16.),
        Event::Create(id) => (0., head_height(ParticipantKind::Participant, &heads[*id])),
        Event::Destroy(_) => (7., 7.),
        _ => (0., 0.),
    }
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
    heads: &[HeadLabels],
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
            .map_or(head_top(model), |i| rows[i]);
        let death = model
            .events
            .iter()
            .position(|e| matches!(e, Event::Destroy(p) if *p == id))
            .map_or(end, |i| rows[i]);
        let height = head_height(participant.kind, &heads[id]);
        let rect = Rect {
            x: xs[id] - heads[id].width().max(60.) / 2. - 12.,
            y: birth,
            width: heads[id].width().max(60.) + 24.,
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
        let labels = &heads[id];
        match &labels.stereotype {
            StereotypeLabel::None => head.shapes.push(text_at(label_rect, labels.name.clone())),
            StereotypeLabel::Present(stereo) => {
                let content_top = label_rect.y + (label_rect.height - labels.height()) / 2.;
                let stereo_left = xs[id] - (stereo.width + labels.spot_width) / 2.;
                if let Stereotype::Label {
                    spot: Spot::Marker(mark, color),
                    ..
                } = &participant.stereotype
                {
                    let spot_rect = Rect {
                        x: stereo_left,
                        y: content_top,
                        width: 16.,
                        height: 16.,
                    };
                    head.shapes
                        .push(Shape::Ellipse(spot_rect, Paint::Color(*color)));
                    let marker = context.label(&mark.to_string())?;
                    head.shapes.push(text_at(spot_rect, marker));
                }
                head.shapes.push(Shape::Text(
                    Point::new(stereo_left + labels.spot_width, content_top),
                    stereo.clone(),
                    Decoration::Italic,
                ));
                head.shapes.push(Shape::Text(
                    Point::new(
                        xs[id] - labels.name.width / 2.,
                        content_top + stereo.height + 4.,
                    ),
                    labels.name.clone(),
                    Decoration::Plain,
                ));
            }
        }
        scene.push(head, context.budget)?;
    }
    Ok(())
}

fn draw_events(
    model: &Sequence,
    labels: &[EventLabel],
    xs: &[f64],
    rows: &[f64],
    scene: &mut Scene,
    context: &mut Context<'_>,
) -> Result<(), RenderError> {
    let mut frames: Vec<(usize, String)> = Vec::new();
    context.budget.charge(model.events.len())?;
    let maximum_depth = frame_depth(model);
    let frame_margin = labels.iter().map(EventLabel::width).fold(80., f64::max) + 60.;
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
                    Point::new(
                        xs[*from].min(xs[*to]) + 10.,
                        y - labels[index].caption.height - 4.,
                    ),
                    labels[index].caption.clone(),
                    Decoration::Plain,
                ));
            }
            Event::Note {
                participant, side, ..
            } => {
                element.kind = "note";
                element.owner = format!("p{participant}");
                let width = labels[index].caption.width + 24.;
                let x = match side {
                    Side::Left => xs[*participant] - width - 24.,
                    Side::Right => xs[*participant] + 24.,
                    Side::Over => xs[*participant] - width / 2.,
                };
                let rect = Rect {
                    x,
                    y: y - labels[index].caption.height / 2.,
                    width,
                    height: labels[index].caption.height + 16.,
                };
                element.shapes = super::shapes::node(rect, "note");
                element
                    .shapes
                    .push(text_at(rect, labels[index].caption.clone()));
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
                    labels[index].caption.clone(),
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
                let x = xs[0] - frame_margin - padding;
                let top = rows[start];
                element.shapes.push(Shape::Rect(
                    Rect {
                        x,
                        y: top,
                        width: xs[xs.len() - 1] - xs[0] + frame_margin * 2. + padding * 2.,
                        height: y - top + 16.,
                    },
                    0.,
                    Paint::None,
                ));
                element.shapes.push(Shape::Text(
                    Point::new(x + FRAME_GAP, top + FRAME_GAP),
                    labels[start].kind.clone(),
                    Decoration::Plain,
                ));
                element.shapes.push(Shape::Text(
                    Point::new(
                        x + FRAME_GAP,
                        top + FRAME_GAP * 2. + labels[start].kind.height,
                    ),
                    labels[start].caption.clone(),
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

    fn draw(source: &str) -> Scene {
        let mut budget = Budget::new(&RenderOptions::default());
        let (preamble, model) = prepare(source, &mut budget);
        let session = RenderEnvironment::deterministic().begin_session().unwrap();
        layout(
            &model,
            &mut Context::new(&session, &mut budget).with_fonts(preamble.fonts),
        )
        .unwrap()
    }

    fn prepare(source: &str, budget: &mut Budget) -> (super::super::style::Preamble, Sequence) {
        let document = super::super::preprocess::prepare(source, budget).unwrap();
        let mut prepared = super::super::parser::document::parse(&document, budget).unwrap();
        let (preamble, _) = super::super::parser::preamble::parse(&mut prepared, budget).unwrap();
        let model = super::super::parser::sequence::parse(prepared.body, budget).unwrap();
        (preamble, model)
    }

    fn occupied(element: &Element) -> (f64, f64) {
        element.shapes.iter().fold(
            (f64::INFINITY, f64::NEG_INFINITY),
            |(top, bottom), shape| {
                let (a, b) = match shape {
                    Shape::Text(point, label, _) => (point.y, point.y + label.height),
                    Shape::Rect(rect, _, _) | Shape::Ellipse(rect, _) => {
                        (rect.y, rect.y + rect.height)
                    }
                    Shape::Line(points, _)
                    | Shape::ColoredLine(points, _, _)
                    | Shape::Polygon(points, _) => (
                        points.iter().map(|p| p.y).fold(f64::INFINITY, f64::min),
                        points.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max),
                    ),
                };
                (top.min(a), bottom.max(b))
            },
        )
    }

    #[test]
    fn should_reserve_measured_nested_frame_headers_at_small_and_large_fonts() {
        // given
        for size in [18, 96] {
            let source = format!(
                "@startuml\nskinparam defaultFontSize {size}\nparticipant A\ngroup **Long wide caption**\\n//Second row//\ncritical __Nested__\nA -> A : inside\nend\nend\nA -> A : after\n@enduml"
            );
            // when
            let scene = draw(&source);
            let frames = scene
                .elements
                .iter()
                .filter(|e| e.kind == "frame")
                .collect::<Vec<_>>();
            // then
            assert_eq!(frames.len(), 2);
            for frame in &frames {
                let Shape::Rect(border, _, _) = frame.shapes[0] else {
                    panic!("frame border missing")
                };
                let mut labels = frame
                    .shapes
                    .iter()
                    .filter_map(|shape| match shape {
                        Shape::Text(point, label, _) => Some((point, label)),
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                labels.sort_by(|a, b| a.0.y.total_cmp(&b.0.y));
                assert_eq!(labels.len(), 2);
                for (point, label) in &labels {
                    assert!(
                        point.y > border.y && point.y + label.height < border.y + border.height,
                        "{size}px frame text crosses horizontal border"
                    );
                    assert!(
                        point.x > border.x && point.x + label.width < border.x + border.width,
                        "{size}px frame text crosses vertical border"
                    );
                }
                assert!(
                    labels[0].0.y + labels[0].1.height < labels[1].0.y,
                    "{size}px kind and caption overlap"
                );
            }
            let inner = occupied(frames[0]);
            let outer = occupied(frames[1]);
            assert!(outer.0 < inner.0 && outer.1 > inner.1);
            let header_bottom = |frame: &Element| {
                frame
                    .shapes
                    .iter()
                    .filter_map(|shape| match shape {
                        Shape::Text(point, label, _) => Some(point.y + label.height),
                        _ => None,
                    })
                    .fold(0., f64::max)
            };
            assert!(
                header_bottom(frames[1]) < inner.0,
                "nested frame crosses outer header"
            );
            let messages = scene
                .elements
                .iter()
                .filter(|e| e.kind == "message")
                .collect::<Vec<_>>();
            assert!(header_bottom(frames[0]) < occupied(messages[0]).0);
            assert!(outer.1 < occupied(messages[1]).0);
        }
    }

    #[test]
    fn should_paint_activations_between_lifelines_and_event_foreground() {
        // given
        let source = "@startuml\nparticipant A\nparticipant B\nactivate B\nactivate B\nnote over B : __visible__\nB -> A : **reply**\ndeactivate B\ndeactivate B\n@enduml";
        // when
        let scene = draw(source);
        let positions = |kind| {
            scene
                .elements
                .iter()
                .enumerate()
                .filter_map(move |(i, e)| (e.kind == kind).then_some(i))
                .collect::<Vec<_>>()
        };
        let activations = positions("activation");
        // then
        assert!(
            positions("lifeline")
                .into_iter()
                .all(|i| i < activations[0])
        );
        assert!(
            positions("note")
                .into_iter()
                .chain(positions("message"))
                .all(|i| i > activations[1])
        );
        let outer = occupied(&scene.elements[activations[0]]);
        let inner = occupied(&scene.elements[activations[1]]);
        assert!(outer.0 < inner.0 && outer.1 > inner.1);
    }

    #[test]
    fn should_reserve_multiline_message_occupied_height_before_its_anchor() {
        // given
        let source = "@startuml\nA -> B : first\nA -> B : line1\\nline2\\nline3\\nline4\\nline5\\nline6\nB -> B : self\\ncall\n@enduml";
        // when
        let scene = draw(source);
        let messages = scene
            .elements
            .iter()
            .filter(|e| e.kind == "message")
            .collect::<Vec<_>>();
        // then
        for pair in messages.windows(2) {
            assert!(
                occupied(pair[0]).1 < occupied(pair[1]).0,
                "later message overlaps earlier event"
            );
        }
    }

    #[test]
    fn should_reserve_multiline_note_occupied_height_after_a_message() {
        // given
        let source = "@startuml\nA -> B : first\nnote over B\nline1\nline2\nline3\nline4\nline5\nline6\nend note\nB -> A : done\n@enduml";
        // when
        let scene = draw(source);
        let events = scene
            .elements
            .iter()
            .filter(|e| matches!(e.kind, "message" | "note"))
            .collect::<Vec<_>>();
        // then
        for pair in events.windows(2) {
            assert!(
                occupied(pair[0]).1 < occupied(pair[1]).0,
                "multiline note overlaps adjacent event"
            );
        }
    }

    #[test]
    fn should_reserve_created_multiline_head_before_the_next_message() {
        // given
        let source = "@startuml\nparticipant A\ncreate \"B\\nline2\\nline3\\nline4\"\nA -> \"B\\nline2\\nline3\\nline4\" : created\ndestroy \"B\\nline2\\nline3\\nline4\"\n@enduml";
        // when
        let scene = draw(source);
        let head = scene.elements.iter().find(|e| e.id == "p1").unwrap();
        let message = scene.elements.iter().find(|e| e.kind == "message").unwrap();
        // then
        assert!(
            occupied(head).1 < occupied(message).0,
            "message crosses created participant head"
        );
    }

    #[test]
    fn should_preserve_lane_order_reverse_message_and_balanced_events() {
        // given
        let source = "@startuml\nparticipant Alpha as A\nA <- B : Response\nactivate A\nA -> A : Self\ndeactivate A\n@enduml";
        let mut budget = Budget::new(&RenderOptions::default());
        // when
        let (_, model) = prepare(source, &mut budget);
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
        let (_, model) = prepare(source, &mut budget);
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
