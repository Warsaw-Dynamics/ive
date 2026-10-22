use std::path::Path;

use super::keys_at;
use crate::syntax::Language;

#[test]
fn should_tell_shell_variable_declaration_from_use_and_composed_use() {
    // given
    let source = "name=world\necho $name \"${greeting}\"\n";

    // when
    let keys = keys_at("Shell", source, ["name", "$name", "greeting", "echo"]);

    // then
    assert_eq!(
        keys,
        [Some("BASH.VAR_DEF"), Some("BASH.VAR_USE"), Some("BASH.VAR_USE_COMPOSED"), Some("BASH.EXTERNAL_COMMAND")]
    );
}

#[test]
fn should_tell_shell_function_declaration_from_command_and_subshell() {
    // given
    let source = "#!/bin/bash\nfunction build() {\n  make -j 4 > log.txt\n  dir=$(pwd)\n}\n";

    // when
    let keys = keys_at("Shell", source, ["#!", "build", "make", "4", ">", "$("]);

    // then
    assert_eq!(
        keys,
        [
            Some("BASH.SHEBANG"),
            Some("BASH.FUNCTION_DEF_NAME"),
            Some("BASH.EXTERNAL_COMMAND"),
            Some("BASH.NUMBER"),
            Some("BASH.REDIRECTION"),
            Some("BASH.SUBSHELL_COMMAND"),
        ]
    );
}

#[test]
fn should_colour_shell_strings_comments_and_heredocs() {
    // given
    let source = "# note\necho \"double\" 'single'\ncat <<END\nbody\nEND\n";

    // when
    let keys = keys_at("Shell", source, ["# note", "\"double\"", "'single'", "END", "body"]);

    // then
    assert_eq!(
        keys,
        [
            Some("BASH.LINE_COMMENT"),
            Some("BASH.STRING"),
            Some("BASH.RAW_STRING"),
            Some("BASH.HERE_DOC_START"),
            Some("BASH.HERE_DOC"),
        ]
    );
}

#[test]
fn should_match_shell_by_file_name_and_shebang() {
    // given
    let files: [(&str, &[u8]); 4] = [
        (".bashrc", b""),
        ("deploy", b"#!/usr/bin/env bash\necho hi\n"),
        ("setup", b"#!/bin/zsh\n"),
        ("run.sh", b""),
    ];

    // when
    let names = files.map(|(path, content)| Language::for_path(Path::new(path), content).map(Language::name));

    // then
    assert_eq!(names, [Some("Shell"); 4]);
}
