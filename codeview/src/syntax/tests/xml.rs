use super::keys_at;

#[test]
fn should_tell_xml_tag_name_from_attribute_name_value_and_data() {
    // given
    let source = "<?xml version=\"1.0\"?>\n<index kind=\"main\">text &amp; more<!-- note --></index>";

    // when
    let keys = keys_at("XML", source, ["index", "kind", "\"main\"", "text", "&amp;", "note", "version"]);

    // then
    assert_eq!(
        keys,
        [
            Some("XML_TAG_NAME"),
            Some("XML_ATTRIBUTE_NAME"),
            Some("XML_ATTRIBUTE_VALUE"),
            Some("XML_TAG_DATA"),
            Some("XML_ENTITY_REFERENCE"),
            Some("XML_COMMENT"),
            Some("XML_ATTRIBUTE_NAME"),
        ]
    );
}

#[test]
fn should_colour_xml_internal_doctype_subset_as_dtd() {
    // given
    let source = "<!DOCTYPE r [\n  <!ENTITY product \"IVE\">\n]>\n<r>&product;</r>";

    // when
    let keys = keys_at("XML", source, ["ENTITY", "product", "\"IVE\"", "&product;"]);

    // then
    assert_eq!(
        keys,
        [Some("XML_TAG_NAME"), Some("XML_ATTRIBUTE_NAME"), Some("XML_ATTRIBUTE_VALUE"), Some("XML_ENTITY_REFERENCE")]
    );
}

#[test]
fn should_tell_dtd_declaration_keyword_from_name_value_and_reference() {
    // given
    let source = "<!ENTITY % common \"id ID #IMPLIED\">\n<!ATTLIST note %common; kind CDATA #REQUIRED>";

    // when
    let keys = keys_at("DTD", source, ["ATTLIST", "note", "\"id", "%common;", "#REQUIRED"]);

    // then
    assert_eq!(
        keys,
        [
            Some("XML_TAG_NAME"),
            Some("XML_ATTRIBUTE_NAME"),
            Some("XML_ATTRIBUTE_VALUE"),
            Some("XML_ENTITY_REFERENCE"),
            Some("XML_TAG_NAME"),
        ]
    );
}
