//! The copies of `ive.rn` and the default `theme.toml` next to `init.rn`: there for the user to read, never loaded
//! and never read back.

use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Write};
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
    make_writable(&path).and_then(|()| fs::rename(&temporary, &path)).map_err(|error| {
        let _ = make_writable(&temporary).and_then(|()| fs::remove_file(&temporary));
        format!("cannot replace {}: {error}", path.display())
    })
}

/// Per process, so two IVEs starting on one profile do not write the same file.
fn temporary_name(name: &str) -> String {
    format!(".{name}.{}.tmp", std::process::id())
}

/// `path` with `text`, read-only from the start; a leftover of a crashed write is removed first.
fn write_new(path: &Path, text: &str) -> std::io::Result<()> {
    match make_writable(path).and_then(|()| fs::remove_file(path)) {
        Err(error) if error.kind() != ErrorKind::NotFound => return Err(error),
        _ => {}
    }
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    read_only(&mut options);
    options.open(path)?.write_all(text.as_bytes())
}

#[cfg(unix)]
fn read_only(options: &mut OpenOptions) {
    std::os::unix::fs::OpenOptionsExt::mode(options, 0o444);
}

/// The new file is read-only; the handle that created it may still write.
#[cfg(windows)]
fn read_only(options: &mut OpenOptions) {
    std::os::windows::fs::OpenOptionsExt::attributes(options, windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_READONLY);
}

/// Unix deletes and replaces read-only files as any other.
#[cfg(unix)]
fn make_writable(_path: &Path) -> std::io::Result<()> {
    Ok(())
}

/// Windows refuses to delete or replace a read-only file; a symlink is left as it is.
#[cfg(windows)]
fn make_writable(path: &Path) -> std::io::Result<()> {
    let metadata = match fs::symlink_metadata(path) {
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(()),
        metadata => metadata?,
    };
    let mut permissions = metadata.permissions();
    if metadata.is_file() && permissions.readonly() {
        // on Windows this clears the read-only attribute and nothing else
        #[allow(clippy::permissions_set_readonly_false)]
        permissions.set_readonly(false);
        return fs::set_permissions(path, permissions);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::script::engine::IVE_RN;

    /// `r--r--r--` on Unix, the read-only attribute on Windows.
    fn is_read_only(path: &Path) -> bool {
        let permissions = fs::metadata(path).unwrap().permissions();
        #[cfg(unix)]
        return std::os::unix::fs::PermissionsExt::mode(&permissions) & 0o777 == 0o444;
        #[cfg(windows)]
        permissions.readonly()
    }

    fn make_read_only(path: &Path) {
        let mut permissions = fs::metadata(path).unwrap().permissions();
        permissions.set_readonly(true);
        fs::set_permissions(path, permissions).unwrap();
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
        assert!(is_read_only(&path));
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
        assert!(is_read_only(&path));
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
        make_read_only(&path);

        // when
        write(dir.path()).unwrap();

        // then
        assert_eq!(fs::read_to_string(&path).unwrap(), IVE_RN);
        assert!(is_read_only(&path));
    }

    #[cfg(unix)]
    #[test]
    fn should_replace_symlink_without_touching_its_target() {
        // given
        use std::os::unix::fs::PermissionsExt;
        let mode = |path: &Path| fs::metadata(path).unwrap().permissions().mode() & 0o777;
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
        let leftover = dir.path().join(temporary_name("ive.rn"));
        fs::write(&leftover, "// a crashed write").unwrap();
        make_read_only(&leftover);

        // when
        write(dir.path()).unwrap();

        // then
        assert_eq!(fs::read_to_string(dir.path().join("ive.rn")).unwrap(), IVE_RN);
        assert!(!dir.path().join(temporary_name("ive.rn")).exists());
    }
}
