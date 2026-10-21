use std::cmp::Reverse;
use std::ops::Range;

use streaming_iterator::StreamingIterator;
use tree_sitter::{Node, QueryCursor};

use super::parse::Layer;
use super::{Grammar, Syntax};
use crate::colors::ColorKey;

/// A run of bytes in one colour key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Highlight {
    pub bytes: Range<usize>,
    pub key: ColorKey,
}

/// A node a query pattern captured; `pattern` is the pattern's index in the query, `depth` its layer's.
pub struct Capture {
    pub bytes: Range<usize>,
    pub depth: usize,
    pub pattern: usize,
    pub key: ColorKey,
}

/// Highlights of `range` of `source` (which `syntax` was parsed from): sorted and non-overlapping. Embedded code
/// paints over its host; within a layer, where captures overlap, the innermost node wins; for the same node, the
/// later pattern.
pub fn spans(syntax: &Syntax, source: &[u8], range: Range<usize>) -> Vec<Highlight> {
    let captures = syntax
        .layers()
        .iter()
        .filter(|layer| layer.overlaps(&range))
        .flat_map(|layer| match layer.language.grammar() {
            Ok(grammar) => captures_in(layer, grammar, source, range.clone()),
            Err(_) => Vec::new(),
        })
        .collect();
    paint(captures, range)
}

/// Queries start at the top-level nodes the range overlaps, not at the root: a ranged query from the root walks
/// every top-level node of some trees (2 500 YAML documents: 25 ms per line instead of 7 µs). Two root queries add
/// the matches that start higher up: at the root itself (a shebang as the file's first child), and at a top-level
/// node within the overlapping ones and their neighbours (a docstring followed by the definition it documents).
fn captures_in(layer: &Layer, grammar: &Grammar, source: &[u8], range: Range<usize>) -> Vec<Capture> {
    let root = layer.tree.root_node();
    let nodes = top_level_nodes(root, &range);
    let window = neighbourhood(&nodes).unwrap_or_else(|| range.clone());
    let starts = [Start::Root, Start::TopLevel { window }].into_iter().chain(nodes.into_iter().map(Start::Within));
    starts.flat_map(|start| captures_from(start.at(root), layer, grammar, source, range.clone())).collect()
}

/// Where a query starts and which of its matches it keeps.
enum Start<'tree> {
    /// matches starting at the root node itself
    Root,
    /// matches starting at a top-level node and lying within `window`
    TopLevel { window: Range<usize> },
    /// matches starting at or below this top-level node
    Within(Node<'tree>),
}

/// A query start resolved to its node, deepest start depth and containing range.
struct Origin<'tree> {
    node: Node<'tree>,
    max_start_depth: Option<u32>,
    window: Option<Range<usize>>,
}

impl<'tree> Start<'tree> {
    fn at(self, root: Node<'tree>) -> Origin<'tree> {
        match self {
            Self::Root => Origin { node: root, max_start_depth: Some(0), window: None },
            Self::TopLevel { window } => Origin { node: root, max_start_depth: Some(1), window: Some(window) },
            Self::Within(node) => Origin { node, max_start_depth: None, window: None },
        }
    }
}

/// The root's children that overlap `range`, in order.
fn top_level_nodes<'tree>(root: Node<'tree>, range: &Range<usize>) -> Vec<Node<'tree>> {
    std::iter::successors(root.first_child_for_byte(range.start), Node::next_sibling)
        .take_while(|node| node.start_byte() < range.end)
        .collect()
}

/// From the sibling before the first of `nodes` to the sibling after the last.
fn neighbourhood(nodes: &[Node]) -> Option<Range<usize>> {
    let first = nodes.first()?;
    let last = nodes.last()?;
    let start = first.prev_sibling().unwrap_or(*first).start_byte();
    let end = last.next_sibling().unwrap_or(*last).end_byte();
    Some(start..end)
}

fn captures_from(origin: Origin, layer: &Layer, grammar: &Grammar, source: &[u8], range: Range<usize>) -> Vec<Capture> {
    let mut cursor = QueryCursor::new();
    cursor.set_byte_range(range);
    cursor.set_max_start_depth(origin.max_start_depth);
    if let Some(window) = origin.window {
        cursor.set_containing_byte_range(window);
    }
    let mut matches = cursor.captures(&grammar.query, origin.node, source);
    let mut captures = Vec::new();
    while let Some((found, index)) = matches.next() {
        let capture = found.captures()[*index];
        let Some(key) = grammar.keys[capture.index as usize] else {
            continue;
        };
        for bytes in within_layer(capture.node.byte_range(), layer) {
            captures.push(Capture { bytes, depth: layer.depth, pattern: found.pattern_index, key });
        }
    }
    captures
}

/// The parts of `bytes` inside `layer`'s ranges: a node of a combined layer (HTML text around ERB tags) may span
/// the host code between them, which it must not paint.
fn within_layer(bytes: Range<usize>, layer: &Layer) -> Vec<Range<usize>> {
    if layer.ranges.is_empty() {
        return vec![bytes];
    }
    layer
        .ranges
        .iter()
        .map(|range| bytes.start.max(range.start_byte)..bytes.end.min(range.end_byte))
        .filter(|part| part.start < part.end)
        .collect()
}

/// Paints shallower layers first; within a layer wider captures first and narrower ones over them, equal ones in
/// pattern order; clipped to `range`.
pub fn paint(mut captures: Vec<Capture>, range: Range<usize>) -> Vec<Highlight> {
    captures.sort_by_key(|capture| (capture.depth, Reverse(capture.bytes.len()), capture.pattern));
    let mut canvas: Vec<Option<ColorKey>> = vec![None; range.len()];
    for capture in &captures {
        let start = capture.bytes.start.clamp(range.start, range.end) - range.start;
        let end = capture.bytes.end.clamp(range.start, range.end) - range.start;
        canvas[start..end].fill(Some(capture.key));
    }
    runs(&canvas, range.start)
}

fn runs(canvas: &[Option<ColorKey>], offset: usize) -> Vec<Highlight> {
    let mut highlights: Vec<Highlight> = Vec::new();
    for (at, painted) in canvas.iter().enumerate() {
        let Some(key) = *painted else {
            continue;
        };
        let byte = offset + at;
        match highlights.last_mut() {
            Some(last) if last.key == key && last.bytes.end == byte => last.bytes.end += 1,
            _ => highlights.push(Highlight { bytes: byte..byte + 1, key }),
        }
    }
    highlights
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::{Cancel, Language, PARSE_BUDGET, lang, parse};

    /// The key highlighting the first occurrence of `needle` in `source`.
    fn key_at(language: Language, source: &str, needle: &str) -> Option<&'static str> {
        let syntax = parse(source.as_bytes(), language, &Cancel::default(), PARSE_BUDGET).unwrap();
        let at = source.find(needle).unwrap_or_else(|| panic!("{needle} not in source"));
        spans(&syntax, source.as_bytes(), 0..source.len())
            .into_iter()
            .find(|highlight| highlight.bytes.contains(&at))
            .map(|highlight| highlight.key.id())
    }

    fn key(id: &str) -> ColorKey {
        ColorKey::by_id(id).unwrap()
    }

    #[test]
    fn should_tell_java_method_declaration_from_call() {
        // given
        let source = "class A { void run() { start(); } }";

        // when
        let keys = (key_at(lang("Java"), source, "run"), key_at(lang("Java"), source, "start"));

        // then
        assert_eq!(keys, (Some("METHOD_DECLARATION_ATTRIBUTES"), Some("METHOD_CALL_ATTRIBUTES")));
    }

    #[test]
    fn should_mark_java_static_final_field_and_annotation() {
        // given
        let source = "@Deprecated class A { static final int MAX = 1; private final int size = 2; }";

        // when
        let keys = [
            key_at(lang("Java"), source, "MAX"),
            key_at(lang("Java"), source, "size"),
            key_at(lang("Java"), source, "Deprecated"),
        ];

        // then
        assert_eq!(
            keys,
            [Some("STATIC_FINAL_FIELD_ATTRIBUTES"), Some("INSTANCE_FINAL_FIELD_ATTRIBUTES"), Some("ANNOTATION_NAME_ATTRIBUTES")]
        );
    }

    #[test]
    fn should_tell_java_doc_comment_from_block_comment() {
        // given
        let source = "/** doc */\n/* block */\nclass A {}";

        // when
        let keys = (key_at(lang("Java"), source, "doc"), key_at(lang("Java"), source, "block"));

        // then
        assert_eq!(keys, (Some("JAVA_DOC_COMMENT"), Some("JAVA_BLOCK_COMMENT")));
    }

    #[test]
    fn should_mark_rust_macro_lifetime_and_keep_doc_comment_slashes() {
        // given
        let source = "/// doc\nfn f<'a>(s: &'a str) { println!(\"x\"); }";

        // when
        let keys = [
            key_at(lang("Rust"), source, "println"),
            key_at(lang("Rust"), source, "'a"),
            key_at(lang("Rust"), source, "///"),
        ];

        // then
        assert_eq!(keys, [Some("RUST.MACRO"), Some("RUST.LIFETIME"), Some("RUST.DOC_COMMENT")]);
    }

    #[test]
    fn should_mark_rust_imported_type_as_type_and_variant_in_path_as_variant() {
        // given
        let source = "use std::cell::Cell;\nuse crate::syntax::{Cancel, MAX_BYTES};\nfn f() { Option::None; }";

        // when
        let keys = [
            key_at(lang("Rust"), source, "Cell;"),
            key_at(lang("Rust"), source, "Cancel"),
            key_at(lang("Rust"), source, "MAX_BYTES"),
            key_at(lang("Rust"), source, "None"),
        ];

        // then
        assert_eq!(keys, [Some("RUST.TYPE"), Some("RUST.TYPE"), Some("RUST.CONSTANT"), Some("RUST.ENUM_VARIANT")]);
    }

    #[test]
    fn should_tell_json_property_key_from_string_value() {
        // given
        let source = r#"{"name": "value"}"#;

        // when
        let keys = (key_at(lang("JSON"), source, "name"), key_at(lang("JSON"), source, "value"));

        // then
        assert_eq!(keys, (Some("JSON.PROPERTY_KEY"), Some("JSON.STRING")));
    }

    #[test]
    fn should_tell_yaml_key_from_value_and_mark_toml_key_and_date() {
        // given
        let yaml = "name: value\n";
        let toml = "when = 1979-05-27\n";

        // when
        let keys = [
            key_at(lang("YAML"), yaml, "name"),
            key_at(lang("YAML"), yaml, "value"),
            key_at(lang("TOML"), toml, "when"),
            key_at(lang("TOML"), toml, "1979"),
        ];

        // then
        assert_eq!(keys, [Some("YAML_SCALAR_KEY"), Some("YAML_SCALAR_VALUE"), Some("org.toml.KEY"), Some("org.toml.DATE")]);
    }

    #[test]
    fn should_mark_typescript_interface_and_javascript_regex() {
        // given
        let ts = "interface Shape { area(): number }";
        let js = "const re = /a+/g;";

        // when
        let keys = (key_at(lang("TypeScript"), ts, "Shape"), key_at(lang("JavaScript"), js, "/g"));

        // then
        assert_eq!(keys, (Some("TS.INTERFACE"), Some("JS.REGEXP")));
    }

    #[test]
    fn should_let_escape_inside_string_win() {
        // given
        let source = r#"class A { String s = "a\nb"; }"#;

        // when
        let keys = (key_at(lang("Java"), source, "\\n"), key_at(lang("Java"), source, "a\\"));

        // then
        assert_eq!(keys, (Some("JAVA_VALID_STRING_ESCAPE"), Some("JAVA_STRING")));
    }

    #[test]
    fn should_highlight_range_starting_inside_block_comment() {
        // given
        let source = "/*\n first\n second\n*/\nclass A {}";
        let syntax = parse(source.as_bytes(), lang("Java"), &Cancel::default(), PARSE_BUDGET).unwrap();
        let second = source.find("second").unwrap();

        // when
        let highlights = spans(&syntax, source.as_bytes(), second..source.len());

        // then
        assert_eq!(highlights.first(), Some(&Highlight { bytes: second..source.find("*/").unwrap() + 2, key: key("JAVA_BLOCK_COMMENT") }));
    }

    #[test]
    fn should_let_later_pattern_win_for_same_node() {
        // given
        let captures = vec![
            Capture { bytes: 0..3, depth: 0, pattern: 5, key: key("JAVA_KEYWORD") },
            Capture { bytes: 0..3, depth: 0, pattern: 2, key: key("CLASS_NAME_ATTRIBUTES") },
        ];

        // when
        let highlights = paint(captures, 0..3);

        // then
        assert_eq!(highlights, [Highlight { bytes: 0..3, key: key("JAVA_KEYWORD") }]);
    }

    #[test]
    fn should_not_paint_embedded_node_over_host_code_between_its_ranges() {
        // given
        let source = "<p>Hi <%= name %> bye</p>";

        // when
        let keys = ["Hi", "<%=", "bye"].map(|needle| key_at(lang("ERB"), source, needle));

        // then
        assert_eq!(keys, [Some("HTML_CODE"), Some("ETMPL.OUTPUT_DELIMITER"), Some("HTML_CODE")]);
    }
}
