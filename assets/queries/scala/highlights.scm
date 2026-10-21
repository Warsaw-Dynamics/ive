; IVE highlight query for Scala. Capture names are colour key ids (src/colors/keys/scala.rs).
; For the same node a later pattern wins; otherwise the innermost node wins.

["(" ")"] @SCALA.PARENTHESES
["[" "]"] @SCALA.BRACKETS
["{" "}"] @SCALA.BRACES
"," @SCALA.COMMA
"." @SCALA.DOT
";" @SCALA.SEMICOLON
["=" "=>" "<-" "->" ":" "<:" ">:" "<%" "=>>" "?=>" "!" "~" "+" "-" "*" "#" "|"] @SCALA.OPERATOR
(operator_identifier) @SCALA.OPERATOR

[(integer_literal) (floating_point_literal)] @SCALA.NUMBER
[(string) (character_literal) (interpolated_string)] @SCALA.STRING
(escape_sequence) @SCALA.ESCAPE
; Embedded code is not string text.
(interpolation [(identifier) (block)] @TEXT)
(interpolation "$" @SCALA.INTERPOLATION)
(interpolation (block ["{" "}"] @SCALA.INTERPOLATION))
(interpolated_string_expression interpolator: (identifier) @SCALA.INTERPOLATION)
(comment) @SCALA.LINE_COMMENT
(block_comment) @SCALA.BLOCK_COMMENT
((block_comment) @SCALA.DOC_COMMENT
 (#match? @SCALA.DOC_COMMENT "^/\\*\\*[^/]"))

[
  "abstract" "case" "catch" "class" "def" "derives" "do" "else" "enum" "export" "extends" "final" "finally"
  "for" "forSome" "given" "if" "implicit" "import" "lazy" "macro" "match" "new" "object" "override" "package"
  "private" "protected" "return" "sealed" "then" "throw" "trait" "try" "type" "using" "val" "var" "while" "with"
  "yield"
] @SCALA.KEYWORD
[
  (boolean_literal) (null_literal) (end_marker) (opaque_modifier) (infix_modifier) (transparent_modifier)
  (open_modifier) (inline_modifier)
] @SCALA.KEYWORD
(extension_definition "extension" @SCALA.KEYWORD)
["this" "super"] @SCALA.PREDEFINED
((identifier) @SCALA.PREDEFINED
 (#any-of? @SCALA.PREDEFINED "this" "super"))
(wildcard) @SCALA.KEYWORD

(type_identifier) @SCALA.TYPE
((identifier) @SCALA.TYPE
 (#match? @SCALA.TYPE "^[A-Z]"))
((identifier) @SCALA.CONSTANT
 (#match? @SCALA.CONSTANT "^[A-Z][A-Z0-9_]+$"))
(class_definition name: (identifier) @SCALA.TYPE)
(enum_definition name: (identifier) @SCALA.TYPE)
(trait_definition name: (identifier) @SCALA.TRAIT)
(object_definition name: (identifier) @SCALA.OBJECT)
(type_definition name: (type_identifier) @SCALA.TYPE)
(type_parameters name: (identifier) @SCALA.TYPE_PARAMETER)
(simple_enum_case name: (identifier) @SCALA.CONSTANT)
(full_enum_case name: (identifier) @SCALA.CONSTANT)

(field_expression field: (identifier) @SCALA.FIELD)
(val_definition pattern: (identifier) @SCALA.LOCAL_VARIABLE)
(var_definition pattern: (identifier) @SCALA.LOCAL_VARIABLE)
(template_body (val_definition pattern: (identifier) @SCALA.FIELD))
(template_body (var_definition pattern: (identifier) @SCALA.FIELD))
(template_body (val_declaration name: (identifier) @SCALA.FIELD))
(template_body (var_declaration name: (identifier) @SCALA.FIELD))
(enumerator . (identifier) @SCALA.LOCAL_VARIABLE)
((val_definition pattern: (identifier) @SCALA.CONSTANT)
 (#match? @SCALA.CONSTANT "^[A-Z][A-Z0-9_]+$"))

(parameter name: (identifier) @SCALA.PARAMETER)
(class_parameter name: (identifier) @SCALA.PARAMETER)
(binding name: (identifier) @SCALA.PARAMETER)
(lambda_expression parameters: (identifier) @SCALA.PARAMETER)
(case_class_pattern pattern: (identifier) @SCALA.LOCAL_VARIABLE)
(arguments (assignment_expression left: (identifier) @SCALA.PARAMETER))

(function_definition name: (identifier) @SCALA.FUNCTION_DECLARATION)
(function_declaration name: (identifier) @SCALA.FUNCTION_DECLARATION)
(call_expression function: (identifier) @SCALA.FUNCTION_CALL)
(generic_function function: (identifier) @SCALA.FUNCTION_CALL)
(call_expression function: (field_expression field: (identifier) @SCALA.METHOD_CALL))
(generic_function function: (field_expression field: (identifier) @SCALA.METHOD_CALL))
((call_expression function: (identifier) @SCALA.TYPE)
 (#match? @SCALA.TYPE "^[A-Z]"))

(annotation "@" @SCALA.ANNOTATION name: (_) @SCALA.ANNOTATION)
