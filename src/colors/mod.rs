//! Colour keys and schemes, modelled on IntelliJ's `TextAttributesKey`s and colour schemes.

mod attributes;
mod checkout;
mod keys;
mod scheme;

pub use attributes::{Entry, RawEntry, TextAttributes};
pub use checkout::{checkout_colour, fnv1a};
pub use keys::{ColorKey, Page};
pub use scheme::{Overrides, ResolvedScheme, Scheme};

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
