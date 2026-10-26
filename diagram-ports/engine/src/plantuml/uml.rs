//! PlantUML MIT 1.2026.9: class/object/description command grammars and scoped entity semantics.
use super::parser::{
    context,
    error::ParseFailure,
    uml::{Declaration, Link},
};
use super::{arrow::Arrow, budget::Budget, scene::Decoration};
use std::ops::Range;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Kind {
    Class,
    Abstract,
    Interface,
    Enum,
    Annotation,
    Object,
    Package,
    Namespace,
    Component,
    Node,
    Database,
    Actor,
    Usecase,
    Rectangle,
    Archimate,
    Note,
}
impl Kind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Class | Self::Abstract => "class",
            Self::Interface => "interface",
            Self::Enum => "enum",
            Self::Annotation => "annotation",
            Self::Object => "object",
            Self::Package => "package",
            Self::Namespace => "namespace",
            Self::Component => "component",
            Self::Node => "node",
            Self::Database => "database",
            Self::Actor => "actor",
            Self::Usecase => "usecase",
            Self::Rectangle => "rectangle",
            Self::Archimate => "archimate",
            Self::Note => "note",
        }
    }
    pub fn members(self) -> bool {
        matches!(
            self,
            Self::Class
                | Self::Abstract
                | Self::Interface
                | Self::Enum
                | Self::Annotation
                | Self::Object
        )
    }
}
#[derive(Clone, Copy, Debug)]
pub(super) enum Parent {
    Root,
    Node(usize),
}
#[derive(Clone, Debug)]
pub(super) enum Member {
    Text(String, Decoration),
    Separator,
}
#[derive(Debug)]
pub(super) struct Entity {
    pub key: String,
    pub alias: String,
    pub label: String,
    pub kind: Kind,
    pub parent: Parent,
    pub members: Vec<Member>,
    pub container: bool,
    pub fill: Fill,
}
#[derive(Clone, Copy, Debug)]
pub(super) enum Fill {
    Default,
    Authored(super::style::Color),
}
#[derive(Debug)]
pub(super) struct Relation {
    pub from: usize,
    pub to: usize,
    pub arrow: Arrow,
    pub label: String,
    pub left: String,
    pub right: String,
}
#[derive(Clone, Copy, Debug)]
pub(super) enum Direction {
    Down,
    Right,
}
#[derive(Debug)]
pub(super) struct Uml {
    pub family: &'static str,
    pub entities: Vec<Entity>,
    pub relations: Vec<Relation>,
    pub direction: Direction,
}

struct PendingRelation {
    scope: Parent,
    span: Range<usize>,
    link: Link,
}
pub(super) struct Builder<'b> {
    pub budget: &'b mut Budget,
    model: Uml,
    groups: Vec<usize>,
    member: usize,
    pending: Vec<PendingRelation>,
}
impl<'b> Builder<'b> {
    pub fn retained_bytes(items: usize) -> usize {
        items.saturating_mul(3).saturating_mul(
            std::mem::size_of::<Entity>()
                + std::mem::size_of::<Member>()
                + std::mem::size_of::<Relation>()
                + std::mem::size_of::<PendingRelation>()
                + std::mem::size_of::<usize>(),
        )
    }
    pub fn new(budget: &'b mut Budget) -> Self {
        Self {
            budget,
            model: Uml {
                family: "description",
                entities: Vec::new(),
                relations: Vec::new(),
                direction: Direction::Down,
            },
            groups: Vec::new(),
            member: 0,
            pending: Vec::new(),
        }
    }

    pub fn tick(&mut self) -> Result<(), ParseFailure> {
        self.budget.charge(1).map_err(ParseFailure::terminal)
    }

    pub fn direct(&mut self, direction: Direction) -> Result<(), ParseFailure> {
        self.tick()?;
        self.model.direction = direction;
        Ok(())
    }

    pub fn declare(&mut self, declaration: Declaration) -> Result<usize, ParseFailure> {
        self.tick()?;
        let Declaration {
            kind,
            alias,
            label,
            fill,
            span,
        } = declaration;
        self.budget
            .charge(self.model.entities.len().saturating_mul(2))
            .map_err(ParseFailure::terminal)?;
        let is_description = |kind: Kind| {
            matches!(
                kind,
                Kind::Component
                    | Kind::Node
                    | Kind::Database
                    | Kind::Actor
                    | Kind::Usecase
                    | Kind::Rectangle
                    | Kind::Archimate
            )
        };
        if (kind.members()
            && self
                .model
                .entities
                .iter()
                .any(|entity| is_description(entity.kind)))
            || (is_description(kind)
                && self
                    .model
                    .entities
                    .iter()
                    .any(|entity| entity.kind.members()))
        {
            return Err(ParseFailure::unsupported(
                span,
                "mixed class/description declarations require allowmixing",
            ));
        }
        let scope = self
            .groups
            .last()
            .map_or("", |id| self.model.entities[*id].key.as_str());
        self.budget
            .text(scope.len() + alias.len() + 1)
            .map_err(ParseFailure::terminal)?;
        let key = if scope.is_empty() {
            alias.clone()
        } else {
            format!("{scope}.{alias}")
        };
        if self.model.entities.iter().any(|entity| entity.key == key) {
            self.budget
                .text(key.len().saturating_mul(4).saturating_add(128))
                .map_err(ParseFailure::terminal)?;
            return Err(ParseFailure::owned_syntax(
                span,
                format!("duplicate entity alias {key:?}"),
            ));
        }
        self.budget.item().map_err(ParseFailure::terminal)?;
        let id = self.model.entities.len();
        context::push(
            &mut self.model.entities,
            Entity {
                key,
                alias,
                label,
                kind,
                parent: self
                    .groups
                    .last()
                    .map_or(Parent::Root, |id| Parent::Node(*id)),
                members: Vec::new(),
                container: false,
                fill,
            },
            self.budget,
        )?;
        if kind.members() {
            self.model.family = if kind == Kind::Object && self.model.family != "class" {
                "object"
            } else {
                "class"
            };
        }
        Ok(id)
    }

    pub fn open(&mut self, declaration: Declaration) -> Result<(), ParseFailure> {
        self.budget
            .depth(self.groups.len() + 1)
            .map_err(ParseFailure::terminal)?;
        let id = self.declare(declaration)?;
        if self.model.entities[id].kind.members() {
            self.member = id;
        } else {
            self.model.entities[id].container = true;
            context::push(&mut self.groups, id, self.budget)?;
        }
        Ok(())
    }

    pub fn close(&mut self) -> Result<(), ParseFailure> {
        self.tick()?;
        self.groups.pop();
        Ok(())
    }

    pub fn append(&mut self, member: Member) -> Result<(), ParseFailure> {
        self.budget.item().map_err(ParseFailure::terminal)?;
        context::push(
            &mut self.model.entities[self.member].members,
            member,
            self.budget,
        )?;
        Ok(())
    }

    pub fn relate(&mut self, link: Link, span: Range<usize>) -> Result<(), ParseFailure> {
        self.budget.item().map_err(ParseFailure::terminal)?;
        context::push(
            &mut self.pending,
            PendingRelation {
                scope: self
                    .groups
                    .last()
                    .map_or(Parent::Root, |id| Parent::Node(*id)),
                span,
                link,
            },
            self.budget,
        )?;
        Ok(())
    }

    pub fn finish(mut self, span: Range<usize>) -> Result<Uml, ParseFailure> {
        for relation in std::mem::take(&mut self.pending) {
            self.budget
                .charge(self.model.entities.len().saturating_mul(6))
                .map_err(ParseFailure::terminal)?;
            let from = self.resolve_name(&relation.link.from, &relation)?;
            let to = self.resolve_name(&relation.link.to, &relation)?;
            if self.model.entities[from].container || self.model.entities[to].container {
                return Err(ParseFailure::unsupported(
                    relation.span,
                    "relations to container boundaries",
                ));
            }
            context::push(
                &mut self.model.relations,
                Relation {
                    from,
                    to,
                    arrow: relation.link.arrow,
                    label: relation.link.label,
                    left: relation.link.left,
                    right: relation.link.right,
                },
                self.budget,
            )?;
        }
        if self.model.entities.is_empty() {
            return Err(ParseFailure::syntax(span, "UML graph has no entities"));
        }
        Ok(self.model)
    }

    fn resolve_name(
        &mut self,
        name: &str,
        relation: &PendingRelation,
    ) -> Result<usize, ParseFailure> {
        self.tick()?;
        let scope = match relation.scope {
            Parent::Root => "",
            Parent::Node(id) => self.model.entities[id].key.as_str(),
        };
        self.budget
            .text(scope.len() + name.len() + 1)
            .map_err(ParseFailure::terminal)?;
        let scoped = if scope.is_empty() {
            name.into()
        } else {
            format!("{scope}.{name}")
        };
        if let Some(id) = self
            .model
            .entities
            .iter()
            .position(|entity| entity.key == scoped)
        {
            return Ok(id);
        }
        if let Some(id) = self
            .model
            .entities
            .iter()
            .position(|entity| entity.key == name)
        {
            return Ok(id);
        }
        let mut matches = self
            .model
            .entities
            .iter()
            .enumerate()
            .filter(|(_, entity)| entity.alias == name);
        let first = matches.next().map(|(id, _)| id);
        if matches.next().is_some() {
            self.budget
                .text(name.len().saturating_mul(4).saturating_add(128))
                .map_err(ParseFailure::terminal)?;
            return Err(ParseFailure::owned_syntax(
                relation.span.clone(),
                format!("ambiguous entity {name:?}; use its namespace"),
            ));
        }
        if let Some(id) = first {
            return Ok(id);
        }
        if self.model.family == "description" {
            self.budget
                .text(name.len().saturating_mul(4).saturating_add(128))
                .map_err(ParseFailure::terminal)?;
            return Err(ParseFailure::owned_syntax(
                relation.span.clone(),
                format!("undeclared description entity {name:?}"),
            ));
        }
        self.budget.item().map_err(ParseFailure::terminal)?;
        let id = self.model.entities.len();
        context::push(
            &mut self.model.entities,
            Entity {
                key: scoped,
                alias: name.into(),
                label: name.into(),
                kind: Kind::Class,
                parent: relation.scope,
                members: Vec::new(),
                container: false,
                fill: Fill::Default,
            },
            self.budget,
        )?;
        Ok(id)
    }
}
