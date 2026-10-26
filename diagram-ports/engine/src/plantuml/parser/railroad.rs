//! Family-owned borrowed lexemes and checked generated railroad parser admission.
use super::{BodySource, context, error::ParseFailure, structured::Cursor};
use crate::plantuml::budget::{Budget, bound};

#[derive(Clone, Debug)]
pub(in crate::plantuml) enum Token<'s> {
    Word(&'s str),
    Text((&'s str, char)),
    Char(char),
    Escape(&'s str),
    Number(usize),
    Eq,
    Semi,
    Comma,
    Pipe,
    Open,
    Close,
    Square,
    EndSquare,
    Brace,
    EndBrace,
    Minus,
    Star,
    Plus,
    Question,
    Dot,
    Negate,
    Noncapture,
}
pub(in crate::plantuml) type Spanned<'s> = (usize, Token<'s>, usize);
#[derive(Clone, Copy, PartialEq, Eq)]
pub(in crate::plantuml) enum Family {
    Ebnf,
    Regex,
}

pub(in crate::plantuml) struct Storage {
    bytes: usize,
    limit: usize,
}
impl Storage {
    pub fn reserve<T>(
        &mut self,
        values: &mut Vec<T>,
        capacity: usize,
        budget: &mut Budget,
    ) -> Result<(), ParseFailure> {
        if capacity <= values.capacity() {
            return Ok(());
        }
        let old = values.capacity().saturating_mul(std::mem::size_of::<T>());
        let growth = capacity.saturating_mul(std::mem::size_of::<T>());
        bound(
            "railroad retained arena bytes",
            self.bytes.saturating_add(growth),
            self.limit,
        )?;
        budget.charge(values.len() + capacity)?;
        values
            .try_reserve_exact(capacity - values.len())
            .map_err(|error| {
                ParseFailure::terminal(crate::RenderError::Backend(format!(
                    "PlantUML railroad arena allocation failed: {error}"
                )))
            })?;
        self.bytes = self
            .bytes
            .saturating_add(values.capacity().saturating_mul(std::mem::size_of::<T>()))
            .saturating_sub(old);
        Ok(())
    }
    pub fn push<T>(
        &mut self,
        values: &mut Vec<T>,
        value: T,
        budget: &mut Budget,
    ) -> Result<(), ParseFailure> {
        if values.len() == values.capacity() {
            let capacity = values.capacity().saturating_mul(2).max(4);
            self.reserve(values, capacity, budget)?;
        }
        budget.charge(1)?;
        values.push(value);
        Ok(())
    }
}

pub(in crate::plantuml) fn lex<'s>(
    body: &BodySource<'s>,
    budget: &mut Budget,
    family: Family,
) -> Result<Vec<Spanned<'s>>, ParseFailure> {
    let mut cursor = Cursor::new(body, budget);
    let mut tokens = Vec::new();
    let mut depth = 0usize;
    let mut class = false;
    let mut count = false;
    // The document body includes framing newlines. Only these are removed for regex;
    // ordinary leading/internal/trailing spaces remain literal expression characters.
    if family == Family::Regex {
        while cursor.peek().is_some_and(|c| matches!(c, '\r' | '\n')) {
            cursor.take()?;
        }
        while cursor.input[..cursor.end].ends_with(['\n', '\r']) {
            cursor.end -= 1;
        }
    }
    while let Some(ch) = cursor.peek() {
        let start = cursor.offset;
        if family == Family::Ebnf && ch.is_whitespace() {
            cursor.take()?;
            continue;
        }
        let token = if family == Family::Ebnf {
            ebnf_token(&mut cursor)?
        } else {
            regex_token(&mut cursor, (class, count))?
        };
        if family == Family::Ebnf
            && matches!(token, Token::Minus)
            && !tokens
                .last()
                .is_some_and(|(_, token, end)| matches!(token, Token::EndBrace) && *end == start)
        {
            return Err(ParseFailure::unsupported(
                start..cursor.offset,
                "EBNF exception-minus",
            ));
        }
        if family == Family::Regex
            && !class
            && matches!(token, Token::Plus | Token::Question)
            && tokens.last().is_some_and(|(_, token, _)| {
                matches!(
                    token,
                    Token::Star | Token::Plus | Token::Question | Token::EndBrace
                )
            })
        {
            return Err(ParseFailure::unsupported(
                start..cursor.offset,
                "regex lazy or possessive quantifier",
            ));
        }
        if matches!(
            token,
            Token::Open | Token::Square | Token::Brace | Token::Noncapture
        ) && !class
            && !count
        {
            depth += 1;
            cursor.budget.depth(depth)?;
        }
        if matches!(token, Token::Close | Token::EndSquare | Token::EndBrace) {
            depth = depth.saturating_sub(1);
        }
        if family == Family::Regex {
            match token {
                Token::Square => class = true,
                Token::EndSquare => class = false,
                Token::Brace if !class => count = true,
                Token::EndBrace => count = false,
                _ => {}
            }
        }
        context::push(&mut tokens, (start, token, cursor.offset), cursor.budget)?;
    }
    Ok(tokens)
}

fn ebnf_token<'s>(cursor: &mut Cursor<'s, '_>) -> Result<Token<'s>, ParseFailure> {
    let start = cursor.offset;
    let ch = cursor.take()?;
    if matches!(ch, '\'' | '"') {
        let from = cursor.offset;
        while let Some(next) = cursor.peek() {
            cursor.take()?;
            if next == ch {
                return Ok(Token::Text((&cursor.input[from..cursor.offset - 1], ch)));
            }
        }
        return Err(ParseFailure::syntax(
            start..cursor.offset,
            "unterminated EBNF terminal",
        ));
    }
    if ch.is_alphanumeric() || ch == '_' {
        while cursor
            .peek()
            .is_some_and(|c| c.is_alphanumeric() || matches!(c, '_' | '-'))
        {
            cursor.take()?;
        }
        return Ok(Token::Word(&cursor.input[start..cursor.offset]));
    }
    match ch {
        '=' => Ok(Token::Eq),
        ';' => Ok(Token::Semi),
        ',' => Ok(Token::Comma),
        '|' => Ok(Token::Pipe),
        '(' if cursor.peek() == Some('*') => Err(ParseFailure::unsupported(
            start..cursor.offset + 1,
            "EBNF comment annotations",
        )),
        '(' => Ok(Token::Open),
        ')' => Ok(Token::Close),
        '[' => Ok(Token::Square),
        ']' => Ok(Token::EndSquare),
        '{' => Ok(Token::Brace),
        '}' => Ok(Token::EndBrace),
        '-' => Ok(Token::Minus),
        '*' => Err(ParseFailure::unsupported(
            start..cursor.offset,
            "EBNF numeric repetition",
        )),
        '?' => Err(ParseFailure::unsupported(
            start..cursor.offset,
            "EBNF special sequence",
        )),
        _ => Err(ParseFailure::syntax(
            start..cursor.offset,
            "invalid EBNF token",
        )),
    }
}

fn regex_token<'s>(
    cursor: &mut Cursor<'s, '_>,
    mode: (bool, bool),
) -> Result<Token<'s>, ParseFailure> {
    let (class, count) = mode;
    let start = cursor.offset;
    let ch = cursor.take()?;
    if matches!(ch, '\r' | '\n') {
        return Err(ParseFailure::unsupported(
            start..cursor.offset,
            "multiple regex expression lines",
        ));
    }
    if ch == '\\' {
        return escape(cursor, start);
    }
    if class {
        if ch == '[' && cursor.peek() == Some(':') {
            return Err(ParseFailure::unsupported(
                start..cursor.offset,
                "regex POSIX class",
            ));
        }
        return Ok(match ch {
            ']' => Token::EndSquare,
            '-' => Token::Minus,
            '^' => Token::Negate,
            _ => Token::Char(ch),
        });
    }
    if count && ch.is_ascii_digit() {
        let mut value = ch as usize - '0' as usize;
        while cursor.peek().is_some_and(|c| c.is_ascii_digit()) {
            value = value
                .saturating_mul(10)
                .saturating_add(cursor.take()? as usize - '0' as usize);
            bound("regex repeat numeric bytes", cursor.offset - start, 7)?;
            bound("regex repeat metadata", value, 1_000_000)?;
        }
        return Ok(Token::Number(value));
    }
    match ch {
        '(' if cursor.peek() == Some('?') => {
            cursor.take()?;
            if cursor.peek() == Some(':') {
                cursor.take()?;
                return Ok(Token::Noncapture);
            }
            Err(ParseFailure::unsupported(
                start..cursor.offset,
                "regex assertion, named group, flag, atomic group or comment",
            ))
        }
        '(' => Ok(Token::Open),
        ')' => Ok(Token::Close),
        '[' => Ok(Token::Square),
        ']' => Ok(Token::EndSquare),
        '{' => Ok(Token::Brace),
        '}' => Ok(Token::EndBrace),
        ',' if count => Ok(Token::Comma),
        '|' => Ok(Token::Pipe),
        '*' => Ok(Token::Star),
        '+' => Ok(Token::Plus),
        '?' => Ok(Token::Question),
        '.' => Ok(Token::Dot),
        '^' | '$' => Err(ParseFailure::unsupported(
            start..cursor.offset,
            "regex anchor assertion",
        )),
        _ => Ok(Token::Char(ch)),
    }
}

fn escape<'s>(cursor: &mut Cursor<'s, '_>, start: usize) -> Result<Token<'s>, ParseFailure> {
    let ch = cursor
        .take()
        .map_err(|_| ParseFailure::syntax(start..cursor.offset, "dangling regex escape"))?;
    if ".*+?^$|()[]{}\\<>-/\"'".contains(ch) {
        return Ok(Token::Char(ch));
    }
    if "dDsSwW".contains(ch) {
        return Ok(Token::Escape(&cursor.input[start..cursor.offset]));
    }
    Err(ParseFailure::unsupported(
        start..cursor.offset,
        "regex backreference, assertion, Unicode/property or unsupported escape",
    ))
}

pub(in crate::plantuml) fn admit(
    input: (&[Spanned<'_>], usize),
    body: &BodySource<'_>,
    budget: &mut Budget,
    family: Family,
) -> Result<Storage, ParseFailure> {
    let (tokens, capacity) = input;
    let mut depth = 0usize;
    let mut maximum = 0usize;
    for (_, token, _) in tokens {
        budget.charge(1)?;
        match token {
            Token::Open | Token::Noncapture | Token::Square | Token::Brace => {
                depth += 1;
                maximum = maximum.max(depth);
            }
            Token::Close | Token::EndSquare | Token::EndBrace => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    // Complete viable-prefix closure over the actual tables proves these bounds;
    // all nonterminals are delimiter-balanced. See plantuml-parser-admission.md.
    let (live, terminals, reductions) = match family {
        Family::Ebnf => (5 * maximum + 9, 13usize, 4usize),
        Family::Regex => (4 * maximum + 6, 18, 6),
    };
    let stack = live.next_power_of_two().max(8);
    let diagnostic = (live + 1).next_power_of_two().max(8);
    let payload = match family {
        Family::Ebnf => std::mem::size_of::<Token<'_>>(),
        Family::Regex => std::mem::size_of::<Token<'_>>().max(std::mem::size_of::<(
            usize,
            crate::plantuml::regex::Upper,
            String,
        )>()),
    };
    let symbol = payload + std::mem::size_of::<usize>();
    let fixed = (stack + stack / 2) * (symbol + 2 * std::mem::size_of::<usize>() + 2)
        + (diagnostic + diagnostic / 2) * 3 * 2
        + terminals.next_power_of_two() * 3 * std::mem::size_of::<String>()
        + terminals * 32;
    let slots = 2 * (stack + stack / 2)
        + 3 * (diagnostic + diagnostic / 2)
        + 3 * terminals.next_power_of_two();
    bound(
        "railroad parser stack slots",
        slots,
        budget.limits.layout_work,
    )?;
    let bytes = capacity
        .saturating_mul(std::mem::size_of::<Spanned<'_>>())
        .saturating_add(body.adapters)
        .saturating_add(budget.limits.source_bytes.saturating_mul(8));
    bound(
        "railroad parser retained bytes",
        bytes,
        budget.limits.source_bytes.saturating_mul(256),
    )?;
    // Fixed stack scratch is separate from source-backed token/model arenas.
    bound(
        "railroad parser fixed bytes",
        fixed,
        budget.limits.layout_work.saturating_mul(symbol + 24),
    )?;
    budget.charge(
        tokens
            .len()
            .saturating_mul(reductions + 1)
            .saturating_add(reductions + slots + terminals * (live + reductions)),
    )?;
    Ok(Storage {
        bytes,
        limit: budget.limits.source_bytes.saturating_mul(256),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        RenderOptions,
        plantuml::{ebnf, preprocess, regex},
    };
    fn ebnf_model(source: &str) -> ebnf::EbnfDocument {
        let mut budget = Budget::new(&RenderOptions::default());
        let expanded =
            preprocess::prepare(&format!("@startebnf\n{source}\n@endebnf"), &mut budget).unwrap();
        let mut document = super::super::document::parse(&expanded, &mut budget).unwrap();
        super::super::preamble::parse(&mut document, &mut budget).unwrap();
        super::super::ebnf::parse(document.body, &mut budget).unwrap()
    }
    fn regex_model(source: &str) -> regex::RegexDocument {
        let mut budget = Budget::new(&RenderOptions::default());
        let expanded =
            preprocess::prepare(&format!("@startregex\n{source}\n@endregex"), &mut budget).unwrap();
        let mut document = super::super::document::parse(&expanded, &mut budget).unwrap();
        super::super::preamble::parse(&mut document, &mut budget).unwrap();
        super::super::regex::parse(document.body, &mut budget).unwrap()
    }
    #[test]
    fn should_preserve_ordered_rule_ids_quote_kinds_and_symbolic_references() {
        // given
        let source = "digit = \"digit\" | digit;\nnext = missing, 'a;b,c'; digit = ;";
        // when
        let model = ebnf_model(source);
        // then
        assert_eq!(
            model
                .rules
                .iter()
                .map(|r| r.name.as_str())
                .collect::<Vec<_>>(),
            ["digit", "next", "digit"]
        );
        assert!(matches!(&model.exprs[0].kind,ebnf::Kind::Terminal(s,'"') if s=="digit"));
        assert!(matches!(&model.exprs[1].kind,ebnf::Kind::Reference(s) if s=="digit"));
        assert!(matches!(
            &model.exprs[model.rules[2].expr].kind,
            ebnf::Kind::Empty
        ));
        assert!(
            model
                .exprs
                .iter()
                .any(|e| matches!(&e.kind,ebnf::Kind::Terminal(s,'\'') if s=="a;b,c"))
        );
        assert!(
            model
                .rules
                .windows(2)
                .all(|r| r[0].span.end <= r[1].span.start)
        );
    }
    #[test]
    fn should_keep_ebnf_sequence_precedence_and_independent_optional_repeat_nodes() {
        // given
        let source = "a = \"x\", b | [c], {d}, {e}-;";
        // when
        let model = ebnf_model(source);
        // then
        let ebnf::Kind::Choice(a, b) = model.exprs[model.rules[0].expr].kind else {
            panic!("choice root")
        };
        assert!(matches!(model.exprs[a].kind, ebnf::Kind::Sequence(..)));
        assert!(matches!(model.exprs[b].kind, ebnf::Kind::Sequence(..)));
        assert_eq!(
            model
                .exprs
                .iter()
                .filter(|e| matches!(e.kind, ebnf::Kind::Optional(_)))
                .count(),
            1
        );
        for min in [0, 1] {
            assert_eq!(
                model
                    .exprs
                    .iter()
                    .filter(|e| matches!(e.kind,ebnf::Kind::Repeat(_,n) if n==min))
                    .count(),
                1
            );
        }
    }
    #[test]
    fn should_parse_regex_precedence_and_count_metadata_without_expansion() {
        // given
        let sources = ["a|bc", "(a|b)c", "x{2,9}", "x{2,999999}"];
        // when
        let models = sources.map(regex_model);
        // then
        assert!(matches!(
            models[0].exprs[models[0].root].kind,
            regex::Kind::Choice(..)
        ));
        assert!(matches!(
            models[1].exprs[models[1].root].kind,
            regex::Kind::Sequence(..)
        ));
        assert_eq!(models[2].exprs.len(), 2);
        assert_eq!(models[3].exprs.len(), 2);
        assert!(matches!(
            models[3].exprs[models[3].root].kind,
            regex::Kind::Repeat {
                child: 0,
                min: 2,
                max: regex::Upper::Finite(999999),
                ..
            }
        ));
    }
    #[test]
    fn should_retain_negation_ranges_escaped_brackets_and_noncapture_kind() {
        // given
        let source = r"(?:[^a-c\]])";
        // when
        let model = regex_model(source);
        // then
        assert!(matches!(
            model.exprs[model.root].kind,
            regex::Kind::Group(_, "noncapture")
        ));
        let regex::Kind::Class {
            negated, members, ..
        } = &model.exprs[0].kind
        else {
            panic!("class")
        };
        assert!(*negated);
        assert!(matches!(
            members[..],
            [regex::Member::Range('a', 'c'), regex::Member::Scalar(']')]
        ));
    }

    #[test]
    fn should_admit_retained_vector_growth_before_allocation_and_keep_exact_payload_witnesses() {
        // given
        let mut storage = Storage { bytes: 0, limit: 1 };
        let mut values = Vec::new();
        let mut budget = Budget::new(&RenderOptions::default());
        // when
        let rejected = storage.push(&mut values, 0usize, &mut budget);
        // then
        assert!(rejected.is_err());
        assert_eq!(values.capacity(), 0);
        assert_eq!(std::mem::size_of::<Token<'_>>(), 24);
        assert_eq!(std::mem::size_of::<Spanned<'_>>(), 40);
        assert_eq!(std::mem::size_of::<(usize, regex::Upper, String)>(), 48);
    }
}
