; IVE highlight query for HEEx. Capture names are colour key ids (src/colors/keys/).
; For the same node a later pattern wins; otherwise the innermost node wins.
; The Elixir code in expressions and directives is injected (injections.scm).

(text) @HEEX.TEXT
(comment) @HEEX.COMMENT
(doctype) @HEEX.TAG
[
  (start_tag) (end_tag) (self_closing_tag) (start_component) (end_component) (self_closing_component)
  (start_slot) (end_slot) (self_closing_slot)
] @HEEX.TAG

(tag_name) @HEEX.TAG_NAME
(slot_name) @HEEX.SLOT
(component_name (function) @HEEX.COMPONENT)
(component_name "." @HEEX.COMPONENT)
(component_name (module) @HEEX.COMPONENT_MODULE)

(attribute_name) @HEEX.ATTRIBUTE_NAME
(special_attribute_name) @HEEX.SPECIAL_ATTRIBUTE
[(attribute_value) (quoted_attribute_value)] @HEEX.ATTRIBUTE_VALUE

(expression ["{" "}"] @HEEX.DELIMITER)
(directive ["<%" "<%=" "<%%" "<%%=" "%>"] @HEEX.DELIMITER)
