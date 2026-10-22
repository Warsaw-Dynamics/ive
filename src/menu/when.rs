//! The part of an item's `when` IVE checks at once: kind, name and path globs, count. `test` runs on the script
//! thread.

use globset::{Glob, GlobBuilder, GlobSet, GlobSetBuilder};

use super::{PathKind, Subject, SubjectPath, Within};
use crate::script::menus::{CountIs, KindIs, When};

/// Whether `when`'s kind, name, path and count hold for `subject`: the first three for every one of its paths, and
/// none of them without paths.
pub fn described(when: &When, subject: &Subject) -> bool {
    let paths = &subject.paths;
    let count = match when.count {
        CountIs::Any => true,
        CountIs::One => paths.len() == 1,
        CountIs::Many => paths.len() > 1,
    };
    let per_path = when.kind != KindIs::Any || !when.name.is_empty() || !when.path.is_empty();
    if per_path && paths.is_empty() {
        return false;
    }
    let (names, relative) = (glob_set(&when.name, Separator::Any), glob_set(&when.path, Separator::Literal));
    count && paths.iter().all(|path| kind_is(when.kind, path) && matches(&names, &name_of(path)) && matches(&relative, &matched_path(path)))
}

/// Checks the globs of a `when` as `config` declares them; the error names the glob.
pub fn check_globs(globs: &[String]) -> Result<(), String> {
    globs.iter().try_for_each(|glob| Glob::new(glob).map(drop).map_err(|error| format!("{glob:?} is not a glob: {error}")))
}

/// Whether `*` crosses a `/`: in a name there is none; in a path it does not, as in `.gitignore`.
#[derive(Clone, Copy)]
enum Separator {
    Any,
    Literal,
}

/// `globs` as one set; `None` for no globs, which matches anything. A glob `config` let through always compiles.
fn glob_set(globs: &[String], separator: Separator) -> Option<GlobSet> {
    if globs.is_empty() {
        return None;
    }
    let mut set = GlobSetBuilder::new();
    let literal = matches!(separator, Separator::Literal);
    for glob in globs.iter().filter_map(|glob| GlobBuilder::new(glob).literal_separator(literal).build().ok()) {
        set.add(glob);
    }
    set.build().ok()
}

fn matches(set: &Option<GlobSet>, text: &str) -> bool {
    set.as_ref().is_none_or(|set| set.is_match(text))
}

fn kind_is(kind: KindIs, path: &SubjectPath) -> bool {
    match kind {
        KindIs::Any => true,
        KindIs::File => path.kind == PathKind::File,
        KindIs::Folder => path.kind == PathKind::Folder,
    }
}

fn name_of(path: &SubjectPath) -> String {
    path.path.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default()
}

/// The path relative to its checkout, or absolute outside every checkout.
fn matched_path(path: &SubjectPath) -> String {
    match &path.within {
        Within::Checkout(relative) => relative.clone(),
        Within::Outside => path.path.to_string_lossy().into_owned(),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::menu::Home;

    fn file(path: &str, relative: &str) -> SubjectPath {
        SubjectPath { path: PathBuf::from(path), within: Within::Checkout(relative.to_owned()), kind: PathKind::File }
    }

    fn folder(path: &str, relative: &str) -> SubjectPath {
        SubjectPath { kind: PathKind::Folder, ..file(path, relative) }
    }

    fn subject(paths: Vec<SubjectPath>) -> Subject {
        Subject { paths, home: Home::Checkout(PathBuf::from("/w")) }
    }

    fn globs(list: &[&str]) -> Vec<String> {
        list.iter().map(|glob| (*glob).to_owned()).collect()
    }

    #[test]
    fn should_match_kind_for_every_path() {
        // given
        let when = When { kind: KindIs::File, ..When::default() };

        // when
        let files = described(&when, &subject(vec![file("/w/a.rs", "a.rs"), file("/w/b.rs", "b.rs")]));
        let mixed = described(&when, &subject(vec![file("/w/a.rs", "a.rs"), folder("/w/src", "src")]));

        // then
        assert_eq!((files, mixed), (true, false));
    }

    #[test]
    fn should_match_name_globs_any_of_list() {
        // given
        let when = When { name: globs(&["*.rs", "Cargo.toml"]), ..When::default() };

        // when
        let matched = [
            described(&when, &subject(vec![file("/w/src/a.rs", "src/a.rs"), file("/w/Cargo.toml", "Cargo.toml")])),
            described(&when, &subject(vec![file("/w/README.md", "README.md")])),
        ];

        // then
        assert_eq!(matched, [true, false]);
    }

    #[test]
    fn should_match_path_glob_against_relative_path() {
        // given
        let when = When { path: globs(&["src/**"]), ..When::default() };

        // when
        let matched = [
            described(&when, &subject(vec![file("/w/src/menu/a.rs", "src/menu/a.rs")])),
            described(&when, &subject(vec![file("/w/tests/src/a.rs", "tests/src/a.rs")])),
        ];

        // then
        assert_eq!(matched, [true, false]);
    }

    #[test]
    fn should_match_absolute_path_outside_checkout() {
        // given
        let when = When { path: globs(&["/home/*/scratches/**"]), ..When::default() };
        let outside = SubjectPath { path: PathBuf::from("/home/me/scratches/a.sh"), within: Within::Outside, kind: PathKind::File };

        // when
        let matched = described(&when, &Subject { paths: vec![outside], home: Home::Mixed });

        // then
        assert!(matched);
    }

    #[test]
    fn should_never_match_path_fields_without_paths() {
        // when
        let matched = [
            described(&When { kind: KindIs::File, ..When::default() }, &Subject::empty()),
            described(&When { name: globs(&["*"]), ..When::default() }, &Subject::empty()),
            described(&When { count: CountIs::One, ..When::default() }, &Subject::empty()),
            described(&When::default(), &Subject::empty()),
        ];

        // then
        assert_eq!(matched, [false, false, false, true]);
    }

    #[test]
    fn should_match_count_one_and_many() {
        // given
        let (one, many) = (When { count: CountIs::One, ..When::default() }, When { count: CountIs::Many, ..When::default() });
        let (single, pair) = (subject(vec![file("/w/a", "a")]), subject(vec![file("/w/a", "a"), file("/w/b", "b")]));

        // when
        let matched = [described(&one, &single), described(&one, &pair), described(&many, &single), described(&many, &pair)];

        // then
        assert_eq!(matched, [true, false, false, true]);
    }
}
