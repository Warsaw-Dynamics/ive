//! The copies of `ive.rn` and the default `theme.toml` next to `init.rn`: there for the user to read, never loaded
//! and never read back.

use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

use super::engine::{IVE_RN, IVE_RN_NAME};
use crate::theme_config::DEFAULT_TEXT;

pub const THEME_NAME: &str = "theme.toml";

/// Writes `dir/ive.rn` and `dir/theme.toml` read-only, replacing whatever is there (an edit, an older IVE's copy, a
/// symlink, whose target is left alone), and creates `dir` (a new profile's config folder) when needed.
pub fn write(dir: &Path) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|error| format!("cannot create {}: {error}", dir.display()))?;
    replace(dir, IVE_RN_NAME, IVE_RN)?;
    replace(dir, THEME_NAME, &DEFAULT_TEXT)
}

fn replace(dir: &Path, name: &str, text: &str) -> Result<(), String> {
    let temporary = dir.join(temporary_name(name));
    write_new(&temporary, text).map_err(|error| format!("cannot write {}: {error}", temporary.display()))?;
    // a rename replaces the entry itself: a symlink there goes, its target stays as it was
    let path = dir.join(name);
    fs::rename(&temporary, &path).map_err(|error| {
        let _ = fs::remove_file(&temporary);
        format!("cannot replace {}: {error}", path.display())
    })
}

/// Per process, so two IVEs starting on one profile do not write the same file.
fn temporary_name(name: &str) -> String {
    format!(".{name}.{}.tmp", std::process::id())
}

/// `path` with `text`, read-only from the start; a leftover of a crashed write is removed first.
fn write_new(path: &Path, text: &str) -> std::io::Result<()> {
    match fs::remove_file(path) {
        Err(error) if error.kind() != ErrorKind::NotFound => return Err(error),
        _ => {}
    }
    OpenOptions::new().write(true).create_new(true).mode(0o444).open(path)?.write_all(text.as_bytes())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    use super::*;
    use crate::script::engine::IVE_RN;

    fn mode(path: &Path) -> u32 {
        fs::metadata(path).unwrap().permissions().mode() & 0o777
    }

    #[test]
    fn should_write_ive_rn_read_only() {
        // given
        let dir = tempfile::tempdir().unwrap();

        // when
        write(dir.path()).unwrap();

        // then
        let path = dir.path().join("ive.rn");
        assert_eq!(fs::read_to_string(&path).unwrap(), IVE_RN);
        assert_eq!(mode(&path), 0o444);
    }

    #[test]
    fn should_write_default_theme_read_only() {
        // given
        let dir = tempfile::tempdir().unwrap();

        // when
        write(dir.path()).unwrap();

        // then
        let path = dir.path().join("theme.toml");
        assert_eq!(fs::read_to_string(&path).unwrap(), *crate::theme_config::DEFAULT_TEXT);
        assert_eq!(mode(&path), 0o444);
    }

    #[test]
    fn should_create_missing_config_folder() {
        // given
        let parent = tempfile::tempdir().unwrap();
        let dir = parent.path().join("new-profile");

        // when
        write(&dir).unwrap();

        // then
        assert_eq!(fs::read_to_string(dir.join("ive.rn")).unwrap(), IVE_RN);
    }

    #[test]
    fn should_overwrite_edited_read_only_ive_rn() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ive.rn");
        fs::write(&path, "// the user's edit").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o444)).unwrap();

        // when
        write(dir.path()).unwrap();

        // then
        assert_eq!(fs::read_to_string(&path).unwrap(), IVE_RN);
        assert_eq!(mode(&path), 0o444);
    }

    #[test]
    fn should_replace_symlink_without_touching_its_target() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("dotfiles-ive.rn");
        fs::write(&target, "// kept").unwrap();
        let target_mode = mode(&target);
        std::os::unix::fs::symlink(&target, dir.path().join("ive.rn")).unwrap();

        // when
        write(dir.path()).unwrap();

        // then
        let path = dir.path().join("ive.rn");
        assert!(fs::symlink_metadata(&path).unwrap().file_type().is_file());
        assert_eq!(fs::read_to_string(&path).unwrap(), IVE_RN);
        assert_eq!((fs::read_to_string(&target).unwrap(), mode(&target)), ("// kept".to_owned(), target_mode));
    }

    #[test]
    fn should_replace_leftover_temporary_file() {
        // given
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(temporary_name("ive.rn")), "// a crashed write").unwrap();

        // when
        write(dir.path()).unwrap();

        // then
        assert_eq!(fs::read_to_string(dir.path().join("ive.rn")).unwrap(), IVE_RN);
        assert!(!dir.path().join(temporary_name("ive.rn")).exists());
    }
}
