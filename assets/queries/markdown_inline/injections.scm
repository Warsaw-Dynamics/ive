; IVE injection query for Markdown inline content (src/syntax/injection.rs): inline HTML tags are coloured as
; HTML, all tags of a paragraph as one document so that `<b>` and `</b>` pair up.

((html_tag) @injection.content
 (#set! injection.language "html")
 (#set! injection.combined))
