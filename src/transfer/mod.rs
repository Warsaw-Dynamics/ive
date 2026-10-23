//! Copying and moving files and folders between checkouts, as tree drops and pastes do it: where they go and what
//! is in the way (`plan`), reading the disk (`scan`) and writing (`write`).

pub mod plan;
pub mod scan;
pub mod write;

use gpui::{Global, Modifiers};

use crate::hub::key_label;

/// What a drop or paste does with its files.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verb {
    Copy,
    Move,
}

/// The modifiers that make a drop copy or move whatever it would do (`config.drop_modifiers` in `ive.rn`); copy also
/// makes a drop of processes start copies of them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DropModifiers {
    pub copy: Modifiers,
    pub moving: Modifiers,
}

impl Global for DropModifiers {}

impl Default for DropModifiers {
    /// `ive.rn`'s, for a window drawn before the scripts loaded.
    fn default() -> Self {
        let (copy, moving) = match cfg!(target_os = "macos") {
            true => (Modifiers { alt: true, ..Modifiers::default() }, Modifiers { platform: true, ..Modifiers::default() }),
            false => (Modifiers { control: true, ..Modifiers::default() }, Modifiers { shift: true, ..Modifiers::default() }),
        };
        Self { copy, moving }
    }
}

impl DropModifiers {
    /// The verb `held` forces: exactly the copy or the move modifiers.
    pub fn forced(&self, held: &Modifiers) -> Option<Verb> {
        match held {
            held if *held == self.copy => Some(Verb::Copy),
            held if *held == self.moving => Some(Verb::Move),
            _ => None,
        }
    }

    /// As hints name it: `Ctrl`, `Ctrl+Alt`.
    pub fn copy_name(&self) -> String {
        key_label::modifiers_label(&self.copy)
    }
}

#[cfg(test)]
mod tests {
    use gpui::Modifiers;

    use super::*;

    fn ctrl() -> Modifiers {
        Modifiers { control: true, ..Modifiers::default() }
    }

    fn shift() -> Modifiers {
        Modifiers { shift: true, ..Modifiers::default() }
    }

    #[test]
    fn should_force_verb_with_configured_modifier() {
        // given
        let configured = DropModifiers { copy: ctrl(), moving: shift() };

        // when
        let forced = [ctrl(), shift()].map(|held| configured.forced(&held));

        // then
        assert_eq!(forced, [Some(Verb::Copy), Some(Verb::Move)]);
    }

    #[test]
    fn should_force_nothing_with_other_modifiers() {
        // given
        let configured = DropModifiers { copy: ctrl(), moving: shift() };

        // when
        let forced = [Modifiers { control: true, shift: true, ..Modifiers::default() }, Modifiers::default()].map(|held| configured.forced(&held));

        // then
        assert_eq!(forced, [None, None]);
    }

    #[test]
    fn should_name_copy_modifier() {
        // given
        let configured = DropModifiers { copy: Modifiers { control: true, alt: true, ..Modifiers::default() }, moving: shift() };

        // when
        let name = configured.copy_name();

        // then
        assert_eq!(name, "Ctrl+Alt");
    }
}
