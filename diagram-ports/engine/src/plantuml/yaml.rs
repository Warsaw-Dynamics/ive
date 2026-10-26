//! MIT PlantUML yaml/parser: narrow string scalars, indentation, literal blocks and ordered replacement.
use super::budget::Budget;
use crate::RenderError;

#[derive(Default)]
pub(in crate::plantuml) struct FlowField {
    text: String,
    quoted: bool,
    has_content: bool,
}

impl FlowField {
    pub fn push(&mut self, ch: char, budget: &mut Budget) -> Result<(), RenderError> {
        budget.text(ch.len_utf8())?;
        self.has_content |= !ch.is_whitespace();
        self.text.push(ch);
        Ok(())
    }

    pub fn starts_quote(&self, ch: char) -> bool {
        !self.quoted && !self.has_content && matches!(ch, '\'' | '"')
    }

    pub fn open_quote(&mut self) {
        self.text.clear();
        self.quoted = true;
        self.has_content = false;
    }

    pub fn finish(&mut self, budget: &mut Budget) -> Result<String, RenderError> {
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

pub(in crate::plantuml) fn remove_comment(text: &str) -> &str {
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

pub(in crate::plantuml) fn unquote(text: &str) -> &str {
    if text.len() >= 2
        && ((text.starts_with('"') && text.ends_with('"'))
            || (text.starts_with('\'') && text.ends_with('\'')))
    {
        return &text[1..text.len() - 1];
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        RenderOptions,
        plantuml::{
            data::Value,
            parser::yaml::{parse, test_flow},
        },
    };
    fn value(text: &str) -> Value {
        let lines = text.to_owned();
        crate::plantuml::parser::test_support::body(
            (&lines, "yaml"),
            &mut Budget::new(&RenderOptions::default()),
            parse,
        )
        .unwrap()
    }

    #[test]
    fn should_check_flow_scalar_depth_before_any_field_accumulation() {
        // given
        for text in ["one", "\"two\"", "", "   ", "\\ ", "one,two"] {
            let mut config = RenderOptions::default();
            config.limits.nesting_depth = 2;
            let mut budget = Budget::new(&config);
            // when
            let result = test_flow(text, 2, &mut budget);
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
            // when
            let result = test_flow(text, 2, &mut budget).unwrap();
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
            // when
            let result = test_flow(&text, 2, &mut budget).unwrap();
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
            // when
            let result = test_flow(&text, 2, &mut budget).unwrap();
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
        let text = "\\x\"y\",\\ \"z\",\\'literal,\\\u{a0}\"spaced\"";
        // when
        let result = test_flow(text, 2, &mut budget).unwrap();
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
            // when
            let error = test_flow(&text, 2, &mut budget).unwrap_err();
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
