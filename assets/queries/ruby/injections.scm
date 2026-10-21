; IVE injection query for Ruby: regex literals are coloured as regular expressions. The grammar splits a literal
; into text and escapes, so all of them are parsed together as one document.

((regex [(string_content) (escape_sequence)] @injection.content)
 (#set! injection.language "regex")
 (#set! injection.combined))
