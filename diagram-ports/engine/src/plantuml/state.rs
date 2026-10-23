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
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Kind {
    State,
    Composite,
    Initial,
    Final,
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
}
#[derive(Debug)]
pub(super) struct State {
    pub nodes: Vec<Node>,
    pub transitions: Vec<Transition>,
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
    Transition(Name, Name, String),
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
            },
            scopes: vec![],
            pending: vec![],
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
        input: (Declaration, Caption),
        span: Range<usize>,
    ) -> Result<(), ParseFailure> {
        let (value, label) = input;
        if value.name == "begin" && matches!(value.spelling, Spelling::Identifier) {
            return Err(ParseFailure::unsupported(
                span,
                "state opener requires a preceding declaration",
            ));
        }
        let id = self.add(value, Kind::State, span.clone())?;
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

    pub fn describe(
        &mut self,
        value: (Name, String),
        span: Range<usize>,
    ) -> Result<(), ParseFailure> {
        self.defer(Statement::Description(value.0, value.1), span)
    }

    pub fn transition(
        &mut self,
        input: (Name, Name, bool, Caption),
        span: Range<usize>,
    ) -> Result<(), ParseFailure> {
        let (left, right, reverse, label) = input;
        if left.quoted || right.quoted {
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
            .charge(self.model.nodes.len() + name.len() + 1)
            .map_err(ParseFailure::terminal)?;
        if name.contains('.') || (name.contains(':') && matches!(spelling, Spelling::Identifier)) {
            return Err(ParseFailure::unsupported(
                span,
                "qualified state declaration",
            ));
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
                    let id = self.resolve((&name.text, scope, Kind::State), span)?;
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
                    let from = self.resolve((&from.text, scope, Kind::Initial), span.clone())?;
                    let to = self.resolve((&to.text, scope, Kind::Final), span)?;
                    self.budget.item().map_err(ParseFailure::terminal)?;
                    self.budget
                        .text(label.len())
                        .map_err(ParseFailure::terminal)?;
                    retain(
                        (&mut self.model.transitions, &mut self.retained_bytes),
                        Transition { from, to, label },
                        self.budget,
                    )?;
                }
            }
        }
        Ok(self.model)
    }

    fn resolve(
        &mut self,
        input: (&str, Scope, Kind),
        span: Range<usize>,
    ) -> Result<usize, ParseFailure> {
        let (name, scope, role) = input;
        let kind = if name == "[*]" { role } else { Kind::State };
        self.budget
            .charge(self.model.nodes.len() + name.len() + 1)
            .map_err(ParseFailure::terminal)?;
        if name != "[*]" && !is_name(name) {
            return Err(ParseFailure::unsupported(
                span,
                "state endpoint; history/concurrency/decorations are unsupported",
            ));
        }
        if let Some(id) = find(&self.model, (name, scope, kind)) {
            return Ok(id);
        }
        if kind == Kind::State
            && find_ancestor(&self.model, (name, scope), self.budget)
                .map_err(ParseFailure::terminal)?
                .is_some()
        {
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
    while let Scope::State(id) = scope {
        scope = model.nodes[id].scope;
        budget.charge(model.nodes.len() + 1)?;
        if let Some(id) = find(model, (name, scope, Kind::State)) {
            return Ok(Some(id));
        }
    }
    Ok(None)
}

fn find(model: &State, input: (&str, Scope, Kind)) -> Option<usize> {
    let (name, scope, kind) = input;
    if matches!(kind, Kind::Initial | Kind::Final) {
        return model
            .nodes
            .iter()
            .position(|n| n.scope == scope && n.kind == kind);
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
            super::super::parser::error::FailureKind::Unsupported("cross-ownership state endpoint")
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
