; IVE highlight query for Python. Capture names are colour key ids (src/colors/keys/python.rs).
; For the same node a later pattern wins; otherwise the innermost node wins.

["(" ")"] @PY.PARENTHS
["[" "]"] @PY.BRACKETS
["{" "}"] @PY.BRACES
"," @PY.COMMA
"." @PY.DOT
[
  "-" "-=" "!=" "*" "**" "**=" "*=" "/" "//" "//=" "/=" "&" "&=" "%" "%=" "^" "^=" "+" "->" "+=" "<" "<<"
  "<<=" "<=" "<>" "=" ":=" "==" ">" ">=" ">>" ">>=" "|" "|=" "~" "@" "@="
] @PY.OPERATION_SIGN

[(integer) (float)] @PY.NUMBER
(comment) @PY.LINE_COMMENT
(string) @PY.STRING.U
((string (string_start) @_prefix) @PY.STRING.B
 (#match? @_prefix "^([rR]?[bB]|[bB][rR])"))
(escape_sequence) @PY.VALID_STRING_ESCAPE
(interpolation) @TEXT
(interpolation ["{" "}"] @PY.FSTRING_FRAGMENT_BRACES)
(type_conversion) @PY.FSTRING_FRAGMENT_TYPE_CONVERSION
(format_specifier ":" @PY.FSTRING_FRAGMENT_COLON)
(format_specifier) @PY.STRING.U

(module . (expression_statement (string) @PY.DOC_COMMENT))
(class_definition body: (block . (expression_statement (string) @PY.DOC_COMMENT)))
(function_definition body: (block . (expression_statement (string) @PY.DOC_COMMENT)))

[
  "as" "assert" "async" "await" "break" "case" "class" "continue" "def" "del" "elif" "else" "except" "exec"
  "finally" "for" "from" "global" "if" "import" "lambda" "match" "nonlocal" "pass" "print" "raise" "return"
  "try" "type" "while" "with" "yield" "and" "in" "is" "not" "or" "is not" "not in"
] @PY.KEYWORD
[(none) (true) (false)] @PY.KEYWORD

(type) @PY.ANNOTATION
(keyword_argument name: (identifier) @PY.KEYWORD_ARGUMENT)

(parameters (identifier) @PY.PARAMETER)
(lambda_parameters (identifier) @PY.PARAMETER)
(typed_parameter . (identifier) @PY.PARAMETER)
(default_parameter name: (identifier) @PY.PARAMETER)
(typed_default_parameter name: (identifier) @PY.PARAMETER)
(list_splat_pattern (identifier) @PY.PARAMETER)
(dictionary_splat_pattern (identifier) @PY.PARAMETER)
(type_parameter (type (identifier) @PY.TYPE_PARAMETER))

(function_definition
  body: (block (expression_statement (assignment left: (identifier) @PY.LOCAL_VARIABLE))))
(function_definition
  body: (block (expression_statement (assignment left: (pattern_list (identifier) @PY.LOCAL_VARIABLE)))))
(function_definition
  body: (block (for_statement left: (identifier) @PY.LOCAL_VARIABLE)))

(call function: (identifier) @PY.FUNCTION_CALL)
(call function: (attribute attribute: (identifier) @PY.METHOD_CALL))

((identifier) @PY.BUILTIN_NAME
 (#any-of? @PY.BUILTIN_NAME
   "abs" "aiter" "all" "anext" "any" "ascii" "bin" "bool" "breakpoint" "bytearray" "bytes" "callable" "chr"
   "classmethod" "compile" "complex" "delattr" "dict" "dir" "divmod" "enumerate" "eval" "filter" "float"
   "format" "frozenset" "getattr" "globals" "hasattr" "hash" "help" "hex" "id" "input" "int" "isinstance"
   "issubclass" "iter" "len" "list" "locals" "map" "max" "memoryview" "min" "next" "object" "oct" "open" "ord"
   "pow" "print" "property" "range" "repr" "reversed" "round" "set" "setattr" "slice" "sorted" "staticmethod"
   "str" "sum" "super" "tuple" "type" "vars" "zip" "__import__" "NotImplemented" "Ellipsis" "__debug__"))
((identifier) @PY.PREDEFINED_USAGE
 (#match? @PY.PREDEFINED_USAGE "^__[a-zA-Z0-9_]+__$"))
((identifier) @PY.SELF_PARAMETER
 (#any-of? @PY.SELF_PARAMETER "self" "cls"))

(class_definition name: (identifier) @PY.CLASS_DEFINITION)
(function_definition name: (identifier) @PY.FUNC_DEFINITION)
(function_definition
  body: (block [
    (function_definition name: (identifier) @PY.NESTED_FUNC_DEFINITION)
    (decorated_definition definition: (function_definition name: (identifier) @PY.NESTED_FUNC_DEFINITION))
  ]))
((function_definition name: (identifier) @PY.PREDEFINED_DEFINITION)
 (#match? @PY.PREDEFINED_DEFINITION "^__[a-zA-Z0-9_]+__$"))

(decorator "@" @PY.DECORATOR)
(decorator [(identifier) (attribute)] @PY.DECORATOR)
(decorator (attribute ["." (identifier)] @PY.DECORATOR))
(decorator (call function: [(identifier) (attribute)] @PY.DECORATOR))
(decorator (call function: (attribute ["." (identifier)] @PY.DECORATOR)))
