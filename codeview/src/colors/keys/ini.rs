use super::Row;

/// IVE ids; parents follow IntelliJ's Properties and TOML, whose keys are keywords and values strings.
pub const ROWS: &[Row] = &[
    ("INI.SECTION", Some("DEFAULT_CLASS_NAME")),
    ("INI.KEY", Some("DEFAULT_KEYWORD")),
    ("INI.VALUE", Some("DEFAULT_STRING")),
    ("INI.NUMBER", Some("DEFAULT_NUMBER")),
    ("INI.BOOLEAN", Some("DEFAULT_PREDEFINED_SYMBOL")),
    ("INI.COMMENT", Some("DEFAULT_LINE_COMMENT")),
    ("INI.OPERATION_SIGN", Some("DEFAULT_OPERATION_SIGN")),
    ("INI.BRACKETS", Some("DEFAULT_BRACKETS")),
];
