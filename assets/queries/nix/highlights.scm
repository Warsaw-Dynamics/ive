; IVE highlight query for Nix. Capture names are colour key ids (src/colors/keys/nix.rs).
; For the same node a later pattern wins; otherwise the innermost node wins.

["(" ")"] @NIX.PARENTHESES
["[" "]"] @NIX.BRACKETS
["{" "}"] @NIX.BRACES
"," @NIX.COMMA
"." @NIX.DOT
";" @NIX.SEMICOLON
["=" "?" "@" ":" (ellipses)] @NIX.OPERATOR
(unary_expression operator: _ @NIX.OPERATOR)
(binary_expression operator: _ @NIX.OPERATOR)

["assert" "else" "if" "in" "inherit" "let" "or" "rec" "then" "with"] @NIX.KEYWORD

(comment) @NIX.LINE_COMMENT
((comment) @NIX.BLOCK_COMMENT
 (#match? @NIX.BLOCK_COMMENT "^/\\*"))
; RFC 145 doc comments: `/** ... */`, but not the empty `/**/`.
((comment) @NIX.DOC_COMMENT
 (#match? @NIX.DOC_COMMENT "^/\\*\\*[^/]"))

[(integer_expression) (float_expression)] @NIX.NUMBER
[(string_expression) (indented_string_expression)] @NIX.STRING
[(escape_sequence) (dollar_escape)] @NIX.ESCAPE
[(path_expression) (hpath_expression) (spath_expression)] @NIX.PATH
(uri_expression) @NIX.URI
(interpolation) @TEXT
(interpolation ["${" "}"] @NIX.INTERPOLATION)

(binding attrpath: (attrpath attr: [(identifier) (string_expression)] @NIX.ATTRIBUTE_BINDING))
(inherited_attrs attr: (identifier) @NIX.ATTRIBUTE_BINDING)
(select_expression attrpath: (attrpath attr: [(identifier) (string_expression)] @NIX.ATTRIBUTE_SELECTION))
(has_attr_expression attrpath: (attrpath attr: [(identifier) (string_expression)] @NIX.ATTRIBUTE_SELECTION))

(function_expression universal: (identifier) @NIX.PARAMETER)
(formal name: (identifier) @NIX.PARAMETER)

(apply_expression function: (variable_expression name: (identifier) @NIX.FUNCTION_CALL))
(apply_expression function: (select_expression attrpath: (attrpath attr: (identifier) @NIX.FUNCTION_CALL .)))

((variable_expression name: (identifier) @NIX.KEYWORD)
 (#any-of? @NIX.KEYWORD "true" "false" "null"))
((variable_expression name: (identifier) @NIX.BUILTIN)
 (#any-of? @NIX.BUILTIN
  "builtins" "abort" "baseNameOf" "derivation" "dirOf" "fetchGit" "fetchTarball" "fetchTree" "fetchurl"
  "fromTOML" "import" "isNull" "map" "placeholder" "removeAttrs" "scopedImport" "throw" "toString"))
