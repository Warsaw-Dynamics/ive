//! PlantUML MIT nwdiag model with typed global-server and membership construction.
//! Copyright 2009-2024 Arnaud Roques; exact source traces/notices in provenance.
use super::{
    budget::{Budget, bound},
    parser::{
        context,
        error::ParseFailure,
        nwdiag::{Field, Properties, PropertyKey},
    },
    style::Color,
};
use std::{collections::BTreeMap, ops::Range};
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

pub(super) struct Builder<'b> {
    pub budget: &'b mut Budget,
    model: NwDiagram,
    scopes: Vec<Scope>,
    servers: BTreeMap<String, ServerId>,
    networks: BTreeMap<String, NetworkId>,
    memberships: BTreeMap<(NetworkId, ServerId), usize>,
}
impl<'b> Builder<'b> {
    pub fn new(budget: &'b mut Budget) -> Self {
        Self {
            budget,
            model: NwDiagram {
                networks: Vec::new(),
                servers: Vec::new(),
                memberships: Vec::new(),
                groups: Vec::new(),
            },
            scopes: Vec::new(),
            servers: BTreeMap::new(),
            networks: BTreeMap::new(),
            memberships: BTreeMap::new(),
        }
    }
    pub fn retained_bytes(items: usize) -> usize {
        let vectors = std::mem::size_of::<Network>()
            + std::mem::size_of::<Server>()
            + std::mem::size_of::<Membership>()
            + std::mem::size_of::<Group>()
            + std::mem::size_of::<ServerId>();
        let tree = 12
            * (std::mem::size_of::<(String, ServerId)>()
                + std::mem::size_of::<((NetworkId, ServerId), usize)>()
                + 3 * std::mem::size_of::<usize>());
        items.saturating_mul(3 * vectors + tree) + 12 * std::mem::size_of::<Scope>()
    }
    pub fn tick(&mut self) -> Result<(), ParseFailure> {
        self.budget.charge(1).map_err(Into::into)
    }
    pub fn root(&mut self) -> Result<(), ParseFailure> {
        self.budget.depth(1)?;
        context::push(&mut self.scopes, Scope::Root, self.budget)
    }
    fn scope(&self) -> Scope {
        self.scopes.last().copied().unwrap_or(Scope::Root)
    }
    pub fn value(&mut self, value: String) -> Result<String, ParseFailure> {
        bound("label bytes", value.len(), 16 * 1024)?;
        self.budget.text(value.len())?;
        Ok(value)
    }
    pub fn network(&mut self, name: String, span: Range<usize>) -> Result<(), ParseFailure> {
        self.tick()?;
        self.budget.depth(self.scopes.len() + 1)?;
        if !matches!(self.scope(), Scope::Root) {
            return Err(ParseFailure::syntax(
                span,
                "networks cannot nest or open inside a group",
            ));
        }
        self.budget
            .charge(name.len() * lookup_steps(self.networks.len()))?;
        if self.networks.contains_key(&name) {
            return Err(ParseFailure::syntax(span, "duplicate network name"));
        }
        self.budget.item()?;
        self.budget.text(name.len() * 3)?;
        let id = NetworkId(self.model.networks.len());
        self.networks.insert(name.clone(), id);
        context::push(
            &mut self.model.networks,
            Network {
                id,
                name: name.clone(),
                display: name,
                address: String::new(),
                fill: Fill::Default,
                width: Width::Members,
            },
            self.budget,
        )?;
        context::push(&mut self.scopes, Scope::Network(id), self.budget)
    }
    pub fn group(&mut self, name: String, span: Range<usize>) -> Result<(), ParseFailure> {
        self.tick()?;
        self.budget.depth(self.scopes.len() + 1)?;
        let scope = self.scope();
        if scope.group().is_some() {
            return Err(ParseFailure::syntax(span, "groups cannot nest"));
        }
        if !name.chars().all(|ch| ch.is_alphanumeric() || ch == '_') {
            return Err(ParseFailure::syntax(
                span,
                "invalid nwdiag group identifier",
            ));
        }
        self.budget.item()?;
        self.budget.text(name.len())?;
        let id = GroupId(self.model.groups.len());
        context::push(
            &mut self.model.groups,
            Group {
                id,
                name,
                description: String::new(),
                fill: Fill::Default,
                members: Vec::new(),
            },
            self.budget,
        )?;
        let nested = scope
            .network()
            .map_or(Scope::Group(id), |net| Scope::NetworkGroup(net, id));
        context::push(&mut self.scopes, nested, self.budget)
    }
    pub fn close(&mut self, span: Range<usize>) -> Result<(), ParseFailure> {
        self.tick()?;
        if let Some(group) = self.scope().group()
            && self.model.groups[group.0].members.is_empty()
        {
            return Err(ParseFailure::syntax(span, "empty nwdiag group"));
        }
        self.scopes.pop();
        Ok(())
    }
    pub fn property(
        &mut self,
        value: (PropertyKey, String),
        span: Range<usize>,
    ) -> Result<(), ParseFailure> {
        self.budget.item()?;
        let (key, value) = value;
        let scope = self.scope();
        if let Some(id) = scope.group() {
            let group = &mut self.model.groups[id.0];
            match key {
                PropertyKey::Description => group.description = value,
                PropertyKey::Color => group.fill = parse_fill(&value, span)?,
                _ => {
                    return Err(ParseFailure::syntax(
                        span,
                        "unsupported nwdiag group property",
                    ));
                }
            }
        } else if let Some(id) = scope.network() {
            let network = &mut self.model.networks[id.0];
            match key {
                PropertyKey::Description => network.display = value,
                PropertyKey::Address => network.address = value,
                PropertyKey::Color => network.fill = parse_fill(&value, span)?,
                PropertyKey::Width if value == "full" => network.width = Width::Full,
                _ => {
                    return Err(ParseFailure::syntax(
                        span,
                        "unsupported nwdiag network property",
                    ));
                }
            }
        } else {
            return Err(ParseFailure::syntax(
                span,
                "nwdiag property outside a network/group",
            ));
        }
        Ok(())
    }
    pub fn server(
        &mut self,
        value: (String, Properties),
        span: Range<usize>,
    ) -> Result<(), ParseFailure> {
        let (name, props) = value;
        let scope = self.scope();
        let network = scope.network();
        let group = scope.group();
        self.budget
            .charge(name.len() * lookup_steps(self.servers.len()))?;
        let id = if let Some(id) = self.servers.get(&name) {
            *id
        } else {
            if network.is_none() {
                return Err(ParseFailure::syntax(
                    span,
                    "standalone/hidden-network server is unsupported; group references must name an existing server",
                ));
            }
            self.budget.item()?;
            self.budget.text(name.len() * 3)?;
            let id = ServerId(self.model.servers.len());
            self.servers.insert(name.clone(), id);
            context::push(
                &mut self.model.servers,
                Server {
                    id,
                    name: name.clone(),
                    display: name,
                    fill: Fill::Default,
                    group: GroupAssignment::Ungrouped,
                },
                self.budget,
            )?;
            id
        };
        if network.is_none() && group.is_none() {
            return Err(ParseFailure::syntax(
                span,
                "standalone server update is unsupported",
            ));
        }
        if let Some(group) = group {
            if self.model.servers[id.0].group != GroupAssignment::Ungrouped {
                return Err(ParseFailure::syntax(
                    span,
                    "server already belongs to a group",
                ));
            }
            self.budget.item()?;
            context::push(&mut self.model.groups[group.0].members, id, self.budget)?;
            self.model.servers[id.0].group = GroupAssignment::Group(group);
        }
        let address = match props.address {
            Field::Missing => String::new(),
            Field::Text(value) => value,
        };
        if let Some(network) = network {
            self.connect((network, id), address)?;
        } else if !address.is_empty() {
            return Err(ParseFailure::syntax(
                span,
                "interface address without network",
            ));
        }
        if let Field::Text(value) = props.description {
            self.model.servers[id.0].display = value;
        }
        if let Field::Text(value) = props.color {
            self.model.servers[id.0].fill = parse_fill(&value, span)?;
        }
        Ok(())
    }
    fn connect(
        &mut self,
        ends: (NetworkId, ServerId),
        address: String,
    ) -> Result<(), ParseFailure> {
        self.budget.charge(lookup_steps(self.memberships.len()))?;
        if let Some(index) = self.memberships.get(&ends) {
            if address.is_empty() {
                return Ok(());
            }
            self.model.memberships[*index].address = address;
        } else {
            self.budget.item()?;
            let index = self.model.memberships.len();
            context::push(
                &mut self.model.memberships,
                Membership {
                    network: ends.0,
                    server: ends.1,
                    address,
                },
                self.budget,
            )?;
            self.memberships.insert(ends, index);
        }
        Ok(())
    }
    pub fn finish(self, span: Range<usize>) -> Result<NwDiagram, ParseFailure> {
        if self.model.networks.is_empty() || self.model.servers.is_empty() {
            return Err(ParseFailure::syntax(
                span,
                "nwdiag requires an explicit network and a member server",
            ));
        }
        Ok(self.model)
    }
}
fn lookup_steps(size: usize) -> usize {
    4 * (size.checked_ilog2().unwrap_or(0) as usize + 1)
}
fn parse_fill(value: &str, span: Range<usize>) -> Result<Fill, ParseFailure> {
    let Some(hex) = value
        .strip_prefix('#')
        .filter(|hex| hex.len() == 6 && hex.bytes().all(|byte| byte.is_ascii_hexdigit()))
    else {
        return Err(ParseFailure::unsupported(
            span,
            "nwdiag color requires #RRGGBB",
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
        for (limit, servers) in [(1, 0), (2, 1)] {
            let mut options = RenderOptions::default();
            options.limits.model_items = limit;
            let mut budget = Budget::new(&options);
            let mut parser = Builder::new(&mut budget);
            parser.root().unwrap();
            parser.network("n".into(), 0..1).unwrap();
            // when
            let error = parser
                .server(("r".into(), Properties::default()), 2..3)
                .unwrap_err();
            // then
            assert!(format!("{error:?}").contains("model items"));
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
        // when
        let error =
            super::super::parser::nwdiag::tokenize_test("nwdiag { network n { a; } }", &mut budget)
                .err()
                .unwrap();
        // then
        assert!(
            format!("{error:?}").contains("model items value 3 exceeds limit 2"),
            "{error:?}"
        );
        let text = format!(
            "nwdiag {{ network n {{ a [description=\"{}\"]; }} }}",
            "x".repeat(16385)
        );
        let error = super::super::parser::nwdiag::tokenize_test(
            &text,
            &mut Budget::new(&RenderOptions::default()),
        )
        .err()
        .unwrap();
        assert!(
            format!("{error:?}").contains("label bytes value 16385"),
            "{error:?}"
        );
    }
    #[test]
    fn should_check_network_depth_before_retaining_the_block() {
        // given
        let mut options = RenderOptions::default();
        options.limits.nesting_depth = 1;
        let mut budget = Budget::new(&options);
        let mut parser = Builder::new(&mut budget);
        parser.root().unwrap();
        // when
        let error = parser.network("n".into(), 0..1).unwrap_err();
        // then
        assert!(format!("{error:?}").contains("nesting depth"));
        assert!(parser.model.networks.is_empty());
    }
}
