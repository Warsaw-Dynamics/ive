use super::Row;

pub const ROWS: &[Row] = &[
    ("CMAKE.KEYWORD", "Keyword", Some("DEFAULT_KEYWORD")),
    ("CMAKE.STRING", "String", Some("DEFAULT_STRING")),
    ("CMAKE.BRACKET_ARGUMENT", "Bracket argument", Some("CMAKE.STRING")),
    ("CMAKE.ESCAPE", "Escape sequence", Some("DEFAULT_VALID_STRING_ESCAPE")),
    ("CMAKE.NUMBER", "Number", Some("DEFAULT_NUMBER")),
    ("CMAKE.LINE_COMMENT", "Line comment", Some("DEFAULT_LINE_COMMENT")),
    ("CMAKE.BLOCK_COMMENT", "Bracket comment", Some("DEFAULT_BLOCK_COMMENT")),
    ("CMAKE.PARENTHESES", "Parentheses", Some("DEFAULT_PARENTHS")),
    ("CMAKE.FUNCTION_DECLARATION", "Function or macro declaration", Some("DEFAULT_FUNCTION_DECLARATION")),
    ("CMAKE.FUNCTION_CALL", "Command call", Some("DEFAULT_FUNCTION_CALL")),
    ("CMAKE.BUILTIN_COMMAND", "Builtin command", Some("DEFAULT_PREDEFINED_SYMBOL")),
    ("CMAKE.KEYWORD_ARGUMENT", "Keyword argument", Some("DEFAULT_CONSTANT")),
    ("CMAKE.PARAMETER", "Parameter", Some("DEFAULT_PARAMETER")),
    ("CMAKE.VARIABLE_DECLARATION", "Variable declaration", Some("DEFAULT_LOCAL_VARIABLE")),
    ("CMAKE.VARIABLE", "Variable reference", Some("DEFAULT_LOCAL_VARIABLE")),
];
