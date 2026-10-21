use std::path::Path;

use super::keys_at;
use crate::syntax::Language;

#[test]
fn should_highlight_a_binding_object_in_rust_colours() {
    // given
    let source = "pub fn init(config) {\n    config.keys = [\n        #{ key: \"Enter\", action: act::SendString(\"\\x1b\\r\") },\n    ];\n}\n";

    // when
    let keys = keys_at("Rune", source, ["pub", "init", "(config", "#{", "key", "\"Enter\"", "\\x1b", "SendString"]);

    // then
    assert_eq!(
        keys,
        [
            Some("RUST.KEYWORD"),
            Some("RUST.FUNCTION_DECLARATION"),
            Some("RUST.PARENTHESES"),
            Some("RUST.BRACES"),
            Some("RUST.FIELD"),
            Some("RUST.STRING"),
            Some("RUST.VALID_STRING_ESCAPE"),
            Some("RUST.FUNCTION_CALL"),
        ]
    );
}

#[test]
fn should_colour_template_string_text_apart_from_its_substitution() {
    // given
    let source = "let mods = `${clipboard}|ALT`;\n";

    // when
    let keys = keys_at("Rune", source, ["`${", "${", "clipboard", "}|", "|ALT"]);

    // then
    assert_eq!(keys, [Some("RUST.STRING"), Some("RUST.BRACES"), None, Some("RUST.BRACES"), Some("RUST.STRING")]);
}

#[test]
fn should_take_is_not_and_select_as_keywords() {
    // given
    let source = "async fn f(a, b) {\n    if a is not String { 1 } else { 2 };\n    select { v = a => v, default => 0 }\n}\n";

    // when
    let keys = keys_at("Rune", source, ["is", "not", "select", "default", "(a"]);

    // then
    assert_eq!(keys, [Some("RUST.KEYWORD"), Some("RUST.KEYWORD"), Some("RUST.KEYWORD"), Some("RUST.KEYWORD"), Some("RUST.PARENTHESES")]);
}

#[test]
fn should_match_rune_files_by_extension_shebang_and_fence() {
    // given
    let path = Path::new("init.rn");

    // when
    let languages = [
        Language::for_path(path, b""),
        Language::for_path(Path::new("script"), b"#!/usr/bin/env rune\n"),
        Language::for_alias("rune"),
    ];

    // then
    assert_eq!(languages.map(|language| language.map(Language::name)), [Some("Rune"); 3]);
}
