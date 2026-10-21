; IVE highlight query for Vue. Capture names are colour key ids (src/colors/keys/).
; For the same node a later pattern wins; otherwise the innermost node wins.

(comment) @VUE.COMMENT
(doctype) @VUE.TAG
[(start_tag) (end_tag) (self_closing_tag) (erroneous_end_tag)] @VUE.TAG

(tag_name) @VUE.TAG_NAME
(erroneous_end_tag_name) @VUE.TAG_NAME
((tag_name) @VUE.COMPONENT_NAME
 (#match? @VUE.COMPONENT_NAME "^[A-Z]|-"))

(attribute_name) @VUE.ATTRIBUTE_NAME
[(attribute_value) (quoted_attribute_value)] @VUE.ATTRIBUTE_VALUE
; a directive's value is TypeScript: what that layer leaves uncoloured is code, not string
(directive_attribute (quoted_attribute_value (attribute_value) @TEXT))
(directive_attribute (attribute_value) @TEXT)

; v-if, v-bind:title, :title, @click, #default
(directive_name) @VUE.DIRECTIVE
(directive_attribute [":" "@" "#"] @VUE.DIRECTIVE)
(directive_value) @VUE.DIRECTIVE_ARGUMENT
(dynamic_directive_value ["[" "]"] @VUE.DIRECTIVE_ARGUMENT)
(dynamic_directive_inner_value) @VUE.DYNAMIC_ARGUMENT
(directive_modifiers "." @VUE.DIRECTIVE_MODIFIER)
(directive_modifier) @VUE.DIRECTIVE_MODIFIER

(interpolation ["{{" "}}"] @VUE.INTERPOLATION)
