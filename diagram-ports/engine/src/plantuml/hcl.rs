//! PlantUML MIT HclParser/HclSource ordered data grammar; never evaluates expressions.
//! Copyright 2009-2024 Arnaud Roques; ordered-object Copyright (c) 2016 EclipseSource.
//! Both MIT grants and exact source traces retained in provenance/.
use super::{budget::Budget, data::Value};
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
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RenderOptions, plantuml::parser::hcl::parse};

    #[test]
    fn should_preserve_hcl_block_field_types_order_and_duplicates() {
        // given
        let lines = ["service \"billing\" { x=3 x=true y=[\"north\",{}] }".to_owned()].join("\n");
        let mut budget = Budget::new(&RenderOptions::default());
        // when
        let document =
            crate::plantuml::parser::test_support::body((&lines, "hcl"), &mut budget, parse)
                .unwrap();
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
}
