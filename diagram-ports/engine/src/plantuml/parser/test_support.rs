use super::BodySource;
use crate::{
    RenderError,
    plantuml::{
        budget::Budget,
        origin::{Document, Origin, Segment, SourceSpan},
        style::Preamble,
    },
};

pub(in crate::plantuml) fn parse<T>(
    source: &str,
    budget: &mut Budget,
    parse: impl FnOnce(BodySource<'_>, &mut Budget) -> Result<T, RenderError>,
) -> Result<T, RenderError> {
    styled(source, budget, parse).map(|(_, model)| model)
}

pub(in crate::plantuml) fn body<T>(
    input: (&str, &str),
    budget: &mut Budget,
    parse: impl FnOnce(BodySource<'_>, &mut Budget) -> Result<T, RenderError>,
) -> Result<T, RenderError> {
    let (body, wrapper) = input;
    self::parse(
        &format!("@start{wrapper}\n{body}\n@end{wrapper}"),
        budget,
        parse,
    )
}

pub(in crate::plantuml) fn styled<T>(
    source: &str,
    budget: &mut Budget,
    parse: impl FnOnce(BodySource<'_>, &mut Budget) -> Result<T, RenderError>,
) -> Result<(Preamble, T), RenderError> {
    let document = crate::plantuml::preprocess::prepare(source, budget)?;
    let mut prepared = super::document::parse(&document, budget)?;
    let (preamble, _) = super::preamble::parse(&mut prepared, budget)?;
    parse(prepared.body, budget).map(|model| (preamble, model))
}

pub(in crate::plantuml) fn literal<T>(
    source: &str,
    budget: &mut Budget,
    parse: impl FnOnce(BodySource<'_>, &mut Budget) -> Result<T, RenderError>,
) -> Result<T, RenderError> {
    let document = document(source, budget)?;
    let mut prepared = super::document::parse(&document, budget)?;
    super::preamble::parse(&mut prepared, budget)?;
    parse(prepared.body, budget)
}

pub(in crate::plantuml) fn document(
    source: &str,
    budget: &mut Budget,
) -> Result<Document, RenderError> {
    budget.text(source.len())?;
    Ok(Document {
        text: source.to_owned(),
        segments: vec![Segment {
            output: 0..source.len(),
            origin: Origin::Direct(SourceSpan {
                start_byte: 0,
                end_byte: source.len(),
                line: 1,
                column: 1,
            }),
        }],
        definitions: Vec::new(),
    })
}
