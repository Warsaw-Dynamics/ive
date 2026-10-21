use super::Row;

/// IVE ids for JSDoc, listed on the JavaScript page.
pub const ROWS: &[Row] = &[
    ("JSDOC.TAG", "JSDoc tag", Some("DEFAULT_DOC_COMMENT_TAG")),
    ("JSDOC.TYPE", "JSDoc type", Some("DEFAULT_DOC_COMMENT_TAG_VALUE")),
    ("JSDOC.PARAMETER", "JSDoc parameter name", Some("DEFAULT_DOC_COMMENT_TAG_VALUE")),
    ("JSDOC.INLINE_TAG", "JSDoc inline tag", Some("DEFAULT_DOC_MARKUP")),
];
