use super::keys_at;

#[test]
fn should_tell_gleam_function_declaration_from_calls_and_parameter() {
    // given
    let source = "import gleam/list\n\npub fn total(items: List(Int)) -> Int {\n  let sum = list.fold(items, 0, add)\n  round(sum)\n}";

    // when
    let keys = keys_at("Gleam", source, ["total", "items:", "List", "sum =", "list.", "fold", "round"]);

    // then
    assert_eq!(
        keys,
        [
            Some("GLEAM.FUNCTION_DECLARATION"),
            Some("GLEAM.PARAMETER"),
            Some("GLEAM.TYPE"),
            Some("GLEAM.VARIABLE"),
            Some("GLEAM.MODULE"),
            Some("GLEAM.FUNCTION_CALL"),
            Some("GLEAM.FUNCTION_CALL"),
        ]
    );
}

#[test]
fn should_tell_gleam_constructor_from_field_label_and_type_variable() {
    // given
    let source = "pub type Box(a) {\n  Box(size: a)\n}\n\nfn wrap(x) {\n  greet(name: x.size, with: Box(size: x))\n}";

    // when
    let keys = keys_at("Gleam", source, ["Box(a", "a)", "Box(size", "size:", "name:", "size,"]);

    // then
    assert_eq!(
        keys,
        [
            Some("GLEAM.TYPE"),
            Some("GLEAM.TYPE_VARIABLE"),
            Some("GLEAM.CONSTRUCTOR"),
            Some("GLEAM.FIELD"),
            Some("GLEAM.LABEL"),
            Some("GLEAM.FIELD"),
        ]
    );
}

#[test]
fn should_tell_gleam_doc_comments_from_plain_comment_constant_and_attribute() {
    // given
    let source = "//// module docs\n/// item docs\n// plain\n@internal\npub const limit = 5";

    // when
    let keys = keys_at("Gleam", source, ["module docs", "item docs", "plain", "internal", "limit"]);

    // then
    assert_eq!(
        keys,
        [
            Some("GLEAM.MODULE_COMMENT"),
            Some("GLEAM.DOC_COMMENT"),
            Some("GLEAM.LINE_COMMENT"),
            Some("GLEAM.ATTRIBUTE"),
            Some("GLEAM.CONSTANT"),
        ]
    );
}
