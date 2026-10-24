//! Ordered Graphviz DOT syntax before attribute inheritance or graph lowering.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphKind {
    Graph,
    Digraph,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdKind {
    Bare,
    Numeral,
    Quoted,
    Html,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Id {
    pub text: String,
    pub kind: IdKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    pub strict: bool,
    pub kind: GraphKind,
    pub id: Option<Id>,
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeId {
    pub id: Id,
    pub port: Vec<Id>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attribute {
    pub name: Id,
    pub value: Option<Id>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeScope {
    Graph,
    Node,
    Edge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeOp {
    Directed,
    Undirected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subgraph {
    pub id: Option<Id>,
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Endpoint {
    Node(NodeId),
    Nodes(Vec<NodeId>),
    Subgraph(Subgraph),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    Node {
        node: NodeId,
        attributes: Vec<Attribute>,
    },
    Nodes {
        nodes: Vec<NodeId>,
        attributes: Vec<Attribute>,
    },
    Edge {
        endpoints: Vec<Endpoint>,
        operators: Vec<EdgeOp>,
        attributes: Vec<Attribute>,
    },
    Attributes {
        scope: AttributeScope,
        attributes: Vec<Attribute>,
    },
    Assignment {
        name: Id,
        value: Id,
    },
    Subgraph(Subgraph),
}

pub(crate) fn statement_from_endpoint(
    head: Endpoint,
    chain: Vec<(EdgeOp, Endpoint)>,
    attributes: Vec<Attribute>,
    span: std::ops::Range<usize>,
) -> Result<Statement, super::DotError> {
    if chain.is_empty() {
        return match head {
            Endpoint::Node(node) => Ok(Statement::Node { node, attributes }),
            Endpoint::Nodes(nodes) => Ok(Statement::Nodes { nodes, attributes }),
            Endpoint::Subgraph(subgraph) if attributes.is_empty() => {
                Ok(Statement::Subgraph(subgraph))
            }
            Endpoint::Subgraph(_) => Err(super::DotError::new(
                span,
                "DOT subgraph statement cannot have trailing attributes",
            )),
        };
    }
    let mut endpoints = vec![head];
    let mut operators = Vec::with_capacity(chain.len());
    for (operator, endpoint) in chain {
        operators.push(operator);
        endpoints.push(endpoint);
    }
    Ok(Statement::Edge {
        endpoints,
        operators,
        attributes,
    })
}

pub(crate) fn concat_quoted(
    mut first: Id,
    tail: Vec<Id>,
    span: std::ops::Range<usize>,
) -> Result<Id, super::DotError> {
    if tail.is_empty() {
        return Ok(first);
    }
    if !matches!(first.kind, IdKind::Quoted | IdKind::Html)
        || tail
            .iter()
            .any(|id| !matches!(id.kind, IdKind::Quoted | IdKind::Html))
    {
        return Err(super::DotError::new(
            span,
            "DOT '+' concatenation requires quoted or HTML IDs",
        ));
    }
    for part in tail {
        first.text.push_str(&part.text);
    }
    first.kind = IdKind::Quoted;
    Ok(first)
}
