//! Lines of processes' output that a contents search finds, and finding them again after more output arrived.

use std::ops::Range;

use regex::Regex;
use serde::{Deserialize, Serialize};

use super::content::shown;
use crate::process::ProcessId;

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
    /// lines below it when it was searched: where to look for it again
    pub from_bottom: usize,
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
            let from_bottom = output.lines.len() - 1 - index;
            hits.push(OutputHit { process: output.process, title: output.title.clone(), line, from_bottom, whole: whole.clone(), text, ranges, bytes: first });
        }
    }
    (hits, false)
}

/// Where `hit`'s line is in `lines`, the output now: the line with its text nearest to where it was, counted from
/// the bottom; `None` once no line has its text.
pub fn locate(lines: &[String], hit: &OutputHit) -> Option<usize> {
    let expected = lines.len().saturating_sub(1).saturating_sub(hit.from_bottom);
    lines.iter().enumerate().filter(|(_, line)| **line == hit.whole).map(|(index, _)| index).min_by_key(|index| index.abs_diff(expected))
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let found: Vec<(u64, u32, usize, Range<usize>)> = hits.iter().map(|hit| (hit.process.value(), hit.line, hit.from_bottom, hit.bytes.clone())).collect();
        assert_eq!(found, [(1, 2, 1, 3..6), (2, 1, 0, 0..3)]);
        assert_eq!((hits[0].ranges.clone(), limited), (vec![3..6], false));
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
}
