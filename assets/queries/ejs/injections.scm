; IVE injection query for EJS: the text between tags is HTML, the code inside them JavaScript; each parsed as one
; document, so a tag or a block may span several template tags.

((content) @injection.content
 (#set! injection.language "html")
 (#set! injection.combined))

((code) @injection.content
 (#set! injection.language "javascript")
 (#set! injection.combined))
