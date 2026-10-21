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

/// A Settings page and the key tables it lists.
struct PageSpec {
    title: &'static str,
    rows: &'static [&'static [Row]],
}

/// `General` comes first, so `TEXT` is key 0. Titles are unique; the registry names its pages by them.
const PAGES: &[PageSpec] = &[
    PageSpec { title: "General", rows: &[defaults::HIDDEN] },
    PageSpec { title: "Language Defaults", rows: &[defaults::LANGUAGE_DEFAULTS] },
    PageSpec { title: "C", rows: &[c::ROWS] },
    PageSpec { title: "C#", rows: &[csharp::ROWS] },
    PageSpec { title: "C++", rows: &[cpp::ROWS] },
    PageSpec { title: "CMake", rows: &[cmake::ROWS] },
    PageSpec { title: "CSS", rows: &[css::ROWS] },
    PageSpec { title: "Dart", rows: &[dart::ROWS] },
    PageSpec { title: "Diff", rows: &[diff::ROWS] },
    PageSpec { title: "Dockerfile", rows: &[dockerfile::ROWS] },
    PageSpec { title: "Elixir", rows: &[elixir::ROWS] },
    PageSpec { title: "Elm", rows: &[elm::ROWS] },
    PageSpec { title: "ERB and EJS", rows: &[embedded_template::ROWS] },
    PageSpec { title: "Erlang", rows: &[erlang::ROWS] },
    PageSpec { title: "F#", rows: &[fsharp::ROWS] },
    PageSpec { title: "fish", rows: &[fish::ROWS] },
    PageSpec { title: "Gleam", rows: &[gleam::ROWS] },
    PageSpec { title: "GLSL", rows: &[glsl::ROWS] },
    PageSpec { title: "Go", rows: &[go::ROWS] },
    PageSpec { title: "go.mod", rows: &[gomod::ROWS] },
    PageSpec { title: "GraphQL", rows: &[graphql::ROWS] },
    PageSpec { title: "Groovy", rows: &[groovy::ROWS] },
    PageSpec { title: "Haskell", rows: &[haskell::ROWS] },
    PageSpec { title: "HCL", rows: &[hcl::ROWS] },
    PageSpec { title: "HEEx", rows: &[heex::ROWS] },
    PageSpec { title: "HTML", rows: &[html::ROWS] },
    PageSpec { title: "INI", rows: &[ini::ROWS] },
    PageSpec { title: "Java", rows: &[java::ROWS] },
    PageSpec { title: "JavaScript", rows: &[javascript::ROWS, jsdoc::ROWS] },
    PageSpec { title: "JSON", rows: &[json::ROWS] },
    PageSpec { title: "Julia", rows: &[julia::ROWS] },
    PageSpec { title: "KDL", rows: &[kdl::ROWS] },
    PageSpec { title: "Kotlin", rows: &[kotlin::ROWS] },
    PageSpec { title: "Lua", rows: &[lua::ROWS] },
    PageSpec { title: "Makefile", rows: &[make::ROWS] },
    PageSpec { title: "Markdown", rows: &[markdown::ROWS] },
    PageSpec { title: "Nix", rows: &[nix::ROWS] },
    PageSpec { title: "Objective-C", rows: &[objc::ROWS] },
    PageSpec { title: "OCaml", rows: &[ocaml::ROWS] },
    PageSpec { title: "PHP", rows: &[php::ROWS] },
    PageSpec { title: "PowerShell", rows: &[powershell::ROWS] },
    PageSpec { title: "Properties", rows: &[properties::ROWS] },
    PageSpec { title: "Protocol Buffers", rows: &[proto::ROWS] },
    PageSpec { title: "Python", rows: &[python::ROWS] },
    PageSpec { title: "R", rows: &[r::ROWS] },
    PageSpec { title: "RegExp", rows: &[regexp::ROWS] },
    PageSpec { title: "Ruby", rows: &[ruby::ROWS] },
    PageSpec { title: "Rust", rows: &[rust::ROWS] },
    PageSpec { title: "Scala", rows: &[scala::ROWS] },
    PageSpec { title: "SCSS", rows: &[scss::ROWS] },
    PageSpec { title: "Shell Script", rows: &[shell::ROWS] },
    PageSpec { title: "Solidity", rows: &[solidity::ROWS] },
    PageSpec { title: "SQL", rows: &[sql::ROWS] },
    PageSpec { title: "Starlark", rows: &[starlark::ROWS] },
    PageSpec { title: "Svelte", rows: &[svelte::ROWS] },
    PageSpec { title: "Swift", rows: &[swift::ROWS] },
    PageSpec { title: "TOML", rows: &[toml::ROWS] },
    PageSpec { title: "TypeScript", rows: &[typescript::ROWS] },
    PageSpec { title: "Vue", rows: &[vue::ROWS] },
    PageSpec { title: "XML", rows: &[xml::ROWS] },
    PageSpec { title: "YAML", rows: &[yaml::ROWS] },
    PageSpec { title: "Zig", rows: &[zig::ROWS] },
];

/// The Settings page that lists a key; keys of the first page, `General`, are resolved like the others but listed
/// nowhere.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Page(u8);

static LISTED: LazyLock<Vec<Page>> = LazyLock::new(|| {
    let mut languages: Vec<Page> = (2..PAGES.len()).map(|index| Page(index as u8)).collect();
    languages.sort_by_key(|page| page.title().to_lowercase());
    std::iter::once(Page::LANGUAGE_DEFAULTS).chain(languages).collect()
});

impl Page {
    pub const LANGUAGE_DEFAULTS: Self = Self(1);

    /// In the order the Settings navigation shows them: Language Defaults, then the languages alphabetically.
    pub fn listed() -> &'static [Self] {
        &LISTED
    }

    pub fn titled(title: &str) -> Option<Self> {
        PAGES.iter().position(|page| page.title == title).map(|index| Self(index as u8))
    }

    pub fn title(self) -> &'static str {
        PAGES[usize::from(self.0)].title
    }
}

struct KeyInfo {
    id: &'static str,
    name: &'static str,
    parent: Option<&'static str>,
    page: Page,
}

static KEYS: LazyLock<Vec<KeyInfo>> = LazyLock::new(|| {
    PAGES
        .iter()
        .enumerate()
        .flat_map(|(index, spec)| {
            let page = Page(index as u8);
            spec.rows.iter().flat_map(|rows| rows.iter()).map(move |&(id, name, parent)| KeyInfo { id, name, parent, page })
        })
        .collect()
});

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

    /// The keys `page` lists, sorted by display name as IntelliJ sorts them.
    pub fn on_page(page: Page) -> Vec<Self> {
        let mut keys: Vec<Self> = Self::all().filter(|key| key.page() == page).collect();
        keys.sort_by_key(|key| key.name());
        keys
    }

    pub fn index(self) -> usize {
        usize::from(self.0)
    }

    pub fn id(self) -> &'static str {
        KEYS[self.index()].id
    }

    pub fn name(self) -> &'static str {
        KEYS[self.index()].name
    }

    pub fn page(self) -> Page {
        KEYS[self.index()].page
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
    fn should_list_page_keys_sorted_by_name() {
        // given
        let page = Page::titled("JSON").unwrap();

        // when
        let names: Vec<&str> = ColorKey::on_page(page).into_iter().map(ColorKey::name).collect();

        // then
        assert_eq!(names.first(), Some(&"Block comment"));
        assert!(names.is_sorted(), "{names:?}");
        assert_eq!(names.len(), json::ROWS.len());
    }
}
