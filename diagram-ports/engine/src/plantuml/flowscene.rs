//! Sparse measured flow cells and exact cardinal side ports; no DAG relayout.
use super::{
    budget::{Budget, bound},
    flow::{Cardinal, FlowTiles, PathKind},
    scene::{Dash, Element, Paint, Point, Rect, Scene, Shape, text_at},
    style::Color,
    text::{Context, Label},
};
use crate::RenderError;
struct Measured {
    label: Label,
    width: f64,
    height: f64,
}
fn push<T>(values: &mut Vec<T>, value: T, budget: &mut Budget) -> Result<(), RenderError> {
    bound(
        "flow layout entries",
        values.len() + 1,
        budget.limits.model_items,
    )?;
    budget.charge(1)?;
    if values.len() == values.capacity() {
        let capacity = values.capacity().saturating_mul(2).max(4);
        bound(
            "flow layout capacity peak",
            capacity.saturating_add(values.capacity()),
            budget.limits.model_items.saturating_mul(3).max(4),
        )?;
        budget.charge(values.len())?;
        values
            .try_reserve_exact(capacity - values.len())
            .map_err(|error| {
                RenderError::Backend(format!("PlantUML flow layout allocation failed: {error}"))
            })?;
    }
    values.push(value);
    Ok(())
}
fn measure(
    model: &FlowTiles,
    context: &mut Context<'_>,
) -> Result<(Vec<Measured>, f64, f64), RenderError> {
    let mut values = Vec::new();
    let (mut width, mut height) = (0f64, 0f64);
    for tile in &model.tiles {
        context.budget.charge(1)?;
        let label = context.flow_label(&tile.label)?;
        let (w, h) = ((label.width + 20.).max(20.), (label.height + 20.).max(20.));
        width = width.max(w);
        height = height.max(h);
        push(
            &mut values,
            Measured {
                label,
                width: w,
                height: h,
            },
            context.budget,
        )?;
    }
    Ok((values, width, height))
}
fn rectangles(
    model: &FlowTiles,
    measured: (&[Measured], f64, f64),
    budget: &mut Budget,
) -> Result<Vec<Rect>, RenderError> {
    let (values, width, height) = measured;
    let (mut min_row, mut min_column) = (0, 0);
    for place in &model.placements {
        budget.charge(1)?;
        min_row = min_row.min(place.cell.row);
        min_column = min_column.min(place.cell.column);
    }
    let pitch_x = (width + 60.).max(200.);
    let pitch_y = (height + 40.).max(70.);
    let mut rectangles = Vec::new();
    for (place, metric) in model.placements.iter().zip(values) {
        budget.charge(1)?;
        let center = Point::new(
            40. + (place.cell.column - min_column) as f64 * pitch_x + width / 2.,
            40. + (place.cell.row - min_row) as f64 * pitch_y + height / 2.,
        );
        let rect = Rect {
            x: center.x - metric.width / 2.,
            y: center.y - metric.height / 2.,
            width: metric.width,
            height: metric.height,
        };
        for value in [
            rect.x,
            rect.y,
            rect.x + rect.width + 4.,
            rect.y + rect.height + 4.,
        ] {
            if !value.is_finite() || !(0. ..=1_000_000.).contains(&value) {
                return Err(super::budget::invalid(
                    1,
                    "nonfinite or excessive measured flow grid geometry",
                ));
            }
        }
        push(&mut rectangles, rect, budget)?;
    }
    Ok(rectangles)
}
fn port(rect: Rect, side: Cardinal) -> Point {
    match side {
        Cardinal::North => Point::new(rect.x + rect.width / 2., rect.y),
        Cardinal::South => Point::new(rect.x + rect.width / 2., rect.y + rect.height),
        Cardinal::East => Point::new(rect.x + rect.width, rect.y + rect.height / 2.),
        Cardinal::West => Point::new(rect.x, rect.y + rect.height / 2.),
    }
}
pub(super) fn layout(model: &FlowTiles, context: &mut Context<'_>) -> Result<Scene, RenderError> {
    let (metrics, width, height) = measure(model, context)?;
    let rectangles = rectangles(model, (&metrics, width, height), context.budget)?;
    let mut scene = Scene::new();
    context.budget.item()?;
    scene.push(Element::new("flow-root", "flow-document"), context.budget)?;
    for (id, tile) in model.tiles.iter().enumerate() {
        context.budget.item()?;
        context.budget.charge(5 + tile.id.len())?;
        let mut element = Element::new(format!("flow-tile-{id}"), "flow-tile");
        element.name = tile.id.clone();
        element.owner = "flow-root".into();
        element.target = format!(
            "bytes:{}..{};declared:{}",
            tile.span.start,
            tile.span.end,
            model.placements[id].declared.name()
        );
        let rect = rectangles[id];
        element
            .shapes
            .push(Shape::Rect(rect, 12.5, Paint::Color(Color(0xfefece))));
        element
            .shapes
            .push(text_at(rect, metrics[id].label.clone()));
        scene.push(element, context.budget)?;
    }
    for (id, path) in model.paths.iter().enumerate() {
        context.budget.item()?;
        context.budget.charge(12)?;
        let from = port(rectangles[path.from.tile], path.from.side);
        let to = port(rectangles[path.to.tile], path.to.side);
        let mut element = Element::new(format!("flow-path-{id}"), "flow-path");
        element.owner = format!("flow-tile-{}", path.from.tile);
        element.target = format!("flow-tile-{}", path.to.tile);
        let kind = match path.kind {
            PathKind::Implicit => "implicit",
            PathKind::Explicit => "explicit",
        };
        element.name = format!(
            "{}->{};{kind};bytes:{}..{}",
            path.from.side.name(),
            path.to.side.name(),
            path.span.start,
            path.span.end
        );
        element.shapes.push(Shape::ColoredLine(
            vec![from, to],
            Dash::Solid,
            Color(0xa80036),
        ));
        element.shapes.push(Shape::Ellipse(
            Rect {
                x: to.x - 3.,
                y: to.y - 3.,
                width: 7.,
                height: 7.,
            },
            Paint::Color(Color(0xa80036)),
        ));
        scene.push(element, context.budget)?;
    }
    Ok(scene)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        RenderOptions,
        plantuml::{flow, parser},
    };
    use merman_render::environment::RenderEnvironment;
    fn draw(body: &str) -> Scene {
        let mut budget = Budget::new(&RenderOptions::default());
        let model =
            parser::test_support::body((body, "flow"), &mut budget, parser::flow::parse).unwrap();
        let session = RenderEnvironment::deterministic().begin_session().unwrap();
        layout(
            &model,
            &mut Context::new(&session, &mut budget).with_fonts(flow::fonts()),
        )
        .unwrap()
    }
    fn rectangle(element: &Element) -> Rect {
        match element.shapes[0] {
            Shape::Rect(r, ..) => r,
            _ => panic!("flow tile rectangle"),
        }
    }
    #[test]
    fn should_keep_measured_long_labels_and_directional_boxes_disjoint_and_contained() {
        // given
        let scene = draw("A \"A very long measured tile label\"\nB \"B\"\ne C \"C\"\nn D \"D\"");
        // when
        let tiles = scene
            .elements
            .iter()
            .filter(|e| e.kind == "flow-tile")
            .collect::<Vec<_>>();
        let boxes = tiles.iter().map(|e| rectangle(e)).collect::<Vec<_>>();
        // then
        for i in 0..boxes.len() {
            for j in i + 1..boxes.len() {
                let (a, b) = (boxes[i], boxes[j]);
                assert!(
                    a.x + a.width < b.x
                        || b.x + b.width < a.x
                        || a.y + a.height < b.y
                        || b.y + b.height < a.y
                );
            }
        }
        for (tile, r) in tiles.iter().zip(boxes) {
            for shape in &tile.shapes {
                if let Shape::Text(p, label, _) = shape {
                    assert!(
                        p.x >= r.x
                            && p.y >= r.y
                            && p.x + label.width <= r.x + r.width
                            && p.y + label.height <= r.y + r.height
                    );
                    assert!(
                        p.x >= scene.min_x
                            && p.y >= scene.min_y
                            && p.x + label.width <= scene.width
                            && p.y + label.height <= scene.height
                    );
                }
            }
        }
    }
    #[test]
    fn should_connect_side_ports_and_original_offset_circle_without_crossing_tile_interiors() {
        // given
        let scene = draw("A \"A\"\ne B \"B\"\nw A\ns C \"C\"");
        // when
        let tiles = scene
            .elements
            .iter()
            .filter(|e| e.kind == "flow-tile")
            .collect::<Vec<_>>();
        let paths = scene
            .elements
            .iter()
            .filter(|e| e.kind == "flow-path")
            .collect::<Vec<_>>();
        // then
        for path in paths {
            let a = rectangle(tiles.iter().find(|n| n.id == path.owner).unwrap());
            let b = rectangle(tiles.iter().find(|n| n.id == path.target).unwrap());
            let Shape::ColoredLine(points, _, color) = &path.shapes[0] else {
                panic!("red connector")
            };
            assert_eq!(color.0, 0xa80036);
            assert_eq!(points.len(), 2);
            let (from, to) = (points[0], points[1]);
            let horizontal = from.y == to.y;
            assert!(horizontal || from.x == to.x);
            assert!(if horizontal {
                (from.x == a.x || from.x == a.x + a.width) && from.y == a.y + a.height / 2.
            } else {
                (from.y == a.y || from.y == a.y + a.height) && from.x == a.x + a.width / 2.
            });
            assert!(if horizontal {
                (to.x == b.x || to.x == b.x + b.width) && to.y == b.y + b.height / 2.
            } else {
                (to.y == b.y || to.y == b.y + b.height) && to.x == b.x + b.width / 2.
            });
            let Shape::Ellipse(marker, Paint::Color(color)) = path.shapes[1] else {
                panic!("destination circle")
            };
            assert_eq!(color.0, 0xa80036);
            assert_eq!(
                (marker.x, marker.y, marker.width, marker.height),
                (to.x - 3., to.y - 3., 7., 7.)
            );
            for tile in &tiles {
                let r = rectangle(tile);
                let crosses = if horizontal {
                    from.y > r.y
                        && from.y < r.y + r.height
                        && from.x.min(to.x) < r.x + r.width
                        && from.x.max(to.x) > r.x
                } else {
                    from.x > r.x
                        && from.x < r.x + r.width
                        && from.y.min(to.y) < r.y + r.height
                        && from.y.max(to.y) > r.y
                };
                assert!(!crosses);
            }
        }
    }
    #[test]
    fn should_preserve_literal_backslashes_and_style_spans_in_display_create_mode() {
        // given
        let scene = draw("A \"a\\b line\\nnext **bold**\"");
        // when
        let tile = scene
            .elements
            .iter()
            .find(|e| e.kind == "flow-tile")
            .unwrap();
        let label = tile
            .shapes
            .iter()
            .find_map(|s| {
                if let Shape::Text(_, l, _) = s {
                    Some(l)
                } else {
                    None
                }
            })
            .unwrap();
        // then
        assert_eq!(label.lines.len(), 1);
        let runs = &label.lines[0].runs;
        assert!(runs.iter().any(|r| r.run.text.contains("a\\b line\\nnext")));
        assert!(
            runs.iter()
                .any(|r| r.run.text == "bold" && r.run.style.font.weight == 700)
        );
        assert!(
            runs.iter()
                .all(|r| r.run.style.font.family.as_ref() == "serif")
        );
    }
}
