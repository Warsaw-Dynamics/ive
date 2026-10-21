; IVE highlight query for C. Capture names are colour key ids (src/colors/keys/).
; For the same node a later pattern wins; otherwise the innermost node wins.
; Variables are told apart only where they are declared: tree-sitter does not resolve names.

["(" ")"] @C.PARENTHESES
["[" "]" "[[" "]]"] @C.BRACKETS
["{" "}"] @C.BRACES
"," @C.COMMA
"." @C.DOT
";" @C.SEMICOLON
[
  "=" "+" "-" "*" "/" "%" "==" "!=" "<" ">" "<=" ">=" "&&" "||" "!" "&" "|" "^" "~" "<<" ">>" "?" ":"
  "->" "++" "--" "+=" "-=" "*=" "/=" "%=" "&=" "|=" "^=" "<<=" ">>=" "..."
] @C.OPERATION_SIGN

(number_literal) @C.NUMBER
[(string_literal) (system_lib_string)] @C.STRING
(char_literal) @C.CHAR
(escape_sequence) @C.VALID_STRING_ESCAPE
((comment) @C.LINE_COMMENT
 (#match? @C.LINE_COMMENT "^//"))
((comment) @C.BLOCK_COMMENT
 (#match? @C.BLOCK_COMMENT "^/\\*"))
((comment) @C.DOC_COMMENT
 (#match? @C.DOC_COMMENT "^(///|/\\*\\*[^/])"))

[
  "_Alignas" "_Alignof" "_Atomic" "_Generic" "_Noreturn" "__attribute__" "__attribute" "__declspec" "__extension__"
  "__inline" "__inline__" "__restrict__" "__thread" "__volatile__" "alignas" "alignof" "asm" "__asm" "__asm__" "auto"
  "break" "case" "const" "constexpr" "continue" "default" "do" "else" "enum" "extern" "for" "goto" "if" "inline"
  "noreturn" "offsetof" "register" "restrict" "return" "sizeof" "static" "struct" "switch" "thread_local" "typedef"
  "union" "volatile" "while"
] @C.KEYWORD
[(true) (false) (null)] @C.KEYWORD
[(primitive_type) (sized_type_specifier)] @C.PRIMITIVE_TYPE

[
  "#define" "#elif" "#elifdef" "#elifndef" "#else" "#endif" "#if" "#ifdef" "#ifndef" "#include" "defined"
] @C.PREPROCESSOR_DIRECTIVE
(preproc_directive) @C.PREPROCESSOR_DIRECTIVE

(type_identifier) @C.TYPE
((identifier) @C.CONSTANT
 (#match? @C.CONSTANT "^[A-Z][A-Z0-9_]+$"))
(enumerator name: (identifier) @C.ENUM_CONSTANT)
(field_identifier) @C.FIELD
(statement_identifier) @C.LABEL

; a declaration is global unless it sits in a block or a for loop's initializer; no pattern is rooted at the
; file node, which would make the query of any line walk every top-level node of the file
(declaration
  declarator: [
    (identifier) @C.GLOBAL_VARIABLE
    (init_declarator declarator: (identifier) @C.GLOBAL_VARIABLE)
    (init_declarator declarator: (pointer_declarator declarator: (identifier) @C.GLOBAL_VARIABLE))
    (init_declarator declarator: (array_declarator declarator: (identifier) @C.GLOBAL_VARIABLE))
    (pointer_declarator declarator: (identifier) @C.GLOBAL_VARIABLE)
    (array_declarator declarator: (identifier) @C.GLOBAL_VARIABLE)
  ])
(compound_statement
  (declaration
    declarator: [
      (identifier) @C.LOCAL_VARIABLE
      (init_declarator declarator: (identifier) @C.LOCAL_VARIABLE)
      (init_declarator declarator: (pointer_declarator declarator: (identifier) @C.LOCAL_VARIABLE))
      (init_declarator declarator: (array_declarator declarator: (identifier) @C.LOCAL_VARIABLE))
      (pointer_declarator declarator: (identifier) @C.LOCAL_VARIABLE)
      (array_declarator declarator: (identifier) @C.LOCAL_VARIABLE)
    ]))
(for_statement
  initializer: (declaration
    declarator: [
      (identifier) @C.LOCAL_VARIABLE
      (init_declarator declarator: (identifier) @C.LOCAL_VARIABLE)
      (init_declarator declarator: (pointer_declarator declarator: (identifier) @C.LOCAL_VARIABLE))
      (init_declarator declarator: (array_declarator declarator: (identifier) @C.LOCAL_VARIABLE))
      (pointer_declarator declarator: (identifier) @C.LOCAL_VARIABLE)
      (array_declarator declarator: (identifier) @C.LOCAL_VARIABLE)
    ]))
(parameter_declaration
  declarator: [
    (identifier) @C.PARAMETER
    (pointer_declarator declarator: (identifier) @C.PARAMETER)
    (pointer_declarator declarator: (pointer_declarator declarator: (identifier) @C.PARAMETER))
    (array_declarator declarator: (identifier) @C.PARAMETER)
  ])

(function_declarator declarator: (identifier) @C.FUNCTION_DECLARATION)
(call_expression function: (identifier) @C.FUNCTION_CALL)
(call_expression function: (field_expression field: (field_identifier) @C.FUNCTION_CALL))

(preproc_def name: (identifier) @C.MACRO)
(preproc_function_def name: (identifier) @C.MACRO)
(preproc_ifdef name: (identifier) @C.MACRO)
(preproc_defined (identifier) @C.MACRO)
((call_expression function: (identifier) @C.MACRO)
 (#match? @C.MACRO "^[A-Z][A-Z0-9_]+$"))
(preproc_params (identifier) @C.PARAMETER)

[(attribute_specifier) (attribute_declaration) (ms_declspec_modifier)] @C.ATTRIBUTE
