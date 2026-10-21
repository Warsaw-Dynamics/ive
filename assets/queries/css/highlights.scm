; IVE highlight query for CSS. Capture names are colour key ids (src/colors/keys/).
; For the same node a later pattern wins; otherwise the innermost node wins.

["{" "}"] @CSS.BRACES
["(" ")"] @CSS.PARENTHESES
["[" "]"] @CSS.BRACKETS
":" @CSS.COLON
";" @CSS.SEMICOLON
"," @CSS.COMMA
"." @CSS.DOT
["~" ">" "+" "-" "*" "/" "=" "^=" "|=" "~=" "$=" "*="] @CSS.OPERATION_SIGN

[(comment) (js_comment)] @CSS.COMMENT
(string_value) @CSS.STRING
[(integer_value) (float_value)] @CSS.NUMBER
(unit) @CSS.UNIT
(color_value) @CSS.COLOR
(important) @CSS.IMPORTANT

(at_keyword) @CSS.KEYWORD
[
  "@media" "@import" "@charset" "@namespace" "@supports" "@keyframes" "@scope"
  "and" "or" "not" "only" "selector" "of"
] @CSS.KEYWORD
[(to) (from)] @CSS.KEYWORD

[(tag_name) (nesting_selector) (universal_selector)] @CSS.TAG_NAME
(class_selector "." @CSS.CLASS_NAME (class_name) @CSS.CLASS_NAME)
(id_selector "#" @CSS.ID_SELECTOR (id_name) @CSS.ID_SELECTOR)
(pseudo_class_selector ":" @CSS.PSEUDO (class_name) @CSS.PSEUDO)
(pseudo_element_selector "::" @CSS.PSEUDO (tag_name) @CSS.PSEUDO)
(attribute_selector (attribute_name) @CSS.ATTRIBUTE_NAME)
(namespace_name) @CSS.IDENT
(keyframes_name) @CSS.IDENT
(feature_name) @CSS.PROPERTY_NAME

(property_name) @CSS.PROPERTY_NAME
(declaration (plain_value) @CSS.PROPERTY_VALUE)
((property_name) @CSS.VARIABLE
 (#match? @CSS.VARIABLE "^--"))
((plain_value) @CSS.VARIABLE
 (#match? @CSS.VARIABLE "^--"))

(function_name) @CSS.FUNCTION
((call_expression
  (function_name) @_url
  (arguments [(plain_value) (string_value)] @CSS.URL))
 (#eq? @_url "url"))
