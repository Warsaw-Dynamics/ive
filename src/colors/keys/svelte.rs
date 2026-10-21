use super::Row;

/// The markup, directives and template blocks; `<script>`, `<style>` and `{expressions}` use their languages' keys.
pub const ROWS: &[Row] = &[
    ("SVELTE.TAG", Some("DEFAULT_TAG")),
    ("SVELTE.TAG_NAME", Some("DEFAULT_KEYWORD")),
    ("SVELTE.COMPONENT_NAME", Some("SVELTE.TAG_NAME")),
    ("SVELTE.ATTRIBUTE_NAME", Some("DEFAULT_ATTRIBUTE")),
    ("SVELTE.ATTRIBUTE_VALUE", Some("DEFAULT_STRING")),
    ("SVELTE.DIRECTIVE", Some("DEFAULT_KEYWORD")),
    ("SVELTE.BLOCK_KEYWORD", Some("DEFAULT_KEYWORD")),
    ("SVELTE.DELIMITER", Some("DEFAULT_TEMPLATE_LANGUAGE_COLOR")),
    ("SVELTE.SNIPPET_NAME", Some("DEFAULT_FUNCTION_DECLARATION")),
    ("SVELTE.COMMENT", Some("DEFAULT_BLOCK_COMMENT")),
    ("SVELTE.ENTITY_REFERENCE", Some("DEFAULT_ENTITY")),
];
