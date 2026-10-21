; IVE highlight query for Zig. Capture names are colour key ids (src/colors/keys/).
; For the same node a later pattern wins; otherwise the innermost node wins.
; Variables are told apart only where they are declared: tree-sitter does not resolve names.

["(" ")"] @ZIG.PARENTHESES
["[" "]"] @ZIG.BRACKETS
["{" "}"] @ZIG.BRACES
"," @ZIG.COMMA
"." @ZIG.DOT
";" @ZIG.SEMICOLON
[
  "=" "*=" "*%=" "*|=" "/=" "%=" "+=" "+%=" "+|=" "-=" "-%=" "-|=" "<<=" "<<|=" ">>=" "&=" "^=" "|=" "!" "~" "-"
  "-%" "&" "==" "!=" ">" ">=" "<=" "<" "^" "|" "<<" ">>" "<<|" "+" "++" "+%" "+|" "-|" "*" "/" "%" "**" "*%" "*|"
  "||" ".*" ".?" "?" ".." "..." "=>" "->" ":"
] @ZIG.OPERATION_SIGN

[(integer) (float)] @ZIG.NUMBER
[(string) (multiline_string)] @ZIG.STRING
(character) @ZIG.CHAR
(escape_sequence) @ZIG.VALID_STRING_ESCAPE
(comment) @ZIG.LINE_COMMENT
((comment) @ZIG.DOC_COMMENT
 (#match? @ZIG.DOC_COMMENT "^//[/!]"))
((comment) @ZIG.LINE_COMMENT
 (#match? @ZIG.LINE_COMMENT "^////"))

[
  "addrspace" "align" "allowzero" "and" "asm" "async" "await" "break" "callconv" "catch" "comptime" "const"
  "continue" "defer" "else" "enum" "errdefer" "error" "export" "extern" "fn" "for" "if" "inline" "linksection"
  "noalias" "noinline" "nosuspend" "opaque" "or" "orelse" "packed" "pub" "resume" "return" "struct" "suspend"
  "switch" "test" "threadlocal" "try" "union" "usingnamespace" "var" "volatile" "while"
] @ZIG.KEYWORD
[(boolean) "null" "undefined" "unreachable"] @ZIG.KEYWORD
[(builtin_type) "anyframe"] @ZIG.BUILTIN_TYPE

((identifier) @ZIG.TYPE
 (#match? @ZIG.TYPE "^[A-Z]"))
((identifier) @ZIG.CONSTANT
 (#match? @ZIG.CONSTANT "^[A-Z][A-Z0-9_]+$"))
(container_field name: (_ (identifier) @ZIG.FIELD))
(container_field name: (identifier) @ZIG.FIELD)
(field_initializer . (identifier) @ZIG.FIELD)
(field_expression member: (identifier) @ZIG.FIELD)
((field_expression member: (identifier) @ZIG.TYPE)
 (#match? @ZIG.TYPE "^[A-Z]"))
[(block_label (identifier)) (break_label (identifier))] @ZIG.LABEL

; a variable is global (container level) unless it is declared in a block; the grammar parses `x = y;` as a
; declaration too
; no pattern is rooted at the file node, which would make the query of any line walk every top-level node of the file
(variable_declaration ["const" "var"] . (identifier) @ZIG.GLOBAL_VARIABLE)
[(block (variable_declaration ["const" "var"] . (identifier) @ZIG.LOCAL_VARIABLE))
 (else_clause (variable_declaration ["const" "var"] . (identifier) @ZIG.LOCAL_VARIABLE))
 (comptime_statement (variable_declaration ["const" "var"] . (identifier) @ZIG.LOCAL_VARIABLE))]
(payload (identifier) @ZIG.LOCAL_VARIABLE)
((variable_declaration ["const" "var"] . (identifier) @ZIG.CONSTANT)
 (#match? @ZIG.CONSTANT "^[A-Z][A-Z0-9_]+$"))
(variable_declaration
  (identifier) @ZIG.TYPE
  "="
  [(struct_declaration) (enum_declaration) (union_declaration) (opaque_declaration) (error_set_declaration)])
(parameter name: (identifier) @ZIG.PARAMETER)

(function_declaration name: (identifier) @ZIG.FUNCTION_DECLARATION)
(call_expression function: (identifier) @ZIG.FUNCTION_CALL)
(call_expression function: (field_expression member: (identifier) @ZIG.FUNCTION_CALL))
(builtin_identifier) @ZIG.BUILTIN_FUNCTION
