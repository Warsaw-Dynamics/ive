//! Timing grammar/cursors translated from the authorized MIT timingdiagram sources.
//! Copyright 2009-2024 Arnaud Roques; exact traces in the provenance manifest.
use super::{
    budget::{Budget, bound, invalid, unsupported},
    source::Line,
};
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
struct Parser<'a> {
    tracks: Vec<TrackBuilder>,
    registry: BTreeMap<String, usize>,
    marks: BTreeSet<i64>,
    constraints: Vec<Constraint>,
    messages: Vec<Message>,
    player: Cursor<usize>,
    time: Cursor<i64>,
    budget: &'a mut Budget,
}

pub(super) fn parse(lines: &[Line], budget: &mut Budget) -> Result<Timing, RenderError> {
    budget.depth(1)?;
    let mut parser = Parser {
        tracks: Vec::new(),
        registry: BTreeMap::new(),
        marks: BTreeSet::new(),
        constraints: Vec::new(),
        messages: Vec::new(),
        player: Cursor::Unset,
        time: Cursor::Unset,
        budget,
    };
    for line in lines {
        parser
            .budget
            .charge(line.text.len().saturating_mul(8) + 1)?;
        parser.statement(line)?;
    }
    parser.finish()
}

impl Parser<'_> {
    fn statement(&mut self, line: &Line) -> Result<(), RenderError> {
        let text = line.text.as_str();
        if let Some(rest) = word_suffix(text, "robust") {
            return self.declare(rest, TrackKind::Robust, line);
        }
        if let Some(rest) = word_suffix(text, "concise") {
            return self.declare(rest, TrackKind::Concise, line);
        }
        if let Some((left, rest)) = text.split_once(char::is_whitespace)
            && let Some(right) = word_suffix(rest.trim_start(), "is")
        {
            return self.change(left, right, line);
        }
        let (head, _) = split_caption(text);
        if let Some(index) = head.find("<->") {
            return self.constraint(&text[..index], &text[index + 3..], line);
        }
        if let Some(index) = head.find("->") {
            return self.message(&text[..index], &text[index + 2..], line);
        }
        if let Some(rest) = text.strip_prefix('@') {
            if is_code(rest) {
                self.player = Cursor::Selected(self.player_id(rest, line)?);
                return Ok(());
            }
            let time = parse_time(rest, self.time.value(), line)?;
            return self.select_time(time);
        }
        Err(unsupported(
            line.number,
            &format!("timing statement {}", line.text),
        ))
    }
    fn declare(&mut self, rest: &str, kind: TrackKind, line: &Line) -> Result<(), RenderError> {
        let (display, code) = if rest.starts_with('"') {
            let (display, suffix) = quoted(rest, line)?;
            (
                display,
                word_suffix(suffix.trim_start(), "as").unwrap_or(""),
            )
        } else {
            (rest, rest)
        };
        if display.is_empty() || !is_code(code) {
            return Err(unsupported(
                line.number,
                &format!("timing track declaration {}", line.text),
            ));
        }
        self.budget
            .charge(search_cost(self.registry.len()).saturating_mul(code.len() + 1))?;
        if self.registry.contains_key(code) {
            return Err(unsupported(
                line.number,
                &format!("duplicate timing track {}", line.text),
            ));
        }
        self.budget.item()?;
        self.budget
            .text(code.len().saturating_mul(2) + display.len())?;
        bound("label bytes", display.len(), 16 * 1024)?;
        let id = self.tracks.len();
        self.registry.insert(code.into(), id);
        self.tracks.push(TrackBuilder {
            track: Track {
                code: code.into(),
                display: display.into(),
                kind,
                states: Vec::new(),
                initial: Initial::Absent,
                changes: Vec::new(),
            },
            states: BTreeMap::new(),
            changes: BTreeMap::new(),
        });
        self.player = Cursor::Selected(id);
        Ok(())
    }
    fn player_id(&mut self, code: &str, line: &Line) -> Result<usize, RenderError> {
        self.budget
            .charge(search_cost(self.registry.len()).saturating_mul(code.len() + 1))?;
        self.registry.get(code).copied().ok_or_else(|| {
            invalid(
                line.number,
                format!("unknown timing track {code:?} in {:?}", line.text),
            )
        })
    }
    fn current_player(&self, line: &Line) -> Result<usize, RenderError> {
        self.player.value().ok_or_else(|| {
            invalid(
                line.number,
                format!("no selected timing track for {:?}", line.text),
            )
        })
    }
    fn select_time(&mut self, time: i64) -> Result<(), RenderError> {
        self.budget.charge(search_cost(self.marks.len()))?;
        if !self.marks.contains(&time) {
            self.budget.item()?;
            self.marks.insert(time);
        }
        self.time = Cursor::Selected(time);
        Ok(())
    }
    fn change(&mut self, left: &str, right: &str, line: &Line) -> Result<(), RenderError> {
        let state = state_atom(right, line)?;
        let track = if is_code(left) {
            self.player_id(left, line)?
        } else {
            let track = self.current_player(line)?;
            let time = parse_time(left, self.time.value(), line)?;
            self.select_time(time)?;
            track
        };
        self.budget
            .charge(search_cost(self.tracks[track].changes.len()))?;
        // Retain losing labels for the shared strict markup/resource validation.
        let id = self.intern_state(track, state)?;
        if let Some(time) = self.time.value()
            && self.tracks[track].changes.contains_key(&time)
        {
            return Ok(());
        }
        if let Some(time) = self.time.value() {
            self.budget.item()?;
            self.tracks[track].changes.insert(time, id);
        } else {
            self.tracks[track].track.initial = Initial::State(id);
        }
        Ok(())
    }
    fn intern_state(&mut self, track: usize, state: &str) -> Result<usize, RenderError> {
        self.budget
            .charge(search_cost(self.tracks[track].states.len()).saturating_mul(state.len() + 1))?;
        if let Some(id) = self.tracks[track].states.get(state) {
            return Ok(*id);
        }
        self.budget.item()?;
        self.budget.text(state.len().saturating_mul(2))?;
        let id = self.tracks[track].track.states.len();
        self.tracks[track].states.insert(state.into(), id);
        self.tracks[track].track.states.push(state.into());
        Ok(id)
    }
    fn constraint(&mut self, left: &str, right: &str, line: &Line) -> Result<(), RenderError> {
        let (code, start) = left.trim().split_once('@').ok_or_else(|| {
            invalid(
                line.number,
                format!("timing constraint requires @time: {:?}", line.text),
            )
        })?;
        let track = if code.is_empty() {
            self.current_player(line)?
        } else {
            self.player_id(code, line)?
        };
        let start = parse_time(start, self.time.value(), line)?;
        let (end, label) = split_caption(right);
        let end = end.strip_prefix('@').ok_or_else(|| {
            invalid(
                line.number,
                format!("timing constraint requires @time: {:?}", line.text),
            )
        })?;
        let end = parse_time(end, Some(start), line)?;
        if start >= end {
            return Err(invalid(
                line.number,
                format!(
                    "timing constraint requires increasing endpoints: {:?}",
                    line.text
                ),
            ));
        }
        self.budget.item()?;
        self.budget.text(label.len())?;
        self.constraints.push(Constraint {
            track,
            start,
            end,
            label: label.into(),
            line: line.number,
        });
        Ok(())
    }
    fn endpoint(&mut self, raw: &str, line: &Line) -> Result<Endpoint, RenderError> {
        let (code, time) = raw
            .trim()
            .split_once('@')
            .map_or((raw.trim(), None), |(code, time)| (code, Some(time)));
        let track = self.player_id(code, line)?;
        let time = match time {
            Some(time) => parse_time(time, self.time.value(), line)?,
            None => self.time.value().ok_or_else(|| {
                invalid(
                    line.number,
                    format!("timing message has no selected time: {:?}", line.text),
                )
            })?,
        };
        Ok(Endpoint { track, time })
    }
    fn message(&mut self, left: &str, right: &str, line: &Line) -> Result<(), RenderError> {
        let (right, label) = split_caption(right);
        let from = self.endpoint(left, line)?;
        let to = self.endpoint(right, line)?;
        if from.track == to.track {
            return Err(unsupported(
                line.number,
                &format!("same-track timing message {}", line.text),
            ));
        }
        self.budget.item()?;
        self.budget.text(label.len())?;
        self.messages.push(Message {
            from,
            to,
            label: label.into(),
            line: line.number,
        });
        Ok(())
    }
    fn finish(self) -> Result<Timing, RenderError> {
        if self.tracks.is_empty() || self.marks.is_empty() {
            return Err(invalid(
                1,
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
                return Err(invalid(
                    1,
                    format!("timing track {:?} has no state", track.code),
                ));
            }
            tracks.push(track);
        }
        self.budget.charge(self.marks.len())?;
        let model = Timing {
            tracks,
            marks: self.marks.into_iter().collect(),
            constraints: self.constraints,
            messages: self.messages,
        };
        model.validate(self.budget)?;
        Ok(model)
    }
}

impl Timing {
    fn validate(&self, budget: &mut Budget) -> Result<(), RenderError> {
        let min = self.marks[0];
        let max = *self.marks.last().unwrap();
        for constraint in &self.constraints {
            budget.charge(1)?;
            check_range(constraint.start, (min, max), constraint.line)?;
            check_range(constraint.end, (min, max), constraint.line)?;
        }
        for message in &self.messages {
            for endpoint in [message.from, message.to] {
                budget.charge(search_cost(self.tracks[endpoint.track].changes.len()))?;
                check_range(endpoint.time, (min, max), message.line)?;
                if self.tracks[endpoint.track]
                    .state_at(endpoint.time)
                    .is_none()
                {
                    return Err(invalid(
                        message.line,
                        format!(
                            "timing message endpoint {}@{} has no signal",
                            self.tracks[endpoint.track].code, endpoint.time
                        ),
                    ));
                }
            }
        }
        Ok(())
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
fn check_range(time: i64, range: (i64, i64), line: usize) -> Result<(), RenderError> {
    if (range.0..=range.1).contains(&time) {
        return Ok(());
    }
    Err(invalid(
        line,
        format!(
            "timing endpoint {time} outside explicit axis {}..{}",
            range.0, range.1
        ),
    ))
}
pub(super) fn search_cost(len: usize) -> usize {
    len.checked_ilog2().unwrap_or(0) as usize + 1
}
fn is_code(code: &str) -> bool {
    let mut chars = code.chars();
    chars
        .next()
        .is_some_and(|ch| ch.is_alphabetic() || ch == '_')
        && chars.all(|ch| ch.is_alphanumeric() || matches!(ch, '_' | '.'))
}
fn parse_time(raw: &str, base: Option<i64>, line: &Line) -> Result<i64, RenderError> {
    let (relative, number) = raw
        .strip_prefix('+')
        .map_or((false, raw), |number| (true, number));
    let digits = number.strip_prefix('-').unwrap_or(number);
    if raw.len() > 12 || digits.is_empty() || !digits.bytes().all(|ch| ch.is_ascii_digit()) {
        return Err(unsupported(
            line.number,
            &format!(
                "timing requires exact integer times in -{TIME_LIMIT}..{TIME_LIMIT}: {}",
                line.text
            ),
        ));
    }
    let value = number
        .parse::<i64>()
        .map_err(|_| invalid(line.number, format!("invalid timing integer {raw:?}")))?;
    if !(-TIME_LIMIT..=TIME_LIMIT).contains(&value) {
        return Err(invalid(
            line.number,
            format!(
                "timing integer {raw:?} exceeds -{TIME_LIMIT}..{TIME_LIMIT} in {:?}",
                line.text
            ),
        ));
    }
    let value = if relative {
        base.unwrap_or(0).checked_add(value)
    } else {
        Some(value)
    };
    match value {
        Some(value) if (-TIME_LIMIT..=TIME_LIMIT).contains(&value) => Ok(value),
        _ => Err(invalid(
            line.number,
            format!(
                "timing integer {raw:?} exceeds -{TIME_LIMIT}..{TIME_LIMIT} in {:?}",
                line.text
            ),
        )),
    }
}
fn quoted<'a>(raw: &'a str, line: &Line) -> Result<(&'a str, &'a str), RenderError> {
    let bytes = raw.as_bytes();
    let mut index = 1;
    while index < bytes.len() {
        if bytes[index] == b'\\' {
            index += 2;
            continue;
        }
        if bytes[index] == b'"' {
            return Ok((&raw[1..index], &raw[index + 1..]));
        }
        index += 1;
    }
    Err(invalid(
        line.number,
        format!("unterminated timing string in {:?}", line.text),
    ))
}
fn state_atom<'a>(raw: &'a str, line: &Line) -> Result<&'a str, RenderError> {
    bound("label bytes", raw.len(), 16 * 1024)?;
    let state = if raw.starts_with('"') {
        let (value, suffix) = quoted(raw, line)?;
        if !suffix.trim().is_empty() {
            return Err(unsupported(
                line.number,
                &format!("timing state suffix {}", line.text),
            ));
        }
        value
    } else {
        let mut chars = raw.chars();
        if !chars
            .next()
            .is_some_and(|ch| ch.is_alphanumeric() || matches!(ch, '_' | '-'))
            || !chars.all(|ch| ch.is_alphanumeric() || matches!(ch, '_' | '-' | '.'))
        {
            return Err(unsupported(
                line.number,
                &format!("timing state {}", line.text),
            ));
        }
        raw
    };
    if state.is_empty() || state.starts_with('{') {
        return Err(unsupported(
            line.number,
            &format!("timing special/empty state {}", line.text),
        ));
    }
    Ok(state)
}
fn split_caption(raw: &str) -> (&str, &str) {
    raw.split_once(':')
        .map_or((raw.trim(), ""), |(head, label)| {
            (head.trim(), label.trim())
        })
}

fn word_suffix<'a>(text: &'a str, word: &str) -> Option<&'a str> {
    text.strip_prefix(word)
        .filter(|suffix| suffix.starts_with(char::is_whitespace))
        .map(str::trim)
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
