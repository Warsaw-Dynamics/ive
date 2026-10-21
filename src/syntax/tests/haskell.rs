use super::keys_at;

#[test]
fn should_tell_haskell_signature_from_definition_call_and_parameter() {
    // given
    let source = "area :: Shape -> Double\narea shape = measure shape";

    // when
    let keys = keys_at("Haskell", source, ["area", "Shape", "area shape", "measure", "shape ="]);

    // then
    assert_eq!(
        keys,
        [
            Some("HASKELL.TYPE_SIGNATURE"),
            Some("HASKELL.TYPE"),
            Some("HASKELL.FUNCTION_DECLARATION"),
            Some("HASKELL.FUNCTION_CALL"),
            Some("HASKELL.PARAMETER"),
        ]
    );
}

#[test]
fn should_tell_haskell_type_class_from_type_variable_and_constructor() {
    // given
    let source = "scale :: Num a => a -> Box a\nscale x = Box x";

    // when
    let keys = keys_at("Haskell", source, ["Num", "a =>", "Box a", "Box x"]);

    // then
    assert_eq!(
        keys,
        [
            Some("HASKELL.TYPE_CLASS"),
            Some("HASKELL.TYPE_VARIABLE"),
            Some("HASKELL.TYPE"),
            Some("HASKELL.CONSTRUCTOR"),
        ]
    );
}

#[test]
fn should_tell_haddock_from_plain_comment_and_pragma() {
    // given
    let source = "{-# LANGUAGE GADTs #-}\n-- plain\nx = 1\n-- | documented\ny = 2";

    // when
    let keys = keys_at("Haskell", source, ["LANGUAGE", "plain", "documented"]);

    // then
    assert_eq!(keys, [Some("HASKELL.PRAGMA"), Some("HASKELL.LINE_COMMENT"), Some("HASKELL.DOC_COMMENT")]);
}
