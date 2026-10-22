//! Add to .gitignore: lines appended to a checkout's root `.gitignore`.

use std::io;
use std::path::Path;

use crate::menu::{PathKind, Subject, SubjectPath, Within};

/// Appends to `gitignore` the `lines` it does not hold yet, after a newline when it does not end in one; creates it
/// when missing.
pub fn append(gitignore: &Path, lines: &[String]) -> io::Result<()> {
    let existing = match std::fs::read_to_string(gitignore) {
        Ok(text) => text,
        Err(error) if error.kind() == io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(error),
    };
    let present: Vec<&str> = existing.lines().map(str::trim).collect();
    let mut added: Vec<&str> = Vec::new();
    for line in lines.iter().map(String::as_str).filter(|line| !present.contains(line)) {
        if !added.contains(&line) {
            added.push(line);
        }
    }
    if added.is_empty() {
        return Ok(());
    }
    let separator = if existing.is_empty() || existing.ends_with('\n') { "" } else { "\n" };
    std::fs::write(gitignore, format!("{existing}{separator}{}\n", added.join("\n")))
}

/// The `.gitignore` lines for the subject's paths in its checkout: anchored at the root, a folder with a trailing `/`.
pub fn ignore_lines(subject: &Subject) -> Vec<String> {
    let line = |path: &SubjectPath| match (&path.within, path.kind) {
        (Within::Checkout(relative), PathKind::Folder) => Some(format!("/{relative}/")),
        (Within::Checkout(relative), PathKind::File) => Some(format!("/{relative}")),
        (Within::Outside, _) => None,
    };
    subject.paths.iter().filter_map(line).collect()
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use super::*;
    use crate::menu::{Home, SubjectPath};

    #[test]
    fn should_append_lines_once_with_a_newline_before() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join(".gitignore");
        fs::write(&file, "target/\n/dist").unwrap();

        // when
        append(&file, &["/notes.md".to_owned(), "/dist".to_owned(), "/out/".to_owned()]).unwrap();

        // then
        assert_eq!(fs::read_to_string(&file).unwrap(), "target/\n/dist\n/notes.md\n/out/\n");
    }

    #[test]
    fn should_create_a_missing_gitignore() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join(".gitignore");

        // when
        append(&file, &["/a.log".to_owned()]).unwrap();

        // then
        assert_eq!(fs::read_to_string(&file).unwrap(), "/a.log\n");
    }

    #[test]
    fn should_leave_the_file_alone_when_every_line_is_there() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join(".gitignore");
        fs::write(&file, "/a.log").unwrap();

        // when
        append(&file, &["/a.log".to_owned()]).unwrap();

        // then
        assert_eq!(fs::read_to_string(&file).unwrap(), "/a.log");
    }

    #[test]
    fn should_anchor_paths_and_mark_folders() {
        // given
        let subject = Subject {
            paths: vec![
                SubjectPath { path: PathBuf::from("/w/build"), within: Within::Checkout("build".to_owned()), kind: PathKind::Folder },
                SubjectPath { path: PathBuf::from("/w/src/gen.rs"), within: Within::Checkout("src/gen.rs".to_owned()), kind: PathKind::File },
            ],
            home: Home::Checkout(PathBuf::from("/w")),
        };

        // when
        let lines = ignore_lines(&subject);

        // then
        assert_eq!(lines, ["/build/", "/src/gen.rs"]);
    }
}
