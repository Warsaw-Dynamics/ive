; IVE highlight query for ERB and EJS. Capture names are colour key ids (src/colors/keys/embedded_template.rs).
; The code and the content between the tags are injected (erb/injections.scm, ejs/injections.scm).

["<%" "<%_" "<%-" "<%|" "%>" "-%>" "_%>" "=%>"] @ETMPL.DELIMITER
["<%=" "<%==" "<%|=" "<%|==" "<%~"] @ETMPL.OUTPUT_DELIMITER
(comment_directive) @ETMPL.COMMENT
(comment_directive "%>" @ETMPL.COMMENT)
