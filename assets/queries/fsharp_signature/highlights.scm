; IVE highlight query for F# signature files (.fsi), with the keys of the F# page (src/colors/keys/fsharp.rs).
; The signature grammar has no expressions, so this is fsharp/highlights.scm without them, plus `val` signatures.
; For the same node a later pattern wins; otherwise the innermost node wins.

["(" ")"] @FSHARP.PARENTHESES
["[" "]"] @FSHARP.BRACKETS
["{" "}"] @FSHARP.BRACES
"," @FSHARP.COMMA
"." @FSHARP.DOT
["|" "=" ">" "<" "-" "->" ":" ":>" "*" "?"] @FSHARP.OPERATOR
(op_identifier) @FSHARP.OPERATOR

[
  (xint) (int) (int16) (uint16) (int32) (uint32) (int64) (uint64) (nativeint) (unativeint) (ieee32) (ieee64)
  (float) (decimal)
] @FSHARP.NUMBER
[(string) (triple_quoted_string) (verbatim_string) (char) (format_string) (format_triple_quoted_string)]
  @FSHARP.STRING
(line_comment) @FSHARP.LINE_COMMENT
(block_comment) @FSHARP.BLOCK_COMMENT
(xml_doc) @FSHARP.DOC_COMMENT
((block_comment (block_comment_content) @_content) @FSHARP.DOC_COMMENT
 (#match? @_content "^\\*"))
[(compiler_directive_decl) (preproc_inactive)] @FSHARP.PREPROCESSOR
["#if" "#else" "#endif"] @FSHARP.PREPROCESSOR

[
  "abstract" "and" "as" "begin" "class" "default" "delegate" "do" "end" "enum" "exception" "extern" "get"
  "global" "inherit" "inline" "interface" "let" "member" "module" "mutable" "namespace" "new" "not" "null" "of"
  "open" "or" "override" "rec" "set" "static" "struct" "then" "type" "val" "when" "with"
] @FSHARP.KEYWORD
[(bool) (access_modifier) (unit)] @FSHARP.KEYWORD

; Types and their definitions.
[(simple_type) (type_argument)] @FSHARP.TYPE
(type_name type_name: (_) @FSHARP.TYPE)
(exception_definition exception_name: (_) @FSHARP.TYPE)
(postfix_type (long_identifier) @FSHARP.TYPE)
(union_type_case (identifier) @FSHARP.UNION_CASE)
(enum_type_case . (identifier) @FSHARP.UNION_CASE)
(union_type_field . (identifier) @FSHARP.FIELD (_))
(record_fields (record_field . (identifier) @FSHARP.FIELD))
(simple_type (long_identifier (identifier) @FSHARP.TYPE))

; Modules and namespaces.
(namespace name: (long_identifier (identifier) @FSHARP.MODULE))
(named_module name: (long_identifier (identifier) @FSHARP.MODULE))
(module_defn (identifier) @FSHARP.MODULE)
(import_decl (long_identifier (identifier) @FSHARP.MODULE))
(module_abbrev (long_identifier (identifier) @FSHARP.MODULE))

; Signatures: `val area: shape: Shape -> float`, `member Add: step: int -> unit`.
(argument_name_spec name: (_) @FSHARP.PARAMETER)
(value_definition (value_declaration_left (identifier_pattern (long_identifier_or_op (identifier) @FSHARP.LOCAL_VARIABLE))))
(value_definition
  (value_declaration_left (identifier_pattern (long_identifier_or_op (identifier) @FSHARP.FUNCTION_DECLARATION)))
  (curried_spec (arguments_spec)))
((value_definition
   (attributes (attribute (simple_type (long_identifier (identifier) @_literal))))
   (value_declaration_left (identifier_pattern (long_identifier_or_op (identifier) @FSHARP.CONSTANT))))
 (#eq? @_literal "Literal"))
(member_signature . (identifier) @FSHARP.FUNCTION_DECLARATION)
(wildcard_pattern) @FSHARP.KEYWORD

(attribute (simple_type (long_identifier (identifier) @FSHARP.ATTRIBUTE)))
(attribute target: (identifier) @FSHARP.KEYWORD)
(attributes ["[<" ">]"] @FSHARP.ATTRIBUTE)
