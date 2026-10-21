use super::Row;

/// Ids and parents from IntelliJ's `PropertiesHighlighter`, names from its `PropertiesBundle`.
pub const ROWS: &[Row] = &[
    ("PROPERTIES.KEY", "Property key", Some("DEFAULT_KEYWORD")),
    ("PROPERTIES.VALUE", "Property value", Some("DEFAULT_STRING")),
    ("PROPERTIES.LINE_COMMENT", "Comment", Some("DEFAULT_LINE_COMMENT")),
    ("PROPERTIES.KEY_VALUE_SEPARATOR", "Key/value separator", Some("DEFAULT_OPERATION_SIGN")),
    ("PROPERTIES.VALID_STRING_ESCAPE", "Valid string escape", Some("DEFAULT_VALID_STRING_ESCAPE")),
    ("PROPERTIES.INVALID_STRING_ESCAPE", "Invalid unicode code points", Some("DEFAULT_INVALID_STRING_ESCAPE")),
];
