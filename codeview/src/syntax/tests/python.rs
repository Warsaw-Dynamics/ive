use super::keys_at;

#[test]
fn should_tell_python_function_definition_from_call_and_builtin() {
    // given
    let source = "def run(items):\n    \"\"\"Runs.\"\"\"\n    total = len(items)\n    start(total)\n    items.clear()";

    // when
    let keys = keys_at("Python", source, ["run", "Runs", "total", "len", "start", "clear"]);

    // then
    assert_eq!(
        keys,
        [
            Some("PY.FUNC_DEFINITION"),
            Some("PY.DOC_COMMENT"),
            Some("PY.LOCAL_VARIABLE"),
            Some("PY.BUILTIN_NAME"),
            Some("PY.FUNCTION_CALL"),
            Some("PY.METHOD_CALL"),
        ]
    );
}

#[test]
fn should_mark_python_decorator_self_keyword_argument_and_special_name() {
    // given
    let source = "class Box:\n    @cached(size=2)\n    def __init__(self, x: int):\n        self.x = f\"{x}\"";

    // when
    let keys = keys_at("Python", source, ["Box", "cached", "size", "__init__", "self", "int", "{"]);

    // then
    assert_eq!(
        keys,
        [
            Some("PY.CLASS_DEFINITION"),
            Some("PY.DECORATOR"),
            Some("PY.KEYWORD_ARGUMENT"),
            Some("PY.PREDEFINED_DEFINITION"),
            Some("PY.SELF_PARAMETER"),
            Some("PY.BUILTIN_NAME"),
            Some("PY.FSTRING_FRAGMENT_BRACES"),
        ]
    );
}

#[test]
fn should_tell_python_bytes_from_text_string_and_keyword_from_builtin() {
    // given
    let source = "data = b'\\x00' if True else u'text'\nprint(data)";

    // when
    let keys = keys_at("Python", source, ["b'", "\\x00", "True", "u'", "print"]);

    // then
    assert_eq!(
        keys,
        [
            Some("PY.STRING.B"),
            Some("PY.VALID_STRING_ESCAPE"),
            Some("PY.KEYWORD"),
            Some("PY.STRING.U"),
            Some("PY.BUILTIN_NAME"),
        ]
    );
}
