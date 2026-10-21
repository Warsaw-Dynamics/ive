; IVE injection query for Elixir: the content of a `~H` sigil is HEEx, of `~r` a regular expression.

((sigil
  (sigil_name) @_name
  (quoted_content) @injection.content)
 (#eq? @_name "H")
 (#set! injection.language "heex"))

; Escapes split a `~r` sigil's content into several nodes; combined, they form one expression again (all `~r`
; sigils of a file are parsed as one document).
((sigil
  (sigil_name) @_name
  [(quoted_content) (escape_sequence)] @injection.content)
 (#any-of? @_name "r" "R")
 (#set! injection.language "regex")
 (#set! injection.combined))
