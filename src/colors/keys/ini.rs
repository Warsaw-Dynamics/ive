use super::Row;

/// IVE ids; parents follow IntelliJ's Properties and TOML, whose keys are keywords and values strings.
pub const ROWS: &[Row] = &[
    ("INI.SECTION", "Section", Some("DEFAULT_CLASS_NAME")),
    ("INI.KEY", "Key", Some("DEFAULT_KEYWORD")),
    ("INI.VALUE", "Value", Some("DEFAULT_STRING")),
    ("INI.NUMBER", "Number", Some("DEFAULT_NUMBER")),
    ("INI.BOOLEAN", "Boolean", Some("DEFAULT_PREDEFINED_SYMBOL")),
    ("INI.COMMENT", "Comment", Some("DEFAULT_LINE_COMMENT")),
    ("INI.OPERATION_SIGN", "Operator sign", Some("DEFAULT_OPERATION_SIGN")),
    ("INI.BRACKETS", "Brackets", Some("DEFAULT_BRACKETS")),
];
