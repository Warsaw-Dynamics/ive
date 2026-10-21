//! Every open project's files as one list for the matcher: one entry per project path, with the checkouts that
//! have it, so the work per keystroke grows with distinct paths, not with worktrees.

use std::collections::BTreeMap;
use std::sync::Arc;

use super::files::{CheckoutFiles, Class};
use super::matcher::lowered;

/// Checkouts of a project the index tells apart (bits of [`Entry::checkouts`]); later ones are left out.
pub const MAX_CHECKOUTS: usize = 64;
/// Paths of all projects together; past it the corpus stops growing and says so.
pub const MAX_CORPUS: usize = 2_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    File,
    /// `scratches/<name>` of its project
    Scratch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// index into the projects the corpus was built from
    pub project: usize,
    pub path: Arc<str>,
    /// `path` lowered char by char, for the matcher
    pub lower: Box<str>,
    /// byte index where the file name starts
    pub name_at: usize,
    /// bit `i` set: the project's checkout `i` has the path; none for a scratch
    pub checkouts: u64,
    pub class: Class,
    pub origin: Origin,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Corpus {
    pub entries: Vec<Entry>,
    /// entries not searched by default: excluded and ignored
    pub excluded: usize,
    /// whether [`MAX_CORPUS`] left paths out
    pub truncated: bool,
}

/// One project's checkouts in its order; `None` for a checkout not read yet.
pub struct ProjectInput<'a> {
    pub checkouts: Vec<Option<&'a CheckoutFiles>>,
    /// gitignored files per checkout, once read
    pub ignored: Vec<Option<&'a CheckoutFiles>>,
    pub scratches: &'a [String],
}

/// One entry per distinct project path, sorted by path within each project, projects in order.
#[tracing::instrument(level = "debug", name = "build_corpus", skip_all, fields(projects = projects.len()))]
pub fn build(projects: &[ProjectInput<'_>]) -> Corpus {
    let mut corpus = Corpus::default();
    for (project, input) in projects.iter().enumerate() {
        for (path, (checkouts, class)) in merged(input) {
            if corpus.entries.len() == MAX_CORPUS {
                corpus.truncated = true;
                return corpus;
            }
            corpus.excluded += usize::from(class != Class::Searched);
            corpus.entries.push(entry(project, path, checkouts, class, Origin::File));
        }
        for name in input.scratches {
            corpus.entries.push(entry(project, &format!("scratches/{name}"), 0, Class::Searched, Origin::Scratch));
        }
    }
    corpus
}

/// Each path with the checkouts that list it and its class; a path searched in one checkout is searched.
fn merged<'a>(input: &ProjectInput<'a>) -> BTreeMap<&'a str, (u64, Class)> {
    let mut paths: BTreeMap<&str, (u64, Class)> = BTreeMap::new();
    let listings = input.checkouts.iter().enumerate().chain(input.ignored.iter().enumerate());
    for (index, files) in listings.filter(|(index, _)| *index < MAX_CHECKOUTS) {
        for (path, class) in files.iter().flat_map(|files| &files.paths) {
            let (checkouts, merged) = paths.entry(path).or_insert((0, *class));
            *checkouts |= 1 << index;
            *merged = (*merged).min(*class);
        }
    }
    paths
}

fn entry(project: usize, path: &str, checkouts: u64, class: Class, origin: Origin) -> Entry {
    let name_at = path.rfind('/').map_or(0, |slash| slash + 1);
    Entry { project, path: Arc::from(path), lower: lowered(path).into_boxed_str(), name_at, checkouts, class, origin }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn files(paths: &[(&str, Class)]) -> CheckoutFiles {
        CheckoutFiles { paths: paths.iter().map(|(path, class)| (path.to_string(), *class)).collect(), skipped: 0 }
    }

    fn searched(paths: &[&str]) -> CheckoutFiles {
        files(&paths.iter().map(|path| (*path, Class::Searched)).collect::<Vec<_>>())
    }

    fn project<'a>(checkouts: Vec<Option<&'a CheckoutFiles>>) -> ProjectInput<'a> {
        ProjectInput { checkouts, ignored: Vec::new(), scratches: &[] }
    }

    fn summary(corpus: &Corpus) -> Vec<(&str, u64)> {
        corpus.entries.iter().map(|entry| (&*entry.path, entry.checkouts)).collect()
    }

    #[test]
    fn should_merge_checkouts_into_one_entry_per_path() {
        // given
        let (main, other) = (searched(&["a", "b"]), searched(&["a", "c"]));

        // when
        let corpus = build(&[project(vec![Some(&main), Some(&other)])]);

        // then
        assert_eq!(summary(&corpus), [("a", 0b11), ("b", 0b01), ("c", 0b10)]);
    }

    #[test]
    fn should_index_only_the_first_64_checkouts() {
        // given
        let one = searched(&["a"]);

        // when
        let corpus = build(&[project(vec![Some(&one); 65])]);

        // then
        assert_eq!(summary(&corpus), [("a", u64::MAX)]);
    }

    #[test]
    fn should_add_scratches_of_a_project() {
        // given
        let scratches = ["notes".to_owned()];
        let input = ProjectInput { checkouts: Vec::new(), ignored: Vec::new(), scratches: &scratches };

        // when
        let corpus = build(&[input]);

        // then
        assert_eq!(summary(&corpus), [("scratches/notes", 0)]);
        assert_eq!(corpus.entries[0].origin, Origin::Scratch);
    }

    #[test]
    fn should_count_excluded_and_ignored() {
        // given
        let listed = files(&[("a", Class::Searched), ("target/b", Class::Excluded)]);
        let ignored = files(&[("out/c", Class::Ignored)]);
        let input = ProjectInput { checkouts: vec![Some(&listed)], ignored: vec![Some(&ignored)], scratches: &[] };

        // when
        let corpus = build(&[input]);

        // then
        assert_eq!(corpus.excluded, 2);
        assert_eq!(corpus.entries.iter().map(|entry| entry.class).collect::<Vec<_>>(), [Class::Searched, Class::Ignored, Class::Excluded]);
    }

    #[test]
    fn should_keep_where_the_file_name_starts() {
        // given
        let listed = searched(&["src/theme.rs"]);

        // when
        let corpus = build(&[project(vec![Some(&listed)])]);

        // then
        assert_eq!(corpus.entries[0].name_at, 4);
        assert_eq!(&*corpus.entries[0].lower, "src/theme.rs");
    }
}
