//! Renaming and deleting files and folders from the tree: the checks a new name passes, the paths that are not
//! IVE's to touch (checkouts, git's own folder), and the operations themselves.

use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt as _;
use std::path::{Path, PathBuf};

use crate::menu::PathKind;
use crate::script::delete_mode::DeleteMode;

/// Where renaming `old` to `name` puts it, beside `old`; refused for a name that is empty, `.` or `..`, holds a `/`
/// or a NUL, or is `old`'s own. Surrounding spaces are dropped.
pub fn checked_name(old: &Path, name: &str) -> Result<PathBuf, String> {
    let name = name.trim();
    if name.is_empty() || name == "." || name == ".." {
        return Err(format!("Rename refused: {name:?} is not a name"));
    }
    if name.contains('/') || name.contains('\0') {
        return Err(format!("Rename refused: {name} has a /; drag the file to move it"));
    }
    if old.file_name().is_some_and(|current| current == name) {
        return Err(format!("Rename refused: the name is {name} already"));
    }
    Ok(old.with_file_name(name))
}

/// Where the rename field's selection ends in `name`: before a file's last extension, at the end of a folder's name
/// and of a name that starts with its only dot.
pub fn stem_end(name: &str, kind: PathKind) -> usize {
    match (kind, name.rfind('.')) {
        (PathKind::File, Some(dot)) if dot > 0 => dot,
        _ => name.len(),
    }
}

/// Why `path` is not renamed or deleted from the tree: it is one of `checkouts`' roots, holds one of them, or is
/// git's own `.git`; `None` for any other path.
pub fn protected(path: &Path, checkouts: &[PathBuf]) -> Option<String> {
    if checkouts.iter().any(|checkout| checkout == path) {
        return Some(format!("{} is a checkout's root", path.display()));
    }
    if checkouts.iter().any(|checkout| checkout.starts_with(path)) {
        return Some(format!("{} holds an open worktree", path.display()));
    }
    if path.file_name().is_some_and(|name| name == ".git") {
        return Some(format!("{} is git's own", path.display()));
    }
    None
}

/// Renames `from` to `to`, never replacing what is there; a change of case alone (which a case-insensitive file
/// system sees as the same name) goes through a temporary name.
pub fn rename(from: &Path, to: &Path) -> Result<(), String> {
    let failed = |error: io::Error| match error.kind() {
        io::ErrorKind::AlreadyExists => format!("Rename refused: {} is taken in {}", name_of(to), folder_of(to)),
        _ => format!("Could not rename {} to {}: {error}", from.display(), name_of(to)),
    };
    if is_case_change(from, to) {
        let between = from.with_file_name(format!(".{}.ive-rename-{}", name_of(from), std::process::id()));
        crate::rename::rename_new(from, &between).map_err(failed)?;
        return crate::rename::rename_new(&between, to).map_err(|error| {
            // the file goes back under its name rather than stay hidden under the temporary one
            let _ = crate::rename::rename_new(&between, from);
            failed(error)
        });
    }
    crate::rename::rename_new(from, to).map_err(failed)
}

/// Whether `to` is `from` with only the case of its name changed, on a file system that sees both as one file.
fn is_case_change(from: &Path, to: &Path) -> bool {
    let lower = |path: &Path| name_of(path).to_lowercase();
    from.parent() == to.parent() && lower(from) == lower(to) && is_same_file(from, to)
}

/// Deletes `path` as `mode` says: to the system trash, or removed with everything in it. A symbolic link goes
/// itself; what it points at stays.
pub fn delete(path: &Path, mode: DeleteMode) -> Result<(), String> {
    let failed = |error: String| format!("Could not delete {}: {error}", path.display());
    match mode {
        DeleteMode::Trash => trash::delete(path).map_err(|error| failed(error.to_string())),
        DeleteMode::Permanent => remove(path).map_err(|error| failed(error.to_string())),
    }
}

fn remove(path: &Path) -> io::Result<()> {
    match fs::symlink_metadata(path)?.is_dir() {
        true => fs::remove_dir_all(path),
        false => fs::remove_file(path),
    }
}

/// Whether `a` and `b` name one file: the same device and inode.
fn is_same_file(a: &Path, b: &Path) -> bool {
    match (fs::symlink_metadata(a), fs::symlink_metadata(b)) {
        (Ok(a), Ok(b)) => (a.dev(), a.ino()) == (b.dev(), b.ino()),
        _ => false,
    }
}

fn name_of(path: &Path) -> String {
    path.file_name().map_or_else(|| path.display().to_string(), |name| name.to_string_lossy().into_owned())
}

fn folder_of(path: &Path) -> String {
    path.parent().map_or_else(String::new, |folder| folder.display().to_string())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use super::*;
    use crate::menu::PathKind;

    #[test]
    fn should_refuse_names_with_a_slash() {
        // when
        let refused = checked_name(Path::new("/r/src/a.rs"), "b/c.rs");

        // then
        assert_eq!(refused, Err("Rename refused: b/c.rs has a /; drag the file to move it".to_owned()));
    }

    #[test]
    fn should_refuse_dot_and_dot_dot() {
        // when
        let refused = [".", "..", "  "].map(|name| checked_name(Path::new("/r/a.rs"), name).is_err());

        // then
        assert_eq!(refused, [true, true, true]);
    }

    #[test]
    fn should_refuse_the_same_name() {
        // when
        let refused = checked_name(Path::new("/r/a.rs"), "a.rs");

        // then
        assert_eq!(refused, Err("Rename refused: the name is a.rs already".to_owned()));
    }

    #[test]
    fn should_place_the_new_name_beside_the_old() {
        // when
        let target = checked_name(Path::new("/r/src/a.rs"), "b.rs");

        // then
        assert_eq!(target, Ok(PathBuf::from("/r/src/b.rs")));
    }

    #[test]
    fn should_select_the_stem_of_a_file() {
        // when
        let end = stem_end("main.test.rs", PathKind::File);

        // then
        assert_eq!(end, "main.test".len());
    }

    #[test]
    fn should_select_all_of_a_dot_file_and_a_folder() {
        // when
        let ends = [stem_end(".gitignore", PathKind::File), stem_end("v1.2", PathKind::Folder)];

        // then
        assert_eq!(ends, [".gitignore".len(), "v1.2".len()]);
    }

    #[test]
    fn should_refuse_a_folder_holding_an_open_worktree() {
        // given
        let checkouts = [PathBuf::from("/r"), PathBuf::from("/r/.worktree/feature")];

        // when
        let refused = [protected(Path::new("/r/.worktree"), &checkouts), protected(Path::new("/r/.worktree/feature"), &checkouts)];

        // then
        assert_eq!(refused, [Some("/r/.worktree holds an open worktree".to_owned()), Some("/r/.worktree/feature is a checkout's root".to_owned())]);
    }

    #[test]
    fn should_refuse_git_folder() {
        // when
        let refused = protected(Path::new("/r/.git"), &[PathBuf::from("/r")]);

        // then
        assert_eq!(refused, Some("/r/.git is git's own".to_owned()));
    }

    #[test]
    fn should_leave_an_ordinary_file_unprotected() {
        // when
        let refused = protected(Path::new("/r/src/a.rs"), &[PathBuf::from("/r"), PathBuf::from("/r/.worktree/feature")]);

        // then
        assert_eq!(refused, None);
    }

    #[test]
    fn should_rename_only_the_case() {
        // given
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("foo.rs"), "x").unwrap();

        // when
        let renamed = rename(&dir.path().join("foo.rs"), &dir.path().join("Foo.rs"));

        // then
        let names: Vec<String> = fs::read_dir(dir.path()).unwrap().map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned()).collect();
        assert_eq!((renamed, names), (Ok(()), vec!["Foo.rs".to_owned()]));
    }

    #[test]
    fn should_refuse_a_hard_link_of_itself_and_keep_the_file() {
        // given
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a.rs"), "a").unwrap();
        fs::hard_link(dir.path().join("a.rs"), dir.path().join("b.rs")).unwrap();

        // when
        let refused = rename(&dir.path().join("a.rs"), &dir.path().join("b.rs"));

        // then
        let mut names: Vec<String> = fs::read_dir(dir.path()).unwrap().map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned()).collect();
        names.sort();
        assert_eq!((refused.is_err(), names), (true, vec!["a.rs".to_owned(), "b.rs".to_owned()]));
    }

    #[test]
    fn should_refuse_a_taken_name() {
        // given
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a.rs"), "a").unwrap();
        fs::write(dir.path().join("b.rs"), "b").unwrap();

        // when
        let refused = rename(&dir.path().join("a.rs"), &dir.path().join("b.rs"));

        // then
        let kept = (fs::read_to_string(dir.path().join("a.rs")).unwrap(), fs::read_to_string(dir.path().join("b.rs")).unwrap());
        assert_eq!((refused.is_err_and(|error| error.contains("b.rs is taken")), kept), (true, ("a".to_owned(), "b".to_owned())));
    }

    #[test]
    fn should_delete_permanently() {
        // given
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("d/e")).unwrap();
        fs::write(dir.path().join("d/e/f"), "f").unwrap();
        fs::write(dir.path().join("g"), "g").unwrap();

        // when
        let deleted = [delete(&dir.path().join("d"), DeleteMode::Permanent), delete(&dir.path().join("g"), DeleteMode::Permanent)];

        // then
        assert_eq!((deleted, fs::read_dir(dir.path()).unwrap().count()), ([Ok(()), Ok(())], 0));
    }

    #[test]
    fn should_delete_a_link_and_keep_what_it_points_at() {
        // given
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join("kept")).unwrap();
        fs::write(dir.path().join("kept/f"), "f").unwrap();
        std::os::unix::fs::symlink(dir.path().join("kept"), dir.path().join("link")).unwrap();

        // when
        let deleted = delete(&dir.path().join("link"), DeleteMode::Permanent);

        // then
        assert_eq!((deleted, dir.path().join("kept/f").exists(), dir.path().join("link").exists()), (Ok(()), true, false));
    }
}
