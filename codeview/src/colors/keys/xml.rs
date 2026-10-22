use super::Row;

/// Ids and parents from IntelliJ's `XmlHighlighterColors`, names from `XMLColorsPage`; DTD uses them too.
pub const ROWS: &[Row] = &[
    ("XML_PROLOGUE", Some("TEXT")),
    ("XML_COMMENT", Some("DEFAULT_LINE_COMMENT")),
    ("XML_TAG", Some("DEFAULT_TAG")),
    ("XML_TAG_NAME", Some("DEFAULT_KEYWORD")),
    ("XML_ATTRIBUTE_NAME", Some("DEFAULT_ATTRIBUTE")),
    ("XML_ATTRIBUTE_VALUE", Some("DEFAULT_STRING")),
    ("XML_TAG_DATA", Some("TEXT")),
    ("XML_ENTITY_REFERENCE", Some("DEFAULT_ENTITY")),
];
