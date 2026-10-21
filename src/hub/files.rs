//! The files of one checkout the hub searches: git's index plus untracked files git does not ignore, or a walk of a
//! folder without git; gitignored files apart, read only once the user asks for them.

use std::collections::BTreeSet;
use std::path::Path;

use ignore::WalkBuilder;

use crate::git;
use crate::project::ProjectKind;

/// What the hub does with a file by default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Class {
    Searched,
    /// below a folder named in `config.excluded`
    Excluded,
    /// gitignored
    Ignored,
}

/// One checkout's files, checkout-relative with `/` between folders, sorted.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CheckoutFiles {
    pub paths: Vec<(String, Class)>,
    /// paths left out because they are not UTF-8
    pub skipped: usize,
}

/// git's index plus untracked files it does not ignore; for a folder without git, a walk honouring `.gitignore` and
/// `.ignore` files, skipping `.git`. A path below a folder named in `excluded` is [`Class::Excluded`]. Reads the
/// disk: run it on the background executor.
#[tracing::instrument(level = "debug", name = "list_files", skip_all, fields(dir = %dir.display()))]
pub fn read_checkout(dir: &Path, kind: ProjectKind, excluded: &[String]) -> Result<CheckoutFiles, String> {
    let paths = match kind {
        ProjectKind::Git => git::files::listed(dir).map_err(|error| error.to_string())?,
        ProjectKind::Plain => walk(dir, true)?.into_iter().collect(),
    };
    Ok(classified(paths, |path| match below_excluded(path, excluded) {
        true => Class::Excluded,
        false => Class::Searched,
    }))
}

/// The gitignored files only, all [`Class::Ignored`]; for a folder without git, what the `.gitignore` files leave out.
#[tracing::instrument(level = "debug", name = "list_ignored_files", skip_all, fields(dir = %dir.display()))]
pub fn read_ignored(dir: &Path, kind: ProjectKind) -> Result<CheckoutFiles, String> {
    let paths = match kind {
        ProjectKind::Git => git::files::ignored(dir).map_err(|error| error.to_string())?,
        ProjectKind::Plain => {
            let honoured = walk(dir, true)?;
            walk(dir, false)?.difference(&honoured).cloned().collect()
        }
    };
    Ok(classified(paths, |_| Class::Ignored))
}

fn classified(paths: Vec<Vec<u8>>, class: impl Fn(&str) -> Class) -> CheckoutFiles {
    let mut files = CheckoutFiles::default();
    for path in paths {
        match String::from_utf8(path) {
            Ok(path) => {
                let class = class(&path);
                files.paths.push((path, class));
            }
            Err(_) => files.skipped += 1,
        }
    }
    files
}

fn below_excluded(path: &str, excluded: &[String]) -> bool {
    path.rsplit_once('/').is_some_and(|(folders, _)| folders.split('/').any(|folder| excluded.iter().any(|name| name == folder)))
}

/// The files below `dir` as relative path bytes; `honour_ignores` applies `.gitignore` and `.ignore` files.
fn walk(dir: &Path, honour_ignores: bool) -> Result<BTreeSet<Vec<u8>>, String> {
    if !dir.is_dir() {
        return Err(format!("cannot list the files of {}: not a folder", dir.display()));
    }
    let mut builder = WalkBuilder::new(dir);
    builder.standard_filters(false).git_ignore(honour_ignores).ignore(honour_ignores).require_git(false);
    builder.filter_entry(|entry| entry.file_name() != ".git");
    let mut paths = BTreeSet::new();
    // an unreadable folder inside the project leaves out its own files, not the project's
    for entry in builder.build().flatten() {
        if entry.file_type().is_some_and(|kind| kind.is_dir()) {
            continue;
        }
        if let Ok(relative) = entry.path().strip_prefix(dir) {
            paths.insert(relative_bytes(relative));
        }
    }
    Ok(paths)
}

fn relative_bytes(relative: &Path) -> Vec<u8> {
    let parts: Vec<&[u8]> = relative.iter().map(|part| part.as_encoded_bytes()).collect();
    parts.join(&b'/')
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::git::testing::TestRepo;

    const EXCLUDED: [String; 0] = [];

    fn searched(paths: &[&str]) -> Vec<(String, Class)> {
        paths.iter().map(|path| (path.to_string(), Class::Searched)).collect()
    }

    fn repo_with_ignored_output() -> TestRepo {
        let repo = TestRepo::init();
        repo.write(".gitignore", "out/\n");
        repo.write("a.rs", "a");
        repo.commit("first");
        repo.write("b.rs", "b");
        repo.write("out/c.rs", "c");
        repo
    }

    #[test]
    fn should_list_tracked_and_untracked_files() {
        // given
        let repo = repo_with_ignored_output();

        // when
        let files = read_checkout(repo.path(), ProjectKind::Git, &EXCLUDED).unwrap();

        // then
        assert_eq!(files.paths, searched(&[".gitignore", "a.rs", "b.rs"]));
    }

    #[test]
    fn should_class_a_named_folder_as_excluded() {
        // given
        let repo = TestRepo::init();
        repo.write("target/x.rs", "x");
        repo.write("src/target.rs", "y");
        repo.commit("first");

        // when
        let files = read_checkout(repo.path(), ProjectKind::Git, &["target".to_owned()]).unwrap();

        // then
        assert_eq!(files.paths, [("src/target.rs".to_owned(), Class::Searched), ("target/x.rs".to_owned(), Class::Excluded)]);
    }

    #[test]
    fn should_read_ignored_files_apart() {
        // given
        let repo = repo_with_ignored_output();

        // when
        let files = read_ignored(repo.path(), ProjectKind::Git).unwrap();

        // then
        assert_eq!(files.paths, [("out/c.rs".to_owned(), Class::Ignored)]);
    }

    #[test]
    fn should_walk_a_folder_without_git_honouring_gitignore() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        for (path, content) in [(".gitignore", "tmp/\n"), (".git/HEAD", "x"), ("tmp/a", "a"), ("src/b", "b")] {
            fs::create_dir_all(root.join(path).parent().unwrap()).unwrap();
            fs::write(root.join(path), content).unwrap();
        }

        // when
        let files = read_checkout(root, ProjectKind::Plain, &EXCLUDED).unwrap();
        let ignored = read_ignored(root, ProjectKind::Plain).unwrap();

        // then
        assert_eq!(files.paths, searched(&[".gitignore", "src/b"]));
        assert_eq!(ignored.paths, [("tmp/a".to_owned(), Class::Ignored)]);
    }

    #[cfg(unix)]
    // APFS refuses names that are not UTF-8, so only Linux has such paths
    #[cfg(target_os = "linux")]
    #[test]
    fn should_skip_a_path_that_is_not_utf8() {
        use std::os::unix::ffi::OsStrExt;

        // given
        let repo = TestRepo::init();
        repo.write("a.rs", "a");
        repo.commit("first");
        repo.write_bytes(std::path::Path::new(std::ffi::OsStr::from_bytes(b"\xff.txt")), b"x");

        // when
        let files = read_checkout(repo.path(), ProjectKind::Git, &EXCLUDED).unwrap();

        // then
        assert_eq!(files.paths, searched(&["a.rs"]));
        assert_eq!(files.skipped, 1);
    }

    #[test]
    fn should_name_the_checkout_when_it_cannot_be_read() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("gone");

        // when
        let error = read_checkout(&missing, ProjectKind::Git, &EXCLUDED).unwrap_err();

        // then
        assert!(error.contains(&missing.display().to_string()), "{error}");
    }
}
