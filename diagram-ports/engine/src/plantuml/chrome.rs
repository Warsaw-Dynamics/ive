//! Measured composition following MIT DiagramChromeFactory's region ordering.
//! Copyright 2009-2024 Arnaud Roques; spacing is native, not upstream geometry.
use super::{
    scene::{Decoration, Element, Point, Scene, Shape},
    style::{Alignment, Preamble, Role},
    text::{Context, Label},
};
use crate::RenderError;

pub(super) fn compose(
    mut body: Scene,
    preamble: &Preamble,
    context: &mut Context<'_>,
) -> Result<Scene, RenderError> {
    body.background = preamble.background;
    if preamble.regions.is_empty() {
        return Ok(body);
    }
    let mut regions = Vec::new();
    for role in [Role::Header, Role::Title, Role::Legend, Role::Footer] {
        if let Some(region) = preamble.regions.iter().find(|region| region.role == role) {
            let label = context.region_label(&region.text, role)?;
            regions.push((role, region.align, label));
        }
    }
    let body_width = body.width - body.min_x;
    let body_height = body.height - body.min_y;
    let width = regions.iter().fold(body_width, |width, (_, _, label)| {
        width.max(label.width + 48.)
    });
    let top = regions
        .iter()
        .filter(|(role, _, _)| matches!(role, Role::Header | Role::Title))
        .map(|(_, _, label)| label.height + 12.)
        .sum::<f64>();
    let offset = Point::new((width - body_width) / 2. - body.min_x, top - body.min_y);
    let mut scene = Scene::new();
    scene.background = preamble.background;
    for mut element in body.elements {
        for shape in &mut element.shapes {
            translate(shape, offset, context)?;
        }
        scene.push(element, context.budget)?;
    }
    let mut above = 24.;
    let mut below = top + body_height + 12.;
    for (role, alignment, label) in regions {
        let y = if matches!(role, Role::Header | Role::Title) {
            let y = above;
            above += label.height + 12.;
            y
        } else {
            let y = below;
            below += label.height + 12.;
            y
        };
        add_region(&mut scene, (role, alignment, label, width, y), context)?;
    }
    scene.width = scene.width.max(width);
    scene.height = scene.height.max(top + body_height);
    Ok(scene)
}

fn add_region(
    scene: &mut Scene,
    region: (Role, Alignment, Label, f64, f64),
    context: &mut Context<'_>,
) -> Result<(), RenderError> {
    let (role, alignment, label, width, y) = region;
    context.budget.item()?;
    let x = match alignment {
        Alignment::Left => 24.,
        Alignment::Center => (width - label.width) / 2.,
        Alignment::Right => width - 24. - label.width,
    };
    let mut element = Element::new(format!("document-{}", role.name()), role.name());
    element
        .shapes
        .push(Shape::Text(Point::new(x, y), label, Decoration::Plain));
    scene.push(element, context.budget)
}

fn translate(
    shape: &mut Shape,
    offset: Point,
    context: &mut Context<'_>,
) -> Result<(), RenderError> {
    context.budget.charge(1)?;
    match shape {
        Shape::Rect(rect, _, _) | Shape::Ellipse(rect, _) => {
            rect.x += offset.x;
            rect.y += offset.y;
        }
        Shape::Text(point, _, _) => {
            point.x += offset.x;
            point.y += offset.y;
        }
        Shape::Line(points, _) | Shape::Polygon(points, _) => {
            for point in points {
                context.budget.charge(1)?;
                point.x += offset.x;
                point.y += offset.y;
            }
        }
    }
    Ok(())
}
