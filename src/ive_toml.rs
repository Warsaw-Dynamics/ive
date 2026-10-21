//! `.ive.toml` in a project's main checkout: where New worktree puts worktrees.

use std::fs;
use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};

use serde::Deserialize;

pub const FILE_NAME: &str = ".ive.toml";
pub const DEFAULT_WORKTREES: &str = ".worktree";

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct IveToml {
    worktrees: Option<String>,
}

/// The folder new worktrees go in, relative to `main_dir`: `worktrees` of its `.ive.toml`, else [`DEFAULT_WORKTREES`].
/// A file that does not parse, has other keys, or names a folder outside the checkout is refused.
pub fn worktrees_folder(main_dir: &Path) -> Result<PathBuf, String> {
    let file = main_dir.join(FILE_NAME);
    let text = match fs::read_to_string(&file) {
        Ok(text) => text,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(PathBuf::from(DEFAULT_WORKTREES)),
        Err(error) => return Err(format!("cannot read {}: {error}", file.display())),
    };
    let read: IveToml = toml::from_str(&text).map_err(|error| format!("{}: {}", file.display(), error.message()))?;
    let Some(folder) = read.worktrees else {
        return Ok(PathBuf::from(DEFAULT_WORKTREES));
    };
    let path = PathBuf::from(&folder);
    let inside = !folder.is_empty() && path.components().all(|component| matches!(component, Component::Normal(_) | Component::CurDir));
    if inside {
        return Ok(path);
    }
    Err(format!("{}: worktrees must be a folder inside the main checkout, such as \"{DEFAULT_WORKTREES}\", not \"{folder}\"", file.display()))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    fn checkout_with(text: &str) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(FILE_NAME), text).unwrap();
        dir
    }

    #[test]
    fn should_default_to_dot_worktree_without_a_file() {
        // given
        let dir = tempfile::tempdir().unwrap();

        // when
        let folder = worktrees_folder(dir.path());

        // then
        assert_eq!(folder, Ok(PathBuf::from(".worktree")));
    }

    #[test]
    fn should_default_to_dot_worktree_without_the_key() {
        // given
        let dir = checkout_with("# nothing set\n");

        // when
        let folder = worktrees_folder(dir.path());

        // then
        assert_eq!(folder, Ok(PathBuf::from(".worktree")));
    }

    #[test]
    fn should_read_the_worktrees_folder() {
        // given
        let dir = checkout_with("worktrees = \".claude/worktrees\"\n");

        // when
        let folder = worktrees_folder(dir.path());

        // then
        assert_eq!(folder, Ok(PathBuf::from(".claude/worktrees")));
    }

    #[test]
    fn should_reject_a_path_leaving_the_checkout() {
        // given
        let dir = checkout_with("worktrees = \"../x\"\n");

        // when
        let error = worktrees_folder(dir.path()).unwrap_err();

        // then
        assert!(error.contains(".ive.toml") && error.contains("\"../x\""), "{error}");
    }

    #[test]
    fn should_reject_an_absolute_path() {
        // given
        let dir = checkout_with("worktrees = \"/tmp/x\"\n");

        // when
        let error = worktrees_folder(dir.path()).unwrap_err();

        // then
        assert!(error.contains("\"/tmp/x\""), "{error}");
    }

    #[test]
    fn should_reject_an_empty_path() {
        // given
        let dir = checkout_with("worktrees = \"\"\n");

        // when
        let error = worktrees_folder(dir.path()).unwrap_err();

        // then
        assert!(error.contains("worktrees"), "{error}");
    }

    #[test]
    fn should_reject_unknown_keys() {
        // given
        let dir = checkout_with("worktree = \"wt\"\n");

        // when
        let error = worktrees_folder(dir.path()).unwrap_err();

        // then
        assert!(error.contains(".ive.toml") && error.contains("worktree"), "{error}");
    }
}
