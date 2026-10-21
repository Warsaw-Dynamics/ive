; IVE highlight query for Lua. Capture names are colour key ids (src/colors/keys/lua.rs).
; For the same node a later pattern wins; otherwise the innermost node wins.

["(" ")"] @LUA.PARENTHESES
["[" "]"] @LUA.BRACKETS
["{" "}"] @LUA.BRACES
"," @LUA.COMMA
["." ":"] @LUA.DOT
";" @LUA.SEMICOLON
[
  "#" "%" "&" "*" "+" "-" "/" "//" ".." "<" "<<" "<=" "=" "==" ">" ">=" ">>" "^" "|" "~" "~="
] @LUA.OPERATION_SIGN

(number) @LUA.NUMBER
(string) @LUA.STRING
(escape_sequence) @LUA.ESCAPE
(comment) @LUA.LINE_COMMENT
((comment) @LUA.BLOCK_COMMENT
 (#match? @LUA.BLOCK_COMMENT "^--\\[=*\\["))
((comment) @LUA.DOC_COMMENT
 (#match? @LUA.DOC_COMMENT "^---"))
(hash_bang_line) @LUA.LINE_COMMENT

[
  "and" "do" "else" "elseif" "end" "for" "function" "global" "goto" "if" "in" "local" "not" "or"
  "repeat" "return" "then" "until" "while"
] @LUA.KEYWORD
[(nil) (true) (false) (break_statement)] @LUA.KEYWORD
(vararg_expression) @LUA.PARAMETER

(field name: (identifier) @LUA.FIELD)
(dot_index_expression field: (identifier) @LUA.FIELD)
(parameters (identifier) @LUA.PARAMETER)
(variable_declaration (variable_list name: (identifier) @LUA.LOCAL_VARIABLE))
(variable_declaration (assignment_statement (variable_list name: (identifier) @LUA.LOCAL_VARIABLE)))
(for_numeric_clause name: (identifier) @LUA.LOCAL_VARIABLE)
(for_generic_clause (variable_list name: (identifier) @LUA.LOCAL_VARIABLE))
(attribute "<" @LUA.ATTRIBUTE (identifier) @LUA.ATTRIBUTE ">" @LUA.ATTRIBUTE)
(label_statement) @LUA.LABEL
(goto_statement (identifier) @LUA.LABEL)

(function_call name: (identifier) @LUA.FUNCTION_CALL)
(function_call name: (dot_index_expression field: (identifier) @LUA.FUNCTION_CALL))
(function_call name: (method_index_expression method: (identifier) @LUA.METHOD_CALL))

((identifier) @LUA.CONSTANT
 (#match? @LUA.CONSTANT "^[A-Z][A-Z0-9_]+$"))
((identifier) @LUA.PREDEFINED
 (#any-of? @LUA.PREDEFINED "self" "_G" "_ENV" "_VERSION"))
((function_call name: (identifier) @LUA.PREDEFINED)
 (#any-of? @LUA.PREDEFINED
   "assert" "collectgarbage" "dofile" "error" "getmetatable" "ipairs" "load" "loadfile" "next" "pairs" "pcall"
   "print" "rawequal" "rawget" "rawlen" "rawset" "require" "select" "setmetatable" "tonumber" "tostring" "type"
   "unpack" "xpcall"))
([(dot_index_expression table: (identifier) @LUA.PREDEFINED)
  (method_index_expression table: (identifier) @LUA.PREDEFINED)]
 (#any-of? @LUA.PREDEFINED "coroutine" "debug" "io" "math" "os" "package" "string" "table" "utf8"))

(function_declaration name: (identifier) @LUA.FUNCTION_DECLARATION)
(function_declaration name: (dot_index_expression field: (identifier) @LUA.FUNCTION_DECLARATION))
(function_declaration name: (method_index_expression method: (identifier) @LUA.FUNCTION_DECLARATION))
(assignment_statement
  (variable_list . name: [(identifier) (dot_index_expression field: (identifier))] @LUA.FUNCTION_DECLARATION)
  (expression_list . value: (function_definition)))
(field name: (identifier) @LUA.FUNCTION_DECLARATION value: (function_definition))
