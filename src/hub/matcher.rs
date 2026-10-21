//! fzf's v1 matching (`algo.go`, `--scheme=path`) with its extended search syntax, as `mock/hub.js` has it.

use std::cmp::Ordering;

// fzf's algo.go constants; the path scheme makes `/` (and the start) the strongest boundary.
const SCORE_MATCH: i32 = 16;
const GAP_START: i32 = -3;
const GAP_EXTENSION: i32 = -1;
const BONUS_BOUNDARY: i32 = 8;
const BONUS_SLASH: i32 = 10;
const BONUS_CAMEL: i32 = 7;
const BONUS_CONSECUTIVE: i32 = 4;
const FIRST_CHAR_MULTIPLIER: i32 = 2;
// Not fzf: a term matched inside the file name beats one spread over folders ("theme" ranks theme.rs first).
const BONUS_FILE_NAME: i32 = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Fuzzy,
    Exact,
    Prefix,
    Suffix,
    Equal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Term {
    kind: Kind,
    inverse: bool,
    case_sensitive: bool,
    pattern: Vec<char>,
}

/// A parsed query: every group must match, a group by any of its terms.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Query {
    groups: Vec<Vec<Term>>,
}

/// A hit: its score and the matched characters' positions (char indices, sorted, each once).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Match {
    pub score: i32,
    pub positions: Vec<usize>,
}

/// A query with a trailing `:<line>` cut off, as `theme.rs:3`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineQuery<'a> {
    pub query: &'a str,
    pub line: Option<u32>,
}

impl Query {
    /// fzf's extended syntax: space = AND, `a | b` = OR, `'exact`, `^prefix`, `suffix$`, `^equal$`, `!not`; a term
    /// with an uppercase letter matches case-sensitively.
    pub fn parse(text: &str) -> Query {
        let mut groups: Vec<Vec<Term>> = Vec::new();
        let mut join_next = false;
        for token in text.split_whitespace() {
            if token == "|" {
                join_next = !groups.is_empty();
                continue;
            }
            let term = parse_term(token);
            match groups.last_mut() {
                Some(group) if join_next => group.push(term),
                _ => groups.push(vec![term]),
            }
            join_next = false;
        }
        Query { groups }
    }

    pub fn is_empty(&self) -> bool {
        self.groups.is_empty()
    }

    /// `lower` is `text` lowercased char by char (as [`lowered`]); `name_at` is the byte index where the file name
    /// starts, for the file-name bonus, or `None` for text that is not a path.
    pub fn matches(&self, text: &str, lower: &str, name_at: Option<usize>) -> Option<Match> {
        if text.is_ascii() && lower.len() == text.len() {
            return self.matches_in(text.as_bytes(), lower.as_bytes(), name_at);
        }
        let chars: Vec<char> = text.chars().collect();
        let lower: Vec<char> = lowered(text).chars().collect();
        let name_at = name_at.map(|byte| text.get(..byte).map_or(chars.len(), |head| head.chars().count()));
        self.matches_in(&chars, &lower, name_at)
    }

    fn matches_in<C: Letter>(&self, text: &[C], lower: &[C], name_at: Option<usize>) -> Option<Match> {
        let mut score = 0;
        let mut positions = Vec::new();
        for group in &self.groups {
            let best = group
                .iter()
                .filter_map(|term| {
                    let mut hit = term.matches(text, lower)?;
                    if name_at.is_some_and(|name_at| hit.positions.first().is_some_and(|first| *first >= name_at)) {
                        hit.score += BONUS_FILE_NAME;
                    }
                    Some(hit)
                })
                .max_by_key(|hit| hit.score)?;
            score += best.score;
            positions.extend(best.positions);
        }
        positions.sort_unstable();
        positions.dedup();
        Some(Match { score, positions })
    }
}

/// `text` lowercased char by char, so its char positions stay those of `text`.
pub fn lowered(text: &str) -> String {
    text.chars().map(|c| c.to_lowercase().next().unwrap_or(c)).collect()
}

/// `file.rs:12` is `file.rs` at line 12; `:12` alone is left whole (it is the hub's line jump).
pub fn split_line(text: &str) -> LineQuery<'_> {
    let trimmed = text.trim();
    let whole = LineQuery { query: text, line: None };
    let Some((head, digits)) = trimmed.rsplit_once(':') else {
        return whole;
    };
    let ends_in_text = head.chars().next_back().is_some_and(|last| !last.is_whitespace());
    let is_number = !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit());
    match digits.parse::<u32>() {
        Ok(line) if ends_in_text && is_number => LineQuery { query: head, line: Some(line) },
        _ => whole,
    }
}

/// fzf's `--scheme=path` tiebreak: score, then the shorter path, then the earlier in the list.
pub fn rank(a: (&Match, &str, usize), b: (&Match, &str, usize)) -> Ordering {
    b.0.score.cmp(&a.0.score).then(a.1.len().cmp(&b.1.len())).then(a.2.cmp(&b.2))
}

fn parse_term(token: &str) -> Term {
    let mut kind = Kind::Fuzzy;
    let mut inverse = false;
    let mut text = token;
    if let Some(rest) = text.strip_prefix('!') {
        (inverse, kind, text) = (true, Kind::Exact, rest);
    }
    if let Some(rest) = text.strip_prefix('\'') {
        (kind, text) = (Kind::Exact, rest);
    }
    if let Some(rest) = text.strip_prefix('^') {
        (kind, text) = (Kind::Prefix, rest);
    }
    if text.chars().count() > 1
        && let Some(rest) = text.strip_suffix('$')
    {
        kind = if kind == Kind::Prefix { Kind::Equal } else { Kind::Suffix };
        text = rest;
    }
    let case_sensitive = text.chars().any(char::is_uppercase);
    let pattern = if case_sensitive { text.chars().collect() } else { lowered(text).chars().collect() };
    Term { kind, inverse, case_sensitive, pattern }
}

/// A character of matched text: a byte of an ASCII path, or a `char`.
trait Letter: Copy {
    fn letter(self) -> char;
}

impl Letter for u8 {
    fn letter(self) -> char {
        char::from(self)
    }
}

impl Letter for char {
    fn letter(self) -> char {
        self
    }
}

impl Term {
    fn matches<C: Letter>(&self, text: &[C], lower: &[C]) -> Option<Match> {
        let hay = if self.case_sensitive { text } else { lower };
        if self.pattern.is_empty() {
            return Some(Match { score: 0, positions: Vec::new() });
        }
        let hit = match self.kind {
            Kind::Fuzzy => self.fuzzy(text, hay),
            _ => self.exact(text, hay),
        };
        match (self.inverse, hit) {
            (false, hit) => hit,
            (true, None) => Some(Match { score: 0, positions: Vec::new() }),
            (true, Some(_)) => None,
        }
    }

    /// The leftmost match, its start pulled right as far as the pattern allows, scored along that window.
    fn fuzzy<C: Letter>(&self, text: &[C], hay: &[C]) -> Option<Match> {
        let pattern = &self.pattern;
        let mut next = 0;
        let mut start = None;
        let mut end = None;
        for (index, letter) in hay.iter().enumerate() {
            if letter.letter() != pattern[next] {
                continue;
            }
            start.get_or_insert(index);
            next += 1;
            if next == pattern.len() {
                end = Some(index);
                break;
            }
        }
        let (mut start, end) = (start?, end?);
        let mut left = pattern.len();
        for index in (start..=end).rev() {
            if hay[index].letter() == pattern[left - 1] {
                left -= 1;
                if left == 0 {
                    start = index;
                    break;
                }
            }
        }
        Some(score_window(text, hay, pattern, start, end))
    }

    fn exact<C: Letter>(&self, text: &[C], hay: &[C]) -> Option<Match> {
        let pattern = &self.pattern;
        let length = pattern.len();
        let equal_at = |at: usize| hay[at..at + length].iter().zip(pattern).all(|(letter, wanted)| letter.letter() == *wanted);
        if length > hay.len() {
            return None;
        }
        let at = match self.kind {
            Kind::Prefix => equal_at(0).then_some(0),
            Kind::Suffix => equal_at(hay.len() - length).then_some(hay.len() - length),
            Kind::Equal => (hay.len() == length && equal_at(0)).then_some(0),
            _ => (0..=hay.len() - length).find(|at| equal_at(*at)),
        }?;
        Some(score_window(text, hay, pattern, at, at + length - 1))
    }
}

fn score_window<C: Letter>(text: &[C], hay: &[C], pattern: &[char], start: usize, end: usize) -> Match {
    let mut score = 0;
    let mut positions = Vec::with_capacity(pattern.len());
    let mut next = 0;
    let mut run = 0;
    let mut in_gap = false;
    let mut first_bonus = 0;
    for (index, letter) in hay.iter().enumerate().take(end + 1).skip(start) {
        if next < pattern.len() && letter.letter() == pattern[next] {
            let mut bonus = bonus_at(text, index);
            if run == 0 {
                first_bonus = bonus;
            } else {
                bonus = bonus.max(first_bonus).max(BONUS_CONSECUTIVE);
            }
            score += SCORE_MATCH + if next == 0 { bonus * FIRST_CHAR_MULTIPLIER } else { bonus };
            positions.push(index);
            next += 1;
            run += 1;
            in_gap = false;
        } else {
            score += if in_gap { GAP_EXTENSION } else { GAP_START };
            in_gap = true;
            run = 0;
        }
    }
    Match { score, positions }
}

fn bonus_at<C: Letter>(text: &[C], index: usize) -> i32 {
    let Some(previous) = index.checked_sub(1).map(|before| text[before].letter()) else {
        return BONUS_SLASH;
    };
    let current = text[index].letter();
    match previous {
        '/' => BONUS_SLASH,
        '-' | '_' | '.' | ' ' => BONUS_BOUNDARY,
        _ if (previous.is_lowercase() || previous.is_ascii_digit()) && current.is_uppercase() => BONUS_CAMEL,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn matched(query: &str, text: &str) -> Option<Match> {
        Query::parse(query).matches(text, &text.to_lowercase(), Some(text.rfind('/').map_or(0, |slash| slash + 1)))
    }

    fn ranked<'a>(query: &str, paths: &[&'a str]) -> Vec<&'a str> {
        let query = Query::parse(query);
        let mut hits: Vec<_> = paths
            .iter()
            .enumerate()
            .filter_map(|(index, path)| {
                let name_at = path.rfind('/').map_or(0, |slash| slash + 1);
                query.matches(path, &path.to_lowercase(), Some(name_at)).map(|hit| (hit, *path, index))
            })
            .collect();
        hits.sort_by(|a, b| rank((&a.0, a.1, a.2), (&b.0, b.1, b.2)));
        hits.into_iter().map(|(_, path, _)| path).collect()
    }

    #[test]
    fn should_score_a_consecutive_match_from_the_start() {
        // given
        let query = Query::parse("ab");

        // when
        let hit = query.matches("ab", "ab", Some(0));

        // then
        assert_eq!(hit, Some(Match { score: 74, positions: vec![0, 1] }));
    }

    #[test]
    fn should_score_a_match_across_a_boundary() {
        // given
        let query = Query::parse("ab");

        // when
        let hit = query.matches("a-b", "a-b", Some(0));

        // then
        assert_eq!(hit, Some(Match { score: 69, positions: vec![0, 2] }));
    }

    #[test]
    fn should_pull_the_match_start_right() {
        // given
        let text = "a/xab";

        // when
        let hit = matched("ab", text);

        // then
        assert_eq!(hit.map(|hit| hit.positions), Some(vec![3, 4]));
    }

    #[test]
    fn should_rank_a_file_name_match_above_a_folder_match() {
        // given
        let paths = ["assets/theme/default.toml", "src/theme.rs"];

        // when
        let order = ranked("theme", &paths);

        // then
        assert_eq!(order, ["src/theme.rs", "assets/theme/default.toml"]);
    }

    #[test]
    fn should_rank_the_shorter_path_on_equal_scores() {
        // given
        let paths = ["src/workspace/tab_menu.rs", "src/tab_menu.rs"];

        // when
        let order = ranked("tabmenu", &paths);

        // then
        assert_eq!(order, ["src/tab_menu.rs", "src/workspace/tab_menu.rs"]);
    }

    #[test]
    fn should_match_case_sensitively_with_an_uppercase_letter() {
        // given
        let paths = ["internal-apps/communication/idp/src/main/java/ovh/atlasinc/idp/api/grants/ApiTokenService.java", "apitoken.txt"];

        // when
        let order = ranked("ApiTok", &paths);

        // then
        assert_eq!(order, [paths[0]]);
    }

    #[test]
    fn should_apply_the_extended_syntax() {
        // given
        let paths = [
            "internal-apps/communication/idp/src/main/jte/page.jte",
            "internal-apps/communication/idp/src/test/jte/page.jte",
            "misc/internal-apps/communication/page.jte",
        ];

        // when
        let order = ranked("'jte ^internal-apps/comm !test", &paths);

        // then
        assert_eq!(order, [paths[0]]);
    }

    #[test]
    fn should_match_a_suffix() {
        // given
        let paths = ["internal-apps/communication/idp/src/main/jte/login.jte", "internal-apps/communication/idp/jte/README.md"];

        // when
        let order = ranked("idp jte$", &paths);

        // then
        assert_eq!(order, [paths[0]]);
    }

    #[test]
    fn should_match_either_side_of_a_bar() {
        // given
        let paths = ["a.toml", "b.json", "c.yaml"];

        // when
        let order = ranked("toml$ | json$", &paths);

        // then
        assert_eq!(order, ["a.toml", "b.json"]);
    }

    #[test]
    fn should_split_a_line_number() {
        // given
        let queries = ["theme.rs:3", "a:b", ":42"];

        // when
        let split = queries.map(|query| {
            let split = split_line(query);
            (split.query, split.line)
        });

        // then
        assert_eq!(split, [("theme.rs", Some(3)), ("a:b", None), (":42", None)]);
    }

    #[test]
    fn should_match_everything_with_an_empty_query() {
        // given
        let query = Query::parse("  ");

        // when
        let hit = query.matches("x", "x", None);

        // then
        assert!(query.is_empty());
        assert_eq!(hit, Some(Match { score: 0, positions: vec![] }));
    }

    #[test]
    fn should_give_char_positions_in_a_non_ascii_path() {
        // given
        let text = "źródło/plik.rs";

        // when
        let hit = matched("plik", text);

        // then
        assert_eq!(hit.map(|hit| hit.positions), Some(vec![7, 8, 9, 10]));
    }
}
