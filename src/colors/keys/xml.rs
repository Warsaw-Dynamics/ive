use super::Row;

/// Ids and parents from IntelliJ's `XmlHighlighterColors`, names from `XMLColorsPage`; DTD uses them too.
pub const ROWS: &[Row] = &[
    ("XML_PROLOGUE", "Prologue", Some("TEXT")),
    ("XML_COMMENT", "Comment", Some("DEFAULT_LINE_COMMENT")),
    ("XML_TAG", "Tag", Some("DEFAULT_TAG")),
    ("XML_TAG_NAME", "Tag name", Some("DEFAULT_KEYWORD")),
    ("XML_ATTRIBUTE_NAME", "Attribute name", Some("DEFAULT_ATTRIBUTE")),
    ("XML_ATTRIBUTE_VALUE", "Attribute value", Some("DEFAULT_STRING")),
    ("XML_TAG_DATA", "Tag data", Some("TEXT")),
    ("XML_ENTITY_REFERENCE", "Entity reference", Some("DEFAULT_ENTITY")),
];
