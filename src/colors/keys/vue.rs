use super::Row;

/// The template's markup and directives; `<script>`, `<style>` and template expressions use their languages' keys.
pub const ROWS: &[Row] = &[
    ("VUE.TAG", "Tag", Some("DEFAULT_TAG")),
    ("VUE.TAG_NAME", "Tag name", Some("DEFAULT_KEYWORD")),
    ("VUE.COMPONENT_NAME", "Component name", Some("VUE.TAG_NAME")),
    ("VUE.ATTRIBUTE_NAME", "Attribute name", Some("DEFAULT_ATTRIBUTE")),
    ("VUE.ATTRIBUTE_VALUE", "Attribute value", Some("DEFAULT_STRING")),
    ("VUE.DIRECTIVE", "Directive", Some("DEFAULT_KEYWORD")),
    ("VUE.DIRECTIVE_ARGUMENT", "Directive argument", Some("DEFAULT_ATTRIBUTE")),
    ("VUE.DYNAMIC_ARGUMENT", "Dynamic argument", Some("DEFAULT_LOCAL_VARIABLE")),
    ("VUE.DIRECTIVE_MODIFIER", "Directive modifier", Some("DEFAULT_METADATA")),
    ("VUE.INTERPOLATION", "Interpolation delimiter", Some("DEFAULT_TEMPLATE_LANGUAGE_COLOR")),
    ("VUE.COMMENT", "Comment", Some("DEFAULT_BLOCK_COMMENT")),
];
