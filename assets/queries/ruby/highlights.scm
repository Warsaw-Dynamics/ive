; IVE highlight query for Ruby. Capture names are colour key ids (src/colors/keys/ruby.rs).
; For the same node a later pattern wins; otherwise the innermost node wins.

["(" ")"] @RUBY.PARENTHESES
["[" "]"] @RUBY.BRACKETS
["{" "}"] @RUBY.BRACES
"," @RUBY.COMMA
["." "&." "::"] @RUBY.DOT
";" @RUBY.SEMICOLON
[
  "!" "!=" "!~" "%" "%=" "&" "&&" "&&=" "&=" "*" "**" "**=" "*=" "+" "+=" "-" "-=" "->" "/" "/=" ".." "..."
  "<" "<<" "<<=" "<=" "<=>" "=" "==" "===" "=>" "=~" ">" ">=" ">>" ">>=" "?" ":" "^" "^=" "|" "|=" "||" "||="
  "~"
] @RUBY.OPERATION_SIGN

[(integer) (float) (rational) (complex)] @RUBY.NUMBER
(comment) @RUBY.LINE_COMMENT
((comment) @RUBY.BLOCK_COMMENT
 (#match? @RUBY.BLOCK_COMMENT "^=begin"))

[(string) (bare_string) (subshell) (character) (string_array) (symbol_array)] @RUBY.STRING
(heredoc_beginning) @RUBY.HEREDOC_ID
(heredoc_body) @RUBY.HEREDOC_CONTENT
(heredoc_body (heredoc_end) @RUBY.HEREDOC_ID)
(regex) @RUBY.REGEXP
(regex "/" @RUBY.REGEXP)
[(simple_symbol) (delimited_symbol) (hash_key_symbol) (bare_symbol)] @RUBY.SYMBOL
(pair key: (hash_key_symbol) ":" @RUBY.SYMBOL)
(escape_sequence) @RUBY.ESCAPE
(interpolation) @TEXT
(interpolation ["#{" "}"] @RUBY.INTERPOLATION)

[
  "alias" "and" "begin" "BEGIN" "break" "case" "class" "def" "defined?" "do" "else" "elsif" "end" "END"
  "ensure" "for" "if" "in" "module" "next" "not" "or" "redo" "rescue" "retry" "return" "then" "undef"
  "unless" "until" "when" "while" "yield"
] @RUBY.KEYWORD
[(self) (super) (nil) (true) (false) (file) (line) (encoding)] @RUBY.KEYWORD

(constant) @RUBY.CLASS
((constant) @RUBY.CONSTANT
 (#match? @RUBY.CONSTANT "^[A-Z][A-Z0-9_]+$"))
(instance_variable) @RUBY.INSTANCE_VARIABLE
(class_variable) @RUBY.CLASS_VARIABLE
(global_variable) @RUBY.GLOBAL_VARIABLE

(method_parameters (identifier) @RUBY.PARAMETER)
(lambda_parameters (identifier) @RUBY.PARAMETER)
(block_parameters (identifier) @RUBY.PARAMETER)
(block_parameter name: (identifier) @RUBY.PARAMETER)
(splat_parameter name: (identifier) @RUBY.PARAMETER)
(hash_splat_parameter name: (identifier) @RUBY.PARAMETER)
(destructured_parameter (identifier) @RUBY.PARAMETER)
(keyword_parameter name: (identifier) @RUBY.PARAMETER)
(optional_parameter name: (identifier) @RUBY.PARAMETER)
(assignment left: (identifier) @RUBY.LOCAL_VARIABLE)
(operator_assignment left: (identifier) @RUBY.LOCAL_VARIABLE)
(left_assignment_list (identifier) @RUBY.LOCAL_VARIABLE)

(call method: [(identifier) (constant)] @RUBY.FUNCTION_CALL)
(call receiver: (_) method: [(identifier) (constant)] @RUBY.METHOD_CALL)
((identifier) @RUBY.KEYWORD
 (#any-of? @RUBY.KEYWORD "private" "protected" "public" "module_function"))
((identifier) @RUBY.PREDEFINED
 (#any-of? @RUBY.PREDEFINED
   "require" "require_relative" "load" "include" "extend" "prepend" "attr_reader" "attr_writer" "attr_accessor"
   "raise" "fail" "puts" "print" "p" "pp" "loop" "lambda" "proc" "block_given?" "catch" "throw" "format"
   "sprintf"))

(method name: [(identifier) (constant) (setter) (operator)] @RUBY.METHOD_DECLARATION)
(singleton_method name: [(identifier) (constant) (setter) (operator)] @RUBY.METHOD_DECLARATION)
(alias name: (identifier) @RUBY.METHOD_DECLARATION)
