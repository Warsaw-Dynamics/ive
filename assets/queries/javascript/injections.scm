; IVE injection query shared by JavaScript, TypeScript and TSX (src/syntax/injection.rs).

((comment) @injection.content
 (#match? @injection.content "^/\\*\\*[^/]")
 (#set! injection.language "jsdoc"))

((regex_pattern) @injection.content
 (#set! injection.language "regex"))
