//! Paths as IVE shows them.

use std::path::{Path, PathBuf};

/// `path` with the home folder written as `~`, e.g. `~/Projects/atlassian`.
pub fn tilde(path: &Path, home: Option<&Path>) -> String {
    match home.and_then(|home| path.strip_prefix(home).ok()) {
        Some(rest) if rest.as_os_str().is_empty() => "~".to_owned(),
        Some(rest) => format!("~/{}", slashed(rest)),
        None => path.display().to_string(),
    }
}

/// A relative path with `/` between its parts on every OS, as git and the hub write them.
pub fn slashed(relative: &Path) -> String {
    relative.iter().map(|part| part.to_string_lossy()).collect::<Vec<_>>().join("/")
}

/// `base` with a [`slashed`] relative path after it, in the OS's separators.
pub fn joined(base: &Path, relative: &str) -> PathBuf {
    relative.split('/').filter(|part| !part.is_empty()).fold(base.to_path_buf(), |path, part| path.join(part))
}

/// A test path written as on Unix (`/a/b`), made absolute on this OS: on drive `C:` on Windows.
#[cfg(test)]
pub fn absolute(path: &str) -> PathBuf {
    if cfg!(windows) {
        return PathBuf::from(format!("C:{path}"));
    }
    PathBuf::from(path)
}

/// Whether `a` and `b` name the same place as the OS compares paths.
#[cfg(unix)]
pub fn same(a: &Path, b: &Path) -> bool {
    a == b
}

/// Windows compares names without case.
#[cfg(windows)]
pub fn same(a: &Path, b: &Path) -> bool {
    let folded = |path: &Path| path.components().map(|part| part.as_os_str().to_string_lossy().to_lowercase()).collect::<Vec<_>>();
    folded(a) == folded(b)
}

/// `path` with this user's home folder written as `~`.
pub fn tilde_home(path: &Path) -> String {
    tilde(path, dirs::home_dir().as_deref())
}

/// Where a worktree card says a checkout is: relative to the main checkout's parent when below it
/// (`atlassian/.claude/worktrees/GEN-817-ive`), else `~`-shortened.
pub fn card_folder(checkout: &Path, main: &Path, home: Option<&Path>) -> String {
    match main.parent().and_then(|parent| checkout.strip_prefix(parent).ok()) {
        Some(relative) => slashed(relative),
        None => tilde(checkout, home),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_shorten_home_to_tilde() {
        // given
        let home = Some(Path::new("/home/kb"));

        // when
        let shown = ["/home/kb/Projects/atlassian", "/home/kb", "/mnt/data"].map(|path| tilde(Path::new(path), home));

        // then
        assert_eq!(shown, ["~/Projects/atlassian", "~", "/mnt/data"]);
    }

    #[test]
    fn should_show_worktree_folder_relative_to_main_checkout_parent() {
        // given
        let main = Path::new("/mnt/data/Projects/atlassian");
        let home = Some(Path::new("/home/kb"));

        // when
        let shown = [
            card_folder(Path::new("/mnt/data/Projects/atlassian/.claude/worktrees/GEN-817-ive"), main, home),
            card_folder(main, main, home),
            card_folder(Path::new("/home/kb/elsewhere/wt"), main, home),
        ];

        // then
        assert_eq!(shown, ["atlassian/.claude/worktrees/GEN-817-ive", "atlassian", "~/elsewhere/wt"]);
    }

    #[test]
    fn should_put_a_slash_between_the_parts_on_every_os() {
        // given
        let relative = Path::new("src").join("hub").join("view.rs");

        // when
        let shown = slashed(&relative);

        // then
        assert_eq!(shown, "src/hub/view.rs");
    }

    #[cfg(windows)]
    #[test]
    fn should_take_paths_differing_in_case_or_separator_for_the_same() {
        // given
        let (written, canonical, other) = (Path::new("c:/users/kb/Atlassian"), Path::new(r"C:\Users\kb\atlassian"), Path::new(r"C:\Users\kb\other"));

        // when
        let compared = [same(written, canonical), same(written, other)];

        // then
        assert_eq!(compared, [true, false]);
    }
}
