//! The terminal modes a program set with `CSI ? Pm h` / `CSI ? Pm l`, so a replay trimmed past them can set them
//! again: a full-screen program (Codex, vim) turns on the alternate screen and mouse tracking once, at its start.

use std::collections::BTreeMap;

const ESC: u8 = 0x1b;
/// Cancel and substitute abort a sequence, as in a terminal.
const CAN: u8 = 0x18;
const SUB: u8 = 0x1a;
/// More parameters than any mode sequence needs; the rest of a longer one is ignored.
const MAX_PARAMS: usize = 16;

/// The modes kept, in the order they are set again: the screen first, so the others apply on it.
const KEPT: [u16; 15] = [
    47,   // alternate screen
    1047, // alternate screen, cleared on leaving
    1049, // alternate screen with saved cursor
    1,    // application cursor keys
    7,    // line wrap
    25,   // cursor shown
    1000, // mouse clicks reported
    1002, // mouse drags reported
    1003, // all mouse motion reported
    1004, // focus reported
    1005, // UTF-8 mouse reports
    1006, // SGR mouse reports
    1015, // urxvt mouse reports
    1007, // wheel as arrow keys on the alternate screen
    2004, // bracketed paste
];

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum State {
    #[default]
    Ground,
    Escape,
    /// after `ESC [`: a `?` makes it a private sequence
    CsiStart,
    Csi,
    /// a sequence that cannot be a mode change, skipped to its final byte
    CsiIgnore,
}

/// Watches output as it is read, across reads that split a sequence.
#[derive(Default, Clone)]
pub struct ModeTracker {
    state: State,
    params: Vec<u16>,
    modes: BTreeMap<u16, bool>,
}

impl ModeTracker {
    pub fn scan(&mut self, data: &[u8]) {
        for &byte in data {
            self.state = self.next(byte);
        }
    }

    /// Sequences that set every kept mode the program changed, as it last left it; empty when it changed none.
    pub fn sequence(&self) -> Vec<u8> {
        KEPT.iter()
            .filter_map(|mode| self.modes.get(mode).map(|&set| format!("\x1b[?{mode}{}", if set { 'h' } else { 'l' })))
            .collect::<String>()
            .into_bytes()
    }

    fn next(&mut self, byte: u8) -> State {
        match (self.state, byte) {
            (_, CAN | SUB) => State::Ground,
            (_, ESC) => State::Escape,
            (State::Ground, _) => State::Ground,
            (State::Escape, b'[') => State::CsiStart,
            // RIS, a full reset, puts every mode back to its default
            (State::Escape, b'c') => self.reset(),
            (State::Escape, _) => State::Ground,
            (State::CsiStart, b'?') => self.start_private(),
            (State::Csi, b'0'..=b'9') => self.digit(byte),
            (State::Csi, b';') => self.separate(),
            (State::Csi, b'h') => self.apply(true),
            (State::Csi, b'l') => self.apply(false),
            (State::CsiStart | State::Csi | State::CsiIgnore, 0x40..=0x7e) => State::Ground,
            (State::CsiStart | State::Csi | State::CsiIgnore, _) => State::CsiIgnore,
        }
    }

    fn reset(&mut self) -> State {
        self.modes.clear();
        State::Ground
    }

    fn start_private(&mut self) -> State {
        self.params.clear();
        self.params.push(0);
        State::Csi
    }

    fn digit(&mut self, byte: u8) -> State {
        if let Some(param) = self.params.last_mut() {
            *param = param.saturating_mul(10).saturating_add(u16::from(byte - b'0'));
        }
        State::Csi
    }

    fn separate(&mut self) -> State {
        if self.params.len() < MAX_PARAMS {
            self.params.push(0);
        }
        State::Csi
    }

    fn apply(&mut self, set: bool) -> State {
        for &mode in self.params.iter().filter(|mode| KEPT.contains(mode)) {
            self.modes.insert(mode, set);
        }
        State::Ground
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_set_again_the_modes_a_program_changed() {
        // given
        let mut tracker = ModeTracker::default();

        // when
        tracker.scan(b"\x1b[?1049h\x1b[?1000h\x1b[?1002h\x1b[?1003h\x1b[?1006h\x1b[?1007l\x1b[?2004htext");

        // then
        let expected = b"\x1b[?1049h\x1b[?1000h\x1b[?1002h\x1b[?1003h\x1b[?1006h\x1b[?1007l\x1b[?2004h";
        assert_eq!(tracker.sequence(), expected);
    }

    #[test]
    fn should_keep_the_last_change_of_each_mode() {
        // given
        let mut tracker = ModeTracker::default();

        // when
        tracker.scan(b"\x1b[?1000;1006h\x1b[?25l\x1b[?1000l\x1b[?25h");

        // then
        assert_eq!(tracker.sequence(), b"\x1b[?25h\x1b[?1000l\x1b[?1006h");
    }

    #[test]
    fn should_read_a_mode_change_split_across_reads() {
        // given
        let mut tracker = ModeTracker::default();

        // when
        tracker.scan(b"out\x1b");
        tracker.scan(b"[?10");
        tracker.scan(b"49");
        tracker.scan(b"h");

        // then
        assert_eq!(tracker.sequence(), b"\x1b[?1049h");
    }

    #[test]
    fn should_ignore_other_sequences_and_modes() {
        // given
        let mut tracker = ModeTracker::default();

        // when
        tracker.scan(b"\x1b[1000h\x1b[?2026h\x1b[?12h\x1b[4l\x1b[38;5;1m\x1b]0;?1000h\x07\x1b[?1000\x18h\x1b[?1000$h");

        // then
        assert_eq!(tracker.sequence(), b"");
    }

    #[test]
    fn should_forget_the_modes_on_a_full_reset() {
        // given
        let mut tracker = ModeTracker::default();
        tracker.scan(b"\x1b[?1049h\x1b[?1000h");

        // when
        tracker.scan(b"\x1bc\x1b[?2004h");

        // then
        assert_eq!(tracker.sequence(), b"\x1b[?2004h");
    }
}
