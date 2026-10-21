; IVE highlight query for TOML. Capture names are colour key ids (src/colors/keys.rs).
; For the same node a later pattern wins; otherwise the innermost node wins.

(comment) @org.toml.COMMENT
(boolean) @org.toml.BOOLEAN
[(integer) (float)] @org.toml.NUMBER
[(offset_date_time) (local_date_time) (local_date) (local_time)] @org.toml.DATE
(string) @org.toml.STRING
(escape_sequence) @org.toml.VALID_STRING_ESCAPE
[(bare_key) (quoted_key)] @org.toml.KEY
