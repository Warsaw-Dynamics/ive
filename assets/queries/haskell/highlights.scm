; IVE highlight query for Haskell. Capture names are colour key ids (src/colors/keys/).
; For the same node a later pattern wins; otherwise the innermost node wins.

["(" ")"] @HASKELL.PARENTHESES
["[" "]"] @HASKELL.BRACKETS
["{" "}"] @HASKELL.BRACES
"," @HASKELL.COMMA
[
  (operator) (constructor_operator) (all_names) "." ".." "=" "|" "::" "=>" "->" "<-" "\\" "`" "@" "~" "!"
] @HASKELL.OPERATION_SIGN
(infix_id ["`"] @HASKELL.OPERATION_SIGN)

[(integer) (float)] @HASKELL.NUMBER
(string) @HASKELL.STRING
(char) @HASKELL.CHAR
(comment) @HASKELL.LINE_COMMENT
((comment) @HASKELL.BLOCK_COMMENT
 (#match? @HASKELL.BLOCK_COMMENT "^\\{-"))
(haddock) @HASKELL.DOC_COMMENT
(pragma) @HASKELL.PRAGMA

[
  "module" "where" "import" "qualified" "as" "hiding" "let" "in" "if" "then" "else" "case" "cases" "of" "do"
  "mdo" "rec" "class" "instance" "data" "newtype" "type" "family" "pattern" "deriving" "via" "stock" "anyclass"
  "infix" "infixl" "infixr" "forall" "foreign"
] @HASKELL.KEYWORD
(wildcard) @HASKELL.KEYWORD

(module_id) @HASKELL.MODULE

(name) @HASKELL.TYPE
(type/variable) @HASKELL.TYPE_VARIABLE
(type_params (variable) @HASKELL.TYPE_VARIABLE)
(quantified_variables (variable) @HASKELL.TYPE_VARIABLE)
(class name: (name) @HASKELL.TYPE_CLASS)
(instance name: (name) @HASKELL.TYPE_CLASS)
(context context: (apply constructor: (name) @HASKELL.TYPE_CLASS))
(context context: (name) @HASKELL.TYPE_CLASS)
(deriving classes: (name) @HASKELL.TYPE_CLASS)
(deriving classes: (tuple (name) @HASKELL.TYPE_CLASS))

(constructor) @HASKELL.CONSTRUCTOR

(field_name (variable) @HASKELL.FIELD)

(pattern/variable) @HASKELL.PARAMETER

(apply function: (variable) @HASKELL.FUNCTION_CALL)
(apply function: (qualified id: (variable) @HASKELL.FUNCTION_CALL))
(infix_id (variable) @HASKELL.FUNCTION_CALL)

(function name: (variable) @HASKELL.FUNCTION_DECLARATION)
(bind name: (variable) @HASKELL.FUNCTION_DECLARATION)
(signature name: (variable) @HASKELL.TYPE_SIGNATURE)
(signature names: (binding_list (variable) @HASKELL.TYPE_SIGNATURE))
(local_binds (bind name: (variable) @HASKELL.LOCAL_VARIABLE))
(bind pattern: (pattern/variable) @HASKELL.LOCAL_VARIABLE)
