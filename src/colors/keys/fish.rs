use super::Row;

pub const ROWS: &[Row] = &[
    ("FISH.KEYWORD", "Keyword", Some("DEFAULT_KEYWORD")),
    ("FISH.STRING", "String", Some("DEFAULT_STRING")),
    ("FISH.ESCAPE", "Escape sequence", Some("DEFAULT_VALID_STRING_ESCAPE")),
    ("FISH.NUMBER", "Number", Some("DEFAULT_NUMBER")),
    ("FISH.LINE_COMMENT", "Comment", Some("DEFAULT_LINE_COMMENT")),
    ("FISH.OPERATOR", "Operator", Some("DEFAULT_OPERATION_SIGN")),
    ("FISH.REDIRECTION", "Redirection", Some("DEFAULT_OPERATION_SIGN")),
    ("FISH.PARENTHESES", "Parentheses", Some("DEFAULT_PARENTHS")),
    ("FISH.BRACKETS", "Brackets", Some("DEFAULT_BRACKETS")),
    ("FISH.BRACES", "Braces", Some("DEFAULT_BRACES")),
    ("FISH.COMMA", "Comma", Some("DEFAULT_COMMA")),
    ("FISH.SEMICOLON", "Semicolon", Some("DEFAULT_SEMICOLON")),
    ("FISH.FUNCTION_DECLARATION", "Function declaration", Some("DEFAULT_FUNCTION_DECLARATION")),
    ("FISH.COMMAND", "Command", Some("DEFAULT_FUNCTION_CALL")),
    ("FISH.BUILTIN", "Builtin command", Some("DEFAULT_PREDEFINED_SYMBOL")),
    ("FISH.OPTION", "Option", Some("DEFAULT_PARAMETER")),
    ("FISH.VARIABLE", "Variable", Some("DEFAULT_LOCAL_VARIABLE")),
    ("FISH.VARIABLE_DECLARATION", "Variable declaration", Some("DEFAULT_LOCAL_VARIABLE")),
];
