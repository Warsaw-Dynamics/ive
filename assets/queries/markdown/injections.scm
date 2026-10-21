; IVE injection query for Markdown blocks (src/syntax/injection.rs). Children of the content nodes are
; `block_continuation`s (a blockquote's `>`, a list's indent), which stay out of the embedded code.

(fenced_code_block
  (info_string (language) @injection.language)
  (code_fence_content) @injection.content)

((html_block) @injection.content
 (#set! injection.language "html"))

((minus_metadata) @injection.content
 (#set! injection.language "yaml"))

((plus_metadata) @injection.content
 (#set! injection.language "toml"))

((inline) @injection.content
 (#set! injection.language "markdown_inline"))
