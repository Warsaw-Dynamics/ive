//! `config.delete`: what Delete in the tree does with files and folders, to the trash or away for good.

use gpui::Global;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DeleteMode {
    /// the system trash, where they can be restored from
    #[default]
    Trash,
    /// removed at once, after a question
    Permanent,
}

impl Global for DeleteMode {}

pub fn parse(value: &rune::Value) -> Result<DeleteMode, String> {
    let json = serde_json::to_value(value).map_err(|error| format!("delete: {error}"))?;
    match json.as_str() {
        Some("trash") => Ok(DeleteMode::Trash),
        Some("permanent") => Ok(DeleteMode::Permanent),
        _ => Err(format!("delete must be \"trash\" or \"permanent\", not {json}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn value(json: serde_json::Value) -> rune::Value {
        serde_json::from_value(json).unwrap()
    }

    #[test]
    fn should_take_trash_and_permanent() {
        // when
        let modes = ["trash", "permanent"].map(|name| parse(&value(serde_json::json!(name))));

        // then
        assert_eq!(modes, [Ok(DeleteMode::Trash), Ok(DeleteMode::Permanent)]);
    }

    #[test]
    fn should_refuse_another_value() {
        // when
        let refused = parse(&value(serde_json::json!("shred")));

        // then
        assert_eq!(refused, Err("delete must be \"trash\" or \"permanent\", not \"shred\"".to_owned()));
    }
}
