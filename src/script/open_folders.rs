//! `config.open_folders`: the folders of a worktree in the tree that start open; the others start folded, until the
//! user unfolds them.

use std::collections::BTreeSet;

use gpui::Global;
use serde_json::Value;

use crate::sidebar::folders::FolderKind;

const NAMES: &str = "\"processes\", \"uncommitted\", \"since-base\", \"commits\" or \"files\"";

/// Kept as the folders left out, so that everything open needs no list.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OpenFolders {
    folded: BTreeSet<FolderKind>,
}

impl Global for OpenFolders {}

/// Every folder open: what the tree shows before the scripts first load.
pub static ALL_OPEN: OpenFolders = OpenFolders { folded: BTreeSet::new() };

impl OpenFolders {
    pub fn is_open(&self, kind: FolderKind) -> bool {
        !self.folded.contains(&kind)
    }
}

/// `value` as `config.open_folders` left it; the error names the offending value.
pub fn parse(value: &rune::Value) -> Result<OpenFolders, String> {
    let json = serde_json::to_value(value).map_err(|error| format!("open_folders: {error}"))?;
    let Value::Array(items) = json else {
        return Err(format!("open_folders must be a list of folders such as [\"processes\"], not {json}"));
    };
    let mut open = BTreeSet::new();
    for item in items {
        let kind: FolderKind = serde_json::from_value(item.clone()).map_err(|_| format!("open_folders: {item} is not {NAMES}"))?;
        if !open.insert(kind) {
            return Err(format!("open_folders: {item} is listed twice"));
        }
    }
    Ok(OpenFolders { folded: FolderKind::ALL.into_iter().filter(|kind| !open.contains(kind)).collect() })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn value(json: serde_json::Value) -> rune::Value {
        serde_json::from_value(json).unwrap()
    }

    #[test]
    fn should_open_the_listed_folders_alone() {
        // given
        let list = value(serde_json::json!(["processes", "since-base"]));

        // when
        let open = parse(&list).unwrap();

        // then
        assert!(open.is_open(FolderKind::Processes) && open.is_open(FolderKind::SinceBase));
        assert!(!open.is_open(FolderKind::Files));
    }

    #[test]
    fn should_reject_an_unknown_or_repeated_folder_and_other_values() {
        // given
        let values = [serde_json::json!(["Files"]), serde_json::json!(["files", "files"]), serde_json::json!("files")];

        // when
        let errors: Vec<String> = values.iter().map(|json| parse(&value(json.clone())).unwrap_err()).collect();

        // then
        assert_eq!(errors, [
            format!("open_folders: \"Files\" is not {NAMES}"),
            "open_folders: \"files\" is listed twice".to_owned(),
            "open_folders must be a list of folders such as [\"processes\"], not \"files\"".to_owned(),
        ]);
    }
}
