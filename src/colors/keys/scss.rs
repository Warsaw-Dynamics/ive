use super::Row;

/// Ids of IntelliJ's Sass plugin (not in intellij-community; the ids its bundled schemes colour, e.g.
/// `SASS_MIXIN`), parented to their CSS counterparts. Selectors, units, colours and punctuation use the CSS keys.
pub const ROWS: &[Row] = &[
    ("SASS_COMMENT", Some("CSS.COMMENT")),
    ("SASS_STRING", Some("CSS.STRING")),
    ("SASS_NUMBER", Some("CSS.NUMBER")),
    ("SASS_KEYWORD", Some("CSS.KEYWORD")),
    ("SASS_EXTEND", Some("CSS.KEYWORD")),
    ("SASS_DEFAULT", Some("CSS.IMPORTANT")),
    ("SASS_IMPORTANT", Some("CSS.IMPORTANT")),
    ("SASS_IDENTIFIER", Some("CSS.IDENT")),
    ("SASS_TAG_NAME", Some("CSS.TAG_NAME")),
    ("SASS_PROPERTY_NAME", Some("CSS.PROPERTY_NAME")),
    ("SASS_PROPERTY_VALUE", Some("CSS.PROPERTY_VALUE")),
    ("SASS_VARIABLE", Some("CSS.VARIABLE")),
    ("SASS_MIXIN", Some("DEFAULT_FUNCTION_DECLARATION")),
    ("SASS_FUNCTION", Some("CSS.FUNCTION")),
    ("SASS_URL", Some("CSS.URL")),
];
