use super::Row;

/// No HCL keys in intellij-community (the Terraform plugin is separate); ids in IntelliJ's style.
pub const ROWS: &[Row] = &[
    ("HCL.KEYWORD", "Keyword", Some("DEFAULT_KEYWORD")),
    ("HCL.NUMBER", "Number", Some("DEFAULT_NUMBER")),
    ("HCL.STRING", "String", Some("DEFAULT_STRING")),
    ("HCL.INTERPOLATION", "Interpolation braces", Some("DEFAULT_VALID_STRING_ESCAPE")),
    ("HCL.HEREDOC_DELIMITER", "Heredoc delimiter", Some("DEFAULT_KEYWORD")),
    ("HCL.LINE_COMMENT", "Line comment", Some("DEFAULT_LINE_COMMENT")),
    ("HCL.BLOCK_COMMENT", "Block comment", Some("DEFAULT_BLOCK_COMMENT")),
    ("HCL.OPERATION_SIGN", "Operator sign", Some("DEFAULT_OPERATION_SIGN")),
    ("HCL.PARENTHESES", "Parentheses", Some("DEFAULT_PARENTHS")),
    ("HCL.BRACKETS", "Brackets", Some("DEFAULT_BRACKETS")),
    ("HCL.BRACES", "Braces", Some("DEFAULT_BRACES")),
    ("HCL.COMMA", "Comma", Some("DEFAULT_COMMA")),
    ("HCL.DOT", "Dot", Some("DEFAULT_DOT")),
    ("HCL.BLOCK_TYPE", "Block type", Some("DEFAULT_KEYWORD")),
    ("HCL.BLOCK_LABEL", "Block label", Some("DEFAULT_CLASS_NAME")),
    ("HCL.ATTRIBUTE", "Attribute or object key", Some("DEFAULT_INSTANCE_FIELD")),
    ("HCL.PROPERTY", "Property access", Some("DEFAULT_INSTANCE_FIELD")),
    ("HCL.FUNCTION_CALL", "Function call", Some("DEFAULT_FUNCTION_CALL")),
    ("HCL.PREDEFINED", "Predefined object (var, local, each...)", Some("DEFAULT_PREDEFINED_SYMBOL")),
    ("HCL.VARIABLE", "Input variable or local value", Some("DEFAULT_STATIC_FIELD")),
    ("HCL.LOCAL_VARIABLE", "Reference or loop variable", Some("DEFAULT_LOCAL_VARIABLE")),
];
