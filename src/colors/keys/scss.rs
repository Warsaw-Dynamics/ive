use super::Row;

/// Ids of IntelliJ's Sass plugin (not in intellij-community; the ids its bundled schemes colour, e.g.
/// `SASS_MIXIN`), parented to their CSS counterparts. Selectors, units, colours and punctuation use the CSS keys.
pub const ROWS: &[Row] = &[
    ("SASS_COMMENT", "Comment", Some("CSS.COMMENT")),
    ("SASS_STRING", "String", Some("CSS.STRING")),
    ("SASS_NUMBER", "Number", Some("CSS.NUMBER")),
    ("SASS_KEYWORD", "Keyword", Some("CSS.KEYWORD")),
    ("SASS_EXTEND", "@extend", Some("CSS.KEYWORD")),
    ("SASS_DEFAULT", "!default", Some("CSS.IMPORTANT")),
    ("SASS_IMPORTANT", "!important", Some("CSS.IMPORTANT")),
    ("SASS_IDENTIFIER", "Identifier", Some("CSS.IDENT")),
    ("SASS_TAG_NAME", "Tag name", Some("CSS.TAG_NAME")),
    ("SASS_PROPERTY_NAME", "Property name", Some("CSS.PROPERTY_NAME")),
    ("SASS_PROPERTY_VALUE", "Property value", Some("CSS.PROPERTY_VALUE")),
    ("SASS_VARIABLE", "Variable", Some("CSS.VARIABLE")),
    ("SASS_MIXIN", "Mixin", Some("DEFAULT_FUNCTION_DECLARATION")),
    ("SASS_FUNCTION", "Function", Some("CSS.FUNCTION")),
    ("SASS_URL", "URL", Some("CSS.URL")),
];
