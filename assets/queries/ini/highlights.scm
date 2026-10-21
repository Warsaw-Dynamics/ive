; IVE highlight query for INI. Capture names are colour key ids (src/colors/keys/).
; For the same node a later pattern wins; otherwise the innermost node wins.

["[" "]"] @INI.BRACKETS
"=" @INI.OPERATION_SIGN

(comment) @INI.COMMENT
(section_name (text) @INI.SECTION)
(setting_name) @INI.KEY

(setting_value) @INI.VALUE
((setting_value) @INI.NUMBER
 (#match? @INI.NUMBER "^\\s*[-+]?[0-9]+(\\.[0-9]+)?\\s*$"))
((setting_value) @INI.BOOLEAN
 (#match? @INI.BOOLEAN "^\\s*(true|false|yes|no|on|off)\\s*$"))
