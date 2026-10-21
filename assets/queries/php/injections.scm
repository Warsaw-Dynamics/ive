; IVE injection query for PHP: the text around the PHP tags is HTML, parsed as one document; a heredoc or nowdoc
; is coloured in the language its label names (`<<<SQL`, `<<<'HTML'`), when IVE knows it.

((text) @injection.content
 (#set! injection.language "html")
 (#set! injection.combined))

(heredoc
  (heredoc_body) @injection.content
  (heredoc_end) @injection.language
  (#set! injection.include-children))

(nowdoc
  (nowdoc_body) @injection.content
  (heredoc_end) @injection.language
  (#set! injection.include-children))
