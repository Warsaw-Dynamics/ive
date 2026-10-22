//! PlantUML MIT: sequence CommandArrow and class CommandLinkClass; native decorations.
use super::{
    budget::unsupported,
    scene::{Dash, Paint, Point, Shape},
};
use crate::RenderError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum End {
    None,
    Open,
    Filled,
    Triangle,
    HollowDiamond,
    Diamond,
}
#[derive(Clone, Copy, Debug)]
pub(super) struct Arrow {
    pub start: End,
    pub end: End,
    pub dash: Dash,
}

pub(super) fn parse(value: &str, line: usize) -> Result<Arrow, RenderError> {
    let (start, rest) = if let Some(rest) = value.strip_prefix("<|") {
        (End::Triangle, rest)
    } else if let Some(rest) = value.strip_prefix("<<") {
        (End::Open, rest)
    } else if let Some(rest) = value.strip_prefix('<') {
        (End::Filled, rest)
    } else if let Some(rest) = value.strip_prefix('o') {
        (End::HollowDiamond, rest)
    } else if let Some(rest) = value.strip_prefix('*') {
        (End::Diamond, rest)
    } else {
        (End::None, value)
    };
    let (end, body) = if let Some(body) = rest.strip_suffix("|>") {
        (End::Triangle, body)
    } else if let Some(body) = rest.strip_suffix(">>") {
        (End::Open, body)
    } else if let Some(body) = rest.strip_suffix('>') {
        (End::Filled, body)
    } else if let Some(body) = rest.strip_suffix('o') {
        (End::HollowDiamond, body)
    } else if let Some(body) = rest.strip_suffix('*') {
        (End::Diamond, body)
    } else {
        (End::None, rest)
    };
    if !matches!(body, "-" | "--" | "---" | ".." | "...") {
        return Err(unsupported(line, value));
    }
    Ok(Arrow {
        start,
        end,
        dash: if body.starts_with('.') {
            Dash::Dashed
        } else {
            Dash::Solid
        },
    })
}

pub(super) fn draw(points: Vec<Point>, arrow: Arrow) -> Vec<Shape> {
    if points.len() < 2 {
        return Vec::new();
    }
    let mut shapes = vec![Shape::Line(points.clone(), arrow.dash)];
    shapes.extend(marker(points[0], points[1], arrow.start));
    shapes.extend(marker(
        points[points.len() - 1],
        points[points.len() - 2],
        arrow.end,
    ));
    shapes
}

fn marker(tip: Point, previous: Point, end: End) -> Vec<Shape> {
    let dx = previous.x - tip.x;
    let dy = previous.y - tip.y;
    let length = dx.hypot(dy).max(0.001);
    let u = Point::new(dx / length, dy / length);
    let at = |along: f64, side: f64| {
        Point::new(
            tip.x + u.x * along - u.y * side,
            tip.y + u.y * along + u.x * side,
        )
    };
    match end {
        End::None => Vec::new(),
        End::Open => vec![Shape::Line(
            vec![at(10., 5.), tip, at(10., -5.)],
            Dash::Solid,
        )],
        End::Filled => vec![Shape::Polygon(
            vec![tip, at(10., 5.), at(10., -5.)],
            Paint::Ink,
        )],
        End::Triangle => vec![Shape::Polygon(
            vec![tip, at(14., 7.), at(14., -7.)],
            Paint::Paper,
        )],
        End::HollowDiamond | End::Diamond => vec![Shape::Polygon(
            vec![tip, at(8., 5.), at(16., 0.), at(8., -5.)],
            if end == End::Diamond {
                Paint::Ink
            } else {
                Paint::Paper
            },
        )],
    }
}
