//! Paths as IVE shows them.

use std::path::Path;

/// `path` with the home folder written as `~`, e.g. `~/Projects/atlassian`.
pub fn tilde(path: &Path, home: Option<&Path>) -> String {
    match home.and_then(|home| path.strip_prefix(home).ok()) {
        Some(rest) if rest.as_os_str().is_empty() => "~".to_owned(),
        Some(rest) => format!("~/{}", rest.display()),
        None => path.display().to_string(),
    }
}

/// `path` with this user's home folder written as `~`.
pub fn tilde_home(path: &Path) -> String {
    tilde(path, dirs::home_dir().as_deref())
}

/// Where a worktree card says a checkout is: relative to the main checkout's parent when below it
/// (`atlassian/.claude/worktrees/GEN-817-ive`), else `~`-shortened.
pub fn card_folder(checkout: &Path, main: &Path, home: Option<&Path>) -> String {
    match main.parent().and_then(|parent| checkout.strip_prefix(parent).ok()) {
        Some(relative) => relative.display().to_string(),
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
}
