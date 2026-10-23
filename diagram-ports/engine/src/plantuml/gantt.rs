//! Day-only Gantt grammar and boundary semantics from PlantUML MIT 1.2026.9.
//! Copyright 2009-2024 Arnaud Roques; exact source traces/notices in provenance.
use super::{
    budget::{Budget, bound, invalid, unsupported},
    source::Line,
};
use crate::RenderError;
use std::collections::BTreeMap;

pub(super) const MAX_SPAN: i32 = 7300;
const MAX_DAY: i32 = 3_652_058; // 9999-12-31, with 0001-01-01 at zero.

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct CivilDay(pub i32);
impl CivilDay {
    fn parse(text: &str, line: &Line) -> Result<Self, RenderError> {
        let bytes = text.as_bytes();
        if bytes.len() != 10
            || bytes[4] != b'-'
            || bytes[7] != b'-'
            || bytes
                .iter()
                .enumerate()
                .any(|(i, c)| i != 4 && i != 7 && !c.is_ascii_digit())
        {
            return Err(invalid(
                line.number,
                format!("expected ISO civil date YYYY-MM-DD in {:?}", line.text),
            ));
        }
        let year = text[..4].parse::<i32>().unwrap();
        let month = text[5..7].parse::<i32>().unwrap();
        let day = text[8..].parse::<i32>().unwrap();
        if year == 0 || !(1..=12).contains(&month) || !(1..=month_days(year, month)).contains(&day)
        {
            return Err(invalid(
                line.number,
                format!("invalid civil date {text:?} in {:?}", line.text),
            ));
        }
        let prior = year - 1;
        let mut ordinal = 365 * prior + prior / 4 - prior / 100 + prior / 400 + day - 1;
        for m in 1..month {
            ordinal += month_days(year, m);
        }
        Ok(Self(ordinal))
    }
    pub fn shift(self, delta: i32, line: usize) -> Result<Self, RenderError> {
        let result = self
            .0
            .checked_add(delta)
            .filter(|v| (0..=MAX_DAY).contains(v))
            .ok_or_else(|| {
                invalid(
                    line,
                    format!(
                        "civil date ordinal {} plus {delta} exceeds years 0001..9999",
                        self.0
                    ),
                )
            })?;
        Ok(Self(result))
    }
    pub fn parts(self) -> (i32, i32, i32) {
        // A bounded binary search avoids walking years or schedule days.
        let (mut low, mut high) = (1, 10000);
        while low + 1 < high {
            let mid = (low + high) / 2;
            let prior = mid - 1;
            let ordinal = 365 * prior + prior / 4 - prior / 100 + prior / 400;
            if ordinal <= self.0 {
                low = mid;
            } else {
                high = mid;
            }
        }
        let prior = low - 1;
        let mut day = self.0 - (365 * prior + prior / 4 - prior / 100 + prior / 400);
        let mut month = 1;
        while day >= month_days(low, month) {
            day -= month_days(low, month);
            month += 1;
        }
        (low, month, day + 1)
    }
    pub fn iso(self) -> String {
        let (y, m, d) = self.parts();
        format!("{y:04}-{m:02}-{d:02}")
    }
}
fn month_days(year: i32, month: i32) -> i32 {
    match month {
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Kind {
    Bar,
    Milestone,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Endpoint {
    Start,
    End,
}
impl Endpoint {
    pub fn name(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::End => "end",
        }
    }
}
pub(super) struct Task {
    pub code: String,
    pub display: String,
    pub start: CivilDay,
    pub end: CivilDay,
    pub kind: Kind,
}
pub(super) struct Dependency {
    pub from: usize,
    pub to: usize,
    pub endpoint: Endpoint,
    pub offset: i32,
    pub kind: Kind,
}
pub(super) struct Gantt {
    pub project_start: CivilDay,
    pub tasks: Vec<Task>,
    pub dependencies: Vec<Dependency>,
}
#[derive(Clone, Copy)]
enum Spec {
    Default,
    Explicit,
}
#[derive(Clone, Copy)]
enum Parent {
    Root,
    Task(usize),
}
struct Builder {
    task: Task,
    duration: Spec,
    start: Spec,
    frozen: Spec,
    parent: Parent,
}
struct Parser<'a> {
    project: CivilDay,
    tasks: Vec<Builder>,
    registry: BTreeMap<String, usize>,
    dependencies: Vec<Dependency>,
    budget: &'a mut Budget,
}

pub(super) fn parse(lines: &[Line], budget: &mut Budget) -> Result<Gantt, RenderError> {
    budget.depth(1)?;
    let first = lines
        .first()
        .ok_or_else(|| invalid(1, "Gantt requires Project starts YYYY-MM-DD"))?;
    budget.charge(first.text.len().saturating_mul(8) + 1)?;
    let project = parse_project(first)?;
    budget.item()?;
    let mut parser = Parser {
        project,
        tasks: Vec::new(),
        registry: BTreeMap::new(),
        dependencies: Vec::new(),
        budget,
    };
    for line in &lines[1..] {
        parser
            .budget
            .charge(line.text.len().saturating_mul(8) + 1)?;
        parser.statement(line)?;
    }
    parser.finish()
}
fn parse_project(line: &Line) -> Result<CivilDay, RenderError> {
    let mut words = line.text.split_whitespace();
    let (subject, verb, date) = (
        words.next().unwrap_or(""),
        words.next().unwrap_or(""),
        words.next().unwrap_or(""),
    );
    if !subject.eq_ignore_ascii_case("project") || verb != "starts" || words.next().is_some() {
        return Err(invalid(
            line.number,
            format!(
                "Gantt requires explicit Project starts YYYY-MM-DD before tasks; found {:?}",
                line.text
            ),
        ));
    }
    CivilDay::parse(date, line)
}
impl Parser<'_> {
    fn statement(&mut self, line: &Line) -> Result<(), RenderError> {
        let (display, rest) = bracket(&line.text, line)?;
        let (alias, tail) = if let Some(rest) = word(rest, "as") {
            let (alias, tail) = bracket(rest, line)?;
            (alias, tail)
        } else {
            (display, rest)
        };
        let (verb, rest) = tail
            .split_once(char::is_whitespace)
            .ok_or_else(|| unsupported(line.number, &format!("Gantt statement {:?}", line.text)))?;
        let rest = rest.trim();
        let id = self.task_id((alias, display), line)?;
        match verb {
            "lasts" | "requires" => self.duration(id, rest, line),
            "starts" => self.set_start((id, Kind::Bar), rest, line),
            "happens" => self.set_start((id, Kind::Milestone), rest, line),
            _ => Err(unsupported(
                line.number,
                &format!("Gantt statement {}", line.text),
            )),
        }
    }
    fn task_id(&mut self, names: (&str, &str), line: &Line) -> Result<usize, RenderError> {
        let (code, display) = names;
        self.budget
            .charge(search_cost(self.registry.len()).saturating_mul(code.len() + 1))?;
        if let Some(&id) = self.registry.get(code) {
            if code != display && self.tasks[id].task.display != display {
                return Err(invalid(
                    line.number,
                    format!("conflicting task alias {code:?} in {:?}", line.text),
                ));
            }
            return Ok(id);
        }
        self.budget.item()?;
        bound("label bytes", display.len(), 16 * 1024)?;
        bound("task identity bytes", code.len(), 16 * 1024)?;
        self.budget
            .text(code.len().saturating_mul(2) + display.len())?;
        let id = self.tasks.len();
        let task = Task {
            code: code.into(),
            display: display.into(),
            start: self.project,
            end: self.project.shift(1, line.number)?,
            kind: Kind::Bar,
        };
        self.tasks.push(Builder {
            task,
            duration: Spec::Default,
            start: Spec::Default,
            frozen: Spec::Default,
            parent: Parent::Root,
        });
        self.registry.insert(code.into(), id);
        Ok(id)
    }
    fn duration(&mut self, id: usize, rest: &str, line: &Line) -> Result<(), RenderError> {
        self.check_frozen(id, line)?;
        let days = parse_days(rest, line)?;
        let task = &mut self.tasks[id];
        if matches!(task.duration, Spec::Explicit) || task.task.kind == Kind::Milestone {
            return Err(invalid(
                line.number,
                format!("conflicting duration in {:?}", line.text),
            ));
        }
        task.task.end = task.task.start.shift(days, line.number)?;
        task.duration = Spec::Explicit;
        Ok(())
    }
    fn set_start(
        &mut self,
        target: (usize, Kind),
        rest: &str,
        line: &Line,
    ) -> Result<(), RenderError> {
        let (id, kind) = target;
        let rest = complement(rest, kind, line)?;
        let (start, parent) = if rest.starts_with('[')
            || rest.starts_with(|c: char| c.is_ascii_digit()) && rest.contains('[')
            || word(rest, "after").is_some()
            || word(rest, "with").is_some()
        {
            self.reference((id, kind), rest, line)?
        } else {
            (CivilDay::parse(rest, line)?, Parent::Root)
        };
        self.check_frozen(id, line)?;
        if matches!(self.tasks[id].start, Spec::Explicit)
            || kind == Kind::Milestone && matches!(self.tasks[id].duration, Spec::Explicit)
        {
            return Err(invalid(
                line.number,
                format!("conflicting start/milestone in {:?}", line.text),
            ));
        }
        if start < self.project {
            return Err(invalid(
                line.number,
                format!("task starts before project date in {:?}", line.text),
            ));
        }
        let task = &mut self.tasks[id];
        let days = if kind == Kind::Milestone {
            1
        } else {
            task.task.end.0 - task.task.start.0
        };
        task.task.end = start.shift(days, line.number)?;
        task.task.start = start;
        task.task.kind = kind;
        task.start = Spec::Explicit;
        task.parent = parent;
        Ok(())
    }
    fn reference(
        &mut self,
        target: (usize, Kind),
        rest: &str,
        line: &Line,
    ) -> Result<(CivilDay, Parent), RenderError> {
        let (prefix, tail) = rest
            .split_once('[')
            .ok_or_else(|| unsupported(line.number, &line.text))?;
        let offset = parse_offset(prefix.trim(), line)?;
        let (code, suffix) = bracket(&rest[rest.len() - tail.len() - 1..], line)?;
        let endpoint = match suffix.strip_prefix("'s").map(str::trim) {
            Some("start") => Endpoint::Start,
            Some("end") => Endpoint::End,
            _ => return Err(unsupported(line.number, &line.text)),
        };
        self.budget
            .charge(search_cost(self.registry.len()).saturating_mul(code.len() + 1))?;
        let from = self.registry.get(code).copied().ok_or_else(|| {
            invalid(
                line.number,
                format!("unknown task {code:?} in {:?}", line.text),
            )
        })?;
        self.check_cycle(target.0, from, line)?;
        let predecessor = &self.tasks[from];
        if predecessor.task.kind == Kind::Milestone {
            return Err(unsupported(
                line.number,
                &format!("dependency on milestone in {}", line.text),
            ));
        }
        if matches!(predecessor.duration, Spec::Default) {
            return Err(invalid(
                line.number,
                format!(
                    "predecessor duration must be explicit before reference in {:?}",
                    line.text
                ),
            ));
        }
        let boundary = match endpoint {
            Endpoint::Start => predecessor.task.start,
            Endpoint::End => predecessor.task.end,
        };
        let adjustment = if target.1 == Kind::Milestone && endpoint == Endpoint::End {
            -1
        } else {
            0
        };
        let start = boundary.shift(offset + adjustment, line.number)?;
        self.budget.item()?;
        self.dependencies.push(Dependency {
            from,
            to: target.0,
            endpoint,
            offset,
            kind: target.1,
        });
        self.tasks[from].frozen = Spec::Explicit;
        Ok((start, Parent::Task(from)))
    }
    fn check_cycle(
        &mut self,
        target: usize,
        mut predecessor: usize,
        line: &Line,
    ) -> Result<(), RenderError> {
        loop {
            self.budget.charge(1)?;
            if predecessor == target {
                return Err(invalid(
                    line.number,
                    format!("cyclic Gantt dependency in {:?}", line.text),
                ));
            }
            match self.tasks[predecessor].parent {
                Parent::Root => return Ok(()),
                Parent::Task(parent) => predecessor = parent,
            }
        }
    }
    fn check_frozen(&self, id: usize, line: &Line) -> Result<(), RenderError> {
        if matches!(self.tasks[id].frozen, Spec::Explicit) {
            return Err(invalid(
                line.number,
                format!(
                    "task {:?} is frozen after reference; later schedule edit {:?} is unsupported",
                    self.tasks[id].task.code, line.text
                ),
            ));
        }
        Ok(())
    }
    fn finish(self) -> Result<Gantt, RenderError> {
        if self.tasks.is_empty() {
            return Err(invalid(1, "Gantt requires at least one task"));
        }
        for builder in &self.tasks {
            self.budget.charge(1)?;
            let span = builder.task.end.0 - self.project.0;
            if span > MAX_SPAN {
                return Err(invalid(
                    1,
                    format!(
                        "Gantt date span {span} for {:?} exceeds {MAX_SPAN} days",
                        builder.task.code
                    ),
                ));
            }
        }
        self.budget.charge(self.tasks.len())?;
        Ok(Gantt {
            project_start: self.project,
            tasks: self.tasks.into_iter().map(|b| b.task).collect(),
            dependencies: self.dependencies,
        })
    }
}
fn bracket<'a>(text: &'a str, line: &Line) -> Result<(&'a str, &'a str), RenderError> {
    if let Some(rest) = text.strip_prefix('[')
        && let Some((value, tail)) = rest.split_once(']')
        && !value.is_empty()
        && !value.contains('[')
    {
        return Ok((value, tail.trim_start()));
    }
    Err(unsupported(
        line.number,
        &format!("Gantt task/project statement {}", line.text),
    ))
}
fn word<'a>(text: &'a str, word: &str) -> Option<&'a str> {
    text.strip_prefix(word)
        .filter(|tail| tail.starts_with(char::is_whitespace))
        .map(str::trim_start)
}
fn parse_days(text: &str, line: &Line) -> Result<i32, RenderError> {
    let mut words = text.split_whitespace();
    let value = number(words.next().unwrap_or(""), line)?;
    if !matches!(words.next(), Some("day" | "days")) || words.next().is_some() || value == 0 {
        return Err(unsupported(
            line.number,
            &format!("positive whole-day duration required in {}", line.text),
        ));
    }
    Ok(value)
}
fn number(text: &str, line: &Line) -> Result<i32, RenderError> {
    if text.is_empty() || text.len() > 4 || !text.bytes().all(|c| c.is_ascii_digit()) {
        return Err(invalid(
            line.number,
            format!("bounded whole-day number required in {:?}", line.text),
        ));
    }
    let value = text.parse::<i32>().unwrap();
    if value > MAX_SPAN {
        return Err(invalid(
            line.number,
            format!("day value {value} exceeds {MAX_SPAN} in {:?}", line.text),
        ));
    }
    Ok(value)
}
fn parse_offset(text: &str, line: &Line) -> Result<i32, RenderError> {
    if matches!(text, "" | "at" | "after" | "with") {
        return Ok(0);
    }
    let mut words = text.split_whitespace();
    let value = number(words.next().unwrap_or(""), line)?;
    if !matches!(words.next(), Some("day" | "days")) {
        return Err(unsupported(line.number, &line.text));
    }
    let offset = match words.next() {
        Some("before") => -value,
        Some("after") => value,
        _ => return Err(unsupported(line.number, &line.text)),
    };
    if words.next().is_some() {
        return Err(unsupported(line.number, &line.text));
    }
    Ok(offset)
}
fn complement<'a>(text: &'a str, kind: Kind, line: &Line) -> Result<&'a str, RenderError> {
    let prefix = word(text, "on").or_else(|| word(text, "at"));
    if kind == Kind::Bar
        && text.contains('[')
        && (word(text, "on").is_some() || prefix.is_some_and(|tail| !tail.starts_with('[')))
    {
        return Err(unsupported(
            line.number,
            &format!("Gantt start complement {}", line.text),
        ));
    }
    Ok(prefix.unwrap_or(text))
}
fn search_cost(len: usize) -> usize {
    len.checked_ilog2().unwrap_or(0) as usize + 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RenderOptions;

    fn model(source: &str) -> Gantt {
        let mut budget = Budget::new(&RenderOptions::default());
        let (_, lines) = super::super::source::prepare_document(source, &mut budget).unwrap();
        parse(&lines, &mut budget).unwrap()
    }
    #[test]
    fn should_keep_typed_task_identity_row_order_and_reference_ownership() {
        // given
        let source = include_str!("../../../fixtures/plantuml/ganttalias.puml");
        // when
        let model = model(source);
        // then
        assert_eq!(
            model
                .tasks
                .iter()
                .map(|t| t.code.as_str())
                .collect::<Vec<_>>(),
            ["draft", "build", "review", "Publish checkpoint"]
        );
        assert_eq!(model.tasks[0].display, "Draft brief");
        assert_eq!(model.tasks[1].start.0 - model.tasks[0].start.0, 2);
        assert_eq!(model.tasks[1].end.0 - model.tasks[1].start.0, 3);
        assert_eq!(model.tasks[2].start.0, model.tasks[1].end.0 - 1);
        assert_eq!(model.tasks[3].start.0, model.tasks[1].end.0);
        assert_eq!(model.tasks[2].kind, Kind::Milestone);
        assert_eq!(
            model
                .dependencies
                .iter()
                .map(|d| (d.from, d.to, d.endpoint, d.offset, d.kind))
                .collect::<Vec<_>>(),
            [
                (0, 1, Endpoint::End, 0, Kind::Bar),
                (1, 2, Endpoint::End, 0, Kind::Milestone),
                (1, 3, Endpoint::End, 1, Kind::Milestone)
            ]
        );
    }
    #[test]
    fn should_round_trip_bounded_civil_ordinals_and_gregorian_centuries() {
        // given
        let row = Line {
            number: 1,
            text: String::new(),
        };
        for (date, ordinal) in [
            ("0001-01-01", 0),
            ("2000-01-01", 730119),
            ("2000-02-29", 730178),
            ("9999-12-31", 3652058),
        ] {
            // when
            let day = CivilDay::parse(date, &row).unwrap();
            // then
            assert_eq!(day.0, ordinal);
            assert_eq!(day.iso(), date);
        }
        assert_eq!(
            CivilDay::parse("1900-02-28", &row)
                .unwrap()
                .shift(1, 1)
                .unwrap()
                .iso(),
            "1900-03-01"
        );
        assert!(CivilDay(0).shift(-1, 1).is_err());
        assert!(CivilDay(MAX_DAY).shift(1, 1).is_err());
    }
    #[test]
    fn should_freeze_predecessor_fields_without_reactive_rescheduling() {
        // given
        for edit in ["[A] starts 2026-11-05", "[A] lasts 9 days"] {
            let source = format!(
                "@startgantt\nProject starts 2026-11-02\n[A] lasts 2 days\n[B] starts at [A]'s end\n{edit}\n@endgantt"
            );
            let mut budget = Budget::new(&RenderOptions::default());
            let (_, lines) = super::super::source::prepare_document(&source, &mut budget).unwrap();
            // when
            let error = match parse(&lines, &mut budget) {
                Err(e) => e.to_string(),
                Ok(_) => panic!("late mutation admitted"),
            };
            // then
            assert!(error.contains("frozen after reference"));
            assert!(error.contains(edit));
        }
    }
}
