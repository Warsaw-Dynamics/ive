use super::Row;

/// IVE ids: nodes and properties are coloured like markup tags and attributes.
pub const ROWS: &[Row] = &[
    ("KDL.NODE", Some("DEFAULT_TAG")),
    ("KDL.PROPERTY", Some("DEFAULT_ATTRIBUTE")),
    ("KDL.TYPE", Some("DEFAULT_CLASS_NAME")),
    ("KDL.BUILTIN_TYPE", Some("DEFAULT_PREDEFINED_SYMBOL")),
    ("KDL.KEYWORD", Some("DEFAULT_KEYWORD")),
    ("KDL.STRING", Some("DEFAULT_STRING")),
    ("KDL.ESCAPE", Some("DEFAULT_VALID_STRING_ESCAPE")),
    ("KDL.NUMBER", Some("DEFAULT_NUMBER")),
    ("KDL.LINE_COMMENT", Some("DEFAULT_LINE_COMMENT")),
    ("KDL.BLOCK_COMMENT", Some("DEFAULT_BLOCK_COMMENT")),
    ("KDL.SLASHDASH_COMMENT", Some("DEFAULT_BLOCK_COMMENT")),
    ("KDL.OPERATOR", Some("DEFAULT_OPERATION_SIGN")),
    ("KDL.BRACES", Some("DEFAULT_BRACES")),
    ("KDL.SEMICOLON", Some("DEFAULT_SEMICOLON")),
];
