; IVE highlight query for fish. Capture names are colour key ids (src/colors/keys/fish.rs).
; For the same node a later pattern wins; otherwise the innermost node wins.

["(" ")"] @FISH.PARENTHESES
["[" "]"] @FISH.BRACKETS
["{" "}"] @FISH.BRACES
"," @FISH.COMMA
";" @FISH.SEMICOLON
["&&" "||" "|" "&|" "&" ".." "!" (glob) (home_dir_expansion)] @FISH.OPERATOR
["2>|" (direction) (stream_redirect)] @FISH.REDIRECTION

[(double_quote_string) (single_quote_string)] @FISH.STRING
(escape_sequence) @FISH.ESCAPE
[(integer) (float)] @FISH.NUMBER
(comment) @FISH.LINE_COMMENT

[
  "and" "begin" "case" "else" "end" "for" "function" "if" "in" "not" "or" "return" "switch" "while"
  (break) (continue)
] @FISH.KEYWORD

((word) @FISH.OPTION
 (#match? @FISH.OPTION "^--?[A-Za-z]"))

(command name: (word) @FISH.COMMAND)
((command name: (word) @FISH.BUILTIN)
 (#any-of? @FISH.BUILTIN
  "abbr" "argparse" "bg" "bind" "block" "builtin" "cd" "command" "commandline" "complete" "contains" "count"
  "disown" "echo" "emit" "eval" "exec" "exit" "false" "fg" "functions" "history" "jobs" "math" "printf" "pwd"
  "random" "read" "realpath" "set" "set_color" "source" "status" "string" "test" "time" "true" "type" "ulimit"
  "wait"))
(function_definition name: [(word) (concatenation)] @FISH.FUNCTION_DECLARATION)

(variable_expansion) @FISH.VARIABLE
(variable_expansion (variable_name) @FISH.VARIABLE)
(for_statement variable: (variable_name) @FISH.VARIABLE_DECLARATION)
((command
   name: (word) @_set
   .
   argument: (word) @FISH.VARIABLE_DECLARATION)
 (#any-of? @_set "set" "read")
 (#not-match? @FISH.VARIABLE_DECLARATION "^-"))
((command
   name: (word) @_set
   .
   argument: (word) @_option
   .
   argument: (word) @FISH.VARIABLE_DECLARATION)
 (#any-of? @_set "set" "read")
 (#match? @_option "^-")
 (#not-match? @FISH.VARIABLE_DECLARATION "^-"))
