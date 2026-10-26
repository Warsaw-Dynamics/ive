//! Original native SVG geometry for MIT PlantUML symbol semantics; not dot/Smetana geometry.
use super::scene::{Dash, Paint, Point, Rect, Shape};

pub(super) fn node(rect: Rect, kind: &str) -> Vec<Shape> {
    let r = rect;
    match kind {
        "actor" => actor(r),
        "usecase" => vec![Shape::Ellipse(r, Paint::Node)],
        "database" => vec![
            Shape::Rect(r, 0., Paint::Node),
            Shape::Ellipse(Rect { height: 16., ..r }, Paint::Node),
            Shape::Ellipse(
                Rect {
                    y: r.y + r.height - 16.,
                    height: 16.,
                    ..r
                },
                Paint::Node,
            ),
        ],
        "node" => vec![
            Shape::Polygon(
                vec![
                    Point::new(r.x, r.y + 8.),
                    Point::new(r.x + 10., r.y),
                    Point::new(r.x + r.width, r.y),
                    Point::new(r.x + r.width, r.y + r.height - 8.),
                    Point::new(r.x + r.width - 10., r.y + r.height),
                    Point::new(r.x, r.y + r.height),
                ],
                Paint::Container,
            ),
            Shape::Line(
                vec![
                    Point::new(r.x, r.y + 8.),
                    Point::new(r.x + r.width - 10., r.y + 8.),
                    Point::new(r.x + r.width, r.y),
                ],
                Dash::Solid,
            ),
            Shape::Line(
                vec![
                    Point::new(r.x + r.width - 10., r.y + 8.),
                    Point::new(r.x + r.width - 10., r.y + r.height),
                ],
                Dash::Solid,
            ),
        ],
        "package" | "namespace" => vec![
            Shape::Rect(r, 0., Paint::Container),
            Shape::Rect(
                Rect {
                    width: r.width.min(90.),
                    height: 12.,
                    ..r
                },
                0.,
                Paint::Paper,
            ),
        ],
        "component" => {
            let mut shapes = vec![Shape::Rect(r, 0., Paint::Node)];
            for y in [r.y + 8., r.y + 22.] {
                shapes.push(Shape::Rect(
                    Rect {
                        x: r.x + 5.,
                        y,
                        width: 10.,
                        height: 8.,
                    },
                    0.,
                    Paint::Paper,
                ));
            }
            shapes
        }
        "note" => vec![Shape::Polygon(
            vec![
                Point::new(r.x, r.y),
                Point::new(r.x + r.width - 10., r.y),
                Point::new(r.x + r.width, r.y + 10.),
                Point::new(r.x + r.width, r.y + r.height),
                Point::new(r.x, r.y + r.height),
            ],
            Paint::Node,
        )],
        _ => vec![Shape::Rect(r, 0., Paint::Node)],
    }
}

fn actor(r: Rect) -> Vec<Shape> {
    let x = r.x + r.width / 2.;
    vec![
        Shape::Ellipse(
            Rect {
                x: x - 8.,
                y: r.y,
                width: 16.,
                height: 16.,
            },
            Paint::Paper,
        ),
        Shape::Line(
            vec![Point::new(x, r.y + 16.), Point::new(x, r.y + 38.)],
            Dash::Solid,
        ),
        Shape::Line(
            vec![
                Point::new(x - 16., r.y + 25.),
                Point::new(x + 16., r.y + 25.),
            ],
            Dash::Solid,
        ),
        Shape::Line(
            vec![
                Point::new(x - 14., r.y + 55.),
                Point::new(x, r.y + 38.),
                Point::new(x + 14., r.y + 55.),
            ],
            Dash::Solid,
        ),
    ]
}
