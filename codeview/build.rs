//! Compiles the tree-sitter grammars in `vendor/grammars/` that have no crate usable with codeview's tree-sitter.

use std::path::Path;

/// Each exports `tree_sitter_<name>`; `src/syntax/vendored.rs` declares them.
const VENDORED: [&str; 7] = ["dockerfile", "fish", "gomod", "rune", "scss", "sql", "vue"];

fn main() {
    for name in VENDORED {
        compile(name);
    }
}

/// One static library per grammar: each brings its own `tree_sitter/parser.h`.
fn compile(name: &str) {
    let src = Path::new("../vendor/grammars").join(name).join("src");
    let mut build = cc::Build::new();
    build.include(&src).std("c11").warnings(false).file(src.join("parser.c"));
    let scanner = src.join("scanner.c");
    if scanner.exists() {
        build.file(&scanner);
    }
    build.compile(&format!("ive-grammar-{name}"));
    println!("cargo::rerun-if-changed={}", src.display());
}
