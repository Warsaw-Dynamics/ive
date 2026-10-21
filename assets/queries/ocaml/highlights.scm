; IVE highlight query for OCaml, shared with OCaml Interface. Capture names are colour key ids (src/colors/keys/).
; For the same node a later pattern wins; otherwise the innermost node wins.

["(" ")"] @OCAML.PARENTHESES
["[" "]" "[|" "|]" "[<" "[>"] @OCAML.BRACKETS
["{" "}"] @OCAML.BRACES
"," @OCAML.COMMA
"." @OCAML.DOT
[";" ";;"] @OCAML.SEMICOLON
[
  (prefix_operator) (sign_operator) (pow_operator) (mult_operator) (add_operator) (concat_operator) (rel_operator)
  (and_operator) (or_operator) (assign_operator) (hash_operator) (indexing_operator) (match_operator)
  ":" "=" "|" "->" ":>" ":=" "::" "<-" "*" "#" ".." "~" "?" "!" "+=" "&"
] @OCAML.OPERATION_SIGN

[(number) (signed_number)] @OCAML.NUMBER
(string) @OCAML.STRING
(quoted_string) @OCAML.STRING
(character) @OCAML.CHAR
(escape_sequence) @OCAML.VALID_STRING_ESCAPE
(conversion_specification) @OCAML.FORMAT_SPECIFIER
(comment) @OCAML.BLOCK_COMMENT
((comment) @OCAML.DOC_COMMENT
 (#match? @OCAML.DOC_COMMENT "^\\(\\*\\*[^*)]"))

[
  "and" "as" "assert" "begin" "class" "constraint" "do" "done" "downto" "else" "end" "exception" "external" "for"
  "fun" "function" "functor" "if" "in" "include" "inherit" "initializer" "lazy" "let" "match" "method" "module"
  "mutable" "new" "nonrec" "object" "of" "open" "private" "rec" "sig" "struct" "then" "to" "try" "type" "val"
  "virtual" "when" "while" "with" "effect"
] @OCAML.KEYWORD
(boolean) @OCAML.KEYWORD
(value_definition [(let_operator) (let_and_operator)] @OCAML.KEYWORD)

[(module_name) (module_type_name)] @OCAML.MODULE

[(type_constructor) (class_name) (class_type_name)] @OCAML.TYPE
((type_constructor) @OCAML.PRIMITIVE_TYPE
 (#match? @OCAML.PRIMITIVE_TYPE "^(int|char|bytes|string|float|bool|unit|exn|array|list|option|nativeint|int32|int64|lazy_t|format|format4|format6)$"))
(type_variable) @OCAML.TYPE_VARIABLE
[(constructor_name) (tag)] @OCAML.CONSTRUCTOR

[(field_name) (instance_variable_name)] @OCAML.FIELD
(value_pattern) @OCAML.PARAMETER
(label_name) @OCAML.LABEL
(parameter ["~" "?"] @OCAML.LABEL . pattern: (value_pattern) @OCAML.LABEL)
(labeled_argument ["~" "?"] @OCAML.LABEL)
(labeled_argument_type "?" @OCAML.LABEL)

(application_expression function: (value_path (value_name) @OCAML.FUNCTION_CALL))
(infix_expression
  left: (value_path (value_name) @OCAML.FUNCTION_CALL)
  operator: (concat_operator) @_operator
  (#eq? @_operator "@@"))
(infix_expression
  operator: (rel_operator) @_operator
  right: (value_path (value_name) @OCAML.FUNCTION_CALL)
  (#eq? @_operator "|>"))
((value_name) @OCAML.BUILTIN
 (#match? @OCAML.BUILTIN "^(raise|raise_notrace|failwith|invalid_arg|ignore|not|fst|snd|ref)$"))

(let_binding pattern: (value_name) @OCAML.VALUE_DECLARATION)
(let_binding pattern: (value_name) @OCAML.FUNCTION_DECLARATION (parameter))
(let_binding pattern: (value_name) @OCAML.FUNCTION_DECLARATION body: [(fun_expression) (function_expression)])
(value_specification (value_name) @OCAML.FUNCTION_DECLARATION)
(external (value_name) @OCAML.FUNCTION_DECLARATION)

(attribute ["[@" "]"] @OCAML.ATTRIBUTE)
(item_attribute ["[@@" "]"] @OCAML.ATTRIBUTE)
(floating_attribute ["[@@@" "]"] @OCAML.ATTRIBUTE)
(attribute_id) @OCAML.ATTRIBUTE
(extension ["[%" "]"] @OCAML.ATTRIBUTE)
(item_extension ["[%%" "]"] @OCAML.ATTRIBUTE)
