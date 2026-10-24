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

/// Which of IVE's windows has the keyboard, as their activation observers tell it. gpui's own `active_window` is
/// not set yet while those observers run on X11.
#[derive(Debug)]
pub struct ActiveWindow<H>(Option<H>);

impl<H> Default for ActiveWindow<H> {
    fn default() -> Self {
        Self(None)
    }
}

impl<H: Copy + PartialEq> ActiveWindow<H> {
    /// Window `handle` became active or stopped being so; a late word of another window going changes nothing.
    pub fn observe(&mut self, handle: H, focus: WindowFocus) {
        match focus {
            WindowFocus::Active => self.0 = Some(handle),
            WindowFocus::Inactive if self.0 == Some(handle) => self.0 = None,
            WindowFocus::Inactive => {}
        }
    }

    pub fn focus_of(&self, handle: H) -> WindowFocus {
        if self.0 == Some(handle) { WindowFocus::Active } else { WindowFocus::Inactive }
    }

    /// Whether any IVE window has the keyboard.
    pub fn is_any(&self) -> bool {
        self.0.is_some()
    }
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
    fn should_follow_the_window_that_became_active() {
        // given
        let mut active = ActiveWindow::default();

        // when
        active.observe("main", WindowFocus::Active);
        active.observe("detached", WindowFocus::Active);
        // X11 may tell of the first window going after the second came
        active.observe("main", WindowFocus::Inactive);

        // then
        assert_eq!((active.focus_of("detached"), active.focus_of("main")), (WindowFocus::Active, WindowFocus::Inactive));
    }

    #[test]
    fn should_have_no_active_window_once_it_goes() {
        // given
        let mut active = ActiveWindow::default();
        active.observe("main", WindowFocus::Active);

        // when
        active.observe("main", WindowFocus::Inactive);

        // then
        assert_eq!((active.focus_of("main"), active.is_any()), (WindowFocus::Inactive, false));
    }

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
