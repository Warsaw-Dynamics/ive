use super::{keys_at, parsed};

#[test]
fn should_tell_elm_annotation_from_declaration_call_and_parameter() {
    // given
    let source = "module Cart exposing (total)\n\ntotal : List Int -> Int\ntotal items =\n    List.sum items";

    // when
    let keys = keys_at("Elm", source, ["total : ", "List Int", "total items", "items =", "List.", "sum"]);

    // then
    assert_eq!(
        keys,
        [
            Some("ELM.TYPE_ANNOTATION"),
            Some("ELM.TYPE"),
            Some("ELM.FUNCTION_DECLARATION"),
            Some("ELM.PARAMETER"),
            Some("ELM.MODULE"),
            Some("ELM.FUNCTION_CALL"),
        ]
    );
}

#[test]
fn should_tell_elm_constructor_from_type_variable_field_and_local_binding() {
    // given
    let source = "module M exposing (..)\n\ntype Box a = Box { size : a }\n\nwrap x =\n    let\n        inner = Box { size = x }\n    in\n    inner";

    // when
    let keys = keys_at("Elm", source, ["Box a", "a =", "Box {", "size :", "inner ="]);

    // then
    assert_eq!(
        keys,
        [
            Some("ELM.TYPE"),
            Some("ELM.TYPE_VARIABLE"),
            Some("ELM.CONSTRUCTOR"),
            Some("ELM.FIELD"),
            Some("ELM.LOCAL_VARIABLE"),
        ]
    );
}

#[test]
fn should_tell_elm_doc_comment_from_plain_comments() {
    // given
    let source = "module M exposing (x)\n\n{-| documented -}\nx = 1 -- line\n{- block -}";

    // when
    let keys = keys_at("Elm", source, ["documented", "line", "block", "1"]);

    // then
    assert_eq!(
        keys,
        [Some("ELM.DOC_COMMENT"), Some("ELM.LINE_COMMENT"), Some("ELM.BLOCK_COMMENT"), Some("ELM.NUMBER")]
    );
}

#[test]
fn should_parse_an_elm_glsl_block_as_glsl() {
    // given
    let source = "module M exposing (shader)\n\nshader =\n    [glsl| void main() { gl_FragColor = vec4(1.0); } |]";

    // when
    let syntax = parsed("Elm", source);

    // then
    let layers: Vec<_> = syntax.layers().iter().map(|layer| (layer.language.name(), layer.depth)).collect();
    assert_eq!(layers, [("Elm", 0), ("GLSL", 1)]);
}
