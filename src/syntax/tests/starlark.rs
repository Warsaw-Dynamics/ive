use std::path::Path;

use super::keys_at;
use crate::syntax::Language;

#[test]
fn should_tell_starlark_rule_call_from_builtin_keyword_argument_and_label() {
    // given
    let source = "go_library(\n    name = \"api\",\n    srcs = glob([\"*.go\"]),\n    deps = [\"//lib:core\"],\n)\n";

    // when
    let keys = keys_at("Starlark", source, ["go_library", "name", "\"api\"", "glob", "\"//lib:core\""]);

    // then
    assert_eq!(
        keys,
        [
            Some("STARLARK.RULE_CALL"),
            Some("STARLARK.KEYWORD_ARGUMENT"),
            Some("STARLARK.STRING"),
            Some("STARLARK.BUILTIN"),
            Some("STARLARK.LABEL"),
        ]
    );
}

#[test]
fn should_tell_starlark_function_declaration_from_parameter_field_and_docstring() {
    // given
    let source = "def _impl(ctx):\n    \"\"\"Builds it.\"\"\"\n    return info(path = ctx.file.src)\n";

    // when
    let keys = keys_at("Starlark", source, ["_impl", "ctx", "\"\"\"Builds", "info", "file"]);

    // then
    assert_eq!(
        keys,
        [
            Some("STARLARK.FUNCTION_DECLARATION"),
            Some("STARLARK.PARAMETER"),
            Some("STARLARK.DOC_STRING"),
            Some("STARLARK.FUNCTION_CALL"),
            Some("STARLARK.FIELD"),
        ]
    );
}

#[test]
fn should_match_bazel_files_by_name_and_extension() {
    // given
    let paths = ["BUILD.bazel", "BUILD", "WORKSPACE", "MODULE.bazel", "defs.bzl"];

    // when
    let names = paths.map(|path| Language::for_path(Path::new(path), b"").map(Language::name));

    // then
    assert_eq!(names, [Some("Starlark"); 5]);
}
