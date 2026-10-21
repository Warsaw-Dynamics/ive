use super::Row;

/// Ids of IntelliJ's CSS plugin (not in intellij-community; the ids its bundled schemes colour, e.g. `CSS.COLOR`);
/// parents follow its `CssHighlighter`.
pub const ROWS: &[Row] = &[
    ("CSS.COMMENT", Some("DEFAULT_BLOCK_COMMENT")),
    ("CSS.STRING", Some("DEFAULT_STRING")),
    ("CSS.NUMBER", Some("DEFAULT_NUMBER")),
    ("CSS.KEYWORD", Some("DEFAULT_KEYWORD")),
    ("CSS.IDENT", Some("DEFAULT_IDENTIFIER")),
    ("CSS.TAG_NAME", Some("DEFAULT_KEYWORD")),
    ("CSS.CLASS_NAME", Some("DEFAULT_CLASS_NAME")),
    ("CSS.ID_SELECTOR", Some("DEFAULT_INSTANCE_FIELD")),
    ("CSS.PSEUDO", Some("DEFAULT_KEYWORD")),
    ("CSS.ATTRIBUTE_NAME", Some("DEFAULT_ATTRIBUTE")),
    ("CSS.PROPERTY_NAME", Some("DEFAULT_INSTANCE_FIELD")),
    ("CSS.PROPERTY_VALUE", Some("DEFAULT_STRING")),
    ("CSS.FUNCTION", Some("DEFAULT_FUNCTION_CALL")),
    ("CSS.URL", Some("DEFAULT_STRING")),
    ("CSS.IMPORTANT", Some("DEFAULT_KEYWORD")),
    ("CSS.COLOR", Some("DEFAULT_NUMBER")),
    ("CSS.UNIT", Some("DEFAULT_NUMBER")),
    ("CSS.VARIABLE", Some("DEFAULT_INSTANCE_FIELD")),
    ("CSS.OPERATION_SIGN", Some("DEFAULT_OPERATION_SIGN")),
    ("CSS.BRACES", Some("DEFAULT_BRACES")),
    ("CSS.BRACKETS", Some("DEFAULT_BRACKETS")),
    ("CSS.PARENTHESES", Some("DEFAULT_PARENTHS")),
    ("CSS.COLON", Some("DEFAULT_SEMICOLON")),
    ("CSS.SEMICOLON", Some("DEFAULT_SEMICOLON")),
    ("CSS.COMMA", Some("DEFAULT_COMMA")),
    ("CSS.DOT", Some("DEFAULT_DOT")),
];
