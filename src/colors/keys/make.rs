use super::Row;

pub const ROWS: &[Row] = &[
    ("MAKE.KEYWORD", "Directive", Some("DEFAULT_KEYWORD")),
    ("MAKE.STRING", "String", Some("DEFAULT_STRING")),
    ("MAKE.LINE_COMMENT", "Comment", Some("DEFAULT_LINE_COMMENT")),
    ("MAKE.OPERATOR", "Operator", Some("DEFAULT_OPERATION_SIGN")),
    ("MAKE.PARENTHESES", "Parentheses", Some("DEFAULT_PARENTHS")),
    ("MAKE.COMMA", "Comma", Some("DEFAULT_COMMA")),
    ("MAKE.TARGET", "Target", Some("DEFAULT_FUNCTION_DECLARATION")),
    ("MAKE.SPECIAL_TARGET", "Special target", Some("DEFAULT_PREDEFINED_SYMBOL")),
    ("MAKE.PREREQUISITE", "Prerequisite", Some("DEFAULT_IDENTIFIER")),
    ("MAKE.VARIABLE_DECLARATION", "Variable declaration", Some("DEFAULT_STATIC_FIELD")),
    ("MAKE.VARIABLE", "Variable reference", Some("DEFAULT_STATIC_FIELD")),
    ("MAKE.AUTOMATIC_VARIABLE", "Automatic variable", Some("DEFAULT_PREDEFINED_SYMBOL")),
    ("MAKE.VALUE", "Variable value", Some("DEFAULT_STRING")),
    ("MAKE.FUNCTION", "Function", Some("DEFAULT_FUNCTION_CALL")),
];
