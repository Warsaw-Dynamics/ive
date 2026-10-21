; IVE highlight query for Starlark (Bazel BUILD, WORKSPACE, MODULE.bazel, .bzl). Capture names are colour key ids
; (src/colors/keys/starlark.rs). For the same node a later pattern wins; otherwise the innermost node wins.

["(" ")"] @STARLARK.PARENTHESES
["[" "]"] @STARLARK.BRACKETS
["{" "}"] @STARLARK.BRACES
"," @STARLARK.COMMA
"." @STARLARK.DOT
[
  "=" "+" "-" "*" "/" "//" "%" "**" "==" "!=" "<" ">" "<=" ">=" "+=" "-=" "*=" "/=" "//=" "%=" "|" "&" "^" "~"
  "<<" ">>" "|=" "&=" "^=" "<<=" ">>=" "->"
] @STARLARK.OPERATOR

[
  "and" "break" "continue" "def" "elif" "else" "for" "if" "in" "lambda" "not" "not in" "or" "pass" "return"
  "while"
] @STARLARK.KEYWORD
[(true) (false) (none)] @STARLARK.KEYWORD

[(integer) (float)] @STARLARK.NUMBER
(string) @STARLARK.STRING
((string) @STARLARK.LABEL
 (#match? @STARLARK.LABEL "^[\"'](@|//|:)"))
(escape_sequence) @STARLARK.ESCAPE
(comment) @STARLARK.LINE_COMMENT
(module . (expression_statement (string) @STARLARK.DOC_STRING))
(function_definition body: (block . (expression_statement (string) @STARLARK.DOC_STRING)))

((identifier) @STARLARK.CONSTANT
 (#match? @STARLARK.CONSTANT "^[A-Z][A-Z0-9_]+$"))
(attribute attribute: (identifier) @STARLARK.FIELD)

(call function: (identifier) @STARLARK.FUNCTION_CALL)
(expression_statement (call function: (identifier) @STARLARK.RULE_CALL))
(call function: (attribute attribute: (identifier) @STARLARK.METHOD_CALL))
((identifier) @STARLARK.BUILTIN
 (#any-of? @STARLARK.BUILTIN "native" "attr"))
((call function: (identifier) @STARLARK.BUILTIN)
 (#any-of? @STARLARK.BUILTIN
  "load" "glob" "select" "package" "licenses" "exports_files" "package_group" "workspace" "module" "bazel_dep"
  "use_extension" "use_repo" "register_toolchains" "register_execution_platforms" "rule" "aspect" "provider"
  "repository_rule" "module_extension" "tag_class" "struct" "depset" "fail" "print" "len" "range" "sorted"
  "reversed" "enumerate" "zip" "type" "str" "int" "bool" "list" "dict" "tuple" "hasattr" "getattr" "any" "all"
  "min" "max" "abs" "hash" "dir" "repr" "Label" "label_keyed_string_dict" "configuration_field" "subpackages"))

(function_definition name: (identifier) @STARLARK.FUNCTION_DECLARATION)
(parameters (identifier) @STARLARK.PARAMETER)
(default_parameter name: (identifier) @STARLARK.PARAMETER)
(typed_parameter (identifier) @STARLARK.PARAMETER)
(typed_default_parameter name: (identifier) @STARLARK.PARAMETER)
(list_splat_pattern (identifier) @STARLARK.PARAMETER)
(dictionary_splat_pattern (identifier) @STARLARK.PARAMETER)
(lambda_parameters (identifier) @STARLARK.PARAMETER)
(keyword_argument name: (identifier) @STARLARK.KEYWORD_ARGUMENT)
