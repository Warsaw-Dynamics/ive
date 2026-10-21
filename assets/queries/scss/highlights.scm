; IVE highlight query for SCSS. Capture names are colour key ids (src/colors/keys/): Sass's own keys where IntelliJ's
; Sass plugin has them, CSS keys for selectors, units, colours and punctuation.
; For the same node a later pattern wins; otherwise the innermost node wins.

["{" "}" "#{"] @CSS.BRACES
["(" ")"] @CSS.PARENTHESES
["[" "]"] @CSS.BRACKETS
":" @CSS.COLON
";" @CSS.SEMICOLON
"," @CSS.COMMA
"." @CSS.DOT
["~" ">" "+" "-" "*" "/" "=" "==" "!=" "<" "<=" ">=" "^=" "|=" "~=" "$=" "*="] @CSS.OPERATION_SIGN

[(comment) (js_comment)] @SASS_COMMENT
(string_value) @SASS_STRING
[(integer_value) (float_value)] @SASS_NUMBER
(unit) @CSS.UNIT
(color_value) @CSS.COLOR
(important) @SASS_IMPORTANT
; the grammar has no `!default`; it parses as an error of its own
((ERROR) @SASS_DEFAULT
 (#eq? @SASS_DEFAULT "!default"))

(at_keyword) @SASS_KEYWORD
[
  "@at-root" "@charset" "@debug" "@each" "@else" "@error" "@for" "@forward" "@function" "@if" "@import"
  "@include" "@keyframes" "@media" "@mixin" "@namespace" "@return" "@supports" "@use" "@warn" "@while"
  "and" "or" "not" "only" "selector" "from" "through" "in" "if"
] @SASS_KEYWORD
[(to) (from)] @SASS_KEYWORD
"@extend" @SASS_EXTEND

(identifier) @SASS_IDENTIFIER
(plain_value) @SASS_IDENTIFIER
[(keyframes_name) (namespace_name) (keyword_query)] @SASS_IDENTIFIER
[(tag_name) (nesting_selector) (universal_selector)] @SASS_TAG_NAME
(class_selector "." @CSS.CLASS_NAME (class_name) @CSS.CLASS_NAME)
(class_name (identifier) @CSS.CLASS_NAME)
(placeholder) @CSS.CLASS_NAME
(placeholder (identifier) @CSS.CLASS_NAME)
(id_selector "#" @CSS.ID_SELECTOR (id_name) @CSS.ID_SELECTOR)
(pseudo_class_selector ":" @CSS.PSEUDO (class_name) @CSS.PSEUDO)
(pseudo_element_selector "::" @CSS.PSEUDO (tag_name) @CSS.PSEUDO)
(attribute_selector (attribute_name) @CSS.ATTRIBUTE_NAME)

(property_name) @SASS_PROPERTY_NAME
(property_name (identifier) @SASS_PROPERTY_NAME)
(feature_name) @SASS_PROPERTY_NAME
(declaration (plain_value) @SASS_PROPERTY_VALUE)
((property_name) @CSS.VARIABLE
 (#match? @CSS.VARIABLE "^--"))
((property_name) @SASS_VARIABLE
 (#match? @SASS_VARIABLE "^\\$"))
(variable) @SASS_VARIABLE

(function_name) @SASS_FUNCTION
(function_statement name: (identifier) @SASS_FUNCTION)
(mixin_statement name: (identifier) @SASS_MIXIN)
(include_statement (identifier) @SASS_MIXIN)
((call_expression
  (function_name) @_url
  (arguments [(plain_value) (string_value)] @SASS_URL))
 (#eq? @_url "url"))
