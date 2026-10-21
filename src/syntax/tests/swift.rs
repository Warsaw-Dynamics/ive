use super::keys_at;

#[test]
fn should_tell_swift_method_declaration_from_function_and_calls() {
    // given
    let source = "class Painter {\n    func paint() { spans.append(1) }\n}\nfunc run() { start() }";

    // when
    let keys = keys_at("Swift", source, ["paint", "append", "run", "start"]);

    // then
    assert_eq!(
        keys,
        [
            Some("SWIFT.METHOD_DECLARATION"),
            Some("SWIFT.METHOD_CALL"),
            Some("SWIFT.FUNCTION_DECLARATION"),
            Some("SWIFT.FUNCTION_CALL"),
        ]
    );
}

#[test]
fn should_tell_swift_property_from_local_global_and_parameter() {
    // given
    let source = "let limit = 3\nstruct Box {\n    var size = 0\n    func grow(by step: Int) { let next = size + step }\n}";

    // when
    let keys = keys_at("Swift", source, ["limit", "size", "by", "step", "next"]);

    // then
    assert_eq!(
        keys,
        [
            Some("SWIFT.GLOBAL_VARIABLE"),
            Some("SWIFT.PROPERTY"),
            Some("SWIFT.ARGUMENT_LABEL"),
            Some("SWIFT.PARAMETER"),
            Some("SWIFT.LOCAL_VARIABLE"),
        ]
    );
}

#[test]
fn should_mark_swift_protocol_attribute_enum_case_and_interpolation() {
    // given
    let source = "protocol Painting {}\n@MainActor enum Kind { case plain }\nlet s = \"n \\(Kind.plain)\"";

    // when
    let keys = keys_at("Swift", source, ["Painting", "@MainActor", "plain", "\\("]);

    // then
    assert_eq!(
        keys,
        [
            Some("SWIFT.PROTOCOL"),
            Some("SWIFT.ATTRIBUTE"),
            Some("SWIFT.ENUM_CASE"),
            Some("SWIFT.VALID_STRING_ESCAPE"),
        ]
    );
}

#[test]
fn should_tell_swift_doc_comment_from_line_comment() {
    // given
    let source = "/// doc\n// plain\nlet a = 1";

    // when
    let keys = keys_at("Swift", source, ["/// doc", "// plain"]);

    // then
    assert_eq!(keys, [Some("SWIFT.DOC_COMMENT"), Some("SWIFT.LINE_COMMENT")]);
}
