; IVE highlight query for Java. Capture names are colour key ids (src/colors/keys.rs).
; For the same node a later pattern wins; otherwise the innermost node wins.

["(" ")"] @JAVA_PARENTH
["[" "]"] @JAVA_BRACKETS
["{" "}"] @JAVA_BRACES
"," @JAVA_COMMA
"." @JAVA_DOT
";" @JAVA_SEMICOLON
["=" "+" "-" "*" "/" "%" "==" "!=" "<=" ">=" "&&" "||" "!" "?" ":" "->" "++" "--" "+=" "-="] @JAVA_OPERATION_SIGN

[
  (decimal_integer_literal)
  (hex_integer_literal)
  (octal_integer_literal)
  (binary_integer_literal)
  (decimal_floating_point_literal)
  (hex_floating_point_literal)
] @JAVA_NUMBER
[(string_literal) (character_literal)] @JAVA_STRING
(escape_sequence) @JAVA_VALID_STRING_ESCAPE
(line_comment) @JAVA_LINE_COMMENT
(block_comment) @JAVA_BLOCK_COMMENT
((block_comment) @JAVA_DOC_COMMENT
 (#match? @JAVA_DOC_COMMENT "^/\\*\\*"))

[
  "abstract" "assert" "break" "case" "catch" "class" "continue" "default" "do" "else" "enum" "exports"
  "extends" "final" "finally" "for" "if" "implements" "import" "instanceof" "interface" "module" "native"
  "new" "non-sealed" "open" "opens" "package" "permits" "private" "protected" "provides" "public"
  "requires" "record" "return" "sealed" "static" "strictfp" "switch" "synchronized" "throw" "throws"
  "to" "transient" "transitive" "try" "uses" "volatile" "when" "while" "with" "yield"
] @JAVA_KEYWORD
[
  (true) (false) (null_literal) (this) (super)
  (boolean_type) (integral_type) (floating_point_type) (void_type)
] @JAVA_KEYWORD

(type_identifier) @CLASS_NAME_ATTRIBUTES
((type_identifier) @JAVA_KEYWORD
 (#eq? @JAVA_KEYWORD "var"))
((identifier) @STATIC_FINAL_FIELD_ATTRIBUTES
 (#match? @STATIC_FINAL_FIELD_ATTRIBUTES "^[A-Z][A-Z0-9_]+$"))
((method_invocation object: (identifier) @CLASS_NAME_ATTRIBUTES)
 (#match? @CLASS_NAME_ATTRIBUTES "^[A-Z]"))
((field_access object: (identifier) @CLASS_NAME_ATTRIBUTES)
 (#match? @CLASS_NAME_ATTRIBUTES "^[A-Z]"))
(class_declaration name: (identifier) @CLASS_NAME_ATTRIBUTES)
(interface_declaration name: (identifier) @INTERFACE_NAME_ATTRIBUTES)
(enum_declaration name: (identifier) @ENUM_NAME_ATTRIBUTES)
(record_declaration name: (identifier) @RECORD_NAME_ATTRIBUTES)
(type_parameter (type_identifier) @TYPE_PARAMETER_NAME_ATTRIBUTES)

(field_declaration
  declarator: (variable_declarator name: (identifier) @INSTANCE_FIELD_ATTRIBUTES))
(field_declaration (modifiers "final")
  declarator: (variable_declarator name: (identifier) @INSTANCE_FINAL_FIELD_ATTRIBUTES))
(field_declaration (modifiers "static")
  declarator: (variable_declarator name: (identifier) @STATIC_FIELD_ATTRIBUTES))
(field_declaration (modifiers "static" "final")
  declarator: (variable_declarator name: (identifier) @STATIC_FINAL_FIELD_ATTRIBUTES))
(field_access object: (this) field: (identifier) @INSTANCE_FIELD_ATTRIBUTES)

(method_invocation name: (identifier) @METHOD_CALL_ATTRIBUTES)
(method_declaration name: (identifier) @METHOD_DECLARATION_ATTRIBUTES)
(method_declaration (modifiers "static") name: (identifier) @STATIC_METHOD_ATTRIBUTES)
(constructor_declaration name: (identifier) @CONSTRUCTOR_DECLARATION_ATTRIBUTES)
(object_creation_expression type: (type_identifier) @CONSTRUCTOR_CALL_ATTRIBUTES)

(formal_parameter name: (identifier) @PARAMETER_ATTRIBUTES)
(lambda_expression parameters: (identifier) @LAMBDA_PARAMETER_ATTRIBUTES)
(lambda_expression parameters: (inferred_parameters (identifier) @LAMBDA_PARAMETER_ATTRIBUTES))
(local_variable_declaration
  declarator: (variable_declarator name: (identifier) @LOCAL_VARIABLE_ATTRIBUTES))

(annotation "@" @ANNOTATION_NAME_ATTRIBUTES name: (_) @ANNOTATION_NAME_ATTRIBUTES)
(marker_annotation "@" @ANNOTATION_NAME_ATTRIBUTES name: (_) @ANNOTATION_NAME_ATTRIBUTES)
(element_value_pair key: (identifier) @ANNOTATION_ATTRIBUTE_NAME_ATTRIBUTES)
