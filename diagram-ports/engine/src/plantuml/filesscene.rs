//! Original native files symbols and measured preorder rows; no external icon assets.
use super::{
    files::{Files, Kind},
    scene::{Decoration, Element, Paint, Point, Rect, Scene, Shape},
    text::Context,
};
use crate::RenderError;

pub(super) fn layout(model: &Files, context: &mut Context<'_>) -> Result<Scene, RenderError> {
    let mut scene = Scene::new();
    context.budget.charge(model.entries.len())?;
    let mut pending: Vec<_> = model.entries[0].children.iter().rev().copied().collect();
    let mut y = 24.;
    while let Some(id) = pending.pop() {
        context.budget.charge(1)?;
        let entry = &model.entries[id];
        let label = if entry.kind == Kind::Note {
            context.tree_label(&entry.name)?
        } else {
            context.data_label(&entry.name)?
        };
        let x = 24. + (entry.depth - 1) as f64 * 21.;
        let note = entry.kind == Kind::Note;
        let height = if note {
            label.height + 20.
        } else {
            label.height.max(20.)
        };
        let mut element = Element::new(
            format!("f{id}"),
            match entry.kind {
                Kind::Folder => "files-folder",
                Kind::File => "files-file",
                Kind::Note => "files-note",
                Kind::Root => unreachable!(),
            },
        );
        context.budget.text(entry.name.len())?;
        element.name = entry.name.clone();
        element.owner = format!("f{}", entry.parent);
        if note {
            let width = (label.width + 26.).max(40.);
            element.shapes.push(Shape::Polygon(
                vec![
                    Point::new(x, y),
                    Point::new(x + width - 8., y),
                    Point::new(x + width, y + 8.),
                    Point::new(x + width, y + height),
                    Point::new(x, y + height),
                ],
                Paint::Node,
            ));
            element.shapes.push(Shape::Line(
                vec![
                    Point::new(x + width - 8., y),
                    Point::new(x + width - 8., y + 8.),
                    Point::new(x + width, y + 8.),
                ],
                super::scene::Dash::Solid,
            ));
            element.shapes.push(Shape::Text(
                Point::new(x + 10., y + 10.),
                label,
                Decoration::Plain,
            ));
        } else {
            element
                .shapes
                .extend(symbol(&entry.kind, Point::new(x, y + (height - 16.) / 2.)));
            element.shapes.push(Shape::Text(
                Point::new(x + 24., y + (height - label.height) / 2.),
                label,
                Decoration::Plain,
            ));
        }
        scene.push(element, context.budget)?;
        y += height + 2.;
        context.budget.charge(entry.children.len())?;
        pending.extend(entry.children.iter().rev().copied());
    }
    Ok(scene)
}

fn symbol(kind: &Kind, p: Point) -> Vec<Shape> {
    if *kind == Kind::Folder {
        return vec![Shape::Polygon(
            vec![
                p,
                Point::new(p.x + 7., p.y),
                Point::new(p.x + 10., p.y + 4.),
                Point::new(p.x + 19., p.y + 4.),
                Point::new(p.x + 19., p.y + 16.),
                Point::new(p.x, p.y + 16.),
            ],
            Paint::Container,
        )];
    }
    vec![
        Shape::Rect(
            Rect {
                x: p.x + 2.,
                y: p.y,
                width: 13.,
                height: 16.,
            },
            0.,
            Paint::Paper,
        ),
        Shape::Line(
            vec![
                Point::new(p.x + 5., p.y + 6.),
                Point::new(p.x + 12., p.y + 6.),
            ],
            super::scene::Dash::Solid,
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::super::{budget::Budget, source::Line};
    use super::*;
    use crate::RenderOptions;
    use merman_render::environment::RenderEnvironment;

    #[test]
    fn should_clear_complete_note_height_and_indent_preorder_rows() {
        // given
        let lines: Vec<_> =
            "/app/src/main.rs\n<note>\nfirst\nsecond\nthird\n</note>\n/app/src/next.rs"
                .lines()
                .enumerate()
                .map(|(i, s)| Line {
                    number: i + 1,
                    text: s.into(),
                })
                .collect();
        let mut budget = Budget::new(&RenderOptions::default());
        let model = super::super::files::parse(&lines, &mut budget).unwrap();
        let session = RenderEnvironment::deterministic().begin_session().unwrap();
        let mut context = Context::new(&session, &mut budget);
        // when
        let scene = layout(&model, &mut context).unwrap();
        let labels: Vec<_> = scene
            .elements
            .iter()
            .map(|e| {
                e.shapes
                    .iter()
                    .find_map(|s| {
                        if let Shape::Text(p, l, _) = s {
                            Some((*p, l))
                        } else {
                            None
                        }
                    })
                    .unwrap()
            })
            .collect();
        // then
        assert_eq!(
            scene
                .elements
                .iter()
                .map(|e| e.id.as_str())
                .collect::<Vec<_>>(),
            ["f1", "f2", "f3", "f4", "f5"]
        );
        assert!(labels[1].0.x > labels[0].0.x);
        assert!(labels[2].0.x > labels[1].0.x);
        assert!(labels[4].0.y > labels[3].0.y + labels[3].1.height + 10.);
        for (p, l) in labels {
            assert!(
                p.x >= scene.min_x
                    && p.y >= scene.min_y
                    && p.x + l.width <= scene.width
                    && p.y + l.height <= scene.height
            );
        }
    }
}
