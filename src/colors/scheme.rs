use std::collections::{BTreeMap, HashMap};
use std::sync::LazyLock;

use gpui::Global;

use super::{ColorKey, Entry, RawEntry, TextAttributes};
use crate::theme::Appearance;

const DARK: &str = include_str!("../../assets/schemes/dark.toml");
const LIGHT: &str = include_str!("../../assets/schemes/light.toml");

static DARK_SCHEME: LazyLock<Scheme> = LazyLock::new(|| Scheme::parse(DARK, "dark").expect("embedded dark scheme is valid"));
static LIGHT_SCHEME: LazyLock<Scheme> =
    LazyLock::new(|| Scheme::parse(LIGHT, "light").expect("embedded light scheme is valid"));

/// A colour scheme: the entries it sets, by key.
pub struct Scheme {
    entries: HashMap<ColorKey, Entry>,
}

/// The user's entries for one scheme, from `settings.toml`.
pub type Overrides = HashMap<ColorKey, Entry>;

impl Scheme {
    /// The built-in scheme of a theme, ported from IntelliJ's New UI schemes.
    pub fn builtin(appearance: Appearance) -> &'static Self {
        match appearance {
            Appearance::Dark => &DARK_SCHEME,
            Appearance::Light => &LIGHT_SCHEME,
        }
    }

    /// A scheme file: one table per key id. Unknown ids are errors here, since IVE generates these files.
    pub fn parse(text: &str, name: &str) -> Result<Self, String> {
        let raw: BTreeMap<String, RawEntry> =
            toml::from_str(text).map_err(|error| format!("scheme {name}: {}", error.message()))?;
        let entries = raw
            .iter()
            .map(|(id, entry)| {
                let key = ColorKey::by_id(id).ok_or_else(|| format!("scheme {name} sets unknown key {id}"))?;
                Ok((key, entry.to_entry(&format!("{name}.\"{id}\""))?))
            })
            .collect::<Result<_, String>>()?;
        Ok(Self { entries })
    }

    pub fn entry(&self, key: ColorKey) -> Option<Entry> {
        self.entries.get(&key).copied()
    }
}

/// Every key's look under a scheme and the user's overrides; what rendering looks up.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedScheme {
    attributes: Vec<TextAttributes>,
}

impl Global for ResolvedScheme {}

impl ResolvedScheme {
    pub fn resolve(scheme: &Scheme, overrides: &Overrides) -> Self {
        Self { attributes: ColorKey::all().map(|key| resolve_key(key, scheme, overrides)).collect() }
    }

    pub fn get(&self, key: ColorKey) -> TextAttributes {
        self.attributes[key.index()]
    }
}

/// User override, then scheme entry, then the parent resolved the same way, then plain text.
fn resolve_key(key: ColorKey, scheme: &Scheme, overrides: &Overrides) -> TextAttributes {
    match overrides.get(&key).copied().or_else(|| scheme.entry(key)) {
        Some(Entry::Own(attributes)) => attributes,
        Some(Entry::Inherit) | None => {
            key.parent().map_or_else(TextAttributes::default, |parent| resolve_key(parent, scheme, overrides))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(id: &str) -> ColorKey {
        ColorKey::by_id(id).unwrap()
    }

    fn colored(color: u32) -> Entry {
        Entry::Own(TextAttributes { foreground: Some(color), ..TextAttributes::default() })
    }

    #[test]
    fn should_resolve_theme_syntax_over_builtin_scheme() {
        // given
        let mut spec = crate::theme_config::defaults();
        spec.dark.syntax.insert("CSS.COLOR".to_owned(), RawEntry { foreground: Some("FF0000".to_owned()), ..RawEntry::default() });

        // when
        let resolved = super::super::resolved(&spec, Appearance::Dark);

        // then
        assert_eq!(resolved.get(key("CSS.COLOR")).foreground, Some(0xff0000));
        assert_eq!(resolved.get(key("CSS.IMPORTANT")).foreground, Scheme::builtin(Appearance::Dark).entry(key("CSS.IMPORTANT")).and_then(|entry| match entry {
            Entry::Own(attributes) => attributes.foreground,
            Entry::Inherit => None,
        }));
    }

    fn scheme(text: &str) -> Scheme {
        Scheme::parse(text, "test").unwrap()
    }

    #[test]
    fn should_parse_both_builtin_schemes_with_intellij_keyword_colours() {
        // given
        let keyword = key("DEFAULT_KEYWORD");

        // when
        let foreground = |scheme: &Scheme| ResolvedScheme::resolve(scheme, &Overrides::new()).get(keyword).foreground;
        let colours = (
            foreground(&Scheme::parse(DARK, "dark").unwrap()),
            foreground(&Scheme::parse(LIGHT, "light").unwrap()),
        );

        // then
        assert_eq!(colours, (Some(0xCF8E6D), Some(0x0033B3)));
    }

    #[test]
    fn should_let_override_beat_scheme() {
        // given
        let scheme = scheme("[JAVA_KEYWORD]\nforeground = \"111111\"\n");
        let overrides = Overrides::from([(key("JAVA_KEYWORD"), colored(0x222222))]);

        // when
        let resolved = ResolvedScheme::resolve(&scheme, &overrides);

        // then
        assert_eq!(resolved.get(key("JAVA_KEYWORD")).foreground, Some(0x222222));
    }

    #[test]
    fn should_let_scheme_entry_beat_parent() {
        // given
        let scheme = scheme("[DEFAULT_KEYWORD]\nforeground = \"111111\"\n[JAVA_KEYWORD]\nforeground = \"333333\"\n");

        // when
        let resolved = ResolvedScheme::resolve(&scheme, &Overrides::new());

        // then
        assert_eq!(resolved.get(key("JAVA_KEYWORD")).foreground, Some(0x333333));
    }

    #[test]
    fn should_let_inherit_override_beat_scheme_entry() {
        // given
        let scheme = scheme("[DEFAULT_KEYWORD]\nforeground = \"111111\"\n[JAVA_KEYWORD]\nforeground = \"333333\"\n");
        let overrides = Overrides::from([(key("JAVA_KEYWORD"), Entry::Inherit)]);

        // when
        let resolved = ResolvedScheme::resolve(&scheme, &overrides);

        // then
        assert_eq!(resolved.get(key("JAVA_KEYWORD")).foreground, Some(0x111111));
    }

    #[test]
    fn should_resolve_three_level_chain() {
        // given
        let scheme = scheme("[DEFAULT_STATIC_FIELD]\nforeground = \"C77DBB\"\nitalic = true\n");

        // when
        let resolved = ResolvedScheme::resolve(&scheme, &Overrides::new());

        // then
        let expected = TextAttributes { foreground: Some(0xC77DBB), italic: true, ..TextAttributes::default() };
        assert_eq!(resolved.get(key("STATIC_FINAL_FIELD_ATTRIBUTES")), expected);
    }

    #[test]
    fn should_fall_back_to_plain_text_without_any_entry() {
        // given
        let scheme = scheme("");

        // when
        let resolved = ResolvedScheme::resolve(&scheme, &Overrides::new());

        // then
        assert_eq!(resolved.get(key("JSON.PROPERTY_KEY")), TextAttributes::default());
    }

    #[test]
    fn should_reject_scheme_with_unknown_key() {
        // given
        let text = "[NOT_A_KEY]\nbold = true\n";

        // when
        let error = Scheme::parse(text, "test").err();

        // then
        assert_eq!(error.as_deref(), Some("scheme test sets unknown key NOT_A_KEY"));
    }
}
