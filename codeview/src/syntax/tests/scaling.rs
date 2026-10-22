//! Highlighting a line must cost about the same however long the file is. A pattern rooted at the file node, such
//! as `(translation_unit (declaration …))`, makes tree-sitter's ranged query walk every top-level node of the file
//! for any line, so in a big file every line costs milliseconds instead of microseconds.

use std::time::{Duration, Instant};

use crate::syntax::{Cancel, Language, Syntax, lang, parse, spans};

const TARGET_LINES: usize = 10_000;
const RUNS: usize = 5;
/// A line of the long file may cost this many times the same line of its sample, plus `SLACK`, before it counts as
/// growth with file length.
const ALLOWED_FACTOR: u32 = 8;
const SLACK: Duration = Duration::from_micros(200);
/// A Nix file is one expression: repeated, the copies nest into one application 10 000 levels deep, which no real
/// file has. `should_highlight_a_line_of_a_large_nix_attribute_set_as_fast_as_of_a_small_one` covers Nix instead.
const NOT_REPEATABLE: [&str; 1] = ["Nix"];

macro_rules! sample {
    ($language:literal, $path:literal) => {
        ($language, include_str!(concat!("../../../../assets/previews/", $path)))
    };
}

/// A short, typical file per language, by registry name.
const SAMPLES: &[(&str, &str)] = &[
    sample!("C", "c.c"),
    sample!("C#", "csharp.cs"),
    sample!("C++", "cpp.cpp"),
    sample!("CMake", "cmake.cmake"),
    sample!("CSS", "css.css"),
    sample!("Dart", "dart.dart"),
    sample!("Diff", "diff.diff"),
    sample!("Dockerfile", "dockerfile.dockerfile"),
    sample!("Elixir", "elixir.ex"),
    sample!("Elm", "elm.elm"),
    sample!("ERB", "embedded_template.erb"),
    sample!("Erlang", "erlang.erl"),
    sample!("F#", "fsharp.fs"),
    sample!("fish", "fish.fish"),
    sample!("Gleam", "gleam.gleam"),
    sample!("GLSL", "glsl.glsl"),
    sample!("Go", "go.go"),
    sample!("go.mod", "gomod.mod"),
    sample!("GraphQL", "graphql.graphql"),
    sample!("Groovy", "groovy.groovy"),
    sample!("Haskell", "haskell.hs"),
    sample!("HCL", "hcl.tf"),
    sample!("HEEx", "heex.heex"),
    sample!("HTML", "html.html"),
    sample!("INI", "ini.ini"),
    sample!("Java", "java.java"),
    sample!("JavaScript", "javascript.js"),
    sample!("JSON", "json.json"),
    sample!("Julia", "julia.jl"),
    sample!("KDL", "kdl.kdl"),
    sample!("Kotlin", "kotlin.kt"),
    sample!("Lua", "lua.lua"),
    sample!("Makefile", "make.mk"),
    sample!("Markdown", "markdown.md"),
    sample!("Nix", "nix.nix"),
    sample!("Objective-C", "objc.m"),
    sample!("OCaml", "ocaml.ml"),
    sample!("PHP", "php.php"),
    sample!("PowerShell", "powershell.ps1"),
    sample!("Properties", "properties.properties"),
    sample!("Protocol Buffers", "proto.proto"),
    sample!("Python", "python.py"),
    sample!("R", "r.r"),
    sample!("RegExp", "regexp.regexp"),
    sample!("Ruby", "ruby.rb"),
    sample!("Rust", "rust.rs"),
    sample!("Scala", "scala.scala"),
    sample!("SCSS", "scss.scss"),
    sample!("Shell", "shell.sh"),
    sample!("Solidity", "solidity.sol"),
    sample!("SQL", "sql.sql"),
    sample!("Starlark", "starlark.bzl"),
    sample!("Svelte", "svelte.svelte"),
    sample!("Swift", "swift.swift"),
    sample!("TOML", "toml.toml"),
    sample!("TypeScript", "typescript.ts"),
    sample!("Vue", "vue.vue"),
    sample!("XML", "xml.xml"),
    sample!("YAML", "yaml.yaml"),
    sample!("Zig", "zig.zig"),
];

/// `sample` repeated to about `TARGET_LINES` lines.
fn long_file(sample: &str) -> String {
    let lines = sample.lines().count().max(1);
    sample.repeat(TARGET_LINES / lines + 1)
}

/// Byte range of line `index` of `text`.
fn line_bytes(text: &str, index: usize) -> std::ops::Range<usize> {
    let start: usize = text.split_inclusive('\n').take(index).map(str::len).sum();
    let length = text[start..].find('\n').unwrap_or(text.len() - start);
    start..start + length
}

/// Fastest of `RUNS` highlightings of line `index`; the minimum keeps scheduler noise out.
fn cost_of_line(syntax: &Syntax, text: &str, index: usize) -> Duration {
    let range = line_bytes(text, index);
    (0..RUNS)
        .map(|_| {
            let started = Instant::now();
            spans(syntax, text.as_bytes(), range.clone());
            started.elapsed()
        })
        .min()
        .unwrap_or_default()
}

/// Cost of highlighting line 20 of `short` and of `long`, both in `language`.
fn line_cost_in(language: Language, short: &str, long: &str) -> Option<(Duration, Duration)> {
    let line = 20.min(short.lines().count().saturating_sub(1));
    let costs = [short, long].map(|text| {
        let syntax = parse(text.as_bytes(), language, &Cancel::default(), Duration::from_secs(60)).ok()?;
        Some(cost_of_line(&syntax, text, line))
    });
    let [short, long] = costs;
    Some((short?, long?))
}

/// Cost of highlighting line 20 of `language`'s sample, and of the same line once the sample is grown to a long file.
fn short_and_long_cost(language: Language, sample: &str) -> Option<(Duration, Duration)> {
    line_cost_in(language, sample, &long_file(sample))
}

/// A Nix file in the shape of nixpkgs' `all-packages.nix`: one attribute set with `bindings` packages.
fn nix_attribute_set(bindings: usize) -> String {
    let body: String = (0..bindings)
        .map(|n| format!("  package{n} = pkgs.callPackage ./pkgs/p{n} {{ enable = n > 3 && true; name = \"p{n}\"; }};\n"))
        .collect();
    format!("{{ pkgs, ... }}:\n{{\n{body}}}\n")
}

#[test]
fn should_highlight_a_line_of_a_large_nix_attribute_set_as_fast_as_of_a_small_one() {
    // given
    let (small, large) = (nix_attribute_set(40), nix_attribute_set(TARGET_LINES));

    // when
    let (short, long) = line_cost_in(lang("Nix"), &small, &large).unwrap();

    // then
    assert!(long <= short * ALLOWED_FACTOR + SLACK, "40 bindings {short:?}, {TARGET_LINES} bindings {long:?}");
}

#[test]
fn should_highlight_a_line_of_a_long_file_about_as_fast_as_of_a_short_one() {
    // given
    let samples = SAMPLES.iter().filter(|(name, _)| !NOT_REPEATABLE.contains(name)).map(|&(name, sample)| (lang(name), sample));

    // when
    let growing: Vec<String> = samples
        .filter_map(|(language, sample)| short_and_long_cost(language, sample).map(|cost| (language, cost)))
        .filter(|(_, (short, long))| *long > *short * ALLOWED_FACTOR + SLACK)
        .map(|(language, (short, long))| format!("{language:?}: sample {short:?}, {TARGET_LINES}-line file {long:?}"))
        .collect();

    // then
    assert!(growing.is_empty(), "highlighting grows with file length:\n{}", growing.join("\n"));
}
