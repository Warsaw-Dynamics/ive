; IVE highlight query for Rust. Capture names are colour key ids (src/colors/keys.rs).
; For the same node a later pattern wins; otherwise the innermost node wins.
; "/" and "!" are left out of the operators: doc comments contain them as tokens.

["(" ")"] @RUST.PARENTHESES
["[" "]"] @RUST.BRACKETS
["{" "}"] @RUST.BRACES
"," @RUST.COMMA
"." @RUST.DOT
";" @RUST.SEMICOLON
["=" "+" "-" "*" "%" "==" "!=" "<=" ">=" "&&" "||" "&" "|" "+=" "-=" "=>" "->" "?" ".." "..="] @RUST.OPERATION_SIGN

[(integer_literal) (float_literal)] @RUST.NUMBER
[(string_literal) (raw_string_literal)] @RUST.STRING
(char_literal) @RUST.CHAR
(escape_sequence) @RUST.VALID_STRING_ESCAPE
(line_comment) @RUST.LINE_COMMENT
(block_comment) @RUST.BLOCK_COMMENT
(line_comment (doc_comment)) @RUST.DOC_COMMENT
(block_comment (doc_comment)) @RUST.DOC_COMMENT

[
  "as" "async" "await" "break" "const" "continue" "default" "dyn" "else" "enum" "extern" "fn" "for"
  "gen" "if" "impl" "in" "let" "loop" "macro_rules!" "match" "mod" "move" "pub" "raw" "ref" "return"
  "static" "struct" "trait" "type" "union" "unsafe" "use" "where" "while" "yield"
] @RUST.KEYWORD
[(crate) (mutable_specifier) (super) (self) (boolean_literal)] @RUST.KEYWORD
(primitive_type) @RUST.PRIMITIVE_TYPE

(type_identifier) @RUST.TYPE
((scoped_identifier path: (identifier) @RUST.TYPE)
 (#match? @RUST.TYPE "^[A-Z]"))
((scoped_type_identifier path: (identifier) @RUST.TYPE)
 (#match? @RUST.TYPE "^[A-Z]"))
(trait_item name: (type_identifier) @RUST.TRAIT)
(type_parameter name: (type_identifier) @RUST.TYPE_PARAMETER)
(enum_variant name: (identifier) @RUST.ENUM_VARIANT)
((scoped_identifier name: (identifier) @RUST.ENUM_VARIANT)
 (#match? @RUST.ENUM_VARIANT "^[A-Z]"))
((identifier) @RUST.CONSTANT
 (#match? @RUST.CONSTANT "^[A-Z][A-Z0-9_]+$"))
(const_item name: (identifier) @RUST.CONSTANT)
(static_item name: (identifier) @RUST.CONSTANT)
; in `use` paths a capitalised name is a type (or trait), not an enum variant; all-caps names stay constants
((use_declaration argument: (scoped_identifier name: (identifier) @RUST.TYPE))
 (#match? @RUST.TYPE "^[A-Z]\\w*[a-z]"))
((use_list (identifier) @RUST.TYPE)
 (#match? @RUST.TYPE "^[A-Z]\\w*[a-z]"))
((use_list (scoped_identifier name: (identifier) @RUST.TYPE))
 (#match? @RUST.TYPE "^[A-Z]\\w*[a-z]"))
((use_as_clause path: (scoped_identifier name: (identifier) @RUST.TYPE))
 (#match? @RUST.TYPE "^[A-Z]\\w*[a-z]"))

[(field_identifier) (shorthand_field_identifier)] @RUST.FIELD
(parameter pattern: (identifier) @RUST.PARAMETER)

(function_item name: (identifier) @RUST.FUNCTION_DECLARATION)
(function_signature_item name: (identifier) @RUST.FUNCTION_DECLARATION)
(call_expression function: (identifier) @RUST.FUNCTION_CALL)
(call_expression function: (scoped_identifier name: (identifier) @RUST.FUNCTION_CALL))
(call_expression function: (field_expression field: (field_identifier) @RUST.METHOD_CALL))
(generic_function function: (identifier) @RUST.FUNCTION_CALL)
(generic_function function: (field_expression field: (field_identifier) @RUST.METHOD_CALL))
(macro_invocation macro: (identifier) @RUST.MACRO "!" @RUST.MACRO)
(macro_invocation macro: (scoped_identifier name: (identifier) @RUST.MACRO) "!" @RUST.MACRO)

(lifetime) @RUST.LIFETIME
[(attribute_item) (inner_attribute_item)] @RUST.ATTRIBUTE
