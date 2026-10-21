use super::Row;

/// No Haskell keys in intellij-community; ids in IntelliJ's style.
pub const ROWS: &[Row] = &[
    ("HASKELL.KEYWORD", "Keyword", Some("DEFAULT_KEYWORD")),
    ("HASKELL.NUMBER", "Number", Some("DEFAULT_NUMBER")),
    ("HASKELL.STRING", "String", Some("DEFAULT_STRING")),
    ("HASKELL.CHAR", "Char", Some("DEFAULT_STRING")),
    ("HASKELL.LINE_COMMENT", "Line comment", Some("DEFAULT_LINE_COMMENT")),
    ("HASKELL.BLOCK_COMMENT", "Block comment", Some("DEFAULT_BLOCK_COMMENT")),
    ("HASKELL.DOC_COMMENT", "Haddock comment", Some("DEFAULT_DOC_COMMENT")),
    ("HASKELL.PRAGMA", "Pragma", Some("DEFAULT_METADATA")),
    ("HASKELL.OPERATION_SIGN", "Operator sign", Some("DEFAULT_OPERATION_SIGN")),
    ("HASKELL.PARENTHESES", "Parentheses", Some("DEFAULT_PARENTHS")),
    ("HASKELL.BRACKETS", "Brackets", Some("DEFAULT_BRACKETS")),
    ("HASKELL.BRACES", "Braces", Some("DEFAULT_BRACES")),
    ("HASKELL.COMMA", "Comma", Some("DEFAULT_COMMA")),
    ("HASKELL.MODULE", "Module", Some("DEFAULT_IDENTIFIER")),
    ("HASKELL.TYPE", "Type", Some("DEFAULT_CLASS_NAME")),
    ("HASKELL.TYPE_CLASS", "Type class", Some("DEFAULT_INTERFACE_NAME")),
    ("HASKELL.TYPE_VARIABLE", "Type variable", Some("DEFAULT_PARAMETER")),
    ("HASKELL.CONSTRUCTOR", "Data constructor", Some("DEFAULT_STATIC_FIELD")),
    ("HASKELL.FIELD", "Record field", Some("DEFAULT_INSTANCE_FIELD")),
    ("HASKELL.TYPE_SIGNATURE", "Type signature", Some("DEFAULT_FUNCTION_DECLARATION")),
    ("HASKELL.FUNCTION_DECLARATION", "Function declaration", Some("DEFAULT_FUNCTION_DECLARATION")),
    ("HASKELL.FUNCTION_CALL", "Function call", Some("DEFAULT_FUNCTION_CALL")),
    ("HASKELL.PARAMETER", "Parameter", Some("DEFAULT_PARAMETER")),
    ("HASKELL.LOCAL_VARIABLE", "Local binding", Some("DEFAULT_LOCAL_VARIABLE")),
];
