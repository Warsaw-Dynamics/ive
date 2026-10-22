use super::keys_at;

#[test]
fn should_tell_c_function_declaration_from_call_parameter_and_local() {
    // given
    let source = "int run(int count) { int total = 0; start(count); return total; }";

    // when
    let keys = keys_at("C", source, ["run", "count", "total", "start"]);

    // then
    assert_eq!(
        keys,
        [Some("C.FUNCTION_DECLARATION"), Some("C.PARAMETER"), Some("C.LOCAL_VARIABLE"), Some("C.FUNCTION_CALL")]
    );
}

#[test]
fn should_mark_c_type_field_global_and_label() {
    // given
    let source = "struct point { int x; };\nstatic int ready;\nvoid f(struct point *p) { p->x = 1; done: return; }";

    // when
    let keys = keys_at("C", source, ["point", "x;", "ready", "done"]);

    // then
    assert_eq!(keys, [Some("C.TYPE"), Some("C.FIELD"), Some("C.GLOBAL_VARIABLE"), Some("C.LABEL")]);
}

#[test]
fn should_mark_c_directive_macro_constant_and_macro_body() {
    // given
    let source = "#define LIMIT 42\n#define SQUARE(n) ((n) * (n))\nint a = SQUARE(LIMIT);";

    // when
    let keys = keys_at("C", source, ["#define", "SQUARE", "LIMIT)", "42"]);

    // then
    assert_eq!(keys, [Some("C.PREPROCESSOR_DIRECTIVE"), Some("C.MACRO"), Some("C.CONSTANT"), Some("C.NUMBER")]);
}

#[test]
fn should_tell_c_doc_comments_from_plain_comments() {
    // given
    let source = "/** doc */\n/// line doc\n/* block */\n// line\nint x;";

    // when
    let keys = keys_at("C", source, ["/** doc", "/// line", "/* block", "// line\n"]);

    // then
    assert_eq!(keys, [Some("C.DOC_COMMENT"), Some("C.DOC_COMMENT"), Some("C.BLOCK_COMMENT"), Some("C.LINE_COMMENT")]);
}
