; IVE highlight query for markup declarations, shared by DTD files and XML's internal DOCTYPE subset (the XML
; grammar parses those itself). Capture names are colour key ids (src/colors/keys/); keys as IntelliJ's
; XmlFileHighlighter colours DTD tokens.
; For the same node a later pattern wins; otherwise the innermost node wins.

[(elementdecl) (AttlistDecl) (GEDecl) (PEDecl) (NotationDecl)] @XML_TAG
(Comment) @XML_COMMENT
(PI) @XML_PROLOGUE

[
  "<!" "ELEMENT" "ATTLIST" "ENTITY" "NOTATION" "SYSTEM" "PUBLIC" "NDATA"
  "#PCDATA" "#REQUIRED" "#IMPLIED" "#FIXED" "EMPTY" "ANY"
] @XML_TAG_NAME
[(StringType) (TokenizedType) (NotationType)] @XML_TAG_NAME
(PI (PITarget) @XML_TAG_NAME)

(Name) @XML_ATTRIBUTE_NAME
(Nmtoken) @XML_ATTRIBUTE_NAME
[(EntityValue) (AttValue) (SystemLiteral) (PubidLiteral)] @XML_ATTRIBUTE_VALUE
[(EntityRef) (CharRef) (PEReference)] @XML_ENTITY_REFERENCE
(EntityRef (Name) @XML_ENTITY_REFERENCE)
(PEReference (Name) @XML_ENTITY_REFERENCE)
