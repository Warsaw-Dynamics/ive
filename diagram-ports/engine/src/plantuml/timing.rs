//! Preserved timing model and typed cursor/reference semantics.
use super::{
    budget::Budget,
    parser::{
        context,
        error::ParseFailure,
        timing::{Command, Site, Target, Time, TimeChoice, fail},
    },
};
#[cfg(test)]
use crate::RenderError;
use std::collections::{BTreeMap, BTreeSet};
pub(super) const TIME_LIMIT: i64 = 1_000_000_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TrackKind {
    Robust,
    Concise,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Initial {
    Absent,
    State(usize),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Change {
    pub time: i64,
    pub state: usize,
}
pub(super) struct Track {
    pub code: String,
    pub display: String,
    pub kind: TrackKind,
    pub states: Vec<String>,
    pub initial: Initial,
    pub changes: Vec<Change>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Endpoint {
    pub track: usize,
    pub time: i64,
}
pub(super) struct Constraint {
    pub track: usize,
    pub start: i64,
    pub end: i64,
    pub label: String,
    pub line: usize,
}
pub(super) struct Message {
    pub from: Endpoint,
    pub to: Endpoint,
    pub label: String,
    pub line: usize,
}
pub(super) struct Timing {
    pub tracks: Vec<Track>,
    pub marks: Vec<i64>,
    pub constraints: Vec<Constraint>,
    pub messages: Vec<Message>,
}

#[derive(Clone, Copy)]
enum Cursor<T> {
    Unset,
    Selected(T),
}
impl<T: Copy> Cursor<T> {
    fn value(self) -> Option<T> {
        match self {
            Self::Unset => None,
            Self::Selected(value) => Some(value),
        }
    }
}
struct TrackBuilder {
    track: Track,
    states: BTreeMap<String, usize>,
    changes: BTreeMap<i64, usize>,
}
pub(super) struct Builder<'b> {
    pub budget: &'b mut Budget,
    tracks: Vec<TrackBuilder>,
    registry: BTreeMap<String, usize>,
    marks: BTreeSet<i64>,
    constraints: Vec<Constraint>,
    messages: Vec<Message>,
    constraint_sites: Vec<Site>,
    message_sites: Vec<Site>,
    player: Cursor<usize>,
    time: Cursor<i64>,
}
impl<'b> Builder<'b> {
    pub fn retained_bytes(items: usize) -> usize {
        let vectors = std::mem::size_of::<TrackBuilder>()
            + std::mem::size_of::<Track>()
            + std::mem::size_of::<Change>()
            + std::mem::size_of::<Constraint>()
            + std::mem::size_of::<Message>()
            + 2 * std::mem::size_of::<Site>()
            + std::mem::size_of::<String>();
        let tree_node = 12
            * (std::mem::size_of::<(String, usize)>()
                + std::mem::size_of::<(i64, usize)>()
                + std::mem::size_of::<i64>()
                + 3 * std::mem::size_of::<usize>());
        items.saturating_mul(3 * vectors + tree_node)
    }
    pub fn tick(&mut self) -> Result<(), ParseFailure> {
        self.budget.charge(1).map_err(Into::into)
    }
    pub fn new(budget: &'b mut Budget) -> Self {
        Self {
            budget,
            tracks: Vec::new(),
            registry: BTreeMap::new(),
            marks: BTreeSet::new(),
            constraints: Vec::new(),
            messages: Vec::new(),
            constraint_sites: Vec::new(),
            message_sites: Vec::new(),
            player: Cursor::Unset,
            time: Cursor::Unset,
        }
    }
    pub fn apply(&mut self, command: Command, site: Site) -> Result<(), ParseFailure> {
        self.budget.charge(1)?;
        match command {
            Command::Declare(kind, code, display) => self.declare((code, display), kind, &site),
            Command::Player(code) => {
                self.player = Cursor::Selected(self.player_id(&code, &site)?);
                Ok(())
            }
            Command::Mark(time) => {
                let value = time.resolve(self.time.value(), &site)?;
                self.select_time(value)
            }
            Command::Change(target, state) => self.change(target, state, &site),
            Command::Constraint(code, start, end, label) => {
                self.constraint((code, start, end, label), site)
            }
            Command::Message(from, to, label) => self.message((from, to, label), site),
        }
    }
    fn declare(
        &mut self,
        names: (String, String),
        kind: TrackKind,
        site: &Site,
    ) -> Result<(), ParseFailure> {
        let (code, display) = names;
        self.budget
            .charge(search_cost(self.registry.len()).saturating_mul(code.len() + 1))?;
        if self.registry.contains_key(&code) {
            return Err(ParseFailure::unsupported(
                site.span.clone(),
                "duplicate timing track",
            ));
        }
        self.budget.item()?;
        self.budget
            .text(code.len().saturating_mul(2) + display.len())?;
        let id = self.tracks.len();
        self.registry.insert(code.clone(), id);
        context::push(
            &mut self.tracks,
            TrackBuilder {
                track: Track {
                    code,
                    display,
                    kind,
                    states: Vec::new(),
                    initial: Initial::Absent,
                    changes: Vec::new(),
                },
                states: BTreeMap::new(),
                changes: BTreeMap::new(),
            },
            self.budget,
        )?;
        self.player = Cursor::Selected(id);
        Ok(())
    }
    fn player_id(&mut self, code: &str, site: &Site) -> Result<usize, ParseFailure> {
        self.budget
            .charge(search_cost(self.registry.len()).saturating_mul(code.len() + 1))?;
        self.registry
            .get(code)
            .copied()
            .ok_or_else(|| fail(site, "unknown timing track"))
    }
    fn current_player(&self, site: &Site) -> Result<usize, ParseFailure> {
        self.player
            .value()
            .ok_or_else(|| fail(site, "no selected timing track"))
    }
    fn select_time(&mut self, time: i64) -> Result<(), ParseFailure> {
        self.budget.charge(search_cost(self.marks.len()))?;
        if !self.marks.contains(&time) {
            self.budget.item()?;
            self.marks.insert(time);
        }
        self.time = Cursor::Selected(time);
        Ok(())
    }
    fn change(&mut self, target: Target, state: String, site: &Site) -> Result<(), ParseFailure> {
        let track = match target {
            Target::Player(code) => self.player_id(&code, site)?,
            Target::Time(time) => {
                let track = self.current_player(site)?;
                let time = time.resolve(self.time.value(), site)?;
                self.select_time(time)?;
                track
            }
        };
        self.budget
            .charge(search_cost(self.tracks[track].changes.len()))?;
        let id = self.intern_state(track, state)?;
        if let Some(time) = self.time.value() {
            if self.tracks[track].changes.contains_key(&time) {
                return Ok(());
            }
            self.budget.item()?;
            self.tracks[track].changes.insert(time, id);
        } else {
            self.tracks[track].track.initial = Initial::State(id);
        }
        Ok(())
    }
    fn intern_state(&mut self, track: usize, state: String) -> Result<usize, ParseFailure> {
        self.budget
            .charge(search_cost(self.tracks[track].states.len()).saturating_mul(state.len() + 1))?;
        if let Some(id) = self.tracks[track].states.get(&state) {
            return Ok(*id);
        }
        self.budget.item()?;
        self.budget.text(state.len().saturating_mul(2))?;
        let id = self.tracks[track].track.states.len();
        self.tracks[track].states.insert(state.clone(), id);
        context::push(&mut self.tracks[track].track.states, state, self.budget)?;
        Ok(id)
    }
    fn constraint(
        &mut self,
        value: (String, Time, Time, String),
        site: Site,
    ) -> Result<(), ParseFailure> {
        let (code, start, end, label) = value;
        let track = if code.is_empty() {
            self.current_player(&site)?
        } else {
            self.player_id(&code, &site)?
        };
        let start = start.resolve(self.time.value(), &site)?;
        let end = end.resolve(Some(start), &site)?;
        if start >= end {
            return Err(fail(
                &site,
                "timing constraint requires increasing endpoints",
            ));
        }
        self.budget.item()?;
        self.budget.text(label.len())?;
        context::push(
            &mut self.constraints,
            Constraint {
                track,
                start,
                end,
                label,
                line: site.line,
            },
            self.budget,
        )?;
        context::push(&mut self.constraint_sites, site, self.budget)
    }
    fn endpoint(
        &mut self,
        value: (String, TimeChoice),
        site: &Site,
    ) -> Result<Endpoint, ParseFailure> {
        let track = self.player_id(&value.0, site)?;
        let time = match value.1 {
            TimeChoice::Current => self
                .time
                .value()
                .ok_or_else(|| fail(site, "timing message has no selected time"))?,
            TimeChoice::Explicit(time) => time.resolve(self.time.value(), site)?,
        };
        Ok(Endpoint { track, time })
    }
    fn message(
        &mut self,
        value: ((String, TimeChoice), (String, TimeChoice), String),
        site: Site,
    ) -> Result<(), ParseFailure> {
        let (from, to, label) = value;
        let from = self.endpoint(from, &site)?;
        let to = self.endpoint(to, &site)?;
        if from.track == to.track {
            return Err(ParseFailure::unsupported(
                site.span.clone(),
                "same-track timing message",
            ));
        }
        self.budget.item()?;
        self.budget.text(label.len())?;
        context::push(
            &mut self.messages,
            Message {
                from,
                to,
                label,
                line: site.line,
            },
            self.budget,
        )?;
        context::push(&mut self.message_sites, site, self.budget)
    }
    pub fn finish(self, site: Site) -> Result<Timing, ParseFailure> {
        if self.tracks.is_empty() || self.marks.is_empty() {
            return Err(fail(
                &site,
                "timing requires declared tracks and an explicit numeric time mark",
            ));
        }
        let mut tracks = Vec::new();
        for builder in self.tracks {
            self.budget.charge(builder.changes.len() + 1)?;
            let mut track = builder.track;
            track.changes = builder
                .changes
                .into_iter()
                .map(|(time, state)| Change { time, state })
                .collect();
            if track.initial == Initial::Absent && track.changes.is_empty() {
                return Err(fail(&site, "timing track has no state"));
            }
            context::push(&mut tracks, track, self.budget)?;
        }
        self.budget.charge(self.marks.len())?;
        let marks: Vec<_> = self.marks.into_iter().collect();
        for (constraint, site) in self.constraints.iter().zip(&self.constraint_sites) {
            let site = Site {
                span: site.span.clone(),
                line: constraint.line,
            };
            self.budget.charge(1)?;
            if constraint.start < marks[0] || constraint.end > *marks.last().unwrap() {
                return Err(fail(&site, "timing endpoint outside explicit axis"));
            }
        }
        for (message, site) in self.messages.iter().zip(&self.message_sites) {
            let site = Site {
                span: site.span.clone(),
                line: message.line,
            };
            for endpoint in [message.from, message.to] {
                self.budget
                    .charge(search_cost(tracks[endpoint.track].changes.len()))?;
                if endpoint.time < marks[0]
                    || endpoint.time > *marks.last().unwrap()
                    || tracks[endpoint.track].state_at(endpoint.time).is_none()
                {
                    return Err(fail(
                        &site,
                        "timing message endpoint outside explicit axis or has no signal",
                    ));
                }
            }
        }
        Ok(Timing {
            tracks,
            marks,
            constraints: self.constraints,
            messages: self.messages,
        })
    }
}
impl Track {
    /// At a transition, messages attach to the new (right-hand) state.
    pub fn state_at(&self, time: i64) -> Option<usize> {
        let index = self.changes.partition_point(|change| change.time <= time);
        if index > 0 {
            return Some(self.changes[index - 1].state);
        }
        match self.initial {
            Initial::Absent => None,
            Initial::State(state) => Some(state),
        }
    }
}

pub(super) fn search_cost(len: usize) -> usize {
    len.checked_ilog2().unwrap_or(0) as usize + 1
}
#[cfg(test)]
pub(super) fn parse(
    lines: &[super::source::Line],
    budget: &mut Budget,
) -> Result<Timing, RenderError> {
    super::parser::timing::parse_test(lines, budget)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::RenderOptions;

    fn model(body: &str) -> Timing {
        let mut budget = Budget::new(&RenderOptions::default());
        let lines =
            super::super::source::prepare(&format!("@startuml\n{body}\n@enduml"), &mut budget)
                .unwrap();
        parse(&lines, &mut budget).unwrap()
    }

    #[test]
    fn should_retain_typed_track_identity_initial_state_and_exact_sorted_changes() {
        // given
        let input = "robust \"Valve\" as V\nV is Discarded\nV is Initial\nconcise C\n@+10\nV is First\nC is Waiting\n@-5\nV is Earlier\n@10\nV is Ignored\n@C\n+-2 is Running\n@20";
        // when
        let model = model(input);
        let valve = &model.tracks[0];
        let control = &model.tracks[1];
        // then
        assert_eq!(valve.code, "V");
        assert_eq!(valve.display, "Valve");
        assert_eq!(valve.kind, TrackKind::Robust);
        assert_eq!(control.kind, TrackKind::Concise);
        let Initial::State(initial) = valve.initial else {
            panic!("lost initial state")
        };
        assert_eq!(valve.states[initial], "Initial");
        assert_eq!(
            valve
                .changes
                .iter()
                .map(|change| (change.time, valve.states[change.state].as_str()))
                .collect::<Vec<_>>(),
            [(-5, "Earlier"), (10, "First")]
        );
        assert_eq!(
            control
                .changes
                .iter()
                .map(|change| change.time)
                .collect::<Vec<_>>(),
            [8, 10]
        );
        assert_eq!(model.marks, [-5, 8, 10, 20]);
        assert_eq!(valve.state_at(-6), Some(initial));
        assert_eq!(valve.state_at(10), Some(valve.changes[1].state));
    }

    #[test]
    fn should_own_constraint_and_message_endpoints_without_mutating_cursors() {
        // given
        let input = "robust V\nconcise C\n@0\nV is A\nC is B\n@15\n@V\n@6 <-> @+9 : interval\nV@6 -> C@+9 : message\n+1 is End\n@30";
        // when
        let model = model(input);
        // then
        let constraint = &model.constraints[0];
        assert_eq!(
            (constraint.track, constraint.start, constraint.end),
            (0, 6, 15)
        );
        let message = &model.messages[0];
        assert_eq!(message.from, Endpoint { track: 0, time: 6 });
        assert_eq!(message.to, Endpoint { track: 1, time: 24 });
        assert_eq!(model.tracks[0].changes[1].time, 16);
        assert_eq!(model.marks, [0, 15, 16, 30]);
    }
}
