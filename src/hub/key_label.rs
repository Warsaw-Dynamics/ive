//! Keys as the hub's key strip shows them: `Ctrl+Shift+W`, `Alt+1`, `↑`.

use gpui::Keystroke;

/// `ctrl-shift-w` as `Ctrl+Shift+W`; modifiers in IntelliJ's order.
pub fn label(keystroke: &Keystroke) -> String {
    let modifiers = &keystroke.modifiers;
    let held = [(modifiers.control, "Ctrl"), (modifiers.alt, "Alt"), (modifiers.shift, "Shift"), (modifiers.platform, "Cmd")];
    let mut parts: Vec<String> = held.iter().filter(|(on, _)| *on).map(|(_, name)| (*name).to_owned()).collect();
    parts.push(key_name(&keystroke.key));
    parts.join("+")
}

fn key_name(key: &str) -> String {
    let named = match key {
        "up" => "↑",
        "down" => "↓",
        "left" => "←",
        "right" => "→",
        "pageup" => "PgUp",
        "pagedown" => "PgDn",
        "escape" => "Esc",
        "enter" => "Enter",
        "backspace" => "Backspace",
        "delete" => "Del",
        "tab" => "Tab",
        "space" => "Space",
        "home" => "Home",
        "end" => "End",
        "insert" => "Ins",
        _ => return key.to_uppercase(),
    };
    named.to_owned()
}

#[cfg(test)]
mod tests {
    use gpui::Keystroke;

    use super::*;

    #[test]
    fn should_label_keys_as_shown_in_the_key_strip() {
        // given
        let keys = ["ctrl-shift-w", "alt-1", "enter", "pagedown", "pageup", "up", "down", "escape", "backspace", "ctrl-y", "tab", "cmd-e"];

        // when
        let labels = keys.map(|key| label(&Keystroke::parse(key).unwrap()));

        // then
        assert_eq!(labels, ["Ctrl+Shift+W", "Alt+1", "Enter", "PgDn", "PgUp", "↑", "↓", "Esc", "Backspace", "Ctrl+Y", "Tab", "Cmd+E"]);
    }
}
