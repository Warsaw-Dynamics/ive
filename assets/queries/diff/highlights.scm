; IVE highlight query for Diff. Capture names are colour key ids (src/colors/keys/).
; For the same node a later pattern wins; otherwise the innermost node wins.

(addition) @DIFF.ADDED
(deletion) @DIFF.REMOVED
(context) @DIFF.CONTEXT
(comment) @DIFF.COMMENT

(command) @DIFF.COMMAND
[(old_file) (new_file)] @DIFF.FILE_HEADER
(location) @DIFF.HUNK_HEADER
(linerange) @DIFF.LINE_RANGE

; `index`, `new file mode`, `rename from`, `similarity index`, `Binary files ... differ`
[(index) (file_change) (similarity) (binary_change)] @DIFF.EXTENDED_HEADER
(commit) @DIFF.COMMIT
[(mode) (score)] @DIFF.MODE
(file_change (filename) @DIFF.FILE_NAME)
(binary_change (filename) @DIFF.FILE_NAME)
