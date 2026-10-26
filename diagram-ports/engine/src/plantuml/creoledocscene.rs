//! Native measured paragraphs over immutable Creole document lines.
use super::{
    budget::bound,
    creoledoc::{Document, Paragraph},
    scene::{Decoration, Element, Point, Scene, Shape},
    text::{Context, Label},
};
use crate::RenderError;
use std::borrow::Cow;

pub(super) fn layout(
    document: &Document<'_>,
    context: &mut Context<'_>,
) -> Result<Scene, RenderError> {
    let mut scene = Scene::new();
    let mut y = 24.;
    let (labels, blank_height) = measure(document, context)?;
    for (index, (paragraph, label)) in document.paragraphs.iter().zip(labels).enumerate() {
        context.budget.charge(1)?;
        let (height, kind) = match paragraph {
            Paragraph::Text(_) => (label.height, "creole-paragraph"),
            Paragraph::Blank(_) => (blank_height, "creole-blank"),
        };
        let mut element = Element::new(format!("creole{index}"), kind);
        element
            .shapes
            .push(Shape::Text(Point::new(24., y), label, Decoration::Plain));
        scene.push(element, context.budget)?;
        y += height;
    }
    scene.width += 24.;
    scene.height = scene.height.max(y + 24.);
    bound(
        "Creole document height",
        scene.height.ceil() as usize,
        1_000_000,
    )?;
    Ok(scene)
}
fn measure(
    document: &Document<'_>,
    context: &mut Context<'_>,
) -> Result<(Vec<Label>, f64), RenderError> {
    let mut labels = Vec::new();
    let count = document.paragraphs.len();
    bound(
        "Creole measured paragraph storage",
        count.saturating_mul(std::mem::size_of::<Label>()),
        context.budget.limits.source_bytes.saturating_mul(256),
    )?;
    context.budget.charge(count)?;
    labels.try_reserve_exact(count).map_err(|error| {
        RenderError::Backend(format!(
            "Creole measured paragraph allocation for {count} labels failed: {error}"
        ))
    })?;
    let mut blank_height: f64 = 18.;
    for paragraph in &document.paragraphs {
        context.budget.item()?;
        let text = match paragraph {
            Paragraph::Text(lines) => Cow::Owned(join(document, lines.clone(), context)?),
            Paragraph::Blank(line) => {
                Cow::Borrowed(&document.source[document.lines[*line].span.clone()])
            }
        };
        let label = context.document_label(&text)?;
        for line in label.lines.iter() {
            context.budget.charge(1)?;
            blank_height = blank_height.max(line.height);
        }
        labels.push(label);
    }
    Ok((labels, blank_height))
}
fn join(
    document: &Document<'_>,
    lines: std::ops::Range<usize>,
    context: &mut Context<'_>,
) -> Result<String, RenderError> {
    let count = lines.len();
    let mut size = count.saturating_sub(1);
    for line in &document.lines[lines.clone()] {
        context.budget.charge(1)?;
        size = size.saturating_add(line.span.len());
    }
    bound("Creole paragraph bytes", size, 16 * 1024)?;
    context.budget.text(size)?;
    let mut text = String::new();
    text.try_reserve_exact(size).map_err(|error| {
        RenderError::Backend(format!(
            "Creole paragraph allocation for {size} bytes failed: {error}"
        ))
    })?;
    for (index, line) in document.lines[lines].iter().enumerate() {
        context.budget.charge(line.span.len() + 1)?;
        if index > 0 {
            text.push('\n');
        }
        text.push_str(&document.source[line.span.clone()]);
    }
    Ok(text)
}
