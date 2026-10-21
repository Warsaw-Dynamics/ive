; DTD-only patterns, appended to dtd/common.scm.

(TextDecl) @XML_PROLOGUE
(TextDecl ["<?" "xml" "?>"] @XML_TAG_NAME)
(TextDecl ["version" "encoding"] @XML_ATTRIBUTE_NAME)
(TextDecl [(VersionNum) (EncName) "\"" "'" "="] @XML_ATTRIBUTE_VALUE)

(conditionalSect ["<![" "INCLUDE" "IGNORE" "]]>"] @XML_TAG_NAME)
