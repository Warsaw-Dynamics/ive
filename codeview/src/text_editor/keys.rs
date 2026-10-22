//! What a key press means in the text editor.

use gpui::{Keystroke, Modifiers};

use super::model::{Motion, Selecting, Unit};
use crate::host::Command;

/// `page_rows` is how far PageUp and PageDown move, known only to the view.
pub fn command(keystroke: &Keystroke, page_rows: usize) -> Option<Command> {
    let modifiers = &keystroke.modifiers;
    let key = keystroke.key.as_str();
    editing(key, modifiers).or_else(|| movement(key, modifiers, page_rows))
}

fn editing(key: &str, modifiers: &Modifiers) -> Option<Command> {
    let unit = if by_word(modifiers) { Unit::Word } else { Unit::Char };
    match key {
        "enter" => Some(Command::Newline),
        "tab" => Some(Command::Indent),
        "backspace" => Some(Command::Backspace(unit)),
        "delete" => Some(Command::Delete(unit)),
        _ => None,
    }
}

fn movement(key: &str, modifiers: &Modifiers, page_rows: usize) -> Option<Command> {
    let word = by_word(modifiers);
    let document = modifiers.control || modifiers.platform;
    let motion = match key {
        "left" if word => Motion::WordLeft,
        "left" => Motion::Left,
        "right" if word => Motion::WordRight,
        "right" => Motion::Right,
        "up" => Motion::Up,
        "down" => Motion::Down,
        "home" if document => Motion::DocumentStart,
        "home" => Motion::LineStart,
        "end" if document => Motion::DocumentEnd,
        "end" => Motion::LineEnd,
        "pageup" => Motion::PageUp(page_rows),
        "pagedown" => Motion::PageDown(page_rows),
        _ => return None,
    };
    let selecting = if modifiers.shift { Selecting::Extend } else { Selecting::Move };
    Some(Command::Move(motion, selecting))
}

/// Ctrl on Linux and Windows, Alt (Option) on macOS, as their native text fields do.
fn by_word(modifiers: &Modifiers) -> bool {
    if cfg!(target_os = "macos") { modifiers.alt } else { modifiers.control }
}

#[cfg(test)]
mod tests {
    use gpui::Keystroke;

    use super::*;

    fn command_for(key: &str) -> Option<Command> {
        command(&Keystroke::parse(key).unwrap(), 30)
    }

    #[test]
    fn should_leave_clipboard_and_history_chords_to_bindings() {
        // given
        let keys = ["secondary-a", "secondary-c", "secondary-x", "secondary-v", "secondary-z", "secondary-shift-z"];

        // when
        let commands = keys.map(command_for);

        // then
        assert_eq!(commands, [None, None, None, None, None, None]);
    }

    #[test]
    fn should_map_editing_keys() {
        // given
        let keys = ["enter", "tab", "backspace", "delete"];

        // when
        let commands = keys.map(command_for);

        // then
        let expected = [Command::Newline, Command::Indent, Command::Backspace(Unit::Char), Command::Delete(Unit::Char)];
        assert_eq!(commands, expected.map(Some));
    }

    #[test]
    fn should_extend_the_selection_with_shift_and_carry_the_page_size() {
        // given
        let keys = ["shift-pagedown", "home", "shift-up"];

        // when
        let commands = keys.map(command_for);

        // then
        let expected = [
            Command::Move(Motion::PageDown(30), Selecting::Extend),
            Command::Move(Motion::LineStart, Selecting::Move),
            Command::Move(Motion::Up, Selecting::Extend),
        ];
        assert_eq!(commands, expected.map(Some));
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn should_move_and_delete_by_words_and_jump_to_the_document_ends_with_ctrl() {
        // given
        let keys = ["ctrl-left", "ctrl-shift-right", "ctrl-backspace", "ctrl-delete", "ctrl-home", "ctrl-shift-end"];

        // when
        let commands = keys.map(command_for);

        // then
        let expected = [
            Command::Move(Motion::WordLeft, Selecting::Move),
            Command::Move(Motion::WordRight, Selecting::Extend),
            Command::Backspace(Unit::Word),
            Command::Delete(Unit::Word),
            Command::Move(Motion::DocumentStart, Selecting::Move),
            Command::Move(Motion::DocumentEnd, Selecting::Extend),
        ];
        assert_eq!(commands, expected.map(Some));
    }

    #[test]
    fn should_leave_typed_characters_escape_and_ive_bindings_alone() {
        // given
        let keys = ["a", "shift-a", "space", "escape", "alt-f12"];

        // when
        let commands = keys.map(command_for);

        // then
        assert!(commands.iter().all(Option::is_none), "{commands:?}");
    }
}
