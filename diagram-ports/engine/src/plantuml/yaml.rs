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

#[derive(Default)]
struct FlowField {
    text: String,
    quoted: bool,
    has_content: bool,
}

impl FlowField {
    fn push(&mut self, ch: char, budget: &mut Budget) -> Result<(), RenderError> {
        budget.text(ch.len_utf8())?;
        self.has_content |= !ch.is_whitespace();
        self.text.push(ch);
        Ok(())
    }

    fn starts_quote(&self, ch: char) -> bool {
        !self.quoted && !self.has_content && matches!(ch, '\'' | '"')
    }

    fn open_quote(&mut self) {
        self.text.clear();
        self.quoted = true;
        self.has_content = false;
    }

    fn finish(&mut self, budget: &mut Budget) -> Result<String, RenderError> {
        let value = if self.quoted {
            std::mem::take(&mut self.text)
        } else {
            budget.charge(self.text.len())?;
            let trimmed = self.text.trim();
            budget.text(trimmed.len())?;
            trimmed.to_owned()
        };
        self.text.clear();
        self.quoted = false;
        self.has_content = false;
        Ok(value)
    }
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

pub(super) fn find_unquoted(text: &str, wanted: char) -> Option<usize> {
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
        self.begin_flow_scalar(depth)?;
        let mut values = Vec::new();
        let mut field = FlowField::default();
        let mut quote = '\0';
        let mut chars = text.chars();
        while let Some(ch) = chars.next() {
            self.budget.charge(ch.len_utf8())?;
            if ch == '\\' {
                if let Some(next) = chars.next() {
                    self.budget.charge(next.len_utf8())?;
                    field.push(next, self.budget)?;
                }
                continue;
            }
            if quote != '\0' {
                if ch == quote {
                    quote = '\0';
                } else {
                    field.push(ch, self.budget)?;
                }
                continue;
            }
            if field.starts_quote(ch) {
                quote = ch;
                field.open_quote();
                continue;
            }
            if ch == ',' {
                values.push(Value::String(field.finish(self.budget)?));
                self.begin_flow_scalar(depth)?;
                continue;
            }
            if matches!(ch, '[' | ']') {
                return Err(unsupported(1, "YAML nested flow containers"));
            }
            field.push(ch, self.budget)?;
        }
        if quote != '\0' {
            return Err(invalid(1, "unterminated YAML flow quote"));
        }
        values.push(Value::String(field.finish(self.budget)?));
        Ok(Value::Array(values))
    }

    fn begin_flow_scalar(&mut self, depth: usize) -> Result<(), RenderError> {
        self.budget.depth(depth + 1)?;
        self.budget.item()
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
    fn should_check_flow_scalar_depth_before_any_field_accumulation() {
        // given
        for text in ["one", "\"two\"", "", "   ", "\\ ", "one,two"] {
            let mut config = RenderOptions::default();
            config.limits.nesting_depth = 2;
            let mut budget = Budget::new(&config);
            let mut parser = Parser {
                tokens: &[],
                index: 0,
                budget: &mut budget,
            };
            // when
            let result = parser.flow(text, 2);
            // then
            let error = result.expect_err("flow scalar depth 3 exceeded limit 2");
            assert!(
                error
                    .to_string()
                    .contains("nesting depth value 3 exceeds limit 2"),
                "{error}"
            );
            assert_eq!(
                budget.work, 0,
                "field work happened before depth rejection for {text:?}"
            );
        }
    }

    #[test]
    fn should_admit_flow_scalars_at_exact_depth_including_mandatory_empty_element() {
        // given
        for (text, expected) in [
            ("one", "one"),
            ("\" two \"", " two "),
            ("", ""),
            ("   ", ""),
        ] {
            let mut config = RenderOptions::default();
            config.limits.nesting_depth = 3;
            let mut budget = Budget::new(&config);
            let mut parser = Parser {
                tokens: &[],
                index: 0,
                budget: &mut budget,
            };
            // when
            let result = parser.flow(text, 2).unwrap();
            // then
            assert_eq!(result, Value::Array(vec![Value::String(expected.into())]));
        }
    }

    #[test]
    fn should_scan_long_whitespace_flow_fields_with_linear_charged_work() {
        // given
        for size in [4_000, 8_000, 16_000] {
            let text = " ".repeat(size);
            let mut config = RenderOptions::default();
            config.limits.layout_work = 4 * text.len() + 32;
            let mut budget = Budget::new(&config);
            let mut parser = Parser {
                tokens: &[],
                index: 0,
                budget: &mut budget,
            };
            // when
            let result = parser.flow(&text, 2).unwrap();
            // then
            assert_eq!(result, Value::Array(vec![Value::String(String::new())]));
            assert!(budget.work >= text.len(), "input scan was not charged");
            eprintln!(
                "flow whitespace bytes={size} bounded_work={} limit={}",
                budget.work, config.limits.layout_work
            );
        }
    }

    #[test]
    fn should_scan_long_quoted_and_escaped_flow_fields_with_linear_charged_work() {
        // given
        for size in [4_000, 8_000, 16_000] {
            let spaces = " ".repeat(size);
            let text = format!(
                "{spaces}\" one \",{spaces}\\ {spaces}tail,{spaces}\\\"literal,{spaces}\\ \' two \'"
            );
            let mut config = RenderOptions::default();
            config.limits.layout_work = 4 * text.len() + 64;
            let mut budget = Budget::new(&config);
            let mut parser = Parser {
                tokens: &[],
                index: 0,
                budget: &mut budget,
            };
            // when
            let result = parser.flow(&text, 2).unwrap();
            // then
            assert_eq!(
                result,
                Value::Array(vec![
                    Value::String(" one ".into()),
                    Value::String("tail".into()),
                    Value::String("\"literal".into()),
                    Value::String(" two ".into())
                ])
            );
            eprintln!(
                "flow mixed bytes={} bounded_work={} limit={}",
                text.len(),
                budget.work,
                config.limits.layout_work
            );
        }
    }

    #[test]
    fn should_preserve_escaped_content_when_deciding_whether_quotes_open() {
        // given
        let mut budget = Budget::new(&RenderOptions::default());
        let mut parser = Parser {
            tokens: &[],
            index: 0,
            budget: &mut budget,
        };
        let text = "\\x\"y\",\\ \"z\",\\'literal,\\\u{a0}\"spaced\"";
        // when
        let result = parser.flow(text, 2).unwrap();
        // then
        assert_eq!(
            result,
            Value::Array(vec![
                Value::String("x\"y\"".into()),
                Value::String("z".into()),
                Value::String("'literal".into()),
                Value::String("spaced".into())
            ])
        );
    }

    #[test]
    fn should_charge_trimmed_flow_copy_before_allocating_a_second_string() {
        // given
        let mut config = RenderOptions::default();
        config.limits.source_bytes = 1;
        let mut budget = Budget::new(&config);
        let mut field = FlowField::default();
        for ch in "original".chars() {
            field.push(ch, &mut budget).unwrap();
        }
        // when
        let error = field.finish(&mut budget).unwrap_err();
        // then
        assert!(
            error
                .to_string()
                .contains("expanded model text bytes value 16 exceeds limit 8"),
            "{error}"
        );
        assert_eq!(field.text, "original");
    }

    #[test]
    fn should_enforce_flow_item_limit_before_next_field_accumulation() {
        // given
        let mut observed_work = Vec::new();
        for suffix in ["two".to_owned(), format!("{}tail", " ".repeat(16_000))] {
            let text = format!("one,{suffix}");
            let mut config = RenderOptions::default();
            config.limits.model_items = 1;
            let mut budget = Budget::new(&config);
            let mut parser = Parser {
                tokens: &[],
                index: 0,
                budget: &mut budget,
            };
            // when
            let error = parser.flow(&text, 2).unwrap_err();
            // then
            assert!(
                error
                    .to_string()
                    .contains("model items value 2 exceeds limit 1"),
                "{error}"
            );
            observed_work.push(budget.work);
        }
        assert_eq!(
            observed_work[0], observed_work[1],
            "rejected field contents were processed"
        );
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
