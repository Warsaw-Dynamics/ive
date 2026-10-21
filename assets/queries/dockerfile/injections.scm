; IVE injection query for Dockerfiles: shell-form RUN, CMD and ENTRYPOINT commands are coloured as Shell.
; The first pattern matching a node wins, so specific patterns come before general ones.

; `RUN <<EOF`: each script line on its own, as the heredoc's newlines are not part of its lines' nodes
((run_instruction
  (shell_command) @_marker
  (heredoc_block (heredoc_line) @injection.content))
 (#match? @_marker "^<<-?[\"']?[A-Za-z_]+[\"']?$")
 (#set! injection.language "bash"))

((shell_command) @injection.content
 (#not-match? @injection.content "^<<")
 (#set! injection.language "bash")
 (#set! injection.include-children))
