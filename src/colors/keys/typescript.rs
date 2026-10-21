use super::Row;

/// Only what TypeScript adds; TypeScript files use the JavaScript keys too.
pub const ROWS: &[Row] = &[
    ("TS.TYPE", "Type reference", Some("DEFAULT_CLASS_NAME")),
    ("TS.PRIMITIVE_TYPE", "Primitive type", Some("DEFAULT_KEYWORD")),
    ("TS.INTERFACE", "Interface", Some("DEFAULT_INTERFACE_NAME")),
    ("TS.TYPE_ALIAS", "Type alias", Some("DEFAULT_CLASS_NAME")),
    ("TS.ENUM", "Enum", Some("DEFAULT_CLASS_NAME")),
    ("TS.TYPE_PARAMETER", "Type parameter", Some("DEFAULT_PARAMETER")),
];
