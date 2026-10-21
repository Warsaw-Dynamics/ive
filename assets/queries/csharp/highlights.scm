; IVE highlight query for C#. Capture names are colour key ids (src/colors/keys/csharp.rs).
; For the same node a later pattern wins; otherwise the innermost node wins.

["(" ")"] @CS.PARENTHESES
["[" "]"] @CS.BRACKETS
["{" "}"] @CS.BRACES
"," @CS.COMMA
"." @CS.DOT
";" @CS.SEMICOLON
[
  "--" "-" "-=" "&" "&=" "&&" "+" "++" "+=" "<" "<=" "<<" "<<=" "=" "==" "!" "!=" "=>" ">" ">=" ">>" ">>="
  ">>>" ">>>=" "|" "|=" "||" "?" "??" "??=" "^" "^=" "~" "*" "*=" "/" "/=" "%" "%=" ":" ".." "->" "::"
] @CS.OPERATOR

[(integer_literal) (real_literal)] @CS.NUMBER
[(string_content) (string_literal_content) (interpolation_quote) (interpolation_start)] @CS.STRING
(string_literal "\"" @CS.STRING)
(string_literal (string_literal_encoding) @CS.STRING)
(interpolated_string_expression "\"" @CS.STRING)
[(verbatim_string_literal) (raw_string_literal) (character_literal)] @CS.STRING
(escape_sequence) @CS.ESCAPE
(interpolation_brace) @CS.INTERPOLATION
(interpolation_format_clause) @CS.STRING
(comment) @CS.LINE_COMMENT
((comment) @CS.BLOCK_COMMENT
 (#match? @CS.BLOCK_COMMENT "^/\\*"))
((comment) @CS.DOC_COMMENT
 (#match? @CS.DOC_COMMENT "^(///[^/]|///$|/\\*\\*[^/])"))
[
  (preproc_if) (preproc_elif) (preproc_else) (preproc_region) (preproc_endregion) (preproc_define)
  (preproc_undef) (preproc_warning) (preproc_error) (preproc_line) (preproc_pragma) (preproc_nullable)
] @CS.PREPROCESSOR
["#if" "#elif" "#else" "#endif" "#region" "#endregion" "#define" "#undef" "#warning" "#error" "#line" "#pragma" "#nullable"]
  @CS.PREPROCESSOR

[
  "abstract" "add" "alias" "and" "as" "ascending" "async" "await" "base" "break" "by" "case" "catch" "checked"
  "class" "const" "continue" "default" "delegate" "descending" "do" "else" "enum" "equals" "event" "explicit"
  "extern" "file" "finally" "fixed" "for" "foreach" "from" "get" "global" "goto" "group" "if" "implicit" "in"
  "init" "interface" "internal" "into" "is" "join" "let" "lock" "namespace" "new" "not" "notnull" "on"
  "operator" "or" "orderby" "out" "override" "params" "partial" "private" "protected" "public" "readonly"
  "record" "ref" "remove" "required" "return" "scoped" "sealed" "select" "set" "sizeof" "stackalloc" "static"
  "struct" "switch" "this" "throw" "try" "typeof" "unchecked" "unmanaged" "unsafe" "using" "var" "virtual"
  "volatile" "when" "where" "while" "with" "yield"
] @CS.KEYWORD
[(boolean_literal) (null_literal) (predefined_type) (implicit_type) (discard)] @CS.KEYWORD

((identifier) @CS.CONSTANT
 (#match? @CS.CONSTANT "^[A-Z][A-Z0-9_]+$"))
(member_access_expression name: (identifier) @CS.FIELD)
((member_access_expression name: (identifier) @CS.PROPERTY)
 (#match? @CS.PROPERTY "^[A-Z]"))
(member_access_expression expression: "this" name: (identifier) @CS.FIELD)
(member_binding_expression name: (identifier) @CS.PROPERTY)
((member_access_expression expression: (identifier) @CS.CLASS)
 (#match? @CS.CLASS "^[A-Z]"))

(qualified_name name: (identifier) @CS.CLASS)
(generic_name (identifier) @CS.CLASS)
(base_list (identifier) @CS.CLASS)
((base_list (identifier) @CS.INTERFACE)
 (#match? @CS.INTERFACE "^I[A-Z]"))
(_ type: (identifier) @CS.CLASS)
(_ returns: (identifier) @CS.CLASS)
(type_argument_list (identifier) @CS.CLASS)
(as_expression right: (identifier) @CS.CLASS)
(is_expression right: (identifier) @CS.CLASS)
(object_creation_expression type: (identifier) @CS.CLASS)
(attribute name: (_) @CS.ATTRIBUTE)
((_ type: (identifier) @CS.INTERFACE)
 (#match? @CS.INTERFACE "^I[A-Z]"))
((type_argument_list (identifier) @CS.INTERFACE)
 (#match? @CS.INTERFACE "^I[A-Z]"))
(using_directive (identifier) @CS.NAMESPACE)
(using_directive (qualified_name (identifier) @CS.NAMESPACE))
(using_directive (qualified_name (qualified_name (identifier) @CS.NAMESPACE)))
(using_directive (qualified_name (qualified_name (qualified_name (identifier) @CS.NAMESPACE))))
(namespace_declaration name: (identifier) @CS.NAMESPACE)
(namespace_declaration name: (qualified_name (identifier) @CS.NAMESPACE))
(namespace_declaration name: (qualified_name (qualified_name (identifier) @CS.NAMESPACE)))
(namespace_declaration name: (qualified_name (qualified_name (qualified_name (identifier) @CS.NAMESPACE))))
(file_scoped_namespace_declaration name: (identifier) @CS.NAMESPACE)
(file_scoped_namespace_declaration name: (qualified_name (identifier) @CS.NAMESPACE))
(file_scoped_namespace_declaration name: (qualified_name (qualified_name (identifier) @CS.NAMESPACE)))
(file_scoped_namespace_declaration name: (qualified_name (qualified_name (qualified_name (identifier) @CS.NAMESPACE))))
(class_declaration name: (identifier) @CS.CLASS)
(record_declaration name: (identifier) @CS.CLASS)
(struct_declaration name: (identifier) @CS.STRUCT)
(interface_declaration name: (identifier) @CS.INTERFACE)
(enum_declaration name: (identifier) @CS.ENUM)
(delegate_declaration name: (identifier) @CS.CLASS)
(type_parameter name: (identifier) @CS.TYPE_PARAMETER)
(type_parameter_constraints_clause . (identifier) @CS.TYPE_PARAMETER)

(field_declaration (variable_declaration (variable_declarator name: (identifier) @CS.FIELD)))
(field_declaration (modifier "static") (variable_declaration (variable_declarator name: (identifier) @CS.STATIC_FIELD)))
(field_declaration (modifier "const") (variable_declaration (variable_declarator name: (identifier) @CS.CONSTANT)))
(property_declaration name: (identifier) @CS.PROPERTY)
(event_field_declaration (variable_declaration (variable_declarator name: (identifier) @CS.FIELD)))
(enum_member_declaration name: (identifier) @CS.CONSTANT)

(local_declaration_statement (variable_declaration (variable_declarator name: (identifier) @CS.LOCAL_VARIABLE)))
(using_statement (variable_declaration (variable_declarator name: (identifier) @CS.LOCAL_VARIABLE)))
(for_statement (variable_declaration (variable_declarator name: (identifier) @CS.LOCAL_VARIABLE)))
(foreach_statement left: (identifier) @CS.LOCAL_VARIABLE)
(declaration_expression name: (identifier) @CS.LOCAL_VARIABLE)
(declaration_pattern name: (identifier) @CS.LOCAL_VARIABLE)
(catch_declaration name: (identifier) @CS.LOCAL_VARIABLE)

(parameter name: (identifier) @CS.PARAMETER)
(implicit_parameter) @CS.PARAMETER
(lambda_expression parameters: (identifier) @CS.PARAMETER)
(argument name: (identifier) @CS.PARAMETER)

(method_declaration name: (identifier) @CS.METHOD_DECLARATION)
(local_function_statement name: (identifier) @CS.METHOD_DECLARATION)
(constructor_declaration name: (identifier) @CS.METHOD_DECLARATION)
(destructor_declaration name: (identifier) @CS.METHOD_DECLARATION)
(invocation_expression function: (identifier) @CS.METHOD_CALL)
(invocation_expression function: (generic_name (identifier) @CS.METHOD_CALL))
(invocation_expression function: (member_access_expression name: (identifier) @CS.METHOD_CALL))
(invocation_expression function: (member_access_expression name: (generic_name (identifier) @CS.METHOD_CALL)))
(invocation_expression function: (member_binding_expression name: (identifier) @CS.METHOD_CALL))
(invocation_expression function: (identifier) @CS.KEYWORD
 (#eq? @CS.KEYWORD "nameof"))

(labeled_statement (identifier) @CS.LABEL)
(goto_statement (identifier) @CS.LABEL)
