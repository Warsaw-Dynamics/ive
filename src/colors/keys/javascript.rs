use super::Row;

/// `JS.REGEXP`, `JS.GLOBAL_FUNCTION`, `JS.INSTANCE_MEMBER_FUNCTION` and `JS.LOCAL_VARIABLE` are ids IntelliJ's
/// schemes use; the rest follow their style.
pub const ROWS: &[Row] = &[
    ("JS.KEYWORD", "Keyword", Some("DEFAULT_KEYWORD")),
    ("JS.NUMBER", "Number", Some("DEFAULT_NUMBER")),
    ("JS.STRING", "String", Some("DEFAULT_STRING")),
    ("JS.VALID_ESCAPE", "Valid escape sequence", Some("DEFAULT_VALID_STRING_ESCAPE")),
    ("JS.REGEXP", "Regular expression", Some("DEFAULT_STRING")),
    ("JS.LINE_COMMENT", "Line comment", Some("DEFAULT_LINE_COMMENT")),
    ("JS.BLOCK_COMMENT", "Block comment", Some("DEFAULT_BLOCK_COMMENT")),
    ("JS.DOC_COMMENT", "JSDoc comment", Some("DEFAULT_DOC_COMMENT")),
    ("JS.OPERATION_SIGN", "Operator sign", Some("DEFAULT_OPERATION_SIGN")),
    ("JS.PARENTHS", "Parentheses", Some("DEFAULT_PARENTHS")),
    ("JS.BRACKETS", "Brackets", Some("DEFAULT_BRACKETS")),
    ("JS.BRACES", "Braces", Some("DEFAULT_BRACES")),
    ("JS.COMMA", "Comma", Some("DEFAULT_COMMA")),
    ("JS.DOT", "Dot", Some("DEFAULT_DOT")),
    ("JS.SEMICOLON", "Semicolon", Some("DEFAULT_SEMICOLON")),
    ("JS.CLASS", "Class", Some("DEFAULT_CLASS_NAME")),
    ("JS.CONSTANT", "Constant", Some("DEFAULT_CONSTANT")),
    ("JS.LOCAL_VARIABLE", "Local variable", Some("DEFAULT_LOCAL_VARIABLE")),
    ("JS.PARAMETER", "Parameter", Some("DEFAULT_PARAMETER")),
    ("JS.INSTANCE_MEMBER_VARIABLE", "Instance member variable", Some("DEFAULT_INSTANCE_FIELD")),
    ("JS.INSTANCE_MEMBER_FUNCTION", "Instance member function", Some("DEFAULT_INSTANCE_METHOD")),
    ("JS.GLOBAL_FUNCTION", "Global function", Some("DEFAULT_FUNCTION_DECLARATION")),
    ("JS.FUNCTION_CALL", "Function call", Some("DEFAULT_FUNCTION_CALL")),
    ("JS.DECORATOR", "Decorator", Some("DEFAULT_METADATA")),
];
