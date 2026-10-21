use super::{keys_at, parsed};

fn layers(language: &str, source: &str) -> Vec<(&'static str, usize)> {
    parsed(language, source).layers().iter().map(|layer| (layer.language.name(), layer.depth)).collect()
}

#[test]
fn should_tell_markdown_header_from_list_quote_and_fence_markers() {
    // given
    let source = "## Title\n\n- item\n\n> quoted\n\n```\nplain\n```\n\n---\n";

    // when
    let keys = keys_at("Markdown", source, ["##", "Title", "- ", "item", "> ", "quoted", "```", "plain", "---"]);

    // then
    assert_eq!(
        keys,
        [
            Some("MARKDOWN_HEADER_MARKER"),
            Some("MARKDOWN_HEADER_LEVEL_2"),
            Some("MARKDOWN_LIST_MARKER"),
            Some("MARKDOWN_UNORDERED_LIST"),
            Some("MARKDOWN_BLOCK_QUOTE_MARKER"),
            Some("MARKDOWN_BLOCK_QUOTE"),
            Some("MARKDOWN_CODE_FENCE_MARKER"),
            Some("MARKDOWN_CODE_FENCE"),
            Some("MARKDOWN_HRULE"),
        ]
    );
}

#[test]
fn should_colour_markdown_inline_emphasis_code_span_and_link() {
    // given
    let source = "Some *it* and **bold** with `code` and [text](http://x.io).\n";

    // when
    let keys = keys_at("Markdown", source, ["Some", "it*", "*it", "bold", "code", "`", "text", "http"]);

    // then
    assert_eq!(
        keys,
        [
            Some("MARKDOWN_TEXT"),
            Some("MARKDOWN_ITALIC"),
            Some("MARKDOWN_ITALIC_MARKER"),
            Some("MARKDOWN_BOLD"),
            Some("MARKDOWN_CODE_SPAN"),
            Some("MARKDOWN_CODE_SPAN_MARKER"),
            Some("MARKDOWN_LINK_TEXT"),
            Some("MARKDOWN_LINK_DESTINATION"),
        ]
    );
}

#[test]
fn should_colour_markdown_fenced_block_in_the_language_of_its_info_string() {
    // given
    let source = "```rust\nfn main() { let n = 42; }\n```\n\n```klingon\nfn x\n```\n";

    // when
    let keys = keys_at("Markdown", source, ["rust", "fn main", "42", "fn x"]);

    // then
    assert_eq!(
        keys,
        [Some("MARKDOWN_CODE_FENCE_LANGUAGE"), Some("RUST.KEYWORD"), Some("RUST.NUMBER"), Some("MARKDOWN_CODE_FENCE")]
    );
}

#[test]
fn should_colour_markdown_html_block_inline_html_and_front_matter() {
    // given
    let source = "---\ntitle: Notes\n---\n\nSee <b>this</b>.\n\n<div class=\"x\">\n</div>\n";

    // when
    let keys = keys_at("Markdown", source, ["title", "b>", "b>.", "class"]);

    // then
    assert_eq!(keys, [Some("YAML_SCALAR_KEY"), Some("HTML_TAG_NAME"), Some("HTML_TAG_NAME"), Some("HTML_ATTRIBUTE_NAME")]);
}

#[test]
fn should_colour_regex_in_javascript_in_html_in_markdown_code_block() {
    // given
    let source = "```html\n<script>\nconst re = /a+/;\n</script>\n```\n";

    // when
    let keys = keys_at("Markdown", source, ["script", "const", "a+", "+"]);

    // then
    assert_eq!(keys, [Some("HTML_TAG_NAME"), Some("JS.KEYWORD"), Some("REGEXP.CHARACTER"), Some("REGEXP.QUANTIFIER")]);
    assert_eq!(layers("Markdown", source), [("Markdown", 0), ("HTML", 1), ("JavaScript", 2), ("RegExp", 3)]);
}

#[test]
fn should_not_parse_a_fourth_embedded_level() {
    // given
    let source = "````markdown\n```html\n<script>\nconst re = /a+/;\n</script>\n```\n````\n";

    // when
    let layers = layers("Markdown", source);
    let keys = keys_at("Markdown", source, ["const", "a+"]);

    // then
    assert_eq!(layers, [("Markdown", 0), ("Markdown", 1), ("HTML", 2), ("JavaScript", 3)]);
    assert_eq!(keys, [Some("JS.KEYWORD"), Some("JS.REGEXP")]);
}
