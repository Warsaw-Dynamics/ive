//! `config.temporary_tabs`: which tabs a click opens temporary, to go once another tab is shown, and how their
//! titles show it.

use gpui::{Global, SharedString};
use serde_json::{Map, Value};

const KEYS: [&str; 3] = ["enabled", "kinds", "marker"];
const KIND_NAMES: [(&str, TabKind); 4] =
    [("Project", TabKind::Project), ("Settings", TabKind::Settings), ("Hub", TabKind::Hub), ("RunConfigs", TabKind::RunConfigs)];
/// Room for a symbol or a short word; a longer prefix would crowd the title out.
const MAX_PREFIX_CHARS: usize = 8;

/// The tabs that may open temporary: a project's overview, Settings, the hub and a checkout's run configurations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabKind {
    Project,
    Settings,
    Hub,
    RunConfigs,
}

/// How a temporary tab's title differs from a permanent one's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TabMarker {
    Italic,
    /// written before the title, with a space
    Prefix(SharedString),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemporaryTabs {
    pub enabled: bool,
    pub kinds: Vec<TabKind>,
    pub marker: TabMarker,
}

impl Global for TemporaryTabs {}

impl TemporaryTabs {
    pub fn opens_temporary(&self, kind: TabKind) -> bool {
        self.enabled && self.kinds.contains(&kind)
    }
}

/// `value` as `config.temporary_tabs` left it, its missing keys taken from `defaults` (what `ive.rn` set); the
/// error names the key and the offending value.
pub fn parse(defaults: &rune::Value, value: &rune::Value) -> Result<TemporaryTabs, String> {
    let mut merged = object(defaults)?;
    merged.extend(object(value)?);
    Ok(TemporaryTabs {
        enabled: merged.get("enabled").and_then(Value::as_bool).ok_or_else(|| invalid("enabled", "true or false", merged.get("enabled")))?,
        kinds: kinds(merged.get("kinds"))?,
        marker: marker(merged.get("marker"))?,
    })
}

fn object(value: &rune::Value) -> Result<Map<String, Value>, String> {
    let json = serde_json::to_value(value).map_err(|error| format!("temporary_tabs: {error}"))?;
    let Value::Object(object) = json else {
        return Err(format!("temporary_tabs must be an object such as #{{ enabled: true }}, not {json}"));
    };
    if let Some(unknown) = object.keys().find(|key| !KEYS.contains(&key.as_str())) {
        return Err(format!("temporary_tabs: unknown key {unknown:?} (keys: {})", KEYS.join(", ")));
    }
    Ok(object)
}

fn kinds(value: Option<&Value>) -> Result<Vec<TabKind>, String> {
    let names = value.and_then(Value::as_array).ok_or_else(|| invalid("kinds", "a list such as [\"Project\"]", value))?;
    let mut kinds = Vec::new();
    for name in names {
        let kind = KIND_NAMES.iter().find(|(known, _)| name.as_str() == Some(known)).map(|(_, kind)| *kind).ok_or_else(|| {
            let known: Vec<String> = KIND_NAMES.iter().map(|(known, _)| format!("{known:?}")).collect();
            format!("temporary_tabs.kinds: {name} is not one of {}", known.join(", "))
        })?;
        if kinds.contains(&kind) {
            return Err(format!("temporary_tabs.kinds: {name} is listed twice"));
        }
        kinds.push(kind);
    }
    Ok(kinds)
}

fn marker(value: Option<&Value>) -> Result<TabMarker, String> {
    if value.and_then(Value::as_str) == Some("Italic") {
        return Ok(TabMarker::Italic);
    }
    let prefix = value.and_then(|marker| marker.get("Prefix")).filter(|_| value.and_then(Value::as_object).is_some_and(|marker| marker.len() == 1));
    let Some(prefix) = prefix else {
        return Err(invalid("marker", "\"Italic\" or #{ Prefix: \"*\" }", value));
    };
    let text = prefix.as_str().filter(|text| (1..=MAX_PREFIX_CHARS).contains(&text.chars().count()));
    text.map(|text| TabMarker::Prefix(text.to_owned().into()))
        .ok_or_else(|| format!("temporary_tabs.marker: Prefix must be 1 to {MAX_PREFIX_CHARS} characters, not {prefix}"))
}

fn invalid(key: &str, expected: &str, value: Option<&Value>) -> String {
    let found = value.map_or_else(|| "and is not set".to_owned(), |value| format!("not {value}"));
    format!("temporary_tabs.{key} must be {expected}, {found}")
}
