use super::Row;

/// Ids and parents from IntelliJ's `JsonSyntaxHighlighterFactory`.
pub const ROWS: &[Row] = &[
    ("JSON.BRACKETS", Some("DEFAULT_BRACKETS")),
    ("JSON.BRACES", Some("DEFAULT_BRACES")),
    ("JSON.COMMA", Some("DEFAULT_COMMA")),
    ("JSON.COLON", Some("DEFAULT_SEMICOLON")),
    ("JSON.NUMBER", Some("DEFAULT_NUMBER")),
    ("JSON.STRING", Some("DEFAULT_STRING")),
    ("JSON.KEYWORD", Some("DEFAULT_KEYWORD")),
    ("JSON.LINE_COMMENT", Some("DEFAULT_LINE_COMMENT")),
    ("JSON.BLOCK_COMMENT", Some("DEFAULT_BLOCK_COMMENT")),
    ("JSON.PROPERTY_KEY", Some("DEFAULT_INSTANCE_FIELD")),
    ("JSON.VALID_ESCAPE", Some("DEFAULT_VALID_STRING_ESCAPE")),
];
