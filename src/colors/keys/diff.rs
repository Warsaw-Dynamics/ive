use super::Row;

/// IVE ids; added and removed lines take tree-sitter-diff's own string and keyword colours.
pub const ROWS: &[Row] = &[
    ("DIFF.ADDED", "Added line", Some("DEFAULT_STRING")),
    ("DIFF.REMOVED", "Removed line", Some("DEFAULT_KEYWORD")),
    ("DIFF.CONTEXT", "Context line", Some("TEXT")),
    ("DIFF.COMMAND", "Command", Some("DEFAULT_METADATA")),
    ("DIFF.FILE_HEADER", "File header", Some("DEFAULT_METADATA")),
    ("DIFF.HUNK_HEADER", "Hunk header", Some("DEFAULT_METADATA")),
    ("DIFF.LINE_RANGE", "Line range", Some("DEFAULT_NUMBER")),
    ("DIFF.EXTENDED_HEADER", "Extended header", Some("DEFAULT_KEYWORD")),
    ("DIFF.COMMIT", "Commit hash", Some("DEFAULT_CONSTANT")),
    ("DIFF.MODE", "Mode and score", Some("DEFAULT_NUMBER")),
    ("DIFF.FILE_NAME", "File name", Some("DEFAULT_STRING")),
    ("DIFF.COMMENT", "Comment", Some("DEFAULT_LINE_COMMENT")),
];
