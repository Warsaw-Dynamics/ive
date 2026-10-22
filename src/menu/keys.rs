//! The key a menu line shows on its right: the first key bound to the line's action, in the scopes where the menu's
//! subject takes keys.

use crate::hub::key_label;
use crate::script::keymap::{Action, Keymap, Scope};
use crate::script::menus::{ItemAction, MenuKind};

use super::resolve::LineAction;

/// The label of the key bound to `action` for a menu of `kind`, e.g. `Ctrl+Shift+C`; `None` when no key runs it.
pub fn shortcut(keymap: &Keymap, kind: MenuKind, action: &LineAction) -> Option<String> {
    let action = match action {
        LineAction::Script(ItemAction::Builtin(builtin)) => Action::Builtin(*builtin),
        LineAction::Script(ItemAction::Invoke { generation, function }) => Action::Invoke { generation: *generation, function: *function },
        LineAction::Choice { .. } => return None,
    };
    let keystroke = scopes(kind).iter().find_map(|scope| keymap.keys_for(*scope, &action).first().copied())?;
    Some(key_label::label(keystroke))
}

/// Where keys act on what a menu of `kind` is for: the tree's selection takes the tree's keys, then the window's; a
/// tab's file the window's.
fn scopes(kind: MenuKind) -> &'static [Scope] {
    match kind {
        MenuKind::Tree => &[Scope::Tree, Scope::Window],
        MenuKind::Tab => &[Scope::Window],
    }
}

#[cfg(test)]
mod tests {
    use gpui::Keystroke;

    use super::*;
    use crate::script::keymap::{Binding, Builtin, Trigger};

    fn bound(key: &str, scope: Scope, action: Action) -> Binding {
        Binding { trigger: Trigger::Key(Keystroke::parse(key).unwrap()), scope, action }
    }

    fn builtin(builtin: Builtin) -> LineAction {
        LineAction::Script(ItemAction::Builtin(builtin))
    }

    #[test]
    fn should_show_the_tree_key_before_the_window_key() {
        // given
        let keymap = Keymap::new(vec![
            bound("ctrl-alt-c", Scope::Window, Action::Builtin(Builtin::CopyFullPath)),
            bound("ctrl-shift-c", Scope::Tree, Action::Builtin(Builtin::CopyFullPath)),
        ])
        .unwrap();

        // when
        let keys = [MenuKind::Tree, MenuKind::Tab].map(|kind| shortcut(&keymap, kind, &builtin(Builtin::CopyFullPath)));

        // then
        assert_eq!(keys, [Some("Ctrl+Shift+C".to_owned()), Some("Ctrl+Alt+C".to_owned())]);
    }

    #[test]
    fn should_show_the_key_of_a_function_item() {
        // given
        let invoke = Action::Invoke { generation: 3, function: 7 };
        let keymap = Keymap::new(vec![bound("alt-i", Scope::Window, invoke)]).unwrap();

        // when
        let key = shortcut(&keymap, MenuKind::Tab, &LineAction::Script(ItemAction::Invoke { generation: 3, function: 7 }));

        // then
        assert_eq!(key, Some("Alt+I".to_owned()));
    }

    #[test]
    fn should_show_no_key_for_unbound_action_or_choice() {
        // given
        let keymap = Keymap::new(vec![bound("ctrl-shift-c", Scope::Tree, Action::Builtin(Builtin::CopyFullPath))]).unwrap();
        let choice = LineAction::Choice { builtin: Builtin::SendPathsToTerminal, choice: "42".to_owned() };

        // when
        let keys = [
            shortcut(&keymap, MenuKind::Tab, &builtin(Builtin::CopyFullPath)),
            shortcut(&keymap, MenuKind::Tree, &builtin(Builtin::CopyFileName)),
            shortcut(&keymap, MenuKind::Tree, &choice),
        ];

        // then
        assert_eq!(keys, [None, None, None]);
    }
}
