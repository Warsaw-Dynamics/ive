//! PlantUML MIT 1.2026.9: class/object/description command grammars and scoped entity semantics.
use super::{
    arrow::{self, Arrow},
    budget::{Budget, invalid, unsupported},
    lexer::{self, Token},
    scene::Decoration,
    source::Line,
    text::validate_label,
};
use crate::RenderError;

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
            Self::Note => "note",
        }
    }
    fn members(self) -> bool {
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
#[derive(Debug)]
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
    line: usize,
    from: String,
    to: String,
    arrow: Arrow,
    label: String,
    left: String,
    right: String,
}
enum Body {
    Outside,
    Members(usize),
}
struct Parser {
    model: Uml,
    groups: Vec<usize>,
    body: Body,
    pending: Vec<PendingRelation>,
}

pub(super) fn is_graph(lines: &[Line]) -> bool {
    for line in lines {
        let word = line.text.split_whitespace().next().unwrap_or("");
        if word == "note" {
            return line
                .text
                .strip_prefix("note ")
                .is_some_and(|rest| rest.starts_with('"'));
        }
        if matches!(
            word,
            "participant"
                | "activate"
                | "deactivate"
                | "group"
                | "alt"
                | "loop"
                | "opt"
                | "create"
                | "destroy"
        ) {
            return false;
        }
        if kind(word).is_some_and(|kind| !matches!(kind, Kind::Actor | Kind::Database))
            || word == "abstract"
            || word.starts_with('[')
            || word == "left"
            || word == "top"
        {
            return true;
        }
    }
    false
}

pub(super) fn parse(lines: &[Line], budget: &mut Budget) -> Result<Uml, RenderError> {
    let mut parser = Parser {
        model: Uml {
            family: "description",
            entities: Vec::new(),
            relations: Vec::new(),
            direction: Direction::Down,
        },
        groups: Vec::new(),
        body: Body::Outside,
        pending: Vec::new(),
    };
    for line in lines {
        budget.charge(1)?;
        parser.line(line, budget)?;
    }
    if !parser.groups.is_empty() || matches!(parser.body, Body::Members(_)) {
        return Err(invalid(
            lines.last().unwrap().number,
            "unclosed entity or package",
        ));
    }
    parser.resolve(budget)?;
    if parser.model.entities.is_empty() {
        return Err(invalid(1, "UML graph has no entities"));
    }
    Ok(parser.model)
}

fn kind(word: &str) -> Option<Kind> {
    Some(match word {
        "class" => Kind::Class,
        "interface" => Kind::Interface,
        "enum" => Kind::Enum,
        "annotation" => Kind::Annotation,
        "object" => Kind::Object,
        "package" => Kind::Package,
        "namespace" => Kind::Namespace,
        "component" => Kind::Component,
        "node" => Kind::Node,
        "database" => Kind::Database,
        "actor" => Kind::Actor,
        "usecase" => Kind::Usecase,
        "rectangle" => Kind::Rectangle,
        "note" => Kind::Note,
        _ => return None,
    })
}

impl Parser {
    fn scope(&self) -> &str {
        self.groups
            .last()
            .map_or("", |id| self.model.entities[*id].key.as_str())
    }
    fn line(&mut self, line: &Line, budget: &mut Budget) -> Result<(), RenderError> {
        if let Body::Members(id) = self.body {
            return self.member(id, line, budget);
        }
        if line.text == "}" {
            self.groups
                .pop()
                .ok_or_else(|| invalid(line.number, "closing brace without group"))?;
            return Ok(());
        }
        if line.text == "left to right direction" {
            self.model.direction = Direction::Right;
            return Ok(());
        }
        if line.text == "top to bottom direction" {
            self.model.direction = Direction::Down;
            return Ok(());
        }
        let (head, label) = lexer::split_label(line);
        let tokens = lexer::tokens(
            &Line {
                number: line.number,
                text: head.into(),
            },
            budget,
        )?;
        if let Some(first) = tokens.first()
            && (kind(&first.text).is_some() || first.text == "abstract")
        {
            if head != line.text {
                return Err(unsupported(line.number, "declaration colon suffix"));
            }
            return self.declare(line, &tokens, budget);
        }
        self.relation(line, &tokens, label, budget)
    }

    fn declare(
        &mut self,
        line: &Line,
        tokens: &[Token],
        budget: &mut Budget,
    ) -> Result<(), RenderError> {
        let (entity_kind, start) = if tokens[0].text == "abstract" {
            if tokens.get(1).is_none_or(|t| t.text != "class") {
                return Err(unsupported(line.number, &line.text));
            }
            (Kind::Abstract, 2)
        } else {
            (kind(&tokens[0].text).unwrap(), 1)
        };
        let open = tokens.last().is_some_and(|t| t.text == "{" && !t.quoted);
        let end = tokens.len() - usize::from(open);
        if entity_kind == Kind::Note
            && (open
                || tokens.len() != 4
                || !tokens[1].quoted
                || tokens[2].text != "as"
                || tokens[3].quoted)
        {
            return Err(unsupported(
                line.number,
                "graph notes require note \"text\" as alias",
            ));
        }
        let (alias, label) = lexer::name(&tokens[start..end], line.number)?;
        let is_uml = entity_kind.members();
        budget.charge(self.model.entities.len().saturating_mul(2))?;
        let is_description = matches!(
            entity_kind,
            Kind::Component
                | Kind::Node
                | Kind::Database
                | Kind::Actor
                | Kind::Usecase
                | Kind::Rectangle
        );
        if (is_uml
            && self.model.entities.iter().any(|e| {
                matches!(
                    e.kind,
                    Kind::Component
                        | Kind::Node
                        | Kind::Database
                        | Kind::Actor
                        | Kind::Usecase
                        | Kind::Rectangle
                )
            }))
            || (is_description && self.model.entities.iter().any(|e| e.kind.members()))
        {
            return Err(unsupported(
                line.number,
                "mixed class/description declarations require allowmixing",
            ));
        }
        validate_label(&label, line.number)?;
        if alias.contains("::") || alias.contains(['<', '>', '#', '$']) {
            return Err(unsupported(line.number, &alias));
        }
        let scope = self.scope();
        budget.text(scope.len() + alias.len() + 1)?;
        let key = if scope.is_empty() {
            alias.clone()
        } else {
            format!("{scope}.{alias}")
        };
        if self.model.entities.iter().any(|e| e.key == key) {
            return Err(invalid(
                line.number,
                format!("duplicate entity alias {key:?}"),
            ));
        }
        budget.item()?;
        let id = self.model.entities.len();
        let container = open && !entity_kind.members();
        let parent = self
            .groups
            .last()
            .map_or(Parent::Root, |id| Parent::Node(*id));
        self.model.entities.push(Entity {
            key,
            alias,
            label,
            kind: entity_kind,
            parent,
            members: Vec::new(),
            container,
        });
        if entity_kind.members() {
            self.model.family = if entity_kind == Kind::Object && self.model.family != "class" {
                "object"
            } else {
                "class"
            };
        }
        if open {
            budget.depth(self.groups.len() + 1)?;
            if entity_kind.members() {
                self.body = Body::Members(id);
            } else {
                self.groups.push(id);
            }
        }
        Ok(())
    }

    fn member(&mut self, id: usize, line: &Line, budget: &mut Budget) -> Result<(), RenderError> {
        if line.text == "}" {
            self.body = Body::Outside;
            return Ok(());
        }
        let plain = line
            .text
            .strip_prefix("{static}")
            .or_else(|| line.text.strip_prefix("{abstract}"))
            .unwrap_or(&line.text);
        if plain.contains(['{', '}']) {
            return Err(unsupported(line.number, &line.text));
        }
        if !matches!(line.text.as_str(), "--" | ".." | "==" | "__") {
            validate_label(&line.text, line.number)?;
        }
        budget.item()?;
        let member = if matches!(line.text.as_str(), "--" | ".." | "==" | "__") {
            Member::Separator
        } else if let Some(text) = line.text.strip_prefix("{static}") {
            Member::Text(text.trim().into(), Decoration::Underlined)
        } else if let Some(text) = line.text.strip_prefix("{abstract}") {
            Member::Text(text.trim().into(), Decoration::Italic)
        } else {
            Member::Text(line.text.clone(), Decoration::Plain)
        };
        self.model.entities[id].members.push(member);
        Ok(())
    }

    fn relation(
        &mut self,
        line: &Line,
        tokens: &[Token],
        label: &str,
        budget: &mut Budget,
    ) -> Result<(), RenderError> {
        let index = tokens
            .iter()
            .position(|t| !t.quoted && lexer::arrow_at(&t.text).is_some())
            .ok_or_else(|| unsupported(line.number, &line.text))?;
        let (from, left) = endpoint(&tokens[..index], line.number)?;
        let (to, right) = endpoint_right(&tokens[index + 1..], line.number)?;
        let arrow = arrow::parse(&tokens[index].text, line.number)?;
        for value in [label, &left, &right] {
            validate_label(value, line.number)?;
        }
        budget.item()?;
        self.pending.push(PendingRelation {
            scope: self
                .groups
                .last()
                .map_or(Parent::Root, |id| Parent::Node(*id)),
            line: line.number,
            from,
            to,
            arrow,
            label: label.into(),
            left,
            right,
        });
        Ok(())
    }

    fn resolve(&mut self, budget: &mut Budget) -> Result<(), RenderError> {
        for relation in std::mem::take(&mut self.pending) {
            budget.charge(self.model.entities.len().saturating_mul(6))?;
            let from = self.resolve_name(&relation.from, &relation, budget)?;
            let to = self.resolve_name(&relation.to, &relation, budget)?;
            if self.model.entities[from].container || self.model.entities[to].container {
                return Err(unsupported(
                    relation.line,
                    "relations to container boundaries",
                ));
            }
            self.model.relations.push(Relation {
                from,
                to,
                arrow: relation.arrow,
                label: relation.label,
                left: relation.left,
                right: relation.right,
            });
        }
        Ok(())
    }

    fn resolve_name(
        &mut self,
        name: &str,
        relation: &PendingRelation,
        budget: &mut Budget,
    ) -> Result<usize, RenderError> {
        let scope = match relation.scope {
            Parent::Root => "",
            Parent::Node(id) => self.model.entities[id].key.as_str(),
        };
        budget.text(scope.len() + name.len() + 1)?;
        let scoped = if scope.is_empty() {
            name.into()
        } else {
            format!("{scope}.{name}")
        };
        if let Some(id) = self.model.entities.iter().position(|e| e.key == scoped) {
            return Ok(id);
        }
        if let Some(id) = self.model.entities.iter().position(|e| e.key == name) {
            return Ok(id);
        }
        let matches: Vec<_> = self
            .model
            .entities
            .iter()
            .enumerate()
            .filter(|(_, e)| e.alias == name)
            .map(|(i, _)| i)
            .collect();
        if matches.len() == 1 {
            return Ok(matches[0]);
        }
        if matches.len() > 1 {
            return Err(invalid(
                relation.line,
                format!("ambiguous entity {name:?}; use its namespace"),
            ));
        }
        if self.model.family == "description" {
            return Err(invalid(
                relation.line,
                format!("undeclared description entity {name:?}"),
            ));
        }
        budget.item()?;
        let id = self.model.entities.len();
        self.model.entities.push(Entity {
            key: scoped,
            alias: name.into(),
            label: name.into(),
            kind: Kind::Class,
            parent: relation.scope,
            members: Vec::new(),
            container: false,
        });
        Ok(id)
    }
}

fn endpoint(tokens: &[Token], line: usize) -> Result<(String, String), RenderError> {
    if let Some(token) = tokens.first() {
        lexer::validate_name(token, line)?;
    }
    match tokens {
        [name] => Ok((name.text.clone(), String::new())),
        [name, multiplicity] if multiplicity.quoted => {
            Ok((name.text.clone(), multiplicity.text.clone()))
        }
        _ => Err(unsupported(
            line,
            "relation endpoint, port or direction constraint",
        )),
    }
}
fn endpoint_right(tokens: &[Token], line: usize) -> Result<(String, String), RenderError> {
    match tokens {
        [multiplicity, name] if multiplicity.quoted => {
            lexer::validate_name(name, line)?;
            Ok((name.text.clone(), multiplicity.text.clone()))
        }
        _ => endpoint(tokens, line),
    }
}
