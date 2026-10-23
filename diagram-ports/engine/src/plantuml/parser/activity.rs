//! Generated structured activity syntax and bounded typed construction.
use super::{
    BodySource, context,
    error::{ParseFailure, generated_failure},
    ownership::Row,
};
use crate::{
    RenderError,
    plantuml::{
        activity::{Block, Branch, GroupKind, Instruction, Join},
        budget::{Budget, bound},
    },
};
use std::ops::Range;

#[derive(Clone, Debug)]
pub(in crate::plantuml) enum Token {
    Start,
    Stop,
    End,
    If,
    ElseIf,
    Else,
    EndIf,
    While,
    EndWhile,
    Repeat,
    RepeatWhile,
    Fork,
    ForkAgain,
    EndFork,
    EndMerge,
    Group(GroupKind),
    Then,
    Is,
    Not,
    Open,
    Close,
    BraceOpen,
    BraceClose,
    Colon,
    Semi,
    Text(String),
    Newline,
}
type Spanned = (usize, Token, usize);

pub(in crate::plantuml) fn parse(
    body: BodySource<'_>,
    budget: &mut Budget,
) -> Result<Block, RenderError> {
    parse_body(&body, budget).map_err(|error| error.into_document(body.document, budget))
}

fn parse_body(body: &BodySource<'_>, budget: &mut Budget) -> Result<Block, ParseFailure> {
    bound(
        "activity source bytes",
        body.range.len(),
        budget.limits.source_bytes,
    )
    .map_err(ParseFailure::terminal)?;
    budget
        .charge(std::mem::size_of::<super::ownership::Ownership>())
        .map_err(ParseFailure::terminal)?;
    let mut lexer = Lexer {
        tokens: Vec::new(),
        budget,
        depth: 0,
        maximum_depth: 0,
        literals: 0,
        action: String::new(),
        in_action: false,
        adapters: body.adapters,
    };
    let mut ownership = body.ownership;
    let mut base = body.range.start;
    for physical in body.document.text[body.range.clone()].split_inclusive('\n') {
        let raw = physical
            .strip_suffix('\n')
            .map_or(physical, |row| row.strip_suffix('\r').unwrap_or(row));
        let row = ownership.classify(raw, lexer.budget)?;
        ownership.advance(&row, &row.text, lexer.budget)?;
        lexer.row(&row, base)?;
        base += physical.len();
    }
    ownership.finish(base)?;
    let retained = admit(&mut lexer, body.adapters)?;
    let mut builder = Builder {
        budget: lexer.budget,
        retained,
    };
    let iterator = context::Checked::new(lexer.tokens, builder.budget.control.clone());
    super::generated_activity::ActivityParser::new()
        .parse(&mut builder, iterator)
        .map_err(|error| {
            generated_failure(
                error,
                body.range.end,
                "invalid or incomplete activity syntax",
            )
        })
}

struct Lexer<'b> {
    tokens: Vec<Spanned>,
    budget: &'b mut Budget,
    depth: usize,
    maximum_depth: usize,
    literals: usize,
    action: String,
    in_action: bool,
    adapters: usize,
}

impl Lexer<'_> {
    fn push(&mut self, token: Token, span: Range<usize>) -> Result<(), ParseFailure> {
        if self.tokens.len() == self.tokens.capacity() {
            self.check_storage(
                self.tokens
                    .capacity()
                    .saturating_mul(2)
                    .max(4)
                    .saturating_mul(std::mem::size_of::<Spanned>()),
            )?;
        }
        context::push(&mut self.tokens, (span.start, token, span.end), self.budget)
    }

    fn check_storage(&self, extra: usize) -> Result<(), ParseFailure> {
        let retained = self
            .adapters
            .saturating_add(
                self.tokens
                    .capacity()
                    .saturating_mul(std::mem::size_of::<Spanned>()),
            )
            .saturating_add(self.literals)
            .saturating_add(self.action.capacity())
            .saturating_add(extra);
        bound(
            "activity lexer retained bytes",
            retained,
            self.budget.limits.source_bytes.saturating_mul(256),
        )
        .map_err(ParseFailure::terminal)
    }

    fn copy(&mut self, value: &str) -> Result<String, ParseFailure> {
        bound("label bytes", value.len(), 16 * 1024).map_err(ParseFailure::terminal)?;
        self.check_storage(value.len())?;
        let literals = self.literals.saturating_add(value.len());
        bound(
            "activity syntax literal bytes",
            literals,
            self.budget.limits.source_bytes.saturating_mul(2),
        )
        .map_err(ParseFailure::terminal)?;
        self.budget
            .text(value.len())
            .map_err(ParseFailure::terminal)?;
        self.literals = literals;
        Ok(value.to_owned())
    }

    fn row(&mut self, row: &Row, base: usize) -> Result<(), ParseFailure> {
        if self.in_action {
            return self.action_row(row, base);
        }
        let start = row.text.len() - row.text.trim_start().len();
        let text = row.text.trim();
        if text.is_empty() {
            return Ok(());
        }
        if text.starts_with(':') {
            self.budget.item().map_err(ParseFailure::terminal)?;
            self.push(Token::Colon, row.span(base, start..start + 1))?;
            self.in_action = true;
            let end = if row.literal {
                row.text.len()
            } else {
                row.text.trim_end().len()
            };
            return self.action_part(&row.text[start + 1..end], row.span(base, start + 1..end));
        }
        let command = text.strip_suffix(';').unwrap_or(text).trim_end();
        let (token, bytes) = command_token(command).ok_or_else(|| {
            ParseFailure::unsupported(
                row.span(base, start..row.text.trim_end().len()),
                "activity command",
            )
        })?;
        match token {
            Token::If | Token::While | Token::Repeat | Token::Fork | Token::Group(_) => {
                self.budget
                    .depth(self.depth + 1)
                    .map_err(ParseFailure::terminal)?;
                self.depth += 1;
                self.maximum_depth = self.maximum_depth.max(self.depth);
                self.budget.item().map_err(ParseFailure::terminal)?;
            }
            Token::EndIf
            | Token::EndWhile
            | Token::RepeatWhile
            | Token::EndFork
            | Token::EndMerge
            | Token::BraceClose => self.depth = self.depth.saturating_sub(1),
            Token::Start | Token::Stop | Token::End | Token::ElseIf | Token::ForkAgain => {
                self.budget.item().map_err(ParseFailure::terminal)?
            }
            _ => {}
        }
        let group = matches!(token, Token::Group(_));
        let repeat = matches!(token, Token::Repeat);
        self.push(token, row.span(base, start..start + bytes))?;
        let tail = &command[bytes..];
        if group {
            self.group(
                &text[bytes..],
                row.span(base, start + bytes..start + text.len()),
            )?;
        } else if repeat {
            self.repeat_entry(
                &text[bytes..],
                row.span(base, start + bytes..start + text.len()),
            )?;
        } else if matches!(text, "else;") {
            return Err(ParseFailure::syntax(
                row.span(base, start..start + text.len()),
                "unexpected activity block ending",
            ));
        } else {
            self.suffix(tail, (row, base, start + bytes))?;
        }
        self.push(
            Token::Newline,
            row.span(base, row.text.len()..row.text.len()),
        )
    }

    fn repeat_entry(&mut self, tail: &str, span: Range<usize>) -> Result<(), ParseFailure> {
        if tail.trim().is_empty() {
            return Ok(());
        }
        let entry = tail
            .trim()
            .strip_prefix(':')
            .and_then(|part| part.strip_suffix(';'))
            .ok_or_else(|| ParseFailure::unsupported(span.clone(), "repeat entry action suffix"))?;
        let value = self.copy(entry)?;
        self.push(Token::Colon, span.clone())?;
        self.push(Token::Text(value), span.clone())?;
        self.push(Token::Semi, span)
    }

    fn action_row(&mut self, row: &Row, base: usize) -> Result<(), ParseFailure> {
        self.append_action("\n")?;
        self.action_part(&row.text, row.span(base, 0..row.text.len()))
    }

    fn action_part(&mut self, part: &str, span: Range<usize>) -> Result<(), ParseFailure> {
        let closing = part.trim_end().strip_suffix(';');
        self.append_action(closing.unwrap_or(part))?;
        if closing.is_some() {
            self.in_action = false;
            let value = std::mem::take(&mut self.action);
            self.literals = self.literals.saturating_add(value.capacity());
            self.push(Token::Text(value), span.clone())?;
            self.push(Token::Semi, span.clone())?;
            self.push(Token::Newline, span)?;
        }
        Ok(())
    }

    fn append_action(&mut self, value: &str) -> Result<(), ParseFailure> {
        let next = self.action.len().saturating_add(value.len());
        bound("label bytes", next, 16 * 1024).map_err(ParseFailure::terminal)?;
        self.budget
            .text(value.len())
            .map_err(ParseFailure::terminal)?;
        if next > self.action.capacity() {
            let capacity = next.max(self.action.capacity().saturating_mul(2)).max(8);
            self.check_storage(capacity)?;
            self.budget
                .charge(self.action.len())
                .map_err(ParseFailure::terminal)?;
            self.action
                .try_reserve_exact(capacity - self.action.len())
                .map_err(|error| {
                    ParseFailure::terminal(RenderError::Backend(format!(
                        "PlantUML activity action allocation for capacity {capacity} failed: {error}"
                    )))
                })?;
            self.check_storage(0)?;
        }
        self.action.push_str(value);
        Ok(())
    }

    fn group(&mut self, tail: &str, span: Range<usize>) -> Result<(), ParseFailure> {
        let name = tail
            .trim()
            .strip_suffix('{')
            .ok_or_else(|| {
                ParseFailure::unsupported(span.clone(), "activity group requires opening brace")
            })?
            .trim();
        if name.contains('#') || name.contains("<<") {
            return Err(ParseFailure::unsupported(span, "activity group decoration"));
        }
        let value = if name.starts_with('"') {
            self.quoted(name, span.clone())?
        } else {
            self.copy(name)?
        };
        self.push(Token::Text(value), span.clone())?;
        self.push(Token::BraceOpen, span)
    }

    fn quoted(&mut self, name: &str, span: Range<usize>) -> Result<String, ParseFailure> {
        bound("quoted activity bytes", name.len(), 16 * 1024 + 2)
            .map_err(ParseFailure::terminal)?;
        self.check_storage(name.len())?;
        self.budget
            .charge(name.len())
            .map_err(ParseFailure::terminal)?;
        let mut value = String::with_capacity(name.len());
        let mut escaped = false;
        for (index, ch) in name.char_indices().skip(1) {
            self.budget
                .charge(ch.len_utf8())
                .map_err(ParseFailure::terminal)?;
            if escaped {
                match ch {
                    'n' | '\\' | '~' => {
                        value.push('\\');
                        value.push(ch);
                    }
                    '"' => value.push(ch),
                    _ => return Err(ParseFailure::unsupported(span, "quoted activity escape")),
                }
                escaped = false;
            } else {
                match ch {
                    '\\' => escaped = true,
                    '"' if index + 1 == name.len() => {
                        self.budget
                            .text(value.len())
                            .map_err(ParseFailure::terminal)?;
                        self.literals = self.literals.saturating_add(value.capacity());
                        return Ok(value);
                    }
                    '"' => {
                        return Err(ParseFailure::unsupported(
                            span,
                            "activity group name suffix",
                        ));
                    }
                    _ => value.push(ch),
                }
            }
            bound("label bytes", value.len(), 16 * 1024).map_err(ParseFailure::terminal)?;
        }
        Err(ParseFailure::syntax(
            span,
            "unterminated quoted activity group",
        ))
    }

    fn suffix(&mut self, tail: &str, location: (&Row, usize, usize)) -> Result<(), ParseFailure> {
        let (row, base, start) = location;
        let mut cursor = 0;
        let mut depth = 0usize;
        while cursor < tail.len() {
            self.budget.charge(1).map_err(ParseFailure::terminal)?;
            let rest = &tail[cursor..];
            let ch = rest.chars().next().unwrap();
            let span = |bytes| row.span(base, start + cursor..start + cursor + bytes);
            match ch {
                '(' => {
                    depth += 1;
                    self.budget.depth(depth).map_err(ParseFailure::terminal)?;
                    self.maximum_depth = self.maximum_depth.max(self.depth + depth);
                    self.push(Token::Open, span(1))?;
                    cursor += 1;
                }
                ')' => {
                    depth = depth.saturating_sub(1);
                    self.push(Token::Close, span(1))?;
                    cursor += 1;
                }
                _ if depth > 0 => {
                    let bytes = self.scan(rest, |ch| matches!(ch, '(' | ')'))?;
                    let value = self.copy(&rest[..bytes])?;
                    self.push(Token::Text(value), span(bytes))?;
                    cursor += bytes;
                }
                _ if ch.is_whitespace() => cursor += ch.len_utf8(),
                '{' => {
                    self.push(Token::BraceOpen, span(1))?;
                    let content = rest.strip_suffix('}').unwrap_or(rest);
                    let bytes = self.scan(&content[1..], |_| false)?;
                    let value = self.copy(&rest[1..1 + bytes])?;
                    self.push(
                        Token::Text(value),
                        row.span(base, start + cursor + 1..start + cursor + 1 + bytes),
                    )?;
                    cursor += 1 + bytes;
                }
                '}' => {
                    self.push(Token::BraceClose, span(1))?;
                    cursor += 1;
                }
                _ => {
                    let (token, bytes) = [
                        ("then", Token::Then),
                        ("equals", Token::Is),
                        ("equal", Token::Is),
                        ("is", Token::Is),
                        ("not", Token::Not),
                    ]
                    .into_iter()
                    .find_map(|(word, token)| rest.starts_with(word).then_some((token, word.len())))
                    .ok_or_else(|| {
                        ParseFailure::unsupported(
                            span(rest.len()),
                            "activity condition or guard suffix",
                        )
                    })?;
                    self.push(token, span(bytes))?;
                    cursor += bytes;
                }
            }
        }
        Ok(())
    }

    fn scan(&mut self, value: &str, stop: impl Fn(char) -> bool) -> Result<usize, ParseFailure> {
        let mut pending = 0;
        for (index, ch) in value.char_indices() {
            if stop(ch) {
                self.budget
                    .charge(pending)
                    .map_err(ParseFailure::terminal)?;
                return Ok(index);
            }
            pending += ch.len_utf8();
            if pending >= 256 {
                self.budget
                    .charge(pending)
                    .map_err(ParseFailure::terminal)?;
                pending = 0;
            }
        }
        self.budget
            .charge(pending)
            .map_err(ParseFailure::terminal)?;
        Ok(value.len())
    }
}

fn command_token(value: &str) -> Option<(Token, usize)> {
    let token = match value {
        "start" => Token::Start,
        "stop" => Token::Stop,
        "end" => Token::End,
        "fork" => Token::Fork,
        "fork again" => Token::ForkAgain,
        "endif" | "end if" => Token::EndIf,
        "}" => Token::BraceClose,
        _ => {
            return [
                ("elseif", Token::ElseIf),
                ("else if", Token::ElseIf),
                ("else", Token::Else),
                ("endwhile", Token::EndWhile),
                ("end while", Token::EndWhile),
                ("while end", Token::EndWhile),
                ("repeat while", Token::RepeatWhile),
                ("repeatwhile", Token::RepeatWhile),
                ("end merge", Token::EndMerge),
                ("end fork", Token::EndFork),
                ("fork end", Token::EndFork),
                ("if", Token::If),
                ("while", Token::While),
                ("partition ", Token::Group(GroupKind::Partition)),
                ("group ", Token::Group(GroupKind::Group)),
            ]
            .into_iter()
            .find_map(|(word, token)| value.starts_with(word).then_some((token, word.len())))
            .or_else(|| {
                (value == "repeat" || value.starts_with("repeat :")).then_some((Token::Repeat, 6))
            });
        }
    };
    Some((token, value.len()))
}

pub(in crate::plantuml) struct Builder<'b> {
    budget: &'b mut Budget,
    retained: usize,
}
impl Builder<'_> {
    pub fn finish(&mut self, block: Block) -> Result<Block, ParseFailure> {
        self.budget.charge(1).map_err(ParseFailure::terminal)?;
        Ok(block)
    }

    pub fn append<T>(&mut self, mut values: Vec<T>, value: T) -> Result<Vec<T>, ParseFailure> {
        self.budget.charge(1).map_err(ParseFailure::terminal)?;
        if values.len() == values.capacity() {
            let capacity = values.capacity().saturating_mul(2).max(4);
            self.retain(capacity.saturating_mul(std::mem::size_of::<T>()))?;
            self.budget
                .charge(values.len())
                .map_err(ParseFailure::terminal)?;
            values
                .try_reserve_exact(capacity - values.len())
                .map_err(|error| {
                    ParseFailure::terminal(RenderError::Backend(format!(
                        "PlantUML activity model allocation for capacity {capacity} failed: {error}"
                    )))
                })?;
            self.retain(
                values
                    .capacity()
                    .saturating_sub(capacity)
                    .saturating_mul(std::mem::size_of::<T>()),
            )?;
        }
        values.push(value);
        Ok(values)
    }

    fn retain(&mut self, bytes: usize) -> Result<(), ParseFailure> {
        let next = self.retained.saturating_add(bytes);
        bound(
            "activity parser retained bytes",
            next,
            self.budget.limits.source_bytes.saturating_mul(256),
        )
        .map_err(ParseFailure::terminal)?;
        self.retained = next;
        Ok(())
    }

    pub fn conditional(
        &mut self,
        first: Branch,
        more: Vec<Branch>,
        otherwise: (String, Block),
    ) -> Result<Instruction, ParseFailure> {
        let mut branches = self.append(Vec::new(), first)?;
        for branch in more {
            branches = self.append(branches, branch)?;
        }
        Ok(Instruction::If {
            branches,
            guard: otherwise.0,
            otherwise: otherwise.1,
        })
    }

    pub fn fork(
        &mut self,
        first: Block,
        more: Vec<Block>,
        tail: (Join, String),
    ) -> Result<Instruction, ParseFailure> {
        let mut lanes = self.append(Vec::new(), first)?;
        for lane in more {
            lanes = self.append(lanes, lane)?;
        }
        Ok(Instruction::Fork {
            lanes,
            join: tail.0,
            label: tail.1,
        })
    }

    pub fn guard(&mut self, value: String, span: Range<usize>) -> Result<String, ParseFailure> {
        self.budget.charge(1).map_err(ParseFailure::terminal)?;
        if value.starts_with("->") || value.starts_with('#') {
            return Err(ParseFailure::unsupported(
                span,
                "activity guard style or suffix",
            ));
        }
        Ok(value)
    }

    pub fn concat(&mut self, mut value: String, part: &str) -> Result<String, ParseFailure> {
        bound(
            "label bytes",
            value.len().saturating_add(part.len()),
            16 * 1024,
        )
        .map_err(ParseFailure::terminal)?;
        self.budget
            .charge(part.len() + 1)
            .map_err(ParseFailure::terminal)?;
        if value.len() + part.len() > value.capacity() {
            let capacity = value
                .capacity()
                .saturating_mul(2)
                .max(value.len() + part.len())
                .max(8);
            self.retain(capacity)?;
            self.budget
                .charge(value.len())
                .map_err(ParseFailure::terminal)?;
            value
                .try_reserve_exact(capacity - value.len())
                .map_err(|error| {
                    ParseFailure::terminal(RenderError::Backend(format!(
                        "PlantUML activity label allocation for capacity {capacity} failed: {error}"
                    )))
                })?;
            self.retain(value.capacity().saturating_sub(capacity))?;
        }
        value.push_str(part);
        Ok(value)
    }

    pub fn nested(&mut self, value: String, nested: String) -> Result<String, ParseFailure> {
        let value = self.concat(value, "(")?;
        let value = self.concat(value, &nested)?;
        self.concat(value, ")")
    }

    pub fn join_label(&mut self, text: String, span: Range<usize>) -> Result<String, ParseFailure> {
        if text.is_empty() {
            return Err(ParseFailure::unsupported(span, "fork join suffix"));
        }
        self.nested_braces(text)
    }

    fn nested_braces(&mut self, text: String) -> Result<String, ParseFailure> {
        let value = self.concat(String::new(), "{")?;
        let value = self.concat(value, &text)?;
        self.concat(value, "}")
    }
}

fn admit(lexer: &mut Lexer<'_>, adapters: usize) -> Result<usize, ParseFailure> {
    // The 92-state/27-terminal tables have no zero-cost stack cycle. Their
    // viable-prefix graph adds at most nine states per live opener, plus six.
    // Eighteen extra slots cover a closing loop's still-live opener and EOF.
    let live = lexer.maximum_depth.saturating_mul(9).saturating_add(24);
    let capacity = live.next_power_of_two().max(8);
    let growing = capacity + capacity / 2;
    let symbol = std::mem::size_of::<Token>()
        + std::mem::size_of::<Instruction>()
        + std::mem::size_of::<Branch>()
        + 8 * std::mem::size_of::<usize>();
    let retained = lexer
        .tokens
        .capacity()
        .saturating_mul(std::mem::size_of::<Spanned>())
        .saturating_add(lexer.literals)
        .saturating_add(lexer.action.capacity())
        .saturating_add(adapters)
        .saturating_add(
            std::mem::size_of::<Builder<'_>>()
                + std::mem::size_of::<Lexer<'_>>()
                + std::mem::size_of::<super::ownership::Ownership>(),
        )
        .saturating_add(growing.saturating_mul(symbol + std::mem::size_of::<usize>()))
        .saturating_add(3 * (live + 1) + 32 * std::mem::size_of::<String>() + 27 * 32);
    bound(
        "activity parser retained bytes",
        retained,
        lexer.budget.limits.source_bytes.saturating_mul(256),
    )
    .map_err(ParseFailure::terminal)?;
    let slots = growing
        .saturating_mul(2)
        .saturating_add(3 * (live + 1) + 32);
    bound(
        "activity parser stack slots",
        slots,
        lexer.budget.limits.layout_work,
    )
    .map_err(ParseFailure::terminal)?;
    lexer
        .budget
        .charge(
            lexer
                .tokens
                .len()
                .saturating_mul(5)
                .saturating_add(4 + slots + 27 * (live + 4)),
        )
        .map_err(ParseFailure::terminal)?;
    Ok(retained)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RenderOptions, plantuml::activity::Instruction};

    fn model(body: &str, options: &RenderOptions) -> Result<Block, RenderError> {
        let source = format!("@startuml\n{body}\n@enduml");
        let mut budget = Budget::new(options);
        let document = crate::plantuml::preprocess::prepare(&source, &mut budget)?;
        let mut prepared = super::super::document::parse(&document, &mut budget)?;
        super::super::preamble::parse(&mut prepared, &mut budget)?;
        parse(prepared.body, &mut budget)
    }

    #[test]
    fn should_build_recursive_activity_models_at_generated_facade() {
        // given
        let body = "start\nif (a(b)) then (yes)\nwhile (ready) is (go)\n:act;\nendwhile (out)\nelseif (other) then (maybe)\nrepeat :entry;\n:again;\nrepeat while (retry) is (yes) not (done)\nelse (no)\nfork\ngroup \"G\" {\nstop\n}\nfork again\nend\nend merge {join}\nendif\nstop";
        // when
        let block = model(body, &RenderOptions::default()).unwrap();
        // then
        assert_eq!(block.len(), 3);
        let Instruction::If {
            branches,
            otherwise,
            guard,
        } = &block[1]
        else {
            panic!("{block:?}")
        };
        assert_eq!(branches.len(), 2);
        assert_eq!(
            (&*branches[0].condition.test, &*branches[0].condition.guard),
            ("a(b)", "yes")
        );
        assert_eq!(guard, "no");
        assert!(
            matches!(&branches[0].body[0], Instruction::While { body, out, .. } if body.len() == 1 && out == "out")
        );
        assert!(
            matches!(&branches[1].body[0], Instruction::Repeat { entry, out, .. } if entry == "entry" && out == "done")
        );
        assert!(
            matches!(&otherwise[0], Instruction::Fork { lanes, label, .. } if lanes.len() == 2 && label == "{join}")
        );
    }

    #[test]
    fn should_reject_crossed_blocks_and_illegal_activity_continuations() {
        // given
        for body in [
            "start\nif (x)\nendwhile",
            "start\nwhile (x)\nendif",
            "start\nelse\nstop",
            "fork\nstop",
            "start\nrepeat\nstop\nrepeat while (x) backward :no;",
            "start\nif (x) then (->style)\nendif",
            "start\nstop\nunknown",
        ] {
            // when
            let result = model(body, &RenderOptions::default());
            // then
            assert!(
                matches!(
                    result,
                    Err(RenderError::InvalidSource(_) | RenderError::UnsupportedFeature(_))
                ),
                "{body}: {result:?}"
            );
        }
    }

    #[test]
    fn should_admit_activity_depth_before_retaining_nested_models() {
        // given
        let mut options = RenderOptions::default();
        options.limits.nesting_depth = 64;
        for (depth, accepted) in [(64, true), (65, false)] {
            let body = format!(
                "{}:leaf;\n{}",
                "if (x)\n".repeat(depth),
                "endif\n".repeat(depth)
            );
            // when
            let result = model(&body, &options);
            // then
            assert_eq!(result.is_ok(), accepted, "{result:?}");
            if let Err(error) = result {
                assert!(error.to_string().contains("nesting depth"), "{error}");
            }
        }
    }

    #[test]
    fn should_enforce_activity_work_model_and_cancellation_limits() {
        // given
        for resource in ["work", "model", "cancel"] {
            let mut options = RenderOptions::default();
            match resource {
                "work" => options.limits.layout_work = 32,
                "model" => options.limits.model_items = 1,
                _ => options.control.cancel(),
            }
            // when
            let result = model("start\n:action;\nstop", &options);
            // then
            let error = result.unwrap_err();
            if resource == "cancel" {
                assert!(matches!(error, RenderError::Cancelled(_)), "{error}");
            } else {
                assert!(error.to_string().contains("exceeds limit"), "{error}");
            }
        }
    }

    #[test]
    fn should_locate_activity_guards_in_the_authored_document() {
        // given
        let body = "start\nif (x) then /' gap '/ (#red)\nendif";
        // when
        let error = model(body, &RenderOptions::default())
            .unwrap_err()
            .to_string();
        // then
        assert!(error.contains("line 3, column 23"), "{error}");
    }

    #[test]
    fn should_reject_the_next_activity_opener_before_syntax_retention() {
        // given
        let options = RenderOptions::default();
        let mut budget = Budget::new(&options);
        let mut lexer = Lexer {
            tokens: Vec::new(),
            budget: &mut budget,
            depth: 64,
            maximum_depth: 64,
            literals: 0,
            action: String::new(),
            in_action: false,
            adapters: 0,
        };
        let row = super::super::ownership::Ownership::default()
            .classify("if (large payload)", lexer.budget)
            .unwrap();
        // when
        let error = lexer.row(&row, 0).unwrap_err();
        // then
        assert!(
            matches!(error.kind, super::super::error::FailureKind::Terminal(RenderError::Backend(ref message)) if message.contains("nesting depth value 65 exceeds limit 64"))
        );
        assert_eq!(
            (lexer.tokens.len(), lexer.tokens.capacity(), lexer.literals),
            (0, 0, 0)
        );
    }

    #[test]
    fn should_own_structure_and_eof_in_the_generated_activity_parser() {
        // given
        for stream in [
            vec![(0, Token::Else, 4), (4, Token::Newline, 5)],
            vec![(0, Token::Fork, 4), (4, Token::Newline, 5)],
            vec![
                (0, Token::Start, 5),
                (5, Token::Newline, 6),
                (6, Token::EndIf, 11),
            ],
        ] {
            let mut budget = Budget::new(&RenderOptions::default());
            let iterator = context::Checked::new(stream, budget.control.clone());
            let mut builder = Builder {
                budget: &mut budget,
                retained: 0,
            };
            // when
            let result = super::super::generated_activity::ActivityParser::new()
                .parse(&mut builder, iterator);
            // then
            assert!(result.is_err(), "{result:?}");
        }
    }

    #[test]
    fn should_cancel_during_generated_activity_delivery_and_construction() {
        // given
        for checkpoint in 1..=7 {
            let stream = vec![
                (0, Token::Colon, 1),
                (1, Token::Text("action".into()), 7),
                (7, Token::Semi, 8),
                (8, Token::Newline, 9),
            ];
            let mut budget = Budget::new(&RenderOptions::default());
            budget.control.cancel_after_checkpoints(checkpoint);
            let iterator = context::Checked::new(stream, budget.control.clone());
            let mut builder = Builder {
                budget: &mut budget,
                retained: 0,
            };
            // when
            let result = super::super::generated_activity::ActivityParser::new()
                .parse(&mut builder, iterator);
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
    fn should_bound_activity_retention_before_token_or_model_growth() {
        // given
        let mut options = RenderOptions::default();
        options.limits.source_bytes = 4;
        let mut budget = Budget::new(&options);
        let mut lexer = Lexer {
            tokens: Vec::new(),
            budget: &mut budget,
            depth: 0,
            maximum_depth: 0,
            literals: 0,
            action: String::new(),
            in_action: false,
            adapters: 0,
        };
        for _ in 0..4 {
            lexer.push(Token::Start, 0..1).unwrap();
        }
        // when
        let error = lexer.push(Token::Start, 0..1).unwrap_err();
        // then
        assert!(matches!(
            error.kind,
            super::super::error::FailureKind::Terminal(RenderError::Backend(_))
        ));
        assert_eq!((lexer.tokens.len(), lexer.tokens.capacity()), (4, 4));

        // given
        options.limits.source_bytes = 1;
        let mut budget = Budget::new(&options);
        let mut builder = Builder {
            budget: &mut budget,
            retained: 0,
        };
        // when
        let error = builder.append(Vec::new(), Instruction::Start).unwrap_err();
        // then
        assert!(matches!(
            error.kind,
            super::super::error::FailureKind::Terminal(RenderError::Backend(_))
        ));
        assert_eq!(builder.retained, 0);
    }

    #[test]
    fn should_preserve_activity_entry_and_join_literal_spelling() {
        // given
        let body =
            "repeat :entry  ;\nstop\nrepeatwhile\nfork\nend\nfork again;\nstop\nfork end {a}b};";
        // when
        let block = model(body, &RenderOptions::default()).unwrap();
        // then
        assert!(matches!(&block[0], Instruction::Repeat { entry, .. } if entry == "entry  "));
        assert!(matches!(&block[1], Instruction::Fork { label, .. } if label == "{a}b}"));
        for body in [
            "repeat;\nstop\nrepeatwhile",
            "group G {;\nstop\n}",
            "if (x)\nelse;\nendif",
        ] {
            assert!(
                model(body, &RenderOptions::default()).is_err(),
                "accepted {body}"
            );
        }
    }

    #[test]
    fn should_continue_activity_ownership_and_map_frozen_guard_uses() {
        // given
        let body = "title P /' open\n'/ start\n:one;\nstop";
        // when
        let block = model(body, &RenderOptions::default()).unwrap();
        // then
        assert!(
            matches!(block.as_slice(), [Instruction::Start, Instruction::Action(text), Instruction::Stop] if text == "one")
        );

        // given
        let body = "!define GUARD #red\nstart\nif (x) then (GUARD)\nendif";
        // when
        let error = model(body, &RenderOptions::default())
            .unwrap_err()
            .to_string();
        // then
        assert!(error.contains("used at line 4, column 14"), "{error}");
        assert!(error.contains("defined at line 2, column 1"), "{error}");
    }

    #[test]
    fn should_trim_comment_normalized_action_openings_before_raw_continuations() {
        // given
        let body = "start\n/' comment '/ :opening   \n  raw  \nclosing;\nstop";
        // when
        let block = model(body, &RenderOptions::default()).unwrap();
        // then
        assert!(
            matches!(&block[1], Instruction::Action(text) if text == "opening\n  raw  \nclosing"),
            "{block:?}"
        );
    }

    #[test]
    fn should_include_unfinished_action_capacity_in_generated_admission() {
        // given
        let mut options = RenderOptions::default();
        options.limits.source_bytes = 100;
        let mut budget = Budget::new(&options);
        let mut lexer = Lexer {
            tokens: Vec::new(),
            budget: &mut budget,
            depth: 0,
            maximum_depth: 0,
            literals: 0,
            action: String::with_capacity(16 * 1024),
            in_action: true,
            adapters: 0,
        };
        // when
        let result = admit(&mut lexer, 0);
        // then
        assert!(
            matches!(result, Err(ParseFailure { kind: super::super::error::FailureKind::Terminal(RenderError::Backend(ref message)), .. }) if message.contains("activity parser retained bytes")),
            "{result:?}"
        );
    }
}
