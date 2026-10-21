use super::Row;

/// The markup, directives and template blocks; `<script>`, `<style>` and `{expressions}` use their languages' keys.
pub const ROWS: &[Row] = &[
    ("SVELTE.TAG", "Tag", Some("DEFAULT_TAG")),
    ("SVELTE.TAG_NAME", "Tag name", Some("DEFAULT_KEYWORD")),
    ("SVELTE.COMPONENT_NAME", "Component name", Some("SVELTE.TAG_NAME")),
    ("SVELTE.ATTRIBUTE_NAME", "Attribute name", Some("DEFAULT_ATTRIBUTE")),
    ("SVELTE.ATTRIBUTE_VALUE", "Attribute value", Some("DEFAULT_STRING")),
    ("SVELTE.DIRECTIVE", "Directive", Some("DEFAULT_KEYWORD")),
    ("SVELTE.BLOCK_KEYWORD", "Block keyword", Some("DEFAULT_KEYWORD")),
    ("SVELTE.DELIMITER", "Template delimiter", Some("DEFAULT_TEMPLATE_LANGUAGE_COLOR")),
    ("SVELTE.SNIPPET_NAME", "Snippet name", Some("DEFAULT_FUNCTION_DECLARATION")),
    ("SVELTE.COMMENT", "Comment", Some("DEFAULT_BLOCK_COMMENT")),
    ("SVELTE.ENTITY_REFERENCE", "Entity reference", Some("DEFAULT_ENTITY")),
];
