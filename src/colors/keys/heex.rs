use super::Row;

/// No HEEx keys in intellij-community; ids in IntelliJ's style. The code in `{...}` and `<%= %>` is coloured as
/// Elixir.
pub const ROWS: &[Row] = &[
    ("HEEX.TEXT", Some("TEXT")),
    ("HEEX.COMMENT", Some("DEFAULT_BLOCK_COMMENT")),
    ("HEEX.TAG", Some("DEFAULT_TAG")),
    ("HEEX.TAG_NAME", Some("HEEX.TAG")),
    ("HEEX.COMPONENT", Some("DEFAULT_FUNCTION_CALL")),
    ("HEEX.COMPONENT_MODULE", Some("DEFAULT_CLASS_NAME")),
    ("HEEX.SLOT", Some("HEEX.TAG_NAME")),
    ("HEEX.ATTRIBUTE_NAME", Some("DEFAULT_ATTRIBUTE")),
    ("HEEX.SPECIAL_ATTRIBUTE", Some("DEFAULT_KEYWORD")),
    ("HEEX.ATTRIBUTE_VALUE", Some("DEFAULT_STRING")),
    ("HEEX.DELIMITER", Some("DEFAULT_TEMPLATE_LANGUAGE_COLOR")),
];
