; IVE highlight query for Kotlin (tree-sitter-kotlin-ng). Capture names are colour key ids (src/colors/keys/kotlin.rs).
; For the same node a later pattern wins; otherwise the innermost node wins.

["(" ")"] @KOTLIN_PARENTHESIS
["[" "]"] @KOTLIN_BRACKETS
["{" "}"] @KOTLIN_BRACES
"," @KOTLIN_COMMA
":" @KOTLIN_COLON
"::" @KOTLIN_DOUBLE_COLON
"." @KOTLIN_DOT
"?." @KOTLIN_SAFE_ACCESS
"!!" @KOTLIN_EXCLEXCL
"->" @KOTLIN_ARROW
[
  "=" "+=" "-=" "*=" "/=" "%=" "++" "--" "+" "-" "*" "/" "%" "!" "&&" "||" "==" "!=" "===" "!==" "<" ">" "<="
  ">=" ".." "..<" "?:"
] @KOTLIN_OPERATION_SIGN
(nullable_type "?" @KOTLIN_QUEST)
(non_nullable_type "&" @KOTLIN_AMPERSAND)
(lambda_literal ["{" "}" "->"] @KOTLIN_FUNCTION_LITERAL_BRACES_AND_ARROW)

[(number_literal) (float_literal)] @KOTLIN_NUMBER
(character_literal) @KOTLIN_STRING
(string_literal "\"" @KOTLIN_STRING)
(multiline_string_literal "\"\"\"" @KOTLIN_STRING)
(string_content) @KOTLIN_STRING
((string_content) @KOTLIN_INVALID_STRING_ESCAPE
 (#match? @KOTLIN_INVALID_STRING_ESCAPE "^\\\\"))
(escape_sequence) @KOTLIN_STRING_ESCAPE
(interpolation ["$" "${" "}"] @KOTLIN_STRING_ESCAPE)

(shebang) @KOTLIN_LINE_COMMENT
(line_comment) @KOTLIN_LINE_COMMENT
(block_comment) @KOTLIN_BLOCK_COMMENT
((block_comment) @KOTLIN_DOC_COMMENT
 (#match? @KOTLIN_DOC_COMMENT "^/\\*\\*[^/]"))

; Hard and non-modifier soft keywords; modifiers are `KOTLIN_BUILTIN_ANNOTATION`, as in IntelliJ.
[
  "as" "as?" "by" "catch" "class" "constructor" "do" "dynamic" "else" "file" "finally" "for" "fun" "get" "if"
  "import" "in" "!in" "init" "interface" "is" "!is" "object" "package" "return" "return@" "set" "super" "super@"
  "this" "this@" "throw" "try" "typealias" "when" "where" "while"
] @KOTLIN_KEYWORD
"val" @KOTLIN_KEYWORD_VAL
"var" @KOTLIN_KEYWORD_VAR
((identifier) @KOTLIN_KEYWORD
 (#any-of? @KOTLIN_KEYWORD "true" "false" "null" "break" "continue"))
(navigation_expression "::" (identifier) @KOTLIN_KEYWORD
 (#eq? @KOTLIN_KEYWORD "class"))
[
  (visibility_modifier) (class_modifier) (member_modifier) (function_modifier) (property_modifier)
  (inheritance_modifier) (parameter_modifier) (platform_modifier) (variance_modifier) (reification_modifier)
  "companion"
] @KOTLIN_BUILTIN_ANNOTATION
(variance_modifier "in" @KOTLIN_KEYWORD)

(label) @KOTLIN_LABEL
((label) @KOTLIN_KEYWORD
 (#match? @KOTLIN_KEYWORD "^(break|continue)@$"))
(labeled_expression (identifier) @KOTLIN_LABEL)
(return_expression label: (identifier) @KOTLIN_LABEL)
(this_expression "this@" (identifier) @KOTLIN_LABEL)
(super_expression "super@" (identifier) @KOTLIN_LABEL)

(user_type (identifier) @KOTLIN_CLASS)
(type_parameter (identifier) @KOTLIN_TYPE_PARAMETER)
(type_constraint (identifier) @KOTLIN_TYPE_PARAMETER)
(class_declaration name: (identifier) @KOTLIN_CLASS)
(class_declaration "interface" name: (identifier) @KOTLIN_TRAIT)
(class_declaration (modifiers (inheritance_modifier "abstract")) name: (identifier) @KOTLIN_ABSTRACT_CLASS)
(class_declaration (modifiers (class_modifier "sealed")) name: (identifier) @KOTLIN_ABSTRACT_CLASS)
(class_declaration (modifiers (class_modifier "data")) name: (identifier) @KOTLIN_DATA_CLASS)
(class_declaration (modifiers (class_modifier "enum")) name: (identifier) @KOTLIN_ENUM)
(class_declaration (modifiers (class_modifier "annotation")) name: (identifier) @KOTLIN_ANNOTATION)
(object_declaration name: (identifier) @KOTLIN_OBJECT)
(object_declaration (modifiers (class_modifier "data")) name: (identifier) @KOTLIN_DATA_OBJECT)
(companion_object name: (identifier) @KOTLIN_OBJECT)
(type_alias type: (identifier) @KOTLIN_TYPE_ALIAS)
(enum_entry (identifier) @KOTLIN_ENUM_ENTRY)

(navigation_expression ["." "?."] (identifier) @KOTLIN_INSTANCE_PROPERTY)
((navigation_expression (identifier) @KOTLIN_CLASS)
 (#match? @KOTLIN_CLASS "^[A-Z].*[a-z]"))
; a property is a package property unless it sits in a body (local) or a class body (instance property)
; no pattern is rooted at the file node, which would make the query of any line walk every top-level node of the file
(property_declaration (variable_declaration (identifier) @KOTLIN_PACKAGE_PROPERTY))
(property_declaration (variable_declaration (identifier) @KOTLIN_PACKAGE_PROPERTY_CUSTOM_PROPERTY_DECLARATION)
  [(getter) (setter)])
[(block (property_declaration (variable_declaration (identifier) @KOTLIN_LOCAL_VARIABLE)))
 (lambda_literal (property_declaration (variable_declaration (identifier) @KOTLIN_LOCAL_VARIABLE)))
 (when_entry (property_declaration (variable_declaration (identifier) @KOTLIN_LOCAL_VARIABLE)))
 (for_statement (property_declaration (variable_declaration (identifier) @KOTLIN_LOCAL_VARIABLE)))
 (while_statement (property_declaration (variable_declaration (identifier) @KOTLIN_LOCAL_VARIABLE)))
 (do_while_statement (property_declaration (variable_declaration (identifier) @KOTLIN_LOCAL_VARIABLE)))]
(property_declaration (multi_variable_declaration (variable_declaration (identifier) @KOTLIN_LOCAL_VARIABLE)))
(for_statement (variable_declaration (identifier) @KOTLIN_LOCAL_VARIABLE))
(for_statement (multi_variable_declaration (variable_declaration (identifier) @KOTLIN_LOCAL_VARIABLE)))
(catch_block (identifier) @KOTLIN_PARAMETER)
(class_body (property_declaration (variable_declaration (identifier) @KOTLIN_INSTANCE_PROPERTY)))
(class_body
  (property_declaration (variable_declaration (identifier) @KOTLIN_INSTANCE_PROPERTY_CUSTOM_PROPERTY_DECLARATION)
    [(getter) (setter)]))
(getter (function_body (identifier) @KOTLIN_BACKING_FIELD_VARIABLE
 (#eq? @KOTLIN_BACKING_FIELD_VARIABLE "field")))
(setter (function_body (block (assignment left: (identifier) @KOTLIN_BACKING_FIELD_VARIABLE)
 (#eq? @KOTLIN_BACKING_FIELD_VARIABLE "field"))))

(parameter (identifier) @KOTLIN_PARAMETER)
(class_parameter (identifier) @KOTLIN_PARAMETER)
(class_parameter ["val" "var"] (identifier) @KOTLIN_INSTANCE_PROPERTY)
(lambda_parameters (variable_declaration (identifier) @KOTLIN_PARAMETER))
(setter (identifier) @KOTLIN_PARAMETER)
((identifier) @KOTLIN_CLOSURE_DEFAULT_PARAMETER
 (#eq? @KOTLIN_CLOSURE_DEFAULT_PARAMETER "it"))

(function_declaration name: (identifier) @KOTLIN_FUNCTION_DECLARATION)
(call_expression (identifier) @KOTLIN_FUNCTION_CALL)
(call_expression (navigation_expression (identifier) @KOTLIN_FUNCTION_CALL .))
(callable_reference (identifier) @KOTLIN_FUNCTION_CALL)
((call_expression (identifier) @KOTLIN_CONSTRUCTOR)
 (#match? @KOTLIN_CONSTRUCTOR "^[A-Z]"))
((call_expression (navigation_expression (identifier) @KOTLIN_CONSTRUCTOR .))
 (#match? @KOTLIN_CONSTRUCTOR "^[A-Z]"))
(delegation_specifier (constructor_invocation (user_type (identifier) @KOTLIN_CONSTRUCTOR)))
(value_argument (identifier) @KOTLIN_NAMED_ARGUMENT "=" @KOTLIN_NAMED_ARGUMENT)

(annotation "@" @KOTLIN_ANNOTATION)
(annotation (user_type (identifier) @KOTLIN_ANNOTATION))
(annotation (constructor_invocation (user_type (identifier) @KOTLIN_ANNOTATION)))
(annotation (use_site_target) @KOTLIN_ANNOTATION)
(annotation (use_site_target ":" @KOTLIN_ANNOTATION))
(file_annotation ["@" "file" ":"] @KOTLIN_ANNOTATION)
(file_annotation (user_type (identifier) @KOTLIN_ANNOTATION))
(file_annotation (constructor_invocation (user_type (identifier) @KOTLIN_ANNOTATION)))
(annotation
  (constructor_invocation
    (value_arguments
      (value_argument (identifier) @KOTLIN_ANNOTATION_ATTRIBUTE_NAME_ATTRIBUTES "=" @KOTLIN_OPERATION_SIGN))))
