use super::Row;

/// Ids and parents from IntelliJ's `TomlColors`.
pub const ROWS: &[Row] = &[
    ("org.toml.KEY", "Keys", Some("DEFAULT_KEYWORD")),
    ("org.toml.COMMENT", "Comments", Some("DEFAULT_LINE_COMMENT")),
    ("org.toml.BOOLEAN", "Boolean", Some("DEFAULT_PREDEFINED_SYMBOL")),
    ("org.toml.NUMBER", "Number", Some("DEFAULT_NUMBER")),
    ("org.toml.DATE", "Date", Some("DEFAULT_PREDEFINED_SYMBOL")),
    ("org.toml.STRING", "String", Some("DEFAULT_STRING")),
    ("org.toml.VALID_STRING_ESCAPE", "Valid escape sequence", Some("DEFAULT_VALID_STRING_ESCAPE")),
];
