//! `config.drop_modifiers = #{ force_copy: "CTRL", force_move: "SHIFT" }`: the modifiers that force a tree drop to
//! copy or move, written as key bindings' `mods` (`move` is a Rune keyword, so it cannot be a key).

use gpui::Modifiers;
use serde_json::Value;

use super::declare::parse_mods;
use crate::transfer::DropModifiers;

const KEYS: [&str; 2] = ["force_copy", "force_move"];

pub fn parse(value: &rune::Value) -> Result<DropModifiers, String> {
    let json = serde_json::to_value(value).map_err(|error| format!("drop_modifiers: {error}"))?;
    let Value::Object(table) = json else {
        return Err(format!("drop_modifiers must be #{{ force_copy: \"CTRL\", force_move: \"SHIFT\" }}, not {json}"));
    };
    if let Some(unknown) = table.keys().find(|key| !KEYS.contains(&key.as_str())) {
        return Err(format!("drop_modifiers: unknown key {unknown:?}, expected force_copy and force_move"));
    }
    let named = |key: &str| -> Result<(String, Modifiers), String> {
        let declared = table.get(key).ok_or_else(|| format!("drop_modifiers.{key} is missing"))?;
        let text = declared.as_str().ok_or_else(|| format!("drop_modifiers.{key} must be a string like \"CTRL\", not {declared}"))?;
        let modifiers = parse_mods(text).map_err(|error| format!("drop_modifiers.{key}: {error}"))?;
        match modifiers == Modifiers::default() {
            true => Err(format!("drop_modifiers.{key}: {text:?} is no modifier")),
            false => Ok((text.to_owned(), modifiers)),
        }
    };
    let (_, copy) = named("force_copy")?;
    let (text, moving) = named("force_move")?;
    if moving == copy {
        return Err(format!("drop_modifiers.force_move: {text:?} is the same as force_copy"));
    }
    Ok(DropModifiers { copy, moving })
}

#[cfg(test)]
mod tests {
    use gpui::Modifiers;

    use super::*;

    fn value(json: serde_json::Value) -> rune::Value {
        serde_json::from_value(json).unwrap()
    }

    #[test]
    fn should_read_drop_modifiers() {
        // given
        let declared = value(serde_json::json!({ "force_copy": "CTRL", "force_move": "SHIFT|ALT" }));

        // when
        let modifiers = parse(&declared).unwrap();

        // then
        assert_eq!(modifiers.copy, Modifiers { control: true, ..Modifiers::default() });
        assert_eq!(modifiers.moving, Modifiers { shift: true, alt: true, ..Modifiers::default() });
    }

    #[test]
    fn should_refuse_drop_modifier_none() {
        // given
        let declared = value(serde_json::json!({ "force_copy": "NONE", "force_move": "SHIFT" }));

        // when
        let error = parse(&declared).unwrap_err();

        // then
        assert_eq!(error, "drop_modifiers.force_copy: \"NONE\" is no modifier");
    }

    #[test]
    fn should_refuse_same_copy_and_move() {
        // given
        let declared = value(serde_json::json!({ "force_copy": "CTRL", "force_move": "CTRL" }));

        // when
        let error = parse(&declared).unwrap_err();

        // then
        assert_eq!(error, "drop_modifiers.force_move: \"CTRL\" is the same as force_copy");
    }

    #[test]
    fn should_refuse_unknown_drop_modifier_key() {
        // given
        let declared = value(serde_json::json!({ "force_copy": "CTRL", "force_move": "SHIFT", "link": "ALT" }));

        // when
        let error = parse(&declared).unwrap_err();

        // then
        assert_eq!(error, "drop_modifiers: unknown key \"link\", expected force_copy and force_move");
    }

    #[test]
    fn should_refuse_missing_drop_modifier() {
        // given
        let declared = value(serde_json::json!({ "force_copy": "CTRL" }));

        // when
        let error = parse(&declared).unwrap_err();

        // then
        assert_eq!(error, "drop_modifiers.force_move is missing");
    }

    #[test]
    fn should_refuse_unknown_modifier_name() {
        // given
        let declared = value(serde_json::json!({ "force_copy": "HYPER", "force_move": "SHIFT" }));

        // when
        let error = parse(&declared).unwrap_err();

        // then
        assert!(error.starts_with("drop_modifiers.force_copy: modifier \"HYPER\""), "{error}");
    }
}
