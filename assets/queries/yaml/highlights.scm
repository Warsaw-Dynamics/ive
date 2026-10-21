; IVE highlight query for YAML. Capture names are colour key ids (src/colors/keys.rs).
; For the same node a later pattern wins; otherwise the innermost node wins.

["-" ":" "?" "," "[" "]" "{" "}"] @YAML_SIGN
(plain_scalar) @YAML_SCALAR_VALUE
(single_quote_scalar) @YAML_SCALAR_STRING
(double_quote_scalar) @YAML_SCALAR_DSTRING
(block_scalar) @YAML_SCALAR_LIST
(comment) @YAML_COMMENT
[(anchor) (alias)] @YAML_ANCHOR
(block_mapping_pair
  key: (flow_node [(plain_scalar) (single_quote_scalar) (double_quote_scalar)] @YAML_SCALAR_KEY))
(flow_pair
  key: (flow_node [(plain_scalar) (single_quote_scalar) (double_quote_scalar)] @YAML_SCALAR_KEY))
