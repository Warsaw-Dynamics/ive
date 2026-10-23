//! MIT PlantUML json/minimal-json ordered value semantics; Copyright (c) 2016 EclipseSource.
//! Both MIT grants retained in provenance/LICENSE-PlantUML-MIT.txt.
use super::{
    budget::{Budget, bound, invalid, unsupported},
    source::Line,
};
use crate::RenderError;

#[derive(Debug, PartialEq)]
pub(super) enum Value {
    Object(Vec<(String, Value)>),
    Array(Vec<Value>),
    String(String),
    Number(String),
    Boolean(bool),
    Null,
}
impl Value {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Object(_) => "object",
            Self::Array(_) => "array",
            Self::String(_) => "string",
            Self::Number(_) => "number",
            Self::Boolean(_) => "boolean",
            Self::Null => "null",
        }
    }
    pub fn is_container(&self) -> bool {
        matches!(self, Self::Object(_) | Self::Array(_))
    }
    pub fn display(&self) -> &str {
        match self {
            Self::String(s) | Self::Number(s) => s,
            Self::Boolean(true) => "☑ true",
            Self::Boolean(false) => "☐ false",
            Self::Null => "␀",
            _ => "",
        }
    }
}

pub(super) fn parse_json(lines: &[Line], budget: &mut Budget) -> Result<Value, RenderError> {
    let bytes = lines.iter().map(|l| l.text.len() + 1).sum::<usize>();
    budget.text(bytes)?;
    let mut input = String::with_capacity(bytes);
    for line in lines {
        input.push_str(&line.text);
        input.push('\n');
    }
    let mut parser = JsonParser {
        input: &input,
        offset: 0,
        line: lines[0].number,
        budget,
    };
    let value = parser.value(1)?;
    parser.space()?;
    if parser.offset != input.len() {
        return Err(invalid(parser.line, "trailing JSON input"));
    }
    Ok(value)
}

struct JsonParser<'a, 'b> {
    input: &'a str,
    offset: usize,
    line: usize,
    budget: &'b mut Budget,
}
impl JsonParser<'_, '_> {
    fn value(&mut self, depth: usize) -> Result<Value, RenderError> {
        self.budget.depth(depth)?;
        self.budget.item()?;
        self.space()?;
        match self.peek() {
            Some(b'{') => self.object(depth),
            Some(b'[') => self.array(depth),
            Some(b'"') => self.string().map(Value::String),
            Some(b't') => {
                self.keyword("true")?;
                Ok(Value::Boolean(true))
            }
            Some(b'f') => {
                self.keyword("false")?;
                Ok(Value::Boolean(false))
            }
            Some(b'n') => {
                self.keyword("null")?;
                Ok(Value::Null)
            }
            Some(b'-' | b'0'..=b'9') => self.number(),
            Some(b'/' | b'#' | b'!') => Err(unsupported(
                self.line,
                "JSON comments, directives or styles outside strict JSON subset",
            )),
            _ => Err(invalid(self.line, "expected JSON value")),
        }
    }
    fn object(&mut self, depth: usize) -> Result<Value, RenderError> {
        self.bump()?;
        self.space()?;
        let mut members = Vec::new();
        if self.take(b'}')? {
            return Ok(Value::Object(members));
        }
        loop {
            self.budget.item()?;
            self.space()?;
            let key = self.string()?;
            self.space()?;
            self.expect(b':')?;
            let value = self.value(depth + 1)?;
            members.push((key, value));
            self.space()?;
            if self.take(b'}')? {
                return Ok(Value::Object(members));
            }
            self.expect(b',')?;
        }
    }
    fn array(&mut self, depth: usize) -> Result<Value, RenderError> {
        self.bump()?;
        self.space()?;
        let mut values = Vec::new();
        if self.take(b']')? {
            return Ok(Value::Array(values));
        }
        loop {
            let value = self.value(depth + 1)?;
            values.push(value);
            self.space()?;
            if self.take(b']')? {
                return Ok(Value::Array(values));
            }
            self.expect(b',')?;
        }
    }
    fn string(&mut self) -> Result<String, RenderError> {
        let start = self.offset;
        self.expect(b'"')?;
        loop {
            bound("JSON string lexeme bytes", self.offset - start, 16 * 1024)?;
            match self.peek() {
                Some(b'"') => {
                    self.bump()?;
                    break;
                }
                Some(b'\\') => {
                    self.bump()?;
                    self.bump()?;
                }
                Some(0..=31) | None => {
                    return Err(invalid(
                        self.line,
                        "unterminated or control character in JSON string",
                    ));
                }
                _ => self.bump()?,
            }
        }
        let raw = &self.input[start..self.offset];
        self.budget.text(raw.len())?;
        serde_json::from_str::<String>(raw)
            .map_err(|error| invalid(self.line, format!("invalid JSON string: {error}")))
    }
    fn number(&mut self) -> Result<Value, RenderError> {
        let start = self.offset;
        self.take(b'-')?;
        if self.take(b'0')? {
        } else {
            self.digits()?;
        }
        if self.take(b'.')? {
            self.digits()?;
        }
        if self.take(b'e')? || self.take(b'E')? {
            if self.take(b'+')? {
            } else {
                self.take(b'-')?;
            }
            let exponent = self.offset;
            self.digits()?;
            let value = self.input[exponent..self.offset]
                .parse::<u16>()
                .unwrap_or(u16::MAX);
            if value > 308 {
                return Err(invalid(
                    self.line,
                    "JSON numeric exponent exceeds native bound 308",
                ));
            }
        }
        let raw = &self.input[start..self.offset];
        bound("JSON number lexeme bytes", raw.len(), 128)?;
        if !raw.parse::<f64>().is_ok_and(f64::is_finite) {
            return Err(invalid(
                self.line,
                format!("nonfinite or malformed JSON number {raw:?}"),
            ));
        }
        self.budget.text(raw.len())?;
        Ok(Value::Number(raw.to_owned()))
    }
    fn digits(&mut self) -> Result<(), RenderError> {
        let start = self.offset;
        while self.peek().is_some_and(|b| b.is_ascii_digit()) {
            bound("JSON numeric digits", self.offset - start + 1, 128)?;
            self.bump()?;
        }
        if self.offset == start {
            return Err(invalid(self.line, "expected JSON numeric digit"));
        }
        Ok(())
    }
    fn keyword(&mut self, word: &str) -> Result<(), RenderError> {
        for byte in word.bytes() {
            self.expect(byte)?;
        }
        Ok(())
    }
    fn space(&mut self) -> Result<(), RenderError> {
        while self
            .peek()
            .is_some_and(|b| matches!(b, b' ' | b'\t' | b'\r' | b'\n'))
        {
            self.bump()?;
        }
        Ok(())
    }
    fn peek(&self) -> Option<u8> {
        self.input.as_bytes().get(self.offset).copied()
    }
    fn bump(&mut self) -> Result<(), RenderError> {
        self.budget.charge(1)?;
        let byte = self
            .peek()
            .ok_or_else(|| invalid(self.line, "unexpected end of JSON"))?;
        self.line += usize::from(byte == b'\n');
        self.offset += 1;
        Ok(())
    }
    fn take(&mut self, byte: u8) -> Result<bool, RenderError> {
        if self.peek() == Some(byte) {
            self.bump()?;
            return Ok(true);
        }
        Ok(false)
    }
    fn expect(&mut self, byte: u8) -> Result<(), RenderError> {
        if self.take(byte)? {
            return Ok(());
        }
        Err(invalid(
            self.line,
            format!(
                "expected JSON {:?}, found {:?}",
                byte as char,
                self.peek().map(char::from)
            ),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RenderOptions;

    #[test]
    fn should_keep_ordered_duplicate_members_and_exact_scalar_identities() {
        // given
        let lines = [Line {
            number: 7,
            text: r#"{"z":1,"a":[null,true,"2",{},[]],"z":3}"#.into(),
        }];
        // when
        let value = parse_json(&lines, &mut Budget::new(&RenderOptions::default())).unwrap();
        // then
        assert_eq!(
            value,
            Value::Object(vec![
                ("z".into(), Value::Number("1".into())),
                (
                    "a".into(),
                    Value::Array(vec![
                        Value::Null,
                        Value::Boolean(true),
                        Value::String("2".into()),
                        Value::Object(vec![]),
                        Value::Array(vec![])
                    ])
                ),
                ("z".into(), Value::Number("3".into()))
            ])
        );
    }

    #[test]
    fn should_reject_json_depth_and_items_before_descending_or_retaining_value() {
        // given
        let mut config = RenderOptions::default();
        config.limits.nesting_depth = 1;
        let mut budget = Budget::new(&config);
        let mut parser = JsonParser {
            input: "[[[]]]",
            offset: 0,
            line: 4,
            budget: &mut budget,
        };
        // when
        let error = parser.value(1).unwrap_err();
        // then
        assert!(error.to_string().contains("nesting depth"));
        assert_eq!(parser.offset, 1);
    }

    #[test]
    fn should_bound_numeric_lexemes_before_copying_and_report_original_line() {
        // given
        let source = format!("[\n{}]", "9".repeat(200));
        let lines = [Line {
            number: 13,
            text: source,
        }];
        // when
        let error = parse_json(&lines, &mut Budget::new(&RenderOptions::default())).unwrap_err();
        // then
        assert!(error.to_string().contains("numeric digits"));
    }
}
