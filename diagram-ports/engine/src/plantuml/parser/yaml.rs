//! Generated PlantUML-specific ordered YAML syntax.
use super::{
    BodySource, context,
    error::{ParseFailure, generated_failure},
    structured::{Arena, DepthToken, Syntax},
};
use crate::{
    RenderError,
    plantuml::{
        budget::{Budget, bound},
        data::Value,
        yaml::{FlowField, find_unquoted, remove_comment, unquote},
    },
};
use std::ops::Range;

#[derive(Clone, Debug)]
pub(in crate::plantuml) enum Token {
    Indent,
    Inline,
    Dedent,
    Dash,
    Colon,
    Newline,
    Scalar(String),
    FlowOpen,
    FlowClose,
    Comma,
    Field(String),
    Pipe,
    Literal(String),
}

impl DepthToken for Token {
    fn depth_delta(&self) -> isize {
        match self {
            Self::Indent | Self::Inline | Self::FlowOpen => 1,
            Self::Dedent | Self::FlowClose => -1,
            _ => 0,
        }
    }
}

type Spanned = (usize, Token, usize);
struct Physical {
    text: String,
    offsets: Vec<(usize, usize)>,
    base: usize,
    literal: bool,
}

impl Physical {
    fn indent(&self) -> usize {
        self.text.len() - self.text.trim_start_matches(' ').len()
    }
    fn content(&self) -> &str {
        remove_comment(self.text.trim())
    }
    fn span(&self, range: Range<usize>) -> Range<usize> {
        let locate = |position| {
            let index = self
                .offsets
                .partition_point(|&(normalized, _)| normalized <= position);
            let (normalized, raw) = index
                .checked_sub(1)
                .map_or((0, 0), |index| self.offsets[index]);
            self.base + raw + position - normalized
        };
        locate(range.start)..locate(range.end)
    }
    fn fragment_span(&self, text: &str) -> Range<usize> {
        // Fragments borrow this row; pointer offsets avoid rescanning long flow prefixes.
        let start = text.as_ptr() as usize - self.text.as_ptr() as usize;
        self.span(start..start + text.len())
    }
}

pub(in crate::plantuml) fn parse(
    body: BodySource<'_>,
    budget: &mut Budget,
) -> Result<Value, RenderError> {
    parse_body(&body, budget).map_err(|failure| failure.into_document(body.document, budget))
}

fn parse_body(body: &BodySource<'_>, budget: &mut Budget) -> Result<Value, ParseFailure> {
    let rows = preflight(body, budget)?;
    let tokens = lex((&rows, rows.capacity()), body.range.end, budget)?;
    drop(rows);
    let arena = Arena::admit((&tokens, tokens.capacity()), body, budget, Syntax::Yaml)?;
    let iterator = context::Checked::new(tokens, budget.control.clone());
    let mut builder = Builder {
        budget,
        arena,
        depth: 0,
    };
    super::generated_yaml::YamlParser::new()
        .parse(&mut builder, iterator)
        .map_err(|error| {
            generated_failure(
                error,
                body.range.end,
                "inconsistent YAML indentation or incomplete map, list or flow structure",
            )
        })
}

fn preflight(body: &BodySource<'_>, budget: &mut Budget) -> Result<Vec<Physical>, ParseFailure> {
    let mut rows = Vec::new();
    let mut literal_indent = usize::MAX;
    let mut base = body.range.start;
    for physical in body.document.text[body.range.clone()].split_inclusive('\n') {
        let raw = physical
            .strip_suffix('\n')
            .map_or(physical, |row| row.strip_suffix('\r').unwrap_or(row));
        let mut row = normalize(raw, base, budget)?;
        let content = row.content();
        let list = content == "-" || content.starts_with("- ");
        let rest = content.strip_prefix("- ").unwrap_or(content);
        let colon = find_unquoted(rest, ':');
        let opens_literal = !list && colon.is_some_and(|colon| rest[colon + 1..].trim() == "|");
        row.literal = literal_indent != usize::MAX
            && (content.is_empty() || (!list && colon.is_none()) || row.indent() > literal_indent);
        if row.literal {
            context::push(&mut rows, row, budget)?;
            base += physical.len();
            continue;
        }
        literal_indent = usize::MAX;
        reject_capabilities(&row, budget)?;
        if opens_literal {
            literal_indent = row.indent();
        }
        context::push(&mut rows, row, budget)?;
        base += physical.len();
    }
    Ok(rows)
}

fn normalize(raw: &str, base: usize, budget: &mut Budget) -> Result<Physical, ParseFailure> {
    budget
        .charge(raw.len() + 1)
        .map_err(ParseFailure::terminal)?;
    let mut offsets = Vec::new();
    let mut extra = 0;
    let mut charged = 0;
    for (offset, ch) in raw.char_indices() {
        if offset - charged >= 256 {
            budget
                .charge(offset - charged)
                .map_err(ParseFailure::terminal)?;
            charged = offset;
        }
        if ch == '\t' {
            extra += 3;
            context::push(&mut offsets, (offset + extra + 1, offset + 1), budget)?;
        }
    }
    budget
        .charge(raw.len() - charged)
        .map_err(ParseFailure::terminal)?;
    budget
        .text(raw.len().saturating_add(extra))
        .map_err(ParseFailure::terminal)?;
    Ok(Physical {
        text: raw.replace('\t', "    "),
        offsets,
        base,
        literal: false,
    })
}

fn reject_capabilities(row: &Physical, budget: &mut Budget) -> Result<(), ParseFailure> {
    let text = row.content();
    let entry = text.strip_prefix("- ").unwrap_or(text);
    if find_unquoted(entry, ':').is_some_and(|colon| entry[..colon].trim() == "<<") {
        return Err(ParseFailure::unsupported(
            row.fragment_span(entry),
            "YAML merge key",
        ));
    }
    let mut quote = '\0';
    let mut escaped = false;
    let mut boundary = true;
    for (offset, ch) in text.char_indices() {
        budget
            .charge(ch.len_utf8())
            .map_err(ParseFailure::terminal)?;
        if quote != '\0' {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == quote {
                quote = '\0';
            }
        } else if matches!(ch, '\'' | '"') {
            quote = ch;
            boundary = false;
        } else {
            if boundary && matches!(ch, '&' | '*' | '!') {
                return Err(ParseFailure::unsupported(
                    row.fragment_span(&text[offset..offset + ch.len_utf8()]),
                    "YAML anchor, alias, tag or directive",
                ));
            }
            if boundary && text[offset..].starts_with("<<:") {
                return Err(ParseFailure::unsupported(
                    row.fragment_span(&text[offset..offset + 3]),
                    "YAML merge key",
                ));
            }
            boundary = ch.is_whitespace() || matches!(ch, '[' | ',' | '{' | ':' | '-');
        }
    }
    Ok(())
}

struct Lexer<'b> {
    tokens: Vec<Spanned>,
    indents: Vec<usize>,
    budget: &'b mut Budget,
    retained: usize,
}

fn lex(
    input: (&[Physical], usize),
    eof: usize,
    budget: &mut Budget,
) -> Result<Vec<Spanned>, ParseFailure> {
    let (rows, capacity) = input;
    let mut retained = capacity
        .saturating_mul(std::mem::size_of::<Physical>())
        .saturating_add(budget.limits.source_bytes.saturating_mul(8));
    for row in rows {
        budget.charge(1).map_err(ParseFailure::terminal)?;
        retained = retained.saturating_add(row.text.capacity()).saturating_add(
            row.offsets
                .capacity()
                .saturating_mul(std::mem::size_of::<(usize, usize)>()),
        );
    }
    bound(
        "YAML lexical retained bytes",
        retained,
        budget.limits.source_bytes.saturating_mul(256),
    )
    .map_err(ParseFailure::terminal)?;
    let mut lexer = Lexer {
        tokens: Vec::new(),
        indents: Vec::new(),
        budget,
        retained,
    };
    for row in rows {
        lexer.row(row)?;
    }
    while lexer.indents.pop().is_some() {
        lexer.push(Token::Dedent, eof..eof)?;
    }
    Ok(lexer.tokens)
}

impl Lexer<'_> {
    fn push(&mut self, token: Token, span: Range<usize>) -> Result<(), ParseFailure> {
        let capacity = if self.tokens.len() == self.tokens.capacity() {
            self.tokens.capacity().saturating_mul(3).max(4)
        } else {
            self.tokens.capacity()
        };
        self.bound_tokens(capacity)?;
        context::push(&mut self.tokens, (span.start, token, span.end), self.budget)?;
        self.bound_tokens(self.tokens.capacity())
    }
    fn bound_tokens(&self, capacity: usize) -> Result<(), ParseFailure> {
        let bytes = self
            .retained
            .saturating_add(capacity.saturating_mul(std::mem::size_of::<Spanned>()))
            .saturating_add(
                self.indents
                    .capacity()
                    .saturating_mul(std::mem::size_of::<usize>()),
            );
        bound(
            "YAML lexical retained bytes",
            bytes,
            self.budget.limits.source_bytes.saturating_mul(256),
        )
        .map_err(ParseFailure::terminal)
    }
    fn copy(&mut self, raw: &str) -> Result<String, ParseFailure> {
        self.budget
            .text(raw.len())
            .map_err(ParseFailure::terminal)?;
        Ok(raw.to_owned())
    }

    fn indent(&mut self, indent: usize, span: Range<usize>) -> Result<(), ParseFailure> {
        while self.indents.last().is_some_and(|&parent| indent < parent) {
            self.indents.pop();
            self.push(Token::Dedent, span.clone())?;
        }
        if self.indents.last().is_none_or(|&parent| indent > parent) {
            self.budget
                .depth(self.indents.len() + 1)
                .map_err(ParseFailure::terminal)?;
            context::push(&mut self.indents, indent, self.budget)?;
            self.push(Token::Indent, span)?;
        }
        Ok(())
    }

    fn row(&mut self, row: &Physical) -> Result<(), ParseFailure> {
        if row.literal {
            let value = self.copy(row.text.trim())?;
            return self.push(Token::Literal(value), row.span(0..row.text.len()));
        }
        let content = row.content();
        if content.is_empty() {
            return Ok(());
        }
        self.indent(row.indent(), row.span(0..row.indent()))?;
        let list = content == "-" || content.starts_with("- ");
        let rest = if list {
            self.push(Token::Dash, row.fragment_span(&content[..1]))?;
            content.strip_prefix("- ").unwrap_or("").trim()
        } else {
            content
        };
        if list && rest.is_empty() {
            return self.push(Token::Newline, row.span(row.text.len()..row.text.len()));
        }
        if list {
            self.budget
                .depth(self.indents.len() + 1)
                .map_err(ParseFailure::terminal)?;
            context::push(&mut self.indents, row.indent() + 2, self.budget)?;
            self.push(Token::Inline, row.fragment_span(rest))?;
        }
        match find_unquoted(rest, ':') {
            Some(colon) => {
                let raw_key = rest[..colon].trim();
                let key = self.copy(unquote(raw_key))?;
                self.push(Token::Scalar(key), row.fragment_span(raw_key))?;
                self.push(Token::Colon, row.fragment_span(&rest[colon..colon + 1]))?;
                self.value(rest[colon + 1..].trim(), row)?;
            }
            None if list => {
                let value = self.copy(unquote(rest))?;
                self.push(Token::Scalar(value), row.fragment_span(rest))?;
            }
            _ => {
                return Err(ParseFailure::unsupported(
                    row.fragment_span(rest),
                    "YAML plain text/document marker outside keyed value",
                ));
            }
        }
        self.push(Token::Newline, row.span(row.text.len()..row.text.len()))
    }

    fn value(&mut self, raw: &str, row: &Physical) -> Result<(), ParseFailure> {
        if raw.is_empty() {
            return Ok(());
        }
        if raw == "|" {
            if row.content().starts_with("- ") {
                return Err(ParseFailure::unsupported(
                    row.fragment_span(raw),
                    "YAML list-item literal block",
                ));
            }
            return self.push(Token::Pipe, row.fragment_span(raw));
        }
        if raw == ">"
            || ["|-", "|+", ">-", ">+"]
                .iter()
                .any(|prefix| raw.starts_with(prefix))
        {
            return Err(ParseFailure::unsupported(
                row.fragment_span(raw),
                "YAML folded block/chomping/indentation indicator",
            ));
        }
        if raw.starts_with('[') && raw.ends_with(']') {
            self.push(Token::FlowOpen, row.fragment_span(&raw[..1]))?;
            let fields = &raw[1..raw.len() - 1];
            lex_flow(
                fields,
                self.indents.len() + 1,
                self.budget,
                |token, range, budget| {
                    let start = fields.as_ptr() as usize - row.text.as_ptr() as usize;
                    let span = row.span(start + range.start..start + range.end);
                    context::push(&mut self.tokens, (span.start, token, span.end), budget)
                },
            )
            .map_err(|mut failure| {
                if !matches!(failure.kind, super::error::FailureKind::Terminal(_)) {
                    let start = fields.as_ptr() as usize - row.text.as_ptr() as usize;
                    failure.span = row.span(start + failure.span.start..start + failure.span.end);
                }
                failure
            })?;
            return self.push(Token::FlowClose, row.fragment_span(&raw[raw.len() - 1..]));
        }
        let value = self.copy(unquote(raw))?;
        self.push(Token::Scalar(value), row.fragment_span(raw))
    }
}

fn lex_flow(
    text: &str,
    depth: usize,
    budget: &mut Budget,
    mut push: impl FnMut(Token, Range<usize>, &mut Budget) -> Result<(), ParseFailure>,
) -> Result<(), ParseFailure> {
    budget.depth(depth).map_err(ParseFailure::terminal)?;
    begin_flow_scalar(depth, budget)?;
    let mut field = FlowField::default();
    let mut quote = '\0';
    let mut chars = text.char_indices();
    let mut start = 0;
    while let Some((offset, ch)) = chars.next() {
        budget
            .charge(ch.len_utf8())
            .map_err(ParseFailure::terminal)?;
        if ch == '\\' {
            if let Some((_, next)) = chars.next() {
                budget
                    .charge(next.len_utf8())
                    .map_err(ParseFailure::terminal)?;
                field.push(next, budget).map_err(ParseFailure::terminal)?;
            }
            continue;
        }
        if quote != '\0' {
            if ch == quote {
                quote = '\0';
            } else {
                field.push(ch, budget).map_err(ParseFailure::terminal)?;
            }
            continue;
        }
        if field.starts_quote(ch) {
            quote = ch;
            field.open_quote();
            continue;
        }
        if ch == ',' {
            push(
                Token::Field(field.finish(budget).map_err(ParseFailure::terminal)?),
                start..offset,
                budget,
            )?;
            push(Token::Comma, offset..offset + 1, budget)?;
            begin_flow_scalar(depth, budget)?;
            start = offset + 1;
            continue;
        }
        if matches!(ch, '[' | ']') {
            return Err(ParseFailure::unsupported(
                offset..offset + 1,
                "YAML nested flow containers",
            ));
        }
        field.push(ch, budget).map_err(ParseFailure::terminal)?;
    }
    if quote != '\0' {
        return Err(ParseFailure::syntax(
            start..text.len(),
            "unterminated YAML flow quote",
        ));
    }
    push(
        Token::Field(field.finish(budget).map_err(ParseFailure::terminal)?),
        start..text.len(),
        budget,
    )
}

fn begin_flow_scalar(depth: usize, budget: &mut Budget) -> Result<(), ParseFailure> {
    budget.depth(depth + 1).map_err(ParseFailure::terminal)?;
    budget.item().map_err(ParseFailure::terminal)
}

pub(in crate::plantuml) struct Builder<'b> {
    budget: &'b mut Budget,
    arena: Arena,
    depth: usize,
}

impl Builder<'_> {
    pub fn open(&mut self) -> Result<(), ParseFailure> {
        self.budget
            .depth(self.depth + 1)
            .map_err(ParseFailure::terminal)?;
        self.budget.item().map_err(ParseFailure::terminal)?;
        self.depth += 1;
        Ok(())
    }
    pub fn inline(&mut self) -> Result<(), ParseFailure> {
        self.budget
            .depth(self.depth + 1)
            .map_err(ParseFailure::terminal)?;
        self.depth += 1;
        Ok(())
    }
    pub fn close(&mut self) -> Result<(), ParseFailure> {
        self.budget.charge(1).map_err(ParseFailure::terminal)?;
        self.depth = self.depth.saturating_sub(1);
        Ok(())
    }
    pub fn item(&mut self) -> Result<(), ParseFailure> {
        self.budget
            .depth(self.depth + 1)
            .map_err(ParseFailure::terminal)?;
        self.budget.item().map_err(ParseFailure::terminal)
    }
    pub fn key(&mut self, key: String) -> Result<String, ParseFailure> {
        self.item()?;
        Ok(key)
    }
    pub fn push<T>(&mut self, mut values: Vec<T>, value: T) -> Result<Vec<T>, ParseFailure> {
        self.arena.push(&mut values, value, self.budget)?;
        Ok(values)
    }
    pub fn replace(
        &mut self,
        mut entries: Vec<(String, Value)>,
        pair: (String, Value),
    ) -> Result<Vec<(String, Value)>, ParseFailure> {
        for (key, value) in &mut entries {
            self.budget
                .charge(key.len().min(pair.0.len()) + 1)
                .map_err(ParseFailure::terminal)?;
            if *key == pair.0 {
                *value = pair.1;
                return Ok(entries);
            }
        }
        self.arena.push(&mut entries, pair, self.budget)?;
        Ok(entries)
    }
    pub fn literal(&mut self, mut value: String, row: String) -> Result<String, ParseFailure> {
        self.budget
            .text(row.len() + 1)
            .map_err(ParseFailure::terminal)?;
        value.push_str(&row);
        value.push('\n');
        Ok(value)
    }
}

#[cfg(test)]
pub(in crate::plantuml) fn test_flow(
    text: &str,
    depth: usize,
    budget: &mut Budget,
) -> Result<Value, RenderError> {
    let mut tokens = vec![(0, Token::FlowOpen, 0)];
    lex_flow(text, depth, budget, |token, span, budget| {
        context::push(&mut tokens, (span.start, token, span.end), budget)
    })
    .map_err(|failure| failure.into_line(1))?;
    context::push(
        &mut tokens,
        (text.len(), Token::FlowClose, text.len()),
        budget,
    )
    .map_err(|failure| failure.into_line(1))?;
    let body = BodySource {
        document: &crate::plantuml::origin::EMPTY,
        range: 0..0,
        ownership: Default::default(),
        adapters: 0,
    };
    let arena = Arena::admit(
        (&tokens, tokens.capacity()),
        &body,
        budget,
        Syntax::YamlFlow,
    )
    .map_err(|failure| failure.into_line(1))?;
    let iterator = context::Checked::new(tokens, budget.control.clone());
    let mut builder = Builder {
        budget,
        arena,
        depth: 0,
    };
    super::generated_yaml::FlowParser::new()
        .parse(&mut builder, iterator)
        .map_err(|error| {
            generated_failure(error, text.len(), "invalid YAML flow sequence").into_line(1)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        RenderOptions,
        plantuml::{parser::outline::test_parse, source::Line},
    };

    fn document(source: &str, budget: &mut Budget) -> crate::plantuml::origin::Document {
        crate::plantuml::preprocess::prepare(source, budget).unwrap()
    }

    #[test]
    fn should_reject_capabilities_before_spending_any_model_item() {
        // given
        let source = "@startyaml\na: &anchor one\nb: *anchor\n@endyaml";
        let mut budget = Budget::new(&RenderOptions::default());
        budget.limits.model_items = 1;
        let document = document(source, &mut budget);
        let mut prepared = super::super::document::parse(&document, &mut budget).unwrap();
        super::super::preamble::parse(&mut prepared, &mut budget).unwrap();
        // when
        let error = preflight(&prepared.body, &mut budget).err().unwrap();
        // then
        assert!(matches!(
            error.kind,
            super::super::error::FailureKind::Unsupported(_)
        ));
        assert!(budget.item().is_ok());
    }

    #[test]
    fn should_stop_before_retaining_the_first_excessive_indent() {
        // given
        let mut budget = Budget::new(&RenderOptions::default());
        let root = normalize("a:", 0, &mut budget).unwrap();
        let child = normalize("  child: unseen", 3, &mut budget).unwrap();
        budget.limits.nesting_depth = 1;
        let mut lexer = Lexer {
            tokens: Vec::new(),
            indents: Vec::new(),
            budget: &mut budget,
            retained: 0,
        };
        lexer.row(&root).unwrap();
        let retained = lexer.tokens.len();
        // when
        let error = lexer.row(&child).unwrap_err();
        // then
        assert!(format!("{error:?}").contains("nesting depth"));
        assert_eq!(lexer.tokens.len(), retained);
        assert_eq!(lexer.indents, [0]);
    }

    #[test]
    fn should_reject_lexical_growth_before_allocating_a_token_buffer() {
        // given
        let mut budget = Budget::new(&RenderOptions::default());
        let retained = budget.limits.source_bytes * 256;
        let mut lexer = Lexer {
            tokens: Vec::new(),
            indents: Vec::new(),
            budget: &mut budget,
            retained,
        };
        // when
        let result = lexer.push(Token::Indent, 0..0);
        // then
        assert!(result.is_err());
        assert_eq!(lexer.tokens.capacity(), 0);
        assert_eq!(lexer.tokens.len(), 0);
    }

    #[test]
    fn should_admit_yaml_value_depth_64_and_reject_65_at_the_actual_source_limit() {
        for count in [63usize, 64] {
            // given
            let mut input = String::new();
            for level in 0..count {
                input.push_str(&" ".repeat(level));
                input.push_str(if level + 1 == count {
                    "a: scalar\n"
                } else {
                    "a:\n"
                });
            }
            let source = format!("@startyaml\n{input}@endyaml");
            let mut options = RenderOptions::default();
            options.limits.source_bytes = source.len();
            let mut budget = Budget::new(&options);
            let document = document(&source, &mut budget);
            let mut prepared = super::super::document::parse(&document, &mut budget).unwrap();
            super::super::preamble::parse(&mut prepared, &mut budget).unwrap();
            // when
            let result = parse(prepared.body, &mut budget);
            // then
            assert_eq!(result.is_ok(), count == 63, "{result:?}");
        }
    }

    #[test]
    fn should_reject_direct_generated_crossed_blocks_and_dangling_flow_fields() {
        for tokens in [
            vec![
                Token::Indent,
                Token::Scalar("key".into()),
                Token::Scalar("value".into()),
                Token::Newline,
                Token::Dedent,
            ],
            vec![
                Token::Indent,
                Token::Dash,
                Token::Inline,
                Token::Scalar("value".into()),
                Token::Newline,
            ],
            vec![
                Token::Indent,
                Token::Scalar("key".into()),
                Token::Colon,
                Token::Newline,
                Token::Indent,
                Token::Scalar("plain".into()),
                Token::Newline,
                Token::Dedent,
                Token::Dedent,
            ],
        ] {
            // given
            let mut budget = Budget::new(&RenderOptions::default());
            let body = BodySource {
                document: &crate::plantuml::origin::EMPTY,
                range: 0..0,
                ownership: Default::default(),
                adapters: 0,
            };
            let tokens = tokens
                .into_iter()
                .map(|token| (0, token, 0))
                .collect::<Vec<_>>();
            let arena = Arena::admit(
                (&tokens, tokens.capacity()),
                &body,
                &mut budget,
                Syntax::Yaml,
            )
            .unwrap();
            let iterator = context::Checked::new(tokens, budget.control.clone());
            let mut builder = Builder {
                budget: &mut budget,
                arena,
                depth: 0,
            };
            // when
            let result =
                super::super::generated_yaml::YamlParser::new().parse(&mut builder, iterator);
            // then
            assert!(result.is_err());
        }
        for tokens in [
            vec![Token::FlowOpen, Token::FlowClose],
            vec![
                Token::FlowOpen,
                Token::Field("a".into()),
                Token::Comma,
                Token::FlowClose,
            ],
            vec![Token::FlowOpen, Token::Field("a".into())],
        ] {
            // given
            let mut budget = Budget::new(&RenderOptions::default());
            let body = BodySource {
                document: &crate::plantuml::origin::EMPTY,
                range: 0..0,
                ownership: Default::default(),
                adapters: 0,
            };
            let tokens = tokens
                .into_iter()
                .map(|token| (0, token, 0))
                .collect::<Vec<_>>();
            let arena = Arena::admit(
                (&tokens, tokens.capacity()),
                &body,
                &mut budget,
                Syntax::YamlFlow,
            )
            .unwrap();
            let iterator = context::Checked::new(tokens, budget.control.clone());
            let mut builder = Builder {
                budget: &mut budget,
                arena,
                depth: 0,
            };
            // when
            let result =
                super::super::generated_yaml::FlowParser::new().parse(&mut builder, iterator);
            // then
            assert!(result.is_err());
        }
    }

    #[test]
    fn should_cancel_yaml_generated_delivery_and_retaining_actions() {
        for checkpoint in 1..=10 {
            // given
            let mut budget = Budget::new(&RenderOptions::default());
            let body = BodySource {
                document: &crate::plantuml::origin::EMPTY,
                range: 0..0,
                ownership: Default::default(),
                adapters: 0,
            };
            let tokens = vec![
                (0, Token::Indent, 0),
                (0, Token::Scalar("key".into()), 0),
                (0, Token::Colon, 0),
                (0, Token::Scalar("value".into()), 0),
                (0, Token::Newline, 0),
                (0, Token::Dedent, 0),
            ];
            let arena = Arena::admit(
                (&tokens, tokens.capacity()),
                &body,
                &mut budget,
                Syntax::Yaml,
            )
            .unwrap();
            let iterator = context::Checked::new(tokens, budget.control.clone());
            let mut builder = Builder {
                budget: &mut budget,
                arena,
                depth: 0,
            };
            builder.budget.control.cancel_after_checkpoints(checkpoint);
            // when
            let result =
                super::super::generated_yaml::YamlParser::new().parse(&mut builder, iterator);
            // then
            assert!(
                matches!(
                    result,
                    Err(lalrpop_util::ParseError::User {
                        error: ParseFailure {
                            kind: super::super::error::FailureKind::Terminal(
                                RenderError::Cancelled(_)
                            ),
                            ..
                        }
                    })
                ),
                "checkpoint {checkpoint}: {result:?}"
            );
        }
    }

    #[test]
    fn should_cancel_yaml_normalization_before_retaining_a_long_row() {
        // given
        let input = "x".repeat(4096);
        let mut budget = Budget::new(&RenderOptions::default());
        budget.control.cancel_after_checkpoints(3);
        // when
        let result = normalize(&input, 0, &mut budget);
        // then
        assert!(matches!(
            result,
            Err(ParseFailure {
                kind: super::super::error::FailureKind::Terminal(RenderError::Cancelled(_)),
                ..
            })
        ));
    }

    #[test]
    fn should_map_yaml_flow_failures_once_at_frozen_definition_uses() {
        // given
        let source = "@startyaml\n!define BAD [\"unclosed]\na: BAD\n@endyaml";
        let mut budget = Budget::new(&RenderOptions::default());
        let document = document(source, &mut budget);
        let mut prepared = super::super::document::parse(&document, &mut budget).unwrap();
        super::super::preamble::parse(&mut prepared, &mut budget).unwrap();
        // when
        let error = parse(prepared.body, &mut budget).unwrap_err().to_string();
        // then
        assert_eq!(error.matches("PlantUML at line 3,").count(), 1, "{error}");
        assert_eq!(error.matches("expansion \"BAD\"").count(), 1, "{error}");
        assert!(error.contains("defined at line 2"), "{error}");
    }

    #[test]
    fn should_locate_capability_tokens_after_expanded_tabs() {
        // given
        let source = "@startyaml\na:\n\tkey: *alias\n@endyaml";
        let mut budget = Budget::new(&RenderOptions::default());
        let document = document(source, &mut budget);
        let mut prepared = super::super::document::parse(&document, &mut budget).unwrap();
        super::super::preamble::parse(&mut prepared, &mut budget).unwrap();
        // when
        let error = parse(prepared.body, &mut budget).unwrap_err().to_string();
        // then
        assert!(error.contains("at line 3, column 7:"), "{error}");
    }

    #[test]
    fn should_generate_ordered_yaml_maps_lists_and_duplicate_replacement() {
        // given
        let lines = [Line {
            number: 2,
            text:
                "z: first\na: 2\nz: last\npeople:\n\t- name: one\n\t  age: true\n\t-\n\t  name: two"
                    .into(),
        }];
        let mut budget = Budget::new(&RenderOptions::default());
        // when
        let actual = test_parse(&lines, "yaml", &mut budget, parse).unwrap();
        // then
        assert_eq!(
            actual,
            Value::Object(vec![
                ("z".into(), Value::String("last".into())),
                ("a".into(), Value::String("2".into())),
                (
                    "people".into(),
                    Value::Array(vec![
                        Value::Object(vec![
                            ("name".into(), Value::String("one".into())),
                            ("age".into(), Value::String("true".into()))
                        ]),
                        Value::Object(vec![("name".into(), Value::String("two".into()))])
                    ])
                )
            ])
        );
    }

    #[test]
    fn should_generate_literal_chunks_and_narrow_escaped_flow_fields() {
        // given
        let lines = [Line { number: 2, text: "a: |\n  first  \nplain\n  second: value\n\nb: [one, \"two,three\", 'x\\ny',]\nempty: []".into() }];
        let mut budget = Budget::new(&RenderOptions::default());
        // when
        let actual = test_parse(&lines, "yaml", &mut budget, parse).unwrap();
        // then
        assert_eq!(
            actual,
            Value::Object(vec![
                (
                    "a".into(),
                    Value::String("first\nplain\nsecond: value\n\n".into())
                ),
                (
                    "b".into(),
                    Value::Array(vec![
                        Value::String("one".into()),
                        Value::String("two,three".into()),
                        Value::String("xny".into()),
                        Value::String("".into())
                    ])
                ),
                ("empty".into(), Value::Array(vec![Value::String("".into())]))
            ])
        );
    }

    #[test]
    fn should_reject_crossed_yaml_indentation_and_unsupported_capabilities() {
        for text in [
            "a:\n  b: one\n c: crossed",
            "a: [[nested]]",
            "a: [\"unclosed]",
            "a: &anchor one",
            "a: *alias",
            "a: !!str one",
            "a: >\n  folded",
            "- key: |\n  value",
            "-\n  - nested",
        ] {
            // given
            let lines = [Line {
                number: 2,
                text: text.into(),
            }];
            let mut budget = Budget::new(&RenderOptions::default());
            // when
            let result = test_parse(&lines, "yaml", &mut budget, parse);
            // then
            assert!(result.is_err(), "{text}");
        }
    }
}
