use super::keys_at;

#[test]
fn should_tell_kotlin_function_declaration_from_call_and_constructor() {
    // given
    let source = "fun run() { start(); val list = ArrayList<Int>(); list.add(1) }";

    // when
    let keys = keys_at("Kotlin", source, ["run", "start", "ArrayList", "add"]);

    // then
    assert_eq!(
        keys,
        [
            Some("KOTLIN_FUNCTION_DECLARATION"),
            Some("KOTLIN_FUNCTION_CALL"),
            Some("KOTLIN_CONSTRUCTOR"),
            Some("KOTLIN_FUNCTION_CALL"),
        ]
    );
}

#[test]
fn should_tell_kotlin_package_and_instance_property_from_local_and_parameter() {
    // given
    let source = "val top = 1\nclass A(val prop: Int, plain: Int) {\n    var size = 0\n    fun f(q: Int) { val local = q }\n}";

    // when
    let keys = keys_at("Kotlin", source, ["top", "prop", "plain", "size", "q:", "local"]);

    // then
    assert_eq!(
        keys,
        [
            Some("KOTLIN_PACKAGE_PROPERTY"),
            Some("KOTLIN_INSTANCE_PROPERTY"),
            Some("KOTLIN_PARAMETER"),
            Some("KOTLIN_INSTANCE_PROPERTY"),
            Some("KOTLIN_PARAMETER"),
            Some("KOTLIN_LOCAL_VARIABLE"),
        ]
    );
}

#[test]
fn should_tell_kotlin_keyword_from_modifier_and_val_from_var() {
    // given
    let source = "private data class D(val a: Int)\nfun f() { var b = null }";

    // when
    let keys = keys_at("Kotlin", source, ["private", "data", "class", "D", "val", "var", "null"]);

    // then
    assert_eq!(
        keys,
        [
            Some("KOTLIN_BUILTIN_ANNOTATION"),
            Some("KOTLIN_BUILTIN_ANNOTATION"),
            Some("KOTLIN_KEYWORD"),
            Some("KOTLIN_DATA_CLASS"),
            Some("KOTLIN_KEYWORD_VAL"),
            Some("KOTLIN_KEYWORD_VAR"),
            Some("KOTLIN_KEYWORD"),
        ]
    );
}

#[test]
fn should_mark_kotlin_template_entry_escape_and_invalid_escape() {
    // given
    let source = "val s = \"a\\n ${x.size} \\e\"";

    // when
    let keys = keys_at("Kotlin", source, ["\"a", "\\n", "${", "size", "\\e"]);

    // then
    assert_eq!(
        keys,
        [
            Some("KOTLIN_STRING"),
            Some("KOTLIN_STRING_ESCAPE"),
            Some("KOTLIN_STRING_ESCAPE"),
            Some("KOTLIN_INSTANCE_PROPERTY"),
            Some("KOTLIN_INVALID_STRING_ESCAPE"),
        ]
    );
}

#[test]
fn should_mark_kotlin_annotation_label_named_argument_and_kdoc() {
    // given
    let source = "/** doc */\n/* block */\n@Deprecated(message = \"x\")\nfun f() { list.forEach lit@ { if (it == 1) return@lit }; g(count = 2) }";

    // when
    let keys = keys_at("Kotlin", source, ["doc", "block", "Deprecated", "message", "lit@", "it ", "count"]);

    // then
    assert_eq!(
        keys,
        [
            Some("KOTLIN_DOC_COMMENT"),
            Some("KOTLIN_BLOCK_COMMENT"),
            Some("KOTLIN_ANNOTATION"),
            Some("KOTLIN_ANNOTATION_ATTRIBUTE_NAME_ATTRIBUTES"),
            Some("KOTLIN_LABEL"),
            Some("KOTLIN_CLOSURE_DEFAULT_PARAMETER"),
            Some("KOTLIN_NAMED_ARGUMENT"),
        ]
    );
}
