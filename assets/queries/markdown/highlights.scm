; IVE highlight query for Markdown blocks. Capture names are colour key ids (src/colors/keys/); keys as IntelliJ's
; MarkdownSyntaxHighlighter colours the same elements. Inline content is a Markdown Inline layer painted over this.
; For the same node a later pattern wins; otherwise the innermost node wins.

(paragraph) @MARKDOWN_TEXT

(atx_heading (atx_h1_marker)) @MARKDOWN_HEADER_LEVEL_1
(atx_heading (atx_h2_marker)) @MARKDOWN_HEADER_LEVEL_2
(atx_heading (atx_h3_marker)) @MARKDOWN_HEADER_LEVEL_3
(atx_heading (atx_h4_marker)) @MARKDOWN_HEADER_LEVEL_4
(atx_heading (atx_h5_marker)) @MARKDOWN_HEADER_LEVEL_5
(atx_heading (atx_h6_marker)) @MARKDOWN_HEADER_LEVEL_6
(setext_heading (setext_h1_underline)) @MARKDOWN_HEADER_LEVEL_1
(setext_heading (setext_h2_underline)) @MARKDOWN_HEADER_LEVEL_2
(setext_heading (paragraph) @MARKDOWN_HEADER_LEVEL_1 (setext_h1_underline))
(setext_heading (paragraph) @MARKDOWN_HEADER_LEVEL_2 (setext_h2_underline))
[
  (atx_h1_marker) (atx_h2_marker) (atx_h3_marker) (atx_h4_marker) (atx_h5_marker) (atx_h6_marker)
] @MARKDOWN_HEADER_MARKER

(block_quote) @MARKDOWN_BLOCK_QUOTE
(block_quote (paragraph) @MARKDOWN_BLOCK_QUOTE)
(block_quote_marker) @MARKDOWN_BLOCK_QUOTE_MARKER
((block_continuation) @MARKDOWN_BLOCK_QUOTE_MARKER
 (#match? @MARKDOWN_BLOCK_QUOTE_MARKER ">"))

(list_item [(list_marker_minus) (list_marker_plus) (list_marker_star)]) @MARKDOWN_UNORDERED_LIST
(list_item [(list_marker_minus) (list_marker_plus) (list_marker_star)] (paragraph) @MARKDOWN_UNORDERED_LIST)
(list_item [(list_marker_dot) (list_marker_parenthesis)]) @MARKDOWN_ORDERED_LIST
(list_item [(list_marker_dot) (list_marker_parenthesis)] (paragraph) @MARKDOWN_ORDERED_LIST)
[
  (list_marker_minus) (list_marker_plus) (list_marker_star) (list_marker_dot) (list_marker_parenthesis)
  (task_list_marker_checked) (task_list_marker_unchecked)
] @MARKDOWN_LIST_MARKER

(thematic_break) @MARKDOWN_HRULE

(indented_code_block) @MARKDOWN_CODE_BLOCK
(fenced_code_block) @MARKDOWN_CODE_FENCE
(fenced_code_block_delimiter) @MARKDOWN_CODE_FENCE_MARKER
(info_string (language) @MARKDOWN_CODE_FENCE_LANGUAGE)

(html_block) @MARKDOWN_HTML_BLOCK

(pipe_table_header "|" @MARKDOWN_TABLE_SEPARATOR)
(pipe_table_row "|" @MARKDOWN_TABLE_SEPARATOR)
(pipe_table_delimiter_row) @MARKDOWN_TABLE_SEPARATOR

(link_reference_definition) @MARKDOWN_LINK_DEFINITION
(link_label) @MARKDOWN_LINK_LABEL
(link_destination) @MARKDOWN_LINK_DESTINATION
(link_title) @MARKDOWN_LINK_TITLE
