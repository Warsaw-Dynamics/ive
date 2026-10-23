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
        regex::{Builder, RegexDocument},
    },
};
pub(in crate::plantuml) fn parse(
    body: BodySource<'_>,
    budget: &mut Budget,
) -> Result<RegexDocument, RenderError> {
    let result = (|| {
        let tokens = railroad::lex(&body, budget, Family::Regex)?;
        let mut storage =
            railroad::admit((&tokens, tokens.capacity()), &body, budget, Family::Regex)?;
        let mut exprs = Vec::new();
        storage.reserve(
            &mut exprs,
            tokens
                .len()
                .saturating_mul(2)
                .min(budget.limits.model_items),
            budget,
        )?;
        let iterator = Checked::new(tokens, budget.control.clone());
        let mut builder = Builder {
            storage,
            document: RegexDocument { root: 0, exprs },
            budget,
        };
        let root = super::generated_regex::DocumentParser::new()
            .parse(&mut builder, iterator)
            .map_err(|error| {
                generated_failure(
                    error,
                    body.range.end,
                    "invalid or incomplete regex expression, class or quantifier",
                )
            })?;
        builder.document.root = root;
        Ok(builder.document)
    })();
    result
        .map_err(|failure: super::error::ParseFailure| failure.into_document(body.document, budget))
}
