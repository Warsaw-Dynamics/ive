//! PlantUML MIT HclParser/HclSource ordered data grammar; never evaluates expressions.
//! Copyright 2009-2024 Arnaud Roques; ordered-object Copyright (c) 2016 EclipseSource.
//! Both MIT grants and exact source traces retained in provenance/.
use super::{
    budget::{Budget, bound, invalid, unsupported},
    data::Value,
    source::Line,
};
use crate::RenderError;

pub(super) struct Field {
    pub name: String,
    pub value: HclValue,
}
pub(super) enum HclValue {
    Object(Vec<Field>),
    Array(Vec<HclValue>),
    String(String),
}
pub(super) struct Block {
    pub name: String,
    pub fields: Vec<Field>,
}
pub(super) struct Document {
    pub blocks: Vec<Block>,
}

pub(super) fn parse(lines: &[Line], budget: &mut Budget) -> Result<Document, RenderError> {
    let bytes = lines.iter().map(|l| l.text.len() + 1).sum();
    budget.text(bytes)?;
    let mut input = String::with_capacity(bytes);
    for line in lines {
        budget.charge(line.text.len() + 1)?;
        if !line.text.trim_start().starts_with('#') {
            input.push_str(&line.text);
        }
        input.push('\n');
    }
    let mut parser = Parser {
        input: &input,
        offset: 0,
        line: lines.first().map_or(1, |l| l.number),
        budget,
    };
    let mut blocks: Vec<Block> = Vec::new();
    while parser.peek()?.is_some() {
        parser.budget.item()?;
        let header_line = parser.line;
        let mut name = String::new();
        while parser.peek()? != Some('{') {
            if parser.line != header_line {
                return Err(invalid(
                    header_line,
                    format!(
                        "HCL block header {name:?} crosses a physical line before '{{'; next token is at line {}",
                        parser.line
                    ),
                ));
            }
            let quoted = parser.peek()? == Some('"');
            let word = parser.string()?;
            let extra = word.len() + usize::from(quoted) * 2 + usize::from(!name.is_empty());
            bound("label bytes", name.len().saturating_add(extra), 16 * 1024)?;
            parser.budget.text(extra)?;
            if !name.is_empty() {
                name.push(' ');
            }
            if quoted {
                name.push('"');
            }
            name.push_str(&word);
            if quoted {
                name.push('"');
            }
        }
        if name.is_empty() {
            return Err(invalid(parser.line, "HCL block requires a name"));
        }
        for block in &blocks {
            parser.budget.charge(block.name.len() + name.len() + 1)?;
            if block.name == name {
                return Err(unsupported(
                    parser.line,
                    &format!("duplicate HCL top block {name:?}"),
                ));
            }
        }
        let fields = parser.object(1)?;
        blocks.push(Block { name, fields });
    }
    if blocks.is_empty() {
        return Err(invalid(parser.line, "HCL requires a named block"));
    }
    Ok(Document { blocks })
}

pub(super) fn lower(document: Document, budget: &mut Budget) -> Result<Value, RenderError> {
    if document.blocks.len() == 1 {
        return lower_value(
            HclValue::Object(document.blocks.into_iter().next().unwrap().fields),
            1,
            budget,
        );
    }
    budget.item()?;
    let mut fields = Vec::new();
    for block in document.blocks {
        fields.push((
            block.name,
            lower_value(HclValue::Object(block.fields), 2, budget)?,
        ));
    }
    Ok(Value::Object(fields))
}
fn lower_value(value: HclValue, depth: usize, budget: &mut Budget) -> Result<Value, RenderError> {
    budget.depth(depth)?;
    budget.charge(1)?;
    Ok(match value {
        HclValue::String(s) => Value::String(s),
        HclValue::Object(fields) => {
            let mut result = Vec::new();
            for field in fields {
                result.push((field.name, lower_value(field.value, depth + 1, budget)?));
            }
            Value::Object(result)
        }
        HclValue::Array(values) => {
            let mut result = Vec::new();
            for value in values {
                result.push(lower_value(value, depth + 1, budget)?);
            }
            Value::Array(result)
        }
    })
}
struct Parser<'a, 'b> {
    input: &'a str,
    offset: usize,
    line: usize,
    budget: &'b mut Budget,
}
impl Parser<'_, '_> {
    fn peek(&mut self) -> Result<Option<char>, RenderError> {
        while let Some(ch) = self.input[self.offset..].chars().next() {
            if !ch.is_whitespace() {
                return Ok(Some(ch));
            }
            self.take()?;
        }
        Ok(None)
    }
    fn take(&mut self) -> Result<Option<char>, RenderError> {
        let ch = self.input[self.offset..].chars().next();
        if let Some(ch) = ch {
            self.budget.charge(ch.len_utf8())?;
            self.offset += ch.len_utf8();
            if ch == '\n' {
                self.line += 1;
            }
        }
        Ok(ch)
    }
    fn expect(&mut self, expected: char) -> Result<(), RenderError> {
        if self.peek()? != Some(expected) {
            return Err(invalid(
                self.line,
                format!(
                    "HCL expected {expected:?}, found {:?}",
                    self.input[self.offset..].chars().next()
                ),
            ));
        }
        self.take()?;
        Ok(())
    }
    fn object(&mut self, depth: usize) -> Result<Vec<Field>, RenderError> {
        self.budget.depth(depth)?;
        self.budget.item()?;
        self.expect('{')?;
        let mut fields = Vec::new();
        while self.peek()? != Some('}') {
            self.budget.item()?;
            let name = self.string()?;
            match self.peek()? {
                Some('=' | ':') => {
                    self.take()?;
                }
                _ => {
                    return Err(invalid(
                        self.line,
                        format!("HCL field {name:?} requires = or :"),
                    ));
                }
            }
            let value = self.value(depth + 1)?;
            fields.push(Field { name, value });
        }
        self.expect('}')?;
        Ok(fields)
    }
    fn value(&mut self, depth: usize) -> Result<HclValue, RenderError> {
        self.budget.depth(depth)?;
        match self.peek()? {
            Some('{') => self.object(depth).map(HclValue::Object),
            Some('[') => self.array(depth),
            _ => {
                self.budget.item()?;
                self.string().map(HclValue::String)
            }
        }
    }
    fn array(&mut self, depth: usize) -> Result<HclValue, RenderError> {
        self.budget.item()?;
        self.expect('[')?;
        let mut values = Vec::new();
        while self.peek()? != Some(']') {
            match self.peek()? {
                Some('"' | '{') => values.push(self.value(depth + 1)?),
                _ => {
                    return Err(unsupported(
                        self.line,
                        "HCL arrays admit quoted strings and objects only",
                    ));
                }
            }
            if self.peek()? == Some(',') {
                self.take()?;
            } else if self.peek()? != Some(']') {
                return Err(invalid(self.line, "HCL array members require commas"));
            }
        }
        self.expect(']')?;
        Ok(HclValue::Array(values))
    }
    fn string(&mut self) -> Result<String, RenderError> {
        let quoted = self.peek()? == Some('"');
        if quoted {
            self.take()?;
        }
        let mut result = String::new();
        loop {
            let ch = self.input[self.offset..]
                .chars()
                .next()
                .ok_or_else(|| invalid(self.line, "unexpected HCL end while reading string"))?;
            if quoted && ch == '"' {
                self.take()?;
                break;
            }
            if !quoted && (ch.is_whitespace() || "{}[]=:,".contains(ch)) {
                break;
            }
            if !quoted && !(ch.is_alphanumeric() || "_.-".contains(ch)) {
                return Err(unsupported(
                    self.line,
                    &format!("HCL expression/call token {ch:?}"),
                ));
            }
            if ch == '\n' || ch == '\r' {
                return Err(invalid(
                    self.line,
                    "multiline HCL quoted string is unsupported",
                ));
            }
            self.take()?;
            let ch = if quoted && ch == '\\' {
                self.take()?
                    .ok_or_else(|| invalid(self.line, "unterminated HCL escape"))?
            } else {
                ch
            };
            if ch == '\n' || ch == '\r' {
                return Err(invalid(self.line, "physical newline in HCL escape"));
            }
            bound(
                "label bytes",
                result.len().saturating_add(ch.len_utf8()),
                16 * 1024,
            )?;
            self.budget.text(ch.len_utf8())?;
            result.push(ch);
        }
        if !quoted && result.is_empty() {
            return Err(invalid(
                self.line,
                format!(
                    "HCL expected string at {:?}",
                    self.input[self.offset..].chars().next()
                ),
            ));
        }
        self.budget.charge(result.len().saturating_mul(2))?;
        if result.contains("${") || result.contains("%{") {
            return Err(unsupported(self.line, "HCL interpolation"));
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RenderOptions;

    #[test]
    fn should_preserve_hcl_block_field_types_order_and_duplicates() {
        // given
        let lines = vec![Line {
            number: 2,
            text: "service \"billing\" { x=3 x=true y=[\"north\",{}] }".into(),
        }];
        let mut budget = Budget::new(&RenderOptions::default());
        // when
        let document = parse(&lines, &mut budget).unwrap();
        // then
        assert_eq!(document.blocks[0].name, "service \"billing\"");
        assert_eq!(
            document.blocks[0]
                .fields
                .iter()
                .map(|f| f.name.as_str())
                .collect::<Vec<_>>(),
            ["x", "x", "y"]
        );
        let HclValue::String(ref first) = document.blocks[0].fields[0].value else {
            panic!()
        };
        assert_eq!(first, "3");
        let lowered = lower(document, &mut budget).unwrap();
        assert_eq!(
            lowered,
            Value::Object(vec![
                ("x".into(), Value::String("3".into())),
                ("x".into(), Value::String("true".into())),
                (
                    "y".into(),
                    Value::Array(vec![
                        Value::String("north".into()),
                        Value::Object(Vec::new())
                    ])
                )
            ])
        );
    }

    #[test]
    fn should_check_hcl_value_depth_and_item_budget_before_scanning_content() {
        // given
        let mut options = RenderOptions::default();
        options.limits.nesting_depth = 1;
        let mut budget = Budget::new(&options);
        let mut parser = Parser {
            input: "\"unseen\"",
            offset: 0,
            line: 9,
            budget: &mut budget,
        };
        // when
        let result = parser.value(2);
        // then
        assert!(result.is_err());
        assert_eq!(parser.offset, 0);
        assert_eq!(parser.budget.work, 0);
        options.limits.model_items = 1;
        let mut budget = Budget::new(&options);
        budget.item().unwrap();
        let mut parser = Parser {
            input: "\"unseen\"",
            offset: 0,
            line: 9,
            budget: &mut budget,
        };
        assert!(parser.value(1).is_err());
        assert_eq!(parser.offset, 0);
    }
}
