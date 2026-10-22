use super::keys_at;

#[test]
fn should_tell_fsharp_function_declaration_from_call_parameter_and_local() {
    // given
    let source = "let area shape =\n    let total = measure shape\n    total";

    // when
    let keys = keys_at("F#", source, ["area", "shape", "total", "measure"]);

    // then
    assert_eq!(
        keys,
        [
            Some("FSHARP.FUNCTION_DECLARATION"),
            Some("FSHARP.PARAMETER"),
            Some("FSHARP.LOCAL_VARIABLE"),
            Some("FSHARP.FUNCTION_CALL"),
        ]
    );
}

#[test]
fn should_tell_fsharp_union_case_field_module_and_literal_constant() {
    // given
    let source = "type Shape =\n    | Circle of radius: float\n[<Literal>]\nlet MaxSize = 1\nlet n = List.length xs";

    // when
    let keys = keys_at("F#", source, ["Circle", "radius", "Literal", "MaxSize", "List"]);

    // then
    assert_eq!(
        keys,
        [
            Some("FSHARP.UNION_CASE"),
            Some("FSHARP.FIELD"),
            Some("FSHARP.ATTRIBUTE"),
            Some("FSHARP.CONSTANT"),
            Some("FSHARP.MODULE"),
        ]
    );
}

#[test]
fn should_keep_fsharp_interpolated_code_out_of_string_colour() {
    // given
    let source = "/// doc\nlet s = $\"total {count}\" // line";

    // when
    let keys = keys_at("F#", source, ["doc", "total", "{", "count", "line"]);

    // then
    assert_eq!(
        keys,
        [
            Some("FSHARP.DOC_COMMENT"),
            Some("FSHARP.STRING"),
            Some("FSHARP.INTERPOLATION"),
            Some("TEXT"),
            Some("FSHARP.LINE_COMMENT"),
        ]
    );
}

#[test]
fn should_tell_fsharp_signature_function_from_value_and_parameter() {
    // given
    let source = "module M\nval area: shape: Shape -> float\nval origin: int";

    // when
    let keys = keys_at("F# Signature", source, ["area", "shape", "Shape", "origin"]);

    // then
    assert_eq!(
        keys,
        [
            Some("FSHARP.FUNCTION_DECLARATION"),
            Some("FSHARP.PARAMETER"),
            Some("FSHARP.TYPE"),
            Some("FSHARP.LOCAL_VARIABLE"),
        ]
    );
}
