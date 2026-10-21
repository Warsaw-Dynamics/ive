; IVE highlight query for JSON. Capture names are colour key ids (src/colors/keys.rs).
; For the same node a later pattern wins; otherwise the innermost node wins.

["[" "]"] @JSON.BRACKETS
["{" "}"] @JSON.BRACES
"," @JSON.COMMA
":" @JSON.COLON
(number) @JSON.NUMBER
(string) @JSON.STRING
(escape_sequence) @JSON.VALID_ESCAPE
[(true) (false) (null)] @JSON.KEYWORD
(comment) @JSON.LINE_COMMENT
((comment) @JSON.BLOCK_COMMENT
 (#match? @JSON.BLOCK_COMMENT "^/\\*"))
(pair key: (string) @JSON.PROPERTY_KEY)
