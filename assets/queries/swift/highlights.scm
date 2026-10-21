; IVE highlight query for Swift. Capture names are colour key ids (src/colors/keys/).
; For the same node a later pattern wins; otherwise the innermost node wins.
; Variables are told apart only where they are declared: tree-sitter does not resolve names.

["(" ")"] @SWIFT.PARENTHESES
["[" "]"] @SWIFT.BRACKETS
["{" "}"] @SWIFT.BRACES
"," @SWIFT.COMMA
"." @SWIFT.DOT
; the grammar hides statement separators; only a `;` between enum cases is a token
";" @SWIFT.SEMICOLON
[
  "+" "-" "*" "/" "%" "=" "+=" "-=" "*=" "/=" "%=" "<" ">" "<<" ">>" "<=" ">=" "++" "--" "^" "&" "&&" "|" "||"
  "~" "!=" "!==" "==" "===" "?" "??" "->" "..<" "..." ":"
] @SWIFT.OPERATION_SIGN
[(custom_operator) (bang)] @SWIFT.OPERATION_SIGN

[(integer_literal) (hex_literal) (oct_literal) (bin_literal) (real_literal)] @SWIFT.NUMBER
[(line_str_text) (multi_line_str_text) (raw_str_part) (raw_str_end_part) (regex_literal) "\"" "\"\"\""] @SWIFT.STRING
(str_escaped_char) @SWIFT.VALID_STRING_ESCAPE
(line_string_literal ["\\(" ")"] @SWIFT.VALID_STRING_ESCAPE)
(multi_line_string_literal ["\\(" ")"] @SWIFT.VALID_STRING_ESCAPE)
(raw_str_interpolation [(raw_str_interpolation_start) ")"] @SWIFT.VALID_STRING_ESCAPE)
(comment) @SWIFT.LINE_COMMENT
(multiline_comment) @SWIFT.BLOCK_COMMENT
((comment) @SWIFT.DOC_COMMENT
 (#match? @SWIFT.DOC_COMMENT "^///([^/]|$)"))
((multiline_comment) @SWIFT.DOC_COMMENT
 (#match? @SWIFT.DOC_COMMENT "^/\\*\\*[^*/]"))

[
  "actor" "any" "as" "as!" "as?" "associatedtype" "async" "await" "borrowing" "break" "case" "class" "consume"
  "consuming" "continue" "convenience" "deinit" "didSet" "discard" "distributed" "do" "dynamic" "each" "enum"
  "extension" "fallthrough" "fileprivate" "final" "for" "func" "get" "guard" "if" "import" "in" "indirect" "infix"
  "init" "inout" "internal" "is" "isolated" "lazy" "let" "macro" "mutating" "nonisolated" "nonmutating" "open"
  "operator" "optional" "override" "package" "postfix" "precedencegroup" "prefix" "private" "protocol" "public"
  "repeat" "required" "return" "sending" "set" "some" "static" "struct" "subscript" "switch" "try" "typealias"
  "unowned" "unsafe" "var" "weak" "while" "willSet" "yield"
] @SWIFT.KEYWORD
[
  (boolean_literal) (nil_literal) (self_expression) (super_expression) (else) (throws) (where_keyword)
  (throw_keyword) (catch_keyword) (try_operator) (default_keyword) (as_operator) (getter_specifier)
  (setter_specifier) (modify_specifier) (visibility_modifier) (member_modifier) (function_modifier)
  (property_modifier) (parameter_modifier) (inheritance_modifier) (mutation_modifier) (ownership_modifier)
  (value_binding_pattern)
] @SWIFT.KEYWORD
[(directive) (diagnostic) (special_literal) (shebang_line) "#if" "#else" "#elseif" "#endif"] @SWIFT.DIRECTIVE

(type_identifier) @SWIFT.TYPE
(protocol_declaration name: (type_identifier) @SWIFT.PROTOCOL)
(type_parameter (type_identifier) @SWIFT.TYPE_PARAMETER)
(enum_entry name: (simple_identifier) @SWIFT.ENUM_CASE)
(prefix_expression "." target: (simple_identifier) @SWIFT.ENUM_CASE)
((navigation_expression target: (simple_identifier) @SWIFT.TYPE)
 (#match? @SWIFT.TYPE "^[A-Z]"))
(statement_label) @SWIFT.LABEL
(control_transfer_statement ["break" "continue"] result: (simple_identifier) @SWIFT.LABEL)

(navigation_suffix suffix: (simple_identifier) @SWIFT.PROPERTY)
; a property is global unless it sits in a statement list (local) or a type's body (property)
; no pattern is rooted at the file node, which would make the query of any line walk every top-level node of the file
(property_declaration name: (pattern bound_identifier: (simple_identifier) @SWIFT.GLOBAL_VARIABLE))
(statements (property_declaration name: (pattern bound_identifier: (simple_identifier) @SWIFT.LOCAL_VARIABLE)))
(for_statement item: (pattern (simple_identifier) @SWIFT.LOCAL_VARIABLE))
(for_statement item: (pattern (pattern (simple_identifier) @SWIFT.LOCAL_VARIABLE)))
(class_body (property_declaration name: (pattern bound_identifier: (simple_identifier) @SWIFT.PROPERTY)))
(enum_class_body (property_declaration name: (pattern bound_identifier: (simple_identifier) @SWIFT.PROPERTY)))
(protocol_property_declaration name: (pattern bound_identifier: (simple_identifier) @SWIFT.PROPERTY))
(parameter name: (simple_identifier) @SWIFT.PARAMETER)
(lambda_parameter name: (simple_identifier) @SWIFT.PARAMETER)
[(parameter external_name: (simple_identifier) @SWIFT.ARGUMENT_LABEL)
 (lambda_parameter external_name: (simple_identifier) @SWIFT.ARGUMENT_LABEL)
 (value_argument_label (simple_identifier) @SWIFT.ARGUMENT_LABEL)]

(function_declaration name: (simple_identifier) @SWIFT.FUNCTION_DECLARATION)
(class_body (function_declaration name: (simple_identifier) @SWIFT.METHOD_DECLARATION))
(enum_class_body (function_declaration name: (simple_identifier) @SWIFT.METHOD_DECLARATION))
(protocol_function_declaration name: (simple_identifier) @SWIFT.METHOD_DECLARATION)
(init_declaration "init" @SWIFT.METHOD_DECLARATION)
(call_expression (simple_identifier) @SWIFT.FUNCTION_CALL)
((call_expression (simple_identifier) @SWIFT.TYPE)
 (#match? @SWIFT.TYPE "^[A-Z]"))
(call_expression (navigation_expression suffix: (navigation_suffix suffix: (simple_identifier) @SWIFT.METHOD_CALL)))

(attribute "@" @SWIFT.ATTRIBUTE (user_type (type_identifier) @SWIFT.ATTRIBUTE))
(attribute ["@autoclosure" "@escaping"] @SWIFT.ATTRIBUTE)
