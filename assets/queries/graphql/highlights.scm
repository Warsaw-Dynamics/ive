; IVE highlight query for GraphQL. Capture names are colour key ids (src/colors/keys/).
; For the same node a later pattern wins; otherwise the innermost node wins.

["(" ")"] @GRAPHQL.PARENTHESES
["[" "]"] @GRAPHQL.BRACKETS
["{" "}"] @GRAPHQL.BRACES
(comma) @GRAPHQL.COMMA
["!" "=" ":" "|" "&" "..."] @GRAPHQL.OPERATION_SIGN

[(int_value) (float_value)] @GRAPHQL.NUMBER
(string_value) @GRAPHQL.STRING
(description (string_value) @GRAPHQL.DESCRIPTION)
(comment) @GRAPHQL.COMMENT

[
  "query" "mutation" "subscription" "fragment" "on" "type" "interface" "union" "enum" "input" "scalar" "schema"
  "extend" "implements" "directive" "repeatable" "true" "false"
] @GRAPHQL.KEYWORD
(null_value) @GRAPHQL.KEYWORD
[(executable_directive_location) (type_system_directive_location)] @GRAPHQL.KEYWORD

(named_type (name) @GRAPHQL.TYPE)
((named_type (name) @GRAPHQL.BUILTIN_TYPE)
 (#any-of? @GRAPHQL.BUILTIN_TYPE "Int" "Float" "String" "Boolean" "ID"))
(scalar_type_definition (name) @GRAPHQL.TYPE)
(object_type_definition (name) @GRAPHQL.TYPE)
(union_type_definition (name) @GRAPHQL.TYPE)
(enum_type_definition (name) @GRAPHQL.TYPE)
(input_object_type_definition (name) @GRAPHQL.TYPE)
(interface_type_definition (name) @GRAPHQL.INTERFACE)
(scalar_type_extension (name) @GRAPHQL.TYPE)
(object_type_extension (name) @GRAPHQL.TYPE)
(interface_type_extension (name) @GRAPHQL.INTERFACE)
(union_type_extension (name) @GRAPHQL.TYPE)
(enum_type_extension (name) @GRAPHQL.TYPE)
(input_object_type_extension (name) @GRAPHQL.TYPE)

(operation_definition (name) @GRAPHQL.OPERATION_NAME)
(fragment_definition (fragment_name (name) @GRAPHQL.FRAGMENT_DEFINITION))
(fragment_spread (fragment_name (name) @GRAPHQL.FRAGMENT_SPREAD))

(field (name) @GRAPHQL.FIELD)
(field_definition (name) @GRAPHQL.FIELD)
(input_fields_definition (input_value_definition (name) @GRAPHQL.FIELD))
(object_field (name) @GRAPHQL.FIELD)
(alias (name) @GRAPHQL.ALIAS)
(argument (name) @GRAPHQL.ARGUMENT)
(arguments_definition (input_value_definition (name) @GRAPHQL.ARGUMENT))
(variable) @GRAPHQL.VARIABLE
(enum_value) @GRAPHQL.ENUM_VALUE

(directive "@" @GRAPHQL.DIRECTIVE (name) @GRAPHQL.DIRECTIVE)
(directive_definition "@" @GRAPHQL.DIRECTIVE (name) @GRAPHQL.DIRECTIVE)
