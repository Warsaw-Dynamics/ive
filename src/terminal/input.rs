use alacritty_terminal::term::TermMode;
use gpui::{Keystroke, Modifiers};

const ESC: u8 = 0x1b;

/// What a key press in the terminal does.
#[derive(Debug, PartialEq, Eq)]
pub enum KeyAction {
    Send(Vec<u8>),
    /// left to IVE's own key bindings
    Unhandled,
}

/// Maps a key press to what the shell receives, xterm style; `mode` carries the program's cursor-key mode.
pub fn key_action(keystroke: &Keystroke, mode: TermMode) -> KeyAction {
    if keystroke.modifiers.platform {
        return KeyAction::Unhandled;
    }
    encode(keystroke, mode).map_or(KeyAction::Unhandled, KeyAction::Send)
}

fn encode(keystroke: &Keystroke, mode: TermMode) -> Option<Vec<u8>> {
    let modifiers = &keystroke.modifiers;
    let key = keystroke.key.as_str();
    if let Some(sequence) = named_key(key, modifiers, mode) {
        return Some(sequence);
    }
    if modifiers.control {
        // Windows reports AltGr as Ctrl+Alt: a character typed with it (ą, €) goes as it is
        if modifiers.alt && let Some(text) = keystroke.key_char.as_deref() {
            return Some(text.as_bytes().to_vec());
        }
        return control_byte(key).map(|byte| alt_prefixed(modifiers.alt, &[byte]));
    }
    // with Alt held some platforms report no typed character; the bare key is the closest one
    let text = keystroke.key_char.as_deref().or((key.chars().count() == 1).then_some(key))?;
    Some(alt_prefixed(modifiers.alt, text.as_bytes()))
}

fn named_key(key: &str, modifiers: &Modifiers, mode: TermMode) -> Option<Vec<u8>> {
    let alt = modifiers.alt;
    match key {
        "enter" => Some(alt_prefixed(alt, b"\r")),
        "escape" => Some(alt_prefixed(alt, &[ESC])),
        "tab" if modifiers.shift => Some(b"\x1b[Z".to_vec()),
        "tab" => Some(alt_prefixed(alt, b"\t")),
        "backspace" if modifiers.control => Some(alt_prefixed(alt, &[0x08])),
        "backspace" => Some(alt_prefixed(alt, &[0x7f])),
        _ => letter_sequence(key, modifiers, mode).or_else(|| tilde_sequence(key, modifiers)),
    }
}

/// Keys sent as `ESC [ 1 ; m X` when modified, otherwise `ESC [ X`, or `ESC O X` for F1–F4 and in
/// application cursor mode.
fn letter_sequence(key: &str, modifiers: &Modifiers, mode: TermMode) -> Option<Vec<u8>> {
    let (letter, always_ss3) = match key {
        "up" => ('A', false),
        "down" => ('B', false),
        "right" => ('C', false),
        "left" => ('D', false),
        "home" => ('H', false),
        "end" => ('F', false),
        "f1" => ('P', true),
        "f2" => ('Q', true),
        "f3" => ('R', true),
        "f4" => ('S', true),
        _ => return None,
    };
    let code = modifier_code(modifiers);
    let sequence = match code {
        1 if always_ss3 || mode.contains(TermMode::APP_CURSOR) => format!("\x1bO{letter}"),
        1 => format!("\x1b[{letter}"),
        _ => format!("\x1b[1;{code}{letter}"),
    };
    Some(sequence.into_bytes())
}

/// Keys sent as `ESC [ n ~`, or `ESC [ n ; m ~` when modified.
fn tilde_sequence(key: &str, modifiers: &Modifiers) -> Option<Vec<u8>> {
    let number = match key {
        "insert" => 2,
        "delete" => 3,
        "pageup" => 5,
        "pagedown" => 6,
        "f5" => 15,
        "f6" => 17,
        "f7" => 18,
        "f8" => 19,
        "f9" => 20,
        "f10" => 21,
        "f11" => 23,
        "f12" => 24,
        _ => return None,
    };
    let sequence = match modifier_code(modifiers) {
        1 => format!("\x1b[{number}~"),
        code => format!("\x1b[{number};{code}~"),
    };
    Some(sequence.into_bytes())
}

/// The xterm modifier parameter: 1, plus 1 for Shift, 2 for Alt and 4 for Ctrl.
fn modifier_code(modifiers: &Modifiers) -> u8 {
    1 + u8::from(modifiers.shift) + 2 * u8::from(modifiers.alt) + 4 * u8::from(modifiers.control)
}

/// The C0 control byte Ctrl+`key` sends, as in xterm.
fn control_byte(key: &str) -> Option<u8> {
    if key == "space" {
        return Some(0);
    }
    let &[byte] = key.as_bytes() else {
        return None;
    };
    match byte.to_ascii_lowercase() {
        letter @ b'a'..=b'z' => Some(letter - b'a' + 1),
        b'@' | b'2' => Some(0),
        b'[' | b'3' => Some(ESC),
        b'\\' | b'4' => Some(0x1c),
        b']' | b'5' => Some(0x1d),
        b'^' | b'6' => Some(0x1e),
        b'_' | b'-' | b'/' | b'7' => Some(0x1f),
        b'8' => Some(0x7f),
        _ => None,
    }
}

/// Alt is sent as an ESC before the key, the xterm "meta sends escape" convention.
fn alt_prefixed(alt: bool, bytes: &[u8]) -> Vec<u8> {
    if alt {
        return [&[ESC], bytes].concat();
    }
    bytes.to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn press(source: &str) -> Keystroke {
        Keystroke::parse(source).unwrap_or_else(|error| panic!("bad keystroke {source:?}: {error}"))
    }

    fn typed(key: &str, key_char: &str) -> Keystroke {
        Keystroke { modifiers: Modifiers::default(), key: key.to_owned(), key_char: Some(key_char.to_owned()) }
    }

    fn sent(keystroke: &Keystroke, mode: TermMode) -> String {
        match key_action(keystroke, mode) {
            KeyAction::Send(bytes) => String::from_utf8(bytes).expect("utf-8"),
            other => panic!("expected bytes, got {other:?}"),
        }
    }

    #[test]
    fn should_switch_cursor_keys_to_ss3_in_application_cursor_mode() {
        // given
        let up = press("up");

        // when
        let normal = sent(&up, TermMode::empty());
        let application = sent(&up, TermMode::APP_CURSOR);

        // then
        assert_eq!((normal.as_str(), application.as_str()), ("\x1b[A", "\x1bOA"));
    }

    #[test]
    fn should_add_xterm_modifier_parameter_to_modified_keys() {
        // given
        let keys = ["ctrl-right", "shift-home", "ctrl-shift-f5", "alt-delete", "ctrl-f1"];

        // when
        let sequences: Vec<_> = keys.iter().map(|key| sent(&press(key), TermMode::APP_CURSOR)).collect();

        // then
        assert_eq!(sequences, ["\x1b[1;5C", "\x1b[1;2H", "\x1b[15;6~", "\x1b[3;3~", "\x1b[1;5P"]);
    }

    #[test]
    fn should_send_control_bytes_for_ctrl_chords() {
        // given
        let keys = ["ctrl-c", "ctrl-d", "ctrl-[", "ctrl-space", "ctrl-alt-a", "ctrl-backspace"];

        // when
        let sequences: Vec<_> = keys.iter().map(|key| sent(&press(key), TermMode::empty())).collect();

        // then
        assert_eq!(sequences, ["\x03", "\x04", "\x1b", "\0", "\x1b\x01", "\x08"]);
    }

    #[test]
    fn should_send_typed_text_with_escape_prefix_for_alt() {
        // given
        let polish = typed("a", "ą");
        let alt_b = press("alt-b");

        // when
        let sequences = [sent(&polish, TermMode::empty()), sent(&alt_b, TermMode::empty())];

        // then
        assert_eq!(sequences, ["ą", "\x1bb"]);
    }

    #[test]
    fn should_send_the_character_altgr_typed_instead_of_a_control_byte() {
        // given
        let altgr = Modifiers { control: true, alt: true, ..Modifiers::default() };
        let polish = Keystroke { modifiers: altgr, key: "a".to_owned(), key_char: Some("ą".to_owned()) };
        let plain_chord = Keystroke { modifiers: altgr, key: "a".to_owned(), key_char: None };

        // when
        let sequences = [sent(&polish, TermMode::empty()), sent(&plain_chord, TermMode::empty())];

        // then
        assert_eq!(sequences, ["ą", "\x1b\x01"]);
    }

    #[test]
    fn should_send_clipboard_chords_as_keys_when_unbound() {
        // given
        let keys = ["ctrl-shift-c", "ctrl-shift-v", "shift-insert"].map(press);

        // when
        let actions = keys.map(|key| key_action(&key, TermMode::empty()));

        // then
        assert!(actions.iter().all(|action| matches!(action, KeyAction::Send(_))), "{actions:?}");
    }

    #[test]
    fn should_send_back_tab_for_shift_tab() {
        // given
        let back_tab = press("shift-tab");

        // when
        let sequence = sent(&back_tab, TermMode::empty());

        // then
        assert_eq!(sequence, "\x1b[Z");
    }
}
