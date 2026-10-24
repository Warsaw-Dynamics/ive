//! A process that called for the user (a notification, the bell, finished progress) while they did not see it, until
//! they do.

use std::time::SystemTime;

/// What a process said, and when it first called since the user last saw it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attention {
    pub text: String,
    pub since: SystemTime,
}

/// Whether a process's tab is the one its window shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sight {
    Shown,
    Hidden,
}

/// Whether the window holding a tab has the keyboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowFocus {
    Active,
    Inactive,
}

/// Whether a call counts: only one the user does not see, its tab hidden or its window in the background.
pub fn counts(sight: Sight, focus: WindowFocus) -> bool {
    (sight, focus) != (Sight::Shown, WindowFocus::Active)
}

/// The most of what a process said that a row beside its title shows, in characters.
const BRIEF: usize = 32;

/// `text` short enough to leave room for a title beside it: cut to [`BRIEF`] characters, ending in `…`.
pub fn brief(text: &str) -> String {
    if text.chars().count() <= BRIEF {
        return text.to_owned();
    }
    let mut cut: String = text.chars().take(BRIEF - 1).collect();
    cut.push('…');
    cut
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_cut_long_texts_briefly() {
        // when
        let texts = [brief("finished"), brief("Claude needs your permission to use Bash"), brief(&"ż".repeat(32))];

        // then
        assert_eq!(texts[0], "finished");
        assert_eq!(texts[1], "Claude needs your permission to…");
        assert_eq!(texts[2], "ż".repeat(32));
    }

    #[test]
    fn should_not_count_a_call_the_user_sees() {
        // when
        let counted = [
            counts(Sight::Shown, WindowFocus::Active),
            counts(Sight::Shown, WindowFocus::Inactive),
            counts(Sight::Hidden, WindowFocus::Active),
            counts(Sight::Hidden, WindowFocus::Inactive),
        ];

        // then
        assert_eq!(counted, [false, true, true, true]);
    }
}
