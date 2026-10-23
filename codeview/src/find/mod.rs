//! Finding a query in a text: compiling it, searching bytes, stepping through the matches. Free of gpui but for the
//! session's memory of the last query.

pub(crate) mod bar;
pub(crate) mod marks;

use std::ops::Range;
use std::sync::Arc;

use gpui::Global;
use regex::bytes::Regex;

use crate::changes::Toward;
use crate::diff::rows::grouped;
use crate::panel::LineSource;

/// The longest query searched, in characters; a longer one is refused as too long.
pub const MAX_QUERY_CHARS: usize = 1_000;

/// Matches kept at most; the count says there are more.
pub const MAX_MATCHES: usize = 100_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FindOptions {
    pub match_case: bool,
    pub whole_word: bool,
    pub regex: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindOption {
    MatchCase,
    WholeWord,
    Regex,
}

impl FindOptions {
    pub fn toggled(self, option: FindOption) -> Self {
        match option {
            FindOption::MatchCase => Self { match_case: !self.match_case, ..self },
            FindOption::WholeWord => Self { whole_word: !self.whole_word, ..self },
            FindOption::Regex => Self { regex: !self.regex, ..self },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FindQuery {
    pub text: String,
    pub options: FindOptions,
}

/// A query made ready to search.
#[derive(Debug, Clone)]
pub enum Compiled {
    Empty,
    /// a regex that does not compile, and why
    Bad(String),
    Ready(Regex),
}

impl FindQuery {
    pub fn compile(&self) -> Compiled {
        if self.text.is_empty() {
            return Compiled::Empty;
        }
        let chars = self.text.chars().count();
        if chars > MAX_QUERY_CHARS {
            return Compiled::Bad(format!("the query has {chars} characters; at most {MAX_QUERY_CHARS}"));
        }
        let FindOptions { match_case, whole_word, regex } = self.options;
        let pattern = if regex { self.text.clone() } else { regex::escape(&self.text) };
        let pattern = if whole_word { format!(r"\b(?:{pattern})\b") } else { pattern };
        let pattern = if match_case { pattern } else { format!("(?i){pattern}") };
        match Regex::new(&pattern) {
            Ok(regex) => Compiled::Ready(regex),
            Err(error) => Compiled::Bad(format!("{:?} is not a regex: {error}", self.text)),
        }
    }
}

/// The matches of a query in a text, in order: their bytes, and the line each starts on (from 0).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Matches {
    pub ranges: Vec<Range<usize>>,
    pub lines: Vec<usize>,
    /// more than [`MAX_MATCHES`] matched
    pub capped: bool,
}

/// The non-empty matches of `regex` in `text`, at most [`MAX_MATCHES`].
pub fn search(text: &[u8], regex: &Regex) -> Matches {
    let mut matches = Matches::default();
    // newlines are counted from the last match on, so the whole text is scanned once
    let (mut line, mut counted) = (0, 0);
    for found in regex.find_iter(text).filter(|found| !found.is_empty()) {
        if matches.ranges.len() == MAX_MATCHES {
            matches.capped = true;
            break;
        }
        line += memchr::memchr_iter(b'\n', &text[counted..found.start()]).count();
        counted = found.start();
        matches.ranges.push(found.range());
        matches.lines.push(line);
    }
    matches
}

/// The matches of each part on its own, in the whole text's bytes and lines.
fn search_parts(bytes: &[u8], starts: &[usize], regex: &Regex) -> Matches {
    let mut matches = Matches::default();
    let mut line = 0;
    for (index, &start) in starts.iter().enumerate() {
        // a part ends before the line break joining it to the next
        let end = starts.get(index + 1).map_or(bytes.len(), |next| next - 1);
        let part = search(&bytes[start..end], regex);
        let room = MAX_MATCHES - matches.ranges.len();
        matches.capped = part.capped || part.ranges.len() > room;
        matches.ranges.extend(part.ranges.into_iter().take(room).map(|range| range.start + start..range.end + start));
        matches.lines.extend(part.lines.into_iter().take(room).map(|within| within + line));
        if matches.capped {
            break;
        }
        line += memchr::memchr_iter(b'\n', &bytes[start..end]).count() + 1;
    }
    matches
}

impl Matches {
    /// The first match starting at or after `offset`, else the first one; `None` when there is none.
    pub fn first_from(&self, offset: usize) -> Option<usize> {
        if self.ranges.is_empty() {
            return None;
        }
        let after = self.ranges.partition_point(|range| range.start < offset);
        Some(if after == self.ranges.len() { 0 } else { after })
    }

    /// The match after (or before) `current`, going round at the ends.
    pub fn stepped(&self, current: usize, toward: Toward) -> usize {
        let count = self.ranges.len().max(1);
        match toward {
            Toward::Next => (current + 1) % count,
            Toward::Previous => (current + count - 1) % count,
        }
    }

    /// `3 of 12`, `No results`, `1 of 100,000+`.
    pub fn count_text(&self, current: Option<usize>) -> String {
        if self.ranges.is_empty() {
            return "No results".to_owned();
        }
        let more = if self.capped { "+" } else { "" };
        format!("{} of {}{more}", current.map_or(0, |current| current + 1), grouped(self.ranges.len()))
    }
}

/// The text a body searches: a file's source, shared, or bytes copied out of an edited text.
#[derive(Clone)]
pub enum FindText {
    Shared(Arc<dyn LineSource>),
    /// held in a `Vec`, so the copy out of a piece table is the only one
    Owned(Arc<Vec<u8>>),
    /// texts joined by line breaks, each starting at its byte of `starts`; no match spans two of them
    Parts { bytes: Arc<Vec<u8>>, starts: Arc<Vec<usize>> },
}

impl FindText {
    /// The matches of `regex`; reads the whole text, so it runs on the background executor.
    pub fn search(&self, regex: &Regex) -> Matches {
        match self {
            Self::Shared(source) => search(&source.bytes(0..source.byte_len()), regex),
            Self::Owned(bytes) => search(bytes, regex),
            Self::Parts { bytes, starts } => search_parts(bytes, starts, regex),
        }
    }
}

/// The last query typed in any find box and its options, for the next box opened this session.
#[derive(Default)]
pub struct FindMemory {
    pub query: FindQuery,
}

impl Global for FindMemory {}

#[cfg(test)]
// expected matches are lists of one range on purpose
#[allow(clippy::single_range_in_vec_init)]
mod tests {
    use super::*;

    fn query(text: &str, options: FindOptions) -> Regex {
        match (FindQuery { text: text.to_owned(), options }).compile() {
            Compiled::Ready(regex) => regex,
            other => panic!("{text:?} did not compile: {other:?}"),
        }
    }

    fn found(text: &[u8], pattern: &str, options: FindOptions) -> Vec<Range<usize>> {
        search(text, &query(pattern, options)).ranges
    }

    const PLAIN: FindOptions = FindOptions { match_case: false, whole_word: false, regex: false };
    const REGEX: FindOptions = FindOptions { regex: true, ..PLAIN };

    #[test]
    fn should_match_plain_text_ignoring_case_unless_asked() {
        // given
        let text = b"Find find FIND";

        // when
        let any_case = found(text, "find", PLAIN);
        let exact = found(text, "find", FindOptions { match_case: true, ..PLAIN });

        // then
        assert_eq!((any_case, exact), (vec![0..4, 5..9, 10..14], vec![5..9]));
    }

    #[test]
    fn should_escape_plain_text_and_read_regex_when_asked() {
        // given
        let text = b"a.b axb";

        // when
        let (plain, regex) = (found(text, "a.b", PLAIN), found(text, "a.b", REGEX));

        // then
        assert_eq!((plain, regex), (vec![0..3], vec![0..3, 4..7]));
    }

    #[test]
    fn should_match_whole_words_only() {
        // given
        let text = b"menu menus Menu";

        // when
        let words = found(text, "menu", FindOptions { whole_word: true, ..PLAIN });

        // then
        assert_eq!(words, [0..4, 11..15]);
    }

    #[test]
    fn should_report_a_bad_regex_and_nothing_for_an_empty_query() {
        // given
        let bad = FindQuery { text: "(".to_owned(), options: REGEX };
        let empty = FindQuery::default();

        // when
        let (bad, empty) = (bad.compile(), empty.compile());

        // then
        assert!(matches!(bad, Compiled::Bad(_)), "{bad:?}");
        assert!(matches!(empty, Compiled::Empty), "{empty:?}");
    }

    #[test]
    fn should_skip_empty_matches_and_count_lines_of_each_match() {
        // given
        let text = b"x\nab\nab";

        // when
        let matches = search(text, &query("^|ab", FindOptions { match_case: true, ..REGEX }));

        // then
        assert_eq!((matches.ranges, matches.lines), (vec![2..4, 5..7], vec![1, 2]));
    }

    #[test]
    fn should_match_across_a_line_break_with_a_regex() {
        // given
        let text = b"a\nb";

        // when
        let matches = search(text, &query(r"a\nb", REGEX));

        // then
        assert_eq!((matches.ranges, matches.lines), (vec![0..3], vec![0]));
    }

    #[test]
    fn should_search_invalid_utf8_as_bytes() {
        // given
        let text = b"caf\xe9 caf";

        // when
        let ranges = found(text, "caf", PLAIN);

        // then
        assert_eq!(ranges, [0..3, 5..8]);
    }

    #[test]
    fn should_stop_at_the_cap() {
        // given
        let text = "a".repeat(MAX_MATCHES + 5);

        // when
        let matches = search(text.as_bytes(), &query("a", PLAIN));

        // then
        assert_eq!((matches.ranges.len(), matches.capped), (MAX_MATCHES, true));
    }

    #[test]
    fn should_go_round_at_both_ends_and_start_from_an_offset() {
        // given
        let matches = Matches { ranges: vec![0..1, 10..11, 20..21], lines: vec![0, 0, 0], capped: false };

        // when
        let starts = (matches.first_from(11), matches.first_from(25), Matches::default().first_from(0));
        let steps = (matches.stepped(2, Toward::Next), matches.stepped(0, Toward::Previous), matches.stepped(0, Toward::Next));

        // then
        assert_eq!((starts, steps), ((Some(2), Some(0), None), (0, 2, 1)));
    }

    #[test]
    fn should_say_the_count() {
        // given
        let twelve = Matches { ranges: vec![0..1; 12], lines: vec![0; 12], capped: false };
        let capped = Matches { ranges: vec![0..1; MAX_MATCHES], lines: vec![0; MAX_MATCHES], capped: true };

        // when
        let texts = [twelve.count_text(Some(2)), Matches::default().count_text(None), capped.count_text(Some(0))];

        // then
        assert_eq!(texts, ["3 of 12", "No results", "1 of 100,000+"]);
    }

    #[test]
    fn should_refuse_a_query_too_long_to_search() {
        // given
        let query = FindQuery { text: "a".repeat(MAX_QUERY_CHARS + 1), options: PLAIN };

        // when
        let compiled = query.compile();

        // then
        assert!(matches!(&compiled, Compiled::Bad(reason) if reason == "the query has 1001 characters; at most 1000"), "{compiled:?}");
    }

    #[test]
    fn should_not_match_across_the_start_of_a_part() {
        // given
        // two parts, "xb" and "a\nb", joined by a line break
        let text = FindText::Parts { bytes: Arc::new(b"xb\na\nb".to_vec()), starts: Arc::new(vec![0, 3]) };

        // when
        let matches = text.search(&query(r"b\na|a\nb", REGEX));

        // then
        assert_eq!((matches.ranges, matches.lines), (vec![3..6], vec![1]));
    }

    #[test]
    fn should_toggle_one_option() {
        // given
        let options = FindOptions::default();

        // when
        let toggled = options.toggled(FindOption::WholeWord).toggled(FindOption::Regex).toggled(FindOption::Regex);

        // then
        assert_eq!(toggled, FindOptions { whole_word: true, ..FindOptions::default() });
    }
}
