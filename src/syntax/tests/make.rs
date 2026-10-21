use std::path::Path;

use super::{keys_at, parsed};
use crate::syntax::Language;

#[test]
fn should_tell_make_target_from_prerequisite_and_variables() {
    // given
    let source = "SRC := $(wildcard *.c)\n.PHONY: app\napp: main.o $(SRC)\n\tcc -o $@ main.o\n";

    // when
    let keys = keys_at("Makefile", source, ["SRC", "wildcard", ".PHONY", "app:", "main.o", "$(SRC)", "$@"]);

    // then
    assert_eq!(
        keys,
        [
            Some("MAKE.VARIABLE_DECLARATION"),
            Some("MAKE.FUNCTION"),
            Some("MAKE.SPECIAL_TARGET"),
            Some("MAKE.TARGET"),
            Some("MAKE.PREREQUISITE"),
            Some("MAKE.VARIABLE"),
            Some("MAKE.AUTOMATIC_VARIABLE"),
        ]
    );
}

#[test]
fn should_colour_make_recipe_as_shell() {
    // given
    let source = "ifeq ($(CI),1)\nlog:\n\t@echo \"start\" > $(LOG)\nendif\n";

    // when
    let keys = keys_at("Makefile", source, ["ifeq", "echo", "\"start\"", ">", "$(LOG)"]);

    // then
    assert_eq!(
        keys,
        [
            Some("MAKE.KEYWORD"),
            Some("BASH.EXTERNAL_COMMAND"),
            Some("BASH.STRING"),
            Some("BASH.REDIRECTION"),
            Some("MAKE.VARIABLE"),
        ]
    );
}

#[test]
fn should_parse_each_make_recipe_line_as_its_own_shell_layer() {
    // given
    let source = "all:\n\tmkdir -p out\n\t$(CC) -o out/app\n\ttouch out/stamp\n";

    // when
    let syntax = parsed("Makefile", source);

    // then
    let layers: Vec<_> = syntax.layers().iter().map(|layer| (layer.language.name(), layer.depth)).collect();
    assert_eq!(layers, [("Makefile", 0), ("Shell", 1), ("Shell", 1)]);
}

#[test]
fn should_match_makefiles_by_file_name_and_extension() {
    // given
    let paths = ["Makefile", "GNUmakefile", "rules.mk"];

    // when
    let names = paths.map(|path| Language::for_path(Path::new(path), b"").map(Language::name));

    // then
    assert_eq!(names, [Some("Makefile"); 3]);
}
