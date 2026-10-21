; IVE injection query for Makefiles: recipe lines and `!=`/`$(shell ...)` commands are coloured as Shell.
; Make references inside them stay make's (children are cut out); a line starting with one ($(CC) ...) is not
; injected, as the shell would then read its first argument as the command.

((shell_text) @injection.content
 (#not-match? @injection.content "^\\$[^$]")
 (#set! injection.language "bash"))

((shell_command) @injection.content
 (#not-match? @injection.content "^\\$[^$]")
 (#set! injection.language "bash"))
