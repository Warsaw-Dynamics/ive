use super::keys_at;

#[test]
fn should_tell_dart_function_declaration_from_call_method_call_and_constructor() {
    // given
    let source = "void run(int p) { start(); list.add(p); Point(1); }";

    // when
    let keys = keys_at("Dart", source, ["run", "start", "add", "Point", "p)"]);

    // then
    assert_eq!(
        keys,
        [
            Some("DART.FUNCTION_DECLARATION"),
            Some("DART.FUNCTION_CALL"),
            Some("DART.METHOD_CALL"),
            Some("DART.CONSTRUCTOR"),
            Some("DART.PARAMETER"),
        ]
    );
}

#[test]
fn should_tell_dart_field_static_field_top_level_variable_and_local() {
    // given
    let source = "var counter = 0;\nclass A { static const zero = 0; final int size; void m() { var local = size; } }";

    // when
    let keys = keys_at("Dart", source, ["counter", "zero", "size;", "m()", "local"]);

    // then
    assert_eq!(
        keys,
        [
            Some("DART.TOP_LEVEL_VARIABLE"),
            Some("DART.STATIC_FIELD"),
            Some("DART.FIELD"),
            Some("DART.METHOD_DECLARATION"),
            Some("DART.LOCAL_VARIABLE"),
        ]
    );
}

#[test]
fn should_keep_dart_interpolated_code_out_of_string_colour() {
    // given
    let source = "var s = \"total ${count}\\n\";";

    // when
    let keys = keys_at("Dart", source, ["total", "${", "count", "\\n"]);

    // then
    assert_eq!(keys, [Some("DART.STRING"), Some("DART.INTERPOLATION"), Some("TEXT"), Some("DART.ESCAPE")]);
}

#[test]
fn should_tell_dart_doc_comment_annotation_enum_constant_and_interface() {
    // given
    let source = "/// doc\n// line\n@immutable interface class Shape {}\nenum Color { red }";

    // when
    let keys = keys_at("Dart", source, ["doc", "line", "immutable", "Shape", "red"]);

    // then
    assert_eq!(
        keys,
        [
            Some("DART.DOC_COMMENT"),
            Some("DART.LINE_COMMENT"),
            Some("DART.ANNOTATION"),
            Some("DART.INTERFACE"),
            Some("DART.CONSTANT"),
        ]
    );
}
