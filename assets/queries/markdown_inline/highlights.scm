; IVE highlight query for Markdown inline content (paragraphs, headings, table cells), embedded in Markdown.
; Capture names are colour key ids (src/colors/keys/); keys as IntelliJ's MarkdownSyntaxHighlighter colours the
; same elements. Plain text is left uncaptured, so it shows in the colour of its block.
; For the same node a later pattern wins; otherwise the innermost node wins.

(emphasis) @MARKDOWN_ITALIC
(emphasis (emphasis_delimiter) @MARKDOWN_ITALIC_MARKER)
(strong_emphasis) @MARKDOWN_BOLD
(strong_emphasis (emphasis_delimiter) @MARKDOWN_BOLD_MARKER)
(strikethrough) @MARKDOWN_STRIKE_THROUGH

(code_span) @MARKDOWN_CODE_SPAN
(code_span_delimiter) @MARKDOWN_CODE_SPAN_MARKER

(inline_link) @MARKDOWN_EXPLICIT_LINK
[(full_reference_link) (collapsed_reference_link) (shortcut_link)] @MARKDOWN_REFERENCE_LINK
(image) @MARKDOWN_IMAGE
[(link_text) (image_description)] @MARKDOWN_LINK_TEXT
(link_label) @MARKDOWN_LINK_LABEL
(link_destination) @MARKDOWN_LINK_DESTINATION
(link_title) @MARKDOWN_LINK_TITLE
[(uri_autolink) (email_autolink)] @MARKDOWN_AUTO_LINK

; GitHub alerts (`> [!NOTE]`); this layer cannot see whether the badge opens a blockquote
((shortcut_link (link_text) @_alert) @MARKDOWN_ALERT_TITLE_NOTE (#eq? @_alert "!NOTE"))
((shortcut_link (link_text) @MARKDOWN_ALERT_TITLE_NOTE) (#eq? @MARKDOWN_ALERT_TITLE_NOTE "!NOTE"))
((shortcut_link (link_text) @_alert) @MARKDOWN_ALERT_TITLE_TIP (#eq? @_alert "!TIP"))
((shortcut_link (link_text) @MARKDOWN_ALERT_TITLE_TIP) (#eq? @MARKDOWN_ALERT_TITLE_TIP "!TIP"))
((shortcut_link (link_text) @_alert) @MARKDOWN_ALERT_TITLE_IMPORTANT (#eq? @_alert "!IMPORTANT"))
((shortcut_link (link_text) @MARKDOWN_ALERT_TITLE_IMPORTANT) (#eq? @MARKDOWN_ALERT_TITLE_IMPORTANT "!IMPORTANT"))
((shortcut_link (link_text) @_alert) @MARKDOWN_ALERT_TITLE_WARNING (#eq? @_alert "!WARNING"))
((shortcut_link (link_text) @MARKDOWN_ALERT_TITLE_WARNING) (#eq? @MARKDOWN_ALERT_TITLE_WARNING "!WARNING"))
((shortcut_link (link_text) @_alert) @MARKDOWN_ALERT_TITLE_CAUTION (#eq? @_alert "!CAUTION"))
((shortcut_link (link_text) @MARKDOWN_ALERT_TITLE_CAUTION) (#eq? @MARKDOWN_ALERT_TITLE_CAUTION "!CAUTION"))
