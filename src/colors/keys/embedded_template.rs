use super::Row;

/// ERB and EJS template tags; the code and the text between them are coloured as Ruby or JavaScript and HTML.
pub const ROWS: &[Row] = &[
    ("ETMPL.DELIMITER", Some("DEFAULT_TEMPLATE_LANGUAGE_COLOR")),
    ("ETMPL.OUTPUT_DELIMITER", Some("ETMPL.DELIMITER")),
    ("ETMPL.COMMENT", Some("DEFAULT_BLOCK_COMMENT")),
];
