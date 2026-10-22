//! Matching a query over the corpus off the UI thread: the best rows kept, every match counted, a newer query's
//! ticket stopping an older scan between chunks.

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet};
use std::sync::Arc;
use std::time::Instant;

use super::corpus::{Corpus, Entry};
use super::files::Class;
use super::matcher::{Match, Query, rank};
use crate::scratch::search::Ticket;

/// Rows a list shows; the counter tells how many more matched.
pub const RENDERED: usize = 80;
/// Entries matched between two looks at the ticket and two partial results: about a frame's work.
pub const CHUNK: usize = 50_000;

/// Which entries a search looks at.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Filter {
    pub project: Option<usize>,
    /// a project and the bit of one of its checkouts
    pub checkout: Option<(usize, u32)>,
    /// excluded and ignored files too
    pub excluded: bool,
    /// paths shown above the files already (Recent rows), by project
    pub skip: HashSet<(usize, Arc<str>)>,
    /// projects the tree header's picker left out
    pub hidden: HashSet<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hit {
    /// index into the corpus' entries
    pub entry: usize,
    pub score: i32,
    pub positions: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Found {
    /// best first, at most [`RENDERED`]
    pub hits: Vec<Hit>,
    pub matched: usize,
    /// entries the filter let through
    pub total: usize,
    pub millis: u32,
    /// every entry was looked at
    pub done: bool,
}

/// Matches `query` over the entries `filter` lets through, in chunks of [`CHUNK`]; after each chunk `partial` gets
/// the best rows so far. `None` once a newer search took `ticket`'s place.
pub fn search(corpus: &Corpus, query: &Query, filter: &Filter, ticket: &Ticket, mut partial: impl FnMut(&Found)) -> Option<Found> {
    let started = Instant::now();
    let mut best = BinaryHeap::with_capacity(RENDERED + 1);
    let (mut matched, mut total) = (0, 0);
    let chunks = corpus.entries.chunks(CHUNK).count();
    for (chunk, entries) in corpus.entries.chunks(CHUNK).enumerate() {
        if ticket.is_stale() {
            return None;
        }
        for (offset, entry) in entries.iter().enumerate() {
            if !filter.admits(entry) {
                continue;
            }
            total += 1;
            let Some(hit) = query.matches(&entry.path, &entry.lower, Some(entry.name_at)) else {
                continue;
            };
            matched += 1;
            best.push(Ranked { hit, path: entry.path.clone(), entry: chunk * CHUNK + offset });
            if best.len() > RENDERED {
                best.pop();
            }
        }
        let found = Found { hits: sorted(&best), matched, total, millis: elapsed_millis(started), done: chunk + 1 == chunks };
        partial(&found);
        if found.done {
            return Some(found);
        }
    }
    Some(Found { hits: Vec::new(), matched, total, millis: elapsed_millis(started), done: true })
}

impl Filter {
    fn admits(&self, entry: &Entry) -> bool {
        let in_project = self.project.is_none_or(|project| project == entry.project) && !self.hidden.contains(&entry.project);
        let in_checkout = self.checkout.is_none_or(|(project, bit)| project == entry.project && 1u64.checked_shl(bit).is_some_and(|mask| entry.checkouts & mask != 0));
        let searched = self.excluded || entry.class == Class::Searched;
        in_project && in_checkout && searched && (self.skip.is_empty() || !self.skip.contains(&(entry.project, entry.path.clone())))
    }
}

/// A hit in the heap of the best ones: the worst on top, so it is the one dropped.
struct Ranked {
    hit: Match,
    path: Arc<str>,
    entry: usize,
}

impl Ranked {
    fn order(&self, other: &Self) -> Ordering {
        rank((&self.hit, &self.path, self.entry), (&other.hit, &other.path, other.entry))
    }
}

impl PartialEq for Ranked {
    fn eq(&self, other: &Self) -> bool {
        self.order(other) == Ordering::Equal
    }
}

impl Eq for Ranked {}

impl PartialOrd for Ranked {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Ranked {
    fn cmp(&self, other: &Self) -> Ordering {
        self.order(other)
    }
}

fn sorted(best: &BinaryHeap<Ranked>) -> Vec<Hit> {
    let mut ranked: Vec<&Ranked> = best.iter().collect();
    ranked.sort_by(|a, b| a.order(b));
    ranked.into_iter().map(|ranked| Hit { entry: ranked.entry, score: ranked.hit.score, positions: ranked.hit.positions.clone() }).collect()
}

fn elapsed_millis(started: Instant) -> u32 {
    u32::try_from(started.elapsed().as_millis()).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hub::corpus::{ProjectInput, build};
    use crate::hub::files::CheckoutFiles;
    use crate::scratch::search::Generations;

    fn listing(paths: &[(&str, Class)]) -> CheckoutFiles {
        CheckoutFiles { paths: paths.iter().map(|(path, class)| (path.to_string(), *class)).collect(), skipped: 0 }
    }

    fn corpus_of(paths: &[&str]) -> Corpus {
        let files = listing(&paths.iter().map(|path| (*path, Class::Searched)).collect::<Vec<_>>());
        build(&[ProjectInput { checkouts: vec![Some(&files)], ignored: Vec::new(), scratches: &[] }])
    }

    fn paths<'a>(corpus: &'a Corpus, found: &Found) -> Vec<&'a str> {
        found.hits.iter().map(|hit| &*corpus.entries[hit.entry].path).collect()
    }

    fn run(corpus: &Corpus, query: &str, filter: &Filter) -> Found {
        search(corpus, &Query::parse(query), filter, &Generations::default().next(), |_| {}).unwrap()
    }

    #[test]
    fn should_keep_the_best_rows_and_count_all_matches() {
        // given
        let names: Vec<String> = (0..200).map(|number| format!("dir{number}/file.rs")).collect();
        let corpus = corpus_of(&names.iter().map(String::as_str).collect::<Vec<_>>());

        // when
        let found = run(&corpus, "file", &Filter::default());

        // then
        assert_eq!((found.hits.len(), found.matched, found.total), (RENDERED, 200, 200));
        let mut sorted = paths(&corpus, &found);
        sorted.sort_by_key(|path| path.len());
        assert_eq!(paths(&corpus, &found), sorted);
    }

    #[test]
    fn should_leave_excluded_files_out_unless_asked() {
        // given
        let files = listing(&[("a.rs", Class::Searched), ("target/a.rs", Class::Excluded)]);
        let corpus = build(&[ProjectInput { checkouts: vec![Some(&files)], ignored: Vec::new(), scratches: &[] }]);

        // when
        let without = run(&corpus, "a", &Filter::default());
        let with = run(&corpus, "a", &Filter { excluded: true, ..Filter::default() });

        // then
        assert_eq!((paths(&corpus, &without), without.total), (vec!["a.rs"], 1));
        assert_eq!((paths(&corpus, &with), with.total), (vec!["a.rs", "target/a.rs"], 2));
    }

    #[test]
    fn should_search_only_the_scoped_checkout() {
        // given
        let (main, other) = (listing(&[("main.rs", Class::Searched)]), listing(&[("other.rs", Class::Searched)]));
        let corpus = build(&[ProjectInput { checkouts: vec![Some(&main), Some(&other)], ignored: Vec::new(), scratches: &[] }]);

        // when
        let found = run(&corpus, "rs", &Filter { checkout: Some((0, 1)), ..Filter::default() });

        // then
        assert_eq!(paths(&corpus, &found), ["other.rs"]);
    }

    #[test]
    fn should_search_only_the_scoped_project() {
        // given
        let (first, second) = (listing(&[("a.rs", Class::Searched)]), listing(&[("b.rs", Class::Searched)]));
        let input = |files| ProjectInput { checkouts: vec![Some(files)], ignored: Vec::new(), scratches: &[] };
        let corpus = build(&[input(&first), input(&second)]);

        // when
        let found = run(&corpus, "rs", &Filter { project: Some(1), ..Filter::default() });

        // then
        assert_eq!((paths(&corpus, &found), found.total), (vec!["b.rs"], 1));
    }

    #[test]
    fn should_skip_paths_listed_in_recent() {
        // given
        let corpus = corpus_of(&["a.rs", "ab.rs"]);
        let skip = HashSet::from([(0, Arc::from("a.rs"))]);

        // when
        let found = run(&corpus, "a", &Filter { skip, ..Filter::default() });

        // then
        assert_eq!((paths(&corpus, &found), found.matched), (vec!["ab.rs"], 1));
    }

    #[test]
    fn should_drop_a_result_of_an_older_generation() {
        // given
        let corpus = corpus_of(&["a.rs"]);
        let generations = Generations::default();
        let old = generations.next();
        let _new = generations.next();

        // when
        let found = search(&corpus, &Query::parse("a"), &Filter::default(), &old, |_| {});

        // then
        assert_eq!(found, None);
    }

    #[test]
    fn should_report_partial_results_per_chunk() {
        // given
        let names: Vec<String> = (0..=CHUNK).map(|number| format!("f{number}")).collect();
        let corpus = corpus_of(&names.iter().map(String::as_str).collect::<Vec<_>>());
        let mut partials = Vec::new();

        // when
        let found = search(&corpus, &Query::parse("f"), &Filter::default(), &Generations::default().next(), |partial| partials.push(partial.done)).unwrap();

        // then
        assert_eq!(partials, [false, true]);
        assert!(found.done);
        assert_eq!(found.matched, CHUNK + 1);
    }

    #[test]
    fn should_search_ready_projects_while_one_loads() {
        // given
        let ready = listing(&[("a.rs", Class::Searched)]);
        let corpus = build(&[
            ProjectInput { checkouts: vec![Some(&ready)], ignored: Vec::new(), scratches: &[] },
            ProjectInput { checkouts: vec![None], ignored: Vec::new(), scratches: &[] },
        ]);

        // when
        let found = run(&corpus, "a", &Filter::default());

        // then
        assert_eq!(paths(&corpus, &found), ["a.rs"]);
    }
}
