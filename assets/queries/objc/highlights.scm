; IVE highlight query for Objective-C. Capture names are colour key ids (src/colors/keys/).
; For the same node a later pattern wins; otherwise the innermost node wins.
; Variables are told apart only where they are declared: tree-sitter does not resolve names.

["(" ")"] @OBJC.PARENTHESES
["[" "]"] @OBJC.BRACKETS
["{" "}"] @OBJC.BRACES
"," @OBJC.COMMA
"." @OBJC.DOT
";" @OBJC.SEMICOLON
[
  "=" "+" "-" "*" "/" "%" "==" "!=" "<" ">" "<=" ">=" "&&" "||" "!" "&" "|" "^" "~" "<<" ">>" "?" ":"
  "->" "++" "--" "+=" "-=" "*=" "/=" "%=" "&=" "|=" "^=" "<<=" ">>=" "..."
] @OBJC.OPERATION_SIGN

(number_literal) @OBJC.NUMBER
[(string_literal) (system_lib_string)] @OBJC.STRING
(char_literal) @OBJC.CHAR
(escape_sequence) @OBJC.VALID_STRING_ESCAPE
((comment) @OBJC.LINE_COMMENT
 (#match? @OBJC.LINE_COMMENT "^//"))
((comment) @OBJC.BLOCK_COMMENT
 (#match? @OBJC.BLOCK_COMMENT "^/\\*"))
((comment) @OBJC.DOC_COMMENT
 (#match? @OBJC.DOC_COMMENT "^(///|/\\*\\*[^/])"))

[
  "_Alignas" "_Alignof" "_Atomic" "_Generic" "_Noreturn" "__attribute__" "__attribute" "__declspec" "__extension__"
  "__inline" "__inline__" "__restrict__" "__thread" "__volatile__" "__typeof" "__typeof__" "__block" "__weak"
  "__strong" "__unsafe_unretained" "__autoreleasing" "__bridge" "__bridge_retained" "__bridge_transfer" "__kindof"
  "_Nonnull" "_Nullable" "_Null_unspecified" "__covariant" "__contravariant" "alignas" "alignof" "asm" "__asm"
  "__asm__" "auto" "break" "bycopy" "byref" "case" "const" "constexpr" "continue" "default" "do" "else" "enum"
  "extern" "for" "goto" "if" "in" "inline" "inout" "noreturn" "nullable" "offsetof" "oneway" "out" "register"
  "restrict" "return" "sizeof" "static" "struct" "switch" "thread_local" "typedef" "typeof" "union" "volatile" "while"
] @OBJC.KEYWORD
[(true) (false) (null)] @OBJC.KEYWORD
((identifier) @OBJC.KEYWORD
 (#any-of? @OBJC.KEYWORD "self" "super" "nil" "Nil" "YES" "NO"))
(property_attribute (identifier) @OBJC.KEYWORD)
(visibility_specification) @OBJC.AT_KEYWORD
[
  "@autoreleasepool" "@available" "@catch" "@compatibility_alias" "@defs" "@dynamic" "@encode" "@end" "@finally"
  "@implementation" "@import" "@interface" "@optional" "@property" "@protocol" "@required" "@selector"
  "@synchronized" "@synthesize" "@throw" "@try"
] @OBJC.AT_KEYWORD
(class_declaration "@" @OBJC.AT_KEYWORD "class" @OBJC.AT_KEYWORD)
(at_expression "@" @OBJC.AT_KEYWORD)
[(primitive_type) (sized_type_specifier) (typedefed_specifier) "BOOL" "Class" "id" "IMP" "SEL"] @OBJC.PRIMITIVE_TYPE

[
  "#define" "#elif" "#elifdef" "#elifndef" "#else" "#endif" "#if" "#ifdef" "#ifndef" "#import" "#include" "#undef"
  "defined"
] @OBJC.PREPROCESSOR_DIRECTIVE
(preproc_directive) @OBJC.PREPROCESSOR_DIRECTIVE

(type_identifier) @OBJC.CLASS
((type_identifier) @OBJC.PRIMITIVE_TYPE
 (#eq? @OBJC.PRIMITIVE_TYPE "instancetype"))
(class_interface . (identifier) @OBJC.CLASS)
(class_interface superclass: (identifier) @OBJC.CLASS)
(class_interface category: (identifier) @OBJC.CLASS)
(class_implementation . (identifier) @OBJC.CLASS)
(class_implementation superclass: (identifier) @OBJC.CLASS)
(class_implementation category: (identifier) @OBJC.CLASS)
(class_declaration (identifier) @OBJC.CLASS)
((message_expression receiver: (identifier) @OBJC.CLASS)
 (#match? @OBJC.CLASS "^[A-Z]"))
(protocol_declaration . (identifier) @OBJC.PROTOCOL)
(protocol_forward_declaration (identifier) @OBJC.PROTOCOL)
(protocol_reference_list (identifier) @OBJC.PROTOCOL)
(parameterized_arguments (type_name (type_identifier) @OBJC.PROTOCOL))

((identifier) @OBJC.CONSTANT
 (#match? @OBJC.CONSTANT "^[A-Z][A-Z0-9_]+$"))
(enumerator name: (identifier) @OBJC.ENUM_CONSTANT)
(statement_identifier) @OBJC.LABEL

(field_identifier) @OBJC.PROPERTY
(field_declaration declarator: (field_identifier) @OBJC.INSTANCE_VARIABLE)
(instance_variable
  (struct_declaration
    (struct_declarator [(identifier) (pointer_declarator declarator: (identifier))] @OBJC.INSTANCE_VARIABLE)))
(property_declaration
  (struct_declaration
    (struct_declarator [
      (identifier) @OBJC.PROPERTY
      (pointer_declarator declarator: (identifier) @OBJC.PROPERTY)
    ])))
(property_implementation (identifier) @OBJC.PROPERTY)

; a declaration is global unless it sits in a block or a for loop's initializer; no pattern is rooted at the
; file node, which would make the query of any line walk every top-level node of the file
(declaration
  declarator: [
    (identifier) @OBJC.GLOBAL_VARIABLE
    (init_declarator declarator: (identifier) @OBJC.GLOBAL_VARIABLE)
    (init_declarator declarator: (pointer_declarator declarator: (identifier) @OBJC.GLOBAL_VARIABLE))
    (pointer_declarator declarator: (identifier) @OBJC.GLOBAL_VARIABLE)
    (array_declarator declarator: (identifier) @OBJC.GLOBAL_VARIABLE)
  ])
(compound_statement
  (declaration
    declarator: [
      (identifier) @OBJC.LOCAL_VARIABLE
      (init_declarator declarator: (identifier) @OBJC.LOCAL_VARIABLE)
      (init_declarator declarator: (pointer_declarator declarator: (identifier) @OBJC.LOCAL_VARIABLE))
      (pointer_declarator declarator: (identifier) @OBJC.LOCAL_VARIABLE)
      (array_declarator declarator: (identifier) @OBJC.LOCAL_VARIABLE)
    ]))
(for_statement
  initializer: (declaration
    declarator: [
      (identifier) @OBJC.LOCAL_VARIABLE
      (init_declarator declarator: (identifier) @OBJC.LOCAL_VARIABLE)
      (init_declarator declarator: (pointer_declarator declarator: (identifier) @OBJC.LOCAL_VARIABLE))
      (pointer_declarator declarator: (identifier) @OBJC.LOCAL_VARIABLE)
      (array_declarator declarator: (identifier) @OBJC.LOCAL_VARIABLE)
    ]))
(parameter_declaration
  declarator: [
    (identifier) @OBJC.PARAMETER
    (pointer_declarator declarator: (identifier) @OBJC.PARAMETER)
    (pointer_declarator declarator: (pointer_declarator declarator: (identifier) @OBJC.PARAMETER))
  ])
(method_parameter (identifier) @OBJC.PARAMETER)
(method_parameter declarator: (identifier) @OBJC.PARAMETER)

(function_declarator declarator: (identifier) @OBJC.FUNCTION_DECLARATION)
(call_expression function: (identifier) @OBJC.FUNCTION_CALL)
[(method_declaration (identifier) @OBJC.METHOD_DECLARATION) (method_definition (identifier) @OBJC.METHOD_DECLARATION)]
(message_expression method: (identifier) @OBJC.MESSAGE_SEND)
; the colons belong to the selector: `setObject:forKey:`
(method_parameter ":" @OBJC.METHOD_DECLARATION)
(message_expression ":" @OBJC.MESSAGE_SEND)
[(method_declaration ["+" "-"] @OBJC.KEYWORD) (method_definition ["+" "-"] @OBJC.KEYWORD)]
(selector_expression) @OBJC.SELECTOR

(preproc_def name: (identifier) @OBJC.MACRO)
(preproc_function_def name: (identifier) @OBJC.MACRO)
(preproc_ifdef name: (identifier) @OBJC.MACRO)
((call_expression function: (identifier) @OBJC.MACRO)
 (#match? @OBJC.MACRO "^[A-Z][A-Z0-9_]+$"))
(preproc_params (identifier) @OBJC.PARAMETER)

[(attribute_specifier) (attribute_declaration) (availability_attribute_specifier)] @OBJC.ATTRIBUTE
