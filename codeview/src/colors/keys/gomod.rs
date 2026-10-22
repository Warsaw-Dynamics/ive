use super::Row;

/// No go.mod keys in intellij-community (GoLand is closed source); ids in IntelliJ's style.
pub const ROWS: &[Row] = &[
    ("GOMOD.KEYWORD", Some("DEFAULT_KEYWORD")),
    ("GOMOD.MODULE_PATH", Some("DEFAULT_IDENTIFIER")),
    ("GOMOD.VERSION", Some("DEFAULT_NUMBER")),
    ("GOMOD.FILE_PATH", Some("DEFAULT_STRING")),
    ("GOMOD.LINE_COMMENT", Some("DEFAULT_LINE_COMMENT")),
    ("GOMOD.OPERATION_SIGN", Some("DEFAULT_OPERATION_SIGN")),
    ("GOMOD.PARENTHESES", Some("DEFAULT_PARENTHS")),
    ("GOMOD.BRACKETS", Some("DEFAULT_BRACKETS")),
    ("GOMOD.COMMA", Some("DEFAULT_COMMA")),
];
