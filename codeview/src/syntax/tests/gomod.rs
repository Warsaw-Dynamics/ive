use super::keys_at;

#[test]
fn should_tell_gomod_directive_from_module_path_version_and_file_path() {
    // given
    let source = "module example.com/ive\n\ngo 1.23\n\nreplace example.com/old v1.2.3 => ../old\n";

    // when
    let keys = keys_at("go.mod", source, ["replace", "example.com/old", "v1.2.3", "=>", "../old"]);

    // then
    assert_eq!(
        keys,
        [
            Some("GOMOD.KEYWORD"),
            Some("GOMOD.MODULE_PATH"),
            Some("GOMOD.VERSION"),
            Some("GOMOD.OPERATION_SIGN"),
            Some("GOMOD.FILE_PATH"),
        ]
    );
}

#[test]
fn should_mark_gomod_go_version_and_comment() {
    // given
    let source = "module example.com/ive\n\ngo 1.23\n\nrequire example.com/dep v0.1.0 // indirect\n";

    // when
    let keys = keys_at("go.mod", source, ["1.23", "// indirect"]);

    // then
    assert_eq!(keys, [Some("GOMOD.VERSION"), Some("GOMOD.LINE_COMMENT")]);
}
