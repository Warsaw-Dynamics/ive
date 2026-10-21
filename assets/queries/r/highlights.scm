; IVE highlight query for R. Capture names are colour key ids (src/colors/keys/r.rs).
; For the same node a later pattern wins; otherwise the innermost node wins.

["(" ")"] @R.PARENTHESES
["[" "]" "[[" "]]"] @R.BRACKETS
["{" "}"] @R.BRACES
(comma) @R.COMMA
[
  "?" "~" "|>" "||" "|" "&&" "&" "<" "<=" ">" ">=" "==" "!=" "+" "-" "*" "/" "::" ":::" "**" "^" "$" "@" ":"
  "!" "special" "\\" "="
] @R.OPERATION_SIGN
(binary_operator operator: ["<-" "<<-" "->" "->>" "=" ":="] @R.ASSIGNMENT)

[(integer) (float) (complex)] @R.NUMBER
(string) @R.STRING
(escape_sequence) @R.ESCAPE
(comment) @R.LINE_COMMENT
((comment) @R.DOC_COMMENT
 (#match? @R.DOC_COMMENT "^#'"))

["if" "else" "for" "in" "while" "repeat" "function"] @R.KEYWORD
[(next) (break)] @R.KEYWORD
[(true) (false) (null) (inf) (nan) (na) (dots) (dot_dot_i)] @R.CONSTANT

(binary_operator lhs: (identifier) @R.LOCAL_VARIABLE operator: ["<-" "<<-" "=" ":="])
(binary_operator operator: ["->" "->>"] rhs: (identifier) @R.LOCAL_VARIABLE)
(parameter name: (identifier) @R.PARAMETER)
(argument name: (identifier) @R.NAMED_ARGUMENT)
(extract_operator rhs: (identifier) @R.FIELD)
(namespace_operator lhs: (identifier) @R.NAMESPACE)

(call function: (identifier) @R.FUNCTION_CALL)
(call function: (namespace_operator rhs: (identifier) @R.FUNCTION_CALL))
((call function: (identifier) @R.PREDEFINED)
 (#any-of? @R.PREDEFINED
   "c" "list" "vector" "matrix" "data.frame" "factor" "length" "names" "nrow" "ncol" "seq" "seq_along"
   "seq_len" "rep" "sum" "mean" "max" "min" "round" "paste" "paste0" "sprintf" "cat" "print" "format" "stop"
   "warning" "message" "library" "require" "source" "is.null" "is.na" "lapply" "sapply" "vapply" "apply"
   "Map" "Reduce" "Filter" "do.call" "invisible" "missing" "match.arg" "stopifnot" "tryCatch" "on.exit"
   "nchar" "substr" "toupper" "tolower" "unlist" "setNames" "structure" "class" "inherits" "identical"))
((call function: (identifier) @R.KEYWORD)
 (#eq? @R.KEYWORD "return"))
(binary_operator
  lhs: (identifier) @R.FUNCTION_DECLARATION
  operator: ["<-" "<<-" "="]
  rhs: (function_definition))
