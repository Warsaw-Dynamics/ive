use super::Row;

/// Ids, names and parents from IntelliJ's `XmlHighlighterColors` and `HTMLColorsPage`.
pub const ROWS: &[Row] = &[
    ("HTML_CODE", Some("TEXT")),
    ("HTML_COMMENT", Some("DEFAULT_LINE_COMMENT")),
    ("HTML_TAG", Some("DEFAULT_TAG")),
    ("HTML_TAG_NAME", Some("DEFAULT_KEYWORD")),
    ("HTML_CUSTOM_TAG_NAME", Some("HTML_TAG_NAME")),
    ("HTML_ATTRIBUTE_NAME", Some("DEFAULT_ATTRIBUTE")),
    ("HTML_ATTRIBUTE_VALUE", Some("DEFAULT_STRING")),
    ("HTML_ENTITY_REFERENCE", Some("DEFAULT_ENTITY")),
];
