//! Calls a program makes for the user's attention: a desktop notification (iTerm2's `OSC 9;<text>`, `OSC
//! 777;notify;<title>;<body>` as Ghostty, VTE and urxvt read it, kitty's `OSC 99`), the bell, and the end of the
//! progress it reported with `OSC 9;4`.

use alacritty_terminal::vte::{Parser, Perform};

use super::progress::Progress;

/// The longest text a call keeps, in characters.
const MAX_TEXT: usize = 200;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Call {
    /// a desktop notification, its text cleaned and bounded
    Notification(String),
    Bell,
    /// busy progress went back to none
    Finished,
}

impl Call {
    /// What the tree, the tabs and a desktop notification say.
    pub fn text(&self) -> String {
        match self {
            Self::Notification(text) => text.clone(),
            Self::Bell => "rang the bell".to_owned(),
            Self::Finished => "finished".to_owned(),
        }
    }
}

/// A call and its place in the session's calls, from 1, so a reader tells a new one from one it has taken.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallReport {
    pub number: u64,
    pub call: Call,
}

/// Whether progress going from `before` to `after` means the program finished what it reported.
pub fn finished(before: Progress, after: Progress) -> bool {
    before.is_busy() && after == Progress::None
}

/// Finds calls in a program's output, also those split across reads. It runs beside the terminal's own parser,
/// which drops `OSC 9`, `777` and `99`, and tells of a bell without saying whether it was replayed.
#[derive(Default)]
pub struct CallScanner {
    parser: Parser,
    calls: LastCall,
}

impl CallScanner {
    /// The last call completed in `bytes`, if any.
    pub fn scan(&mut self, bytes: &[u8]) -> Option<Call> {
        self.parser.advance(&mut self.calls, bytes);
        self.calls.0.take()
    }
}

#[derive(Default)]
struct LastCall(Option<Call>);

impl Perform for LastCall {
    fn execute(&mut self, byte: u8) {
        if byte == 0x07 {
            self.0 = Some(Call::Bell);
        }
    }

    fn osc_dispatch(&mut self, params: &[&[u8]], _bell_terminated: bool) {
        if let Some(text) = notification_text(params) {
            self.0 = Some(Call::Notification(cleaned(&text)));
        }
    }
}

/// The raw text of a notification in OSC parameters `params`; `None` for any other OSC.
fn notification_text(params: &[&[u8]]) -> Option<String> {
    let joined = |parts: &[&[u8]]| String::from_utf8_lossy(&parts.join(&b';')).into_owned();
    match params {
        // ConEmu's commands (`9;4;…` progress, `9;9;…` directory) start with a number
        [b"9", first, ..] if is_number(first) => None,
        [b"9", rest @ ..] => Some(joined(rest)),
        [b"777", b"notify", title, body @ ..] => {
            let body = joined(body);
            if body.trim().is_empty() { Some(joined(&[title])) } else { Some(body) }
        }
        [b"99", metadata, ..] if is_base64(metadata) => None,
        [b"99", _metadata, payload @ ..] => Some(joined(payload)),
        _ => None,
    }
}

fn is_number(param: &[u8]) -> bool {
    !param.is_empty() && param.iter().all(u8::is_ascii_digit)
}

/// kitty's `e=1`: the payload is base64, which IVE does not decode.
fn is_base64(metadata: &[u8]) -> bool {
    metadata.split(|&byte| byte == b':').any(|key| key == b"e=1")
}

/// `text` on one line: control characters dropped, whitespace runs one space, trimmed, at most [`MAX_TEXT`]
/// characters; `notified` when nothing is left.
fn cleaned(text: &str) -> String {
    let printable = text.chars().map(|c| if c.is_whitespace() { ' ' } else { c }).filter(|c| !c.is_control());
    let words = printable.collect::<String>().split_whitespace().collect::<Vec<_>>().join(" ");
    if words.is_empty() {
        return "notified".to_owned();
    }
    if words.chars().count() <= MAX_TEXT {
        return words;
    }
    let mut cut: String = words.chars().take(MAX_TEXT - 1).collect();
    cut.push('…');
    cut
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scan(bytes: &[u8]) -> Option<Call> {
        CallScanner::default().scan(bytes)
    }

    fn notification(text: &str) -> Option<Call> {
        Some(Call::Notification(text.to_owned()))
    }

    #[test]
    fn should_read_iterm2_notifications() {
        // given
        let reports: [&[u8]; 3] = [b"\x1b]9;Claude needs you\x07", b"\x1b]9;Agent turn complete\x1b\\", b"\x1b]9;a;b\x07"];

        // when
        let calls = reports.map(scan);

        // then
        assert_eq!(calls, [notification("Claude needs you"), notification("Agent turn complete"), notification("a;b")]);
    }

    #[test]
    fn should_not_take_conemu_commands_for_notifications() {
        // given
        let reports: [&[u8]; 3] = [b"\x1b]9;4;1;5\x1b\\", b"\x1b]9;9;/tmp\x1b\\", b"\x1b]0;title\x1b\\"];

        // when
        let calls = reports.map(scan);

        // then
        assert_eq!(calls, [None, None, None]);
    }

    #[test]
    fn should_read_osc_777_body_else_title() {
        // given
        let reports: [&[u8]; 2] = [b"\x1b]777;notify;Claude Code;Waiting for input\x07", b"\x1b]777;notify;Build done;\x07"];

        // when
        let calls = reports.map(scan);

        // then
        assert_eq!(calls, [notification("Waiting for input"), notification("Build done")]);
    }

    #[test]
    fn should_read_kitty_payload_and_skip_base64() {
        // given
        let reports: [&[u8]; 3] = [b"\x1b]99;;Done\x1b\\", b"\x1b]99;i=1:p=body;Tests pass\x1b\\", b"\x1b]99;e=1;RG9uZQ==\x1b\\"];

        // when
        let calls = reports.map(scan);

        // then
        assert_eq!(calls, [notification("Done"), notification("Tests pass"), None]);
    }

    #[test]
    fn should_read_a_bell() {
        // when
        let call = scan(b"ab\x07c");

        // then
        assert_eq!(call, Some(Call::Bell));
    }

    #[test]
    fn should_not_take_osc_terminator_for_a_bell() {
        // when
        let call = scan(b"\x1b]0;title\x07\x1b]2;other\x07plain");

        // then
        assert_eq!(call, None);
    }

    #[test]
    fn should_find_a_call_split_across_reads() {
        // given
        let mut scanner = CallScanner::default();

        // when
        let first = scanner.scan(b"output\x1b]9;Wait");
        let second = scanner.scan(b"ing\x07more");
        let third = scanner.scan(b"more");

        // then
        assert_eq!((first, second, third), (None, notification("Waiting"), None));
    }

    #[test]
    fn should_clean_and_bound_texts() {
        // given
        let long = "x".repeat(10_000);

        // when
        let texts = [cleaned("a\x1b[1m\tb\n"), cleaned(&long), cleaned(" \n "), cleaned("zażółć  gęślą")];

        // then
        assert_eq!(texts[0], "a[1m b");
        assert_eq!(texts[1], format!("{}…", "x".repeat(199)));
        assert_eq!(texts[2], "notified");
        assert_eq!(texts[3], "zażółć gęślą");
    }

    #[test]
    fn should_finish_when_busy_progress_ends() {
        // when
        let ends = [
            finished(Progress::Normal(5), Progress::None),
            finished(Progress::Indeterminate, Progress::None),
            finished(Progress::None, Progress::None),
            finished(Progress::Normal(5), Progress::Normal(6)),
        ];

        // then
        assert_eq!(ends, [true, true, false, false]);
    }

    #[test]
    fn should_say_what_each_call_means() {
        // when
        let texts = [Call::Notification("hi".to_owned()).text(), Call::Bell.text(), Call::Finished.text()];

        // then
        assert_eq!(texts, ["hi", "rang the bell", "finished"]);
    }
}
