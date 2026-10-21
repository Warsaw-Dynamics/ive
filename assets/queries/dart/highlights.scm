; IVE highlight query for Dart. Capture names are colour key ids (src/colors/keys/dart.rs).
; For the same node a later pattern wins; otherwise the innermost node wins.

["(" ")"] @DART.PARENTHESES
["[" "]"] @DART.BRACKETS
["{" "}"] @DART.BRACES
"," @DART.COMMA
["." "?."] @DART.DOT
";" @DART.SEMICOLON
[
  "=" "+=" "-=" "*=" "/=" "%=" "~/=" "<<=" ">>=" ">>>=" "&=" "^=" "|=" "??=" "==" "!=" "+" "-" "*" "/" "%" "~/"
  "<<" ">>" ">>>" "&" "^" "|" "&&" "||" "??" "!" "~" "++" "--" ".." "?.." "..." "...?" "=>" "?" ":"
] @DART.OPERATOR
[(relational_operator) (prefix_operator) (negate_operator) (binary_operator)] @DART.OPERATOR

[(decimal_integer_literal) (hex_integer_literal) (decimal_floating_point_literal)] @DART.NUMBER
(string_literal) @DART.STRING
(escape_sequence) @DART.ESCAPE
(template_substitution (_) @TEXT)
(template_substitution ["$" "{" "}"] @DART.INTERPOLATION)
(identifier_dollar_escaped) @DART.INTERPOLATION
(symbol_literal) @DART.SYMBOL
(comment) @DART.LINE_COMMENT
(block_comment) @DART.BLOCK_COMMENT
((comment) @DART.DOC_COMMENT
 (#match? @DART.DOC_COMMENT "^///"))
(documentation_block_comment) @DART.DOC_COMMENT

[
  "abstract" "as" "assert" "async" "async*" "augment" "await" "base" "break" "case" "catch" "class" "const"
  "continue" "covariant" "default" "deferred" "do" "else" "enum" "export" "extends" "extension" "external"
  "factory" "final" "finally" "for" "Function" "get" "hide" "if" "implements" "import" "in" "inline" "interface"
  "is" "late" "library" "mixin" "native" "new" "on" "operator" "part" "required" "return" "sealed" "set" "show"
  "static" "switch" "sync*" "throw" "try" "type" "typedef" "var" "when" "while" "with" "yield"
] @DART.KEYWORD
[(true) (false) (null_literal) (void_type) (is_operator) "this" "super"] @DART.KEYWORD
((type_identifier) @DART.KEYWORD
 (#eq? @DART.KEYWORD "dynamic"))

; Uses: `a.b` is a field, `Type.member` starts with a class, an upper-case name a constant.
((identifier) @DART.CONSTANT
 (#match? @DART.CONSTANT "^[A-Z][A-Z0-9_]+$"))
(member_expression property: (identifier) @DART.FIELD)
(null_aware_member_expression property: (identifier) @DART.FIELD)
(cascade_member_expression property: (identifier) @DART.FIELD)
((member_expression object: (identifier) @DART.CLASS)
 (#match? @DART.CLASS "^[A-Z]"))
(import_specification alias: (identifier) @DART.IMPORT_PREFIX)

; Types and their declarations.
(type_identifier) @DART.CLASS
(type_parameter name: (type_identifier) @DART.TYPE_PARAMETER)
(class_declaration name: (identifier) @DART.CLASS)
(class_declaration "interface" name: (identifier) @DART.INTERFACE)
(mixin_declaration name: (identifier) @DART.CLASS)
(extension_declaration name: (identifier) @DART.CLASS)
(extension_type_declaration name: (extension_type_name (identifier) @DART.CLASS))
(enum_declaration name: (identifier) @DART.ENUM)
(enum_constant name: (identifier) @DART.CONSTANT)
(type_alias (type_identifier) @DART.CLASS)

; Variables: top-level, fields (static or not), locals, parameters.
(top_level_variable_declaration (static_final_declaration_list (static_final_declaration name: (identifier) @DART.TOP_LEVEL_VARIABLE)))
(top_level_variable_declaration (initialized_identifier_list (initialized_identifier name: (identifier) @DART.TOP_LEVEL_VARIABLE)))
(class_member (declaration (static_final_declaration_list (static_final_declaration name: (identifier) @DART.FIELD))))
(class_member (declaration (initialized_identifier_list (initialized_identifier name: (identifier) @DART.FIELD))))
(class_member (declaration (identifier_list (identifier) @DART.FIELD)))
(class_member (declaration "static" (static_final_declaration_list (static_final_declaration name: (identifier) @DART.STATIC_FIELD))))
(class_member (declaration "static" (initialized_identifier_list (initialized_identifier name: (identifier) @DART.STATIC_FIELD))))
(initialized_variable_definition name: (identifier) @DART.LOCAL_VARIABLE)
(for_statement name: (identifier) @DART.LOCAL_VARIABLE)
(variable_pattern (identifier) @DART.LOCAL_VARIABLE)
(catch_clause (identifier) @DART.LOCAL_VARIABLE)
(formal_parameter name: (identifier) @DART.PARAMETER)
(constructor_param (identifier) @DART.PARAMETER)
(super_formal_parameter (identifier) @DART.PARAMETER)
(named_argument (label (identifier) @DART.PARAMETER))
(field_initializer (identifier) @DART.FIELD)

; Functions, methods and constructors.
(function_signature name: (identifier) @DART.FUNCTION_DECLARATION)
(getter_signature name: (identifier) @DART.FUNCTION_DECLARATION)
(setter_signature name: (identifier) @DART.FUNCTION_DECLARATION)
(method_signature (function_signature name: (identifier) @DART.METHOD_DECLARATION))
(method_signature (getter_signature name: (identifier) @DART.METHOD_DECLARATION))
(method_signature (setter_signature name: (identifier) @DART.METHOD_DECLARATION))
(class_member (declaration (function_signature name: (identifier) @DART.METHOD_DECLARATION)))
(class_member (declaration (getter_signature name: (identifier) @DART.METHOD_DECLARATION)))
(class_member (declaration (setter_signature name: (identifier) @DART.METHOD_DECLARATION)))
(constructor_signature name: (identifier) @DART.CONSTRUCTOR)
(constant_constructor_signature (identifier) @DART.CONSTRUCTOR)
(factory_constructor_signature (identifier) @DART.CONSTRUCTOR)
(redirecting_factory_constructor_signature (identifier) @DART.CONSTRUCTOR)

(call_expression function: (identifier) @DART.FUNCTION_CALL)
((call_expression function: (identifier) @DART.CONSTRUCTOR)
 (#match? @DART.CONSTRUCTOR "^[A-Z]"))
(call_expression function: (member_expression property: (identifier) @DART.METHOD_CALL))
(call_expression function: (null_aware_member_expression property: (identifier) @DART.METHOD_CALL))
(cascade_call_expression property: (identifier) @DART.METHOD_CALL)
(new_expression (type (type_identifier) @DART.CONSTRUCTOR))
(const_object_expression (type (type_identifier) @DART.CONSTRUCTOR))

(annotation "@" @DART.ANNOTATION)
(annotation name: (_) @DART.ANNOTATION)
(labeled_statement (identifier) @DART.LABEL)
(break_statement (identifier) @DART.LABEL)
(continue_statement (identifier) @DART.LABEL)

((static_final_declaration name: (identifier) @DART.CONSTANT)
 (#match? @DART.CONSTANT "^[A-Z][A-Z0-9_]+$"))
((initialized_identifier name: (identifier) @DART.CONSTANT)
 (#match? @DART.CONSTANT "^[A-Z][A-Z0-9_]+$"))
