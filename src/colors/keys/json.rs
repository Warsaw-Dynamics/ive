use super::Row;

/// Ids and parents from IntelliJ's `JsonSyntaxHighlighterFactory`.
pub const ROWS: &[Row] = &[
    ("JSON.BRACKETS", "Brackets", Some("DEFAULT_BRACKETS")),
    ("JSON.BRACES", "Braces", Some("DEFAULT_BRACES")),
    ("JSON.COMMA", "Comma", Some("DEFAULT_COMMA")),
    ("JSON.COLON", "Colon", Some("DEFAULT_SEMICOLON")),
    ("JSON.NUMBER", "Number", Some("DEFAULT_NUMBER")),
    ("JSON.STRING", "String", Some("DEFAULT_STRING")),
    ("JSON.KEYWORD", "Keyword", Some("DEFAULT_KEYWORD")),
    ("JSON.LINE_COMMENT", "Line comment", Some("DEFAULT_LINE_COMMENT")),
    ("JSON.BLOCK_COMMENT", "Block comment", Some("DEFAULT_BLOCK_COMMENT")),
    ("JSON.PROPERTY_KEY", "Property key", Some("DEFAULT_INSTANCE_FIELD")),
    ("JSON.VALID_ESCAPE", "Valid escape sequence", Some("DEFAULT_VALID_STRING_ESCAPE")),
];
