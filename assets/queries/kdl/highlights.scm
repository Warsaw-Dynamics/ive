; IVE highlight query for KDL. Capture names are colour key ids (src/colors/keys/kdl.rs).
; For the same range a later pattern wins; otherwise the narrowest capture wins.

["{" "}"] @KDL.BRACES
";" @KDL.SEMICOLON
"=" @KDL.OPERATOR

(single_line_comment) @KDL.LINE_COMMENT
(multi_line_comment) @KDL.BLOCK_COMMENT

(string) @KDL.STRING
[(escape) (escaped_whitespace)] @KDL.ESCAPE
(number) @KDL.NUMBER
(keyword) @KDL.KEYWORD

(node name: (identifier) @KDL.NODE)
(node name: (identifier (string) @KDL.NODE))
(prop key: (identifier) @KDL.PROPERTY)
(prop key: (identifier (string) @KDL.PROPERTY))

(type) @KDL.TYPE
; Matched on the whole annotation, not its name, so the slashdash patterns below need one level less.
((type) @KDL.BUILTIN_TYPE
 (#match? @KDL.BUILTIN_TYPE
  "^\\(\\s*(i8|i16|i32|i64|i128|u8|u16|u32|u64|u128|isize|usize|f32|f64|decimal64|decimal128|date-time|time|date|duration|decimal|currency|country-2|country-3|country-subdivision|email|idn-email|hostname|idn-hostname|ipv4|ipv6|url|url-reference|irl|irl-reference|url-template|uuid|regex|base64|base85)\\s*\\)$"))

; Slashdash `/-` comments out a node, an argument or property, or a children block. The narrowest capture wins, so
; what the patterns above capture inside the commented part is captured again here. Queries cannot match "any
; descendant", and this grammar's query compiler takes 1-100 s for any pattern putting `node_field` or `type` under
; `node`, so a commented node is found by its text, and its own properties, arguments and type annotation, and its
; children's, keep their colours.
(version) @KDL.SLASHDASH_COMMENT
[(node_field_comment) (node_children_comment)] @KDL.SLASHDASH_COMMENT
((node) @KDL.SLASHDASH_COMMENT
 (#match? @KDL.SLASHDASH_COMMENT "^/-"))
((node name: (identifier) @KDL.SLASHDASH_COMMENT) @_commented
 (#match? @_commented "^/-"))
((node name: (identifier (string) @KDL.SLASHDASH_COMMENT)) @_commented
 (#match? @_commented "^/-"))
((node children: (node_children ["{" "}" ";"] @KDL.SLASHDASH_COMMENT)) @_commented
 (#match? @_commented "^/-"))
((node children: (node_children (node name: (identifier) @KDL.SLASHDASH_COMMENT))) @_commented
 (#match? @_commented "^/-"))

(node_field_comment (prop key: (identifier) @KDL.SLASHDASH_COMMENT))
(node_field_comment (prop "=" @KDL.SLASHDASH_COMMENT))
(node_field_comment (prop value: (value [(type) (string) (number) (keyword)] @KDL.SLASHDASH_COMMENT)))
(node_field_comment (value [(type) (string) (number) (keyword)] @KDL.SLASHDASH_COMMENT))
(node_field_comment (prop value: (value (string [(escape) (escaped_whitespace)] @KDL.SLASHDASH_COMMENT))))
(node_field_comment (value (string [(escape) (escaped_whitespace)] @KDL.SLASHDASH_COMMENT)))

(node_children_comment (node_children ["{" "}" ";"] @KDL.SLASHDASH_COMMENT))
(node_children_comment (node_children (node name: (identifier) @KDL.SLASHDASH_COMMENT)))
