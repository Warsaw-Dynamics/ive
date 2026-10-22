use super::{keys_at, parsed};

/// Sorted, as combined injections come in no particular order.
fn layer_languages(language: &str, source: &str) -> Vec<(&'static str, usize)> {
    let mut layers: Vec<_> =
        parsed(language, source).layers().iter().map(|layer| (layer.language.name(), layer.depth)).collect();
    layers.sort();
    layers
}

#[test]
fn should_colour_erb_ruby_code_and_html_around_it() {
    // given
    let source = "<%# list %>\n<ul class=\"x\">\n<% @items.each do |item| %>\n  <li><%= item.name %></li>\n<% end %>\n</ul>";

    // when
    let keys = keys_at("ERB", source, ["list", "ul", "class", "<% @", "@items", "<%=", "name", "|item"]);

    // then
    assert_eq!(
        keys,
        [
            Some("ETMPL.COMMENT"),
            Some("HTML_TAG_NAME"),
            Some("HTML_ATTRIBUTE_NAME"),
            Some("ETMPL.DELIMITER"),
            Some("RUBY.INSTANCE_VARIABLE"),
            Some("ETMPL.OUTPUT_DELIMITER"),
            Some("RUBY.METHOD_CALL"),
            Some("RUBY.OPERATION_SIGN"),
        ]
    );
}

#[test]
fn should_parse_erb_code_as_one_ruby_document_and_content_as_one_html_document() {
    // given
    let source = "<% if ok %>\n<p>yes</p>\n<% else %>\n<p>no</p>\n<% end %>";

    // when
    let layers = layer_languages("ERB", source);

    // then
    assert_eq!(layers, [("ERB", 0), ("HTML", 1), ("Ruby", 1)]);
}

#[test]
fn should_colour_ejs_code_as_javascript() {
    // given
    let source = "<% if (user) { %>\n  <h2><%= user.name %></h2>\n<% } %>";

    // when
    let keys = keys_at("EJS", source, ["if", "h2", "<%="]);

    // then
    assert_eq!(keys, [Some("JS.KEYWORD"), Some("HTML_TAG_NAME"), Some("ETMPL.OUTPUT_DELIMITER")]);
}
