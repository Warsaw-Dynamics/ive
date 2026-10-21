//! `config.tab_menus`: named menus of tab actions, which `act::ShowTabMenu(name)` opens on a tab.

use gpui::Global;
use serde_json::{Map, Value};

use super::declare;
use super::keymap::{Action, Builtin};

const SEPARATOR: &str = "Separator";
const MAX_MENUS: usize = 32;
const MAX_ITEMS: usize = 32;
const MAX_NAME_CHARS: usize = 32;

/// A line of a tab menu as declared.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuEntry {
    /// an action [`Builtin::is_tab_command`] accepts
    Item(Builtin),
    Separator,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TabMenus {
    menus: Vec<(String, Vec<MenuEntry>)>,
}

impl Global for TabMenus {}

impl TabMenus {
    pub fn get(&self, name: &str) -> Option<&[MenuEntry]> {
        self.menus.iter().find(|(known, _)| known == name).map(|(_, entries)| entries.as_slice())
    }
}

/// `value` as `config.tab_menus` left it, over `defaults` (what `ive.rn` set) per menu name; the error names the menu,
/// the item's place and the offending value.
pub fn parse(defaults: &rune::Value, value: &rune::Value) -> Result<TabMenus, String> {
    let mut merged = object(defaults)?;
    merged.extend(object(value)?);
    if merged.len() > MAX_MENUS {
        return Err(format!("tab_menus: {} menus declared, at most {MAX_MENUS}", merged.len()));
    }
    let mut menus = Vec::new();
    for (name, items) in merged {
        let count = name.chars().count();
        if count == 0 || count > MAX_NAME_CHARS {
            return Err(format!("tab_menus: menu name {name:?} must be 1 to {MAX_NAME_CHARS} characters"));
        }
        let entries = entries(&name, &items)?;
        menus.push((name, entries));
    }
    Ok(TabMenus { menus })
}

/// Fails on the first `ShowTabMenu` of `names` that `menus` does not declare.
pub fn check_shown<'a>(menus: &TabMenus, mut names: impl Iterator<Item = &'a str>) -> Result<(), String> {
    match names.find(|name| menus.get(name).is_none()) {
        Some(name) => Err(format!("act::ShowTabMenu({name:?}): config.tab_menus declares no menu {name:?}")),
        None => Ok(()),
    }
}

fn object(value: &rune::Value) -> Result<Map<String, Value>, String> {
    let json = serde_json::to_value(value).map_err(|error| format!("tab_menus: {error}"))?;
    match json {
        Value::Null => Ok(Map::new()),
        Value::Object(object) => Ok(object),
        other => Err(format!("tab_menus must be an object such as #{{ main: [act::CloseTab] }}, not {other}")),
    }
}

fn entries(name: &str, items: &Value) -> Result<Vec<MenuEntry>, String> {
    let Some(items) = items.as_array() else {
        return Err(format!("tab_menus.{name} must be a list such as [act::CloseTab], not {items}"));
    };
    if items.len() > MAX_ITEMS {
        return Err(format!("tab_menus.{name}: {} items, at most {MAX_ITEMS}", items.len()));
    }
    items.iter().enumerate().map(|(index, item)| entry(item).map_err(|problem| format!("tab_menus.{name}[{index}]: {problem}"))).collect()
}

fn entry(item: &Value) -> Result<MenuEntry, String> {
    let not_an_item = || format!("{item} is not a tab action (KeepTab, StopProgram, KillProgram, CloseTab, CloseOtherTabs, CloseTabsToTheLeft, CloseTabsToTheRight) or \"{SEPARATOR}\"");
    let name = item.as_str().ok_or_else(not_an_item)?;
    if name == SEPARATOR {
        return Ok(MenuEntry::Separator);
    }
    match declare::unit_action(name) {
        Ok(Action::Builtin(builtin)) if builtin.is_tab_command() => Ok(MenuEntry::Item(builtin)),
        _ => Err(not_an_item()),
    }
}
