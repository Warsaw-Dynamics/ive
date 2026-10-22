use super::keys_at;

#[test]
fn should_tell_properties_key_separator_value_and_comment() {
    // given
    let source = "# note\ngreeting = Hello";

    // when
    let keys = keys_at("Properties", source, ["note", "greeting", "=", "Hello"]);

    // then
    assert_eq!(
        keys,
        [
            Some("PROPERTIES.LINE_COMMENT"),
            Some("PROPERTIES.KEY"),
            Some("PROPERTIES.KEY_VALUE_SEPARATOR"),
            Some("PROPERTIES.VALUE"),
        ]
    );
}

#[test]
fn should_tell_properties_valid_escapes_from_invalid_unicode_escape() {
    // given
    let source = "a\\=b : x\\ty\\u0041z\\uXXXX";

    // when
    let keys = keys_at("Properties", source, ["\\=", "\\t", "\\u0041", "\\uX"]);

    // then
    assert_eq!(
        keys,
        [
            Some("PROPERTIES.VALID_STRING_ESCAPE"),
            Some("PROPERTIES.VALID_STRING_ESCAPE"),
            Some("PROPERTIES.VALID_STRING_ESCAPE"),
            Some("PROPERTIES.INVALID_STRING_ESCAPE"),
        ]
    );
}
