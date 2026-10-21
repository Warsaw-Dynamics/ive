use super::Row;

/// Ids, names and parents from IntelliJ's `XmlHighlighterColors` and `HTMLColorsPage`.
pub const ROWS: &[Row] = &[
    ("HTML_CODE", "HTML code", Some("TEXT")),
    ("HTML_COMMENT", "Comment", Some("DEFAULT_LINE_COMMENT")),
    ("HTML_TAG", "Tag", Some("DEFAULT_TAG")),
    ("HTML_TAG_NAME", "Tag name", Some("DEFAULT_KEYWORD")),
    ("HTML_CUSTOM_TAG_NAME", "Custom tag name", Some("HTML_TAG_NAME")),
    ("HTML_ATTRIBUTE_NAME", "Attribute name", Some("DEFAULT_ATTRIBUTE")),
    ("HTML_ATTRIBUTE_VALUE", "Attribute value", Some("DEFAULT_STRING")),
    ("HTML_ENTITY_REFERENCE", "Entity reference", Some("DEFAULT_ENTITY")),
];
