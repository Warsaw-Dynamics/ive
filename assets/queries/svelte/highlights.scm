; IVE highlight query for Svelte. Capture names are colour key ids (src/colors/keys/).
; For the same node a later pattern wins; otherwise the innermost node wins.

(comment) @SVELTE.COMMENT
(doctype) @SVELTE.TAG
[(start_tag) (end_tag) (self_closing_tag) (erroneous_end_tag)] @SVELTE.TAG

(tag_name) @SVELTE.TAG_NAME
(erroneous_end_tag_name) @SVELTE.TAG_NAME
; <Counter>, <ui.Button>, <svelte:window>, <custom-element>
((tag_name) @SVELTE.COMPONENT_NAME
 (#match? @SVELTE.COMPONENT_NAME "^[A-Z]|[-.:]"))

(attribute_name) @SVELTE.ATTRIBUTE_NAME
((attribute_name) @SVELTE.DIRECTIVE
 (#match? @SVELTE.DIRECTIVE "^(on|bind|class|style|use|transition|in|out|animate|let):"))
[(attribute_value) (quoted_attribute_value)] @SVELTE.ATTRIBUTE_VALUE
(entity) @SVELTE.ENTITY_REFERENCE

; {#if}, {:else}, {/if}, {@html}, {expression}
["{" "}" "#" ":" "/" "@"] @SVELTE.DELIMITER
["if" "else" "else if" "each" "as" "await" "then" "catch" "key" "snippet" "html" "render" "const" "debug"]
  @SVELTE.BLOCK_KEYWORD
(snippet_name) @SVELTE.SNIPPET_NAME

; an expression is TypeScript: what that layer leaves uncoloured is code, not markup or string
(svelte_raw_text) @TEXT
