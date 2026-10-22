use super::keys_at;

#[test]
fn should_tell_scss_variable_from_mixin_function_and_property() {
    // given
    let source = "$gap: 4px !default;\n@mixin pad($n) { padding: $n; }\n.a { @include pad(double($gap)); color: red; }";

    // when
    let keys = keys_at("SCSS", source, ["$gap", "!default", "pad(", "double", "padding", "red", "@include", ".a"]);

    // then
    assert_eq!(
        keys,
        [
            Some("SASS_VARIABLE"),
            Some("SASS_DEFAULT"),
            Some("SASS_MIXIN"),
            Some("SASS_FUNCTION"),
            Some("SASS_PROPERTY_NAME"),
            Some("SASS_PROPERTY_VALUE"),
            Some("SASS_KEYWORD"),
            Some("CSS.CLASS_NAME"),
        ]
    );
}

#[test]
fn should_mark_scss_extend_url_and_line_comment() {
    // given
    let source = ".b {\n  @extend .a; // shared\n  background: url(\"x.png\");\n}";

    // when
    let keys = keys_at("SCSS", source, ["@extend", "// shared", "\"x.png\""]);

    // then
    assert_eq!(keys, [Some("SASS_EXTEND"), Some("SASS_COMMENT"), Some("SASS_URL")]);
}
