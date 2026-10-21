//! xterm mouse reporting: what a program that turned on mouse tracking (mc, Claude Code, vim) receives
//! instead of IVE selecting text or scrolling history.

use alacritty_terminal::term::TermMode;
use gpui::Modifiers;

/// Legacy reports carry each value as one byte offset by 32, so coordinates past 223 cannot be sent.
const LEGACY_MAX: usize = 255 - 32;
/// UTF-8 reports carry each value as a character of at most two bytes, offset by 32.
const UTF8_MAX: usize = 0x7ff - 32;

/// A mouse button as the program sees it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Button {
    Left,
    Middle,
    Right,
    WheelUp,
    WheelDown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Press,
    Release,
    /// with the button held, or none
    Motion,
}

/// A cell of the visible screen, counted from zero at the top-left.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenCell {
    pub column: usize,
    pub line: usize,
}

/// A mouse gesture on a screen cell, as an agent sends it over MCP.
#[derive(Debug, Clone, Copy)]
pub struct Gesture {
    pub kind: GestureKind,
    pub cell: ScreenCell,
    pub modifiers: Modifiers,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GestureKind {
    /// press and release; 2 clicks select a word, 3 a line
    Click { button: Button, clicks: usize },
    Press(Button),
    Release(Button),
    /// a drag while a button is pressed, else a hover
    Move,
    /// lines, positive is up
    Scroll(i32),
}

#[derive(Debug, Clone, Copy)]
pub struct MouseEvent {
    pub action: Action,
    pub button: Option<Button>,
    pub cell: ScreenCell,
    pub modifiers: Modifiers,
}

/// Whether the mouse goes to the program: it asked for it, and Shift is not held — Shift keeps
/// selection and scrollback working, as in xterm.
pub fn is_reported(mode: TermMode, modifiers: &Modifiers) -> bool {
    mode.intersects(TermMode::MOUSE_MODE) && !modifiers.shift
}

/// The report for `event`, or `None` when the program's tracking mode does not want it.
pub fn report(event: &MouseEvent, mode: TermMode) -> Option<Vec<u8>> {
    if !wants(event, mode) {
        return None;
    }
    let code = button_code(event) + modifier_code(&event.modifiers);
    let ScreenCell { column, line } = event.cell;
    if mode.contains(TermMode::SGR_MOUSE) {
        let end = if event.action == Action::Release { 'm' } else { 'M' };
        return Some(format!("\x1b[<{code};{};{}{end}", column + 1, line + 1).into_bytes());
    }
    // legacy releases do not say which button; the code is 3
    let code = if event.action == Action::Release { 3 + modifier_code(&event.modifiers) } else { code };
    if mode.contains(TermMode::UTF8_MOUSE) {
        return utf8_report(code, column + 1, line + 1);
    }
    legacy_report(code, column + 1, line + 1)
}

fn wants(event: &MouseEvent, mode: TermMode) -> bool {
    let wheel = matches!(event.button, Some(Button::WheelUp | Button::WheelDown));
    match event.action {
        Action::Press => mode.intersects(TermMode::MOUSE_MODE),
        Action::Release => !wheel && mode.intersects(TermMode::MOUSE_MODE),
        Action::Motion if event.button.is_some() => mode.intersects(TermMode::MOUSE_DRAG | TermMode::MOUSE_MOTION),
        Action::Motion => mode.contains(TermMode::MOUSE_MOTION),
    }
}

/// xterm button numbers; motion adds 32, and motion with no button held reports button 3.
fn button_code(event: &MouseEvent) -> usize {
    let button = match event.button {
        Some(Button::Left) => 0,
        Some(Button::Middle) => 1,
        Some(Button::Right) => 2,
        None => 3,
        Some(Button::WheelUp) => 64,
        Some(Button::WheelDown) => 65,
    };
    if event.action == Action::Motion { button + 32 } else { button }
}

fn modifier_code(modifiers: &Modifiers) -> usize {
    4 * usize::from(modifiers.shift) + 8 * usize::from(modifiers.alt) + 16 * usize::from(modifiers.control)
}

fn legacy_report(code: usize, column: usize, line: usize) -> Option<Vec<u8>> {
    if column > LEGACY_MAX || line > LEGACY_MAX {
        return None;
    }
    let byte = |value: usize| (value + 32) as u8;
    Some(vec![0x1b, b'[', b'M', byte(code), byte(column), byte(line)])
}

fn utf8_report(code: usize, column: usize, line: usize) -> Option<Vec<u8>> {
    if column > UTF8_MAX || line > UTF8_MAX {
        return None;
    }
    let mut report = String::from("\x1b[M");
    for value in [code, column, line] {
        report.push(char::from_u32((value + 32) as u32)?);
    }
    Some(report.into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(action: Action, button: Option<Button>, column: usize, line: usize) -> MouseEvent {
        MouseEvent { action, button, cell: ScreenCell { column, line }, modifiers: Modifiers::default() }
    }

    fn sent(event: &MouseEvent, mode: TermMode) -> Option<String> {
        report(event, mode).map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
    }

    #[test]
    fn should_report_sgr_press_and_release_with_one_based_cells() {
        // given
        let mode = TermMode::MOUSE_REPORT_CLICK | TermMode::SGR_MOUSE;

        // when
        let press = sent(&at(Action::Press, Some(Button::Left), 4, 9), mode);
        let release = sent(&at(Action::Release, Some(Button::Right), 4, 9), mode);

        // then
        assert_eq!((press.as_deref(), release.as_deref()), (Some("\x1b[<0;5;10M"), Some("\x1b[<2;5;10m")));
    }

    #[test]
    fn should_report_legacy_bytes_with_release_as_button_three() {
        // given
        let mode = TermMode::MOUSE_REPORT_CLICK;

        // when
        let press = report(&at(Action::Press, Some(Button::Middle), 0, 0), mode);
        let release = report(&at(Action::Release, Some(Button::Middle), 0, 0), mode);

        // then
        assert_eq!(press, Some(vec![0x1b, b'[', b'M', 32 + 1, 33, 33]));
        assert_eq!(release, Some(vec![0x1b, b'[', b'M', 32 + 3, 33, 33]));
    }

    #[test]
    fn should_add_modifier_bits_to_the_button_code() {
        // given
        let mut event = at(Action::Press, Some(Button::Left), 0, 0);
        event.modifiers = Modifiers { control: true, alt: true, ..Modifiers::default() };

        // when
        let report = sent(&event, TermMode::MOUSE_REPORT_CLICK | TermMode::SGR_MOUSE);

        // then
        assert_eq!(report.as_deref(), Some("\x1b[<24;1;1M"));
    }

    #[test]
    fn should_report_motion_only_in_the_modes_that_ask_for_it() {
        // given
        let drag = at(Action::Motion, Some(Button::Left), 2, 3);
        let hover = at(Action::Motion, None, 2, 3);
        let sgr = TermMode::SGR_MOUSE;

        // when
        let reports = [
            sent(&drag, sgr | TermMode::MOUSE_REPORT_CLICK),
            sent(&drag, sgr | TermMode::MOUSE_DRAG),
            sent(&hover, sgr | TermMode::MOUSE_DRAG),
            sent(&hover, sgr | TermMode::MOUSE_MOTION),
        ];

        // then
        assert_eq!(reports, [None, Some("\x1b[<32;3;4M".to_owned()), None, Some("\x1b[<35;3;4M".to_owned())]);
    }

    #[test]
    fn should_report_wheel_as_presses_without_release() {
        // given
        let mode = TermMode::MOUSE_REPORT_CLICK | TermMode::SGR_MOUSE;

        // when
        let up = sent(&at(Action::Press, Some(Button::WheelUp), 0, 0), mode);
        let down = sent(&at(Action::Press, Some(Button::WheelDown), 0, 0), mode);
        let release = sent(&at(Action::Release, Some(Button::WheelUp), 0, 0), mode);

        // then
        assert_eq!([up, down, release], [Some("\x1b[<64;1;1M".to_owned()), Some("\x1b[<65;1;1M".to_owned()), None]);
    }

    #[test]
    fn should_encode_large_cells_only_where_the_encoding_can() {
        // given
        let far = at(Action::Press, Some(Button::Left), 300, 0);

        // when
        let legacy = report(&far, TermMode::MOUSE_REPORT_CLICK);
        let utf8 = sent(&far, TermMode::MOUSE_REPORT_CLICK | TermMode::UTF8_MOUSE);

        // then
        assert_eq!(legacy, None);
        assert_eq!(utf8, Some(format!("\x1b[M {}!", char::from_u32(301 + 32).expect("valid char"))));
    }

    #[test]
    fn should_leave_the_mouse_to_ive_without_tracking_or_with_shift() {
        // given
        let shift = Modifiers { shift: true, ..Modifiers::default() };

        // when
        let decisions = [
            is_reported(TermMode::empty(), &Modifiers::default()),
            is_reported(TermMode::MOUSE_DRAG, &shift),
            is_reported(TermMode::MOUSE_DRAG, &Modifiers::default()),
        ];

        // then
        assert_eq!(decisions, [false, false, true]);
    }
}
