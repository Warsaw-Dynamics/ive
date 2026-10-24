//! Colour keys and schemes, modelled on IntelliJ's `TextAttributesKey`s and colour schemes.

mod checkout;

pub use checkout::{checkout_colour, fnv1a, mixed, tinted};
pub use codeview::colors::{ColorKey, Overrides, RawEntry, ResolvedScheme, Scheme};

use gpui::App;

use crate::theme::{Appearance, Theme};
use crate::theme_config::ThemeSpec;

/// Rebuilds the `ResolvedScheme` global from the current appearance's scheme and the theme's syntax entries.
pub fn apply(cx: &mut App) {
    let appearance = cx.global::<Theme>().appearance;
    let resolved = resolved(cx.global::<ThemeSpec>(), appearance);
    cx.set_global(resolved);
}

/// Every key's look in `appearance`: `spec`'s syntax entries over the built-in scheme.
pub fn resolved(spec: &ThemeSpec, appearance: Appearance) -> ResolvedScheme {
    ResolvedScheme::resolve(Scheme::builtin(appearance), &overrides(spec, appearance))
}

/// `spec`'s syntax entries for `appearance`, which [`crate::theme_config::parse`] already checked.
pub fn overrides(spec: &ThemeSpec, appearance: Appearance) -> Overrides {
    let syntax = match appearance {
        Appearance::Dark => &spec.dark.syntax,
        Appearance::Light => &spec.light.syntax,
    };
    syntax.iter().filter_map(|(id, raw)| Some((ColorKey::by_id(id)?, raw.to_entry(id).ok()?))).collect()
}

#[cfg(test)]
mod tests {
    use codeview::colors::Entry;

    use super::*;

    #[test]
    fn should_resolve_theme_syntax_over_builtin_scheme() {
        // given
        let mut spec = crate::theme_config::defaults();
        spec.dark.syntax.insert("CSS.COLOR".to_owned(), RawEntry { foreground: Some("FF0000".to_owned()), ..RawEntry::default() });

        // when
        let resolved = resolved(&spec, Appearance::Dark);

        // then
        let key = |id: &str| ColorKey::by_id(id).unwrap();
        assert_eq!(resolved.get(key("CSS.COLOR")).foreground, Some(0xff0000));
        assert_eq!(resolved.get(key("CSS.IMPORTANT")).foreground, Scheme::builtin(Appearance::Dark).entry(key("CSS.IMPORTANT")).and_then(|entry| match entry {
            Entry::Own(attributes) => attributes.foreground,
            Entry::Inherit => None,
        }));
    }
}
