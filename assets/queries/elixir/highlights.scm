; IVE highlight query for Elixir. Capture names are colour key ids (src/colors/keys/).
; For the same node a later pattern wins; otherwise the innermost node wins.

["(" ")"] @ELIXIR.PARENTHESES
["[" "]" "<<" ">>"] @ELIXIR.BRACKETS
["{" "}"] @ELIXIR.BRACES
"," @ELIXIR.COMMA
"%" @ELIXIR.OPERATION_SIGN
(operator_identifier) @ELIXIR.OPERATION_SIGN
(unary_operator operator: _ @ELIXIR.OPERATION_SIGN)
(binary_operator operator: _ @ELIXIR.OPERATION_SIGN)
(stab_clause operator: _ @ELIXIR.OPERATION_SIGN)
(binary_operator operator: "|>" @ELIXIR.PIPE)
(dot operator: _ @ELIXIR.DOT)

[(integer) (float)] @ELIXIR.NUMBER
(char) @ELIXIR.CHAR
(comment) @ELIXIR.LINE_COMMENT

(string) @ELIXIR.STRING
(charlist) @ELIXIR.CHARLIST
(escape_sequence) @ELIXIR.ESCAPE
(interpolation ["#{" "}"] @ELIXIR.INTERPOLATION)

; The content of `~H` and `~r` is coloured by its embedded language (injections.scm), so it is left unpainted here.
(sigil "~" @ELIXIR.SIGIL)
(sigil (sigil_name) @ELIXIR.SIGIL)
(sigil (sigil_modifiers) @ELIXIR.SIGIL)
(sigil quoted_start: _ @ELIXIR.SIGIL)
(sigil quoted_end: _ @ELIXIR.SIGIL)
((sigil
  (sigil_name) @_name
  (quoted_content) @ELIXIR.SIGIL)
 (#not-any-of? @_name "H" "r" "R"))

[(atom) (quoted_atom) (keyword) (quoted_keyword)] @ELIXIR.ATOM
(alias) @ELIXIR.MODULE
(call target: (dot left: (atom) @ELIXIR.MODULE))

(identifier) @ELIXIR.VARIABLE
[(boolean) (nil)] @ELIXIR.PREDEFINED
((identifier) @ELIXIR.PREDEFINED
 (#any-of? @ELIXIR.PREDEFINED "__MODULE__" "__DIR__" "__ENV__" "__CALLER__" "__STACKTRACE__"))

(call target: (identifier) @ELIXIR.FUNCTION_CALL)
(call target: (dot right: (identifier) @ELIXIR.REMOTE_CALL))
(call
  target: (dot
    left: [(identifier) (call) (access_call) (unary_operator)]
    right: (identifier) @ELIXIR.FIELD)
  .)
(binary_operator operator: "|>" right: (identifier) @ELIXIR.FUNCTION_CALL)
(unary_operator
  operator: "&"
  operand: (binary_operator left: (identifier) @ELIXIR.FUNCTION_CALL operator: "/"))

(anonymous_function (stab_clause left: (arguments (identifier) @ELIXIR.PARAMETER)))

(unary_operator
  operator: "@" @ELIXIR.MODULE_ATTRIBUTE
  operand: [
    (identifier) @ELIXIR.MODULE_ATTRIBUTE
    (call target: (identifier) @ELIXIR.MODULE_ATTRIBUTE)
  ])
((unary_operator
  operator: "@"
  operand: (call
    target: (identifier) @_attribute
    (arguments [(string) (charlist) (sigil)] @ELIXIR.DOC_COMMENT)))
 (#any-of? @_attribute "moduledoc" "typedoc" "doc"))

[
  "when" "and" "or" "not" "in" "not in" "fn" "do" "end" "catch" "rescue" "after" "else"
] @ELIXIR.KEYWORD
((keyword) @ELIXIR.KEYWORD
 (#match? @ELIXIR.KEYWORD "^(do|else):"))
((call target: (identifier) @ELIXIR.KEYWORD)
 (#any-of? @ELIXIR.KEYWORD
  "def" "defp" "defmacro" "defmacrop" "defguard" "defguardp" "defdelegate" "defn" "defnp" "defmodule"
  "defprotocol" "defimpl" "defstruct" "defexception" "defoverridable" "alias" "import" "require" "use" "case"
  "cond" "for" "if" "unless" "with" "receive" "try" "raise" "reraise" "throw" "quote" "unquote"
  "unquote_splicing" "super"))

; The name and the parameters of a function head: `def name(params)`, `def name(params) when guard`.
((call
  target: (identifier) @_def
  (arguments
    [
      (identifier) @ELIXIR.FUNCTION_DECLARATION
      (call target: (identifier) @ELIXIR.FUNCTION_DECLARATION)
      (binary_operator left: (call target: (identifier) @ELIXIR.FUNCTION_DECLARATION) operator: "when")
    ]))
 (#any-of? @_def "def" "defp" "defmacro" "defmacrop" "defguard" "defguardp" "defdelegate" "defn" "defnp"))
((call
  target: (identifier) @_def
  (arguments
    [
      (call (arguments [
        (identifier) @ELIXIR.PARAMETER
        (binary_operator left: (identifier) @ELIXIR.PARAMETER operator: "\\\\")
        (tuple (identifier) @ELIXIR.PARAMETER)
        (list (identifier) @ELIXIR.PARAMETER)
        (map (map_content (keywords (pair value: (identifier) @ELIXIR.PARAMETER))))
        (binary_operator operator: "=" right: (identifier) @ELIXIR.PARAMETER)
      ]))
      (binary_operator
        left: (call (arguments [
          (identifier) @ELIXIR.PARAMETER
          (binary_operator left: (identifier) @ELIXIR.PARAMETER operator: "\\\\")
          (tuple (identifier) @ELIXIR.PARAMETER)
          (list (identifier) @ELIXIR.PARAMETER)
          (map (map_content (keywords (pair value: (identifier) @ELIXIR.PARAMETER))))
          (binary_operator operator: "=" right: (identifier) @ELIXIR.PARAMETER)
        ]))
        operator: "when")
    ]))
 (#any-of? @_def "def" "defp" "defmacro" "defmacrop" "defguard" "defguardp" "defn" "defnp"))
