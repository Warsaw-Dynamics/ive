use super::Row;

/// No HEEx keys in intellij-community; ids in IntelliJ's style. The code in `{...}` and `<%= %>` is coloured as
/// Elixir.
pub const ROWS: &[Row] = &[
    ("HEEX.TEXT", "Text", Some("TEXT")),
    ("HEEX.COMMENT", "Comment", Some("DEFAULT_BLOCK_COMMENT")),
    ("HEEX.TAG", "Tag", Some("DEFAULT_TAG")),
    ("HEEX.TAG_NAME", "Tag name", Some("HEEX.TAG")),
    ("HEEX.COMPONENT", "Function component", Some("DEFAULT_FUNCTION_CALL")),
    ("HEEX.COMPONENT_MODULE", "Component module", Some("DEFAULT_CLASS_NAME")),
    ("HEEX.SLOT", "Slot", Some("HEEX.TAG_NAME")),
    ("HEEX.ATTRIBUTE_NAME", "Attribute name", Some("DEFAULT_ATTRIBUTE")),
    ("HEEX.SPECIAL_ATTRIBUTE", "Special attribute", Some("DEFAULT_KEYWORD")),
    ("HEEX.ATTRIBUTE_VALUE", "Attribute value", Some("DEFAULT_STRING")),
    ("HEEX.DELIMITER", "Expression delimiter", Some("DEFAULT_TEMPLATE_LANGUAGE_COLOR")),
];
