//! Chen lexical ownership and undirected associations from the authorized MIT edition.
use super::{
    budget::{Budget, bound},
    parser::{context, error::ParseFailure},
};
#[cfg(test)]
use crate::RenderError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum AttributeOwner {
    Entity(usize),
    Relationship(usize),
    Attribute(usize),
}
impl AttributeOwner {
    pub fn id(self) -> usize {
        match self {
            Self::Entity(id) | Self::Relationship(id) | Self::Attribute(id) => id,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ChenNode {
    Entity,
    Relationship,
    Attribute(AttributeOwner),
}
#[derive(Debug)]
pub(super) struct Node {
    pub key: String,
    pub alias: String,
    pub display: String,
    pub kind: ChenNode,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Participation {
    Normal,
    Bold,
}
#[derive(Debug)]
pub(super) enum Link {
    Ownership {
        attribute: usize,
        owner: AttributeOwner,
    },
    Association {
        from: usize,
        to: usize,
        cardinality: String,
        participation: Participation,
    },
}
impl Link {
    pub fn endpoints(&self) -> (usize, usize) {
        match self {
            Self::Ownership { attribute, owner } => (owner.id(), *attribute),
            Self::Association { from, to, .. } => (*from, *to),
        }
    }
    pub fn cardinality(&self) -> &str {
        match self {
            Self::Association { cardinality, .. } => cardinality,
            _ => "",
        }
    }
}
#[derive(Debug)]
pub(super) struct Chen {
    pub nodes: Vec<Node>,
    pub links: Vec<Link>,
}
#[derive(Debug)]
pub(in crate::plantuml) struct Name {
    alias: String,
    display: String,
}

pub(in crate::plantuml) struct Builder<'b> {
    pub model: Chen,
    pub owners: Vec<AttributeOwner>,
    budget: &'b mut Budget,
    retained: usize,
}

impl<'b> Builder<'b> {
    pub fn new(budget: &'b mut Budget, retained: usize) -> Self {
        Self {
            model: Chen {
                nodes: Vec::new(),
                links: Vec::new(),
            },
            owners: Vec::new(),
            budget,
            retained,
        }
    }

    pub fn control(&self) -> crate::RenderControl {
        self.budget.control.clone()
    }

    pub fn into_model(self) -> Chen {
        self.model
    }

    pub fn finish(&mut self) -> Result<(), ParseFailure> {
        self.budget.charge(1).map_err(ParseFailure::terminal)?;
        if self.owners.is_empty() && !self.model.nodes.is_empty() {
            return Ok(());
        }
        Err(ParseFailure::syntax(
            0..0,
            "Chen requires entities and balanced owner braces",
        ))
    }

    pub fn single(
        &mut self,
        alias: String,
        span: std::ops::Range<usize>,
    ) -> Result<Name, ParseFailure> {
        validate_alias(&alias, span)?;
        self.budget
            .text(alias.len().saturating_mul(2))
            .map_err(ParseFailure::terminal)?;
        self.retain(alias.len())?;
        Ok(Name {
            display: alias.clone(),
            alias,
        })
    }

    pub fn named(
        &mut self,
        alias: String,
        display: String,
        span: std::ops::Range<usize>,
    ) -> Result<Name, ParseFailure> {
        validate_alias(&alias, span)?;
        bound("label bytes", display.len(), 16 * 1024).map_err(ParseFailure::terminal)?;
        self.budget
            .text(alias.len().saturating_add(display.len()))
            .map_err(ParseFailure::terminal)?;
        Ok(Name { alias, display })
    }

    pub fn open(
        &mut self,
        kind: ChenNode,
        name: Name,
        span: std::ops::Range<usize>,
    ) -> Result<(), ParseFailure> {
        self.budget.depth(1).map_err(ParseFailure::terminal)?;
        self.budget
            .text(name.alias.len())
            .map_err(ParseFailure::terminal)?;
        self.retain(name.alias.len())?;
        let key = name.alias.clone();
        let id = self.insert(
            Node {
                key,
                alias: name.alias,
                display: name.display,
                kind,
            },
            span,
        )?;
        let owner = match kind {
            ChenNode::Entity => AttributeOwner::Entity(id),
            _ => AttributeOwner::Relationship(id),
        };
        push(&mut self.owners, owner, (self.budget, &mut self.retained))
    }

    pub fn composite(
        &mut self,
        name: Name,
        span: std::ops::Range<usize>,
    ) -> Result<(), ParseFailure> {
        self.budget
            .depth(self.owners.len() + 1)
            .map_err(ParseFailure::terminal)?;
        let id = self.add_attribute(name, span)?;
        push(
            &mut self.owners,
            AttributeOwner::Attribute(id),
            (self.budget, &mut self.retained),
        )
    }

    pub fn attribute(
        &mut self,
        name: Name,
        span: std::ops::Range<usize>,
    ) -> Result<(), ParseFailure> {
        self.add_attribute(name, span).map(|_| ())
    }

    fn add_attribute(
        &mut self,
        name: Name,
        span: std::ops::Range<usize>,
    ) -> Result<usize, ParseFailure> {
        let owner = self
            .owners
            .last()
            .copied()
            .ok_or_else(|| ParseFailure::syntax(span.clone(), "Chen attribute has no owner"))?;
        let bytes = self.model.nodes[owner.id()]
            .key
            .len()
            .saturating_add(name.alias.len())
            .saturating_add(1);
        self.budget.text(bytes).map_err(ParseFailure::terminal)?;
        self.retain(bytes)?;
        let mut key = String::with_capacity(bytes);
        key.push_str(&self.model.nodes[owner.id()].key);
        key.push('/');
        key.push_str(&name.alias);
        self.budget.item().map_err(ParseFailure::terminal)?;
        let id = self.insert(
            Node {
                key,
                alias: name.alias,
                display: name.display,
                kind: ChenNode::Attribute(owner),
            },
            span,
        )?;
        push(
            &mut self.model.links,
            Link::Ownership {
                attribute: id,
                owner,
            },
            (self.budget, &mut self.retained),
        )?;
        Ok(id)
    }

    pub fn close(&mut self) -> Result<(), ParseFailure> {
        self.budget.charge(1).map_err(ParseFailure::terminal)?;
        self.owners
            .pop()
            .map(|_| ())
            .ok_or_else(|| ParseFailure::syntax(0..0, "Chen closing brace has no owner"))
    }

    fn insert(&mut self, node: Node, span: std::ops::Range<usize>) -> Result<usize, ParseFailure> {
        for existing in &self.model.nodes {
            self.budget
                .charge(1 + existing.key.len().min(node.key.len()))
                .map_err(ParseFailure::terminal)?;
            if existing.key == node.key {
                self.budget
                    .text(node.key.len().saturating_mul(4).saturating_add(32))
                    .map_err(ParseFailure::terminal)?;
                return Err(ParseFailure::owned_syntax(
                    span,
                    format!("duplicate Chen identity {:?}", node.key),
                ));
            }
        }
        self.budget.item().map_err(ParseFailure::terminal)?;
        self.budget
            .text(
                node.key
                    .len()
                    .saturating_add(node.alias.len())
                    .saturating_add(node.display.len()),
            )
            .map_err(ParseFailure::terminal)?;
        let id = self.model.nodes.len();
        push(
            &mut self.model.nodes,
            node,
            (self.budget, &mut self.retained),
        )?;
        Ok(id)
    }

    pub fn associate(
        &mut self,
        endpoints: (String, String),
        label: (String, Participation),
        span: std::ops::Range<usize>,
    ) -> Result<(), ParseFailure> {
        validate_alias(&endpoints.0, span.clone())?;
        validate_alias(&endpoints.1, span.clone())?;
        let from = self.resolve(&endpoints.0, span.clone())?;
        let to = self.resolve(&endpoints.1, span)?;
        self.budget.item().map_err(ParseFailure::terminal)?;
        self.budget
            .text(label.0.len())
            .map_err(ParseFailure::terminal)?;
        push(
            &mut self.model.links,
            Link::Association {
                from,
                to,
                cardinality: label.0,
                participation: label.1,
            },
            (self.budget, &mut self.retained),
        )
    }

    fn resolve(&mut self, name: &str, span: std::ops::Range<usize>) -> Result<usize, ParseFailure> {
        for (id, node) in self.model.nodes.iter().enumerate() {
            self.budget
                .charge(1 + node.alias.len().min(name.len()))
                .map_err(ParseFailure::terminal)?;
            if node.alias == name && !matches!(node.kind, ChenNode::Attribute(_)) {
                return Ok(id);
            }
        }
        self.budget
            .text(name.len().saturating_mul(4).saturating_add(32))
            .map_err(ParseFailure::terminal)?;
        Err(ParseFailure::owned_syntax(
            span,
            format!("undeclared Chen endpoint {name:?}"),
        ))
    }

    pub fn atom(
        &mut self,
        value: String,
        span: std::ops::Range<usize>,
    ) -> Result<String, ParseFailure> {
        self.budget
            .charge(value.len())
            .map_err(ParseFailure::terminal)?;
        if value.is_empty() || value.chars().any(|ch| !ch.is_alphanumeric()) {
            return Err(ParseFailure::unsupported(span, "Chen cardinality"));
        }
        Ok(value)
    }

    pub fn cardinality(
        &mut self,
        parts: (String, String, String),
        span: std::ops::Range<usize>,
    ) -> Result<String, ParseFailure> {
        let left = self.atom(parts.0, span.clone())?;
        let right = self.atom(parts.2, span)?;
        let bytes = left
            .len()
            .saturating_add(parts.1.len())
            .saturating_add(right.len())
            .saturating_add(3);
        bound("label bytes", bytes, 16 * 1024).map_err(ParseFailure::terminal)?;
        self.budget.charge(bytes).map_err(ParseFailure::terminal)?;
        self.retain(bytes)?;
        let mut value = String::with_capacity(bytes);
        value.push('(');
        value.push_str(&left);
        value.push(',');
        value.push_str(&parts.1);
        value.push_str(&right);
        value.push(')');
        Ok(value)
    }

    fn retain(&mut self, bytes: usize) -> Result<(), ParseFailure> {
        retain(&mut self.retained, bytes, self.budget)
    }
}

fn validate_alias(value: &str, span: std::ops::Range<usize>) -> Result<(), ParseFailure> {
    bound("Chen alias bytes", value.len(), 16 * 1024).map_err(ParseFailure::terminal)?;
    if value.is_empty()
        || value
            .chars()
            .any(|ch| !(ch.is_alphanumeric() || matches!(ch, '_' | '.')))
    {
        return Err(ParseFailure::unsupported(span, "Chen alias"));
    }
    Ok(())
}

fn retain(retained: &mut usize, bytes: usize, budget: &mut Budget) -> Result<(), ParseFailure> {
    let next = retained.saturating_add(bytes);
    bound(
        "Chen parser retained bytes",
        next,
        budget.limits.source_bytes.saturating_mul(256),
    )
    .map_err(ParseFailure::terminal)?;
    *retained = next;
    Ok(())
}

fn push<T>(
    values: &mut Vec<T>,
    value: T,
    accounting: (&mut Budget, &mut usize),
) -> Result<(), ParseFailure> {
    let (budget, retained) = accounting;
    let capacity = if values.len() == values.capacity() {
        values.capacity().saturating_mul(2).max(4)
    } else {
        values.capacity()
    };
    if values.len() == values.capacity() {
        retain(
            retained,
            capacity.saturating_mul(std::mem::size_of::<T>()),
            budget,
        )?;
    }
    context::push(values, value, budget)?;
    retain(
        retained,
        values
            .capacity()
            .saturating_sub(capacity)
            .saturating_mul(std::mem::size_of::<T>()),
        budget,
    )
}

#[cfg(test)]
pub(super) fn parse(source: &str, budget: &mut Budget) -> Result<Chen, RenderError> {
    let document = super::preprocess::prepare(source, budget)?;
    let mut parsed = super::parser::document::parse(&document, budget)?;
    super::parser::preamble::parse(&mut parsed, budget)?;
    super::parser::chen::parse(parsed.body, budget)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RenderOptions;

    fn lines(body: &str, _budget: &mut Budget) -> String {
        format!("@startchen\n{body}\n@endchen")
    }

    #[test]
    fn should_resolve_keyword_named_chen_endpoints() {
        // given
        let mut budget = Budget::new(&RenderOptions::default());
        let input = lines(
            "entity entity {\n}\nrelationship relationship {\n}\nentity -1- relationship\nrelationship =N= entity",
            &mut budget,
        );
        // when
        let model = parse(&input, &mut budget).unwrap();
        // then
        assert_eq!(
            model
                .nodes
                .iter()
                .map(|node| node.key.as_str())
                .collect::<Vec<_>>(),
            ["entity", "relationship"]
        );
        assert_eq!(
            model.links.iter().map(Link::endpoints).collect::<Vec<_>>(),
            [(0, 1), (1, 0)]
        );
    }

    #[test]
    fn should_keep_spaced_and_compact_keyword_composites_identical() {
        // given
        for attribute in ["entity {", "entity{"] {
            let mut budget = Budget::new(&RenderOptions::default());
            let input = lines(
                &format!("entity C {{\n{attribute}\nleaf\n}}\n}}"),
                &mut budget,
            );
            // when
            let model = parse(&input, &mut budget).unwrap();
            // then
            assert_eq!(
                model
                    .nodes
                    .iter()
                    .map(|node| node.key.as_str())
                    .collect::<Vec<_>>(),
                ["C", "C/entity", "C/entity/leaf"]
            );
            assert_eq!(
                model.nodes[2].kind,
                ChenNode::Attribute(AttributeOwner::Attribute(1))
            );
        }
    }

    #[test]
    fn should_keep_typed_owners_and_distinct_scoped_attribute_ids() {
        // given
        let mut budget = Budget::new(&RenderOptions::default());
        let input = lines(
            "entity \"Customer\" as C {\nname\naddress {\ncity\n}\n}\nrelationship R {\nname\n}\nC -(0, N)- R",
            &mut budget,
        );
        // when
        let model = parse(&input, &mut budget).unwrap();
        // then
        assert_eq!(
            model
                .nodes
                .iter()
                .map(|n| n.key.as_str())
                .collect::<Vec<_>>(),
            ["C", "C/name", "C/address", "C/address/city", "R", "R/name"]
        );
        assert_eq!(model.nodes[0].display, "Customer");
        assert_eq!(
            model.nodes[1].kind,
            ChenNode::Attribute(AttributeOwner::Entity(0))
        );
        assert_eq!(
            model.nodes[3].kind,
            ChenNode::Attribute(AttributeOwner::Attribute(2))
        );
        assert_eq!(
            model.nodes[5].kind,
            ChenNode::Attribute(AttributeOwner::Relationship(4))
        );
        assert_eq!(
            model.links.iter().map(Link::endpoints).collect::<Vec<_>>(),
            [(0, 1), (0, 2), (2, 3), (4, 5), (0, 4)]
        );
        assert_eq!(model.links[4].cardinality(), "(0, N)");
    }

    #[test]
    fn should_check_owner_depth_before_retaining_composite_node_or_link() {
        // given
        let mut options = RenderOptions::default();
        options.limits.nesting_depth = 1;
        let mut budget = Budget::new(&options);
        let mut parser = Builder::new(&mut budget, 0);
        let name = parser.single("C".into(), 0..1).unwrap();
        parser.open(ChenNode::Entity, name, 0..1).unwrap();
        let name = parser.single("address".into(), 2..9).unwrap();
        // when
        let error = parser.composite(name, 2..9).unwrap_err().into_line(3);
        // then
        assert!(error.to_string().contains("nesting depth"));
        assert_eq!(parser.model.nodes.len(), 1);
        assert!(parser.model.links.is_empty());
        assert_eq!(parser.owners, [AttributeOwner::Entity(0)]);
    }

    #[test]
    fn should_bound_expanded_owner_keys_before_deep_amplification() {
        // given
        let name = "x".repeat(1000);
        let body = format!(
            "entity C {{\n{}{}",
            format!("{name} {{\n").repeat(40),
            "}\n".repeat(41)
        );
        let mut options = RenderOptions::default();
        options.limits.source_bytes = body.len() + 64;
        let mut budget = Budget::new(&options);
        let input = lines(&body, &mut budget);
        // when
        let error = parse(&input, &mut budget).unwrap_err();
        // then
        assert!(
            error.to_string().contains("expanded model text bytes"),
            "{error}"
        );
    }

    #[test]
    fn should_reject_oversized_attribute_before_retaining_any_attribute() {
        // given
        let mut budget = Budget::new(&RenderOptions::default());
        let mut parser = Builder::new(&mut budget, 0);
        let name = parser.single("C".into(), 0..1).unwrap();
        parser.open(ChenNode::Entity, name, 0..1).unwrap();
        // when
        let error = parser
            .single("x".repeat(16385), 2..16387)
            .unwrap_err()
            .into_line(3);
        // then
        assert!(error.to_string().contains("16385 exceeds limit 16384"));
        assert_eq!(parser.model.nodes.len(), 1);
        assert!(parser.model.links.is_empty());
    }

    #[test]
    fn should_charge_tokens_before_scanning_the_next_token_or_retaining_nodes() {
        // given
        let mut options = RenderOptions::default();
        options.limits.model_items = 1;
        let mut budget = Budget::new(&options);
        // when
        let error = parse("@startchen\nentity \"unterminated\n@endchen", &mut budget).unwrap_err();
        // then
        assert!(
            error
                .to_string()
                .contains("model items value 2 exceeds limit 1"),
            "{error}"
        );
    }
}
