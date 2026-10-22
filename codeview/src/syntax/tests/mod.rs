//! Per-language highlighting tests: each file asserts the distinctions its language's query makes.

mod c;
mod cmake;
mod cpp;
mod csharp;
mod dart;
mod diff;
mod dockerfile;
mod elixir;
mod elm;
mod embedded_template;
mod erlang;
mod fish;
mod fsharp;
mod gleam;
mod glsl;
mod go;
mod gomod;
mod graphql;
mod groovy;
mod haskell;
mod hcl;
mod heex;
mod ini;
mod jsdoc;
mod julia;
mod kdl;
mod kotlin;
mod lua;
mod make;
mod markdown;
mod nix;
mod objc;
mod ocaml;
mod php;
mod powershell;
mod properties;
mod proto;
mod python;
mod r;
mod regexp;
mod ruby;
mod rune;
mod scala;
mod scaling;
mod scss;
mod shell;
mod solidity;
mod sql;
mod starlark;
mod svelte;
mod swift;
mod vue;
mod xml;
mod zig;

use super::{Cancel, PARSE_BUDGET, Syntax, lang, parse, spans};

/// `source` parsed as the registry row named `language`, with its embedded code.
pub fn parsed(language: &str, source: &str) -> Syntax {
    parse(source.as_bytes(), lang(language), &Cancel::default(), PARSE_BUDGET)
        .unwrap_or_else(|error| panic!("{language} did not parse: {error}"))
}

/// The key highlighting the first occurrence of `needle` in `source`, parsed as `language`.
pub fn key_at(language: &str, source: &str, needle: &str) -> Option<&'static str> {
    let syntax = parsed(language, source);
    let at = source.find(needle).unwrap_or_else(|| panic!("{needle} not in source"));
    spans(&syntax, source.as_bytes(), 0..source.len())
        .into_iter()
        .find(|highlight| highlight.bytes.contains(&at))
        .map(|highlight| highlight.key.id())
}

/// `key_at` for each needle, so one assertion shows every distinction at once.
pub fn keys_at<const N: usize>(language: &str, source: &str, needles: [&str; N]) -> [Option<&'static str>; N] {
    needles.map(|needle| key_at(language, source, needle))
}
