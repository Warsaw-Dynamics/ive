use super::Row;

/// Ids and parents from IntelliJ's `TomlColors`.
pub const ROWS: &[Row] = &[
    ("org.toml.KEY", Some("DEFAULT_KEYWORD")),
    ("org.toml.COMMENT", Some("DEFAULT_LINE_COMMENT")),
    ("org.toml.BOOLEAN", Some("DEFAULT_PREDEFINED_SYMBOL")),
    ("org.toml.NUMBER", Some("DEFAULT_NUMBER")),
    ("org.toml.DATE", Some("DEFAULT_PREDEFINED_SYMBOL")),
    ("org.toml.STRING", Some("DEFAULT_STRING")),
    ("org.toml.VALID_STRING_ESCAPE", Some("DEFAULT_VALID_STRING_ESCAPE")),
];
