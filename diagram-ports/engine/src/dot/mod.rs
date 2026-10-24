//! Bounded Graphviz DOT syntax parser. Layout and attribute semantics are separate.

mod ast;
mod lexer;
#[rustfmt::skip]
#[allow(clippy::all, unused, private_interfaces)]
mod generated;

pub use ast::{
    Attribute, AttributeScope, Document, EdgeOp, Endpoint, GraphKind, Id, IdKind, NodeId,
    Statement, Subgraph,
};
pub(crate) use lexer::Token;

use crate::RenderControl;
use lalrpop_util::ParseError;
use std::{cell::Cell, fmt, ops::Range};

#[derive(Debug, Clone)]
pub struct ParseOptions {
    pub source_bytes: usize,
    pub tokens: usize,
    pub statements: usize,
    pub nesting_depth: usize,
    pub control: RenderControl,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            source_bytes: 256 * 1024,
            tokens: 100_000,
            statements: 10_000,
            nesting_depth: 64,
            control: RenderControl::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DotError {
    pub span: Range<usize>,
    pub message: String,
}

impl DotError {
    pub(crate) fn new(span: Range<usize>, message: impl Into<String>) -> Self {
        Self {
            span,
            message: message.into(),
        }
    }
}

impl fmt::Display for DotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "DOT at bytes {}..{}: {}",
            self.span.start, self.span.end, self.message
        )
    }
}

impl std::error::Error for DotError {}

/// Parses bounded DOT syntax while preserving statement and attribute order.
pub fn parse(source: &str, options: &ParseOptions) -> Result<Document, DotError> {
    if source.len() > options.source_bytes || options.source_bytes == 0 {
        return Err(DotError::new(
            0..source.len(),
            format!(
                "DOT source has {} bytes, exceeding limit {}",
                source.len(),
                options.source_bytes
            ),
        ));
    }
    if options.tokens == 0 || options.statements == 0 || options.nesting_depth == 0 {
        return Err(DotError::new(
            0..0,
            "DOT parse options have a zero token, statement, or nesting limit",
        ));
    }
    let tokens = lexer::lex(source, options)?;
    let stream = tokens.into_iter().map(|(start, token, end)| {
        options
            .control
            .checkpoint()
            .map_err(|error| {
                DotError::new(start..end, format!("DOT parsing was cancelled: {error}"))
            })
            .map(|_| (start, token, end))
    });
    let statement_count = Cell::new(0);
    let document = generated::DocumentParser::new()
        .parse(options, &statement_count, stream)
        .map_err(parse_error)?;
    check_statements(&document.statements, options, source.len())?;
    Ok(document)
}

fn check_statements(
    statements: &[Statement],
    options: &ParseOptions,
    source_len: usize,
) -> Result<(), DotError> {
    let mut pending = vec![(statements, 0usize)];
    let mut count = 0usize;
    while let Some((list, depth)) = pending.pop() {
        options.control.checkpoint().map_err(|error| {
            DotError::new(
                0..source_len,
                format!("DOT validation was cancelled: {error}"),
            )
        })?;
        if depth > options.nesting_depth {
            return Err(DotError::new(
                0..source_len,
                format!("DOT nesting exceeds {}", options.nesting_depth),
            ));
        }
        for statement in list {
            options.control.checkpoint().map_err(|error| {
                DotError::new(
                    0..source_len,
                    format!("DOT validation was cancelled: {error}"),
                )
            })?;
            count += 1;
            if count > options.statements {
                return Err(DotError::new(
                    0..source_len,
                    format!("DOT statement count exceeds {}", options.statements),
                ));
            }
            match statement {
                Statement::Subgraph(subgraph) => pending.push((&subgraph.statements, depth + 1)),
                Statement::Edge { endpoints, .. } => {
                    for endpoint in endpoints {
                        if let Endpoint::Subgraph(subgraph) = endpoint {
                            pending.push((&subgraph.statements, depth + 1));
                        }
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}

fn parse_error(error: ParseError<usize, Token, DotError>) -> DotError {
    match error {
        ParseError::InvalidToken { location } => {
            DotError::new(location..location, "invalid DOT token")
        }
        ParseError::UnrecognizedEof { location, expected } => DotError::new(
            location..location,
            format!("incomplete DOT document; expected {}", expected.join(", ")),
        ),
        ParseError::UnrecognizedToken {
            token: (start, token, end),
            expected,
        } => DotError::new(
            start..end,
            format!(
                "unexpected DOT token {token:?}; expected {}",
                expected.join(", ")
            ),
        ),
        ParseError::ExtraToken {
            token: (start, token, end),
        } => DotError::new(start..end, format!("extra DOT token {token:?}")),
        ParseError::User { error } => error,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_parse_ordered_directed_statements_and_subgraphs() {
        // given
        let source = r#"strict digraph G {
            // defaults and an edge chain
            node [shape=box, color=blue];
            subgraph cluster_0 { a:p:n; b [label="two words"]; }
            a:p:n -> b -> { c; d; } [label=<<B>bold</B>>];
        }"#;
        // when
        let graph = parse(source, &ParseOptions::default()).unwrap();
        // then
        assert!(graph.strict);
        assert_eq!(graph.kind, GraphKind::Digraph);
        assert_eq!(graph.statements.len(), 3);
        assert!(
            matches!(&graph.statements[2], Statement::Edge { endpoints, operators, .. } if endpoints.len() == 3 && operators.len() == 2)
        );
        let Statement::Edge {
            endpoints,
            operators,
            ..
        } = &graph.statements[2]
        else {
            panic!("edge chain")
        };
        let Endpoint::Node(first) = &endpoints[0] else {
            panic!("node endpoint")
        };
        assert_eq!(
            first
                .port
                .iter()
                .map(|part| part.text.as_str())
                .collect::<Vec<_>>(),
            vec!["p", "n"]
        );
        assert_eq!(operators, &[EdgeOp::Directed, EdgeOp::Directed]);
    }

    #[test]
    fn should_follow_graphviz_attribute_separator_grammar() {
        // given
        let valid = [
            "digraph{a[color=red shape=box]}",
            "digraph{a[color=red,shape=box]}",
            "digraph{a[color=red,]}",
        ];
        let invalid = [
            "digraph{a[color=red,,shape=box]}",
            "digraph{a[,color=red]}",
            "digraph{a[color=red;;shape=box]}",
        ];
        // when
        let accepted = valid.map(|source| parse(source, &ParseOptions::default()).is_ok());
        let rejected = invalid.map(|source| parse(source, &ParseOptions::default()).is_err());
        // then
        assert_eq!(accepted, [true; 3]);
        assert_eq!(rejected, [true; 3]);
        let doubled = invalid[0];
        let error = parse(doubled, &ParseOptions::default()).unwrap_err();
        let second_comma = doubled.find(",,").unwrap() + 1;
        assert_eq!(error.span, second_comma..second_comma + 1);
    }

    #[test]
    fn should_treat_non_ascii_whitespace_as_bare_id_text() {
        // given
        let source = "digraph{a\u{a0}b; a\u{2003}b}";
        // when
        let document = parse(source, &ParseOptions::default()).unwrap();
        // then
        let names = document
            .statements
            .iter()
            .map(|statement| {
                let Statement::Node { node, .. } = statement else {
                    panic!("node statement")
                };
                node.id.text.as_str()
            })
            .collect::<Vec<_>>();
        assert_eq!(names, ["a\u{a0}b", "a\u{2003}b"]);
        assert!(parse("digraph{a\u{c}b}", &ParseOptions::default()).is_err());
    }

    #[test]
    fn should_preserve_backslash_before_crlf_in_quoted_id() {
        // given
        let source = "digraph{a[label=\"a\\\r\nb\"]}";
        // when
        let document = parse(source, &ParseOptions::default()).unwrap();
        // then
        let Statement::Node { attributes, .. } = &document.statements[0] else {
            panic!("node")
        };
        assert_eq!(attributes[0].value.as_ref().unwrap().text, "a\\\r\nb");
    }

    #[test]
    fn should_parse_undirected_numeral_quoted_and_html_ids() {
        // given
        let source =
            "graph { -1.5 -- \"node name\"; <<TABLE><TR><TD>x</TD></TR></TABLE>> [label=\"x\"]; }";
        // when
        let graph = parse(source, &ParseOptions::default()).unwrap();
        // then
        assert_eq!(graph.kind, GraphKind::Graph);
        assert_eq!(graph.statements.len(), 2);
    }

    #[test]
    fn should_parse_assignments_anonymous_subgraphs_and_decimal_ids() {
        // given
        let source = "digraph { rankdir=LR; { x; y; } -.5 -> 1.; }";
        // when
        let graph = parse(source, &ParseOptions::default()).unwrap();
        // then
        assert!(
            matches!(&graph.statements[0], Statement::Assignment { name, value } if name.text == "rankdir" && value.text == "LR")
        );
        assert!(
            matches!(&graph.statements[1], Statement::Subgraph(subgraph) if subgraph.id.is_none() && subgraph.statements.len() == 2)
        );
        assert!(
            matches!(&graph.statements[2], Statement::Edge { endpoints, .. } if endpoints.len() == 2)
        );
    }

    #[test]
    fn should_concatenate_quoted_ids_and_preserve_escapes() {
        // given
        let source = r#"digraph { a [label="hello"+" world", tooltip="a\N"]; }"#;
        // when
        let graph = parse(source, &ParseOptions::default()).unwrap();
        // then
        let Statement::Node { attributes, .. } = &graph.statements[0] else {
            panic!("node statement")
        };
        assert_eq!(attributes[0].value.as_ref().unwrap().text, "hello world");
        assert_eq!(attributes[1].value.as_ref().unwrap().text, r"a\N");
    }

    #[test]
    fn should_reject_unquoted_id_concatenation() {
        // given
        let source = "digraph { a [label=first+second]; }";
        // when
        let error = parse(source, &ParseOptions::default()).unwrap_err();
        // then
        assert!(error.message.contains("requires quoted or HTML IDs"));
    }

    #[test]
    fn should_limit_statement_count_and_nesting() {
        // given
        let many = ParseOptions {
            statements: 1,
            ..Default::default()
        };
        let shallow = ParseOptions {
            nesting_depth: 1,
            ..Default::default()
        };
        // when
        let items = parse("digraph { a; b; }", &many).unwrap_err();
        let depth = parse("digraph { subgraph cluster_a { a; } }", &shallow).unwrap_err();
        // then
        assert!(items.message.contains("statement count exceeds 1"));
        assert!(depth.message.contains("nesting exceeds 1"));
    }

    #[test]
    fn should_reject_unterminated_quoted_id_with_span() {
        // given
        let source = "digraph { a [label=\"unfinished] }";
        // when
        let error = parse(source, &ParseOptions::default()).unwrap_err();
        // then
        assert!(error.message.contains("unterminated"));
        assert!(error.span.start < error.span.end);
    }

    #[test]
    fn should_stop_when_cancellation_is_requested() {
        // given
        let control = RenderControl::default();
        control.cancel();
        let options = ParseOptions {
            control,
            ..Default::default()
        };
        // when
        let error = parse("digraph { a -> b; }", &options).unwrap_err();
        // then
        assert!(error.message.contains("cancelled"));
    }

    #[test]
    fn should_stop_before_exceeding_token_limit() {
        // given
        let options = ParseOptions {
            tokens: 5,
            ..Default::default()
        };
        // when
        let error = parse("digraph { a -> b; c -> d; }", &options).unwrap_err();
        // then
        assert!(error.message.contains("token count exceeds 5"));
    }

    #[test]
    fn should_parse_comma_separated_node_lists() {
        // given
        let source = "digraph { a,b -> c,d [weight=2]; e,f [color=blue]; }";
        // when
        let graph = parse(source, &ParseOptions::default()).unwrap();
        // then
        assert!(
            matches!(&graph.statements[0], Statement::Edge { endpoints, operators, .. }
            if matches!(&endpoints[0], Endpoint::Nodes(nodes) if nodes.len() == 2)
            && matches!(&endpoints[1], Endpoint::Nodes(nodes) if nodes.len() == 2)
            && operators == &[EdgeOp::Directed])
        );
        assert!(
            matches!(&graph.statements[1], Statement::Nodes { nodes, attributes }
            if nodes.len() == 2 && attributes[0].name.text == "color")
        );
    }

    #[test]
    fn should_skip_utf8_bom_and_match_keywords_case_sensitively() {
        // given
        let source = "\u{feff}digraph { DiGraph; }";
        // when
        let graph = parse(source, &ParseOptions::default()).unwrap();
        // then
        assert!(matches!(&graph.statements[0], Statement::Node { node, .. }
            if node.id.text == "DiGraph"));
    }

    #[test]
    fn should_strip_outer_html_delimiters_and_concatenate_quoted_atoms() {
        // given
        let source = "digraph { a [label=<<B>bold</B>>, tooltip=<foo>+\"bar\"]; }";
        // when
        let graph = parse(source, &ParseOptions::default()).unwrap();
        // then
        let Statement::Node { attributes, .. } = &graph.statements[0] else {
            panic!("node statement")
        };
        assert_eq!(attributes[0].value.as_ref().unwrap().text, "<B>bold</B>");
        assert_eq!(attributes[0].value.as_ref().unwrap().kind, IdKind::Html);
        assert_eq!(attributes[1].value.as_ref().unwrap().text, "foobar");
        assert_eq!(attributes[1].value.as_ref().unwrap().kind, IdKind::Quoted);
    }

    #[test]
    fn should_reject_attribute_without_value() {
        // given
        let source = "digraph { a [shape]; }";
        // when
        let error = parse(source, &ParseOptions::default()).unwrap_err();
        // then
        assert!(error.message.contains("unexpected DOT token"));
        assert!(error.message.contains("CloseAttrs"));
    }

    #[test]
    fn should_keep_semicolon_as_edge_statement_boundary() {
        // given
        let source = "digraph { a -> b; -> c; }";
        // when
        let error = parse(source, &ParseOptions::default()).unwrap_err();
        // then
        assert!(error.message.contains("Arrow"));
    }

    #[test]
    fn should_reject_edge_operator_for_wrong_graph_kind() {
        // given
        let directed = "graph { a -> b; }";
        let undirected = "digraph { a -- b; }";
        // when
        let directed_error = parse(directed, &ParseOptions::default()).unwrap_err();
        let undirected_error = parse(undirected, &ParseOptions::default()).unwrap_err();
        // then
        assert!(directed_error.message.contains("undirected graph"));
        assert!(undirected_error.message.contains("directed graph"));
    }

    #[test]
    fn should_accept_graphviz_attribute_macro_syntax() {
        // given
        let source = "digraph { node defaults=[shape=box]; a; }";
        // when
        let graph = parse(source, &ParseOptions::default()).unwrap();
        // then
        assert!(
            matches!(&graph.statements[0], Statement::Attributes { scope: AttributeScope::Node, attributes }
            if attributes[0].name.text == "shape")
        );
    }

    #[test]
    fn should_split_badly_delimited_numerals_like_graphviz() {
        // given
        let source = "graph { 1.2.3; }";
        // when
        let graph = parse(source, &ParseOptions::default()).unwrap();
        // then
        assert!(
            matches!(&graph.statements[0], Statement::Node { node, .. } if node.id.text == "1.2")
        );
        assert!(
            matches!(&graph.statements[1], Statement::Node { node, .. } if node.id.text == ".3")
        );
    }
}
