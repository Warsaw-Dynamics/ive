use super::keys_at;

#[test]
fn should_tell_csharp_method_declaration_from_call_and_parameter_from_local() {
    // given
    let source = "class A { void Run(int p) { var local = p; list.Add(local); } }";

    // when
    let keys = keys_at("C#", source, ["Run", "Add", "p)", "local", "var"]);

    // then
    assert_eq!(
        keys,
        [
            Some("CS.METHOD_DECLARATION"),
            Some("CS.METHOD_CALL"),
            Some("CS.PARAMETER"),
            Some("CS.LOCAL_VARIABLE"),
            Some("CS.KEYWORD"),
        ]
    );
}

#[test]
fn should_tell_csharp_field_property_constant_and_static_field() {
    // given
    let source = "class A { const int Max = 1; static int count; int size; public int Size => size; }";

    // when
    let keys = keys_at("C#", source, ["Max", "count", "size;", "Size =>"]);

    // then
    assert_eq!(keys, [Some("CS.CONSTANT"), Some("CS.STATIC_FIELD"), Some("CS.FIELD"), Some("CS.PROPERTY")]);
}

#[test]
fn should_keep_csharp_interpolated_code_out_of_string_colour() {
    // given
    let source = "var s = $\"total {count}\\n\";";

    // when
    let keys = keys_at("C#", source, ["total", "{", "count", "\\n"]);

    // then
    assert_eq!(keys, [Some("CS.STRING"), Some("CS.INTERPOLATION"), None, Some("CS.ESCAPE")]);
}

#[test]
fn should_tell_csharp_doc_comment_namespace_attribute_and_interface() {
    // given
    let source = "using System.Text;\n/// doc\n// line\n[Serializable] class A : IShape { }";

    // when
    let keys = keys_at("C#", source, ["Text", "doc", "line", "Serializable", "IShape"]);

    // then
    assert_eq!(
        keys,
        [
            Some("CS.NAMESPACE"),
            Some("CS.DOC_COMMENT"),
            Some("CS.LINE_COMMENT"),
            Some("CS.ATTRIBUTE"),
            Some("CS.INTERFACE"),
        ]
    );
}
