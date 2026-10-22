//! IVE's side of `codeview::host`: its key bindings and its clipboard.

use std::rc::Rc;

use codeview::host::{Command, Host, KeyOutcome};
use gpui::{App, KeyDownEvent};

use crate::clipboard;
use crate::script::keymap::{Action, Builtin, Place};
use crate::script::state::{bound_action, invoke};

pub fn install(cx: &mut App) {
    cx.set_global(Host {
        keys: Rc::new(key_outcome),
        clipboard: Rc::new(|target, text, cx| clipboard::write(target, text, cx)),
        paste: Rc::new(|source, cx| clipboard::read(source, cx)),
    });
}

/// What the editor does with `event`: its bound action, if any.
fn key_outcome(event: &KeyDownEvent, cx: &App) -> KeyOutcome {
    match bound_action(event, Place::Editor, cx) {
        None => KeyOutcome::Unbound,
        Some(Action::Builtin(builtin)) => match command_of(builtin) {
            Some(command) => KeyOutcome::Command(command),
            // the shown tab is an editor, not a terminal: it does nothing
            None if builtin == Builtin::Nop => KeyOutcome::Handled,
            None => KeyOutcome::Foreign,
        },
        Some(Action::Invoke { generation, function }) => {
            invoke(generation, function, None, cx);
            KeyOutcome::Handled
        }
        // the shown tab is an editor, not a terminal, so a bound `SendString` has nowhere to go
        Some(Action::Send(_)) => KeyOutcome::Handled,
        // a window action (Settings, a new terminal) is the workspace's to run
        Some(Action::ShowTabMenu(_) | Action::Disable) => KeyOutcome::Foreign,
    }
}

/// The editor's own built-ins; the rest are the workspace's.
fn command_of(builtin: Builtin) -> Option<Command> {
    Some(match builtin {
        Builtin::SelectAll => Command::SelectAll,
        Builtin::CopyTo(target) => Command::Copy(target),
        Builtin::Cut => Command::Cut,
        Builtin::PasteFrom(source) => Command::Paste(source),
        Builtin::Undo => Command::Undo,
        Builtin::Redo => Command::Redo,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use codeview::host::ClipboardTarget;

    use super::*;

    #[test]
    fn should_map_editor_builtins_to_commands() {
        // given
        let builtins = [Builtin::Undo, Builtin::CopyTo(ClipboardTarget::Both), Builtin::SelectAll, Builtin::OpenHub];

        // when
        let commands = builtins.map(command_of);

        // then
        assert_eq!(commands, [Some(Command::Undo), Some(Command::Copy(ClipboardTarget::Both)), Some(Command::SelectAll), None]);
    }
}
