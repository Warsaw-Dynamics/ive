use super::keys_at;

#[test]
fn should_tell_kdl_node_name_from_property_type_annotation_and_keyword() {
    // given
    let source = "window width=(u16)1280 title=(label)\"IVE\" maximized=#true theme=#null\n";

    // when
    let keys = keys_at("KDL", source, ["window", "width", "(u16)", "1280", "(label)", "#true", "#null"]);

    // then
    assert_eq!(
        keys,
        [
            Some("KDL.NODE"),
            Some("KDL.PROPERTY"),
            Some("KDL.BUILTIN_TYPE"),
            Some("KDL.NUMBER"),
            Some("KDL.TYPE"),
            Some("KDL.KEYWORD"),
            Some("KDL.KEYWORD"),
        ]
    );
}

#[test]
fn should_colour_kdl_slashdashed_node_argument_and_children_as_comment() {
    // given
    let source = "/- sidebar {\n    outline\n}\npanel /-hidden=#true size=2 /-{ recent }\n";

    // when
    let keys = keys_at("KDL", source, ["sidebar", "outline", "hidden", "#true", "size", "recent"]);

    // then
    assert_eq!(
        keys,
        [
            Some("KDL.SLASHDASH_COMMENT"),
            Some("KDL.SLASHDASH_COMMENT"),
            Some("KDL.SLASHDASH_COMMENT"),
            Some("KDL.SLASHDASH_COMMENT"),
            Some("KDL.PROPERTY"),
            Some("KDL.SLASHDASH_COMMENT"),
        ]
    );
}
