; IVE highlight query for Erlang. Capture names are colour key ids (src/colors/keys/).
; For the same node a later pattern wins; otherwise the innermost node wins.

["(" ")"] @ERLANG.PARENTHESES
["[" "]" "<<" ">>"] @ERLANG.BRACKETS
["{" "}"] @ERLANG.BRACES
"," @ERLANG.COMMA
"." @ERLANG.DOT
";" @ERLANG.SEMICOLON
[
  "!" "->" "<-" "<=" "||" "#" "::" ":" "=" "|" "+" "-" "/" "*" "++" "--" "==" "/=" "=<" "<" ">=" ">" "=:=" "=/="
  "=>" ":="
] @ERLANG.OPERATION_SIGN

[(integer) (float)] @ERLANG.NUMBER
(char) @ERLANG.CHAR
(string) @ERLANG.STRING
(comment) @ERLANG.LINE_COMMENT

(atom) @ERLANG.ATOM
(var) @ERLANG.VARIABLE

[
  "after" "and" "andalso" "band" "begin" "bnot" "bor" "bsl" "bsr" "bxor" "case" "catch" "div" "end" "fun" "if" "of"
  "or" "orelse" "receive" "rem" "try" "when" "xor" "not" "maybe" "else"
] @ERLANG.KEYWORD

; `-module(...)`, `-export(...)`, `-define(...)` and the other attributes and preprocessor directives.
(_
  .
  "-" @ERLANG.ATTRIBUTE
  .
  [
    "module" "export" "export_type" "import" "behaviour" "behavior" "record" "spec" "callback" "type" "opaque"
    "nominal" "compile" "file" "optional_callbacks" "deprecated" "feature" "define" "undef" "ifdef" "ifndef" "else"
    "endif" "if" "elif" "include" "include_lib"
  ] @ERLANG.ATTRIBUTE)
(attr_name "-" @ERLANG.ATTRIBUTE name: (_) @ERLANG.ATTRIBUTE)

((wild_attribute
  name: (attr_name name: (atom) @_name)
  value: [(string) (multi_string)] @ERLANG.DOC_COMMENT)
 (#any-of? @_name "doc" "moduledoc"))

(macro_call_expr "?" @ERLANG.MACRO name: (_) @ERLANG.MACRO)
(macro_lhs name: (_) @ERLANG.MACRO)
(macro_string) @ERLANG.MACRO

(module_attribute name: (atom) @ERLANG.MODULE)
(behaviour_attribute name: (atom) @ERLANG.MODULE)
(import_attribute module: (atom) @ERLANG.MODULE)
(remote_module module: (atom) @ERLANG.MODULE)
(module name: (atom) @ERLANG.MODULE)

(record_decl name: (atom) @ERLANG.RECORD)
(record_name "#" @ERLANG.RECORD name: (_) @ERLANG.RECORD)
(qualified_record_name name: (_) @ERLANG.RECORD)
(record_field name: (atom) @ERLANG.RECORD_FIELD)
(record_field_name name: (atom) @ERLANG.RECORD_FIELD)

(call expr: (atom) @ERLANG.FUNCTION_CALL)
(remote fun: (call expr: (atom) @ERLANG.REMOTE_CALL))
(fa fun: (atom) @ERLANG.FUNCTION_CALL)
(internal_fun fun: (atom) @ERLANG.FUNCTION_CALL)
(external_fun fun: (atom) @ERLANG.REMOTE_CALL)

(type_name name: (atom) @ERLANG.TYPE)
(type_alias ty: (call expr: (atom) @ERLANG.TYPE))
(type_alias ty: (_ (call expr: (atom) @ERLANG.TYPE)))
(type_alias ty: (_ (_ (call expr: (atom) @ERLANG.TYPE))))
(type_sig args: (expr_args args: (call expr: (atom) @ERLANG.TYPE)))
(type_sig ty: (call expr: (atom) @ERLANG.TYPE))
(field_type expr: (call expr: (atom) @ERLANG.TYPE))
(export_type_attribute types: (fa fun: (atom) @ERLANG.TYPE))

(function_clause name: (atom) @ERLANG.FUNCTION_DECLARATION)
(spec fun: (atom) @ERLANG.FUNCTION_DECLARATION)
(callback fun: (atom) @ERLANG.FUNCTION_DECLARATION)

(function_clause
  args: (expr_args
    args: [
      (var) @ERLANG.PARAMETER
      (tuple expr: (var) @ERLANG.PARAMETER)
      (list exprs: (var) @ERLANG.PARAMETER)
      (match_expr [(var) @ERLANG.PARAMETER (_ (var) @ERLANG.PARAMETER)])
      (map_expr fields: (map_field value: (var) @ERLANG.PARAMETER))
      (record_expr fields: (record_field expr: (field_expr expr: (var) @ERLANG.PARAMETER)))
    ]))
(fun_clause
  args: (expr_args args: (var) @ERLANG.PARAMETER))
