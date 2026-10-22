//! What `init` declared in `config.keys` and `config.mouse_bindings`, read from Rune values into plain-data
//! [`Binding`]s: WezTerm's key names, modifiers, mouse events and actions.

use std::collections::HashMap;

use gpui::{Keystroke, Modifiers, SharedString};
use rune::Value;
use rune::runtime::Function;

use super::config::{Carried, Config};
use super::engine::describe;
use crate::theme::Fonts;
use super::keymap::{Action, Binding, Builtin, ClipboardTarget, MouseButton, MouseKind, MouseTrigger, PasteKind, Scope, SelectionMode, Trigger};

/// WezTerm's names of the keys that are not one character, and gpui's names for them.
const NAMED_KEYS: [(&str, &str); 15] = [
    ("Enter", "enter"),
    ("Tab", "tab"),
    ("Escape", "escape"),
    ("Backspace", "backspace"),
    ("Delete", "delete"),
    ("Insert", "insert"),
    ("Home", "home"),
    ("End", "end"),
    ("PageUp", "pageup"),
    ("PageDown", "pagedown"),
    ("UpArrow", "up"),
    ("DownArrow", "down"),
    ("LeftArrow", "left"),
    ("RightArrow", "right"),
    ("Space", "space"),
];
/// Longer than any real family name; a longer one is a mistake.
const MAX_FONT_NAME: usize = 100;
const KEY_FIELDS: [&str; 4] = ["key", "mods", "action", "scope"];
const MOUSE_FIELDS: [&str; 5] = ["event", "mods", "action", "scope", "mouse_reporting"];

/// Where the functions bound by a load go, and the load they belong to.
pub struct Functions<'a> {
    pub bound: &'a mut Vec<Function>,
    pub generation: u64,
}

/// What one script's `init` set: its bindings, the fonts it chose (`None` where it chose none), and what it left
/// for the next script.
pub struct Declared {
    pub bindings: Vec<Binding>,
    pub fonts: FontChoice,
    pub carried: Carried,
}

#[derive(Debug, Default)]
pub struct FontChoice {
    pub ui: Option<SharedString>,
    pub mono: Option<SharedString>,
}

impl FontChoice {
    /// The chosen fonts, and `fonts` for those not chosen.
    pub fn over(self, fonts: Fonts) -> Fonts {
        Fonts { ui: self.ui.unwrap_or(fonts.ui), mono: self.mono.unwrap_or(fonts.mono) }
    }
}

/// What `config` holds after `file`'s `init` ran; an error names the file, the field or list and the entry.
pub fn declared(config: Config, file: &str, functions: &mut Functions) -> Result<Declared, String> {
    let parts = config.into_parts();
    let ui = font_family(parts.ui_font, "ui_font").map_err(|problem| format!("{file}: {problem}"))?;
    let mono = font_family(parts.font, "font").map_err(|problem| format!("{file}: {problem}"))?;
    let bindings = bindings(parts.keys, parts.mouse_bindings, file, functions)?;
    Ok(Declared { bindings, fonts: FontChoice { ui, mono }, carried: parts.carried })
}

/// A font family `init` named, or none when it left `field` unset.
fn font_family(value: Value, field: &str) -> Result<Option<SharedString>, String> {
    if rune::from_value::<()>(value.clone()).is_ok() {
        return Ok(None);
    }
    let name = string(&value).map(|name| name.trim().to_owned()).filter(|name| (1..=MAX_FONT_NAME).contains(&name.chars().count()));
    let example = if field == "font" { "JetBrains Mono" } else { "Inter" };
    name.map(|name| Some(name.into())).ok_or_else(|| format!("{field} must be a font family name such as {example:?}, not {}", describe(value)))
}

fn bindings(keys: Value, mouse_bindings: Value, file: &str, functions: &mut Functions) -> Result<Vec<Binding>, String> {
    let mut bindings = Vec::new();
    for (index, entry) in list(keys).map_err(|problem| format!("{file}: keys {problem}"))?.into_iter().enumerate() {
        bindings.push(key_binding(entry, functions).map_err(|problem| format!("{file}: keys[{index}]: {problem}"))?);
    }
    for (index, entry) in list(mouse_bindings).map_err(|problem| format!("{file}: mouse_bindings {problem}"))?.into_iter().enumerate() {
        bindings.push(mouse_binding(entry, functions).map_err(|problem| format!("{file}: mouse_bindings[{index}]: {problem}"))?);
    }
    Ok(bindings)
}

/// One key in WezTerm's terms: `key` (`"a"`, `"F5"`, `"UpArrow"`) and `mods` (`"CTRL|SHIFT"`, `"NONE"`).
pub fn parse_key(key: &str, mods: &str) -> Result<Keystroke, String> {
    let modifiers = parse_mods(mods)?;
    Ok(Keystroke { modifiers, key: key_name(key)?, key_char: None })
}

fn key_name(key: &str) -> Result<String, String> {
    if key.chars().count() == 1 {
        return Ok(key.to_lowercase());
    }
    if let Some((_, name)) = NAMED_KEYS.iter().find(|(wezterm, _)| *wezterm == key) {
        return Ok((*name).to_owned());
    }
    let function = key.strip_prefix('F').and_then(|number| number.parse::<u8>().ok()).filter(|number| (1..=24).contains(number));
    if let Some(number) = function {
        return Ok(format!("f{number}"));
    }
    let named: Vec<&str> = NAMED_KEYS.iter().map(|(wezterm, _)| *wezterm).collect();
    Err(format!("key {key:?} is not a key IVE knows (one character, F1-F24, or {})", named.join(", ")))
}

fn parse_mods(mods: &str) -> Result<Modifiers, String> {
    let mut modifiers = Modifiers::default();
    for name in mods.split('|').map(str::trim).filter(|name| !name.is_empty() && *name != "NONE") {
        match name {
            "CTRL" => modifiers.control = true,
            "SHIFT" => modifiers.shift = true,
            "ALT" | "OPT" | "META" => modifiers.alt = true,
            "SUPER" | "CMD" | "WIN" => modifiers.platform = true,
            "SECONDARY" if cfg!(target_os = "macos") => modifiers.platform = true,
            "SECONDARY" => modifiers.control = true,
            _ => return Err(format!("modifier {name:?} in {mods:?} is not one of NONE, CTRL, SHIFT, ALT, SUPER, SECONDARY")),
        }
    }
    Ok(modifiers)
}

/// A list `init` assigned, or none when it left it unset.
fn list(value: Value) -> Result<Vec<Value>, String> {
    if rune::from_value::<()>(value.clone()).is_ok() {
        return Ok(Vec::new());
    }
    rune::from_value::<Vec<Value>>(value).map_err(|_| "must be a list of objects".to_owned())
}

fn object(value: Value, fields: &[&str]) -> Result<HashMap<String, Value>, String> {
    let object = rune::from_value::<HashMap<String, Value>>(value).map_err(|_| "must be an object such as #{ ... }".to_owned())?;
    if let Some(unknown) = object.keys().find(|field| !fields.contains(&field.as_str())) {
        return Err(format!("unknown field {unknown:?} (fields: {})", fields.join(", ")));
    }
    Ok(object)
}

fn text(object: &HashMap<String, Value>, field: &str) -> Result<Option<String>, String> {
    object.get(field).map(|value| string(value).ok_or_else(|| format!("{field} must be a string"))).transpose()
}

/// A Rune string, borrowed: taking it would empty a variable that another entry also uses.
pub(super) fn string(value: &Value) -> Option<String> {
    value.borrow_string_ref().ok().map(|text| text.to_owned())
}

fn required<'a>(object: &'a HashMap<String, Value>, field: &str) -> Result<&'a Value, String> {
    object.get(field).ok_or_else(|| format!("{field} is missing"))
}

fn key_binding(entry: Value, functions: &mut Functions) -> Result<Binding, String> {
    let entry = object(entry, &KEY_FIELDS)?;
    let key = text(&entry, "key")?.ok_or("key is missing")?;
    let keystroke = parse_key(&key, &text(&entry, "mods")?.unwrap_or_default())?;
    let scope = match text(&entry, "scope")?.as_deref() {
        None | Some("window") => Scope::Window,
        Some("terminal") => Scope::Terminal,
        Some("editor") => Scope::Editor,
        Some("hub") => Scope::Hub,
        Some("tree") => Scope::Tree,
        Some(other) => return Err(format!("scope {other:?} is not a key scope (window, terminal, editor, hub, tree)")),
    };
    let action = action(required(&entry, "action")?.clone(), scope, functions)?;
    if matches!(&action, Action::Builtin(builtin) if builtin.is_mouse_only()) {
        return Err(format!("{} works only in mouse_bindings", action_name(&action)));
    }
    Ok(Binding { trigger: Trigger::Key(keystroke), scope, action })
}

fn mouse_binding(entry: Value, functions: &mut Functions) -> Result<Binding, String> {
    let entry = object(entry, &MOUSE_FIELDS)?;
    let scope = match text(&entry, "scope")?.as_deref() {
        Some("tab") => Scope::Tab,
        Some("tree") => Scope::Tree,
        Some("terminal") => Scope::Terminal,
        Some(other) => return Err(format!("scope {other:?} is not a mouse scope (tab, tree, terminal)")),
        None => return Err("scope is missing: tab, tree or terminal".to_owned()),
    };
    let mouse_reporting = entry.get("mouse_reporting").map(|value| rune::from_value::<bool>(value.clone())).transpose().map_err(|_| "mouse_reporting must be true or false")?;
    if mouse_reporting.is_some() && scope != Scope::Terminal {
        return Err("mouse_reporting applies only to the terminal scope".to_owned());
    }
    let (kind, button, streak) = event(required(&entry, "event")?.clone())?;
    let mods = parse_mods(&text(&entry, "mods")?.unwrap_or_default())?;
    let trigger = MouseTrigger { kind, button, streak, mods, mouse_reporting: mouse_reporting.unwrap_or(false) };
    let action = action(required(&entry, "action")?.clone(), scope, functions)?;
    Ok(Binding { trigger: Trigger::Mouse(trigger), scope, action })
}

/// `#{ Up: #{ streak: 1, button: "Middle" } }`.
fn event(value: Value) -> Result<(MouseKind, MouseButton, u8), String> {
    let outer = object(value, &["Down", "Up", "Drag"])?;
    let mut kinds = outer.into_iter();
    let (Some((kind, inner)), None) = (kinds.next(), kinds.next()) else {
        return Err("event must hold one of Down, Up, Drag".to_owned());
    };
    let kind = match kind.as_str() {
        "Down" => MouseKind::Down,
        "Up" => MouseKind::Up,
        _ => MouseKind::Drag,
    };
    let inner = object(inner, &["streak", "button"])?;
    let button = match text(&inner, "button")?.as_deref() {
        Some("Left") => MouseButton::Left,
        Some("Middle") => MouseButton::Middle,
        Some("Right") => MouseButton::Right,
        other => return Err(format!("button {other:?} is not Left, Middle or Right")),
    };
    let streak = inner.get("streak").map(|value| rune::from_value::<i64>(value.clone())).transpose().map_err(|_| "streak must be a number")?.unwrap_or(1);
    let streak = u8::try_from(streak).ok().filter(|streak| (1..=3).contains(streak)).ok_or_else(|| format!("streak {streak} is not 1, 2 or 3"))?;
    Ok((kind, button, streak))
}

fn action(value: Value, scope: Scope, functions: &mut Functions) -> Result<Action, String> {
    if let Ok(function) = rune::from_value::<Function>(value.clone()) {
        functions.bound.push(function);
        return Ok(Action::Invoke { generation: functions.generation, function: functions.bound.len() - 1 });
    }
    let action = match string(&value) {
        Some(name) => unit_action(&name)?,
        None => tagged_action(value)?,
    };
    supported(action, scope)
}

pub(super) fn unit_action(name: &str) -> Result<Action, String> {
    let builtin = match name {
        "DisableDefaultAssignment" => return Ok(Action::Disable),
        "Nop" => Builtin::Nop,
        "OpenSettings" => Builtin::OpenSettings,
        "OpenTerminal" => Builtin::OpenTerminal,
        "ActivateTab" => Builtin::ActivateTab,
        "CloseTab" => Builtin::CloseTab,
        "KeepTab" => Builtin::KeepTab,
        "StopProgram" => Builtin::StopProgram,
        "KillProgram" => Builtin::KillProgram,
        "CloseOtherTabs" => Builtin::CloseOtherTabs,
        "CloseTabsToTheLeft" => Builtin::CloseTabsToTheLeft,
        "CloseTabsToTheRight" => Builtin::CloseTabsToTheRight,
        "ToggleChangesLayout" => Builtin::ToggleChangesLayout,
        "OpenTerminalIn" => Builtin::OpenTerminalIn,
        "ActivateRow" => Builtin::ActivateRow,
        "ActivateRowToKeep" => Builtin::ActivateRowToKeep,
        "ToggleRow" => Builtin::ToggleRow,
        "ToggleRowSelection" => Builtin::ToggleRowSelection,
        "ExtendRowSelection" => Builtin::ExtendRowSelection,
        "SelectAll" => Builtin::SelectAll,
        "Cut" => Builtin::Cut,
        "Undo" => Builtin::Undo,
        "Redo" => Builtin::Redo,
        "OpenHub" => Builtin::OpenHub,
        "HubDown" => Builtin::HubDown,
        "HubUp" => Builtin::HubUp,
        "HubPageDown" => Builtin::HubPageDown,
        "HubPageUp" => Builtin::HubPageUp,
        "HubOpen" => Builtin::HubOpen,
        "HubOpenBehind" => Builtin::HubOpenBehind,
        "HubTypePath" => Builtin::HubTypePath,
        "HubCopyPath" => Builtin::HubCopyPath,
        "HubStop" => Builtin::HubStop,
        "CopyFullPath" => Builtin::CopyFullPath,
        "HubCycleScope" => Builtin::HubCycleScope,
        "HubWiden" => Builtin::HubWiden,
        "HubToggleExcluded" => Builtin::HubToggleExcluded,
        "HubToggleChanged" => Builtin::HubToggleChanged,
        "HubBack" => Builtin::HubBack,
        "SendPathsToTerminal" => Builtin::SendPathsToTerminal,
        "CopyRelativePath" => Builtin::CopyRelativePath,
        "CopyFileName" => Builtin::CopyFileName,
        "FindInFiles" => Builtin::FindInFiles,
        "OpenFindInFiles" => Builtin::OpenFindInFiles,
        "OpenTerminalHere" => Builtin::OpenTerminalHere,
        "RevealInFileManager" => Builtin::RevealInFileManager,
        "OpenWithDefaultApp" => Builtin::OpenWithDefaultApp,
        "OpenInBrowser" => Builtin::OpenInBrowser,
        "RunFile" => Builtin::RunFile,
        "GitShowDiff" => Builtin::GitShowDiff,
        "GitCompareWithBase" => Builtin::GitCompareWithBase,
        "GitHistory" => Builtin::GitHistory,
        "GitIgnore" => Builtin::GitIgnore,
        "CompareFiles" => Builtin::CompareFiles,
        "OpenInNewWindow" => Builtin::OpenInNewWindow,
        "HubToggleContent" => Builtin::HubToggleContent,
        "HubToggleRegex" => Builtin::HubToggleRegex,
        "HubToggleWholeWord" => Builtin::HubToggleWholeWord,
        _ => return Err(format!("action {name:?} is not an act:: action")),
    };
    Ok(Action::Builtin(builtin))
}

fn tagged_action(value: Value) -> Result<Action, String> {
    let not_an_action = "action must be an act:: action or a function";
    let tagged = rune::from_value::<HashMap<String, Value>>(value).map_err(|_| not_an_action.to_owned())?;
    let mut entries = tagged.into_iter();
    let (Some((name, argument)), None) = (entries.next(), entries.next()) else {
        return Err(not_an_action.to_owned());
    };
    if name == "HubOpenRow" {
        return hub_row(&argument);
    }
    let argument = string(&argument).ok_or_else(|| format!("{name} takes a string"))?;
    let builtin = match name.as_str() {
        "SendString" => return Ok(Action::Send(argument)),
        "ShowTabMenu" => return Ok(Action::ShowTabMenu(argument)),
        "ShowTreeMenu" => return Ok(Action::ShowTreeMenu(argument)),
        "CopyTo" => Builtin::CopyTo(clipboard(&argument)?),
        "PasteFrom" => Builtin::PasteFrom(paste_source(&argument)?),
        "PasteTextFrom" => Builtin::Paste(PasteKind::Text, paste_source(&argument)?),
        "PasteImageFrom" => Builtin::Paste(PasteKind::Image, paste_source(&argument)?),
        "PasteTextAndImageFrom" => Builtin::Paste(PasteKind::TextAndImage, paste_source(&argument)?),
        "SelectTextAtMouseCursor" => Builtin::SelectTextAtMouseCursor(selection_mode(&argument)?),
        "ExtendSelectionToMouseCursor" => Builtin::ExtendSelectionToMouseCursor(selection_mode(&argument)?),
        "CompleteSelection" => Builtin::CompleteSelection(clipboard(&argument)?),
        _ => return Err(format!("action {name:?} is not an act:: action")),
    };
    Ok(Action::Builtin(builtin))
}

/// `act::HubOpenRow(n)`: one of the rows numbered on screen.
fn hub_row(argument: &Value) -> Result<Action, String> {
    let row = rune::from_value::<i64>(argument.clone()).map_err(|_| "act::HubOpenRow takes a row number 1 to 9".to_owned())?;
    match u8::try_from(row).ok().filter(|row| (1..=9).contains(row)) {
        Some(row) => Ok(Action::Builtin(Builtin::HubOpenRow(row))),
        None => Err(format!("act::HubOpenRow takes a row number 1 to 9, not {row}")),
    }
}

fn clipboard(target: &str) -> Result<ClipboardTarget, String> {
    match target {
        "Clipboard" => Ok(ClipboardTarget::Clipboard),
        "PrimarySelection" => Ok(ClipboardTarget::Primary),
        "ClipboardAndPrimarySelection" => Ok(ClipboardTarget::Both),
        _ => Err(format!("{target:?} is not Clipboard, PrimarySelection or ClipboardAndPrimarySelection")),
    }
}

fn paste_source(source: &str) -> Result<ClipboardTarget, String> {
    match clipboard(source)? {
        ClipboardTarget::Both => Err("paste from Clipboard or PrimarySelection, not both".to_owned()),
        one => Ok(one),
    }
}

fn selection_mode(mode: &str) -> Result<SelectionMode, String> {
    match mode {
        "Cell" => Ok(SelectionMode::Cell),
        "Word" => Ok(SelectionMode::Word),
        "Line" => Ok(SelectionMode::Line),
        _ => Err(format!("selection mode {mode:?} is not Cell, Word or Line")),
    }
}

fn supported(action: Action, scope: Scope) -> Result<Action, String> {
    let fits = match &action {
        Action::Builtin(builtin) => builtin.supports(scope),
        Action::Send(_) => matches!(scope, Scope::Window | Scope::Terminal),
        Action::ShowTabMenu(_) => scope == Scope::Tab,
        Action::ShowTreeMenu(_) => scope == Scope::Tree,
        Action::Invoke { .. } | Action::Disable => true,
    };
    if fits {
        return Ok(action);
    }
    Err(format!("{} does nothing in the {} scope", action_name(&action), scope.name()))
}

fn action_name(action: &Action) -> String {
    match action {
        Action::Builtin(Builtin::Paste(kind, _)) => kind.action_name().to_owned(),
        Action::Builtin(builtin) => format!("{builtin:?}").split('(').next().unwrap_or_default().to_owned(),
        Action::Send(_) => "SendString".to_owned(),
        Action::Invoke { .. } => "a function".to_owned(),
        Action::Disable => "DisableDefaultAssignment".to_owned(),
        Action::ShowTabMenu(_) => "ShowTabMenu".to_owned(),
        Action::ShowTreeMenu(_) => "ShowTreeMenu".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(key: &str, mods: &str) -> String {
        parse_key(key, mods).unwrap().unparse()
    }

    #[test]
    fn should_parse_wezterm_key_names() {
        // when
        let keys = [key("Enter", "SHIFT"), key("F12", "ALT"), key("C", "CTRL|SHIFT"), key("UpArrow", "NONE"), key("PageDown", "")];

        // then
        assert_eq!(keys, ["shift-enter", "alt-f12", "ctrl-shift-c", "up", "pagedown"]);
    }

    #[test]
    fn should_map_secondary_to_the_platform_modifier() {
        // when
        let settings = parse_key(",", "SECONDARY").unwrap();

        // then
        assert!(settings.modifiers.secondary());
        assert_eq!(settings.key, ",");
    }

    #[test]
    fn should_accept_modifier_aliases() {
        // when
        let keys = [key("a", "OPT"), key("a", "META"), key("a", "CMD"), key("a", "WIN")];

        // then
        assert_eq!(keys, ["alt-a", "alt-a", "cmd-a", "cmd-a"].map(|expected| Keystroke::parse(expected).unwrap().unparse()));
    }

    #[test]
    fn should_reject_unknown_key_name() {
        // when
        let error = parse_key("Banana", "NONE").unwrap_err();

        // then
        assert!(error.contains("Banana") && error.contains("UpArrow"), "{error}");
    }

    #[test]
    fn should_reject_unknown_modifier() {
        // when
        let error = parse_key("a", "CTRL|HYPER").unwrap_err();

        // then
        assert!(error.contains("HYPER"), "{error}");
    }
}
