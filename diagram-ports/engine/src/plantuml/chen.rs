//! Chen lexical ownership and undirected associations from the authorized MIT edition.
use super::{
    budget::{Budget, bound, invalid, unsupported},
    lexer,
    source::Line,
    text::validate_label,
};
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
struct Parser {
    model: Chen,
    owners: Vec<AttributeOwner>,
}

pub(super) fn parse(lines: &[Line], budget: &mut Budget) -> Result<Chen, RenderError> {
    let mut parser = Parser {
        model: Chen {
            nodes: Vec::new(),
            links: Vec::new(),
        },
        owners: Vec::new(),
    };
    for line in lines {
        budget.charge(line.text.len() + 1)?;
        parser.line(line, budget)?;
    }
    if parser.owners.is_empty() && !parser.model.nodes.is_empty() {
        return Ok(parser.model);
    }
    Err(invalid(
        lines.last().map_or(1, |l| l.number),
        "Chen requires entities and balanced owner braces",
    ))
}

impl Parser {
    fn line(&mut self, line: &Line, budget: &mut Budget) -> Result<(), RenderError> {
        if line.text == "}" {
            self.owners
                .pop()
                .ok_or_else(|| invalid(line.number, "Chen closing brace has no owner"))?;
            return Ok(());
        }
        if matches!(
            line.text.split_whitespace().next(),
            Some("entity" | "relationship")
        ) && line.text.split_whitespace().nth(1).is_some()
        {
            if self.owners.is_empty() {
                return self.declare(line, budget);
            }
            return Err(unsupported(
                line.number,
                &format!("nested Chen declaration {:?}", line.text),
            ));
        }
        if self.owners.is_empty() {
            return self.associate(line, budget);
        }
        self.attribute(line, budget)
    }

    fn declare(&mut self, line: &Line, budget: &mut Budget) -> Result<(), RenderError> {
        let tokens = lexer::model_tokens(line, budget)?;
        if tokens.last().is_none_or(|t| t.text != "{" || t.quoted) {
            return Err(invalid(
                line.number,
                format!(
                    "Chen declaration requires opening owner brace: {:?}",
                    line.text
                ),
            ));
        }
        let (alias, display) = chen_name(&tokens[1..tokens.len() - 1], line.number, budget)?;
        let kind = if tokens[0].text == "entity" {
            ChenNode::Entity
        } else {
            ChenNode::Relationship
        };
        budget.depth(1)?;
        budget.text(alias.len())?;
        let id = self.insert((alias.clone(), alias, display, kind), line.number, budget)?;
        self.owners.push(match kind {
            ChenNode::Entity => AttributeOwner::Entity(id),
            _ => AttributeOwner::Relationship(id),
        });
        Ok(())
    }

    fn attribute(&mut self, line: &Line, budget: &mut Budget) -> Result<(), RenderError> {
        let tokens = lexer::model_tokens(line, budget)?;
        let composite = tokens.last().is_some_and(|t| t.text == "{" && !t.quoted);
        let (alias, display) = chen_name(
            &tokens[..tokens.len() - usize::from(composite)],
            line.number,
            budget,
        )?;
        if composite {
            budget.depth(self.owners.len() + 1)?;
        }
        let owner = *self.owners.last().unwrap();
        let parent = &self.model.nodes[owner.id()].key;
        budget.text(parent.len().saturating_add(alias.len()).saturating_add(1))?;
        let key = format!("{parent}/{alias}");
        budget.item()?;
        let id = self.insert(
            (key, alias, display, ChenNode::Attribute(owner)),
            line.number,
            budget,
        )?;
        self.model.links.push(Link::Ownership {
            attribute: id,
            owner,
        });
        if composite {
            self.owners.push(AttributeOwner::Attribute(id));
        }
        Ok(())
    }

    fn insert(
        &mut self,
        node: (String, String, String, ChenNode),
        line: usize,
        budget: &mut Budget,
    ) -> Result<usize, RenderError> {
        let (key, alias, display, kind) = node;
        for existing in &self.model.nodes {
            budget.charge(1 + existing.key.len().min(key.len()))?;
            if existing.key == key {
                return Err(invalid(line, format!("duplicate Chen identity {key:?}")));
            }
        }
        budget.item()?;
        budget.text(
            key.len()
                .saturating_add(alias.len())
                .saturating_add(display.len()),
        )?;
        let id = self.model.nodes.len();
        self.model.nodes.push(Node {
            key,
            alias,
            display,
            kind,
        });
        Ok(id)
    }

    fn associate(&mut self, line: &Line, budget: &mut Budget) -> Result<(), RenderError> {
        let text = line.text.as_str();
        let start = text
            .find(['-', '='])
            .ok_or_else(|| unsupported(line.number, text))?;
        let from = text[..start].trim();
        let marker = text.as_bytes()[start] as char;
        let rest = &text[start + 1..];
        let end = rest
            .find(['-', '='])
            .ok_or_else(|| invalid(line.number, format!("incomplete Chen association {text:?}")))?;
        if rest.as_bytes()[end] as char != marker {
            return Err(unsupported(
                line.number,
                &format!("mismatched Chen participation markers {text:?}"),
            ));
        }
        let cardinality = &rest[..end];
        let to = rest[end + 1..].trim();
        validate_alias(from, line.number)?;
        validate_alias(to, line.number)?;
        validate_cardinality(cardinality, line.number)?;
        let from = self.resolve(from, line.number, budget)?;
        let to = self.resolve(to, line.number, budget)?;
        budget.item()?;
        budget.text(cardinality.len())?;
        self.model.links.push(Link::Association {
            from,
            to,
            cardinality: cardinality.into(),
            participation: if marker == '=' {
                Participation::Bold
            } else {
                Participation::Normal
            },
        });
        Ok(())
    }

    fn resolve(&self, name: &str, line: usize, budget: &mut Budget) -> Result<usize, RenderError> {
        for (id, node) in self.model.nodes.iter().enumerate() {
            budget.charge(1 + node.alias.len().min(name.len()))?;
            if node.alias == name && !matches!(node.kind, ChenNode::Attribute(_)) {
                return Ok(id);
            }
        }
        Err(invalid(line, format!("undeclared Chen endpoint {name:?}")))
    }
}

fn chen_name(
    tokens: &[lexer::Token],
    line: usize,
    budget: &mut Budget,
) -> Result<(String, String), RenderError> {
    let (alias, display) = match tokens {
        [alias] if !alias.quoted => (&alias.text, &alias.text),
        [display, join, alias]
            if display.quoted && join.text == "as" && !join.quoted && !alias.quoted =>
        {
            (&alias.text, &display.text)
        }
        _ => {
            return Err(unsupported(
                line,
                &format!("Chen name/decorations {tokens:?}"),
            ));
        }
    };
    validate_alias(alias, line)?;
    validate_label(display, line)?;
    bound("label bytes", display.len(), 16 * 1024)?;
    budget.text(alias.len().saturating_add(display.len()))?;
    Ok((alias.clone(), display.clone()))
}

fn validate_alias(value: &str, line: usize) -> Result<(), RenderError> {
    bound("Chen alias bytes", value.len(), 16 * 1024)?;
    if value.is_empty()
        || value
            .chars()
            .any(|c| !(c.is_alphanumeric() || matches!(c, '_' | '.')))
    {
        return Err(unsupported(line, &format!("Chen alias {value:?}")));
    }
    Ok(())
}

fn validate_cardinality(value: &str, line: usize) -> Result<(), RenderError> {
    bound("label bytes", value.len(), 16 * 1024)?;
    let atom = |s: &str| !s.is_empty() && s.chars().all(char::is_alphanumeric);
    if value.is_empty() || atom(value) {
        return Ok(());
    }
    if let Some(pair) = value.strip_prefix('(').and_then(|v| v.strip_suffix(')'))
        && let Some((left, right)) = pair.split_once(',')
        && atom(left)
        && atom(right.trim_start())
    {
        return Ok(());
    }
    Err(unsupported(line, &format!("Chen cardinality {value:?}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RenderOptions;

    fn lines(body: &str, budget: &mut Budget) -> Vec<Line> {
        super::super::source::prepare_document(&format!("@startchen\n{body}\n@endchen"), budget)
            .unwrap()
            .1
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
        let input = lines("entity C {\naddress {\ncity\n}\n}", &mut budget);
        let mut parser = Parser {
            model: Chen {
                nodes: Vec::new(),
                links: Vec::new(),
            },
            owners: Vec::new(),
        };
        parser.line(&input[0], &mut budget).unwrap();
        // when
        let error = parser.line(&input[1], &mut budget).unwrap_err();
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
        let mut parser = Parser {
            model: Chen {
                nodes: Vec::new(),
                links: Vec::new(),
            },
            owners: Vec::new(),
        };
        parser
            .line(
                &Line {
                    number: 2,
                    text: "entity C {".into(),
                },
                &mut budget,
            )
            .unwrap();
        // when
        let error = parser
            .line(
                &Line {
                    number: 3,
                    text: "x".repeat(16385),
                },
                &mut budget,
            )
            .unwrap_err();
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
        let mut parser = Parser {
            model: Chen {
                nodes: Vec::new(),
                links: Vec::new(),
            },
            owners: Vec::new(),
        };
        // when
        let error = parser
            .line(
                &Line {
                    number: 2,
                    text: "entity \"unterminated".into(),
                },
                &mut budget,
            )
            .unwrap_err();
        // then
        assert!(
            error
                .to_string()
                .contains("model items value 2 exceeds limit 1"),
            "{error}"
        );
        assert!(parser.model.nodes.is_empty());
        assert!(parser.model.links.is_empty());
    }
}
