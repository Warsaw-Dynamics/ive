; IVE highlight query for Solidity. Capture names are colour key ids (src/colors/keys/).
; For the same node a later pattern wins; otherwise the innermost node wins.
; Variables are told apart only where they are declared: tree-sitter does not resolve names.

["(" ")"] @SOL.PARENTHESES
["[" "]"] @SOL.BRACKETS
["{" "}"] @SOL.BRACES
"," @SOL.COMMA
"." @SOL.DOT
";" @SOL.SEMICOLON
[
  "=" "+" "-" "*" "/" "%" "**" "==" "!=" "<" ">" "<=" ">=" "&&" "||" "!" "&" "|" "^" "~" "<<" ">>" "?" ":"
  "++" "--" "+=" "-=" "*=" "/=" "%=" "&=" "|=" "^=" "<<=" ">>=" "=>" "->" ":="
] @SOL.OPERATION_SIGN

[(number_literal) (yul_decimal_number) (yul_hex_number)] @SOL.NUMBER
[(string) (hex_string_literal) (unicode_string_literal) (yul_string_literal)] @SOL.STRING
((comment) @SOL.LINE_COMMENT
 (#match? @SOL.LINE_COMMENT "^//"))
((comment) @SOL.BLOCK_COMMENT
 (#match? @SOL.BLOCK_COMMENT "^/\\*"))
((comment) @SOL.DOC_COMMENT
 (#match? @SOL.DOC_COMMENT "^(///|/\\*\\*[^/])"))

[
  "abstract" "anonymous" "as" "assembly" "break" "calldata" "case" "catch" "constant" "constructor" "continue"
  "contract" "default" "delete" "do" "else" "emit" "enum" "error" "event" "external" "fallback" "for" "from"
  "function" "global" "if" "import" "indexed" "interface" "internal" "is" "layout" "let" "library" "mapping"
  "memory" "modifier" "new" "override" "payable" "pragma" "private" "public" "pure" "receive" "return" "returns"
  "revert" "storage" "struct" "switch" "transient" "try" "type" "using" "var" "view" "while"
] @SOL.KEYWORD
[(true) (false) (immutable) (virtual) (number_unit) (yul_leave) (yul_break) (yul_continue) (yul_boolean) (unchecked)] @SOL.KEYWORD
(primitive_type) @SOL.PRIMITIVE_TYPE
(payable_conversion_expression "payable" @SOL.PRIMITIVE_TYPE)

[(solidity_version_comparison_operator) (solidity_version) (any_pragma_token)] @SOL.PRAGMA
(solidity_pragma_token "solidity" @SOL.PRAGMA)

(user_defined_type (identifier) @SOL.TYPE)
((identifier) @SOL.CONSTANT
 (#match? @SOL.CONSTANT "^[A-Z][A-Z0-9_]+$"))
(enum_value) @SOL.CONSTANT

(state_variable_declaration name: (identifier) @SOL.STATE_VARIABLE)
(state_variable_declaration "constant" name: (identifier) @SOL.CONSTANT)
(constant_variable_declaration name: (identifier) @SOL.CONSTANT)
(variable_declaration name: (identifier) @SOL.LOCAL_VARIABLE)
(yul_variable_declaration left: (yul_identifier) @SOL.LOCAL_VARIABLE)
[
  (parameter name: (identifier) @SOL.PARAMETER)
  (event_parameter name: (identifier) @SOL.PARAMETER)
  (error_parameter name: (identifier) @SOL.PARAMETER)
]
(type_name key_identifier: (identifier) @SOL.PARAMETER)
(type_name value_identifier: (identifier) @SOL.PARAMETER)

(struct_member name: (identifier) @SOL.FIELD)
(member_expression property: (identifier) @SOL.FIELD)
(struct_field_assignment name: (identifier) @SOL.FIELD)
(call_struct_argument name: (identifier) @SOL.FIELD)

[
  (contract_declaration name: (identifier) @SOL.TYPE)
  (library_declaration name: (identifier) @SOL.TYPE)
  (struct_declaration name: (identifier) @SOL.TYPE)
  (enum_declaration name: (identifier) @SOL.TYPE)
  (user_defined_type_definition name: (identifier) @SOL.TYPE)
]
(import_directive import_name: (identifier) @SOL.TYPE)
(interface_declaration name: (identifier) @SOL.INTERFACE)
; references can only tell an interface by the IName convention
((import_directive import_name: (identifier) @SOL.INTERFACE)
 (#match? @SOL.INTERFACE "^I[A-Z]"))
((user_defined_type (identifier) @SOL.INTERFACE)
 (#match? @SOL.INTERFACE "^I[A-Z]"))

(function_definition name: (identifier) @SOL.FUNCTION_DECLARATION)
(yul_function_definition . (yul_identifier) @SOL.FUNCTION_DECLARATION)
(call_expression function: (expression (identifier) @SOL.FUNCTION_CALL))
(call_expression function: (expression (member_expression property: (identifier) @SOL.METHOD_CALL)))
(yul_function_call function: (yul_identifier) @SOL.FUNCTION_CALL)
(struct_expression type: (expression (identifier) @SOL.TYPE))
((call_expression function: (expression (identifier) @SOL.TYPE))
 (#match? @SOL.TYPE "^[A-Z][a-z]"))

(event_definition name: (identifier) @SOL.EVENT)
(error_declaration name: (identifier) @SOL.EVENT)
(emit_statement name: (expression (identifier) @SOL.EVENT))
(revert_statement error: (expression (identifier) @SOL.EVENT))

(modifier_definition name: (identifier) @SOL.MODIFIER)
(modifier_invocation (identifier) @SOL.MODIFIER)
((identifier) @SOL.MODIFIER
 (#eq? @SOL.MODIFIER "_"))

((identifier) @SOL.BUILTIN
 (#any-of? @SOL.BUILTIN "abi" "block" "msg" "super" "this" "tx"))
((call_expression function: (expression (identifier) @SOL.BUILTIN))
 (#any-of? @SOL.BUILTIN
   "addmod" "assert" "blobhash" "blockhash" "ecrecover" "gasleft" "keccak256" "mulmod" "require" "revert"
   "ripemd160" "selfdestruct" "sha256"))
(yul_evm_builtin) @SOL.BUILTIN
