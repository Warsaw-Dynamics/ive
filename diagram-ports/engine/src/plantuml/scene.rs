use super::{
    budget::{Budget, bound, invalid},
    text::Label,
};
use crate::RenderError;

#[derive(Clone, Copy, Debug)]
pub(super) struct Point {
    pub x: f64,
    pub y: f64,
}
impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Clone, Copy, Debug)]
pub(super) enum Paint {
    Paper,
    Node,
    Container,
    Ink,
    None,
}
impl Paint {
    pub fn value(self) -> &'static str {
        match self {
            Self::Paper => "#ffffff",
            Self::Node => "#eef2ff",
            Self::Container => "#f8fafc",
            Self::Ink => "#334155",
            Self::None => "none",
        }
    }
}

#[derive(Clone, Debug)]
pub(super) enum Shape {
    Rect(Rect, f64, Paint),
    Ellipse(Rect, Paint),
    Line(Vec<Point>, Dash),
    Polygon(Vec<Point>, Paint),
    Text(Point, Label, Decoration),
}
#[derive(Clone, Copy, Debug)]
pub(super) enum Dash {
    Solid,
    Dashed,
}
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub(super) enum Decoration {
    Plain,
    Underlined,
    Italic,
}

#[derive(Debug)]
pub(super) struct Element {
    pub id: String,
    pub kind: &'static str,
    pub name: String,
    pub owner: String,
    pub target: String,
    pub shapes: Vec<Shape>,
}
impl Element {
    pub fn new(id: impl Into<String>, kind: &'static str) -> Self {
        Self {
            id: id.into(),
            kind,
            name: String::new(),
            owner: String::new(),
            target: String::new(),
            shapes: Vec::new(),
        }
    }
}

pub(super) struct Scene {
    pub elements: Vec<Element>,
    pub width: f64,
    pub height: f64,
}
impl Scene {
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
            width: 1.,
            height: 1.,
        }
    }
    pub fn push(&mut self, element: Element, budget: &mut Budget) -> Result<(), RenderError> {
        budget.charge(element.shapes.len() + 1)?;
        bound(
            "scene elements",
            self.elements.len() + 1,
            budget.limits.model_items.saturating_mul(32),
        )?;
        for shape in &element.shapes {
            match shape {
                Shape::Rect(rect, radius, _)
                    if rect.width < 0.
                        || rect.height < 0.
                        || !radius.is_finite()
                        || *radius < 0. =>
                {
                    return Err(invalid(1, "invalid rectangle dimensions or radius"));
                }
                Shape::Ellipse(rect, _) if rect.width < 0. || rect.height < 0. => {
                    return Err(invalid(1, "negative ellipse dimensions"));
                }
                _ => {}
            }
            for point in shape_bounds(shape) {
                if !point.x.is_finite()
                    || !point.y.is_finite()
                    || point.x.abs() > 1_000_000.
                    || point.y.abs() > 1_000_000.
                {
                    return Err(invalid(1, "nonfinite or excessive scene geometry"));
                }
                self.width = self.width.max(point.x + 24.);
                self.height = self.height.max(point.y + 24.);
            }
        }
        self.elements.push(element);
        Ok(())
    }
}

fn shape_bounds(shape: &Shape) -> Vec<Point> {
    match shape {
        Shape::Rect(r, _, _) | Shape::Ellipse(r, _) => vec![
            Point::new(r.x, r.y),
            Point::new(r.x + r.width, r.y + r.height),
        ],
        Shape::Line(points, _) | Shape::Polygon(points, _) => points.clone(),
        Shape::Text(p, label, _) => vec![*p, Point::new(p.x + label.width, p.y + label.height)],
    }
}

pub(super) fn text_at(rect: Rect, label: Label) -> Shape {
    Shape::Text(
        Point::new(
            rect.x + (rect.width - label.width) / 2.,
            rect.y + (rect.height - label.height) / 2.,
        ),
        label,
        Decoration::Plain,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RenderOptions;

    #[test]
    fn should_reject_nonfinite_or_negative_shape_geometry_before_retention() {
        // given
        for (width, x) in [(f64::NAN, 0.), (20., f64::INFINITY), (-1., 0.)] {
            let mut budget = Budget::new(&RenderOptions::default());
            let mut scene = Scene::new();
            let mut element = Element::new("bad", "test");
            element.shapes.push(Shape::Rect(
                Rect {
                    x,
                    y: 0.,
                    width,
                    height: 20.,
                },
                0.,
                Paint::Paper,
            ));
            // when
            let result = scene.push(element, &mut budget);
            // then
            assert!(result.is_err(), "accepted width={width}, x={x}");
            assert!(scene.elements.is_empty());
        }
    }
}
