use std::path::Path;

use super::keys_at;
use crate::syntax::Language;

#[test]
fn should_tell_fish_variable_declaration_from_expansion_and_option() {
    // given
    let source = "set -gx editor vim\necho $editor\n";

    // when
    let keys = keys_at("fish", source, ["set", "-gx", "editor", "$editor"]);

    // then
    assert_eq!(
        keys,
        [Some("FISH.BUILTIN"), Some("FISH.OPTION"), Some("FISH.VARIABLE_DECLARATION"), Some("FISH.VARIABLE")]
    );
}

#[test]
fn should_tell_fish_function_declaration_from_command_and_keyword() {
    // given
    let source = "function deploy\n    rsync \"dist\\n\" host: 2>&1\nend\n";

    // when
    let keys = keys_at("fish", source, ["deploy", "rsync", "\\n", "2>&1", "end"]);

    // then
    assert_eq!(
        keys,
        [
            Some("FISH.FUNCTION_DECLARATION"),
            Some("FISH.COMMAND"),
            Some("FISH.ESCAPE"),
            Some("FISH.REDIRECTION"),
            Some("FISH.KEYWORD"),
        ]
    );
}

#[test]
fn should_match_fish_by_extension_and_shebang() {
    // given
    let files: [(&str, &[u8]); 2] = [("config.fish", b""), ("install", b"#!/usr/bin/env fish\n")];

    // when
    let names = files.map(|(path, content)| Language::for_path(Path::new(path), content).map(Language::name));

    // then
    assert_eq!(names, [Some("fish"); 2]);
}
