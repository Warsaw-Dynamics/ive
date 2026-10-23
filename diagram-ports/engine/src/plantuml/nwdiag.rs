//! PlantUML MIT nwdiag grammar, global servers and per-network addresses.
//! Copyright 2009-2024 Arnaud Roques; exact source traces/notices in provenance.
use super::{
    budget::{Budget, bound, invalid, unsupported},
    source::Line,
    style::Color,
};
use crate::RenderError;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct NetworkId(pub usize);
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct ServerId(pub usize);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct GroupId(pub usize);
#[derive(Clone, Copy, Debug)]
pub(super) enum Fill {
    Default,
    Color(Color),
}
#[derive(Clone, Copy, Debug)]
pub(super) enum Width {
    Members,
    Full,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum GroupAssignment {
    Ungrouped,
    Group(GroupId),
}
pub(super) struct Network {
    pub id: NetworkId,
    pub name: String,
    pub display: String,
    pub address: String,
    pub fill: Fill,
    pub width: Width,
}
pub(super) struct Server {
    pub id: ServerId,
    pub name: String,
    pub display: String,
    pub fill: Fill,
    pub group: GroupAssignment,
}
pub(super) struct Membership {
    pub network: NetworkId,
    pub server: ServerId,
    pub address: String,
}
pub(super) struct Group {
    pub id: GroupId,
    pub name: String,
    pub description: String,
    pub fill: Fill,
    pub members: Vec<ServerId>,
}
pub(super) struct NwDiagram {
    pub networks: Vec<Network>,
    pub servers: Vec<Server>,
    pub memberships: Vec<Membership>,
    pub groups: Vec<Group>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind<'a> {
    Atom(&'a str),
    Quoted(&'a str),
    Symbol(char),
    End,
}
#[derive(Clone, Copy)]
struct Token<'a> {
    kind: Kind<'a>,
    line: usize,
}
struct Parser<'a, 'b> {
    tokens: Vec<Token<'a>>,
    cursor: usize,
    budget: &'b mut Budget,
    model: NwDiagram,
    servers: BTreeMap<String, ServerId>,
    networks: BTreeMap<String, NetworkId>,
    memberships: BTreeMap<(NetworkId, ServerId), usize>,
}
#[derive(Clone, Copy)]
enum Scope {
    Root,
    Network(NetworkId),
    Group(GroupId),
    NetworkGroup(NetworkId, GroupId),
}
impl Scope {
    fn network(self) -> Option<NetworkId> {
        match self {
            Self::Network(id) | Self::NetworkGroup(id, _) => Some(id),
            _ => None,
        }
    }
    fn group(self) -> Option<GroupId> {
        match self {
            Self::Group(id) | Self::NetworkGroup(_, id) => Some(id),
            _ => None,
        }
    }
}

pub(super) fn parse(lines: &[Line], budget: &mut Budget) -> Result<NwDiagram, RenderError> {
    let mut parser = Parser {
        tokens: tokenize(lines, budget)?,
        cursor: 0,
        budget,
        model: NwDiagram {
            networks: Vec::new(),
            servers: Vec::new(),
            memberships: Vec::new(),
            groups: Vec::new(),
        },
        servers: BTreeMap::new(),
        networks: BTreeMap::new(),
        memberships: BTreeMap::new(),
    };
    parser.skip_ends();
    parser.expect(Kind::Atom("nwdiag"))?;
    parser.expect(Kind::Symbol('{'))?;
    parser.block(Scope::Root, 1)?;
    parser.skip_ends();
    if parser.cursor != parser.tokens.len() {
        return Err(parser.error("unexpected text after nwdiag block"));
    }
    if parser.model.networks.is_empty() || parser.model.servers.is_empty() {
        return Err(parser.error("nwdiag requires an explicit network and a member server"));
    }
    Ok(parser.model)
}

fn tokenize<'a>(lines: &'a [Line], budget: &mut Budget) -> Result<Vec<Token<'a>>, RenderError> {
    let mut tokens = Vec::new();
    for line in lines {
        budget.charge(line.text.len() * 2 + 1)?;
        if super::parser::preamble::has_marker(&line.text) {
            return Err(unsupported(
                line.number,
                &format!(
                    "nwdiag styles/chrome must precede the body: {:?}",
                    line.text
                ),
            ));
        }
        let mut cursor = 0;
        while cursor < line.text.len() {
            let rest = &line.text[cursor..];
            let ch = rest.chars().next().unwrap();
            if ch.is_whitespace() {
                cursor += ch.len_utf8();
                continue;
            }
            budget.item()?;
            let kind = if ch == '"' {
                let end = rest[1..].find('"').ok_or_else(|| {
                    invalid(line.number, format!("unclosed nwdiag quote {rest:?}"))
                })? + 1;
                let value = &rest[1..end];
                bound("label bytes", value.len(), 16 * 1024)?;
                cursor += end + 1;
                Kind::Quoted(value)
            } else if "{}[],=;".contains(ch) {
                cursor += 1;
                if ch == ';' {
                    Kind::End
                } else {
                    Kind::Symbol(ch)
                }
            } else {
                let end = rest
                    .find(|c: char| c.is_whitespace() || "{}[],=;\"".contains(c))
                    .unwrap_or(rest.len());
                cursor += end;
                Kind::Atom(&rest[..end])
            };
            tokens.push(Token {
                kind,
                line: line.number,
            });
        }
        budget.item()?;
        tokens.push(Token {
            kind: Kind::End,
            line: line.number,
        });
    }
    Ok(tokens)
}

impl<'a> Parser<'a, '_> {
    fn peek(&self) -> Option<Kind<'a>> {
        self.tokens.get(self.cursor).map(|t| t.kind)
    }
    fn error(&self, message: &str) -> RenderError {
        let token = self.tokens.get(self.cursor).or_else(|| self.tokens.last());
        invalid(
            token.map_or(1, |t| t.line),
            format!("{message}; found {:?}", token.map(|t| t.kind)),
        )
    }
    fn expect(&mut self, kind: Kind<'a>) -> Result<(), RenderError> {
        self.budget.charge(1)?;
        if self.peek() != Some(kind) {
            return Err(self.error(&format!("expected {kind:?}")));
        }
        self.cursor += 1;
        Ok(())
    }
    fn skip_ends(&mut self) {
        while self.peek() == Some(Kind::End) {
            self.cursor += 1;
        }
    }
    fn atom(&mut self) -> Result<&'a str, RenderError> {
        let Some(Kind::Atom(value)) = self.peek() else {
            return Err(self.error("expected nwdiag name"));
        };
        self.cursor += 1;
        Ok(value)
    }
    fn name(&mut self) -> Result<&'a str, RenderError> {
        let name = self.atom()?;
        if name
            .chars()
            .all(|c| c.is_alphanumeric() || "-._".contains(c))
        {
            return Ok(name);
        }
        Err(self.error(&format!("invalid nwdiag identifier {name:?}")))
    }
    fn value(&mut self) -> Result<String, RenderError> {
        let value = match self.peek() {
            Some(Kind::Atom(value) | Kind::Quoted(value)) => value,
            _ => return Err(self.error("expected nwdiag property value")),
        };
        bound("label bytes", value.len(), 16 * 1024)?;
        self.budget.text(value.len())?;
        self.cursor += 1;
        Ok(value.to_owned())
    }
    fn terminate(&mut self) -> Result<(), RenderError> {
        match self.peek() {
            Some(Kind::End | Kind::Symbol('}')) => Ok(()),
            _ => Err(self.error("expected nwdiag statement terminator")),
        }
    }
    fn block(&mut self, scope: Scope, depth: usize) -> Result<(), RenderError> {
        self.budget.depth(depth)?;
        loop {
            self.budget.charge(1)?;
            self.skip_ends();
            match self.peek() {
                Some(Kind::Symbol('}')) => {
                    self.cursor += 1;
                    return Ok(());
                }
                Some(Kind::Atom("network")) if self.opens_block() => {
                    self.open_network(scope, depth)?
                }
                Some(Kind::Atom("group")) if self.opens_block() => self.open_group(scope, depth)?,
                Some(Kind::Atom(_)) => self.statement(scope)?,
                _ => return Err(self.error("expected nwdiag statement or closing brace")),
            }
        }
    }
    fn opens_block(&self) -> bool {
        matches!(
            self.tokens.get(self.cursor + 1).map(|t| t.kind),
            Some(Kind::Atom(_))
        )
    }
    fn open_network(&mut self, scope: Scope, depth: usize) -> Result<(), RenderError> {
        if matches!(scope, Scope::Root) {
        } else {
            return Err(self.error("networks cannot nest or open inside a group"));
        }
        self.cursor += 1;
        let name = self.name()?;
        self.budget
            .charge(name.len() * lookup_steps(self.networks.len()))?;
        if self.networks.contains_key(name) {
            return Err(self.error(&format!("duplicate network name {name:?}")));
        }
        self.expect(Kind::Symbol('{'))?;
        self.budget.depth(depth + 1)?;
        self.budget.item()?;
        self.budget.text(name.len() * 3)?;
        let id = NetworkId(self.model.networks.len());
        self.model.networks.push(Network {
            id,
            name: name.into(),
            display: name.into(),
            address: String::new(),
            fill: Fill::Default,
            width: Width::Members,
        });
        self.networks.insert(name.into(), id);
        self.block(Scope::Network(id), depth + 1)
    }
    fn open_group(&mut self, scope: Scope, depth: usize) -> Result<(), RenderError> {
        if scope.group().is_some() {
            return Err(self.error("groups cannot nest"));
        }
        self.cursor += 1;
        let name = self.name()?;
        if name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        } else {
            return Err(self.error(&format!("invalid nwdiag group identifier {name:?}")));
        }
        self.expect(Kind::Symbol('{'))?;
        self.budget.depth(depth + 1)?;
        self.budget.item()?;
        self.budget.text(name.len())?;
        let id = GroupId(self.model.groups.len());
        self.model.groups.push(Group {
            id,
            name: name.into(),
            description: String::new(),
            fill: Fill::Default,
            members: Vec::new(),
        });
        let nested = scope
            .network()
            .map_or(Scope::Group(id), |net| Scope::NetworkGroup(net, id));
        self.block(nested, depth + 1)?;
        if self.model.groups[id.0].members.is_empty() {
            return Err(self.error(&format!("empty nwdiag group {name:?}")));
        }
        Ok(())
    }
    fn statement(&mut self, scope: Scope) -> Result<(), RenderError> {
        let name = self.name()?;
        if self.peek() == Some(Kind::Symbol('=')) {
            self.cursor += 1;
            let value = self.value()?;
            self.terminate()?;
            return self.property(scope, name, value);
        }
        let props = self.properties()?;
        self.terminate()?;
        self.add_server(scope, name, props)
    }
    fn properties(&mut self) -> Result<BTreeMap<&'a str, String>, RenderError> {
        let mut result = BTreeMap::new();
        if self.peek() != Some(Kind::Symbol('[')) {
            return Ok(result);
        }
        self.cursor += 1;
        if self.peek() == Some(Kind::Symbol(']')) {
            self.cursor += 1;
            return Ok(result);
        }
        loop {
            let key = self.atom()?;
            if !matches!(key, "address" | "description" | "color") {
                return Err(self.error(&format!("unsupported nwdiag server property {key:?}")));
            }
            if result.contains_key(key) {
                return Err(self.error(&format!("duplicate nwdiag server property {key:?}")));
            }
            self.expect(Kind::Symbol('='))?;
            let value = self.value()?;
            self.budget.item()?;
            result.insert(key, value);
            match self.peek() {
                Some(Kind::Symbol(']')) => {
                    self.cursor += 1;
                    return Ok(result);
                }
                Some(Kind::Symbol(',')) => self.cursor += 1,
                _ => return Err(self.error("expected comma or closing property bracket")),
            }
        }
    }
    fn property(&mut self, scope: Scope, key: &str, value: String) -> Result<(), RenderError> {
        self.budget.item()?;
        let line = self.tokens[self.cursor.saturating_sub(1)].line;
        if let Some(id) = scope.group() {
            let group = &mut self.model.groups[id.0];
            match key {
                "description" => group.description = value,
                "color" => group.fill = parse_fill(&value, line)?,
                _ => {
                    return Err(self.error(&format!(
                        "unsupported nwdiag group property {key:?}={value:?}"
                    )));
                }
            }
        } else if let Some(id) = scope.network() {
            let network = &mut self.model.networks[id.0];
            match key {
                "description" => network.display = value,
                "address" => network.address = value,
                "color" => network.fill = parse_fill(&value, line)?,
                "width" if value == "full" => network.width = Width::Full,
                _ => {
                    return Err(self.error(&format!(
                        "unsupported nwdiag network property {key:?}={value:?}"
                    )));
                }
            }
        } else {
            return Err(self.error(&format!(
                "nwdiag property outside a network/group: {key:?}={value:?}"
            )));
        }
        Ok(())
    }
    fn add_server(
        &mut self,
        scope: Scope,
        name: &str,
        mut props: BTreeMap<&str, String>,
    ) -> Result<(), RenderError> {
        let network = scope.network();
        let group = scope.group();
        self.budget
            .charge(name.len() * lookup_steps(self.servers.len()))?;
        let id = if let Some(id) = self.servers.get(name) {
            *id
        } else {
            if network.is_none() {
                return Err(self.error(&format!("standalone/hidden-network server {name:?} is unsupported; group references must name an existing server")));
            }
            self.budget.item()?;
            self.budget.text(name.len() * 3)?;
            let id = ServerId(self.model.servers.len());
            self.model.servers.push(Server {
                id,
                name: name.into(),
                display: name.into(),
                fill: Fill::Default,
                group: GroupAssignment::Ungrouped,
            });
            self.servers.insert(name.into(), id);
            id
        };
        if network.is_none() && group.is_none() {
            return Err(self.error(&format!("standalone server update {name:?} is unsupported")));
        }
        if let Some(group) = group {
            if self.model.servers[id.0].group != GroupAssignment::Ungrouped {
                return Err(self.error(&format!("server {name:?} already belongs to a group")));
            }
            self.budget.item()?;
            self.model.servers[id.0].group = GroupAssignment::Group(group);
            self.model.groups[group.0].members.push(id);
        }
        let address = props.remove("address").unwrap_or_default();
        if let Some(network) = network {
            self.connect(network, id, address)?;
        } else if address.is_empty() {
        } else {
            return Err(self.error(&format!(
                "interface address without network for server {name:?}"
            )));
        }
        if let Some(value) = props.remove("description") {
            self.model.servers[id.0].display = value;
        }
        if let Some(value) = props.remove("color") {
            self.model.servers[id.0].fill =
                parse_fill(&value, self.tokens[self.cursor.saturating_sub(1)].line)?;
        }
        Ok(())
    }
    fn connect(
        &mut self,
        network: NetworkId,
        server: ServerId,
        address: String,
    ) -> Result<(), RenderError> {
        self.budget.charge(lookup_steps(self.memberships.len()))?;
        if let Some(index) = self.memberships.get(&(network, server)) {
            if address.is_empty() {
                return Ok(());
            }
            self.model.memberships[*index].address = address;
        } else {
            self.budget.item()?;
            let index = self.model.memberships.len();
            self.model.memberships.push(Membership {
                network,
                server,
                address,
            });
            self.memberships.insert((network, server), index);
        }
        Ok(())
    }
}

fn lookup_steps(size: usize) -> usize {
    4 * (size.checked_ilog2().unwrap_or(0) as usize + 1)
}
fn parse_fill(value: &str, line: usize) -> Result<Fill, RenderError> {
    let Some(hex) = value
        .strip_prefix('#')
        .filter(|hex| hex.len() == 6 && hex.bytes().all(|b| b.is_ascii_hexdigit()))
    else {
        return Err(unsupported(
            line,
            &format!("nwdiag color {value:?}; expected #RRGGBB"),
        ));
    };
    Ok(Fill::Color(Color(u32::from_str_radix(hex, 16).unwrap())))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RenderOptions;

    #[test]
    fn should_bound_servers_and_memberships_before_retaining_them() {
        // given
        let lines = [Line {
            number: 1,
            text: "nwdiag { network n { r; } }".into(),
        }];
        for (limit, servers) in [(1, 0), (2, 1)] {
            let mut options = RenderOptions::default();
            let tokens = tokenize(&lines, &mut Budget::new(&options)).unwrap();
            options.limits.model_items = limit;
            let mut budget = Budget::new(&options);
            let mut parser = Parser {
                tokens,
                cursor: 2,
                budget: &mut budget,
                model: NwDiagram {
                    networks: Vec::new(),
                    servers: Vec::new(),
                    memberships: Vec::new(),
                    groups: Vec::new(),
                },
                servers: BTreeMap::new(),
                networks: BTreeMap::new(),
                memberships: BTreeMap::new(),
            };
            // when
            let error = parser.block(Scope::Root, 1).unwrap_err().to_string();
            // then
            assert!(error.contains("model items"));
            assert_eq!(parser.model.networks.len(), 1);
            assert_eq!(parser.model.servers.len(), servers);
            assert!(parser.model.memberships.is_empty());
        }
    }
    #[test]
    fn should_bound_token_retention_and_property_text_before_allocating_values() {
        // given
        let mut options = RenderOptions::default();
        options.limits.model_items = 2;
        let mut budget = Budget::new(&options);
        let lines = [Line {
            number: 4,
            text: "nwdiag { network n { a; } }".into(),
        }];
        // when
        let error = tokenize(&lines, &mut budget).err().unwrap().to_string();
        // then
        assert!(
            error.contains("model items value 3 exceeds limit 2"),
            "{error}"
        );
        let lines = [Line {
            number: 5,
            text: format!(
                "nwdiag {{ network n {{ a [description=\"{}\"]; }} }}",
                "x".repeat(16385)
            ),
        }];
        let error = tokenize(&lines, &mut Budget::new(&RenderOptions::default()))
            .err()
            .unwrap()
            .to_string();
        assert!(error.contains("label bytes value 16385"), "{error}");
    }

    #[test]
    fn should_check_network_depth_before_retaining_the_block() {
        // given
        let lines = [Line {
            number: 1,
            text: "nwdiag { network n { r; } }".into(),
        }];
        let mut options = RenderOptions::default();
        let tokens = tokenize(&lines, &mut Budget::new(&options)).unwrap();
        options.limits.nesting_depth = 1;
        let mut budget = Budget::new(&options);
        let mut parser = Parser {
            tokens,
            cursor: 2,
            budget: &mut budget,
            model: NwDiagram {
                networks: Vec::new(),
                servers: Vec::new(),
                memberships: Vec::new(),
                groups: Vec::new(),
            },
            servers: BTreeMap::new(),
            networks: BTreeMap::new(),
            memberships: BTreeMap::new(),
        };
        // when
        let error = parser.block(Scope::Root, 1).unwrap_err().to_string();
        // then
        assert!(error.contains("nesting depth"));
        assert!(parser.model.networks.is_empty());
    }
}
