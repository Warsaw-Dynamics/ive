; IVE highlight query for .properties files. Capture names are IntelliJ's colour key ids
; (src/colors/keys/properties.rs). For the same node a later pattern wins; otherwise the innermost node wins.

(comment) @PROPERTIES.LINE_COMMENT
(key) @PROPERTIES.KEY
(value) @PROPERTIES.VALUE
(property ["=" ":"] @PROPERTIES.KEY_VALUE_SEPARATOR)
(escape) @PROPERTIES.VALID_STRING_ESCAPE
((escape) @PROPERTIES.INVALID_STRING_ESCAPE
 (#match? @PROPERTIES.INVALID_STRING_ESCAPE "^\\\\u"))
((escape) @PROPERTIES.VALID_STRING_ESCAPE
 (#match? @PROPERTIES.VALID_STRING_ESCAPE "^\\\\u[0-9a-fA-F]{4}$"))
