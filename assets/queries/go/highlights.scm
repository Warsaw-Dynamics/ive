; IVE highlight query for Go. Capture names are colour key ids (src/colors/keys/).
; For the same node a later pattern wins; otherwise the innermost node wins.
; Variables are told apart only where they are declared: tree-sitter does not resolve names.

["(" ")"] @GO.PARENTHESES
["[" "]"] @GO.BRACKETS
["{" "}"] @GO.BRACES
"," @GO.COMMA
"." @GO.DOT
";" @GO.SEMICOLON
[
  "=" ":=" "+" "-" "*" "/" "%" "==" "!=" "<" ">" "<=" ">=" "&&" "||" "!" "&" "|" "^" "&^" "~" "<<" ">>" "<-"
  "++" "--" "+=" "-=" "*=" "/=" "%=" "&=" "|=" "^=" "&^=" "<<=" ">>=" "..." ":"
] @GO.OPERATION_SIGN

[(int_literal) (float_literal) (imaginary_literal)] @GO.NUMBER
[(interpreted_string_literal) (raw_string_literal)] @GO.STRING
(rune_literal) @GO.RUNE
(escape_sequence) @GO.VALID_STRING_ESCAPE
((comment) @GO.LINE_COMMENT
 (#match? @GO.LINE_COMMENT "^//"))
((comment) @GO.BLOCK_COMMENT
 (#match? @GO.BLOCK_COMMENT "^/\\*"))

[
  "break" "case" "chan" "const" "continue" "default" "defer" "else" "fallthrough" "for" "func" "go" "goto" "if"
  "import" "interface" "map" "package" "range" "return" "select" "struct" "switch" "type" "var"
] @GO.KEYWORD
[(true) (false) (nil) (iota)] @GO.KEYWORD

(package_identifier) @GO.PACKAGE

(type_identifier) @GO.TYPE
((type_identifier) @GO.BUILTIN_TYPE
 (#any-of? @GO.BUILTIN_TYPE
   "any" "bool" "byte" "comparable" "complex64" "complex128" "error" "float32" "float64" "int" "int8" "int16"
   "int32" "int64" "rune" "string" "uint" "uint8" "uint16" "uint32" "uint64" "uintptr"))
(type_spec name: (type_identifier) @GO.STRUCT type: (struct_type))
(type_spec name: (type_identifier) @GO.INTERFACE type: (interface_type))
(type_parameter_declaration name: (identifier) @GO.TYPE_PARAMETER)

(field_identifier) @GO.FIELD
(keyed_element key: (literal_element (identifier) @GO.FIELD))
(label_name) @GO.LABEL

; a variable is global unless it is declared in a function body
; no pattern is rooted at the file node, which would make the query of any line walk every top-level node of the file
(var_spec name: (identifier) @GO.GLOBAL_VARIABLE)
(statement_list (var_declaration (var_spec name: (identifier) @GO.LOCAL_VARIABLE)))
(statement_list (var_declaration (var_spec_list (var_spec name: (identifier) @GO.LOCAL_VARIABLE))))
((short_var_declaration left: (expression_list (identifier) @GO.LOCAL_VARIABLE))
 (#not-eq? @GO.LOCAL_VARIABLE "_"))
((range_clause left: (expression_list (identifier) @GO.LOCAL_VARIABLE))
 (#not-eq? @GO.LOCAL_VARIABLE "_"))
(const_spec name: (identifier) @GO.CONSTANT)
(parameter_declaration name: (identifier) @GO.PARAMETER)
(variadic_parameter_declaration name: (identifier) @GO.PARAMETER)
(method_declaration receiver: (parameter_list (parameter_declaration name: (identifier) @GO.RECEIVER)))

(function_declaration name: (identifier) @GO.FUNCTION_DECLARATION)
(method_declaration name: (field_identifier) @GO.METHOD_DECLARATION)
(method_elem name: (field_identifier) @GO.METHOD_DECLARATION)
(call_expression function: (identifier) @GO.FUNCTION_CALL)
(call_expression function: (selector_expression field: (field_identifier) @GO.METHOD_CALL))
((call_expression function: (identifier) @GO.BUILTIN_FUNCTION)
 (#any-of? @GO.BUILTIN_FUNCTION
   "append" "cap" "clear" "close" "complex" "copy" "delete" "imag" "len" "make" "max" "min" "new" "panic" "print"
   "println" "real" "recover"))
; tree-sitter cannot tell `pkg.F()` from `value.M()`; the standard library's common packages are recognised by name
((call_expression
   function: (selector_expression
     operand: (identifier) @GO.PACKAGE
     field: (field_identifier) @GO.FUNCTION_CALL))
 (#any-of? @GO.PACKAGE
   "atomic" "base64" "bufio" "bytes" "context" "errors" "exec" "filepath" "fmt" "hex" "http" "io" "json" "log"
   "maps" "math" "os" "path" "rand" "reflect" "regexp" "slices" "sort" "strconv" "strings" "sync" "testing" "time"
   "unicode" "url" "utf8"))
