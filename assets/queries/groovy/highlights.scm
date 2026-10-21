; IVE highlight query for Groovy, Gradle and Jenkinsfile. Capture names are colour key ids (src/colors/keys/groovy.rs);
; IntelliJ's Groovy ids contain spaces, which a capture name spells as `_` (`@Groovy_var` is the key "Groovy var").
; For the same node a later pattern wins; otherwise the innermost node wins.

["(" ")"] @Parentheses
["[" "]"] @Brackets
["{" "}"] @Braces
[
  "=" "+=" "-=" "*=" "/=" "%=" "&=" "|=" "^=" "<<=" ">>=" ">>>=" "++" "--" "+" "-" "*" "**" "/" "%" "!" "~" "&&"
  "||" "&" "|" "^" "<<" ">>" ">>>" "==" "!=" "<" ">" "<=" ">=" "?" ":" ".." "..." "->" "::"
] @Operation_sign
(closure ["{" "}"] @Closure_braces)
(closure (lambda_expression "->" @Closure_braces))
(lambda_expression "->" @Lambda_braces body: (closure ["{" "}"] @Lambda_braces))
; The grammar reads `if` and `else` blocks as closures.
(if_statement consequence: (closure ["{" "}"] @Braces))

[
  (decimal_integer_literal)
  (hex_integer_literal)
  (octal_integer_literal)
  (binary_integer_literal)
  (decimal_floating_point_literal)
  (hex_floating_point_literal)
] @Number
(character_literal) @String
(string_literal) @GString
(escape_sequence) @Valid_string_escape
(shebang) @Line_comment
(line_comment) @Line_comment
(block_comment) @Block_comment
((block_comment) @Groovydoc_comment
 (#match? @Groovydoc_comment "^/\\*\\*[^/]"))

[
  "abstract" "as" "assert" "break" "case" "catch" "class" "continue" "def" "default" "do" "else" "enum" "exports"
  "extends" "final" "finally" "for" "if" "implements" "import" "in" "instanceof" "interface" "module" "native"
  "new" "non-sealed" "open" "opens" "package" "permits" "private" "protected" "provides" "public" "record"
  "requires" "return" "sealed" "static" "strictfp" "switch" "synchronized" "throw" "throws" "to" "transient"
  "transitive" "try" "uses" "volatile" "when" "while" "with" "yield"
] @GROOVY_KEYWORD
[(true) (false) (null_literal) (this) (super) (boolean_type) (integral_type) (floating_point_type) (void_type)]
  @GROOVY_KEYWORD

(type_identifier) @Class
((type_identifier) @GROOVY_KEYWORD
 (#any-of? @GROOVY_KEYWORD "def" "var"))
((identifier) @Static_field
 (#match? @Static_field "^[A-Z][A-Z0-9_]+$"))
((method_invocation object: (identifier) @Class)
 (#match? @Class "^[A-Z]"))
((field_access object: (identifier) @Class)
 (#match? @Class "^[A-Z]"))
(class_declaration name: (identifier) @Class)
(class_declaration (modifiers "abstract") name: (identifier) @Abstract_class_name)
(interface_declaration name: (identifier) @Interface_name)
(super_interfaces (type_list (type_identifier) @Interface_name))
(extends_interfaces (type_list (type_identifier) @Interface_name))
(enum_declaration name: (identifier) @Enum_name)
(annotation_type_declaration "@interface" @GROOVY_KEYWORD name: (identifier) @Annotation)
(type_parameter (type_identifier) @Type_parameter)

(field_access field: (identifier) @Instance_property_reference_ID)
((field_access object: (identifier) @_class field: (identifier) @Static_property_reference_ID)
 (#match? @_class "^[A-Z]"))
(field_declaration declarator: (variable_declarator name: (identifier) @Instance_field))
(field_declaration (modifiers "static") declarator: (variable_declarator name: (identifier) @Static_field))
(enum_constant name: (identifier) @Static_field)
(field_access object: (this) field: (identifier) @Instance_field)

(method_invocation name: (identifier) @Method_call)
(juxt_function_call name: (identifier) @Method_call)
((method_invocation object: (identifier) @_class name: (identifier) @Static_method_access)
 (#match? @_class "^[A-Z]"))
(method_declaration name: (identifier) @Groovy_method_declaration)
(function_definition name: (identifier) @Groovy_method_declaration)
(annotation_type_element_declaration name: (identifier) @Groovy_method_declaration)
(constructor_declaration name: (identifier) @Groovy_constructor_declaration)
(object_creation_expression type: (type_identifier) @Groovy_constructor_call (argument_list))
(object_creation_expression type: (generic_type (type_identifier) @Groovy_constructor_call))
(object_creation_expression type: (type_identifier) @Anonymous_class_name (class_body))

(formal_parameter name: (identifier) @Groovy_parameter)
(catch_formal_parameter name: (identifier) @Groovy_parameter)
(lambda_expression parameters: (identifier) @Groovy_parameter)
(lambda_expression parameters: (inferred_parameters (identifier) @Groovy_parameter))
(lambda_expression parameters: (formal_parameters (formal_parameter name: (identifier) @Groovy_parameter)))
((identifier) @Groovy_parameter
 (#eq? @Groovy_parameter "it"))
(local_variable_declaration declarator: (variable_declarator name: (identifier) @Groovy_var))
(enhanced_for_statement name: (identifier) @Groovy_var)

(map_item key: (identifier) @Map_key)
(map_item key: (character_literal) @Map_key)
(labeled_statement (identifier) @Label)
(break_statement (identifier) @Label)
(continue_statement (identifier) @Label)

(annotation "@" @Annotation name: (_) @Annotation)
(marker_annotation "@" @Annotation name: (_) @Annotation)
(element_value_pair key: (identifier) @Anotation_attribute_name)
