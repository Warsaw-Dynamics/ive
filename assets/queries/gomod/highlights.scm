; IVE highlight query for go.mod. Capture names are colour key ids (src/colors/keys/).
; For the same node a later pattern wins; otherwise the innermost node wins.
; Quoted paths and versions lex as plain words (the grammar's identifier token wins), so they take their role's key.

["(" ")"] @GOMOD.PARENTHESES
["[" "]"] @GOMOD.BRACKETS
"," @GOMOD.COMMA
"=>" @GOMOD.OPERATION_SIGN

["module" "go" "toolchain" "require" "replace" "exclude" "retract"] @GOMOD.KEYWORD
(comment) @GOMOD.LINE_COMMENT
(module_path) @GOMOD.MODULE_PATH
[(version) (go_version) (toolchain_name)] @GOMOD.VERSION
(file_path) @GOMOD.FILE_PATH
