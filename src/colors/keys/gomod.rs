use super::Row;

/// No go.mod keys in intellij-community (GoLand is closed source); ids in IntelliJ's style.
pub const ROWS: &[Row] = &[
    ("GOMOD.KEYWORD", "Directive", Some("DEFAULT_KEYWORD")),
    ("GOMOD.MODULE_PATH", "Module path", Some("DEFAULT_IDENTIFIER")),
    ("GOMOD.VERSION", "Version", Some("DEFAULT_NUMBER")),
    ("GOMOD.FILE_PATH", "File path", Some("DEFAULT_STRING")),
    ("GOMOD.LINE_COMMENT", "Line comment", Some("DEFAULT_LINE_COMMENT")),
    ("GOMOD.OPERATION_SIGN", "Replacement arrow", Some("DEFAULT_OPERATION_SIGN")),
    ("GOMOD.PARENTHESES", "Parentheses", Some("DEFAULT_PARENTHS")),
    ("GOMOD.BRACKETS", "Brackets", Some("DEFAULT_BRACKETS")),
    ("GOMOD.COMMA", "Comma", Some("DEFAULT_COMMA")),
];
