; IVE highlight query for Protocol Buffers. Capture names are colour key ids (src/colors/keys/proto.rs).
; For the same range a later pattern wins; otherwise the narrowest capture wins.

["(" ")"] @PROTO.PARENTHESES
["[" "]" "<" ">"] @PROTO.BRACKETS
["{" "}"] @PROTO.BRACES
"," @PROTO.COMMA
"." @PROTO.DOT
";" @PROTO.SEMICOLON
["=" "-" "+" ":"] @PROTO.OPERATOR

[
  "edition" "enum" "export" "extend" "extensions" "group" "import" "local" "map" "max" "message" "oneof" "option"
  "optional" "package" "public" "repeated" "required" "reserved" "returns" "rpc" "service" "stream" "syntax" "to"
  "weak"
] @PROTO.KEYWORD
(bool) @PROTO.KEYWORD
[(type) (key_type)] @PROTO.SCALAR_TYPE

(comment) @PROTO.LINE_COMMENT
((comment) @PROTO.BLOCK_COMMENT
 (#match? @PROTO.BLOCK_COMMENT "^/\\*"))

[(string) (reserved_identifier)] @PROTO.STRING
(escape_sequence) @PROTO.ESCAPE
[(int_lit) (float_lit)] @PROTO.NUMBER
(field_number) @PROTO.FIELD_NUMBER

(package (full_ident) @PROTO.PACKAGE)
(message_name) @PROTO.MESSAGE
(enum_name) @PROTO.ENUM
(service_name) @PROTO.SERVICE
(rpc_name) @PROTO.RPC
(message_or_enum_type (identifier) @PROTO.TYPE)
(extend (full_ident (identifier) @PROTO.TYPE))
(extension_name type: (full_ident (identifier) @PROTO.TYPE))

[(field (identifier) @PROTO.FIELD) (oneof_field (identifier) @PROTO.FIELD) (map_field (identifier) @PROTO.FIELD)]
(oneof (identifier) @PROTO.FIELD)
(block_lit (identifier) @PROTO.FIELD)
(extension_name name: (full_ident (identifier) @PROTO.FIELD))
(enum_field (identifier) @PROTO.ENUM_VALUE)
(constant (full_ident (identifier) @PROTO.ENUM_VALUE))

[(option (identifier) @PROTO.OPTION) (field_option (identifier) @PROTO.OPTION) (enum_value_option (identifier) @PROTO.OPTION)]
[
  (option (full_ident (identifier) @PROTO.OPTION))
  (field_option (full_ident (identifier) @PROTO.OPTION))
  (enum_value_option (full_ident (identifier) @PROTO.OPTION))
]
