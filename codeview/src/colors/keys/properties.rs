use super::Row;

/// Ids and parents from IntelliJ's `PropertiesHighlighter`, names from its `PropertiesBundle`.
pub const ROWS: &[Row] = &[
    ("PROPERTIES.KEY", Some("DEFAULT_KEYWORD")),
    ("PROPERTIES.VALUE", Some("DEFAULT_STRING")),
    ("PROPERTIES.LINE_COMMENT", Some("DEFAULT_LINE_COMMENT")),
    ("PROPERTIES.KEY_VALUE_SEPARATOR", Some("DEFAULT_OPERATION_SIGN")),
    ("PROPERTIES.VALID_STRING_ESCAPE", Some("DEFAULT_VALID_STRING_ESCAPE")),
    ("PROPERTIES.INVALID_STRING_ESCAPE", Some("DEFAULT_INVALID_STRING_ESCAPE")),
];
