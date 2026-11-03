//! `config.folder_switches`: where a worktree's folders (Processes, Uncommitted, Since base, Commits, Files) are
//! switched on and off: in every worktree at once, and in one worktree from badges on its row.

use gpui::{App, Global};
use serde::Deserialize;

/// What switches folders; a key `init.rn` leaves out keeps its default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct FolderSwitches {
    /// the folders menu's ticks in the sidebar header, `act::ToggleFolderEverywhere` and Alt+click on a badge
    pub every_worktree: bool,
    /// a badge per folder on a worktree's row, shown on hover: a click switches the folder in that worktree alone
    pub per_worktree: bool,
}

impl Default for FolderSwitches {
    fn default() -> Self {
        Self { every_worktree: true, per_worktree: false }
    }
}

impl Global for FolderSwitches {}

/// `config.folder_switches.every_worktree`; on before the scripts first load.
pub fn switches_every_worktree(cx: &App) -> bool {
    cx.try_global::<FolderSwitches>().is_none_or(|switches| switches.every_worktree)
}

/// `value` as `config.folder_switches` left it; the error names the offending value.
pub fn parse(value: &rune::Value) -> Result<FolderSwitches, String> {
    let json = serde_json::to_value(value).map_err(|error| format!("folder_switches: {error}"))?;
    serde_json::from_value(json.clone())
        .map_err(|error| format!("folder_switches must be #{{ every_worktree: true, per_worktree: false }} with true or false, not {json} ({error})"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn value(json: serde_json::Value) -> rune::Value {
        serde_json::from_value(json).unwrap()
    }

    #[test]
    fn should_take_both_switches() {
        // given
        let switches = value(serde_json::json!({ "every_worktree": false, "per_worktree": true }));

        // when
        let parsed = parse(&switches);

        // then
        assert_eq!(parsed, Ok(FolderSwitches { every_worktree: false, per_worktree: true }));
    }

    #[test]
    fn should_keep_the_default_of_a_switch_left_out() {
        // given
        let switches = value(serde_json::json!({ "per_worktree": true }));

        // when
        let parsed = parse(&switches);

        // then
        assert_eq!(parsed, Ok(FolderSwitches { every_worktree: true, per_worktree: true }));
    }

    #[test]
    fn should_reject_an_unknown_switch_or_a_non_boolean() {
        // given
        let values = [serde_json::json!({ "per_worktrees": true }), serde_json::json!({ "per_worktree": "yes" }), serde_json::json!(true)];

        // when
        let errors: Vec<String> = values.iter().map(|json| parse(&value(json.clone())).unwrap_err()).collect();

        // then
        for (error, json) in errors.iter().zip(&values) {
            assert!(error.starts_with(&format!("folder_switches must be #{{ every_worktree: true, per_worktree: false }} with true or false, not {json} (")), "{error}");
        }
    }
}
