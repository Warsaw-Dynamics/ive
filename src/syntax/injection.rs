use std::collections::{HashMap, HashSet};

use streaming_iterator::StreamingIterator;
use tree_sitter::{Node, Query, QueryCursor, QueryMatch, Range};

use super::Language;
use super::parse::Layer;

/// Embedded code found in a layer: its language and the ranges to parse as one document, sorted and disjoint.
#[derive(Debug)]
pub struct Injection {
    pub language: Language,
    pub ranges: Vec<Range>,
}

/// The embedded code `query` (the layer language's injection query) finds in `layer`. Matches naming an unknown
/// language are skipped; the first match naming a language claims its nodes, so a query lists specific patterns
/// (`<script type="application/json">`) before general ones. Matches of a `(#set! injection.combined)` pattern
/// become one injection per language.
pub fn injections(layer: &Layer, query: &Query, source: &[u8]) -> Vec<Injection> {
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(query, layer.tree.root_node(), source);
    let mut separate = Vec::new();
    let mut combined: HashMap<(usize, Language), Vec<Range>> = HashMap::new();
    let mut claimed = HashSet::new();
    while let Some(found) = matches.next() {
        let Some(language) = language_of(query, found, source) else {
            continue;
        };
        let nodes: Vec<Node> = captured(query, found, "injection.content").collect();
        if nodes.iter().any(|node| !claimed.insert(node.id())) {
            continue;
        }
        let ranges = content_ranges(query, found.pattern_index, &nodes, &layer.ranges);
        if ranges.is_empty() {
            continue;
        }
        if has_property(query, found.pattern_index, "injection.combined") {
            combined.entry((found.pattern_index, language)).or_default().extend(ranges);
        } else {
            separate.push(Injection { language, ranges });
        }
    }
    let joined = combined.into_iter().map(|((_, language), mut ranges)| {
        ranges.sort_by_key(|range| range.start_byte);
        Injection { language, ranges }
    });
    separate.into_iter().chain(joined).collect()
}

/// From `(#set! injection.language "css")`, else the text of the `@injection.language` capture.
fn language_of(query: &Query, found: &QueryMatch, source: &[u8]) -> Option<Language> {
    let set = query
        .property_settings(found.pattern_index)
        .iter()
        .find(|property| &*property.key == "injection.language")
        .and_then(|property| property.value.as_deref().map(str::to_owned));
    let name = set.or_else(|| {
        let node = captured(query, found, "injection.language").next()?;
        Some(String::from_utf8_lossy(source.get(node.byte_range())?).into_owned())
    })?;
    Language::for_alias(&name)
}

fn has_property(query: &Query, pattern: usize, key: &str) -> bool {
    query.property_settings(pattern).iter().any(|property| &*property.key == key)
}

fn captured<'tree>(query: &Query, found: &QueryMatch<'_, 'tree>, name: &str) -> impl Iterator<Item = Node<'tree>> {
    let index = query.capture_index_for_name(name);
    found.captures().iter().filter(move |capture| Some(capture.index) == index).map(|capture| capture.node)
}

/// The `@injection.content` nodes' ranges, without their children unless the pattern sets
/// `injection.include-children`, and within the parent layer's `bounds` (none: the whole document).
fn content_ranges(query: &Query, pattern: usize, nodes: &[Node], bounds: &[Range]) -> Vec<Range> {
    let include_children = has_property(query, pattern, "injection.include-children");
    nodes
        .iter()
        .flat_map(|&node| if include_children { vec![node.range()] } else { without_children(node) })
        .flat_map(|range| within(range, bounds))
        .filter(|range| range.start_byte < range.end_byte)
        .collect()
}

/// `node`'s range with its named children cut out, e.g. a comment's text without the tags a grammar parsed inside
/// it. Anonymous children stay (as in Neovim): Markdown's `inline` has one per `*` or `=` of the embedded text.
fn without_children(node: Node) -> Vec<Range> {
    let mut ranges = Vec::new();
    let mut start = node.range();
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        let child = child.range();
        if child.start_byte > start.start_byte {
            ranges.push(Range { end_byte: child.start_byte, end_point: child.start_point, ..start });
        }
        start = Range { start_byte: child.end_byte, start_point: child.end_point, ..start };
    }
    if start.start_byte < node.end_byte() {
        ranges.push(Range { end_byte: node.end_byte(), end_point: node.end_position(), ..start });
    }
    ranges
}

/// The parts of `range` inside `bounds`; all of it when `bounds` is empty (a host layer covers everything).
fn within(range: Range, bounds: &[Range]) -> Vec<Range> {
    if bounds.is_empty() {
        return vec![range];
    }
    bounds
        .iter()
        .filter(|bound| bound.start_byte < range.end_byte && range.start_byte < bound.end_byte)
        .map(|bound| {
            let (start_byte, start_point) = if bound.start_byte > range.start_byte {
                (bound.start_byte, bound.start_point)
            } else {
                (range.start_byte, range.start_point)
            };
            let (end_byte, end_point) =
                if bound.end_byte < range.end_byte { (bound.end_byte, bound.end_point) } else { (range.end_byte, range.end_point) };
            Range { start_byte, end_byte, start_point, end_point }
        })
        .collect()
}


#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tree_sitter::Point;

    use super::*;
    use crate::syntax::{Cancel, PARSE_BUDGET, Syntax, lang, parse, spans};

    fn range(bytes: std::ops::Range<usize>) -> Range {
        Range {
            start_byte: bytes.start,
            end_byte: bytes.end,
            start_point: Point { row: 0, column: bytes.start },
            end_point: Point { row: 0, column: bytes.end },
        }
    }

    fn parsed(language: &str, source: &str) -> Syntax {
        parse(source.as_bytes(), lang(language), &Cancel::default(), PARSE_BUDGET).unwrap()
    }

    /// The key highlighting the first occurrence of `needle` when highlighting `range` of `source`.
    fn key_in(syntax: &Syntax, source: &str, range: std::ops::Range<usize>, needle: &str) -> Option<&'static str> {
        let at = source.find(needle).unwrap_or_else(|| panic!("{needle} not in source"));
        spans(syntax, source.as_bytes(), range)
            .into_iter()
            .find(|highlight| highlight.bytes.contains(&at))
            .map(|highlight| highlight.key.id())
    }

    fn key_at(language: &str, source: &str, needle: &str) -> Option<&'static str> {
        key_in(&parsed(language, source), source, 0..source.len(), needle)
    }

    fn layer_languages(syntax: &Syntax) -> Vec<(&'static str, usize)> {
        syntax.layers().iter().map(|layer| (layer.language.name(), layer.depth)).collect()
    }

    #[test]
    fn should_clip_range_to_parent_bounds() {
        // given
        let bounds = [range(0..4), range(8..12)];

        // when
        let parts: Vec<_> = within(range(2..10), &bounds).into_iter().map(|part| part.start_byte..part.end_byte).collect();

        // then
        assert_eq!(parts, [2..4, 8..10]);
    }

    #[test]
    fn should_colour_html_script_and_style_in_their_languages() {
        // given
        let source = "<html><style>h1 { color: red; }</style><script>let n = 42;</script></html>";

        // when
        let keys = [key_at("HTML", source, "style"), key_at("HTML", source, "color"), key_at("HTML", source, "42")];

        // then
        assert_eq!(keys, [Some("HTML_TAG_NAME"), Some("CSS.PROPERTY_NAME"), Some("JS.NUMBER")]);
    }

    #[test]
    fn should_pick_script_language_from_type_and_lang_attributes() {
        // given
        let source = concat!(
            "<script type=\"application/ld+json\">{\"a\": 1}</script>\n",
            "<script lang=\"ts\">let n: number = 1;</script>\n",
            "<script lang=\"klingon\">let k = 2;</script>",
        );

        // when
        let syntax = parsed("HTML", source);

        // then
        assert_eq!(layer_languages(&syntax), [("HTML", 0), ("JSON", 1), ("TypeScript", 1), ("JavaScript", 1)]);
    }

    #[test]
    fn should_keep_anonymous_children_of_content_node_in_embedded_code() {
        // given
        let source = "```js\nlet n = 4 * 2;\n```\n";

        // when
        let keys = [key_at("Markdown", source, "="), key_at("Markdown", source, "*")];

        // then
        assert_eq!(keys, [Some("JS.OPERATION_SIGN"), Some("JS.OPERATION_SIGN")]);
    }

    #[test]
    fn should_colour_range_starting_inside_embedded_code() {
        // given
        let source = "<script>\nlet a = 1;\nlet b = 2;\n</script>";
        let syntax = parsed("HTML", source);
        let second = source.find("let b").unwrap();

        // when
        let key = key_in(&syntax, source, second..second + 10, "2;");

        // then
        assert_eq!(key, Some("JS.NUMBER"));
    }

    #[test]
    fn should_keep_host_layer_when_budget_runs_out_before_embedded_code() {
        // given
        let source = "<p>hi</p><script>let n = 42;</script>";

        // when
        let syntax = parse(source.as_bytes(), lang("HTML"), &Cancel::default(), Duration::ZERO).unwrap();

        // then
        assert_eq!(layer_languages(&syntax), [("HTML", 0)]);
    }
}
