use super::Row;

/// Ids of IntelliJ's CSS plugin (not in intellij-community; the ids its bundled schemes colour, e.g. `CSS.COLOR`);
/// parents follow its `CssHighlighter`.
pub const ROWS: &[Row] = &[
    ("CSS.COMMENT", "Comment", Some("DEFAULT_BLOCK_COMMENT")),
    ("CSS.STRING", "String", Some("DEFAULT_STRING")),
    ("CSS.NUMBER", "Number", Some("DEFAULT_NUMBER")),
    ("CSS.KEYWORD", "Keyword", Some("DEFAULT_KEYWORD")),
    ("CSS.IDENT", "Identifier", Some("DEFAULT_IDENTIFIER")),
    ("CSS.TAG_NAME", "Tag name", Some("DEFAULT_KEYWORD")),
    ("CSS.CLASS_NAME", "Class name", Some("DEFAULT_CLASS_NAME")),
    ("CSS.ID_SELECTOR", "Id selector", Some("DEFAULT_INSTANCE_FIELD")),
    ("CSS.PSEUDO", "Pseudo selector", Some("DEFAULT_KEYWORD")),
    ("CSS.ATTRIBUTE_NAME", "Attribute name", Some("DEFAULT_ATTRIBUTE")),
    ("CSS.PROPERTY_NAME", "Property name", Some("DEFAULT_INSTANCE_FIELD")),
    ("CSS.PROPERTY_VALUE", "Property value", Some("DEFAULT_STRING")),
    ("CSS.FUNCTION", "Function", Some("DEFAULT_FUNCTION_CALL")),
    ("CSS.URL", "URL", Some("DEFAULT_STRING")),
    ("CSS.IMPORTANT", "!important", Some("DEFAULT_KEYWORD")),
    ("CSS.COLOR", "Color", Some("DEFAULT_NUMBER")),
    ("CSS.UNIT", "Unit", Some("DEFAULT_NUMBER")),
    ("CSS.VARIABLE", "Custom property", Some("DEFAULT_INSTANCE_FIELD")),
    ("CSS.OPERATION_SIGN", "Operator sign", Some("DEFAULT_OPERATION_SIGN")),
    ("CSS.BRACES", "Braces", Some("DEFAULT_BRACES")),
    ("CSS.BRACKETS", "Brackets", Some("DEFAULT_BRACKETS")),
    ("CSS.PARENTHESES", "Parentheses", Some("DEFAULT_PARENTHS")),
    ("CSS.COLON", "Colon", Some("DEFAULT_SEMICOLON")),
    ("CSS.SEMICOLON", "Semicolon", Some("DEFAULT_SEMICOLON")),
    ("CSS.COMMA", "Comma", Some("DEFAULT_COMMA")),
    ("CSS.DOT", "Dot", Some("DEFAULT_DOT")),
];
