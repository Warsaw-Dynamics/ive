use super::keys_at;

#[test]
fn should_tell_zig_function_declaration_from_call_parameter_and_local() {
    // given
    let source = "fn run(limit: u32) u32 {\n    const total = start(limit);\n    return total;\n}";

    // when
    let keys = keys_at("Zig", source, ["run", "limit", "total", "start", "u32"]);

    // then
    assert_eq!(
        keys,
        [
            Some("ZIG.FUNCTION_DECLARATION"),
            Some("ZIG.PARAMETER"),
            Some("ZIG.LOCAL_VARIABLE"),
            Some("ZIG.FUNCTION_CALL"),
            Some("ZIG.BUILTIN_TYPE"),
        ]
    );
}

#[test]
fn should_mark_zig_builtin_type_field_and_container_variable() {
    // given
    let source = "const std = @import(\"std\");\nconst Span = struct { start: u32 };";

    // when
    let keys = keys_at("Zig", source, ["std", "@import", "Span", "start"]);

    // then
    assert_eq!(
        keys,
        [Some("ZIG.GLOBAL_VARIABLE"), Some("ZIG.BUILTIN_FUNCTION"), Some("ZIG.TYPE"), Some("ZIG.FIELD")]
    );
}

#[test]
fn should_tell_zig_doc_comments_from_line_comment() {
    // given
    let source = "//! module doc\n/// item doc\n// plain\nconst a = 1;";

    // when
    let keys = keys_at("Zig", source, ["//!", "/// item", "// plain"]);

    // then
    assert_eq!(keys, [Some("ZIG.DOC_COMMENT"), Some("ZIG.DOC_COMMENT"), Some("ZIG.LINE_COMMENT")]);
}

#[test]
fn should_not_take_zig_assignment_for_declaration() {
    // given
    let source = "fn f() void {\n    var count: u8 = 0;\n    count = 1;\n}";

    // when
    let keys = keys_at("Zig", source, ["count:", "count ="]);

    // then
    assert_eq!(keys, [Some("ZIG.LOCAL_VARIABLE"), None]);
}
