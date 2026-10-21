; IVE highlight query for JSDoc, embedded in JavaScript and TypeScript `/** */` comments. Capture names are colour
; key ids (src/colors/keys/). Text left uncaptured shows in the host's doc comment colour.
; For the same node a later pattern wins; otherwise the innermost node wins.

(tag (tag_name) @JSDOC.TAG)
(tag ["{" "}"] @JSDOC.TYPE)
(type) @JSDOC.TYPE
(tag [(identifier) (optional_identifier) (member_expression) (array_expression)] @JSDOC.PARAMETER)
(tag (array_expression ["[" "]"] @JSDOC.PARAMETER))

(inline_tag) @JSDOC.INLINE_TAG
