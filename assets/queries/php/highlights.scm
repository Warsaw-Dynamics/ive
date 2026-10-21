; IVE highlight query for PHP. Capture names are colour key ids (src/colors/keys/php.rs).
; For the same node a later pattern wins; otherwise the innermost node wins. The text around the PHP tags is
; injected as HTML (php/injections.scm).

[(php_tag) (php_end_tag)] @PHP.TAG

["(" ")"] @PHP.PARENTHESES
["[" "]"] @PHP.BRACKETS
["{" "}"] @PHP.BRACES
"," @PHP.COMMA
";" @PHP.SEMICOLON
[
  "!" "!=" "!==" "%" "%=" "&" "&&" "&=" "*" "**" "**=" "*=" "+" "++" "+=" "-" "--" "-=" "->" "?->" "." ".="
  "..." "/" "/=" ":" "::" "<" "<<" "<<=" "<=" "<=>" "<>" "=" "==" "===" "=>" ">" ">=" ">>" ">>=" "?" "??" "??="
  "@" "^" "^=" "|" "|=" "|>" "||" "~" "\\"
] @PHP.OPERATION_SIGN

[(integer) (float)] @PHP.NUMBER
(comment) @PHP.LINE_COMMENT
((comment) @PHP.BLOCK_COMMENT
 (#match? @PHP.BLOCK_COMMENT "^/\\*"))
((comment) @PHP.DOC_COMMENT
 (#match? @PHP.DOC_COMMENT "^/\\*\\*"))
[(string) (encapsed_string) (shell_command_expression)] @PHP.STRING
[(heredoc) (nowdoc)] @PHP.HEREDOC
(escape_sequence) @PHP.ESCAPE
([(encapsed_string) (heredoc_body)] ["{" "}"] @PHP.ESCAPE)

[
  "abstract" "and" "array" "as" "break" "case" "catch" "class" "clone" "const" "continue" "declare" "default"
  "do" "echo" "else" "elseif" "enddeclare" "endfor" "endforeach" "endif" "endswitch" "endwhile" "enum" "exit"
  "extends" "final" "finally" "fn" "for" "foreach" "function" "global" "goto" "if" "implements" "include"
  "include_once" "instanceof" "insteadof" "interface" "list" "match" "namespace" "new" "or" "print"
  "readonly" "require" "require_once" "return" "static" "switch" "throw" "trait" "try" "unset" "use"
  "while" "xor" "yield" "yield from"
  (abstract_modifier) (final_modifier) (readonly_modifier) (static_modifier) (visibility_modifier)
  (var_modifier) (primitive_type) (cast_type) (boolean) (null) (relative_scope)
] @PHP.KEYWORD
((name) @PHP.KEYWORD
 (#any-of? @PHP.KEYWORD "true" "false" "null" "TRUE" "FALSE" "NULL"))

(namespace_name (name) @PHP.NAMESPACE)
(named_type (name) @PHP.CLASS)
(named_type (qualified_name (name) @PHP.CLASS))
(base_clause [(name) (qualified_name (name))] @PHP.CLASS)
(class_interface_clause [(name) (qualified_name (name))] @PHP.INTERFACE)
(object_creation_expression [(name) (qualified_name (name))] @PHP.CLASS)
(scoped_call_expression scope: [(name) (qualified_name (name))] @PHP.CLASS)
(class_constant_access_expression . [(name) (qualified_name (name))] @PHP.CLASS)
(scoped_property_access_expression scope: [(name) (qualified_name (name))] @PHP.CLASS)
(namespace_use_clause [(name) (qualified_name (name))] @PHP.CLASS)
(class_declaration name: (name) @PHP.CLASS)
(enum_declaration name: (name) @PHP.CLASS)
(interface_declaration name: (name) @PHP.INTERFACE)
(trait_declaration name: (name) @PHP.INTERFACE)
((named_type (name) @PHP.KEYWORD)
 (#any-of? @PHP.KEYWORD "static" "self" "parent"))

(variable_name) @PHP.VARIABLE
(simple_parameter name: (variable_name) @PHP.PARAMETER)
(variadic_parameter name: (variable_name) @PHP.PARAMETER)
(property_promotion_parameter name: (variable_name) @PHP.PARAMETER)
(property_element (variable_name) @PHP.PROPERTY)
(member_access_expression name: (name) @PHP.PROPERTY)
(nullsafe_member_access_expression name: (name) @PHP.PROPERTY)
(scoped_property_access_expression name: (variable_name) @PHP.PROPERTY)
((variable_name) @PHP.PREDEFINED
 (#match? @PHP.PREDEFINED "^\\$(this|GLOBALS|_SERVER|_GET|_POST|_FILES|_COOKIE|_SESSION|_REQUEST|_ENV)$"))

((name) @PHP.CONSTANT
 (#match? @PHP.CONSTANT "^_?[A-Z][A-Z0-9_]+$"))
(const_element (name) @PHP.CONSTANT)
(class_constant_access_expression (_) (name) @PHP.CONSTANT)
((name) @PHP.PREDEFINED
 (#match? @PHP.PREDEFINED "^__[A-Z]+__$"))

(function_call_expression function: [(name) (qualified_name (name))] @PHP.FUNCTION_CALL)
(member_call_expression name: (name) @PHP.METHOD_CALL)
(nullsafe_member_call_expression name: (name) @PHP.METHOD_CALL)
(scoped_call_expression name: (name) @PHP.STATIC_METHOD_CALL)
(function_definition name: (name) @PHP.FUNCTION_DECLARATION)
(method_declaration name: (name) @PHP.METHOD_DECLARATION)

(attribute_group "#[" @PHP.ATTRIBUTE)
(attribute_group "]" @PHP.ATTRIBUTE)
(attribute [(name) (qualified_name)] @PHP.ATTRIBUTE)
