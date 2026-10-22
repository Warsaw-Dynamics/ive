//! Grammars compiled by `build.rs` from `vendor/grammars/`: their crates pin tree-sitter versions IVE cannot use.

use tree_sitter_language::LanguageFn;

unsafe extern "C" {
    fn tree_sitter_dockerfile() -> *const ();
    fn tree_sitter_fish() -> *const ();
    fn tree_sitter_gomod() -> *const ();
    fn tree_sitter_rune() -> *const ();
    fn tree_sitter_scss() -> *const ();
    fn tree_sitter_sql() -> *const ();
    fn tree_sitter_vue() -> *const ();
}

// SAFETY: each function is a tree-sitter grammar's generated entry point, returning a static `TSLanguage`.
pub const DOCKERFILE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_dockerfile) };
pub const FISH: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_fish) };
pub const GOMOD: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_gomod) };
pub const RUNE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_rune) };
pub const SCSS: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_scss) };
pub const SQL: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_sql) };
pub const VUE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_vue) };

#[cfg(test)]
mod tests {
    use tree_sitter::Parser;

    use super::*;

    #[test]
    fn should_load_and_parse_with_every_vendored_grammar() {
        // given
        let samples = [
            ("dockerfile", DOCKERFILE, "FROM alpine:3\nRUN echo hi\n"),
            ("fish", FISH, "function hi\n  echo hi\nend\n"),
            ("gomod", GOMOD, "module example.com/x\n\ngo 1.22\n"),
            ("rune", RUNE, include_str!("../../../assets/scripts/ive.rn")),
            ("scss", SCSS, "$c: red;\n.a { .b { color: $c; } }\n"),
            ("sql", SQL, "SELECT id FROM users WHERE id = 1;\n"),
            ("vue", VUE, "<template><div>{{ x }}</div></template>\n"),
        ];

        // when
        let broken: Vec<&str> = samples
            .into_iter()
            .filter(|(_, grammar, source)| {
                let mut parser = Parser::new();
                let loaded = parser.set_language(&(*grammar).into()).is_ok();
                !loaded || parser.parse(source, None).is_none_or(|tree| tree.root_node().has_error())
            })
            .map(|(name, _, _)| name)
            .collect();

        // then
        assert!(broken.is_empty(), "grammars that failed to load or parse their sample: {broken:?}");
    }
}
