use std::path::Path;

use super::{keys_at, parsed};
use crate::syntax::Language;

#[test]
fn should_tell_dockerfile_image_parts_from_stage_and_variables() {
    // given
    let source = "ARG BASE=3.20\nFROM alpine:${BASE} AS builder\nENV HOME_DIR=/app\nWORKDIR $HOME_DIR\n";

    // when
    let keys = keys_at("Dockerfile", source, ["FROM", "alpine", "BASE}", "builder", "HOME_DIR", "$HOME_DIR"]);

    // then
    assert_eq!(
        keys,
        [
            Some("DOCKER.KEYWORD"),
            Some("DOCKER.IMAGE_NAME"),
            Some("DOCKER.VARIABLE"),
            Some("DOCKER.STAGE_NAME"),
            Some("DOCKER.VARIABLE_DECLARATION"),
            Some("DOCKER.VARIABLE"),
        ]
    );
}

#[test]
fn should_colour_dockerfile_run_shell_form_as_shell_and_exec_form_as_strings() {
    // given
    let source = "FROM alpine\nRUN apk add curl > /dev/null && echo \"ok\"\nCMD [\"sh\", \"-c\"]\n";

    // when
    let keys = keys_at("Dockerfile", source, ["RUN", "apk", ">", "\"ok\"", "\"sh\""]);

    // then
    assert_eq!(
        keys,
        [
            Some("DOCKER.KEYWORD"),
            Some("BASH.EXTERNAL_COMMAND"),
            Some("BASH.REDIRECTION"),
            Some("BASH.STRING"),
            Some("DOCKER.STRING"),
        ]
    );
}

#[test]
fn should_colour_dockerfile_run_heredoc_lines_as_shell_and_copy_heredoc_as_text() {
    // given
    let source = "FROM alpine\nRUN <<EOF\nmkdir /out\nEOF\nCOPY <<CONF /etc/app.conf\nlevel = 1\nCONF\n";

    // when
    let syntax = parsed("Dockerfile", source);
    let keys = keys_at("Dockerfile", source, ["mkdir", "level"]);

    // then
    let layers: Vec<_> = syntax.layers().iter().map(|layer| (layer.language.name(), layer.depth)).collect();
    assert_eq!(layers, [("Dockerfile", 0), ("Shell", 1)]);
    assert_eq!(keys, [Some("BASH.EXTERNAL_COMMAND"), Some("DOCKER.HEREDOC")]);
}

#[test]
fn should_match_dockerfiles_by_file_name_and_extension() {
    // given
    let paths = ["Dockerfile", "Containerfile", "web.dockerfile"];

    // when
    let names = paths.map(|path| Language::for_path(Path::new(path), b"").map(Language::name));

    // then
    assert_eq!(names, [Some("Dockerfile"); 3]);
}
