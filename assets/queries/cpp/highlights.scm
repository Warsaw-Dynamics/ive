; IVE highlight query for C++. Capture names are colour key ids (src/colors/keys/).
; For the same node a later pattern wins; otherwise the innermost node wins.
; Variables are told apart only where they are declared: tree-sitter does not resolve names.

["(" ")" "()"] @CPP.PARENTHESES
["[" "]" "[[" "]]" "[]"] @CPP.BRACKETS
["{" "}"] @CPP.BRACES
"," @CPP.COMMA
"." @CPP.DOT
";" @CPP.SEMICOLON
[
  "=" "+" "-" "*" "/" "%" "==" "!=" "<" ">" "<=" ">=" "<=>" "&&" "||" "!" "&" "|" "^" "~" "<<" ">>" "?" ":"
  "::" "->" "->*" ".*" "++" "--" "+=" "-=" "*=" "/=" "%=" "&=" "|=" "^=" "<<=" ">>=" "..."
  "and" "and_eq" "bitand" "bitor" "compl" "not" "not_eq" "or" "or_eq" "xor" "xor_eq"
] @CPP.OPERATION_SIGN

[(number_literal) (user_defined_literal)] @CPP.NUMBER
[(string_literal) (raw_string_literal) (system_lib_string)] @CPP.STRING
(char_literal) @CPP.CHAR
(escape_sequence) @CPP.VALID_STRING_ESCAPE
((comment) @CPP.LINE_COMMENT
 (#match? @CPP.LINE_COMMENT "^//"))
((comment) @CPP.BLOCK_COMMENT
 (#match? @CPP.BLOCK_COMMENT "^/\\*"))
((comment) @CPP.DOC_COMMENT
 (#match? @CPP.DOC_COMMENT "^(///|//!|/\\*\\*[^/]|/\\*!)"))

[
  "_Alignas" "_Alignof" "_Atomic" "_Generic" "_Noreturn" "__attribute__" "__attribute" "__declspec" "__extension__"
  "__inline" "__inline__" "__restrict__" "__thread" "__volatile__" "alignas" "alignof" "asm" "__asm" "__asm__"
  "break" "case" "catch" "class" "co_await" "co_return" "co_yield" "concept" "const" "consteval" "constexpr"
  "constinit" "continue" "decltype" "default" "delete" "do" "else" "enum" "explicit" "extern" "final" "for" "friend"
  "goto" "if" "inline" "mutable" "namespace" "new" "noexcept" "noreturn" "offsetof" "operator" "override" "private"
  "protected" "public" "register" "requires" "restrict" "return" "sizeof" "static" "static_assert" "struct" "switch"
  "template" "thread_local" "throw" "try" "typedef" "typename" "union" "using" "virtual" "volatile" "while"
] @CPP.KEYWORD
[(true) (false) (null) (this) (auto)] @CPP.KEYWORD
[(primitive_type) (sized_type_specifier)] @CPP.PRIMITIVE_TYPE

[
  "#define" "#elif" "#elifdef" "#elifndef" "#else" "#endif" "#if" "#ifdef" "#ifndef" "#include" "defined"
] @CPP.PREPROCESSOR_DIRECTIVE
(preproc_directive) @CPP.PREPROCESSOR_DIRECTIVE

(namespace_identifier) @CPP.NAMESPACE
((namespace_identifier) @CPP.TYPE
 (#match? @CPP.TYPE "^[A-Z]"))
(type_identifier) @CPP.TYPE
(type_parameter_declaration (type_identifier) @CPP.TYPE_PARAMETER)
(optional_type_parameter_declaration name: (type_identifier) @CPP.TYPE_PARAMETER)
(variadic_type_parameter_declaration (type_identifier) @CPP.TYPE_PARAMETER)
(concept_definition name: (identifier) @CPP.CONCEPT)
((identifier) @CPP.CONSTANT
 (#match? @CPP.CONSTANT "^[A-Z][A-Z0-9_]+$"))
(enumerator name: (identifier) @CPP.ENUM_CONSTANT)
(field_identifier) @CPP.FIELD
((field_identifier) @CPP.CONSTANT
 (#match? @CPP.CONSTANT "^[A-Z][A-Z0-9_]+$"))
(statement_identifier) @CPP.LABEL

; a declaration is global unless it sits in a block or a for loop's initializer; no pattern is rooted at the
; file node, which would make the query of any line walk every top-level node of the file
(declaration
  declarator: [
    (identifier) @CPP.GLOBAL_VARIABLE
    (init_declarator declarator: (identifier) @CPP.GLOBAL_VARIABLE)
    (init_declarator declarator: (pointer_declarator declarator: (identifier) @CPP.GLOBAL_VARIABLE))
    (init_declarator declarator: (reference_declarator (identifier) @CPP.GLOBAL_VARIABLE))
    (init_declarator declarator: (array_declarator declarator: (identifier) @CPP.GLOBAL_VARIABLE))
    (pointer_declarator declarator: (identifier) @CPP.GLOBAL_VARIABLE)
    (array_declarator declarator: (identifier) @CPP.GLOBAL_VARIABLE)
  ])
(compound_statement
  (declaration
    declarator: [
      (identifier) @CPP.LOCAL_VARIABLE
      (init_declarator declarator: (identifier) @CPP.LOCAL_VARIABLE)
      (init_declarator declarator: (pointer_declarator declarator: (identifier) @CPP.LOCAL_VARIABLE))
      (init_declarator declarator: (reference_declarator (identifier) @CPP.LOCAL_VARIABLE))
      (init_declarator declarator: (array_declarator declarator: (identifier) @CPP.LOCAL_VARIABLE))
      (pointer_declarator declarator: (identifier) @CPP.LOCAL_VARIABLE)
      (array_declarator declarator: (identifier) @CPP.LOCAL_VARIABLE)
    ]))
(for_statement
  initializer: (declaration
    declarator: [
      (identifier) @CPP.LOCAL_VARIABLE
      (init_declarator declarator: (identifier) @CPP.LOCAL_VARIABLE)
      (init_declarator declarator: (pointer_declarator declarator: (identifier) @CPP.LOCAL_VARIABLE))
      (init_declarator declarator: (reference_declarator (identifier) @CPP.LOCAL_VARIABLE))
      (init_declarator declarator: (array_declarator declarator: (identifier) @CPP.LOCAL_VARIABLE))
      (pointer_declarator declarator: (identifier) @CPP.LOCAL_VARIABLE)
      (array_declarator declarator: (identifier) @CPP.LOCAL_VARIABLE)
    ]))
(for_range_loop
  declarator: [
    (identifier) @CPP.LOCAL_VARIABLE
    (reference_declarator (identifier) @CPP.LOCAL_VARIABLE)
    (structured_binding_declarator (identifier) @CPP.LOCAL_VARIABLE)
  ])
(structured_binding_declarator (identifier) @CPP.LOCAL_VARIABLE)
(declaration_list
  (declaration
    declarator: [
      (identifier) @CPP.GLOBAL_VARIABLE
      (init_declarator declarator: (identifier) @CPP.GLOBAL_VARIABLE)
      (init_declarator declarator: (pointer_declarator declarator: (identifier) @CPP.GLOBAL_VARIABLE))
      (init_declarator declarator: (array_declarator declarator: (identifier) @CPP.GLOBAL_VARIABLE))
      (pointer_declarator declarator: (identifier) @CPP.GLOBAL_VARIABLE)
      (array_declarator declarator: (identifier) @CPP.GLOBAL_VARIABLE)
    ]))
(parameter_declaration
  declarator: [
    (identifier) @CPP.PARAMETER
    (pointer_declarator declarator: (identifier) @CPP.PARAMETER)
    (pointer_declarator declarator: (pointer_declarator declarator: (identifier) @CPP.PARAMETER))
    (reference_declarator (identifier) @CPP.PARAMETER)
    (array_declarator declarator: (identifier) @CPP.PARAMETER)
  ])
(optional_parameter_declaration
  declarator: [
    (identifier) @CPP.PARAMETER
    (pointer_declarator declarator: (identifier) @CPP.PARAMETER)
    (pointer_declarator declarator: (pointer_declarator declarator: (identifier) @CPP.PARAMETER))
    (reference_declarator (identifier) @CPP.PARAMETER)
    (array_declarator declarator: (identifier) @CPP.PARAMETER)
  ])
; a constrained template parameter (`template <Measured Range>`) parses as a parameter
(template_parameter_list
  (parameter_declaration type: (type_identifier) declarator: (identifier) @CPP.TYPE_PARAMETER))
(variadic_parameter_declaration
  declarator: [
    (variadic_declarator (identifier) @CPP.PARAMETER)
    (reference_declarator (variadic_declarator (identifier) @CPP.PARAMETER))
  ])

(function_declarator declarator: (identifier) @CPP.FUNCTION_DECLARATION)
(function_declarator declarator: (qualified_identifier name: (identifier) @CPP.FUNCTION_DECLARATION))
(function_declarator declarator: (field_identifier) @CPP.METHOD_DECLARATION)
((function_declarator
   declarator: (qualified_identifier scope: (namespace_identifier) @_type name: (identifier) @CPP.METHOD_DECLARATION))
 (#match? @_type "^[A-Z]"))
(field_declaration_list
  (_ declarator: (function_declarator declarator: (identifier) @CPP.METHOD_DECLARATION)))
(field_declaration_list
  (template_declaration
    (_ declarator: (function_declarator declarator: [(identifier) (field_identifier)] @CPP.METHOD_DECLARATION))))
(destructor_name) @CPP.METHOD_DECLARATION

(call_expression function: (identifier) @CPP.FUNCTION_CALL)
(call_expression function: (qualified_identifier name: (identifier) @CPP.FUNCTION_CALL))
(call_expression function: (template_function name: (identifier) @CPP.FUNCTION_CALL))
(call_expression function: (qualified_identifier name: (template_function name: (identifier) @CPP.FUNCTION_CALL)))
(call_expression function: (field_expression field: (field_identifier) @CPP.METHOD_CALL))
(call_expression function: (field_expression field: (template_method name: (field_identifier) @CPP.METHOD_CALL)))

(operator_name _ @CPP.OVERLOADED_OPERATOR)

(preproc_def name: (identifier) @CPP.MACRO)
(preproc_function_def name: (identifier) @CPP.MACRO)
(preproc_ifdef name: (identifier) @CPP.MACRO)
(preproc_defined (identifier) @CPP.MACRO)
((call_expression function: (identifier) @CPP.MACRO)
 (#match? @CPP.MACRO "^[A-Z][A-Z0-9_]+$"))
(preproc_params (identifier) @CPP.PARAMETER)

[(attribute_specifier) (attribute_declaration) (ms_declspec_modifier)] @CPP.ATTRIBUTE
