use super::Row;

/// No SQL keys in intellij-community (DataGrip is closed source); ids in IntelliJ's style.
pub const ROWS: &[Row] = &[
    ("SQL.KEYWORD", "Keyword", Some("DEFAULT_KEYWORD")),
    ("SQL.BUILTIN_TYPE", "Builtin type", Some("DEFAULT_KEYWORD")),
    ("SQL.NUMBER", "Number", Some("DEFAULT_NUMBER")),
    ("SQL.STRING", "String", Some("DEFAULT_STRING")),
    ("SQL.LINE_COMMENT", "Line comment", Some("DEFAULT_LINE_COMMENT")),
    ("SQL.BLOCK_COMMENT", "Block comment", Some("DEFAULT_BLOCK_COMMENT")),
    ("SQL.OPERATION_SIGN", "Operator sign", Some("DEFAULT_OPERATION_SIGN")),
    ("SQL.PARENTHESES", "Parentheses", Some("DEFAULT_PARENTHS")),
    ("SQL.BRACKETS", "Brackets", Some("DEFAULT_BRACKETS")),
    ("SQL.COMMA", "Comma", Some("DEFAULT_COMMA")),
    ("SQL.DOT", "Dot", Some("DEFAULT_DOT")),
    ("SQL.SEMICOLON", "Semicolon", Some("DEFAULT_SEMICOLON")),
    ("SQL.SCHEMA", "Schema or database", Some("DEFAULT_IDENTIFIER")),
    ("SQL.TABLE", "Table", Some("DEFAULT_CLASS_NAME")),
    ("SQL.TYPE", "Type", Some("DEFAULT_CLASS_NAME")),
    ("SQL.COLUMN", "Column", Some("DEFAULT_INSTANCE_FIELD")),
    ("SQL.ALIAS", "Alias", Some("DEFAULT_LOCAL_VARIABLE")),
    ("SQL.PARAMETER", "Parameter", Some("DEFAULT_PARAMETER")),
    ("SQL.FUNCTION_DECLARATION", "Function declaration", Some("DEFAULT_FUNCTION_DECLARATION")),
    ("SQL.FUNCTION_CALL", "Function call", Some("DEFAULT_FUNCTION_CALL")),
    ("SQL.BUILTIN_FUNCTION", "Builtin function", Some("DEFAULT_PREDEFINED_SYMBOL")),
];
