use super::{keys_at, parsed};

#[test]
fn should_tell_heex_tag_from_components_slot_and_special_attribute() {
    // given
    let source = "<div class=\"x\">\n  <MyApp.Card>\n    <:header :let={card}>Hi</:header>\n  </MyApp.Card>\n  <.button />\n</div>";

    // when
    let keys = keys_at("HEEx", source, ["div", "class", "\"x\"", "MyApp", "Card", "header", ":let", "button", "Hi"]);

    // then
    assert_eq!(
        keys,
        [
            Some("HEEX.TAG_NAME"),
            Some("HEEX.ATTRIBUTE_NAME"),
            Some("HEEX.ATTRIBUTE_VALUE"),
            Some("HEEX.COMPONENT_MODULE"),
            Some("HEEX.COMPONENT_MODULE"),
            Some("HEEX.SLOT"),
            Some("HEEX.SPECIAL_ATTRIBUTE"),
            Some("HEEX.COMPONENT"),
            Some("HEEX.TEXT"),
        ]
    );
}

#[test]
fn should_colour_heex_expressions_and_directives_as_elixir() {
    // given
    let source =
        "<%!-- note --%>\n<p title={@title}>\n<%= for item <- list_items() do %>\n  {item.name}\n<% end %>\n</p>";

    // when
    let keys = keys_at("HEEx", source, ["note", "{@", "@title", "<%=", "for", "list_items", "name", "end"]);

    // then
    assert_eq!(
        keys,
        [
            Some("HEEX.COMMENT"),
            Some("HEEX.DELIMITER"),
            Some("ELIXIR.MODULE_ATTRIBUTE"),
            Some("HEEX.DELIMITER"),
            Some("ELIXIR.KEYWORD"),
            Some("ELIXIR.FUNCTION_CALL"),
            Some("ELIXIR.FIELD"),
            Some("ELIXIR.KEYWORD"),
        ]
    );
}

#[test]
fn should_parse_heex_block_directives_as_one_elixir_document() {
    // given
    let source = "<%= if @ok do %>\n<p>yes</p>\n<% end %>";

    // when
    let syntax = parsed("HEEx", source);

    // then
    let layers: Vec<_> = syntax.layers().iter().map(|layer| (layer.language.name(), layer.depth)).collect();
    assert_eq!(layers, [("HEEx", 0), ("Elixir", 1)]);
}
