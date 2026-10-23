//! Theme reports, Contour's mode 2031: a program sets `CSI ? 2031 h`, and the terminal then tells it of each switch
//! between dark and light with `CSI ? 997 ; 1 n` (dark) or `CSI ? 997 ; 2 n` (light), so it can query the colours
//! again. Claude Code asks for them. https://contour-terminal.org/vt-extensions/color-palette-update-notifications/

use alacritty_terminal::vte::{Params, Parser, Perform};

use crate::theme::Appearance;

const THEME_REPORTS_MODE: u16 = 2031;

/// What tells a program that asked for theme reports that the theme turned `appearance`.
pub fn theme_report(appearance: Appearance) -> &'static [u8] {
    match appearance {
        Appearance::Dark => b"\x1b[?997;1n",
        Appearance::Light => b"\x1b[?997;2n",
    }
}

/// Follows whether the program wants theme reports, also when its request is split across reads. It runs beside the
/// terminal's own parser, which ignores mode 2031.
#[derive(Default)]
pub struct ThemeReportScanner {
    parser: Parser,
    mode: LastMode,
}

impl ThemeReportScanner {
    /// Whether the program wants theme reports after the last set or reset of mode 2031 in `bytes`; `None` when
    /// `bytes` completes neither.
    pub fn scan(&mut self, bytes: &[u8]) -> Option<bool> {
        self.parser.advance(&mut self.mode, bytes);
        self.mode.0.take()
    }
}

#[derive(Default)]
struct LastMode(Option<bool>);

impl Perform for LastMode {
    fn csi_dispatch(&mut self, params: &Params, intermediates: &[u8], ignore: bool, action: char) {
        let names_mode = params.iter().any(|param| param == [THEME_REPORTS_MODE]);
        if ignore || intermediates != b"?" || !names_mode {
            return;
        }
        match action {
            'h' => self.0 = Some(true),
            'l' => self.0 = Some(false),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_follow_set_and_reset() {
        // given
        let mut scanner = ThemeReportScanner::default();

        // when
        let modes = ["\x1b[?2031h", "text\x1b[?2031l", "\x1b[?2004;2031;1004h"].map(|bytes| scanner.scan(bytes.as_bytes()));

        // then
        assert_eq!(modes, [Some(true), Some(false), Some(true)]);
    }

    #[test]
    fn should_read_request_split_across_reads() {
        // given
        let mut scanner = ThemeReportScanner::default();

        // when
        let first = scanner.scan(b"prompt\x1b[?20");
        let second = scanner.scan(b"31h");

        // then
        assert_eq!((first, second), (None, Some(true)));
    }

    #[test]
    fn should_keep_last_change_of_a_read() {
        // given
        let mut scanner = ThemeReportScanner::default();

        // when
        let mode = scanner.scan(b"\x1b[?2031h\x1b[?2031l");

        // then
        assert_eq!(mode, Some(false));
    }

    #[test]
    fn should_ignore_other_modes_and_public_mode_2031() {
        // given
        let mut scanner = ThemeReportScanner::default();

        // when
        let mode = scanner.scan(b"\x1b[?2004h\x1b[2031h\x1b[?997;1n\x1b[?1049l");

        // then
        assert_eq!(mode, None);
    }

    #[test]
    fn should_report_dark_as_1_and_light_as_2() {
        // when
        let reports = [Appearance::Dark, Appearance::Light].map(theme_report);

        // then
        assert_eq!(reports, [b"\x1b[?997;1n".as_slice(), b"\x1b[?997;2n".as_slice()]);
    }
}
