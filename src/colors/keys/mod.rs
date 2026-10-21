use std::collections::HashMap;
use std::sync::LazyLock;

mod c;
mod cmake;
mod cpp;
mod csharp;
mod css;
mod dart;
mod defaults;
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
mod html;
mod ini;
mod java;
mod javascript;
mod jsdoc;
mod json;
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
mod rust;
mod scala;
mod scss;
mod shell;
mod solidity;
mod sql;
mod starlark;
mod svelte;
mod swift;
mod toml;
mod typescript;
mod vue;
mod xml;
mod yaml;
mod zig;

/// id, display name, parent id
pub type Row = (&'static str, &'static str, Option<&'static str>);

/// Every key table; `defaults::HIDDEN` comes first, so `TEXT` is key 0.
const TABLES: &[&[Row]] = &[
    defaults::HIDDEN,
    defaults::LANGUAGE_DEFAULTS,
    c::ROWS,
    csharp::ROWS,
    cpp::ROWS,
    cmake::ROWS,
    css::ROWS,
    dart::ROWS,
    diff::ROWS,
    dockerfile::ROWS,
    elixir::ROWS,
    elm::ROWS,
    embedded_template::ROWS,
    erlang::ROWS,
    fsharp::ROWS,
    fish::ROWS,
    gleam::ROWS,
    glsl::ROWS,
    go::ROWS,
    gomod::ROWS,
    graphql::ROWS,
    groovy::ROWS,
    haskell::ROWS,
    hcl::ROWS,
    heex::ROWS,
    html::ROWS,
    ini::ROWS,
    java::ROWS,
    javascript::ROWS,
    jsdoc::ROWS,
    json::ROWS,
    julia::ROWS,
    kdl::ROWS,
    kotlin::ROWS,
    lua::ROWS,
    make::ROWS,
    markdown::ROWS,
    nix::ROWS,
    objc::ROWS,
    ocaml::ROWS,
    php::ROWS,
    powershell::ROWS,
    properties::ROWS,
    proto::ROWS,
    python::ROWS,
    r::ROWS,
    regexp::ROWS,
    ruby::ROWS,
    rust::ROWS,
    scala::ROWS,
    scss::ROWS,
    shell::ROWS,
    solidity::ROWS,
    sql::ROWS,
    starlark::ROWS,
    svelte::ROWS,
    swift::ROWS,
    toml::ROWS,
    typescript::ROWS,
    vue::ROWS,
    xml::ROWS,
    yaml::ROWS,
    zig::ROWS,
];

struct KeyInfo {
    id: &'static str,
    parent: Option<&'static str>,
}

static KEYS: LazyLock<Vec<KeyInfo>> =
    LazyLock::new(|| TABLES.iter().flat_map(|rows| rows.iter()).map(|&(id, _, parent)| KeyInfo { id, parent }).collect());

static BY_ID: LazyLock<HashMap<&'static str, ColorKey>> =
    LazyLock::new(|| ColorKey::all().map(|key| (key.id(), key)).collect());


/// One colour key, e.g. `JAVA_KEYWORD`; cheap to copy and compare.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ColorKey(u16);

impl ColorKey {
    /// Plain text: the editor's default colour.
    pub const TEXT: Self = Self(0);

    pub fn by_id(id: &str) -> Option<Self> {
        BY_ID.get(id).copied()
    }

    #[cfg(test)]
    pub fn count() -> usize {
        KEYS.len()
    }

    pub fn all() -> impl Iterator<Item = Self> {
        (0..KEYS.len()).map(|index| Self(index as u16))
    }

    pub fn index(self) -> usize {
        usize::from(self.0)
    }

    pub fn id(self) -> &'static str {
        KEYS[self.index()].id
    }

    /// The key this one inherits from; the tables are checked by tests, so a missing parent is a bug.
    pub fn parent(self) -> Option<Self> {
        let parent = KEYS[self.index()].parent?;
        Some(Self::by_id(parent).unwrap_or_else(|| panic!("colour key {} names unknown parent {parent}", self.id())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::colors::Scheme;
    use crate::theme::Appearance;
    use std::collections::HashSet;

    #[test]
    fn should_give_every_key_a_unique_id() {
        // given
        let ids: Vec<&str> = ColorKey::all().map(ColorKey::id).collect();

        // when
        let unique: HashSet<&str> = ids.iter().copied().collect();

        // then
        assert_eq!(unique.len(), ids.len());
    }

    #[test]
    fn should_resolve_every_parent_to_a_key() {
        // given
        let parents: Vec<(&str, &str)> =
            KEYS.iter().filter_map(|info| info.parent.map(|parent| (info.id, parent))).collect();

        // when
        let missing: Vec<_> = parents.iter().filter(|(_, parent)| ColorKey::by_id(parent).is_none()).collect();

        // then
        assert!(missing.is_empty(), "unknown parents: {missing:?}");
    }

    #[test]
    fn should_end_every_parent_chain() {
        // given
        let longest_possible = ColorKey::count();

        // when
        let looping: Vec<&str> = ColorKey::all()
            .filter(|key| std::iter::successors(Some(*key), |key| key.parent()).nth(longest_possible).is_some())
            .map(ColorKey::id)
            .collect();

        // then
        assert!(looping.is_empty(), "parent cycles through {looping:?}");
    }

    #[test]
    fn should_start_with_plain_text() {
        // given
        let first = ColorKey::all().next();

        // when
        let id = first.map(ColorKey::id);

        // then
        assert_eq!((first, id), (Some(ColorKey::TEXT), Some("TEXT")));
    }

    #[test]
    fn should_give_every_intellij_keyed_table_a_colour_from_the_ported_dark_scheme() {
        // given
        let tables = [
            ("CSS", css::ROWS),
            ("Groovy", groovy::ROWS),
            ("HTML", html::ROWS),
            ("Kotlin", kotlin::ROWS),
            ("Markdown", markdown::ROWS),
            ("Properties", properties::ROWS),
            ("Python", python::ROWS),
            ("RegExp", regexp::ROWS),
            ("SCSS", scss::ROWS),
            ("Shell Script", shell::ROWS),
            ("XML", xml::ROWS),
        ];
        let dark = Scheme::builtin(Appearance::Dark);

        // when
        let uncoloured: Vec<&str> = tables
            .into_iter()
            .filter(|(_, rows)| rows.iter().all(|&(id, _, _)| ColorKey::by_id(id).and_then(|key| dark.entry(key)).is_none()))
            .map(|(table, _)| table)
            .collect();

        // then
        assert!(uncoloured.is_empty(), "tables without a ported colour: {uncoloured:?}");
    }
}
