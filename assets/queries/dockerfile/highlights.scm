; IVE highlight query for Dockerfiles. Capture names are colour key ids (src/colors/keys/dockerfile.rs).
; For the same node a later pattern wins; otherwise the innermost node wins.
; Shell-form commands are coloured as Shell by injections.scm.

["[" "]"] @DOCKER.BRACKETS
"," @DOCKER.COMMA
["=" ":" "@" (line_continuation)] @DOCKER.OPERATOR

[
  "FROM" "AS" "RUN" "CMD" "LABEL" "EXPOSE" "ENV" "ADD" "COPY" "ENTRYPOINT" "VOLUME" "USER" "WORKDIR" "ARG"
  "ONBUILD" "STOPSIGNAL" "HEALTHCHECK" "SHELL" "MAINTAINER" "CROSS_BUILD"
] @DOCKER.KEYWORD

(comment) @DOCKER.LINE_COMMENT
((source_file . (comment) @DOCKER.PARSER_DIRECTIVE)
 (#match? @DOCKER.PARSER_DIRECTIVE "^#\\s*(syntax|escape|check)\\s*="))
[(double_quoted_string) (single_quoted_string) (json_string)] @DOCKER.STRING
(escape_sequence) @DOCKER.ESCAPE
(expose_port) @DOCKER.NUMBER
(heredoc_line) @DOCKER.HEREDOC
; a `RUN <<EOF` script is Shell; plain text under it, so the shell colours alone show
((run_instruction
  (shell_command) @_marker
  (heredoc_block (heredoc_line) @TEXT))
 (#match? @_marker "^<<-?[\"']?[A-Za-z_]+[\"']?$"))
[(heredoc_marker) (heredoc_end)] @DOCKER.HEREDOC_MARKER

(image_name) @DOCKER.IMAGE_NAME
[(image_tag) (image_digest)] @DOCKER.IMAGE_TAG
(image_alias) @DOCKER.STAGE_NAME
[(param) (mount_param)] @DOCKER.OPTION
(mount_param name: _ @DOCKER.OPTION)
(label_pair key: (_) @DOCKER.LABEL_KEY)

(arg_instruction name: (_) @DOCKER.VARIABLE_DECLARATION)
(env_pair name: (_) @DOCKER.VARIABLE_DECLARATION)
(expansion) @DOCKER.VARIABLE
(expansion ["$" "{" "}"] @DOCKER.VARIABLE)
(expansion (variable) @DOCKER.VARIABLE)
