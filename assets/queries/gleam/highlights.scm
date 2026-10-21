; IVE highlight query for Gleam. Capture names are colour key ids (src/colors/keys/).
; For the same node a later pattern wins; otherwise the innermost node wins.

["(" ")"] @GLEAM.PARENTHESES
["[" "]" "<<" ">>"] @GLEAM.BRACKETS
["{" "}"] @GLEAM.BRACES
"," @GLEAM.COMMA
"." @GLEAM.DOT
["->" "<-" ":" "=" "#" ".." "-" "!" "|"] @GLEAM.OPERATION_SIGN
(binary_expression operator: _ @GLEAM.OPERATION_SIGN)

[(integer) (float)] @GLEAM.NUMBER
(string) @GLEAM.STRING
(escape_sequence) @GLEAM.ESCAPE
(comment) @GLEAM.LINE_COMMENT
(statement_comment) @GLEAM.DOC_COMMENT
(module_comment) @GLEAM.MODULE_COMMENT

[
  (visibility_modifier) (opacity_modifier) "as" "assert" "case" "const" "echo" "fn" "if" "import" "let" "panic"
  "todo" "type" "use"
] @GLEAM.KEYWORD
(discard) @GLEAM.KEYWORD

(identifier) @GLEAM.VARIABLE

(module) @GLEAM.MODULE
(import alias: (identifier) @GLEAM.MODULE)
(remote_type_identifier module: (identifier) @GLEAM.MODULE)
(remote_constructor_name module: (identifier) @GLEAM.MODULE)

(type_identifier) @GLEAM.TYPE
[(type_var) (type_parameter)] @GLEAM.TYPE_VARIABLE
(constructor_name) @GLEAM.CONSTRUCTOR
(unqualified_import name: (type_identifier) @GLEAM.CONSTRUCTOR)
(unqualified_import "type" name: (type_identifier) @GLEAM.TYPE)
(unqualified_import name: (identifier) @GLEAM.FUNCTION_CALL)

(label) @GLEAM.FIELD
(argument label: (label) @GLEAM.LABEL)
(function_parameter label: (label) @GLEAM.LABEL)
(tuple_access index: (integer) @GLEAM.FIELD)

(constant name: (identifier) @GLEAM.CONSTANT)
(attribute "@" @GLEAM.ATTRIBUTE name: (identifier) @GLEAM.ATTRIBUTE)

(function_parameter name: (identifier) @GLEAM.PARAMETER)
(anonymous_function (function_parameters (function_parameter name: (identifier) @GLEAM.PARAMETER)))

(function_call function: (identifier) @GLEAM.FUNCTION_CALL)
(function_call function: (field_access record: (identifier) @GLEAM.MODULE field: (label) @GLEAM.FUNCTION_CALL))
(binary_expression operator: "|>" right: (identifier) @GLEAM.FUNCTION_CALL)
(binary_expression
  operator: "|>"
  right: (field_access record: (identifier) @GLEAM.MODULE field: (label) @GLEAM.FUNCTION_CALL))

(function name: (identifier) @GLEAM.FUNCTION_DECLARATION)
(external_function name: (identifier) @GLEAM.FUNCTION_DECLARATION)
