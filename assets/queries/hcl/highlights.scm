; IVE highlight query for HCL and Terraform. Capture names are colour key ids (src/colors/keys/).
; For the same node a later pattern wins; otherwise the innermost node wins.

["(" ")"] @HCL.PARENTHESES
["[" "]" (tuple_start) (tuple_end)] @HCL.BRACKETS
["{" "}" (object_start) (object_end) (block_start) (block_end)] @HCL.BRACES
"," @HCL.COMMA
"." @HCL.DOT
[
  "!" "!=" "%" "&&" "*" "+" "-" "/" ":" "<" "<=" "=" "==" "=>" ">" ">=" "?" "||" ".*" "[*]" (ellipsis)
] @HCL.OPERATION_SIGN

(numeric_lit) @HCL.NUMBER
[(bool_lit) (null_lit)] @HCL.KEYWORD
[(string_lit) (quoted_template) (heredoc_template)] @HCL.STRING
[
  (template_interpolation_start) (template_interpolation_end) (template_directive_start) (template_directive_end)
  (strip_marker)
] @HCL.INTERPOLATION
[(heredoc_start) (heredoc_identifier)] @HCL.HEREDOC_DELIMITER
((comment) @HCL.LINE_COMMENT
 (#match? @HCL.LINE_COMMENT "^(#|//)"))
((comment) @HCL.BLOCK_COMMENT
 (#match? @HCL.BLOCK_COMMENT "^/\\*"))

["for" "in" "if" "else" "endif" "endfor"] @HCL.KEYWORD

(block (identifier) @HCL.BLOCK_TYPE)
(block (string_lit) @HCL.BLOCK_LABEL)
(block (string_lit (template_literal) @HCL.BLOCK_LABEL))
(attribute (identifier) @HCL.ATTRIBUTE)

(variable_expr (identifier) @HCL.LOCAL_VARIABLE)
(for_intro (identifier) @HCL.LOCAL_VARIABLE)
(template_for_start (identifier) @HCL.LOCAL_VARIABLE)
(get_attr (identifier) @HCL.PROPERTY)
; Terraform's primitive type constraints: `type = string`, `list(number)`
((variable_expr (identifier) @HCL.KEYWORD)
 (#any-of? @HCL.KEYWORD "string" "number" "bool" "any"))
((variable_expr (identifier) @HCL.PREDEFINED)
 (#any-of? @HCL.PREDEFINED "var" "local" "module" "data" "each" "count" "path" "self" "terraform"))
((expression
   (variable_expr (identifier) @_root)
   .
   (get_attr (identifier) @HCL.VARIABLE))
 (#any-of? @_root "var" "local"))

(object_elem key: (expression (variable_expr (identifier) @HCL.ATTRIBUTE)))

(function_call (identifier) @HCL.FUNCTION_CALL)
