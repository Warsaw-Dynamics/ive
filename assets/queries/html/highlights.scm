; IVE highlight query for HTML. Capture names are colour key ids (src/colors/keys/).
; For the same node a later pattern wins; otherwise the innermost node wins.

(text) @HTML_CODE
(comment) @HTML_COMMENT
(doctype) @HTML_TAG
[(start_tag) (end_tag) (self_closing_tag) (erroneous_end_tag)] @HTML_TAG

(tag_name) @HTML_TAG_NAME
(erroneous_end_tag_name) @HTML_TAG_NAME
((tag_name) @HTML_CUSTOM_TAG_NAME
 (#match? @HTML_CUSTOM_TAG_NAME "-"))

(attribute_name) @HTML_ATTRIBUTE_NAME
[(attribute_value) (quoted_attribute_value)] @HTML_ATTRIBUTE_VALUE
(entity) @HTML_ENTITY_REFERENCE
