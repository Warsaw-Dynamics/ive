use super::{
    BodySource,
    context::Checked,
    error::generated_failure,
    railroad::{self, Family},
};
use crate::{
    RenderError,
    plantuml::{
        budget::Budget,
        ebnf::{Builder, EbnfDocument},
    },
};
pub(in crate::plantuml) fn parse(
    body: BodySource<'_>,
    budget: &mut Budget,
) -> Result<EbnfDocument, RenderError> {
    let result = (|| {
        let tokens = railroad::lex(&body, budget, Family::Ebnf)?;
        let mut storage =
            railroad::admit((&tokens, tokens.capacity()), &body, budget, Family::Ebnf)?;
        let mut exprs = Vec::new();
        storage.reserve(
            &mut exprs,
            tokens.len().min(budget.limits.model_items),
            budget,
        )?;
        let iterator = Checked::new(tokens, budget.control.clone());
        let mut builder = Builder {
            storage,
            document: EbnfDocument {
                rules: Vec::new(),
                exprs,
            },
            budget,
        };
        super::generated_ebnf::DocumentParser::new()
            .parse(&mut builder, iterator)
            .map_err(|error| {
                generated_failure(
                    error,
                    body.range.end,
                    "invalid or incomplete EBNF rule/expression",
                )
            })?;
        Ok(builder.document)
    })();
    result
        .map_err(|failure: super::error::ParseFailure| failure.into_document(body.document, budget))
}
