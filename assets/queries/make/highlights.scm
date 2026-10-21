; IVE highlight query for Makefiles. Capture names are colour key ids (src/colors/keys/make.rs).
; For the same node a later pattern wins; otherwise the innermost node wins.
; Recipe lines are coloured as Shell by injections.scm.

["(" ")"] @MAKE.PARENTHESES
"," @MAKE.COMMA
["=" ":=" "::=" "?=" "+=" "!=" ":" "&:" "::" "|" ";"] @MAKE.OPERATOR
(recipe_line ["@" "-" "+"] @MAKE.OPERATOR)

(comment) @MAKE.LINE_COMMENT
(string) @MAKE.STRING
(variable_assignment value: (text) @MAKE.VALUE)
(define_directive value: (raw_text) @MAKE.VALUE)

[
  "define" "endef" "export" "unexport" "override" "private" "undefine" "vpath" "include" "sinclude" "-include"
  "ifeq" "ifneq" "ifdef" "ifndef" "else" "endif"
] @MAKE.KEYWORD

(targets (word) @MAKE.TARGET)
((targets (word) @MAKE.SPECIAL_TARGET)
 (#match? @MAKE.SPECIAL_TARGET "^\\.[A-Z_]+$"))
(prerequisites (word) @MAKE.PREREQUISITE)

(function_call ["$" function: _] @MAKE.FUNCTION)
(shell_function ["$" function: _] @MAKE.FUNCTION)

[(variable_reference) (substitution_reference)] @MAKE.VARIABLE
(variable_reference ["$" "(" ")" "{" "}"] @MAKE.VARIABLE)
(substitution_reference ["$" "(" ")" "{" "}" ":" "="] @MAKE.VARIABLE)
(substitution_reference text: (word) @MAKE.VARIABLE)
(automatic_variable) @MAKE.AUTOMATIC_VARIABLE
(automatic_variable ["$" "(" ")" "{" "}"] @MAKE.AUTOMATIC_VARIABLE)
[
  (ifdef_directive variable: (_) @MAKE.VARIABLE)
  (ifndef_directive variable: (_) @MAKE.VARIABLE)
]

(variable_assignment name: (word) @MAKE.VARIABLE_DECLARATION)
(shell_assignment name: (word) @MAKE.VARIABLE_DECLARATION)
(define_directive name: (word) @MAKE.VARIABLE_DECLARATION)
(undefine_directive variable: (word) @MAKE.VARIABLE_DECLARATION)
(export_directive variables: (list (word) @MAKE.VARIABLE_DECLARATION))
