use super::keys_at;

#[test]
fn should_tell_go_function_declaration_from_call_and_field() {
    // given
    let source = "package main\nfunc run(p T) { start(); p.size = 1 }";

    // when
    let keys = keys_at("Go", source, ["run", "start", "size", "1"]);

    // then
    assert_eq!(keys, [Some("GO.FUNCTION_DECLARATION"), Some("GO.FUNCTION_CALL"), Some("GO.FIELD"), Some("GO.NUMBER")]);
}

#[test]
fn should_tell_go_method_declaration_from_method_call_and_receiver() {
    // given
    let source = "package main\nfunc (s *Span) Len() int { return s.inner.Width() }";

    // when
    let keys = keys_at("Go", source, ["s *", "Len", "Width", "int"]);

    // then
    assert_eq!(
        keys,
        [Some("GO.RECEIVER"), Some("GO.METHOD_DECLARATION"), Some("GO.METHOD_CALL"), Some("GO.BUILTIN_TYPE")]
    );
}

#[test]
fn should_mark_go_package_builtin_struct_and_interface() {
    // given
    let source = "package colors\ntype Span struct{}\ntype Painter interface{}\nfunc f() { fmt.Println(len(x)) }";

    // when
    let keys = keys_at("Go", source, ["colors", "Span", "Painter", "fmt", "Println", "len"]);

    // then
    assert_eq!(
        keys,
        [
            Some("GO.PACKAGE"),
            Some("GO.STRUCT"),
            Some("GO.INTERFACE"),
            Some("GO.PACKAGE"),
            Some("GO.FUNCTION_CALL"),
            Some("GO.BUILTIN_FUNCTION"),
        ]
    );
}

#[test]
fn should_mark_go_constant_package_variable_local_and_label() {
    // given
    let source = "package main\nconst limit = 3\nvar ready bool\nfunc f() { count := limit\nloop:\n for { break loop } }";

    // when
    let keys = keys_at("Go", source, ["limit", "ready", "count", "loop"]);

    // then
    assert_eq!(keys, [Some("GO.CONSTANT"), Some("GO.GLOBAL_VARIABLE"), Some("GO.LOCAL_VARIABLE"), Some("GO.LABEL")]);
}
