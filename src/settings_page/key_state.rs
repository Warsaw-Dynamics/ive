use gpui::App;

use crate::colors::{self, ColorKey, Entry, Overrides, ResolvedScheme, Scheme, TextAttributes};
use crate::theme::Appearance;
use crate::theme_config::ThemeSpec;

/// What the attributes panel shows for one key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyState {
    pub key: ColorKey,
    /// the theme's entry, if any
    pub user: Option<Entry>,
    /// the scheme's entry, if any
    pub scheme: Option<Entry>,
    /// how the key looks now
    pub shown: TextAttributes,
}

impl KeyState {
    pub fn new(key: ColorKey, scheme: &Scheme, overrides: &Overrides, resolved: &ResolvedScheme) -> Self {
        Self { key, user: overrides.get(&key).copied(), scheme: scheme.entry(key), shown: resolved.get(key) }
    }

    /// The state of `key` under the current theme and `appearance`'s scheme.
    pub fn of(key: ColorKey, appearance: Appearance, cx: &App) -> Self {
        let overrides = colors::overrides(cx.global::<ThemeSpec>(), appearance);
        Self::new(key, Scheme::builtin(appearance), &overrides, cx.global::<ResolvedScheme>())
    }

    /// Whether *Inherit values from* is ticked: the key has a parent and no own attributes in effect.
    pub fn inherits(&self) -> bool {
        self.key.parent().is_some() && matches!(self.user.or(self.scheme), Some(Entry::Inherit) | None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(id: &str) -> ColorKey {
        ColorKey::by_id(id).unwrap()
    }

    #[test]
    fn should_never_inherit_key_without_parent() {
        // given
        let default_key = KeyState { key: key("DEFAULT_KEYWORD"), user: None, scheme: None, shown: TextAttributes::default() };

        // when
        let inherits = default_key.inherits();

        // then
        assert!(!inherits);
    }
}
