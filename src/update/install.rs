//! Where this IVE is installed, and swapping a staged release in for it.

use std::fs;
use std::path::{Path, PathBuf};

/// The staging folder's name, beside the install so the swap is a rename on one file system.
pub const STAGING_NAME: &str = ".ive-update";

/// Where the old `.app` waits, inside the staging folder, between the two renames of a swap.
const OLD_APP: &str = "old.app";

/// Files of a Linux or Windows release beside the binary; a swap refreshes those the install already has.
const BESIDE_BINARY: [&str; 4] = ["LICENSE", "NOTICE", "THIRD-PARTY-NOTICES.md", "licenses"];

/// Programs of a Windows release beside `ive.exe`, which terminal hosts run and load while IVE is gone; a swap moves
/// each in as it moves `ive.exe` in.
#[cfg(windows)]
const PROGRAMS_BESIDE: [&str; 3] = ["ive-cli.exe", "conpty.dll", "OpenConsole.exe"];
#[cfg(not(windows))]
const PROGRAMS_BESIDE: [&str; 0] = [];

/// What a release replaces: the executable, or on macOS the app bundle around it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Install {
    Binary(PathBuf),
    App(PathBuf),
}

impl Install {
    /// The install `exe` belongs to: the `.app` when it runs from `X.app/Contents/MacOS/`, the executable otherwise.
    pub fn of_exe(exe: &Path) -> Self {
        let app = exe.parent().filter(|dir| dir.ends_with("Contents/MacOS")).and_then(Path::parent).and_then(Path::parent);
        match app {
            Some(app) if app.extension().is_some_and(|extension| extension == "app") => Self::App(app.to_path_buf()),
            _ => Self::Binary(exe.to_path_buf()),
        }
    }

    pub fn path(&self) -> &Path {
        match self {
            Self::Binary(path) | Self::App(path) => path,
        }
    }

    fn parent(&self) -> &Path {
        self.path().parent().unwrap_or(Path::new("/"))
    }

    pub fn staging_dir(&self) -> PathBuf {
        self.parent().join(STAGING_NAME)
    }

    /// Whether IVE may create and rename files beside the install, as staging and swapping do.
    pub fn check_writable(&self) -> Result<(), String> {
        let probe = self.parent().join(format!(".ive-update-probe-{}", std::process::id()));
        fs::write(&probe, "").map_err(|error| format!("cannot update: {} is not writable ({error})", self.parent().display()))?;
        fs::remove_file(&probe).map_err(|error| format!("cannot remove {}: {error}", probe.display()))
    }
}

/// Puts the release staged at `staged` (an executable or a `.app`, as `install` is) in place of `install`.
pub fn swap(install: &Install, staged: &Path) -> Result<(), String> {
    match install {
        Install::Binary(current) => swap_binary(current, staged),
        Install::App(current) => swap_app(current, staged, &install.staging_dir().join(OLD_APP)),
    }
}

fn swap_binary(current: &Path, staged: &Path) -> Result<(), String> {
    replace_program(staged, current)?;
    let (Some(from), Some(to)) = (staged.parent(), current.parent()) else {
        return Ok(());
    };
    for name in PROGRAMS_BESIDE.iter().filter(|name| from.join(name).exists()) {
        replace_program(&from.join(name), &to.join(name))?;
    }
    for name in BESIDE_BINARY.iter().filter(|name| to.join(name).exists() && from.join(name).exists()) {
        replace_beside(&from.join(name), &to.join(name))?;
    }
    Ok(())
}

fn replace_program(staged: &Path, current: &Path) -> Result<(), String> {
    if current.exists() {
        set_aside_running(current)?;
    }
    rename(staged, current)
}

/// Windows cannot replace a running program, only rename it: to a name of this swap's own, as terminal hosts may
/// still run one an earlier swap set aside. The next start deletes those no process runs any more.
#[cfg(windows)]
fn set_aside_running(current: &Path) -> Result<(), String> {
    let name = current.file_name().map(|name| name.to_string_lossy()).unwrap_or_default();
    rename(current, &current.with_file_name(format!("{name}.{}.old", std::process::id())))
}

/// Deletes the programs swaps set aside beside this executable, except those still running.
#[cfg(windows)]
pub fn delete_set_aside_programs() {
    let Some(dir) = std::env::current_exe().ok().and_then(|exe| exe.parent().map(Path::to_path_buf)) else {
        return;
    };
    let Ok(entries) = fs::read_dir(&dir) else {
        return;
    };
    for path in entries.filter_map(Result::ok).map(|entry| entry.path()).filter(|path| is_set_aside(path)) {
        // a running one is locked; a later start deletes it
        let _ = fs::remove_file(path);
    }
}

#[cfg(windows)]
fn is_set_aside(path: &Path) -> bool {
    let name = path.file_name().map(|name| name.to_string_lossy()).unwrap_or_default();
    let program = |program: &&str| name.strip_prefix(program).is_some_and(|rest| rest.starts_with('.'));
    name.ends_with(".old") && ["ive.exe"].iter().chain(PROGRAMS_BESIDE.iter()).any(program)
}

/// Unix renames over a running executable; the running process keeps the old file.
#[cfg(not(windows))]
fn set_aside_running(_current: &Path) -> Result<(), String> {
    Ok(())
}

fn replace_beside(from: &Path, to: &Path) -> Result<(), String> {
    let removed = if to.is_dir() { fs::remove_dir_all(to) } else { fs::remove_file(to) };
    removed.map_err(|error| format!("cannot replace {}: {error}", to.display()))?;
    rename(from, to)
}

/// Two renames, with the first undone when the second fails, so the install is never missing.
fn swap_app(current: &Path, staged: &Path, old: &Path) -> Result<(), String> {
    if old.exists() {
        fs::remove_dir_all(old).map_err(|error| format!("cannot remove {}: {error}", old.display()))?;
    }
    rename(current, old)?;
    if let Err(error) = rename(staged, current) {
        let _ = fs::rename(old, current);
        return Err(error);
    }
    // the swap is done; an old app left behind only takes space until the next swap removes it
    let _ = fs::remove_dir_all(old);
    Ok(())
}

fn rename(from: &Path, to: &Path) -> Result<(), String> {
    fs::rename(from, to).map_err(|error| format!("cannot move {} to {}: {error}", from.display(), to.display()))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use super::*;

    fn write(path: &Path, text: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    #[test]
    fn should_see_an_app_around_a_macos_executable() {
        // given
        let exe = Path::new("/Applications/IVE.app/Contents/MacOS/ive");

        // when
        let install = Install::of_exe(exe);

        // then
        assert_eq!(install, Install::App(PathBuf::from("/Applications/IVE.app")));
        assert_eq!(install.staging_dir(), PathBuf::from("/Applications/.ive-update"));
    }

    #[test]
    fn should_see_a_bare_binary() {
        // given
        let exe = Path::new("/home/me/opt/ive/ive");

        // when
        let install = Install::of_exe(exe);

        // then
        assert_eq!(install, Install::Binary(PathBuf::from("/home/me/opt/ive/ive")));
        assert_eq!(install.staging_dir(), PathBuf::from("/home/me/opt/ive/.ive-update"));
    }

    #[test]
    fn should_find_a_writable_install_folder_writable() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let install = Install::Binary(dir.path().join("ive"));

        // when
        let writable = install.check_writable();

        // then
        assert_eq!(writable, Ok(()));
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 0);
    }

    #[test]
    fn should_swap_a_binary() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let current = dir.path().join("ive");
        write(&current, "old");
        write(&dir.path().join("LICENSE"), "old licence");
        let staged = dir.path().join(".ive-update/extracted/ive-linux-x86_64-abc/ive");
        write(&staged, "new");
        write(&staged.with_file_name("LICENSE"), "new licence");
        write(&staged.with_file_name("NOTICE"), "new notice");

        // when
        let swapped = swap(&Install::Binary(current.clone()), &staged);

        // then
        assert_eq!(swapped, Ok(()));
        assert_eq!(fs::read_to_string(&current).unwrap(), "new");
        assert_eq!(fs::read_to_string(dir.path().join("LICENSE")).unwrap(), "new licence");
        assert!(!dir.path().join("NOTICE").exists());
    }

    #[cfg(windows)]
    #[test]
    fn should_swap_the_programs_of_a_windows_release_and_delete_those_set_aside() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let current = dir.path().join("ive.exe");
        write(&current, "old");
        write(&dir.path().join("ive-cli.exe"), "old cli");
        let staged = dir.path().join(".ive-update/extracted/ive-windows-x86_64-abc/ive.exe");
        for name in ["ive.exe", "ive-cli.exe", "conpty.dll", "OpenConsole.exe"] {
            write(&staged.with_file_name(name), &format!("new {name}"));
        }

        // when
        let swapped = swap(&Install::Binary(current.clone()), &staged);

        // then
        assert_eq!(swapped, Ok(()));
        for name in ["ive.exe", "ive-cli.exe", "conpty.dll", "OpenConsole.exe"] {
            assert_eq!(fs::read_to_string(dir.path().join(name)).unwrap(), format!("new {name}"));
        }
        let set_aside = dir.path().join(format!("ive-cli.exe.{}.old", std::process::id()));
        assert_eq!(fs::read_to_string(&set_aside).unwrap(), "old cli");
        assert!(is_set_aside(&set_aside));
        assert!(!is_set_aside(&dir.path().join("ive-cli.exe")));
    }

    #[test]
    fn should_swap_an_app() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let current = dir.path().join("IVE.app");
        write(&current.join("Contents/MacOS/ive"), "old");
        let staged = dir.path().join(".ive-update/extracted/ive-macos-arm64-abc/IVE.app");
        write(&staged.join("Contents/MacOS/ive"), "new");

        // when
        let swapped = swap(&Install::App(current.clone()), &staged);

        // then
        assert_eq!(swapped, Ok(()));
        assert_eq!(fs::read_to_string(current.join("Contents/MacOS/ive")).unwrap(), "new");
        assert!(!dir.path().join(".ive-update").join(OLD_APP).exists());
    }

    #[test]
    fn should_restore_the_old_app_when_the_new_one_cannot_move_in() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let current = dir.path().join("IVE.app");
        write(&current.join("Contents/MacOS/ive"), "old");
        fs::create_dir_all(dir.path().join(".ive-update")).unwrap();
        let staged = dir.path().join(".ive-update/extracted/missing/IVE.app");

        // when
        let swapped = swap(&Install::App(current.clone()), &staged);

        // then
        assert!(swapped.unwrap_err().contains("IVE.app"));
        assert_eq!(fs::read_to_string(current.join("Contents/MacOS/ive")).unwrap(), "old");
    }
}
