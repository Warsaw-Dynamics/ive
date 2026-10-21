use super::Row;

/// ERB and EJS template tags; the code and the text between them are coloured as Ruby or JavaScript and HTML.
pub const ROWS: &[Row] = &[
    ("ETMPL.DELIMITER", "Template delimiter", Some("DEFAULT_TEMPLATE_LANGUAGE_COLOR")),
    ("ETMPL.OUTPUT_DELIMITER", "Output delimiter", Some("ETMPL.DELIMITER")),
    ("ETMPL.COMMENT", "Comment", Some("DEFAULT_BLOCK_COMMENT")),
];
