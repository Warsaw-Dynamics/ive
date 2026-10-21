use super::Row;

/// Ids and parents from IntelliJ's `YAMLHighlighter`; `HighlighterColors.TEXT` is IVE's `TEXT`.
pub const ROWS: &[Row] = &[
    ("YAML_SCALAR_KEY", Some("DEFAULT_KEYWORD")),
    ("YAML_SCALAR_VALUE", Some("TEXT")),
    ("YAML_SCALAR_STRING", Some("DEFAULT_STRING")),
    ("YAML_SCALAR_DSTRING", Some("DEFAULT_STRING")),
    ("YAML_SCALAR_LIST", Some("TEXT")),
    ("YAML_COMMENT", Some("DEFAULT_DOC_COMMENT")),
    ("YAML_SIGN", Some("DEFAULT_OPERATION_SIGN")),
    ("YAML_ANCHOR", Some("DEFAULT_IDENTIFIER")),
];
