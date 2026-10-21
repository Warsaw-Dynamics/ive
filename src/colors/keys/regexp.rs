use super::Row;

/// Ids and parents from IntelliJ's `RegExpHighlighter`, names from `RegExpColorsPage`.
pub const ROWS: &[Row] = &[
    ("REGEXP.CHARACTER", "Plain character", Some("DEFAULT_STRING")),
    ("REGEXP.META", "Operator character", Some("DEFAULT_KEYWORD")),
    ("REGEXP.ESC_CHARACTER", "Escaped character", Some("DEFAULT_VALID_STRING_ESCAPE")),
    ("REGEXP.BRACES", "Brace", Some("DEFAULT_BRACES")),
    ("REGEXP.BRACKETS", "Bracket", Some("DEFAULT_BRACKETS")),
    ("REGEXP.PARENTHS", "Parenthesis", Some("DEFAULT_PARENTHS")),
    ("REGEXP.COMMA", "Comma", Some("DEFAULT_COMMA")),
    ("REGEXP.CHAR_CLASS", "Character class", Some("DEFAULT_ENTITY")),
    ("REGEXP.QUANTIFIER", "Quantifier", Some("DEFAULT_NUMBER")),
    ("REGEXP.DOT", "Dot", Some("DEFAULT_DOT")),
    ("REGEXP.OPTIONS", "Inline option", Some("DEFAULT_PREDEFINED_SYMBOL")),
    ("REGEXP.NAME", "Name", Some("DEFAULT_IDENTIFIER")),
];
