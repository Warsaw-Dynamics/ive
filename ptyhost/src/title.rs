//! The last window title a program set with `OSC 0` or `OSC 2`, so a replay trimmed past it can set it again.

const ESC: u8 = 0x1b;
const BEL: u8 = 0x07;
/// Cancel and substitute abort a sequence, as in a terminal.
const CAN: u8 = 0x18;
const SUB: u8 = 0x1a;
/// Longer OSC sequences (images, clipboard) are not titles and are skipped rather than buffered.
const MAX_OSC: usize = 4096;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum State {
    #[default]
    Ground,
    Escape,
    Osc,
    /// an `ESC` inside an OSC: `\` ends it, anything else aborts it
    OscEscape,
}

/// Watches output as it is read, across reads that split a sequence.
#[derive(Default)]
pub struct TitleTracker {
    state: State,
    osc: Vec<u8>,
    overflowed: bool,
    title: Option<Vec<u8>>,
}

impl TitleTracker {
    pub fn scan(&mut self, data: &[u8]) {
        for &byte in data {
            self.state = self.next(byte);
        }
    }

    /// The last title as an `OSC 2` sequence; `None` until the program sets one.
    pub fn sequence(&self) -> Option<Vec<u8>> {
        let title = self.title.as_ref()?;
        Some([b"\x1b]2;".as_slice(), title, &[BEL]].concat())
    }

    fn next(&mut self, byte: u8) -> State {
        match (self.state, byte) {
            (_, CAN | SUB) => State::Ground,
            (State::Ground, ESC) => State::Escape,
            (State::Ground, _) => State::Ground,
            (State::Escape, b']') => self.start_osc(),
            (State::Escape, ESC) => State::Escape,
            (State::Escape, _) => State::Ground,
            (State::Osc, BEL) | (State::OscEscape, b'\\') => self.end_osc(),
            (State::Osc, ESC) => State::OscEscape,
            (State::Osc, _) => self.collect(byte),
            // an ESC that is not ST aborts the OSC and starts a new sequence
            (State::OscEscape, b']') => self.start_osc(),
            (State::OscEscape, _) => State::Ground,
        }
    }

    fn start_osc(&mut self) -> State {
        self.osc.clear();
        self.overflowed = false;
        State::Osc
    }

    fn collect(&mut self, byte: u8) -> State {
        if self.osc.len() < MAX_OSC {
            self.osc.push(byte);
        } else {
            self.overflowed = true;
        }
        State::Osc
    }

    fn end_osc(&mut self) -> State {
        let title = self.osc.strip_prefix(b"0;").or_else(|| self.osc.strip_prefix(b"2;"));
        if let Some(title) = title.filter(|_| !self.overflowed) {
            self.title = Some(title.to_vec());
        }
        State::Ground
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_keep_the_last_title_set_with_osc_0_or_2() {
        // given
        let mut tracker = TitleTracker::default();

        // when
        tracker.scan(b"\x1b]0;first\x07text\x1b]2;\xe2\x9c\xb3 second\x1b\\more");

        // then
        assert_eq!(tracker.sequence(), Some(b"\x1b]2;\xe2\x9c\xb3 second\x07".to_vec()));
    }

    #[test]
    fn should_read_a_title_split_across_reads() {
        // given
        let mut tracker = TitleTracker::default();

        // when
        tracker.scan(b"out\x1b");
        tracker.scan(b"]0;spl");
        tracker.scan(b"it\x1b");
        tracker.scan(b"\\");

        // then
        assert_eq!(tracker.sequence(), Some(b"\x1b]2;split\x07".to_vec()));
    }

    #[test]
    fn should_ignore_other_osc_and_aborted_titles() {
        // given
        let mut tracker = TitleTracker::default();

        // when
        tracker.scan(b"\x1b]0;kept\x07\x1b]9;4;3\x07\x1b]8;;http://x\x07\x1b]0;cancelled\x18\x1b]2;aborted\x1b[0m");

        // then
        assert_eq!(tracker.sequence(), Some(b"\x1b]2;kept\x07".to_vec()));
    }

    #[test]
    fn should_have_no_title_until_one_is_set() {
        // given
        let mut tracker = TitleTracker::default();

        // when
        tracker.scan(b"\x1b[1mplain output\n");

        // then
        assert_eq!(tracker.sequence(), None);
    }
}
