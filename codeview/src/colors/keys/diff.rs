use super::Row;

/// IVE ids; added and removed lines take tree-sitter-diff's own string and keyword colours.
pub const ROWS: &[Row] = &[
    ("DIFF.ADDED", Some("DEFAULT_STRING")),
    ("DIFF.REMOVED", Some("DEFAULT_KEYWORD")),
    ("DIFF.CONTEXT", Some("TEXT")),
    ("DIFF.COMMAND", Some("DEFAULT_METADATA")),
    ("DIFF.FILE_HEADER", Some("DEFAULT_METADATA")),
    ("DIFF.HUNK_HEADER", Some("DEFAULT_METADATA")),
    ("DIFF.LINE_RANGE", Some("DEFAULT_NUMBER")),
    ("DIFF.EXTENDED_HEADER", Some("DEFAULT_KEYWORD")),
    ("DIFF.COMMIT", Some("DEFAULT_CONSTANT")),
    ("DIFF.MODE", Some("DEFAULT_NUMBER")),
    ("DIFF.FILE_NAME", Some("DEFAULT_STRING")),
    ("DIFF.COMMENT", Some("DEFAULT_LINE_COMMENT")),
];
