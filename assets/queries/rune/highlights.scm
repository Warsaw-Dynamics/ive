; IVE highlight query for Rune, appended to Rust's: the grammar is tree-sitter-rust's plus Rune's syntax, and Rune
; shares Rust's colours and Settings page. For the same node a later pattern wins.

["is" "not" "select"] @RUST.KEYWORD
(select_arm "default" @RUST.KEYWORD)

(object_expression ["#{" "}"] @RUST.BRACES)
(object_pattern ["#{" "}"] @RUST.BRACES)
(object_entry key: (identifier) @RUST.FIELD)
(object_pattern_entry key: (identifier) @RUST.FIELD)
(object_pattern (identifier) @RUST.FIELD)

(template_string "`" @RUST.STRING)
(template_content) @RUST.STRING
(template_substitution ["${" "}"] @RUST.BRACES)

; Rune parameters are bare patterns, without Rust's `name: Type`
(parameters (identifier) @RUST.PARAMETER)
