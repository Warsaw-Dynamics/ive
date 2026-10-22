//! Paths typed into a terminal: relative to the terminal's directory when the terminal works in the checkout of
//! every path, else absolute; quoted for the shell.

use std::path::{Component, Path, PathBuf};

use crate::pasted::shell_word;

/// What typed paths are written against.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Base<'a> {
    /// relative to this directory, `../` allowed
    Relative(&'a Path),
    Absolute,
}

/// The base for paths all in `checkout_of_paths` (`None`: no one checkout has them all), typed into a terminal of
/// `terminal_checkout` working in `directory` (`None`: not known yet, so its checkout's root stands in).
pub fn base_for<'a>(checkout_of_paths: Option<&'a Path>, terminal_checkout: Option<&'a Path>, directory: Option<&'a Path>) -> Base<'a> {
    let Some(checkout) = checkout_of_paths.filter(|checkout| terminal_checkout == Some(*checkout)) else {
        return Base::Absolute;
    };
    match directory {
        None => Base::Relative(checkout),
        Some(directory) if directory.starts_with(checkout) => Base::Relative(directory),
        Some(_) => Base::Absolute,
    }
}

/// `paths` as shell words against `base`, separated by spaces, with a space after the last.
pub fn typed_paths(paths: &[PathBuf], base: Base) -> String {
    let written = paths.iter().map(|path| match base {
        Base::Relative(base) => relative(path, base),
        Base::Absolute => path.clone(),
    });
    written.map(|path| format!("{} ", shell_word(&path))).collect()
}

/// `path` relative to `base`, both absolute, with `..` for each folder of `base` it is not in; `.` for `base` itself.
pub fn relative(path: &Path, base: &Path) -> PathBuf {
    let (path_parts, base_parts): (Vec<Component>, Vec<Component>) = (path.components().collect(), base.components().collect());
    let shared = path_parts.iter().zip(&base_parts).take_while(|(a, b)| a == b).count();
    let ups = std::iter::repeat_n(Component::ParentDir, base_parts.len() - shared);
    let relative: PathBuf = ups.chain(path_parts[shared..].iter().copied()).collect();
    if relative.as_os_str().is_empty() {
        return PathBuf::from(".");
    }
    relative
}

/// `paths` as [`typed_paths`] types them into a terminal of `terminal` (its checkout, its directory when known),
/// against the open `checkouts`.
pub fn typed_into(paths: &[PathBuf], checkouts: &[PathBuf], terminal: (Option<&Path>, Option<&Path>)) -> String {
    let home = checkout_of(paths, checkouts.iter().map(PathBuf::as_path));
    typed_paths(paths, base_for(home.as_deref(), terminal.0, terminal.1))
}

/// The innermost of `checkouts` that holds every one of `paths`; `None` when there are no paths or no one checkout
/// has them all.
pub fn checkout_of<'a>(paths: &[PathBuf], checkouts: impl IntoIterator<Item = &'a Path>) -> Option<PathBuf> {
    let checkouts: Vec<&Path> = checkouts.into_iter().collect();
    let innermost = |path: &PathBuf| checkouts.iter().filter(|checkout| path.starts_with(checkout)).max_by_key(|checkout| checkout.components().count()).copied();
    let mut each = paths.iter().map(innermost);
    let first = each.next()??;
    each.all(|checkout| checkout == Some(first)).then(|| first.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paths(list: &[&str]) -> Vec<PathBuf> {
        list.iter().map(PathBuf::from).collect()
    }

    fn typed(list: &[&str], checkout: Option<&str>, terminal_checkout: Option<&str>, directory: Option<&str>) -> String {
        let base = base_for(checkout.map(Path::new), terminal_checkout.map(Path::new), directory.map(Path::new));
        typed_paths(&paths(list), base)
    }

    #[test]
    fn should_type_paths_relative_to_terminal_directory() {
        // when
        let text = typed(&["/w/src/a.rs", "/w/README.md"], Some("/w"), Some("/w"), Some("/w/src"));

        // then
        assert_eq!(text, "a.rs ../README.md ");
    }

    #[test]
    fn should_use_checkout_root_when_directory_unknown() {
        // when
        let text = typed(&["/w/src/a.rs", "/w/README.md"], Some("/w"), Some("/w"), None);

        // then
        assert_eq!(text, "src/a.rs README.md ");
    }

    #[test]
    fn should_type_absolute_paths_when_terminal_is_in_another_checkout() {
        // when
        let text = typed(&["/w/src/a.rs", "/w/README.md"], Some("/w"), Some("/x"), Some("/x"));

        // then
        assert_eq!(text, "/w/src/a.rs /w/README.md ");
    }

    #[test]
    fn should_type_absolute_paths_when_directory_left_the_checkout() {
        // when
        let text = typed(&["/w/src/a.rs"], Some("/w"), Some("/w"), Some("/tmp"));

        // then
        assert_eq!(text, "/w/src/a.rs ");
    }

    #[test]
    fn should_type_absolute_paths_for_paths_in_several_checkouts() {
        // when
        let text = typed(&["/w/a.rs", "/x/b.rs"], None, Some("/w"), Some("/w"));

        // then
        assert_eq!(text, "/w/a.rs /x/b.rs ");
    }

    #[test]
    fn should_quote_paths_with_spaces_and_quotes() {
        // when
        let text = typed(&["/w/my notes/it's.md"], Some("/w"), Some("/w"), Some("/w"));

        // then
        assert_eq!(text, r"'my notes/it'\''s.md' ");
    }

    #[test]
    fn should_quote_a_path_starting_with_a_tilde() {
        // when
        let text = typed(&["/w/~draft"], Some("/w"), Some("/w"), Some("/w"));

        // then
        assert_eq!(text, "'~draft' ");
    }

    #[test]
    fn should_name_the_directory_itself_dot() {
        // when
        let text = typed(&["/w/src"], Some("/w"), Some("/w"), Some("/w/src"));

        // then
        assert_eq!(text, ". ");
    }

    #[test]
    fn should_find_the_innermost_checkout_of_all_paths() {
        // given
        let checkouts = [Path::new("/w"), Path::new("/w/.worktree/b")];

        // when
        let inner = checkout_of(&paths(&["/w/.worktree/b/a.rs", "/w/.worktree/b/c"]), checkouts);
        let mixed = checkout_of(&paths(&["/w/.worktree/b/a.rs", "/w/a.rs"]), checkouts);

        // then
        assert_eq!((inner, mixed), (Some(PathBuf::from("/w/.worktree/b")), None));
    }

    #[test]
    fn should_type_a_dropped_tree_selection_relative_to_the_terminal() {
        // given
        let checkouts = [PathBuf::from("/w"), PathBuf::from("/x")];
        let dropped = paths(&["/w/src/a.rs", "/w/docs"]);

        // when
        let text = typed_into(&dropped, &checkouts, (Some(Path::new("/w")), Some(Path::new("/w/src"))));

        // then
        assert_eq!(text, "a.rs ../docs ");
    }

    #[test]
    fn should_find_no_checkout_for_no_paths() {
        // when
        let found = checkout_of(&[], [Path::new("/w")]);

        // then
        assert_eq!(found, None);
    }
}
