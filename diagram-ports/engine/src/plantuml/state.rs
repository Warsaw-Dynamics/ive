//! MIT PlantUML scoped state identity and two-phase transition resolution.
//! Exact source hashes/notices are recorded in the provenance manifest.
use super::{
    budget::{Budget, bound},
    parser::error::ParseFailure,
};
use crate::RenderError;
use std::ops::Range;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum Scope {
    Root,
    State(usize),
    Region(usize),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Kind {
    State,
    Composite,
    Initial,
    Final,
    History,
    DeepHistory,
    Choice,
    Junction,
    ForkJoin,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Axis {
    Stacked,
    SideBySide,
}
#[derive(Debug)]
pub(super) struct Region {
    pub owner: usize,
    pub scope: Scope,
    pub ordinal: usize,
    pub axis: Axis,
}
#[derive(Debug)]
pub(super) struct Node {
    span: Range<usize>,
    pub name: String,
    pub label: String,
    pub descriptions: Vec<String>,
    pub scope: Scope,
    pub kind: Kind,
}
#[derive(Debug)]
pub(super) struct Transition {
    pub from: usize,
    pub to: usize,
    pub label: String,
    pub span: Range<usize>,
}
#[derive(Debug)]
pub(super) struct State {
    pub nodes: Vec<Node>,
    pub transitions: Vec<Transition>,
    pub regions: Vec<Region>,
}
pub(in crate::plantuml) struct NamePath {
    pub names: Vec<Name>,
    pub end: usize,
}
pub(in crate::plantuml) enum Endpoint {
    Named(NamePath),
    Role,
    History(Kind),
    ScopedHistory(NamePath, Kind),
}

#[derive(Clone, Debug)]
pub(in crate::plantuml) struct Name {
    pub text: String,
    pub quoted: bool,
}
pub(in crate::plantuml) struct Declaration {
    name: String,
    label: String,
    spelling: Spelling,
}
enum Spelling {
    Quoted,
    Identifier,
}
pub(in crate::plantuml) enum Caption {
    Absent,
    Present(String),
}
enum Statement {
    Description(Name, String),
    Transition(Endpoint, Endpoint, String),
}
struct Pending {
    statement: Statement,
    scope: Scope,
    span: Range<usize>,
}

pub(in crate::plantuml) struct Builder<'b> {
    pub budget: &'b mut Budget,
    model: State,
    scopes: Vec<Scope>,
    pending: Vec<Pending>,
    retained_bytes: usize,
}

impl<'b> Builder<'b> {
    pub fn new(budget: &'b mut Budget, retained_bytes: usize) -> Self {
        Self {
            budget,
            model: State {
                nodes: vec![],
                transitions: vec![],
                regions: vec![],
            },
            scopes: vec![],
            pending: vec![],
            retained_bytes,
        }
    }

    pub fn tick(&mut self) -> Result<(), ParseFailure> {
        self.budget.charge(1).map_err(ParseFailure::terminal)
    }

    pub fn path(&mut self, name: Name, start: usize) -> Result<NamePath, ParseFailure> {
        let end = start + name.text.len();
        let mut names = Vec::new();
        retain((&mut names, &mut self.retained_bytes), name, self.budget)?;
        Ok(NamePath { names, end })
    }

    pub fn extend_path(
        &mut self,
        input: (NamePath, Name),
        positions: (usize, usize),
    ) -> Result<NamePath, ParseFailure> {
        let (mut path, name) = input;
        let (dot, start) = positions;
        if path.names.iter().any(|n| n.quoted) || name.quoted || path.end != dot || dot + 1 != start
        {
            return Err(ParseFailure::unsupported(
                dot..start,
                "qualified state names require contiguous unquoted components",
            ));
        }
        self.budget.depth(path.names.len() + 1)?;
        path.end = start + name.text.len();
        retain(
            (&mut path.names, &mut self.retained_bytes),
            name,
            self.budget,
        )?;
        Ok(path)
    }

    pub fn single(&mut self, name: Name) -> Result<Declaration, ParseFailure> {
        self.budget
            .charge(name.text.len())
            .map_err(ParseFailure::terminal)?;
        Ok(Declaration {
            name: name.text.clone(),
            label: name.text,
            spelling: if name.quoted {
                Spelling::Quoted
            } else {
                Spelling::Identifier
            },
        })
    }

    pub fn alias(
        &mut self,
        names: (Name, Name),
        span: Range<usize>,
    ) -> Result<Declaration, ParseFailure> {
        self.tick()?;
        let (left, right) = names;
        if left.quoted == right.quoted {
            return Err(ParseFailure::unsupported(
                span,
                "state alias requires one quoted display",
            ));
        }
        let (name, label) = if left.quoted {
            (right.text, left.text)
        } else {
            (left.text, right.text)
        };
        Ok(Declaration {
            name,
            label,
            spelling: Spelling::Identifier,
        })
    }

    pub fn declare(
        &mut self,
        input: (Declaration, Kind, Caption),
        span: Range<usize>,
    ) -> Result<(), ParseFailure> {
        let (value, kind, label) = input;
        if value.name == "begin" && matches!(value.spelling, Spelling::Identifier) {
            return Err(ParseFailure::unsupported(
                span,
                "state opener requires a preceding declaration",
            ));
        }
        let id = self.add(value, kind, span.clone())?;
        if let Caption::Present(label) = label {
            self.budget
                .charge(self.model.nodes[id].name.len())
                .map_err(ParseFailure::terminal)?;
            let name = Name {
                text: self.model.nodes[id].name.clone(),
                quoted: false,
            };
            self.describe((name, label), span)?;
        }
        Ok(())
    }

    pub fn open(&mut self, value: Declaration, span: Range<usize>) -> Result<(), ParseFailure> {
        self.budget
            .depth(self.scopes.len() + 1)
            .map_err(ParseFailure::terminal)?;
        if matches!(value.spelling, Spelling::Quoted) {
            return Err(ParseFailure::unsupported(
                span,
                "composite state requires an identifier or alias",
            ));
        }
        let id = self.add(value, Kind::Composite, span)?;
        retain(
            (&mut self.scopes, &mut self.retained_bytes),
            Scope::State(id),
            self.budget,
        )
    }

    pub fn close(&mut self) -> Result<(), ParseFailure> {
        self.tick()?;
        self.scopes.pop();
        Ok(())
    }

    pub fn separate(
        &mut self,
        input: (Axis, usize),
        span: Range<usize>,
    ) -> Result<(), ParseFailure> {
        let (axis, length) = input;
        self.tick()?;
        if length < 2 {
            return Err(ParseFailure::unsupported(
                span,
                "state concurrent separator requires at least two contiguous markers",
            ));
        }
        let owner = match self.scope() {
            Scope::State(id) => id,
            Scope::Region(id) => self.model.regions[id].owner,
            Scope::Root => {
                return Err(ParseFailure::unsupported(
                    span,
                    "root state concurrent separators are unsupported",
                ));
            }
        };
        self.budget.charge(self.model.regions.len())?;
        let axis = if let Some(region) = self.model.regions.iter().find(|r| r.owner == owner) {
            region.axis
        } else {
            self.budget.item()?;
            retain(
                (&mut self.model.regions, &mut self.retained_bytes),
                Region {
                    owner,
                    scope: Scope::State(owner),
                    ordinal: 0,
                    axis,
                },
                self.budget,
            )?;
            axis
        };
        let ordinal = self
            .model
            .regions
            .iter()
            .filter(|r| r.owner == owner)
            .count();
        let id = self.model.regions.len();
        self.budget.item()?;
        retain(
            (&mut self.model.regions, &mut self.retained_bytes),
            Region {
                owner,
                scope: Scope::Region(id),
                ordinal,
                axis,
            },
            self.budget,
        )?;
        *self.scopes.last_mut().unwrap() = Scope::Region(id);
        Ok(())
    }

    pub fn stereotype(&mut self, name: Name, span: Range<usize>) -> Result<Kind, ParseFailure> {
        self.budget.charge(name.text.len())?;
        if name.quoted {
            return Err(ParseFailure::unsupported(
                span,
                "quoted state stereotype is unsupported",
            ));
        }
        match name.text.to_ascii_lowercase().as_str() {
            "choice" => Ok(Kind::Choice),
            "junction" => Ok(Kind::Junction),
            "fork" | "join" => Ok(Kind::ForkJoin),
            "start" => Ok(Kind::Initial),
            "end" => Ok(Kind::Final),
            "history" => Ok(Kind::History),
            "history*" => Ok(Kind::DeepHistory),
            _ => Err(ParseFailure::unsupported(
                span,
                "unsupported state stereotype",
            )),
        }
    }

    pub fn describe(
        &mut self,
        value: (Name, String),
        span: Range<usize>,
    ) -> Result<(), ParseFailure> {
        self.defer(Statement::Description(value.0, value.1), span)
    }

    pub fn transition(
        &mut self,
        input: (Endpoint, Endpoint, bool, Caption),
        span: Range<usize>,
    ) -> Result<(), ParseFailure> {
        let (left, right, reverse, label) = input;
        if [&left, &right].iter().any(|value| matches!(value, Endpoint::Named(path) | Endpoint::ScopedHistory(path, _) if path.names.iter().any(|name|name.quoted))) {
            return Err(ParseFailure::unsupported(
                span,
                "quoted state transition endpoint",
            ));
        }
        let (from, to) = if reverse {
            (right, left)
        } else {
            (left, right)
        };
        let label = match label {
            Caption::Absent => String::new(),
            Caption::Present(label) if label.is_empty() => {
                return Err(ParseFailure::unsupported(
                    span,
                    "empty state transition label",
                ));
            }
            Caption::Present(label) => label,
        };
        self.defer(Statement::Transition(from, to, label), span)
    }

    fn scope(&self) -> Scope {
        self.scopes.last().copied().unwrap_or(Scope::Root)
    }

    fn defer(&mut self, statement: Statement, span: Range<usize>) -> Result<(), ParseFailure> {
        self.budget.item().map_err(ParseFailure::terminal)?;
        let value = Pending {
            statement,
            scope: self.scope(),
            span,
        };
        retain(
            (&mut self.pending, &mut self.retained_bytes),
            value,
            self.budget,
        )
    }

    fn add(
        &mut self,
        value: Declaration,
        kind: Kind,
        span: Range<usize>,
    ) -> Result<usize, ParseFailure> {
        let Declaration {
            name,
            label,
            spelling,
        } = value;
        let scope = self.scope();
        self.budget
            .charge(self.model.nodes.len() + name.len().saturating_mul(2) + 1)
            .map_err(ParseFailure::terminal)?;
        if name.contains('.') || (name.contains(':') && matches!(spelling, Spelling::Identifier)) {
            return Err(ParseFailure::unsupported(
                span,
                "qualified state declaration",
            ));
        }
        if matches!(spelling, Spelling::Identifier)
            && !name.chars().all(|ch| ch.is_alphanumeric() || ch == '_')
        {
            self.budget
                .text(name.len().saturating_mul(4).saturating_add(96))?;
            return Err(ParseFailure {
                span,
                kind: super::parser::error::FailureKind::Unsupported(format!("state declaration identifier {name:?} requires letters, numbers or underscores").into()),
            });
        }
        if self
            .model
            .nodes
            .iter()
            .any(|node| node.scope == scope && node.name == name)
        {
            return Err(ParseFailure::unsupported(
                span,
                "duplicate state declaration",
            ));
        }
        self.budget.item().map_err(ParseFailure::terminal)?;
        self.budget
            .text(name.len() + label.len())
            .map_err(ParseFailure::terminal)?;
        let id = self.model.nodes.len();
        retain(
            (&mut self.model.nodes, &mut self.retained_bytes),
            Node {
                span,
                name,
                label,
                descriptions: vec![],
                scope,
                kind,
            },
            self.budget,
        )?;
        Ok(id)
    }

    pub fn finish(mut self) -> Result<State, ParseFailure> {
        for node in &self.model.nodes {
            if find_ancestor(&self.model, (&node.name, node.scope), self.budget)
                .map_err(ParseFailure::terminal)?
                .is_some()
            {
                return Err(ParseFailure::unsupported(
                    node.span.clone(),
                    "cross-ownership state declaration",
                ));
            }
        }
        for Pending {
            statement,
            scope,
            span,
        } in std::mem::take(&mut self.pending)
        {
            match statement {
                Statement::Description(name, label) => {
                    let id = self.resolve((&name.text, scope, Kind::State), span.clone())?;
                    if !matches!(self.model.nodes[id].kind, Kind::State | Kind::Composite) {
                        return Err(ParseFailure::unsupported(
                            span,
                            "descriptions on state pseudostates are unsupported",
                        ));
                    }
                    self.budget
                        .text(label.len())
                        .map_err(ParseFailure::terminal)?;
                    retain(
                        (
                            &mut self.model.nodes[id].descriptions,
                            &mut self.retained_bytes,
                        ),
                        label,
                        self.budget,
                    )?;
                }
                Statement::Transition(from, to, label) => {
                    let from = self.endpoint(from, (scope, Kind::Initial), span.clone())?;
                    let to = self.endpoint(to, (scope, Kind::Final), span.clone())?;
                    if concurrent_ancestor(&self.model, self.model.nodes[from].scope, self.budget)?
                        != concurrent_ancestor(
                            &self.model,
                            self.model.nodes[to].scope,
                            self.budget,
                        )?
                    {
                        return Err(ParseFailure::unsupported(
                            span,
                            "state transition crosses concurrent region ownership",
                        ));
                    }
                    self.budget.item().map_err(ParseFailure::terminal)?;
                    self.budget
                        .text(label.len())
                        .map_err(ParseFailure::terminal)?;
                    retain(
                        (&mut self.model.transitions, &mut self.retained_bytes),
                        Transition {
                            from,
                            to,
                            label,
                            span,
                        },
                        self.budget,
                    )?;
                }
            }
        }
        Ok(self.model)
    }

    fn endpoint(
        &mut self,
        endpoint: Endpoint,
        input: (Scope, Kind),
        span: Range<usize>,
    ) -> Result<usize, ParseFailure> {
        let (scope, role) = input;
        match endpoint {
            Endpoint::Named(path) => self.resolve_path(path, scope, span),
            Endpoint::Role => self.resolve(("[*]", scope, role), span),
            Endpoint::History(kind) => self.resolve(
                (
                    if kind == Kind::History { "[H]" } else { "[H*]" },
                    scope,
                    kind,
                ),
                span,
            ),
            Endpoint::ScopedHistory(path, kind) => {
                let id = self.history_owner(path, scope, span.clone())?;
                self.resolve(
                    (
                        if kind == Kind::History { "[H]" } else { "[H*]" },
                        Scope::State(id),
                        kind,
                    ),
                    span,
                )
            }
        }
    }

    fn resolve_path(
        &mut self,
        path: NamePath,
        scope: Scope,
        span: Range<usize>,
    ) -> Result<usize, ParseFailure> {
        let mut parts = path.names.into_iter();
        let first = parts.next().unwrap();
        let mut id = self.resolve((&first.text, scope, Kind::State), span.clone())?;
        for part in parts {
            self.budget
                .charge(self.model.nodes.len().saturating_mul(part.text.len() + 1))?;
            id = find(&self.model, (&part.text, Scope::State(id), Kind::State)).ok_or_else(
                || ParseFailure::unsupported(span.clone(), "unresolved qualified state endpoint"),
            )?;
        }
        let direct = self.model.nodes[id].scope;
        if (matches!(scope, Scope::Region(_)) && direct != scope)
            || (matches!(direct, Scope::Region(_)) && direct != scope)
        {
            return Err(ParseFailure::unsupported(
                span,
                "ordinary state endpoint is outside its direct concurrent region",
            ));
        }
        Ok(id)
    }

    fn history_owner(
        &mut self,
        path: NamePath,
        mut scope: Scope,
        span: Range<usize>,
    ) -> Result<usize, ParseFailure> {
        let mut result = 0;
        for name in path.names {
            self.budget
                .charge(self.model.nodes.len().saturating_mul(name.text.len() + 1))?;
            let mut parent = scope;
            let mut depth = 0;
            while parent != Scope::Root {
                self.budget.charge(1)?;
                depth += 1;
                parent = match parent {
                    Scope::Region(id) => self.model.nodes[self.model.regions[id].owner].scope,
                    _ => parent_scope(&self.model, parent),
                };
            }
            self.budget.depth(depth + 1)?;
            let existing = if let Some(id) = find(&self.model, (&name.text, scope, Kind::State)) {
                Some(id)
            } else {
                find_ancestor(&self.model, (&name.text, scope), self.budget)?
            };
            let id = if let Some(id) = existing {
                id
            } else {
                self.resolve((&name.text, scope, Kind::State), span.clone())?
            };
            if !matches!(self.model.nodes[id].kind, Kind::State | Kind::Composite) {
                return Err(ParseFailure::unsupported(
                    span,
                    "history target is not a composite state",
                ));
            }
            self.model.nodes[id].kind = Kind::Composite;
            scope = Scope::State(id);
            result = id;
        }
        Ok(result)
    }

    fn resolve(
        &mut self,
        input: (&str, Scope, Kind),
        span: Range<usize>,
    ) -> Result<usize, ParseFailure> {
        let (name, scope, role) = input;
        let kind = if matches!(name, "[*]" | "[H]" | "[H*]") {
            role
        } else {
            Kind::State
        };
        self.budget
            .charge(self.model.nodes.len() + name.len() + 1)
            .map_err(ParseFailure::terminal)?;
        if kind == Kind::State && !is_name(name) {
            return Err(ParseFailure::unsupported(
                span,
                "unsupported state endpoint",
            ));
        }
        if let Some(id) = find(&self.model, (name, scope, kind)) {
            return Ok(id);
        }
        if kind == Kind::State
            && let Some(id) = find_ancestor(&self.model, (name, scope), self.budget)?
        {
            if primary_concurrency(&self.model, scope, self.budget)? {
                return Ok(id);
            }
            return Err(ParseFailure::unsupported(
                span,
                "cross-ownership state endpoint",
            ));
        }
        if name.contains('.') || name.contains(':') {
            return Err(ParseFailure::unsupported(
                span,
                "unresolved qualified state endpoint",
            ));
        }
        if kind == Kind::State {
            self.budget
                .charge(self.model.nodes.len().saturating_mul(name.len() + 1))?;
            for node in &self.model.nodes {
                if node.name == name
                    && node.scope != scope
                    && concurrent_ancestor(&self.model, node.scope, self.budget)?.is_some()
                {
                    return Err(ParseFailure::unsupported(
                        span,
                        "state endpoint belongs to another concurrent region",
                    ));
                }
            }
        }
        self.budget.item().map_err(ParseFailure::terminal)?;
        self.budget
            .text(name.len() * 2)
            .map_err(ParseFailure::terminal)?;
        let id = self.model.nodes.len();
        retain(
            (&mut self.model.nodes, &mut self.retained_bytes),
            Node {
                span,
                name: name.into(),
                label: if kind == Kind::State {
                    name.into()
                } else {
                    String::new()
                },
                descriptions: vec![],
                scope,
                kind,
            },
            self.budget,
        )?;
        Ok(id)
    }
}

fn is_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_alphanumeric() || matches!(c, '_' | '.' | ':'))
}

fn find_ancestor(
    model: &State,
    input: (&str, Scope),
    budget: &mut Budget,
) -> Result<Option<usize>, RenderError> {
    let (name, mut scope) = input;
    while scope != Scope::Root {
        scope = parent_scope(model, scope);
        budget.charge(model.nodes.len() + 1)?;
        if let Some(id) = find(model, (name, scope, Kind::State)) {
            return Ok(Some(id));
        }
    }
    Ok(None)
}

fn find(model: &State, input: (&str, Scope, Kind)) -> Option<usize> {
    let (name, scope, kind) = input;
    if matches!(name, "[*]" | "[H]" | "[H*]") {
        return model
            .nodes
            .iter()
            .position(|n| n.scope == scope && n.kind == kind && n.name == name);
    }
    if let Scope::State(id) = scope
        && model.nodes[id].name == name
    {
        return Some(id);
    }
    model
        .nodes
        .iter()
        .position(|n| n.scope == scope && n.name == name)
}

pub(super) fn parent_scope(model: &State, scope: Scope) -> Scope {
    match scope {
        Scope::Root => Scope::Root,
        Scope::State(id) => model.nodes[id].scope,
        Scope::Region(id) => Scope::State(model.regions[id].owner),
    }
}

fn concurrent_ancestor(
    model: &State,
    mut scope: Scope,
    budget: &mut Budget,
) -> Result<Option<usize>, RenderError> {
    while scope != Scope::Root {
        budget.charge(1)?;
        if let Scope::Region(id) = scope {
            return Ok(Some(id));
        }
        scope = parent_scope(model, scope);
    }
    Ok(None)
}

fn primary_concurrency(
    model: &State,
    mut scope: Scope,
    budget: &mut Budget,
) -> Result<bool, RenderError> {
    while scope != Scope::Root {
        budget.charge(model.regions.len() + 1)?;
        if matches!(scope, Scope::Region(_)) {
            return Ok(false);
        }
        if let Scope::State(owner) = scope
            && model.regions.iter().any(|r| r.owner == owner)
        {
            return Ok(true);
        }
        scope = parent_scope(model, scope);
    }
    Ok(false)
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
        bound(
            "state retained model arena bytes",
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
                    "PlantUML state model allocation for capacity {capacity} failed: {error}"
                )))
            })?;
        *retained = retained
            .saturating_sub(old)
            .saturating_add(values.capacity().saturating_mul(std::mem::size_of::<T>()));
        bound(
            "state actual model arena bytes",
            *retained,
            budget.limits.source_bytes.saturating_mul(256),
        )
        .map_err(ParseFailure::terminal)?;
    }
    values.push(value);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_admit_state_arena_growth_before_allocating() {
        // given
        let mut budget = Budget::new(&crate::RenderOptions::default());
        let retained = budget.limits.source_bytes * 256;
        let mut builder = Builder::new(&mut budget, retained);
        let value = Declaration {
            name: "A".into(),
            label: "A".into(),
            spelling: Spelling::Identifier,
        };
        // when
        let result = builder.add(value, Kind::State, 0..1);
        // then
        assert!(matches!(
            result.unwrap_err().kind,
            super::super::parser::error::FailureKind::Terminal(RenderError::Backend(_))
        ));
        assert_eq!(builder.model.nodes.len(), 0);
        assert_eq!(builder.model.nodes.capacity(), 0);
    }

    #[test]
    fn should_leave_state_identity_unchanged_when_rejecting_an_ancestor_endpoint() {
        // given
        let mut budget = Budget::new(&crate::RenderOptions::default());
        let document = super::super::preprocess::prepare(
            "@startuml\nstate A\nstate Outer {\nstate B\n}\n@enduml",
            &mut budget,
        )
        .unwrap();
        let mut parsed = super::super::parser::document::parse(&document, &mut budget).unwrap();
        super::super::parser::preamble::parse(&mut parsed, &mut budget).unwrap();
        let model = super::super::parser::state::parse(parsed.body, &mut budget).unwrap();
        let mut builder = Builder {
            budget: &mut budget,
            model,
            scopes: vec![],
            pending: vec![],
            retained_bytes: 0,
        };
        // when
        let result = builder.resolve(("A", Scope::State(1), Kind::State), 40..41);
        // then
        assert!(matches!(
            result.unwrap_err().kind,
            super::super::parser::error::FailureKind::Unsupported(std::borrow::Cow::Borrowed(
                "cross-ownership state endpoint"
            ))
        ));
        assert_eq!(builder.model.nodes.len(), 3);
        assert_eq!(
            builder
                .model
                .nodes
                .iter()
                .filter(|node| node.name == "A")
                .count(),
            1
        );
        assert_eq!(builder.model.nodes[0].scope, Scope::Root);
    }

    #[test]
    fn should_charge_declaration_identity_search_before_scanning() {
        // given
        let source = format!(
            "@startuml\n{}@enduml",
            (0..1000)
                .map(|i| format!("state S{i}\n"))
                .collect::<String>()
        );
        let mut options = crate::RenderOptions::default();
        options.limits.layout_work = 100_000;
        let mut budget = Budget::new(&options);
        let document = super::super::preprocess::prepare(&source, &mut budget).unwrap();
        let mut parsed = super::super::parser::document::parse(&document, &mut budget).unwrap();
        super::super::parser::preamble::parse(&mut parsed, &mut budget).unwrap();
        // when
        let result = super::super::parser::state::parse(parsed.body, &mut budget);
        // then
        assert!(
            result.is_err(),
            "identity comparisons escaped the work budget"
        );
    }
}
