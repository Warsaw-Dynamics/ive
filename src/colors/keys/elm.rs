use super::Row;

/// No Elm keys in intellij-community; ids in IntelliJ's style.
pub const ROWS: &[Row] = &[
    ("ELM.KEYWORD", "Keyword", Some("DEFAULT_KEYWORD")),
    ("ELM.NUMBER", "Number", Some("DEFAULT_NUMBER")),
    ("ELM.STRING", "String", Some("DEFAULT_STRING")),
    ("ELM.CHAR", "Char", Some("DEFAULT_STRING")),
    ("ELM.ESCAPE", "Escape sequence", Some("DEFAULT_VALID_STRING_ESCAPE")),
    ("ELM.LINE_COMMENT", "Line comment", Some("DEFAULT_LINE_COMMENT")),
    ("ELM.BLOCK_COMMENT", "Block comment", Some("DEFAULT_BLOCK_COMMENT")),
    ("ELM.DOC_COMMENT", "Doc comment", Some("DEFAULT_DOC_COMMENT")),
    ("ELM.OPERATION_SIGN", "Operator sign", Some("DEFAULT_OPERATION_SIGN")),
    ("ELM.PARENTHESES", "Parentheses", Some("DEFAULT_PARENTHS")),
    ("ELM.BRACKETS", "Brackets", Some("DEFAULT_BRACKETS")),
    ("ELM.BRACES", "Braces", Some("DEFAULT_BRACES")),
    ("ELM.COMMA", "Comma", Some("DEFAULT_COMMA")),
    ("ELM.DOT", "Dot", Some("DEFAULT_DOT")),
    ("ELM.MODULE", "Module", Some("DEFAULT_IDENTIFIER")),
    ("ELM.TYPE", "Type", Some("DEFAULT_CLASS_NAME")),
    ("ELM.TYPE_VARIABLE", "Type variable", Some("DEFAULT_PARAMETER")),
    ("ELM.CONSTRUCTOR", "Variant constructor", Some("DEFAULT_STATIC_FIELD")),
    ("ELM.FIELD", "Record field", Some("DEFAULT_INSTANCE_FIELD")),
    ("ELM.TYPE_ANNOTATION", "Type annotation", Some("DEFAULT_FUNCTION_DECLARATION")),
    ("ELM.FUNCTION_DECLARATION", "Function declaration", Some("DEFAULT_FUNCTION_DECLARATION")),
    ("ELM.FUNCTION_CALL", "Function call", Some("DEFAULT_FUNCTION_CALL")),
    ("ELM.PARAMETER", "Parameter", Some("DEFAULT_PARAMETER")),
    ("ELM.LOCAL_VARIABLE", "Local binding", Some("DEFAULT_LOCAL_VARIABLE")),
];
