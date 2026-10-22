use super::Row;

/// The template's markup and directives; `<script>`, `<style>` and template expressions use their languages' keys.
pub const ROWS: &[Row] = &[
    ("VUE.TAG", Some("DEFAULT_TAG")),
    ("VUE.TAG_NAME", Some("DEFAULT_KEYWORD")),
    ("VUE.COMPONENT_NAME", Some("VUE.TAG_NAME")),
    ("VUE.ATTRIBUTE_NAME", Some("DEFAULT_ATTRIBUTE")),
    ("VUE.ATTRIBUTE_VALUE", Some("DEFAULT_STRING")),
    ("VUE.DIRECTIVE", Some("DEFAULT_KEYWORD")),
    ("VUE.DIRECTIVE_ARGUMENT", Some("DEFAULT_ATTRIBUTE")),
    ("VUE.DYNAMIC_ARGUMENT", Some("DEFAULT_LOCAL_VARIABLE")),
    ("VUE.DIRECTIVE_MODIFIER", Some("DEFAULT_METADATA")),
    ("VUE.INTERPOLATION", Some("DEFAULT_TEMPLATE_LANGUAGE_COLOR")),
    ("VUE.COMMENT", Some("DEFAULT_BLOCK_COMMENT")),
];
