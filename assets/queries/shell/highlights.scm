; IVE highlight query for Shell (Bash, Zsh, sh). Capture names are colour key ids (src/colors/keys/shell.rs).
; For the same node a later pattern wins; otherwise the innermost node wins.

["(" ")" "((" "))"] @BASH.PAREN
["{" "}"] @BASH.BRACE
["[" "]" "[[" "]]"] @BASH.BRACKET
[">" ">>" "<" "&>" "&>>" ">&" "<&" ">&-" "<&-" ">|" "<<" "<<-" "<<<"] @BASH.REDIRECTION
["&&" "||" "!" "==" "!=" "=~"] @BASH.CONDITIONAL
(test_operator) @BASH.CONDITIONAL

[
  "case" "declare" "do" "done" "elif" "else" "esac" "export" "fi" "for" "function" "if" "in" "local"
  "readonly" "select" "then" "typeset" "unset" "unsetenv" "until" "while"
] @BASH.KEYWORD

[(number) (file_descriptor)] @BASH.NUMBER
((word) @BASH.NUMBER
 (#match? @BASH.NUMBER "^[0-9]+$"))
[(string) (translated_string)] @BASH.STRING
[(raw_string) (ansi_c_string)] @BASH.RAW_STRING
(comment) @BASH.LINE_COMMENT
(program . (comment) @BASH.SHEBANG
 (#match? @BASH.SHEBANG "^#!"))

(heredoc_body) @BASH.HERE_DOC
(heredoc_start) @BASH.HERE_DOC_START
(heredoc_end) @BASH.HERE_DOC_END

(command_name) @BASH.EXTERNAL_COMMAND
(command_substitution ["$(" ")"] @BASH.SUBSHELL_COMMAND)
(process_substitution ["<(" ">(" ")"] @BASH.SUBSHELL_COMMAND)
(command_substitution "`" @BASH.BACKQUOTE)
(function_definition name: (word) @BASH.FUNCTION_DEF_NAME)

(variable_name) @BASH.VAR_USE
[(simple_expansion) (expansion)] @BASH.VAR_USE
(expansion ["${" "}"] @BASH.VAR_USE)
(expansion (variable_name) @BASH.VAR_USE_COMPOSED)
(variable_assignment name: (variable_name) @BASH.VAR_DEF)
(variable_assignment name: (subscript name: (variable_name) @BASH.VAR_DEF))
(for_statement variable: (variable_name) @BASH.VAR_DEF)
(declaration_command (variable_name) @BASH.VAR_DEF)
