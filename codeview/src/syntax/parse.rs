use std::fmt;
use std::ops::ControlFlow;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use tree_sitter::{InputEdit, ParseOptions, ParseState, Parser, Range, Tree};

use super::Language;
use super::injection::{Injection, injections};

/// Above this a file is shown plain: the parse would take seconds and the tree several times the file's memory.
pub const MAX_HIGHLIGHT_BYTES: usize = 8 * 1024 * 1024;
/// A parse still running after this is abandoned (an 8 MB Java file parses in about 1 s). It covers the host and
/// all embedded parses together.
pub const PARSE_BUDGET: Duration = Duration::from_secs(2);
/// Embedded code nested deeper than this below the file's own language shows in its parent layer's colours.
pub const MAX_INJECTION_DEPTH: usize = 3;

/// A parsed document: the host tree, then one layer per piece of embedded code, parents before children. A clone
/// shares the trees.
#[derive(Clone)]
pub struct Syntax {
    layers: Vec<Layer>,
}

/// One parse: of the whole document (depth 0), or of embedded code within `ranges`.
#[derive(Clone)]
pub struct Layer {
    pub language: Language,
    pub tree: Tree,
    pub depth: usize,
    /// sorted, disjoint; empty for the host, which covers the whole document
    pub ranges: Vec<Range>,
}

impl Syntax {
    #[cfg(test)]
    pub fn language(&self) -> Language {
        self.layers[0].language
    }

    pub fn layers(&self) -> &[Layer] {
        &self.layers
    }
}

impl Layer {
    pub fn overlaps(&self, bytes: &std::ops::Range<usize>) -> bool {
        match (self.ranges.first(), self.ranges.last()) {
            (Some(first), Some(last)) => first.start_byte < bytes.end && bytes.start < last.end_byte,
            _ => true,
        }
    }
}

/// Set once a parse result is no longer wanted, e.g. its tab closed; the parse then stops at its next check.
#[derive(Clone, Default)]
pub struct Cancel(Arc<AtomicBool>);

impl Cancel {
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Relaxed);
    }

    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}

#[derive(Debug)]
pub enum ParseError {
    TooLarge { bytes: usize },
    Grammar(String),
    Cancelled,
    OverBudget { elapsed: Duration },
    Failed,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooLarge { bytes } => write!(f, "{bytes} bytes is over the highlighting limit of {MAX_HIGHLIGHT_BYTES}"),
            Self::Grammar(reason) => write!(f, "{reason}"),
            Self::Cancelled => write!(f, "parse cancelled"),
            Self::OverBudget { elapsed } => {
                write!(f, "parse abandoned after {elapsed:?} (budget {PARSE_BUDGET:?})")
            }
            Self::Failed => write!(f, "tree-sitter returned no tree"),
        }
    }
}

/// When parsing has to stop: `cancel` set, or `budget` used up since `started`.
struct Deadline<'a> {
    cancel: &'a Cancel,
    started: Instant,
    budget: Duration,
}

impl Deadline<'_> {
    fn is_over(&self) -> bool {
        self.cancel.is_cancelled() || self.started.elapsed() > self.budget
    }

    fn error(&self) -> ParseError {
        if self.cancel.is_cancelled() {
            return ParseError::Cancelled;
        }
        let elapsed = self.started.elapsed();
        if elapsed > self.budget {
            return ParseError::OverBudget { elapsed };
        }
        ParseError::Failed
    }
}

/// Parses `source` as `language`, then the code embedded in it; gives up when `cancel` is set or after `budget`.
/// Running out of budget among the embedded parses keeps the layers finished so far.
pub fn parse(source: &[u8], language: Language, cancel: &Cancel, budget: Duration) -> Result<Syntax, ParseError> {
    if source.len() > MAX_HIGHLIGHT_BYTES {
        return Err(ParseError::TooLarge { bytes: source.len() });
    }
    let deadline = Deadline { cancel, started: Instant::now(), budget };
    let tree = parse_text(&Text { source, language, ranges: &[] }, None, &deadline)?;
    with_embedded(Layer { language, tree, depth: 0, ranges: Vec::new() }, source, &deadline)
}

/// Parses `source`, the text `old` was parsed from changed by `edits` (in order, each in the offsets the ones before
/// it left), reusing what the edits left of `old`'s host tree; the embedded code is found and parsed afresh. Gives up
/// as [`parse`] does.
pub fn reparse(old: &Syntax, edits: &[InputEdit], source: &[u8], cancel: &Cancel, budget: Duration) -> Result<Syntax, ParseError> {
    if source.len() > MAX_HIGHLIGHT_BYTES {
        return Err(ParseError::TooLarge { bytes: source.len() });
    }
    let language = old.layers[0].language;
    let mut edited = old.layers[0].tree.clone();
    for edit in edits {
        edited.edit(edit);
    }
    let deadline = Deadline { cancel, started: Instant::now(), budget };
    let tree = parse_text(&Text { source, language, ranges: &[] }, Some(&edited), &deadline)?;
    with_embedded(Layer { language, tree, depth: 0, ranges: Vec::new() }, source, &deadline)
}

/// `host` and the layers of the code embedded in it, parents before children.
fn with_embedded(host: Layer, source: &[u8], deadline: &Deadline) -> Result<Syntax, ParseError> {
    let language = host.language;
    let mut layers = vec![host];
    let mut next = 0;
    while let Some(parent) = layers.get(next) {
        let depth = parent.depth + 1;
        for injection in embedded_in(parent, source, depth) {
            match parse_injection(source, &injection, deadline) {
                Ok(tree) => layers.push(Layer { language: injection.language, tree, depth, ranges: injection.ranges }),
                Err(ParseError::Cancelled) => return Err(ParseError::Cancelled),
                Err(error @ ParseError::OverBudget { .. }) => return Ok(stopped(layers, &error)),
                Err(error) => eprintln!("ive: {:?} embedded in {language:?} shown in host colours: {error}", injection.language),
            }
        }
        next += 1;
    }
    Ok(Syntax { layers })
}

fn parse_injection(source: &[u8], injection: &Injection, deadline: &Deadline) -> Result<Tree, ParseError> {
    // tree-sitter checks the deadline only now and then, so small embedded parses would all slip through it
    if deadline.is_over() {
        return Err(deadline.error());
    }
    parse_text(&Text { source, language: injection.language, ranges: &injection.ranges }, None, deadline)
}

/// The injections of `parent` that may still nest at `depth`.
fn embedded_in(parent: &Layer, source: &[u8], depth: usize) -> Vec<Injection> {
    if depth > MAX_INJECTION_DEPTH {
        return Vec::new();
    }
    match parent.language.grammar() {
        Ok(grammar) => grammar.injections.as_ref().map(|query| injections(parent, query, source)).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

fn stopped(layers: Vec<Layer>, error: &ParseError) -> Syntax {
    eprintln!("ive: some code embedded in {:?} shown in host colours: {error}", layers[0].language);
    Syntax { layers }
}

/// What to parse: the `ranges` of `source` as one document in `language`; no ranges is the whole source.
struct Text<'a> {
    source: &'a [u8],
    language: Language,
    ranges: &'a [Range],
}

/// Parses `text`, reusing what is still valid of `old`, a tree already edited to fit it.
fn parse_text(text: &Text, old: Option<&Tree>, deadline: &Deadline) -> Result<Tree, ParseError> {
    let Text { source, language, ranges } = *text;
    let grammar = language.grammar().map_err(|reason| ParseError::Grammar(reason.to_owned()))?;
    let mut parser = Parser::new();
    parser.set_language(&grammar.language).map_err(|error| ParseError::Grammar(error.to_string()))?;
    parser.set_included_ranges(ranges).map_err(|error| ParseError::Grammar(format!("{language:?} ranges: {error:?}")))?;
    let mut progress = |_: &ParseState| {
        if deadline.is_over() {
            return ControlFlow::Break(());
        }
        ControlFlow::Continue(())
    };
    let options = ParseOptions::new().progress_callback(&mut progress);
    let tree = parser.parse_with_options(&mut |at, _| source.get(at..).unwrap_or_default(), old, Some(options));
    tree.ok_or_else(|| deadline.error())
}

#[cfg(test)]
mod tests {
    use tree_sitter::Point;

    use super::*;
    use crate::syntax::{lang, spans};

    /// About 3 MB of Java: long enough that tree-sitter checks progress several times.
    fn big_java() -> Vec<u8> {
        (0..40_000).map(|n| format!("class C{n} {{ static final int X = {n}; void f() {{ g(\"s\"); }} }}\n")).collect::<String>().into_bytes()
    }

    #[test]
    fn should_parse_small_file() {
        // given
        let source = b"fn main() {}";

        // when
        let syntax = parse(source, lang("Rust"), &Cancel::default(), PARSE_BUDGET);

        // then
        assert_eq!(syntax.map(|syntax| syntax.language()).ok(), Some(lang("Rust")));
    }

    #[test]
    fn should_colour_a_reparse_like_a_fresh_parse() {
        // given
        let old_text = "fn main() {\n    let n = 1;\n    call(n);\n}\n";
        let old = parse(old_text.as_bytes(), lang("Rust"), &Cancel::default(), PARSE_BUDGET).unwrap();
        let at = old_text.find("call").unwrap();
        let new_text = format!("{}// {}", &old_text[..at], &old_text[at..]);
        let point = Point { row: 2, column: 4 };
        let edit = InputEdit {
            start_byte: at,
            old_end_byte: at,
            new_end_byte: at + 3,
            start_position: point,
            old_end_position: point,
            new_end_position: Point { row: 2, column: 7 },
        };

        // when
        let reparsed = reparse(&old, &[edit], new_text.as_bytes(), &Cancel::default(), PARSE_BUDGET).unwrap();

        // then
        let fresh = parse(new_text.as_bytes(), lang("Rust"), &Cancel::default(), PARSE_BUDGET).unwrap();
        let whole = 0..new_text.len();
        assert_eq!(spans(&reparsed, new_text.as_bytes(), whole.clone()), spans(&fresh, new_text.as_bytes(), whole));
    }

    #[test]
    fn should_refuse_file_over_limit_without_parsing() {
        // given
        let source = vec![b' '; MAX_HIGHLIGHT_BYTES + 1];

        // when
        let result = parse(&source, lang("JSON"), &Cancel::default(), PARSE_BUDGET);

        // then
        assert!(matches!(result, Err(ParseError::TooLarge { bytes }) if bytes == MAX_HIGHLIGHT_BYTES + 1));
    }

    #[test]
    fn should_stop_when_cancelled() {
        // given
        let cancel = Cancel::default();
        cancel.cancel();

        // when
        let result = parse(&big_java(), lang("Java"), &cancel, PARSE_BUDGET);

        // then
        assert!(matches!(result, Err(ParseError::Cancelled)));
    }

    #[test]
    fn should_give_up_past_budget() {
        // given
        let budget = Duration::ZERO;

        // when
        let result = parse(&big_java(), lang("Java"), &Cancel::default(), budget);

        // then
        assert!(matches!(result, Err(ParseError::OverBudget { .. })));
    }
}
