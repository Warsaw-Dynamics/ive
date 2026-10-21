; TypeScript-only patterns, appended to javascript/common.scm.

[
  "abstract" "declare" "enum" "implements" "interface" "keyof" "namespace" "private" "protected"
  "public" "readonly" "type" "override" "satisfies"
] @JS.KEYWORD

(type_identifier) @TS.TYPE
(predefined_type) @TS.PRIMITIVE_TYPE
(class_declaration name: (type_identifier) @JS.CLASS)
(interface_declaration name: (type_identifier) @TS.INTERFACE)
(type_alias_declaration name: (type_identifier) @TS.TYPE_ALIAS)
(enum_declaration name: (identifier) @TS.ENUM)
(type_parameter name: (type_identifier) @TS.TYPE_PARAMETER)
(required_parameter pattern: (identifier) @JS.PARAMETER)
(optional_parameter pattern: (identifier) @JS.PARAMETER)
