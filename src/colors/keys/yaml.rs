use super::Row;

/// Ids and parents from IntelliJ's `YAMLHighlighter`; `HighlighterColors.TEXT` is IVE's `TEXT`.
pub const ROWS: &[Row] = &[
    ("YAML_SCALAR_KEY", "Key", Some("DEFAULT_KEYWORD")),
    ("YAML_SCALAR_VALUE", "Scalar value", Some("TEXT")),
    ("YAML_SCALAR_STRING", "Single-quoted string", Some("DEFAULT_STRING")),
    ("YAML_SCALAR_DSTRING", "Double-quoted string", Some("DEFAULT_STRING")),
    ("YAML_SCALAR_LIST", "Block scalar", Some("TEXT")),
    ("YAML_COMMENT", "Comment", Some("DEFAULT_DOC_COMMENT")),
    ("YAML_SIGN", "Sign", Some("DEFAULT_OPERATION_SIGN")),
    ("YAML_ANCHOR", "Anchor and alias", Some("DEFAULT_IDENTIFIER")),
];
