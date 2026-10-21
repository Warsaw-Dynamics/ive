; IVE highlight query for Elm. Capture names are colour key ids (src/colors/keys/).
; For the same node a later pattern wins; otherwise the innermost node wins.

["(" ")"] @ELM.PARENTHESES
["[" "]"] @ELM.BRACKETS
["{" "}"] @ELM.BRACES
"," @ELM.COMMA
(dot) @ELM.DOT
[(operator_identifier) (eq) (arrow) (backslash) (colon) (double_dot) "|"] @ELM.OPERATION_SIGN

(number_constant_expr) @ELM.NUMBER
(number_literal) @ELM.NUMBER
(string_constant_expr) @ELM.STRING
(char_constant_expr) @ELM.CHAR
(string_escape) @ELM.ESCAPE
(line_comment) @ELM.LINE_COMMENT
(block_comment) @ELM.BLOCK_COMMENT
((block_comment) @ELM.DOC_COMMENT
 (#match? @ELM.DOC_COMMENT "^\\{-\\|"))

[
  "if" "then" "else" "let" "in" (case) (of) (type) (alias) (module) (import) (exposing) (as) (port) (where)
  (effect) (infix) (underscore)
] @ELM.KEYWORD

(module_declaration name: (upper_case_qid) @ELM.MODULE)
(import_clause moduleName: (upper_case_qid) @ELM.MODULE)
(as_clause name: (upper_case_identifier) @ELM.MODULE)
(value_qid (upper_case_identifier) @ELM.MODULE)

(type_ref (upper_case_qid) @ELM.TYPE)
(type_declaration name: (upper_case_identifier) @ELM.TYPE)
(type_alias_declaration name: (upper_case_identifier) @ELM.TYPE)
(exposed_type (upper_case_identifier) @ELM.TYPE)
[(type_variable) (lower_type_name)] @ELM.TYPE_VARIABLE

(lower_pattern) @ELM.LOCAL_VARIABLE

(function_call_expr target: (value_expr) @ELM.FUNCTION_CALL)
(bin_op_expr (operator (operator_identifier) @_pipe) . (value_expr) @ELM.FUNCTION_CALL
 (#any-of? @_pipe "|>"))

(union_variant name: (upper_case_identifier) @ELM.CONSTRUCTOR)
(union_pattern constructor: (upper_case_qid) @ELM.CONSTRUCTOR)
(nullary_constructor_argument_pattern) @ELM.CONSTRUCTOR
(value_expr name: (upper_case_qid) @ELM.CONSTRUCTOR)

(field_type name: (lower_case_identifier) @ELM.FIELD)
(field name: (lower_case_identifier) @ELM.FIELD)
(field_access_expr (lower_case_identifier) @ELM.FIELD)
(field_accessor_function_expr (lower_case_identifier) @ELM.FIELD)
(record_pattern (lower_pattern) @ELM.FIELD)

(type_annotation name: (lower_case_identifier) @ELM.TYPE_ANNOTATION)
(port_annotation name: (lower_case_identifier) @ELM.TYPE_ANNOTATION)
(function_declaration_left (lower_case_identifier) @ELM.FUNCTION_DECLARATION)
(let_in_expr
  (value_declaration (function_declaration_left . (lower_case_identifier) @ELM.LOCAL_VARIABLE .)))
(let_in_expr (type_annotation name: (lower_case_identifier) @ELM.LOCAL_VARIABLE))

(function_declaration_left (lower_pattern) @ELM.PARAMETER)
(function_declaration_left (_ (lower_pattern) @ELM.PARAMETER))
(function_declaration_left (_ (_ (lower_pattern) @ELM.PARAMETER)))
(anonymous_function_expr param: (pattern (lower_pattern) @ELM.PARAMETER))
