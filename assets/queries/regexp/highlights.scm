; IVE highlight query for regular expressions (JavaScript flavour). Capture names are colour key ids
; (src/colors/keys/); keys as IntelliJ's RegExpHighlighter colours the same tokens.
; For the same node a later pattern wins; otherwise the innermost node wins.

[(pattern_character) (class_character) (posix_class_name)] @REGEXP.CHARACTER
(class_range "-" @REGEXP.CHARACTER)
(any_character) @REGEXP.DOT

["|" "^"] @REGEXP.META
[(start_assertion) (end_assertion)] @REGEXP.META

[(identity_escape) (control_escape) (control_letter_escape) (unicode_character_escape)] @REGEXP.ESC_CHARACTER
[(character_class_escape) (boundary_assertion) (non_boundary_assertion) (posix_character_class)]
  @REGEXP.CHAR_CLASS

["(" ")" "(?" "(?:" "(?<" "(?P<" "(?P=" ">"] @REGEXP.PARENTHS
(lookaround_assertion ["=" "!" "<"] @REGEXP.PARENTHS)
["[" "]" "[:" ":]"] @REGEXP.BRACKETS
(count_quantifier ["{" "}"] @REGEXP.BRACES)
(count_quantifier "," @REGEXP.COMMA)

[(zero_or_more) (one_or_more) (optional) (count_quantifier) (lazy)] @REGEXP.QUANTIFIER
(count_quantifier (decimal_digits) @REGEXP.QUANTIFIER)

(inline_flags_group [(flags) "-"] @REGEXP.OPTIONS)

[(group_name) (decimal_escape)] @REGEXP.NAME
(backreference_escape ["\\k" "<" ">"] @REGEXP.CHAR_CLASS)
