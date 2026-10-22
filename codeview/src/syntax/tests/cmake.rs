use std::path::Path;

use super::keys_at;
use crate::syntax::Language;

#[test]
fn should_tell_cmake_builtin_command_from_user_command_and_declaration() {
    // given
    let source = "function(copy_assets target)\n  add_custom_target(${target})\nendfunction()\ncopy_assets(app)\n";

    // when
    let keys = keys_at("CMake", source, ["copy_assets", "target)", "add_custom_target", "${target}", "copy_assets(app"]);

    // then
    assert_eq!(
        keys,
        [
            Some("CMAKE.FUNCTION_DECLARATION"),
            Some("CMAKE.PARAMETER"),
            Some("CMAKE.BUILTIN_COMMAND"),
            Some("CMAKE.VARIABLE"),
            Some("CMAKE.FUNCTION_CALL"),
        ]
    );
}

#[test]
fn should_tell_cmake_keyword_arguments_from_variables_and_bracket_arguments() {
    // given
    let source = "set(Flags [[-O2]])\nfind_package(Boost 1.80 REQUIRED)\nif(NOT Flags)\nendif()\n#[[ old ]]\n";

    // when
    let keys = keys_at("CMake", source, ["Flags", "[[-O2]]", "1.80", "REQUIRED", "NOT", "Flags)", "#[["]);

    // then
    assert_eq!(
        keys,
        [
            Some("CMAKE.VARIABLE_DECLARATION"),
            Some("CMAKE.BRACKET_ARGUMENT"),
            Some("CMAKE.NUMBER"),
            Some("CMAKE.KEYWORD_ARGUMENT"),
            Some("CMAKE.KEYWORD"),
            Some("CMAKE.VARIABLE"),
            Some("CMAKE.BLOCK_COMMENT"),
        ]
    );
}

#[test]
fn should_match_cmake_by_file_name_and_extension() {
    // given
    let paths = ["CMakeLists.txt", "toolchain.cmake"];

    // when
    let names = paths.map(|path| Language::for_path(Path::new(path), b"").map(Language::name));

    // then
    assert_eq!(names, [Some("CMake"); 2]);
}
