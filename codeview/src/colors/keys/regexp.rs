use super::Row;

/// Ids and parents from IntelliJ's `RegExpHighlighter`, names from `RegExpColorsPage`.
pub const ROWS: &[Row] = &[
    ("REGEXP.CHARACTER", Some("DEFAULT_STRING")),
    ("REGEXP.META", Some("DEFAULT_KEYWORD")),
    ("REGEXP.ESC_CHARACTER", Some("DEFAULT_VALID_STRING_ESCAPE")),
    ("REGEXP.BRACES", Some("DEFAULT_BRACES")),
    ("REGEXP.BRACKETS", Some("DEFAULT_BRACKETS")),
    ("REGEXP.PARENTHS", Some("DEFAULT_PARENTHS")),
    ("REGEXP.COMMA", Some("DEFAULT_COMMA")),
    ("REGEXP.CHAR_CLASS", Some("DEFAULT_ENTITY")),
    ("REGEXP.QUANTIFIER", Some("DEFAULT_NUMBER")),
    ("REGEXP.DOT", Some("DEFAULT_DOT")),
    ("REGEXP.OPTIONS", Some("DEFAULT_PREDEFINED_SYMBOL")),
    ("REGEXP.NAME", Some("DEFAULT_IDENTIFIER")),
];
