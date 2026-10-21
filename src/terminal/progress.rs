//! Progress a program reports with ConEmu's `OSC 9;4;<state>;<percent>`, as Windows Terminal, WezTerm and Ghostty
//! read it: https://conemu.github.io/en/AnsiEscapeCodes.html#ConEmu_specific_OSC

use alacritty_terminal::vte::{Parser, Perform};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Progress {
    #[default]
    None,
    /// percent done, 0 to 100
    Normal(u8),
    Error(u8),
    Paused(u8),
    /// busy, with no idea how far along
    Indeterminate,
}

impl Progress {
    /// As MCP reports it.
    pub fn describe(self) -> String {
        match self {
            Self::None => "none".to_owned(),
            Self::Normal(percent) => format!("{percent}%"),
            Self::Error(percent) => format!("error {percent}%"),
            Self::Paused(percent) => format!("paused {percent}%"),
            Self::Indeterminate => "indeterminate".to_owned(),
        }
    }

    /// The report in OSC parameters `9;4;<state>[;<percent>]`; `None` for any other OSC or a state ConEmu does not define.
    fn parse(params: &[&[u8]]) -> Option<Self> {
        let [b"9", b"4", state, rest @ ..] = params else {
            return None;
        };
        let percent = rest.first().and_then(|percent| std::str::from_utf8(percent).ok()?.parse::<u32>().ok()).unwrap_or(0).min(100) as u8;
        match *state {
            b"0" => Some(Self::None),
            b"1" => Some(Self::Normal(percent)),
            b"2" => Some(Self::Error(percent)),
            b"3" => Some(Self::Indeterminate),
            b"4" => Some(Self::Paused(percent)),
            _ => None,
        }
    }
}

/// Finds progress reports in a program's output, also those split across reads. It runs beside the terminal's own
/// parser, which drops `OSC 9` without telling anyone.
#[derive(Default)]
pub struct ProgressScanner {
    parser: Parser,
    reports: LastReport,
}

impl ProgressScanner {
    /// The last report completed in `bytes`, if any.
    pub fn scan(&mut self, bytes: &[u8]) -> Option<Progress> {
        self.parser.advance(&mut self.reports, bytes);
        self.reports.0.take()
    }
}

#[derive(Default)]
struct LastReport(Option<Progress>);

impl Perform for LastReport {
    fn osc_dispatch(&mut self, params: &[&[u8]], _bell_terminated: bool) {
        if let Some(progress) = Progress::parse(params) {
            self.0 = Some(progress);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_read_each_state() {
        // given
        let mut scanner = ProgressScanner::default();

        // when
        let states = ["\x1b]9;4;1;42\x07", "\x1b]9;4;2;7\x1b\\", "\x1b]9;4;4;50\x07", "\x1b]9;4;3\x07", "\x1b]9;4;0\x07"]
            .map(|report| scanner.scan(report.as_bytes()));

        // then
        assert_eq!(
            states,
            [Some(Progress::Normal(42)), Some(Progress::Error(7)), Some(Progress::Paused(50)), Some(Progress::Indeterminate), Some(Progress::None)]
        );
    }

    #[test]
    fn should_read_report_split_across_reads() {
        // given
        let mut scanner = ProgressScanner::default();

        // when
        let first = scanner.scan(b"building\x1b]9;4;");
        let second = scanner.scan(b"1;30\x07done");

        // then
        assert_eq!((first, second), (None, Some(Progress::Normal(30))));
    }

    #[test]
    fn should_keep_last_report_of_a_read() {
        // given
        let mut scanner = ProgressScanner::default();

        // when
        let progress = scanner.scan(b"\x1b]9;4;3\x07\x1b]0;title\x07\x1b]9;4;1;80\x07\x1b]9;9;/tmp\x07");

        // then
        assert_eq!(progress, Some(Progress::Normal(80)));
    }

    #[test]
    fn should_bound_percent_and_default_it_to_zero() {
        // given
        let mut scanner = ProgressScanner::default();

        // when
        let reports = ["\x1b]9;4;1;250\x07", "\x1b]9;4;1\x07", "\x1b]9;4;1;x\x07"].map(|report| scanner.scan(report.as_bytes()));

        // then
        assert_eq!(reports, [Some(Progress::Normal(100)), Some(Progress::Normal(0)), Some(Progress::Normal(0))]);
    }

    #[test]
    fn should_ignore_unknown_state_and_other_osc() {
        // given
        let mut scanner = ProgressScanner::default();

        // when
        let progress = scanner.scan(b"\x1b]9;4;7;10\x07\x1b]9;hello\x07\x1b]2;title\x07plain text");

        // then
        assert_eq!(progress, None);
    }
}
