//! Lines of processes' output that a contents search finds, and finding them again after more output arrived.

use std::ops::Range;
use std::sync::Arc;
use std::time::{Duration, Instant};

use regex::Regex;
use serde::{Deserialize, Serialize};

use super::content::shown;
use super::sources::HubScope;
use crate::process::ProcessId;
use crate::project::CheckoutRef;
use crate::terminal::LineAt;

/// One process's output, taken on the UI thread for a search off it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputLines {
    pub process: ProcessId,
    pub title: String,
    pub lines: Vec<String>,
}

/// A matching line of a process's output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputHit {
    pub process: ProcessId,
    pub title: String,
    /// from 1, counted from the top of the output when it was searched
    pub line: u32,
    /// the output's first line when it was searched: while it is still first, no line left the scrollback and `line`
    /// is where this one still is
    pub top: String,
    /// the whole line, to find it again by
    pub whole: String,
    /// the line, or at most `content::SHOWN_CHARS` of it around its first match
    pub text: String,
    /// the matches, as byte ranges of `text`
    pub ranges: Vec<Range<usize>>,
    /// the first match, as bytes of `whole`
    pub bytes: Range<usize>,
}

/// The lines of `outputs` matching `regex`, by process then line, at most `limit`; and whether the limit cut them.
pub fn search_output(outputs: &[OutputLines], regex: &Regex, limit: usize) -> (Vec<OutputHit>, bool) {
    let mut hits = Vec::new();
    for output in outputs {
        for (index, whole) in output.lines.iter().enumerate() {
            let ranges: Vec<Range<usize>> = regex.find_iter(whole).map(|found| found.range()).filter(|range| !range.is_empty()).collect();
            let Some(first) = ranges.first().cloned() else {
                continue;
            };
            if hits.len() == limit {
                return (hits, true);
            }
            let (text, ranges) = shown(whole, first.clone(), ranges);
            let line = u32::try_from(index + 1).unwrap_or(u32::MAX);
            let top = output.lines[0].clone();
            hits.push(OutputHit { process: output.process, title: output.title.clone(), line, top, whole: whole.clone(), text, ranges, bytes: first });
        }
    }
    (hits, false)
}

/// How long a hub searches the output it read again rather than reading every terminal anew: typing a query searches
/// once per key.
pub const OUTPUT_FRESH: Duration = Duration::from_secs(2);

/// The output a hub read for a search of `scope`, and when.
#[derive(Debug, Clone)]
pub struct Taken {
    scope: HubScope,
    at: Instant,
    outputs: Arc<Vec<OutputLines>>,
}

impl Taken {
    pub fn new(scope: HubScope, at: Instant, outputs: Vec<OutputLines>) -> Self {
        Self { scope, at, outputs: Arc::new(outputs) }
    }

    /// Whether a search of `scope` at `now` may search this output instead of reading it again.
    pub fn reusable(&self, scope: &HubScope, now: Instant) -> bool {
        self.scope == *scope && now.saturating_duration_since(self.at) < OUTPUT_FRESH
    }

    pub fn outputs(&self) -> Arc<Vec<OutputLines>> {
        self.outputs.clone()
    }
}

pub const PROCESS_ENDED: &str = "This process has ended";
pub const LINE_GONE: &str = "This line is no longer in the output";

/// Where a terminal shows `hit`, given its output's `lines` now (`None` once the process is gone); else why it
/// cannot.
pub fn frame_at<S: AsRef<str>>(lines: Option<&[S]>, hit: &OutputHit) -> Result<LineAt, &'static str> {
    let lines = lines.ok_or(PROCESS_ENDED)?;
    let index = locate(lines, hit).ok_or(LINE_GONE)?;
    Ok(LineAt { index, bytes: hit.bytes.clone() })
}

/// Whether a process working in `checkout` is in the hub's `scope`; one in no checkout only in all projects.
pub fn in_scope(scope: &HubScope, checkout: Option<&CheckoutRef>) -> bool {
    match (scope, checkout) {
        (HubScope::All, _) => true,
        (HubScope::Project(project), Some(checkout)) => checkout.project == *project,
        (HubScope::Worktree(only), Some(checkout)) => only.path == checkout.path,
        (_, None) => false,
    }
}

/// Where `hit`'s line is in `lines`, the output now: where it was while no line left the top of the scrollback; else
/// the line with its text nearest to where it was, one above before one below; `None` once no line has its text.
pub fn locate<S: AsRef<str>>(lines: &[S], hit: &OutputHit) -> Option<usize> {
    let was = hit.line as usize - 1;
    let same = |index: usize| lines.get(index).is_some_and(|line| line.as_ref() == hit.whole);
    if lines.first().is_some_and(|first| first.as_ref() == hit.top) && same(was) {
        return Some(was);
    }
    (0..lines.len()).filter(|index| same(*index)).min_by_key(|index| (index.abs_diff(was), *index > was))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::ProjectId;

    fn output(id: u64, lines: &[&str]) -> OutputLines {
        OutputLines { process: ProcessId::for_tests(id), title: format!("p{id}"), lines: lines.iter().map(|line| (*line).to_owned()).collect() }
    }

    fn hit_at(lines: &[&str], index: usize) -> OutputHit {
        let regex = Regex::new(&regex::escape(lines[index])).unwrap();
        let (hits, _) = search_output(&[output(1, lines)], &regex, 100);
        hits.into_iter().find(|hit| hit.line as usize == index + 1).unwrap()
    }

    fn owned(lines: &[&str]) -> Vec<String> {
        lines.iter().map(|line| (*line).to_owned()).collect()
    }

    #[test]
    fn should_find_matching_output_lines() {
        // given
        let outputs = [output(1, &["ok", "an error here", "fine"]), output(2, &["error: boom"])];

        // when
        let (hits, limited) = search_output(&outputs, &Regex::new("err").unwrap(), 100);

        // then
        let found: Vec<(u64, u32, &str, Range<usize>)> = hits.iter().map(|hit| (hit.process.value(), hit.line, hit.top.as_str(), hit.bytes.clone())).collect();
        assert_eq!(found, [(1, 2, "ok", 3..6), (2, 1, "error: boom", 0..3)]);
        assert_eq!((hits[0].ranges.clone(), limited), (std::iter::once(3..6).collect::<Vec<_>>(), false));
    }

    #[test]
    fn should_stop_at_the_limit() {
        // given
        let outputs = [output(1, &["a1", "a2", "a3"])];

        // when
        let (hits, limited) = search_output(&outputs, &Regex::new("a").unwrap(), 2);

        // then
        assert_eq!((hits.len(), limited), (2, true));
    }

    #[test]
    fn should_locate_an_unchanged_line() {
        // given
        let lines = ["one", "target", "three"];
        let hit = hit_at(&lines, 1);

        // when
        let index = locate(&owned(&lines), &hit);

        // then
        assert_eq!(index, Some(1));
    }

    #[test]
    fn should_locate_a_line_that_moved_up() {
        // given
        let hit = hit_at(&["0", "1", "2", "3", "4", "target", "6", "7", "8", "9"], 5);
        let later = owned(&["3", "4", "target", "6", "7", "8", "9", "10", "11", "12"]);

        // when
        let index = locate(&later, &hit);

        // then
        assert_eq!(index, Some(2));
    }

    #[test]
    fn should_locate_the_searched_copy_after_a_new_one_was_printed() {
        // given
        let hit = hit_at(&["a", "error", "b", "c"], 1);
        let later = owned(&["a", "error", "b", "c", "d", "error", "e"]);

        // when
        let index = locate(&later, &hit);

        // then
        assert_eq!(index, Some(1));
    }

    #[test]
    fn should_locate_the_nearest_of_repeated_lines() {
        // given
        let lines = ["progress", "a", "b", "progress", "c", "d", "progress"];
        let hit = hit_at(&lines, 3);

        // when
        let index = locate(&owned(&lines), &hit);

        // then
        assert_eq!(index, Some(3));
    }

    #[test]
    fn should_not_locate_a_line_gone() {
        // given
        let hit = hit_at(&["gone", "b"], 0);

        // when
        let index = locate(&owned(&["b", "c"]), &hit);

        // then
        assert_eq!(index, None);
    }

    fn checkout(project: &str, path: &str) -> CheckoutRef {
        CheckoutRef { project: ProjectId::parse(project).unwrap(), path: std::path::PathBuf::from(path) }
    }

    #[test]
    fn should_search_a_process_of_the_scope_worktree() {
        // given
        let scope = HubScope::Worktree(checkout("ive", "/w/ive"));

        // when
        let searched = [in_scope(&scope, Some(&checkout("ive", "/w/ive"))), in_scope(&scope, Some(&checkout("ive", "/w/ive-other")))];

        // then
        assert_eq!(searched, [true, false]);
    }

    #[test]
    fn should_skip_a_process_of_another_project() {
        // given
        let scope = HubScope::Project(ProjectId::parse("ive").unwrap());

        // when
        let searched = [in_scope(&scope, Some(&checkout("ive", "/w/ive-x"))), in_scope(&scope, Some(&checkout("notes", "/w/notes")))];

        // then
        assert_eq!(searched, [true, false]);
    }

    #[test]
    fn should_search_a_process_without_checkout_only_in_all() {
        // given
        let scopes = [HubScope::All, HubScope::Project(ProjectId::parse("ive").unwrap())];

        // when
        let searched = scopes.map(|scope| in_scope(&scope, None));

        // then
        assert_eq!(searched, [true, false]);
    }

    #[test]
    fn should_look_at_the_located_line() {
        // given
        let lines = ["one", "target", "three"];
        let hit = hit_at(&lines, 1);

        // when
        let at = frame_at(Some(&owned(&lines)), &hit);

        // then
        assert_eq!(at, Ok(LineAt { index: 1, bytes: 0..6 }));
    }

    #[test]
    fn should_say_a_removed_process_has_ended() {
        // given
        let hit = hit_at(&["target"], 0);

        // when
        let at = frame_at(None::<&[String]>, &hit);

        // then
        assert_eq!(at, Err(PROCESS_ENDED));
    }

    #[test]
    fn should_say_a_line_gone_from_the_output() {
        // given
        let hit = hit_at(&["target"], 0);

        // when
        let at = frame_at(Some(&owned(&["other"])), &hit);

        // then
        assert_eq!(at, Err(LINE_GONE));
    }

    #[test]
    fn should_reuse_output_taken_for_the_same_scope_a_moment_ago() {
        // given
        let now = std::time::Instant::now();
        let taken = Taken::new(HubScope::All, now, Vec::new());

        // when
        let reused = [taken.reusable(&HubScope::All, now + Duration::from_millis(500)), taken.reusable(&HubScope::All, now + OUTPUT_FRESH)];

        // then
        assert_eq!(reused, [true, false]);
    }

    #[test]
    fn should_take_output_again_for_another_scope() {
        // given
        let now = std::time::Instant::now();
        let taken = Taken::new(HubScope::All, now, Vec::new());

        // when
        let reused = taken.reusable(&HubScope::Project(ProjectId::parse("ive").unwrap()), now);

        // then
        assert!(!reused);
    }
}
