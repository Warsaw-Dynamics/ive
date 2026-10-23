//! MIT PlantUML yaml/parser: narrow string scalars, indentation, literal blocks and ordered replacement.
use super::{
    budget::{Budget, invalid, unsupported},
    data::Value,
    source::Line,
};
use crate::RenderError;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Empty,
    Plain,
    Dash,
    Scalar,
    Entry,
}
struct YamlLine {
    number: usize,
    indent: usize,
    kind: Kind,
    list: bool,
    key: String,
    value: String,
    raw_value: String,
    raw: String,
}

pub(super) fn parse(lines: &[Line], budget: &mut Budget) -> Result<Value, RenderError> {
    let mut tokens = Vec::new();
    let mut block = Block::None;
    // This entire capability pass finishes before constructing ANY value model.
    for line in lines {
        budget.charge(line.text.len() + 1)?;
        let token = classify(line, budget)?;
        if let Block::Literal(indent) = block {
            if matches!(token.kind, Kind::Empty | Kind::Plain) || token.indent > indent {
                tokens.push(token);
                continue;
            }
            block = Block::None;
        }
        reject_capabilities(&line.text, line.number, budget)?;
        if token.kind == Kind::Entry && token.raw_value == "|" && !token.list {
            block = Block::Literal(token.indent);
        }
        tokens.push(token);
    }
    let mut parser = Parser {
        tokens: &tokens,
        index: 0,
        budget,
    };
    parser.skip_empty()?;
    if parser.index == tokens.len() {
        return Err(invalid(1, "YAML has no map or list"));
    }
    let indent = tokens[parser.index].indent;
    let value = parser.block(indent, 1)?;
    parser.skip_empty()?;
    if parser.index != tokens.len() {
        return Err(invalid(
            tokens[parser.index].number,
            "inconsistent YAML indentation or trailing content",
        ));
    }
    Ok(value)
}

enum Block {
    None,
    Literal(usize),
}

fn classify(line: &Line, budget: &mut Budget) -> Result<YamlLine, RenderError> {
    let tabs = line.text.bytes().filter(|&b| b == b'\t').count();
    let size = line.text.len() + tabs * 3;
    budget.text(size)?;
    let raw = line.text.replace('\t', "    ");
    let indent = raw.len() - raw.trim_start_matches(' ').len();
    let content = remove_comment(raw.trim());
    let list = content == "-" || content.starts_with("- ");
    let text = if list {
        content.strip_prefix("- ").unwrap_or("").trim()
    } else {
        content
    };
    let (kind, key, value) = if content.is_empty() {
        (Kind::Empty, "", "")
    } else if content == "-" {
        (Kind::Dash, "", "")
    } else if let Some(colon) = find_unquoted(text, ':') {
        (Kind::Entry, text[..colon].trim(), text[colon + 1..].trim())
    } else if list {
        (Kind::Scalar, "", text)
    } else {
        (Kind::Plain, "", text)
    };
    budget.text(key.len() + value.len() * 2)?;
    Ok(YamlLine {
        number: line.number,
        indent,
        kind,
        list,
        key: unquote(key).to_owned(),
        value: unquote(value).to_owned(),
        raw_value: value.to_owned(),
        raw,
    })
}

fn find_unquoted(text: &str, wanted: char) -> Option<usize> {
    let mut quote = '\0';
    for (offset, ch) in text.char_indices() {
        if quote != '\0' {
            if ch == quote {
                quote = '\0';
            }
        } else if matches!(ch, '\'' | '"') {
            quote = ch;
        } else if ch == wanted {
            return Some(offset);
        }
    }
    None
}

fn remove_comment(text: &str) -> &str {
    if text.starts_with('#') {
        return "";
    }
    let mut quote = '\0';
    for (offset, ch) in text.char_indices() {
        if matches!(ch, '\'' | '"') {
            if quote == '\0' {
                quote = ch;
            } else if quote == ch {
                quote = '\0';
            }
        }
        if quote == '\0' && ch == ' ' && text[offset..].starts_with(" #") {
            return &text[..offset];
        }
    }
    text
}

fn unquote(text: &str) -> &str {
    if text.len() >= 2
        && ((text.starts_with('"') && text.ends_with('"'))
            || (text.starts_with('\'') && text.ends_with('\'')))
    {
        return &text[1..text.len() - 1];
    }
    text
}

fn reject_capabilities(raw: &str, line: usize, budget: &mut Budget) -> Result<(), RenderError> {
    let text = remove_comment(raw.trim());
    let entry = text.strip_prefix("- ").unwrap_or(text);
    if find_unquoted(entry, ':').is_some_and(|colon| entry[..colon].trim() == "<<") {
        return Err(unsupported(line, "YAML merge key"));
    }
    let mut quote = '\0';
    let mut boundary = true;
    let mut escaped = false;
    for (offset, ch) in text.char_indices() {
        budget.charge(1)?;
        if quote != '\0' {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == quote {
                quote = '\0';
            }
            continue;
        }
        if matches!(ch, '\'' | '"') {
            quote = ch;
            boundary = false;
            continue;
        }
        if boundary && matches!(ch, '&' | '*' | '!') {
            return Err(unsupported(
                line,
                &format!(
                    "YAML anchor, alias, tag or directive at {:?}",
                    &text[offset..]
                ),
            ));
        }
        if boundary && text[offset..].starts_with("<<:") {
            return Err(unsupported(line, "YAML merge key"));
        }
        boundary = ch.is_whitespace() || matches!(ch, '[' | ',' | '{' | ':' | '-');
    }
    Ok(())
}

struct Parser<'a, 'b> {
    tokens: &'a [YamlLine],
    index: usize,
    budget: &'b mut Budget,
}
impl Parser<'_, '_> {
    fn skip_empty(&mut self) -> Result<(), RenderError> {
        while self
            .tokens
            .get(self.index)
            .is_some_and(|line| line.kind == Kind::Empty)
        {
            self.budget.charge(1)?;
            self.index += 1;
        }
        Ok(())
    }
    fn block(&mut self, indent: usize, depth: usize) -> Result<Value, RenderError> {
        self.budget.depth(depth)?;
        self.budget.item()?;
        if self.tokens[self.index].list {
            return self.list(indent, depth);
        }
        self.map(indent, depth)
    }
    fn map(&mut self, indent: usize, depth: usize) -> Result<Value, RenderError> {
        let mut entries = Vec::new();
        loop {
            self.skip_empty()?;
            let Some(line) = self.tokens.get(self.index) else {
                break;
            };
            if line.indent != indent || line.list {
                break;
            }
            if line.kind != Kind::Entry {
                return Err(unsupported(
                    line.number,
                    "YAML plain text/document marker outside keyed value",
                ));
            }
            let pair = self.entry(depth)?;
            self.replace(&mut entries, pair)?;
        }
        if entries.is_empty() {
            return Err(invalid(
                self.tokens[self.index].number,
                "expected YAML map entry",
            ));
        }
        Ok(Value::Object(entries))
    }
    fn list(&mut self, indent: usize, depth: usize) -> Result<Value, RenderError> {
        let mut values = Vec::new();
        loop {
            self.skip_empty()?;
            let Some(line) = self.tokens.get(self.index) else {
                break;
            };
            if line.indent != indent || !line.list {
                break;
            }
            self.budget.depth(depth + 1)?;
            self.budget.item()?;
            let value = match line.kind {
                Kind::Scalar => {
                    let value = Value::String(line.value.clone());
                    self.index += 1;
                    value
                }
                Kind::Entry => {
                    let mut entries = Vec::new();
                    let pair = self.entry(depth + 1)?;
                    self.replace(&mut entries, pair)?;
                    loop {
                        self.skip_empty()?;
                        if !self.tokens.get(self.index).is_some_and(|l| {
                            l.indent == indent + 2 && !l.list && l.kind == Kind::Entry
                        }) {
                            break;
                        }
                        let pair = self.entry(depth + 1)?;
                        self.replace(&mut entries, pair)?;
                    }
                    Value::Object(entries)
                }
                Kind::Dash => {
                    self.index += 1;
                    self.skip_empty()?;
                    let next = self
                        .tokens
                        .get(self.index)
                        .ok_or_else(|| invalid(line.number, "empty YAML plain dash item"))?;
                    if next.indent <= indent || next.list {
                        return Err(unsupported(
                            next.number,
                            "YAML empty/list-of-lists plain dash",
                        ));
                    }
                    self.block(next.indent, depth + 1)?
                }
                _ => return Err(invalid(line.number, "invalid YAML list item")),
            };
            values.push(value);
        }
        Ok(Value::Array(values))
    }
    fn entry(&mut self, depth: usize) -> Result<(String, Value), RenderError> {
        let line = &self.tokens[self.index];
        self.budget.item()?;
        self.budget.depth(depth + 1)?;
        let key = line.key.clone();
        self.index += 1;
        let value = if line.raw_value == "|" {
            if line.list {
                return Err(unsupported(line.number, "YAML list-item literal block"));
            }
            self.literal(line.indent)?
        } else if line.raw_value == ">"
            || ["|-", "|+", ">-", ">+"]
                .iter()
                .any(|s| line.raw_value.starts_with(s))
        {
            return Err(unsupported(
                line.number,
                "YAML folded block/chomping/indentation indicator",
            ));
        } else if line.raw_value.starts_with('[') && line.raw_value.ends_with(']') {
            self.flow(&line.raw_value[1..line.raw_value.len() - 1], depth + 1)?
        } else if line.raw_value.is_empty() {
            self.skip_empty()?;
            if let Some(next) = self
                .tokens
                .get(self.index)
                .filter(|l| l.indent > line.indent + if line.list { 2 } else { 0 })
            {
                if next.kind == Kind::Plain {
                    return Err(unsupported(
                        next.number,
                        "YAML plain continuation (outside admitted subset)",
                    ));
                }
                self.block(next.indent, depth + 1)?
            } else {
                Value::String(String::new())
            }
        } else {
            Value::String(line.value.clone())
        };
        Ok((key, value))
    }
    fn replace(
        &mut self,
        entries: &mut Vec<(String, Value)>,
        pair: (String, Value),
    ) -> Result<(), RenderError> {
        for (key, value) in entries.iter_mut() {
            self.budget.charge(key.len().min(pair.0.len()) + 1)?;
            if *key == pair.0 {
                *value = pair.1;
                return Ok(());
            }
        }
        entries.push(pair);
        Ok(())
    }
    fn literal(&mut self, indent: usize) -> Result<Value, RenderError> {
        let mut value = String::new();
        while let Some(line) = self.tokens.get(self.index) {
            self.budget.charge(1)?;
            if !matches!(line.kind, Kind::Plain | Kind::Empty) && line.indent <= indent {
                break;
            }
            let text = line.raw.trim();
            self.budget.text(text.len() + 1)?;
            value.push_str(text);
            value.push('\n');
            self.index += 1;
        }
        Ok(Value::String(value))
    }
    fn flow(&mut self, text: &str, depth: usize) -> Result<Value, RenderError> {
        self.budget.depth(depth)?;
        let mut values = Vec::new();
        let mut field = String::new();
        let mut quote = '\0';
        let mut quoted = false;
        let mut chars = text.chars();
        while let Some(ch) = chars.next() {
            self.budget.charge(ch.len_utf8())?;
            if ch == '\\' {
                if let Some(next) = chars.next() {
                    self.budget.text(next.len_utf8())?;
                    field.push(next);
                }
                continue;
            }
            if quote != '\0' {
                if ch == quote {
                    quote = '\0';
                } else {
                    self.budget.text(ch.len_utf8())?;
                    field.push(ch);
                }
                continue;
            }
            if !quoted && field.trim().is_empty() && matches!(ch, '\'' | '"') {
                quote = ch;
                quoted = true;
                field.clear();
                continue;
            }
            if ch == ',' {
                self.budget.item()?;
                values.push(Value::String(if quoted {
                    std::mem::take(&mut field)
                } else {
                    let value = field.trim().to_owned();
                    field.clear();
                    value
                }));
                quoted = false;
                continue;
            }
            if matches!(ch, '[' | ']') {
                return Err(unsupported(1, "YAML nested flow containers"));
            }
            self.budget.text(ch.len_utf8())?;
            field.push(ch);
        }
        if quote != '\0' {
            return Err(invalid(1, "unterminated YAML flow quote"));
        }
        self.budget.item()?;
        values.push(Value::String(if quoted {
            field
        } else {
            field.trim().to_owned()
        }));
        Ok(Value::Array(values))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RenderOptions;
    fn value(text: &str) -> Value {
        let lines: Vec<_> = text
            .lines()
            .enumerate()
            .map(|(i, s)| Line {
                number: i + 2,
                text: s.into(),
            })
            .collect();
        parse(&lines, &mut Budget::new(&RenderOptions::default())).unwrap()
    }

    #[test]
    fn should_replace_yaml_duplicate_without_moving_first_key_and_keep_string_scalars() {
        // given
        let source = "z: first\na: 2\nz: last\nb: true\nn: null\nobj: {}\nseq: []";
        // when
        let actual = value(source);
        // then
        assert_eq!(
            actual,
            Value::Object(vec![
                ("z".into(), Value::String("last".into())),
                ("a".into(), Value::String("2".into())),
                ("b".into(), Value::String("true".into())),
                ("n".into(), Value::String("null".into())),
                ("obj".into(), Value::String("{}".into())),
                ("seq".into(), Value::Array(vec![Value::String("".into())]))
            ])
        );
    }

    #[test]
    fn should_apply_literal_block_trimming_and_nonstructured_stopping_quirk() {
        // given
        let source = "a: |\n  first  \nplain\n  second: value\n\nb: next";
        // when
        let actual = value(source);
        // then
        assert_eq!(
            actual,
            Value::Object(vec![
                (
                    "a".into(),
                    Value::String("first\nplain\nsecond: value\n\n".into())
                ),
                ("b".into(), Value::String("next".into()))
            ])
        );
    }

    #[test]
    fn should_distinguish_map_unquoting_from_flow_list_escaping() {
        // given
        let source = "\"a:b\": \"x\\ny # text\"\nlist: [one, \"two,three\", 'x\\ny']\nnext: 'it''s' # comment";
        // when
        let actual = value(source);
        // then
        assert_eq!(
            actual,
            Value::Object(vec![
                ("a:b".into(), Value::String("x\\ny # text".into())),
                (
                    "list".into(),
                    Value::Array(vec![
                        Value::String("one".into()),
                        Value::String("two,three".into()),
                        Value::String("xny".into())
                    ])
                ),
                ("next".into(), Value::String("it''s".into()))
            ])
        );
    }

    #[test]
    fn should_preserve_list_of_maps_and_plain_dash_map_ownership() {
        // given
        let source =
            "people:\n  - name: one\n    age: 2\n  - name: two\n    age: 3\n  -\n    name: three";
        // when
        let actual = value(source);
        // then
        assert_eq!(
            actual,
            Value::Object(vec![(
                "people".into(),
                Value::Array(vec![
                    Value::Object(vec![
                        ("name".into(), Value::String("one".into())),
                        ("age".into(), Value::String("2".into()))
                    ]),
                    Value::Object(vec![
                        ("name".into(), Value::String("two".into())),
                        ("age".into(), Value::String("3".into()))
                    ]),
                    Value::Object(vec![("name".into(), Value::String("three".into()))])
                ])
            )])
        );
    }
}
