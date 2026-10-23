//! MIT PlantUML json/minimal-json ordered value semantics; Copyright (c) 2016 EclipseSource.
//! Both MIT grants retained in provenance/LICENSE-PlantUML-MIT.txt.

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        RenderOptions,
        plantuml::{
            budget::Budget,
            parser::{json::parse, outline::test_parse},
            source::Line,
        },
    };

    #[test]
    fn should_keep_ordered_duplicate_members_and_exact_scalar_identities() {
        // given
        let lines = [Line {
            number: 7,
            text: r#"{"z":1,"a":[null,true,"2",{},[]],"z":3}"#.into(),
        }];
        // when
        let value = test_parse(
            &lines,
            "json",
            &mut Budget::new(&RenderOptions::default()),
            parse,
        )
        .unwrap();
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
    fn should_bound_numeric_lexemes_before_copying_and_report_original_line() {
        // given
        let source = format!("[\n{}]", "9".repeat(200));
        let lines = [Line {
            number: 13,
            text: source,
        }];
        // when
        let error = test_parse(
            &lines,
            "json",
            &mut Budget::new(&RenderOptions::default()),
            parse,
        )
        .unwrap_err();
        // then
        assert!(error.to_string().contains("numeric digits"));
    }
}
