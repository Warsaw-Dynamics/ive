//! MIT PlantUML modern activity grammar and structured instruction semantics.
//! Source/notice mappings are recorded in the provenance manifest.
use super::{
    budget::{Budget, invalid, unsupported},
    lexer,
    source::Line,
    text,
};
use crate::RenderError;

pub(super) type Block = Vec<Instruction>;
#[derive(Debug)]
pub(super) enum Instruction {
    Start,
    Stop,
    End,
    Action(String),
    If {
        branches: Vec<Branch>,
        otherwise: Block,
        guard: String,
    },
    While {
        condition: Condition,
        body: Block,
        out: String,
    },
    Repeat {
        entry: String,
        body: Block,
        condition: Condition,
        out: String,
    },
    Fork {
        lanes: Vec<Block>,
        join: Join,
        label: String,
    },
    Group {
        name: String,
        kind: GroupKind,
        body: Block,
    },
}
#[derive(Debug)]
pub(super) struct Condition {
    pub test: String,
    pub guard: String,
}
#[derive(Debug)]
pub(super) struct Branch {
    pub condition: Condition,
    pub body: Block,
}
#[derive(Clone, Copy, Debug)]
pub(super) enum Join {
    Synchronize,
    Merge,
}
#[derive(Clone, Copy, Debug)]
pub(super) enum GroupKind {
    Partition,
    Group,
}

pub(super) fn parse(lines: &[Line], budget: &mut Budget) -> Result<Block, RenderError> {
    let mut parser = Parser {
        lines,
        index: 0,
        budget,
    };
    let block = parser.block(0)?;
    if parser.index != lines.len() {
        return Err(invalid(
            lines[parser.index].number,
            "unexpected activity block ending",
        ));
    }
    Ok(block)
}

struct Parser<'a, 'b> {
    lines: &'a [Line],
    index: usize,
    budget: &'b mut Budget,
}
impl<'a> Parser<'a, '_> {
    fn block(&mut self, depth: usize) -> Result<Block, RenderError> {
        self.budget.depth(depth)?;
        let mut instructions = Vec::new();
        while let Some(line) = self.lines.get(self.index) {
            if is_closer(&line.text) {
                break;
            }
            self.budget.item()?;
            self.budget.charge(line.text.len())?;
            instructions.push(self.instruction(depth)?);
        }
        Ok(instructions)
    }

    fn instruction(&mut self, depth: usize) -> Result<Instruction, RenderError> {
        let line = &self.lines[self.index];
        let command = line.text.strip_suffix(';').unwrap_or(&line.text).trim_end();
        self.index += 1;
        match command {
            "start" => Ok(Instruction::Start),
            "stop" => Ok(Instruction::Stop),
            "end" => Ok(Instruction::End),
            _ if line.text.starts_with(':') => self.action(line),
            _ if command.starts_with("if") => self.conditional(line, depth),
            _ if command.starts_with("while") => self.while_loop(line, depth),
            _ if command == "repeat" || command.starts_with("repeat :") => {
                self.repeat_loop(line, depth)
            }
            "fork" => self.fork(depth),
            _ if command.starts_with("partition ") || command.starts_with("group ") => {
                self.group(line, depth)
            }
            _ => Err(unsupported(line.number, &line.text)),
        }
    }

    fn action(&mut self, first: &Line) -> Result<Instruction, RenderError> {
        let mut value = String::new();
        let mut part = &first.text[1..];
        loop {
            let closing = part.trim_end().strip_suffix(';');
            let text = closing.unwrap_or(part);
            super::text::append_label(&mut value, text, self.budget)?;
            if closing.is_some() {
                return Ok(Instruction::Action(keep(
                    &value,
                    first.number,
                    self.budget,
                )?));
            }
            let next = self
                .lines
                .get(self.index)
                .ok_or_else(|| invalid(first.number, "unterminated activity action"))?;
            super::text::append_label(&mut value, "\n", self.budget)?;
            part = &next.text;
            self.index += 1;
        }
    }

    fn conditional(&mut self, first: &Line, depth: usize) -> Result<Instruction, RenderError> {
        let first_condition = condition(&first.text[2..], first.number, "then")?;
        let mut branches = vec![Branch {
            condition: self.keep_condition(first_condition, first.number)?,
            body: self.block(depth + 1)?,
        }];
        while let Some(line) = self.lines.get(self.index) {
            let tail = line
                .text
                .strip_prefix("elseif")
                .or_else(|| line.text.strip_prefix("else if"));
            let Some(tail) = tail else {
                break;
            };
            self.budget.item()?;
            let condition =
                self.keep_condition(condition(tail, line.number, "then")?, line.number)?;
            self.index += 1;
            branches.push(Branch {
                condition,
                body: self.block(depth + 1)?,
            });
        }
        let (guard, otherwise) = if let Some(line) = self.lines.get(self.index).filter(|l| {
            l.text == "else" || l.text.starts_with("else ") || l.text.starts_with("else(")
        }) {
            let guard = guard_only(&line.text[4..], line.number)?;
            let guard = keep(&guard, line.number, self.budget)?;
            self.index += 1;
            (guard, self.block(depth + 1)?)
        } else {
            (String::new(), vec![])
        };
        self.close(&["endif", "end if"], first.number)?;
        Ok(Instruction::If {
            branches,
            otherwise,
            guard,
        })
    }

    fn while_loop(&mut self, first: &Line, depth: usize) -> Result<Instruction, RenderError> {
        let condition = self.keep_condition(
            condition(&first.text[5..], first.number, "is")?,
            first.number,
        )?;
        let body = self.block(depth + 1)?;
        let line = self.current(first.number)?;
        let tail = ["endwhile", "end while", "while end"]
            .iter()
            .find_map(|prefix| line.text.strip_prefix(prefix))
            .ok_or_else(|| invalid(line.number, "expected endwhile"))?;
        let out = guard_only(tail, line.number)?;
        let out = keep(&out, line.number, self.budget)?;
        self.index += 1;
        Ok(Instruction::While {
            condition,
            body,
            out,
        })
    }

    fn repeat_loop(&mut self, first: &Line, depth: usize) -> Result<Instruction, RenderError> {
        let tail = first.text[6..].trim();
        let entry = if tail.is_empty() {
            String::new()
        } else {
            let entry = tail
                .strip_prefix(':')
                .and_then(|s| s.strip_suffix(';'))
                .ok_or_else(|| unsupported(first.number, "repeat entry action suffix"))?;
            keep(entry, first.number, self.budget)?
        };
        let body = self.block(depth + 1)?;
        let line = self.current(first.number)?;
        let tail = line
            .text
            .strip_prefix("repeat while")
            .or_else(|| line.text.strip_prefix("repeatwhile"))
            .ok_or_else(|| invalid(line.number, "expected repeat while"))?;
        let (condition, out) = repeat_condition(tail, line.number)?;
        let condition = self.keep_condition(condition, line.number)?;
        let out = keep(&out, line.number, self.budget)?;
        self.index += 1;
        Ok(Instruction::Repeat {
            entry,
            body,
            condition,
            out,
        })
    }

    fn fork(&mut self, depth: usize) -> Result<Instruction, RenderError> {
        let start = self.lines[self.index - 1].number;
        let mut lanes = vec![self.block(depth + 1)?];
        while self
            .lines
            .get(self.index)
            .is_some_and(|l| trim_command(&l.text) == "fork again")
        {
            self.budget.item()?;
            self.index += 1;
            lanes.push(self.block(depth + 1)?);
        }
        let line = self.current(start)?;
        let (join, tail) = if let Some(tail) = line.text.strip_prefix("end merge") {
            (Join::Merge, tail)
        } else {
            let tail = line
                .text
                .strip_prefix("end fork")
                .or_else(|| line.text.strip_prefix("fork end"))
                .ok_or_else(|| invalid(line.number, "expected end fork or end merge"))?;
            (Join::Synchronize, tail)
        };
        let tail = trim_command(tail);
        let label = if tail.is_empty() {
            String::new()
        } else {
            tail.strip_prefix('{')
                .and_then(|s| s.strip_suffix('}'))
                .filter(|s| !s.is_empty())
                .ok_or_else(|| unsupported(line.number, "fork join suffix"))?;
            keep(tail, line.number, self.budget)?
        };
        self.index += 1;
        Ok(Instruction::Fork { lanes, join, label })
    }

    fn group(&mut self, first: &Line, depth: usize) -> Result<Instruction, RenderError> {
        let (kind, name) = if let Some(name) = first.text.strip_prefix("partition ") {
            (GroupKind::Partition, name)
        } else {
            (GroupKind::Group, &first.text[6..])
        };
        let name = name
            .strip_suffix('{')
            .ok_or_else(|| unsupported(first.number, "activity group requires opening brace"))?
            .trim();
        if name.contains('#') || name.contains("<<") {
            return Err(unsupported(first.number, "activity group decoration"));
        }
        let name = if name.starts_with('"') {
            let tokens = lexer::tokens(
                &Line {
                    number: first.number,
                    text: name.into(),
                },
                self.budget,
            )?;
            match tokens.as_slice() {
                [token] if token.quoted => token.text.clone(),
                _ => return Err(unsupported(first.number, "activity group name suffix")),
            }
        } else {
            name.to_owned()
        };
        let name = keep(&name, first.number, self.budget)?;
        let body = self.block(depth + 1)?;
        self.close(&["}"], first.number)?;
        Ok(Instruction::Group { name, kind, body })
    }

    fn keep_condition(
        &mut self,
        condition: Condition,
        line: usize,
    ) -> Result<Condition, RenderError> {
        Ok(Condition {
            test: keep(&condition.test, line, self.budget)?,
            guard: keep(&condition.guard, line, self.budget)?,
        })
    }
    fn current(&self, start: usize) -> Result<&'a Line, RenderError> {
        self.lines
            .get(self.index)
            .ok_or_else(|| invalid(start, "unclosed activity block"))
    }
    fn close(&mut self, expected: &[&str], start: usize) -> Result<(), RenderError> {
        let line = self.current(start)?;
        if !expected.contains(&trim_command(&line.text)) {
            return Err(invalid(
                line.number,
                format!("expected {expected:?}, found {:?}", line.text),
            ));
        }
        self.index += 1;
        Ok(())
    }
}

fn keep(value: &str, line: usize, budget: &mut Budget) -> Result<String, RenderError> {
    budget.text(value.len())?;
    text::validate_label(value, line)?;
    Ok(value.to_owned())
}
fn trim_command(value: &str) -> &str {
    value
        .trim()
        .strip_suffix(';')
        .unwrap_or(value.trim())
        .trim_end()
}
fn is_closer(value: &str) -> bool {
    [
        "else",
        "endif",
        "end if",
        "endwhile",
        "end while",
        "while end",
        "repeat while",
        "repeatwhile",
        "fork again",
        "end fork",
        "fork end",
        "end merge",
        "}",
    ]
    .iter()
    .any(|prefix| value.starts_with(prefix))
}

fn parenthesized(value: &str, line: usize) -> Result<(&str, &str), RenderError> {
    let value = value.trim_start();
    let rest = value
        .strip_prefix('(')
        .ok_or_else(|| invalid(line, "expected parenthesized activity condition or guard"))?;
    let mut depth = 1;
    for (offset, ch) in rest.char_indices() {
        if ch == '(' {
            depth += 1;
        }
        if ch == ')' {
            depth -= 1;
        }
        if depth == 0 {
            return Ok((&rest[..offset], rest[offset + 1..].trim_start()));
        }
    }
    Err(invalid(line, "unclosed activity condition or guard"))
}

fn condition(value: &str, line: usize, keyword: &str) -> Result<Condition, RenderError> {
    let (test, tail) = parenthesized(trim_command(value), line)?;
    let guard = if tail.is_empty() {
        String::new()
    } else {
        let tail = tail
            .strip_prefix(keyword)
            .or_else(|| {
                if keyword == "is" {
                    tail.strip_prefix("equals")
                        .or_else(|| tail.strip_prefix("equal"))
                } else {
                    None
                }
            })
            .ok_or_else(|| unsupported(line, "activity condition suffix"))?;
        guard_only(tail, line)?
    };
    Ok(Condition {
        test: test.into(),
        guard,
    })
}

fn guard_only(value: &str, line: usize) -> Result<String, RenderError> {
    let value = trim_command(value);
    if value.is_empty() {
        return Ok(String::new());
    }
    let (guard, tail) = parenthesized(value, line)?;
    if !tail.is_empty() || guard.starts_with("->") || guard.starts_with('#') {
        return Err(unsupported(line, "activity guard style or suffix"));
    }
    Ok(guard.into())
}

fn repeat_condition(value: &str, line: usize) -> Result<(Condition, String), RenderError> {
    let value = trim_command(value);
    if value.is_empty() {
        return Ok((
            Condition {
                test: String::new(),
                guard: String::new(),
            },
            String::new(),
        ));
    }
    let (test, tail) = parenthesized(value, line)?;
    let (guard, tail) = if let Some(tail) = tail
        .strip_prefix("is")
        .or_else(|| tail.strip_prefix("equals"))
        .or_else(|| tail.strip_prefix("equal"))
    {
        parenthesized(tail, line)?
    } else {
        ("", tail)
    };
    let out = if let Some(tail) = tail.strip_prefix("not") {
        guard_only(tail, line)?
    } else if tail.is_empty() {
        String::new()
    } else {
        return Err(unsupported(line, "repeat while suffix/backward route"));
    };
    Ok((
        Condition {
            test: test.into(),
            guard: guard.into(),
        },
        out,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn should_scan_whitespace_only_multiline_actions_once() {
        // given
        let source = format!("@startuml\n:\n{};\nstop\n@enduml", " \n".repeat(60_000));
        let mut options = crate::RenderOptions::default();
        options.limits.timeout = std::time::Duration::from_millis(500);
        let mut budget = Budget::new(&options);
        let lines = super::super::source::prepare(&source, &mut budget).unwrap();
        // when
        let result = parse(&lines, &mut budget);
        // then
        assert!(
            matches!(result, Err(RenderError::Backend(ref message)) if message == "PlantUML label bytes value 16385 exceeds limit 16384"),
            "{result:?}"
        );
        assert!(budget.work < source.len() * 4);
    }

    #[test]
    fn should_parse_below_cap_multiline_actions_and_check_cancellation() {
        // given
        let source = format!("@startuml\n:\n{};\nstop\n@enduml", " \n".repeat(4_000));
        let options = crate::RenderOptions::default();
        let mut budget = Budget::new(&options);
        let lines = super::super::source::prepare(&source, &mut budget).unwrap();
        // when
        let result = parse(&lines, &mut budget);
        options.control.cancel();
        let cancelled = parse(&lines, &mut budget);
        // then
        assert!(result.is_ok(), "{result:?}");
        assert!(budget.work < source.len() * 8);
        assert!(matches!(cancelled, Err(RenderError::Cancelled(_))));
    }
}
