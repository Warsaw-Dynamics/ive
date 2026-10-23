//! Day-only Gantt grammar and boundary semantics from PlantUML MIT 1.2026.9.
//! Copyright 2009-2024 Arnaud Roques; exact source traces/notices in provenance.
use super::{
    budget::{Budget, bound},
    parser::{
        context,
        error::ParseFailure,
        gantt::{Command, Schedule},
        timing::Site,
    },
    source::Line,
};
use crate::RenderError;
use std::collections::BTreeMap;

pub(super) const MAX_SPAN: i32 = 7300;
const MAX_DAY: i32 = 3_652_058; // 9999-12-31, with 0001-01-01 at zero.

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct CivilDay(pub i32);
impl CivilDay {
    pub fn parse(text: &str, line: &Line) -> Result<Self, RenderError> {
        let bytes = text.as_bytes();
        if bytes.len() != 10
            || bytes[4] != b'-'
            || bytes[7] != b'-'
            || bytes
                .iter()
                .enumerate()
                .any(|(i, c)| i != 4 && i != 7 && !c.is_ascii_digit())
        {
            return Err(super::budget::invalid(
                line.number,
                format!("expected ISO civil date YYYY-MM-DD in {:?}", line.text),
            ));
        }
        let year = text[..4].parse::<i32>().unwrap();
        let month = text[5..7].parse::<i32>().unwrap();
        let day = text[8..].parse::<i32>().unwrap();
        if year == 0 || !(1..=12).contains(&month) || !(1..=month_days(year, month)).contains(&day)
        {
            return Err(super::budget::invalid(
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
                super::budget::invalid(
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
struct TaskBuilder {
    site: Site,
    task: Task,
    duration: Spec,
    start: Spec,
    frozen: Spec,
    parent: Parent,
}
pub(super) struct Builder<'a> {
    project: CivilDay,
    tasks: Vec<TaskBuilder>,
    registry: BTreeMap<String, usize>,
    dependencies: Vec<Dependency>,
    pub budget: &'a mut Budget,
}
fn invalid(start: usize, message: impl std::fmt::Display) -> ParseFailure {
    ParseFailure::owned_syntax(start..start, format!("{message}"))
}
fn unsupported(start: usize, message: &str) -> ParseFailure {
    ParseFailure {
        span: start..start,
        kind: super::parser::error::FailureKind::Unsupported(message.to_owned().into()),
    }
}
fn shift(day: CivilDay, delta: i32, site: &Site) -> Result<CivilDay, ParseFailure> {
    day.0
        .checked_add(delta)
        .filter(|value| (0..=MAX_DAY).contains(value))
        .map(CivilDay)
        .ok_or_else(|| {
            invalid(
                site.span.start,
                "civil date arithmetic exceeds years 0001..9999",
            )
        })
}
impl<'a> Builder<'a> {
    pub fn retained_bytes(items: usize) -> usize {
        let vector = std::mem::size_of::<TaskBuilder>()
            + std::mem::size_of::<Task>()
            + std::mem::size_of::<Dependency>();
        let tree_node =
            12 * (std::mem::size_of::<(String, usize)>() + std::mem::size_of::<usize>());
        items.saturating_mul(3 * vector + tree_node)
    }
    pub fn tick(&mut self) -> Result<(), ParseFailure> {
        self.budget.charge(1).map_err(Into::into)
    }
    pub fn new(budget: &'a mut Budget) -> Self {
        Self {
            project: CivilDay(0),
            tasks: Vec::new(),
            registry: BTreeMap::new(),
            dependencies: Vec::new(),
            budget,
        }
    }
    pub fn project(&mut self, day: CivilDay) -> Result<(), ParseFailure> {
        self.budget.item()?;
        self.project = day;
        Ok(())
    }
    pub fn apply(
        &mut self,
        names: (String, String),
        command: Command,
        site: Site,
    ) -> Result<(), ParseFailure> {
        self.budget.charge(1)?;
        let id = self.task_id((&names.0, &names.1), &site)?;
        match command {
            Command::Duration(days) => self.duration(id, days, &site),
            Command::Start(kind, schedule) => self.set_start((id, kind), schedule, &site),
        }
    }
    fn task_id(&mut self, names: (&str, &str), line: &Site) -> Result<usize, ParseFailure> {
        let (code, display) = names;
        self.budget
            .charge(search_cost(self.registry.len()).saturating_mul(code.len() + 1))?;
        if let Some(&id) = self.registry.get(code) {
            if code != display && self.tasks[id].task.display != display {
                return Err(invalid(
                    line.span.start,
                    format!("conflicting task alias {code:?}"),
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
            end: shift(self.project, 1, line)?,
            kind: Kind::Bar,
        };
        context::push(
            &mut self.tasks,
            TaskBuilder {
                site: line.clone(),
                task,
                duration: Spec::Default,
                start: Spec::Default,
                frozen: Spec::Default,
                parent: Parent::Root,
            },
            self.budget,
        )?;
        self.registry.insert(code.into(), id);
        Ok(id)
    }
    fn duration(&mut self, id: usize, days: i32, line: &Site) -> Result<(), ParseFailure> {
        self.check_frozen(id, line)?;
        let task = &mut self.tasks[id];
        if matches!(task.duration, Spec::Explicit) || task.task.kind == Kind::Milestone {
            return Err(invalid(line.span.start, "conflicting duration"));
        }
        task.task.end = shift(task.task.start, days, line)?;
        task.duration = Spec::Explicit;
        Ok(())
    }
    fn set_start(
        &mut self,
        target: (usize, Kind),
        schedule: Schedule,
        line: &Site,
    ) -> Result<(), ParseFailure> {
        let (id, kind) = target;
        let (start, parent) = match schedule {
            Schedule::Date(day) => (day, Parent::Root),
            Schedule::Reference {
                code,
                endpoint,
                offset,
            } => self.reference((id, kind), (code, endpoint, offset), line)?,
        };
        self.check_frozen(id, line)?;
        if matches!(self.tasks[id].start, Spec::Explicit)
            || kind == Kind::Milestone && matches!(self.tasks[id].duration, Spec::Explicit)
        {
            return Err(invalid(line.span.start, "conflicting start/milestone"));
        }
        if start < self.project {
            return Err(invalid(line.span.start, "task starts before project date"));
        }
        let task = &mut self.tasks[id];
        let days = if kind == Kind::Milestone {
            1
        } else {
            task.task.end.0 - task.task.start.0
        };
        task.task.end = shift(start, days, line)?;
        task.task.start = start;
        task.task.kind = kind;
        task.start = Spec::Explicit;
        task.parent = parent;
        Ok(())
    }
    fn reference(
        &mut self,
        target: (usize, Kind),
        reference: (String, Endpoint, i32),
        line: &Site,
    ) -> Result<(CivilDay, Parent), ParseFailure> {
        let (code, endpoint, offset) = reference;
        self.budget
            .charge(search_cost(self.registry.len()).saturating_mul(code.len() + 1))?;
        let from = self
            .registry
            .get(&code)
            .copied()
            .ok_or_else(|| invalid(line.span.start, format!("unknown task {code:?}")))?;
        self.check_cycle(target.0, from, line)?;
        let predecessor = &self.tasks[from];
        if predecessor.task.kind == Kind::Milestone {
            return Err(unsupported(line.span.start, "dependency on milestone"));
        }
        if matches!(predecessor.duration, Spec::Default) {
            return Err(invalid(
                line.span.start,
                "predecessor duration must be explicit before reference",
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
        let start = shift(boundary, offset + adjustment, line)?;
        self.budget.item()?;
        context::push(
            &mut self.dependencies,
            Dependency {
                from,
                to: target.0,
                endpoint,
                offset,
                kind: target.1,
            },
            self.budget,
        )?;
        self.tasks[from].frozen = Spec::Explicit;
        Ok((start, Parent::Task(from)))
    }
    fn check_cycle(
        &mut self,
        target: usize,
        mut predecessor: usize,
        line: &Site,
    ) -> Result<(), ParseFailure> {
        loop {
            self.budget.charge(1)?;
            if predecessor == target {
                return Err(invalid(line.span.start, "cyclic Gantt dependency"));
            }
            match self.tasks[predecessor].parent {
                Parent::Root => return Ok(()),
                Parent::Task(parent) => predecessor = parent,
            }
        }
    }
    fn check_frozen(&self, id: usize, line: &Site) -> Result<(), ParseFailure> {
        if matches!(self.tasks[id].frozen, Spec::Explicit) {
            return Err(invalid(
                line.span.start,
                format!(
                    "task {:?} is frozen after reference; later schedule edit is unsupported",
                    self.tasks[id].task.code
                ),
            ));
        }
        Ok(())
    }
    pub fn finish(self, site: Site) -> Result<Gantt, ParseFailure> {
        if self.tasks.is_empty() {
            return Err(invalid(site.span.start, "Gantt requires at least one task"));
        }
        for builder in &self.tasks {
            self.budget.charge(1)?;
            let span = builder.task.end.0 - self.project.0;
            if span > MAX_SPAN {
                return Err(invalid(
                    builder.site.span.start,
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
fn search_cost(len: usize) -> usize {
    len.checked_ilog2().unwrap_or(0) as usize + 1
}
#[cfg(test)]
pub(super) fn parse(lines: &[Line], budget: &mut Budget) -> Result<Gantt, RenderError> {
    super::parser::gantt::parse_test(lines, budget)
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
