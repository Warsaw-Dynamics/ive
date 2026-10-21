use super::Row;

/// Ids and parents from IntelliJ's `ShHighlighterColors`, names from its `ShColorPage` (`ShBundle`). `BASH.VAR_USE`'s
/// IntelliJ parent `DEFAULT_GLOBAL_VARIABLE` falls back to `DEFAULT_IDENTIFIER`, which IVE uses directly.
pub const ROWS: &[Row] = &[
    ("BASH.KEYWORD", "Keyword", Some("DEFAULT_KEYWORD")),
    ("BASH.NUMBER", "Number", Some("DEFAULT_NUMBER")),
    ("BASH.STRING", "String", Some("DEFAULT_STRING")),
    ("BASH.RAW_STRING", "Raw string", Some("DEFAULT_STRING")),
    ("BASH.LINE_COMMENT", "Line comment", Some("DEFAULT_LINE_COMMENT")),
    ("BASH.SHEBANG", "Shebang comment", Some("BASH.LINE_COMMENT")),
    ("BASH.VAR_USE", "Variable usage", Some("DEFAULT_IDENTIFIER")),
    ("BASH.VAR_DEF", "Variable declaration", Some("BASH.VAR_USE")),
    ("BASH.VAR_USE_COMPOSED", "Composed variable", Some("BASH.VAR_USE")),
    ("BASH.HERE_DOC", "Here documents", Some("DEFAULT_TEMPLATE_LANGUAGE_COLOR")),
    ("BASH.HERE_DOC_START", "Here documents start", Some("DEFAULT_KEYWORD")),
    ("BASH.HERE_DOC_END", "Here documents end", Some("DEFAULT_KEYWORD")),
    ("BASH.PAREN", "Parentheses", Some("DEFAULT_PARENTHS")),
    ("BASH.BRACE", "Curly brackets", Some("DEFAULT_BRACES")),
    ("BASH.BRACKET", "Square brackets", Some("DEFAULT_BRACKETS")),
    ("BASH.BACKQUOTE", "Backquotes", Some("BASH.EXTERNAL_COMMAND")),
    ("BASH.REDIRECTION", "Redirection", Some("DEFAULT_OPERATION_SIGN")),
    ("BASH.EXTERNAL_COMMAND", "Generic command", Some("DEFAULT_IDENTIFIER")),
    ("BASH.SUBSHELL_COMMAND", "Subshell command", Some("BASH.EXTERNAL_COMMAND")),
    ("BASH.CONDITIONAL", "Conditional operators", Some("DEFAULT_KEYWORD")),
    ("BASH.FUNCTION_DEF_NAME", "Function declaration", Some("DEFAULT_FUNCTION_DECLARATION")),
];
