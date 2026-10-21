; IVE highlight query for F# (.fs, .fsx). Capture names are colour key ids (src/colors/keys/fsharp.rs).
; For the same node a later pattern wins; otherwise the innermost node wins.

["(" ")"] @FSHARP.PARENTHESES
["[" "]" "[|" "|]"] @FSHARP.BRACKETS
["{" "}" "{|" "|}"] @FSHARP.BRACES
"," @FSHARP.COMMA
"." @FSHARP.DOT
";" @FSHARP.SEMICOLON
["|" "=" ">" "<" "-" "~" "->" "<-" "&" "&&" "||" ":" "::" ":>" ":?>" ".." "*" "?"] @FSHARP.OPERATOR
[(infix_op) (prefix_op) (op_identifier)] @FSHARP.OPERATOR

[
  (xint) (int) (int16) (uint16) (int32) (uint32) (int64) (uint64) (nativeint) (unativeint) (ieee32) (ieee64)
  (float) (decimal)
] @FSHARP.NUMBER
[(string) (triple_quoted_string) (verbatim_string) (char) (format_string) (format_triple_quoted_string)]
  @FSHARP.STRING
(format_string_eval (_) @TEXT)
(format_string_eval ["{" "}"] @FSHARP.INTERPOLATION)
(line_comment) @FSHARP.LINE_COMMENT
(block_comment) @FSHARP.BLOCK_COMMENT
(xml_doc) @FSHARP.DOC_COMMENT
((block_comment (block_comment_content) @_content) @FSHARP.DOC_COMMENT
 (#match? @_content "^\\*"))
[(compiler_directive_decl) (preproc_inactive)] @FSHARP.PREPROCESSOR
["#if" "#else" "#endif" "#line" "#r" "#load"] @FSHARP.PREPROCESSOR

[
  "abstract" "and" "and!" "as" "assert" "begin" "class" "default" "delegate" "do" "do!" "done" "downcast" "downto"
  "elif" "else" "end" "enum" "exception" "extern" "finally" "for" "fun" "function" "get" "global" "if" "in"
  "inherit" "inline" "interface" "lazy" "let" "let!" "match" "match!" "member" "module" "mutable" "namespace"
  "new" "not" "null" "of" "open" "or" "override" "rec" "return" "return!" "set" "static" "struct" "then" "to"
  "try" "type" "upcast" "use" "use!" "val" "when" "while" "with" "yield" "yield!"
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
(field_initializer field: (long_identifier (identifier) @FSHARP.FIELD))

; Paths: `point.X` is a field, `Math.PI` a member of a type, `List.map` a function of a core module.
(long_identifier (identifier) (identifier) @FSHARP.FIELD .)
((long_identifier . (identifier) @FSHARP.TYPE (identifier))
 (#match? @FSHARP.TYPE "^[A-Z]"))
((long_identifier . (identifier) @FSHARP.MODULE (identifier))
 (#any-of? @FSHARP.MODULE "Array" "Async" "List" "Map" "Option" "Result" "Seq" "Set" "String" "Task" "ValueOption"))
((long_identifier . (identifier) @FSHARP.PREDEFINED (identifier))
 (#any-of? @FSHARP.PREDEFINED "this" "base" "self"))
((long_identifier_or_op (identifier) @FSHARP.PREDEFINED)
 (#any-of? @FSHARP.PREDEFINED "this" "base" "self"))
(simple_type (long_identifier (identifier) @FSHARP.TYPE))
(dot_expression field: (long_identifier_or_op (identifier) @FSHARP.FIELD))

; Modules and namespaces.
(namespace name: (long_identifier (identifier) @FSHARP.MODULE))
(named_module name: (long_identifier (identifier) @FSHARP.MODULE))
(module_defn (identifier) @FSHARP.MODULE)
(import_decl (long_identifier (identifier) @FSHARP.MODULE))
(module_abbrev (long_identifier (identifier) @FSHARP.MODULE))

; Values, parameters and functions.
(value_declaration_left (identifier_pattern (long_identifier_or_op (identifier) @FSHARP.LOCAL_VARIABLE)))
(rules (rule pattern: (identifier_pattern (identifier_pattern (long_identifier_or_op (identifier) @FSHARP.LOCAL_VARIABLE)))))
(rules (rule pattern: (cons_pattern (identifier_pattern (long_identifier_or_op (identifier) @FSHARP.LOCAL_VARIABLE)))))
(rules (rule pattern: (identifier_pattern . (long_identifier_or_op (identifier) @FSHARP.LOCAL_VARIABLE) .)))
(argument_patterns (long_identifier (identifier) @FSHARP.PARAMETER))
(argument_patterns (_ (identifier_pattern (long_identifier_or_op (identifier) @FSHARP.PARAMETER))))
(argument_patterns (identifier_pattern (long_identifier_or_op (identifier) @FSHARP.PARAMETER)))
(argument_patterns (typed_pattern (identifier_pattern (long_identifier_or_op (identifier) @FSHARP.PARAMETER))))
(argument_patterns (paren_pattern (typed_pattern (identifier_pattern (long_identifier_or_op (identifier) @FSHARP.PARAMETER)))))
(primary_constr_args (typed_pattern (identifier_pattern (long_identifier_or_op (identifier) @FSHARP.PARAMETER))))
(primary_constr_args (identifier_pattern (long_identifier_or_op (identifier) @FSHARP.PARAMETER)))
(method_or_prop_defn args: (_ (typed_pattern (identifier_pattern (long_identifier_or_op (identifier) @FSHARP.PARAMETER)))))
(method_or_prop_defn args: (identifier_pattern (long_identifier_or_op (identifier) @FSHARP.PARAMETER)))
(fun_expression (argument_patterns (long_identifier (identifier) @FSHARP.PARAMETER)))
(property_or_ident instance: (identifier) @FSHARP.PREDEFINED)
(wildcard_pattern) @FSHARP.KEYWORD

(function_declaration_left . (identifier) @FSHARP.FUNCTION_DECLARATION)
(method_or_prop_defn (property_or_ident . (identifier) @FSHARP.FUNCTION_DECLARATION .) args: (_))
(method_or_prop_defn (property_or_ident method: (identifier) @FSHARP.FUNCTION_DECLARATION) args: (_))
(method_or_prop_defn (property_or_ident . (identifier) @FSHARP.FIELD .) . "=")
(method_or_prop_defn (property_or_ident method: (identifier) @FSHARP.FIELD) . "=")

(application_expression . (long_identifier_or_op (identifier) @FSHARP.FUNCTION_CALL) . (_))
(application_expression . (long_identifier_or_op (long_identifier (identifier) @FSHARP.FUNCTION_CALL .)) . (_))
(application_expression . (dot_expression field: (long_identifier_or_op (identifier) @FSHARP.FUNCTION_CALL)) . (_))
; The function side of a pipe: `xs |> f`, `xs |> List.map g` (read as `(xs |> List.map) g`).
((infix_expression (infix_op) @_pipe . (long_identifier_or_op (identifier) @FSHARP.FUNCTION_CALL))
 (#any-of? @_pipe "|>" "||>" "|||>"))
((infix_expression (infix_op) @_pipe . (long_identifier_or_op (long_identifier (identifier) @FSHARP.FUNCTION_CALL .)))
 (#any-of? @_pipe "|>" "||>" "|||>"))
((long_identifier_or_op . (identifier) @FSHARP.UNION_CASE .)
 (#match? @FSHARP.UNION_CASE "^[A-Z]"))
((long_identifier_or_op (long_identifier . (identifier) @FSHARP.UNION_CASE .))
 (#match? @FSHARP.UNION_CASE "^[A-Z]"))
((long_identifier_or_op (identifier) @FSHARP.KEYWORD)
 (#eq? @FSHARP.KEYWORD "not"))
((identifier) @FSHARP.PREDEFINED
 (#any-of? @FSHARP.PREDEFINED "typeof" "typedefof" "sizeof" "nameof" "failwith" "failwithf" "raise" "reraise"))

(attribute (simple_type (long_identifier (identifier) @FSHARP.ATTRIBUTE)))
(attribute target: (identifier) @FSHARP.KEYWORD)
(attributes ["[<" ">]"] @FSHARP.ATTRIBUTE)
((declaration_expression
   (attributes (attribute (simple_type (long_identifier (identifier) @_literal))))
   (function_or_value_defn
     (value_declaration_left . (identifier_pattern (long_identifier_or_op (identifier) @FSHARP.CONSTANT)))))
 (#eq? @_literal "Literal"))
((value_declaration
   (attributes (attribute (simple_type (long_identifier (identifier) @_literal))))
   (function_or_value_defn
     (value_declaration_left . (identifier_pattern (long_identifier_or_op (identifier) @FSHARP.CONSTANT)))))
 (#eq? @_literal "Literal"))
