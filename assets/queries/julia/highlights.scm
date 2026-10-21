; IVE highlight query for Julia. Capture names are colour key ids (src/colors/keys/julia.rs).
; For the same node a later pattern wins; otherwise the innermost node wins.

["(" ")"] @JULIA.PARENTHESES
["[" "]"] @JULIA.BRACKETS
["{" "}"] @JULIA.BRACES
"," @JULIA.COMMA
"." @JULIA.DOT
";" @JULIA.SEMICOLON
[(operator) "::" "..." "->" "=" "?" ":" "'" "$" ".=" "$="] @JULIA.OPERATION_SIGN
((operator) @JULIA.KEYWORD
 (#any-of? @JULIA.KEYWORD "in" "isa" "∈"))

[(integer_literal) (float_literal)] @JULIA.NUMBER
((identifier) @JULIA.NUMBER
 (#any-of? @JULIA.NUMBER "NaN" "NaN16" "NaN32" "Inf" "Inf16" "Inf32"))
[(string_literal) (character_literal) (command_literal) (prefixed_string_literal) (prefixed_command_literal)]
  @JULIA.STRING
(escape_sequence) @JULIA.ESCAPE
(string_interpolation) @TEXT
(string_interpolation "$" @JULIA.INTERPOLATION)
(string_interpolation "$" @JULIA.INTERPOLATION . "(" @JULIA.INTERPOLATION)
(string_interpolation ")" @JULIA.INTERPOLATION .)
(quote_expression ":" @JULIA.SYMBOL [(identifier) (operator)] @JULIA.SYMBOL)
(line_comment) @JULIA.LINE_COMMENT
(block_comment) @JULIA.BLOCK_COMMENT
((string_literal) @JULIA.DOC_COMMENT
 .
 [
   (abstract_definition) (assignment) (const_statement) (function_definition) (macro_definition)
   (module_definition) (struct_definition)
 ])

[
  "abstract" "baremodule" "begin" "catch" "const" "do" "else" "elseif" "end" "export" "finally" "for"
  "function" "global" "if" "import" "let" "local" "macro" "module" "mutable" "outer" "primitive" "public"
  "quote" "return" "struct" "try" "type" "using" "where" "while" "as"
] @JULIA.KEYWORD
[(break_statement) (continue_statement) (boolean_literal)] @JULIA.KEYWORD

(field_expression (identifier) @JULIA.FIELD .)
(assignment . (identifier) @JULIA.LOCAL_VARIABLE)
(assignment . (tuple_expression (identifier) @JULIA.LOCAL_VARIABLE))
(compound_assignment_expression . (identifier) @JULIA.LOCAL_VARIABLE)
(for_binding . (identifier) @JULIA.LOCAL_VARIABLE)
(for_binding . (tuple_expression (identifier) @JULIA.LOCAL_VARIABLE))
((identifier) @JULIA.CONSTANT
 (#match? @JULIA.CONSTANT "^[A-Z][A-Z0-9_]+$"))
(const_statement (assignment . (identifier) @JULIA.CONSTANT))

(call_expression . (identifier) @JULIA.FUNCTION_CALL)
(call_expression . (field_expression (identifier) @JULIA.FUNCTION_CALL .))
(broadcast_call_expression . (identifier) @JULIA.FUNCTION_CALL)
(broadcast_call_expression . (field_expression (identifier) @JULIA.FUNCTION_CALL .))
((identifier) @JULIA.PREDEFINED
 (#any-of? @JULIA.PREDEFINED
   "nothing" "missing" "print" "println" "show" "string" "length" "size" "push!" "pop!" "append!" "isempty"
   "sum" "map" "filter" "reduce" "foreach" "collect" "enumerate" "zip" "eachindex" "keys" "values" "haskey"
   "get" "error" "throw" "typeof" "isa" "convert" "round" "floor" "ceil" "abs" "min" "max" "first" "last"
   "sort" "sort!" "in" "iterate" "parse" "tryparse" "eltype" "similar" "zeros" "ones" "fill" "rand"))

((call_expression . (identifier) @JULIA.TYPE)
 (#match? @JULIA.TYPE "^[A-Z][a-z]"))

(type_head (_) @JULIA.TYPE)
(type_head (parametrized_type_expression . (identifier) @JULIA.TYPE))
(parametrized_type_expression (curly_expression (_) @JULIA.TYPE))
(parametrized_type_expression . [(identifier) (field_expression (identifier) .)] @JULIA.TYPE)
(typed_expression (_) "::" [(identifier) (field_expression (identifier) .)] @JULIA.TYPE .)
(unary_typed_expression [(identifier) (field_expression (identifier) .)] @JULIA.TYPE .)
(where_expression (_) [(identifier) (curly_expression (_))] @JULIA.TYPE .)
(binary_expression (_) @JULIA.TYPE (operator) @_subtype (_) @JULIA.TYPE
 (#any-of? @_subtype "<:" ">:"))
(module_definition name: (identifier) @JULIA.MODULE)

(signature (call_expression (argument_list (identifier) @JULIA.PARAMETER)))
(signature (call_expression (argument_list (typed_expression . (identifier) @JULIA.PARAMETER))))
(signature (call_expression (argument_list (splat_expression (identifier) @JULIA.PARAMETER))))
(signature (call_expression (argument_list (named_argument . (identifier) @JULIA.PARAMETER))))
(signature (typed_expression (call_expression (argument_list (identifier) @JULIA.PARAMETER))))
(assignment . (call_expression (argument_list (identifier) @JULIA.PARAMETER)))
(assignment . (call_expression (argument_list (typed_expression . (identifier) @JULIA.PARAMETER))))
(arrow_function_expression . (identifier) @JULIA.PARAMETER)
(arrow_function_expression . (argument_list (identifier) @JULIA.PARAMETER))
(do_clause (argument_list (identifier) @JULIA.PARAMETER))

(signature (call_expression . (identifier) @JULIA.FUNCTION_DECLARATION))
(signature (call_expression . (field_expression (identifier) @JULIA.FUNCTION_DECLARATION .)))
(signature (typed_expression (call_expression . (identifier) @JULIA.FUNCTION_DECLARATION)))
(signature (where_expression (call_expression . (identifier) @JULIA.FUNCTION_DECLARATION)))
(assignment . (call_expression . (identifier) @JULIA.FUNCTION_DECLARATION))

(macro_identifier) @JULIA.MACRO
(macro_identifier (identifier) @JULIA.MACRO)
(macro_definition (signature (call_expression . (identifier) @JULIA.MACRO)))
(prefixed_string_literal prefix: (identifier) @JULIA.MACRO)
(prefixed_command_literal prefix: (identifier) @JULIA.MACRO)
