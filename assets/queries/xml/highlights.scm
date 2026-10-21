; XML-only patterns, appended to dtd/common.scm (markup declarations of the DOCTYPE's internal subset).
; Keys as IntelliJ's XmlFileHighlighter colours XML tokens.

(XMLDecl) @XML_PROLOGUE
(XMLDecl ["<?" "xml" "?>"] @XML_TAG_NAME)
(XMLDecl ["version" "encoding" "standalone"] @XML_ATTRIBUTE_NAME)
(XMLDecl [(VersionNum) (EncName) "yes" "no" "\"" "'" "="] @XML_ATTRIBUTE_VALUE)
[(StyleSheetPI) (XmlModelPI)] @XML_PROLOGUE
(StyleSheetPI ["<?" "xml-stylesheet" "?>"] @XML_TAG_NAME)
(XmlModelPI ["<?" "xml-model" "?>"] @XML_TAG_NAME)
(PseudoAtt (Name) @XML_ATTRIBUTE_NAME)
(PseudoAtt (PseudoAttValue) @XML_ATTRIBUTE_VALUE)

(doctypedecl) @XML_TAG
(doctypedecl ["<!" "DOCTYPE"] @XML_TAG_NAME)

(CharData) @XML_TAG_DATA
(CDSect (CData) @XML_TAG_DATA)

[(STag) (ETag) (EmptyElemTag)] @XML_TAG
(STag (Name) @XML_TAG_NAME)
(ETag (Name) @XML_TAG_NAME)
(EmptyElemTag (Name) @XML_TAG_NAME)
(Attribute (Name) @XML_ATTRIBUTE_NAME)
(Attribute ["=" (AttValue)] @XML_ATTRIBUTE_VALUE)
