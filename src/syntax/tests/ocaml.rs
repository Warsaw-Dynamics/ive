use super::keys_at;

#[test]
fn should_tell_ocaml_function_from_value_call_and_label() {
    // given
    let source = "let limit = 10\nlet clamp ~max x = min max x\nlet y = clamp ~max:limit 3";

    // when
    let keys = keys_at("OCaml", source, ["limit", "clamp", "max x", "min", "max:"]);

    // then
    assert_eq!(
        keys,
        [
            Some("OCAML.VALUE_DECLARATION"),
            Some("OCAML.FUNCTION_DECLARATION"),
            Some("OCAML.LABEL"),
            Some("OCAML.FUNCTION_CALL"),
            Some("OCAML.LABEL"),
        ]
    );
}

#[test]
fn should_tell_ocaml_module_constructor_attribute_and_doc_comment() {
    // given
    let source = "(** Doc *)\ntype t = Leaf | Node of int [@@deriving show]\nlet x = List.length [Leaf] (* plain *)";

    // when
    let keys = keys_at("OCaml", source, ["Doc", "Leaf", "deriving", "List", "plain"]);

    // then
    assert_eq!(
        keys,
        [
            Some("OCAML.DOC_COMMENT"),
            Some("OCAML.CONSTRUCTOR"),
            Some("OCAML.ATTRIBUTE"),
            Some("OCAML.MODULE"),
            Some("OCAML.BLOCK_COMMENT"),
        ]
    );
}

#[test]
fn should_colour_ocaml_interface_with_the_shared_query() {
    // given
    let source = "(** Area. *)\nval area : ?scale:float -> shape -> float";

    // when
    let keys = keys_at("OCaml Interface", source, ["Area", "area", "scale", "float", "shape"]);

    // then
    assert_eq!(
        keys,
        [
            Some("OCAML.DOC_COMMENT"),
            Some("OCAML.FUNCTION_DECLARATION"),
            Some("OCAML.LABEL"),
            Some("OCAML.PRIMITIVE_TYPE"),
            Some("OCAML.TYPE"),
        ]
    );
}
