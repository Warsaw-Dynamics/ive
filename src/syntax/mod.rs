//! Parsing documents with tree-sitter and highlighting them with IVE's queries.

mod highlight;
mod injection;
mod language;
mod parse;
mod registry;
#[cfg(test)]
mod tests;
mod vendored;

pub use highlight::{Highlight, spans};
pub use language::{Grammar, Language};
pub use parse::{Cancel, MAX_HIGHLIGHT_BYTES, PARSE_BUDGET, ParseError, Syntax, parse};

/// The registry row named `name`; for tests, where a wrong name is a typo.
#[cfg(test)]
pub fn lang(name: &str) -> Language {
    Language::named(name).unwrap_or_else(|| panic!("no language named {name}"))
}
