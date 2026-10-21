use super::Row;

/// IVE ids: nodes and properties are coloured like markup tags and attributes.
pub const ROWS: &[Row] = &[
    ("KDL.NODE", "Node name", Some("DEFAULT_TAG")),
    ("KDL.PROPERTY", "Property key", Some("DEFAULT_ATTRIBUTE")),
    ("KDL.TYPE", "Type annotation", Some("DEFAULT_CLASS_NAME")),
    ("KDL.BUILTIN_TYPE", "Reserved type annotation", Some("DEFAULT_PREDEFINED_SYMBOL")),
    ("KDL.KEYWORD", "Keyword", Some("DEFAULT_KEYWORD")),
    ("KDL.STRING", "String", Some("DEFAULT_STRING")),
    ("KDL.ESCAPE", "Escape sequence", Some("DEFAULT_VALID_STRING_ESCAPE")),
    ("KDL.NUMBER", "Number", Some("DEFAULT_NUMBER")),
    ("KDL.LINE_COMMENT", "Line comment", Some("DEFAULT_LINE_COMMENT")),
    ("KDL.BLOCK_COMMENT", "Block comment", Some("DEFAULT_BLOCK_COMMENT")),
    ("KDL.SLASHDASH_COMMENT", "Slashdash comment", Some("DEFAULT_BLOCK_COMMENT")),
    ("KDL.OPERATOR", "Operator", Some("DEFAULT_OPERATION_SIGN")),
    ("KDL.BRACES", "Braces", Some("DEFAULT_BRACES")),
    ("KDL.SEMICOLON", "Semicolon", Some("DEFAULT_SEMICOLON")),
];
