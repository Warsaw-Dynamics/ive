; IVE injection query for HTML: embedded code is coloured in its own language (src/syntax/injection.rs).
; The first pattern matching a node wins, so specific patterns come before general ones.

((script_element
  (start_tag
    (attribute
      (attribute_name) @_type
      (quoted_attribute_value (attribute_value) @_mime)))
  (raw_text) @injection.content)
 (#eq? @_type "type")
 (#match? @_mime "json")
 (#set! injection.language "json"))

((script_element
  (start_tag
    (attribute
      (attribute_name) @_lang
      (quoted_attribute_value (attribute_value) @injection.language)))
  (raw_text) @injection.content)
 (#eq? @_lang "lang"))

((script_element
  (raw_text) @injection.content)
 (#set! injection.language "javascript"))

((style_element
  (raw_text) @injection.content)
 (#set! injection.language "css"))
