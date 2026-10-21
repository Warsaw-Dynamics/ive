; IVE injection query for Svelte: embedded code is coloured in its own language (src/syntax/injection.rs).
; The first pattern matching a node wins, so specific patterns come before general ones. Expressions are
; TypeScript, a superset of JavaScript using the same keys, as a query cannot see the `<script>`'s `lang`.

((style_element
  (start_tag
    (attribute
      (attribute_name) @_lang
      (quoted_attribute_value (attribute_value) @injection.language)))
  (raw_text) @injection.content)
 (#eq? @_lang "lang"))

((style_element
  (start_tag
    (attribute
      (attribute_name) @_lang
      (quoted_attribute_value (attribute_value) @_dialect)))
  (raw_text) @injection.content)
 (#eq? @_lang "lang")
 (#any-of? @_dialect "less" "postcss")
 (#set! injection.language "scss"))

((style_element
  (raw_text) @injection.content)
 (#set! injection.language "css"))

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

((svelte_raw_text) @injection.content
 (#set! injection.language "typescript"))
