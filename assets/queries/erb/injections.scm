; IVE injection query for ERB: the text between tags is HTML, the code inside them Ruby; each parsed as one
; document, so a tag or a block may span several template tags.

((content) @injection.content
 (#set! injection.language "html")
 (#set! injection.combined))

((code) @injection.content
 (#set! injection.language "ruby")
 (#set! injection.combined))
