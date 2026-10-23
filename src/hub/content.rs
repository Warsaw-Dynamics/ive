//! Find in files: the lines of files under some roots that match a query, off the UI thread, a newer search's
//! ticket stopping an older one.

use std::fs::File;
use std::io::Read as _;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use ignore::{WalkBuilder, WalkState};
use regex::Regex;

use super::output::OutputHit;
use crate::scratch::search::Ticket;
use crate::script::excluded::Excluded;

/// Lines a search lists at most; the list then says it was cut.
pub const MAX_MATCHES: usize = 2_000;
/// Bigger files are not searched, as they are not coloured.
pub const MAX_FILE_BYTES: u64 = 8 * 1024 * 1024;
/// A file with a NUL byte this early is binary and not searched.
const BINARY_PROBE: usize = 8 * 1024;
/// A row shows this much of a long line, around its first match.
pub const SHOWN_CHARS: usize = 300;
/// A query shorter than this searches nothing: one letter matches nearly every line.
pub const MIN_QUERY_CHARS: usize = 2;
/// Partial results at most this often.
const PARTIAL_EVERY: Duration = Duration::from_millis(50);

/// How the query's text matches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum Matching {
    #[default]
    Plain,
    Regex,
    WholeWord,
    RegexWholeWord,
}

impl Matching {
    pub fn with_regex(self) -> Self {
        match self {
            Self::Plain => Self::Regex,
            Self::WholeWord => Self::RegexWholeWord,
            Self::Regex => Self::Plain,
            Self::RegexWholeWord => Self::WholeWord,
        }
    }

    pub fn with_whole_word(self) -> Self {
        match self {
            Self::Plain => Self::WholeWord,
            Self::Regex => Self::RegexWholeWord,
            Self::WholeWord => Self::Plain,
            Self::RegexWholeWord => Self::Regex,
        }
    }

    pub fn is_regex(self) -> bool {
        matches!(self, Self::Regex | Self::RegexWholeWord)
    }

    pub fn is_whole_word(self) -> bool {
        matches!(self, Self::WholeWord | Self::RegexWholeWord)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentQuery {
    pub text: String,
    pub mode: Matching,
}

impl ContentQuery {
    /// The query as a regex: smart case (any capital makes it exact), the text escaped unless it is a regex, words
    /// whole when asked; the error names the text.
    pub fn compile(&self) -> Result<Regex, String> {
        let pattern = if self.mode.is_regex() { self.text.clone() } else { regex::escape(&self.text) };
        let pattern = if self.mode.is_whole_word() { format!(r"\b(?:{pattern})\b") } else { pattern };
        let pattern = if self.has_capitals() { pattern } else { format!("(?i){pattern}") };
        Regex::new(&pattern).map_err(|error| format!("{:?} is not a regex: {error}", self.text))
    }

    /// Whether the text has a capital letter of its own: in a regex, `\W`, `\S`, `\D`, `\B` and the like are escapes.
    fn has_capitals(&self) -> bool {
        let mut chars = self.text.chars();
        let mut found = false;
        while let Some(c) = chars.next() {
            if c == '\\' && self.mode.is_regex() {
                chars.next();
                continue;
            }
            found |= c.is_uppercase();
        }
        found
    }
}

/// Where a root came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Picked {
    /// a checkout or project of the hub's scope
    Scope,
    /// right-clicked in the tree: searched even when ignored or excluded itself
    FromTree,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Root {
    pub path: PathBuf,
    pub picked: Picked,
}

/// Whether gitignored files and `config.excluded` folders are searched (Alt+I).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ignored {
    Skipped,
    Searched,
}

/// One search: what, where, and what is left out.
pub struct Search<'a> {
    pub roots: &'a [Root],
    /// matching lines kept at most
    pub limit: usize,
    pub query: &'a ContentQuery,
    pub ignored: Ignored,
    pub excluded: &'a Excluded,
}

/// A matching line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentHit {
    pub path: PathBuf,
    /// from 1
    pub line: u32,
    /// the line, or at most [`SHOWN_CHARS`] of it around its first match
    pub text: String,
    /// the matches, as byte ranges of `text`
    pub ranges: Vec<Range<usize>>,
    /// the first match, as bytes of the whole line
    pub bytes: Range<usize>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ContentFound {
    /// by path, then line
    pub hits: Vec<ContentHit>,
    /// lines of processes' output, by process then line; found before the files
    pub output: Vec<OutputHit>,
    /// files with a match
    pub files: usize,
    pub millis: u32,
    /// the limit cut the list
    pub limited: bool,
    /// every file was searched
    pub done: bool,
}

/// The lines of the files under `search.roots` matching its query, on several threads; `partial` gets the hits so far
/// now and then. `None` once a newer search took `ticket`'s place, or when the query does not compile.
pub fn search(search: &Search, ticket: &Ticket, partial: impl Fn(&ContentFound) + Sync) -> Option<ContentFound> {
    let regex = search.query.compile().ok()?;
    let started = Instant::now();
    let shared = Shared { hits: Mutex::new(Vec::new()), count: AtomicUsize::new(0), limit: search.limit, limited: AtomicBool::new(false), last_partial: Mutex::new(started) };
    for root in search.roots {
        if ticket.is_stale() || shared.limited.load(Ordering::Relaxed) {
            break;
        }
        walker(root, search).build_parallel().run(|| {
            let (regex, shared, partial) = (&regex, &shared, &partial);
            Box::new(move |entry| {
                if ticket.is_stale() || shared.limited.load(Ordering::Relaxed) {
                    return WalkState::Quit;
                }
                if let Ok(entry) = entry
                    && entry.file_type().is_some_and(|kind| kind.is_file())
                {
                    shared.add(search_file(entry.path(), regex), started, partial);
                }
                WalkState::Continue
            })
        });
    }
    if ticket.is_stale() {
        return None;
    }
    Some(shared.found(started, true))
}

/// What the threads of one search share.
struct Shared {
    hits: Mutex<Vec<ContentHit>>,
    count: AtomicUsize,
    limit: usize,
    limited: AtomicBool,
    last_partial: Mutex<Instant>,
}

impl Shared {
    /// Takes one file's hits, up to the search's limit in all, and hands out the hits so far when it is time.
    fn add(&self, file_hits: Vec<ContentHit>, started: Instant, partial: &(impl Fn(&ContentFound) + Sync)) {
        if file_hits.is_empty() {
            return;
        }
        let taken = file_hits.len().min(self.limit.saturating_sub(self.count.fetch_add(file_hits.len(), Ordering::Relaxed)));
        if taken < file_hits.len() {
            self.limited.store(true, Ordering::Relaxed);
        }
        self.hits.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).extend(file_hits.into_iter().take(taken));
        let mut last = self.last_partial.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        if last.elapsed() >= PARTIAL_EVERY {
            *last = Instant::now();
            drop(last);
            partial(&self.found(started, false));
        }
    }

    fn found(&self, started: Instant, done: bool) -> ContentFound {
        let mut hits = self.hits.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).clone();
        hits.sort_by(|a, b| (&a.path, a.line).cmp(&(&b.path, b.line)));
        let mut files: Vec<&PathBuf> = hits.iter().map(|hit| &hit.path).collect();
        files.dedup();
        let millis = u32::try_from(started.elapsed().as_millis()).unwrap_or(u32::MAX);
        ContentFound { files: files.len(), millis, limited: self.limited.load(Ordering::Relaxed), done, hits, output: Vec::new() }
    }
}

/// The files under `root`: `.git` never, gitignored files and excluded folders unless asked; a root picked from the
/// tree is searched even when its own parents' `.gitignore` leaves it out.
fn walker(root: &Root, search: &Search) -> WalkBuilder {
    let honour = search.ignored == Ignored::Skipped;
    let mut builder = WalkBuilder::new(&root.path);
    builder.standard_filters(false).git_ignore(honour).git_exclude(honour).ignore(honour).require_git(false).parents(honour);
    let excluded: Vec<String> = if honour { search.excluded.names().to_vec() } else { Vec::new() };
    builder.filter_entry(move |entry| {
        let name = entry.file_name().to_string_lossy();
        entry.depth() == 0 || (name != ".git" && !excluded.iter().any(|folder| *folder == name && entry.file_type().is_some_and(|kind| kind.is_dir())))
    });
    builder
}

/// The matching lines of one file; none for a file too big, binary or unreadable.
fn search_file(path: &Path, regex: &Regex) -> Vec<ContentHit> {
    let Some(bytes) = read_text(path) else {
        return Vec::new();
    };
    let text = String::from_utf8_lossy(&bytes);
    let mut hits = Vec::new();
    for (index, line) in text.split('\n').enumerate() {
        let line = line.strip_suffix('\r').unwrap_or(line);
        let ranges: Vec<Range<usize>> = regex.find_iter(line).map(|found| found.range()).filter(|range| !range.is_empty()).collect();
        if let Some(first) = ranges.first().cloned() {
            let bytes = first.clone();
            let (text, ranges) = shown(line, first, ranges);
            hits.push(ContentHit { path: path.to_path_buf(), line: u32::try_from(index + 1).unwrap_or(u32::MAX), text, ranges, bytes });
        }
        // one past the limit tells the search it was cut
        if hits.len() > MAX_MATCHES {
            break;
        }
    }
    hits
}

/// The file's bytes, unless it is over [`MAX_FILE_BYTES`], binary, or unreadable.
fn read_text(path: &Path) -> Option<Vec<u8>> {
    let file = File::open(path).ok()?;
    if file.metadata().ok()?.len() > MAX_FILE_BYTES {
        return None;
    }
    let mut bytes = Vec::new();
    file.take(MAX_FILE_BYTES).read_to_end(&mut bytes).ok()?;
    let probe = &bytes[..bytes.len().min(BINARY_PROBE)];
    (!probe.contains(&0)).then_some(bytes)
}

/// `line` as a row shows it: whole when short, else [`SHOWN_CHARS`] of it starting a little before the `first`
/// match; `ranges` moved into it, those outside it dropped.
pub(super) fn shown(line: &str, first: Range<usize>, ranges: Vec<Range<usize>>) -> (String, Vec<Range<usize>>) {
    if line.chars().count() <= SHOWN_CHARS {
        return (line.to_owned(), ranges);
    }
    let lead = SHOWN_CHARS / 3;
    let starts: Vec<usize> = line.char_indices().map(|(at, _)| at).collect();
    let first_char = starts.partition_point(|at| *at < first.start);
    let start_char = first_char.saturating_sub(lead).min(starts.len() - SHOWN_CHARS);
    let start = starts[start_char];
    let end = starts.get(start_char + SHOWN_CHARS).copied().unwrap_or(line.len());
    let inside = ranges.into_iter().filter(|range| range.start >= start && range.end <= end).map(|range| range.start - start..range.end - start);
    (line[start..end].to_owned(), inside.collect())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::scratch::search::Generations;

    fn query(text: &str) -> ContentQuery {
        ContentQuery { text: text.to_owned(), mode: Matching::Plain }
    }

    fn excluded(names: &[&str]) -> Excluded {
        crate::script::excluded::parse(&serde_json::from_value(serde_json::json!(names)).unwrap()).unwrap()
    }

    fn root(path: &Path) -> Root {
        Root { path: path.to_path_buf(), picked: Picked::Scope }
    }

    fn found(roots: &[Root], query: &ContentQuery, ignored: Ignored) -> ContentFound {
        let ticket = Generations::default().next();
        search(&Search { roots, limit: MAX_MATCHES, query, ignored, excluded: &excluded(&["node_modules"]) }, &ticket, |_| {}).expect("not cancelled")
    }

    fn lines(found: &ContentFound) -> Vec<String> {
        found.hits.iter().map(|hit| format!("{}:{}:{}", hit.path.file_name().unwrap().to_string_lossy(), hit.line, hit.text)).collect()
    }

    fn project() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a.rs"), "fn main() {}\nlet Menu = 1;\nlet menu = 2;\n").unwrap();
        fs::write(dir.path().join("b.txt"), "the menus\nmenu bar\n").unwrap();
        dir
    }

    #[test]
    fn should_find_lines_ignoring_case_for_lowercase_query() {
        // given
        let dir = project();

        // when
        let found = found(&[root(dir.path())], &query("menu"), Ignored::Skipped);

        // then
        assert_eq!(lines(&found), ["a.rs:2:let Menu = 1;", "a.rs:3:let menu = 2;", "b.txt:1:the menus", "b.txt:2:menu bar"]);
        assert_eq!(found.files, 2);
    }

    #[test]
    fn should_match_case_when_query_has_capitals() {
        // given
        let dir = project();

        // when
        let found = found(&[root(dir.path())], &query("Menu"), Ignored::Skipped);

        // then
        assert_eq!(lines(&found), ["a.rs:2:let Menu = 1;"]);
    }

    #[test]
    fn should_match_regex_and_whole_word() {
        // given
        let dir = project();
        let regex = ContentQuery { text: r"let \w+ = 2".to_owned(), mode: Matching::Regex };
        let word = ContentQuery { text: "menu".to_owned(), mode: Matching::WholeWord };

        // when
        let (by_regex, by_word) = (found(&[root(dir.path())], &regex, Ignored::Skipped), found(&[root(dir.path())], &word, Ignored::Skipped));

        // then
        assert_eq!(lines(&by_regex), ["a.rs:3:let menu = 2;"]);
        assert_eq!(lines(&by_word), ["a.rs:2:let Menu = 1;", "a.rs:3:let menu = 2;", "b.txt:2:menu bar"]);
    }

    #[test]
    fn should_ignore_case_for_a_regex_whose_only_capitals_are_escapes() {
        // given
        let dir = project();
        let regex = ContentQuery { text: r"menu\W".to_owned(), mode: Matching::Regex };

        // when
        let found = found(&[root(dir.path())], &regex, Ignored::Skipped);

        // then
        assert_eq!(lines(&found), ["a.rs:2:let Menu = 1;", "a.rs:3:let menu = 2;", "b.txt:2:menu bar"]);
    }

    #[test]
    fn should_mark_the_matches_in_the_line() {
        // given
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a"), "menu and menu\n").unwrap();

        // when
        let found = found(&[root(dir.path())], &query("menu"), Ignored::Skipped);

        // then
        assert_eq!(found.hits[0].ranges, [0..4, 9..13]);
    }

    #[test]
    fn should_skip_binary_and_big_files() {
        // given
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("bin"), b"menu\0\x01").unwrap();
        fs::write(dir.path().join("big"), format!("menu\n{}", "x".repeat(MAX_FILE_BYTES as usize))).unwrap();
        fs::write(dir.path().join("ok"), "menu\n").unwrap();

        // when
        let found = found(&[root(dir.path())], &query("menu"), Ignored::Skipped);

        // then
        assert_eq!(lines(&found), ["ok:1:menu"]);
    }

    #[test]
    fn should_leave_out_gitignored_and_excluded_files_unless_asked() {
        // given
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(".gitignore"), "target/\n").unwrap();
        for folder in ["target", "node_modules", "src"] {
            fs::create_dir(dir.path().join(folder)).unwrap();
            fs::write(dir.path().join(folder).join("x"), "menu\n").unwrap();
        }

        // when
        let skipped = found(&[root(dir.path())], &query("menu"), Ignored::Skipped);
        let searched = found(&[root(dir.path())], &query("menu"), Ignored::Searched);

        // then
        let folders = |found: &ContentFound| found.hits.iter().map(|hit| hit.path.parent().unwrap().file_name().unwrap().to_string_lossy().into_owned()).collect::<Vec<_>>();
        assert_eq!(folders(&skipped), ["src"]);
        assert_eq!(folders(&searched), ["node_modules", "src", "target"]);
    }

    #[test]
    fn should_search_a_picked_ignored_root() {
        // given
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(".gitignore"), "target/\n").unwrap();
        fs::create_dir(dir.path().join("target")).unwrap();
        fs::write(dir.path().join("target").join("x"), "menu\n").unwrap();
        let picked = Root { path: dir.path().join("target"), picked: Picked::FromTree };

        // when
        let found = found(&[picked], &query("menu"), Ignored::Skipped);

        // then
        assert_eq!(lines(&found), ["x:1:menu"]);
    }

    #[test]
    fn should_leave_out_what_the_root_gitignore_ignores_inside_a_picked_folder() {
        // given
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(".gitignore"), "dist/\n").unwrap();
        let modules = dir.path().join("frontend").join("dist");
        fs::create_dir_all(&modules).unwrap();
        fs::write(modules.join("lib.js"), "menu\n").unwrap();
        fs::write(dir.path().join("frontend").join("app.js"), "menu\n").unwrap();
        let picked = Root { path: dir.path().join("frontend"), picked: Picked::FromTree };

        // when
        let found = found(&[picked], &query("menu"), Ignored::Skipped);

        // then
        assert_eq!(lines(&found), ["app.js:1:menu"]);
    }

    #[test]
    fn should_search_a_picked_file() {
        // given
        let dir = project();
        let picked = Root { path: dir.path().join("b.txt"), picked: Picked::FromTree };

        // when
        let found = found(&[picked], &query("menu"), Ignored::Skipped);

        // then
        assert_eq!(found.hits.len(), 2);
    }

    #[test]
    fn should_stop_at_the_match_limit() {
        // given
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("many"), "menu\n".repeat(MAX_MATCHES + 10)).unwrap();

        // when
        let found = found(&[root(dir.path())], &query("menu"), Ignored::Skipped);

        // then
        assert_eq!((found.hits.len(), found.limited), (MAX_MATCHES, true));
    }

    #[test]
    fn should_cut_a_long_line_around_the_match() {
        // given
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("min.js"), format!("{}menu{}\n", "a".repeat(5000), "b".repeat(5000))).unwrap();

        // when
        let found = found(&[root(dir.path())], &query("menu"), Ignored::Skipped);

        // then
        let hit = &found.hits[0];
        assert!(hit.text.chars().count() <= SHOWN_CHARS, "{}", hit.text.len());
        assert_eq!(&hit.text[hit.ranges[0].clone()], "menu");
    }

    #[test]
    fn should_keep_the_match_bytes_of_the_whole_line() {
        // given
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("long.txt"), format!("{}needle{}\n", "a".repeat(420), "b".repeat(80))).unwrap();

        // when
        let found = found(&[root(dir.path())], &query("needle"), Ignored::Skipped);

        // then
        let hit = &found.hits[0];
        assert_eq!(hit.bytes, 420..426);
        assert!(hit.text.len() < 506, "{}", hit.text.len());
    }

    #[test]
    fn should_stop_when_ticket_goes_stale() {
        // given
        let dir = project();
        let generations = Generations::default();
        let ticket = generations.next();
        generations.next();

        // when
        let found = search(&Search { roots: &[root(dir.path())], limit: MAX_MATCHES, query: &query("menu"), ignored: Ignored::Skipped, excluded: &excluded(&[]) }, &ticket, |_| {});

        // then
        assert!(found.is_none());
    }

    #[test]
    fn should_refuse_a_bad_regex() {
        // when
        let compiled = ContentQuery { text: "(".to_owned(), mode: Matching::Regex }.compile();

        // then
        assert!(compiled.unwrap_err().contains("("));
    }
}
