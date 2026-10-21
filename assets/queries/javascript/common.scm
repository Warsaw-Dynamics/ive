; IVE highlight query shared by JavaScript and TypeScript.
; Capture names are colour key ids (src/colors/keys.rs).
; For the same node a later pattern wins; otherwise the innermost node wins.
; "/" is left out of the operators: regexes contain it as a token.

["(" ")"] @JS.PARENTHS
["[" "]"] @JS.BRACKETS
["{" "}"] @JS.BRACES
"," @JS.COMMA
"." @JS.DOT
";" @JS.SEMICOLON
["=" "+" "-" "*" "%" "==" "===" "!=" "!==" "<=" ">=" "&&" "||" "??" "!" "=>" "+=" "-=" "++" "--" "?" ":" "..."] @JS.OPERATION_SIGN

(number) @JS.NUMBER
[(string) (template_string)] @JS.STRING
(escape_sequence) @JS.VALID_ESCAPE
(regex) @JS.REGEXP
(comment) @JS.LINE_COMMENT
((comment) @JS.BLOCK_COMMENT
 (#match? @JS.BLOCK_COMMENT "^/\\*"))
((comment) @JS.DOC_COMMENT
 (#match? @JS.DOC_COMMENT "^/\\*\\*"))

[
  "as" "async" "await" "break" "case" "catch" "class" "const" "continue" "debugger" "default" "delete"
  "do" "else" "export" "extends" "finally" "for" "from" "function" "get" "if" "import" "in" "instanceof"
  "let" "new" "of" "return" "set" "static" "switch" "target" "throw" "try" "typeof" "var" "void"
  "while" "with" "yield"
] @JS.KEYWORD
[(true) (false) (null) (undefined) (this) (super)] @JS.KEYWORD

(property_identifier) @JS.INSTANCE_MEMBER_VARIABLE
(variable_declarator name: (identifier) @JS.LOCAL_VARIABLE)
((identifier) @JS.CLASS
 (#match? @JS.CLASS "^[A-Z]"))
((identifier) @JS.CONSTANT
 (#match? @JS.CONSTANT "^[A-Z_][A-Z0-9_]+$"))

(function_declaration name: (identifier) @JS.GLOBAL_FUNCTION)
(call_expression function: (identifier) @JS.FUNCTION_CALL)
(method_definition name: (property_identifier) @JS.INSTANCE_MEMBER_FUNCTION)
(call_expression
  function: (member_expression property: (property_identifier) @JS.INSTANCE_MEMBER_FUNCTION))
(arrow_function parameter: (identifier) @JS.PARAMETER)

(decorator) @JS.DECORATOR
(decorator (identifier) @JS.DECORATOR)
