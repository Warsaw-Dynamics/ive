; IVE highlight query for PowerShell. Capture names are colour key ids (src/colors/keys/powershell.rs).
; For the same node a later pattern wins; otherwise the innermost node wins.

["(" ")" "@(" "$("] @PS.PARENTHESES
["[" "]"] @PS.BRACKETS
["{" "}" "@{"] @PS.BRACES
"," @PS.COMMA
"." @PS.DOT
";" @PS.SEMICOLON
[
  (comparison_operator) (assignement_operator) (format_operator) (command_invokation_operator)
  "!" "%" "*" "+" "++" "-" "--" "/" ".." "::" "|" "||" "&&" "="
  "-and" "-or" "-xor" "-not" "-band" "-bor" "-bxor" "-bnot" "-as" "-join" "-split" "-replace" "-is" "-isnot"
  "-in" "-notin" "-contains" "-notcontains" "-like" "-notlike" "-match" "-notmatch" "-shl" "-shr" "-f"
] @PS.OPERATION_SIGN

[(integer_literal) (real_literal)] @PS.NUMBER
(string_literal) @PS.STRING
(expandable_string_literal (sub_expression) @TEXT)
(expandable_string_literal (sub_expression ["$(" ")"] @PS.INTERPOLATION))
(expandable_here_string_literal (sub_expression) @TEXT)
(expandable_here_string_literal (sub_expression ["$(" ")"] @PS.INTERPOLATION))
(comment) @PS.LINE_COMMENT
((comment) @PS.BLOCK_COMMENT
 (#match? @PS.BLOCK_COMMENT "^<#"))
((comment) @PS.DOC_COMMENT
 (#match? @PS.DOC_COMMENT "(?is)^<#\\s*\\.(synopsis|description|parameter|example|notes|outputs|inputs|link)"))

[
  "begin" "break" "catch" "continue" "data" "do" "dynamicparam" "else" "elseif" "end" "exit" "filter"
  "finally" "for" "foreach" "function" "if" "in" "inlinescript" "parallel" "param" "process" "return"
  "sequence" "switch" "throw" "trap" "try" "until" "while" "workflow"
] @PS.KEYWORD
(class_attribute) @PS.KEYWORD

(variable) @PS.VARIABLE
(script_parameter (variable) @PS.PARAMETER)
(class_method_parameter (variable) @PS.PARAMETER)
(class_property_definition (variable) @PS.PROPERTY)
((variable) @PS.PREDEFINED
 (#match? @PS.PREDEFINED
   "(?i)^\\$(this|_|true|false|null|args|input|psitem|psscriptroot|pscmdlet|host|error|lastexitcode|matches|pshome|home|pwd|profile|myinvocation)$"))
((variable) @PS.ENVIRONMENT_VARIABLE
 (#match? @PS.ENVIRONMENT_VARIABLE "(?i)^\\$\\{?env:"))

(type_literal) @PS.TYPE
(type_spec) @PS.TYPE
(class_statement (simple_name) @PS.TYPE)
(enum_statement (simple_name) @PS.TYPE)
(attribute (attribute_name) @PS.ATTRIBUTE)
(attribute_argument (simple_name) @PS.PARAMETER)

(member_access (member_name) @PS.PROPERTY)
(key_expression (simple_name) @PS.PROPERTY)
(invokation_expression (member_name) @PS.METHOD_CALL)
(command command_name: (command_name) @PS.COMMAND)
(command_parameter) @PS.COMMAND_PARAMETER
(function_statement (function_name) @PS.FUNCTION_DECLARATION)
(class_method_definition (simple_name) @PS.FUNCTION_DECLARATION)
(enum_member (simple_name) @PS.CONSTANT)
(label) @PS.LABEL
