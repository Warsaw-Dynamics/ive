; IVE injection query for HEEx: the code in `{...}` and `<%= %>` is Elixir (src/syntax/injection.rs).
; Directives that open or close a block (`<%= for x <- xs do %>` ... `<% end %>`) are parsed as one document.

((directive
  [(partial_expression_value) (ending_expression_value)] @injection.content)
 (#set! injection.language "elixir")
 (#set! injection.include-children)
 (#set! injection.combined))

((directive (expression_value) @injection.content)
 (#set! injection.language "elixir"))

; An expression's value may contain nested braces, which are its children.
((expression (expression_value) @injection.content)
 (#set! injection.language "elixir")
 (#set! injection.include-children))
