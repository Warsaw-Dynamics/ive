//! Original measured native Git lane geometry; no Smetana layout translation.
use super::{
    budget::{Budget, bound},
    git::{Comment, GitHistory, Position},
    scene::{Dash, Decoration, Element, Paint, Point, Rect, Scene, Shape},
    text::Context,
};
use crate::RenderError;

pub(super) fn layout(model: &GitHistory, context: &mut Context<'_>) -> Result<Scene, RenderError> {
    let mut labels = Vec::new();
    let mut pitch: f64 = 48.;
    let mut row_pitch: f64 = 36.;
    for commit in &model.commits {
        let text = caption(&commit.name, &commit.comment, context.budget)?;
        let label = context.data_label(&text)?;
        pitch = pitch.max(label.width + 48.);
        row_pitch = row_pitch.max(label.height + 24.);
        push(&mut labels, label, context.budget)?;
    }
    let origin = model.source_lanes.first().copied().unwrap_or(0);
    let geometry = Geometry {
        origin,
        pitch,
        row_pitch,
    };
    let mut scene = Scene::new();
    for (index, parent) in model.parents.iter().enumerate() {
        let mut points = Vec::new();
        for &position in &parent.path {
            push(&mut points, geometry.point(position)?, context.budget)?;
        }
        let end = points.len() - 1;
        points[0] = boundary(points[0], points[1]);
        points[end] = boundary(points[end], points[end - 1]);
        context.budget.item()?;
        let mut element = Element::new(format!("git-edge{index}"), "git-parent");
        element.owner = format!("git{}", parent.child);
        element.target = format!("git{}", parent.parent);
        element.shapes.push(Shape::Line(points, Dash::Solid));
        scene.push(element, context.budget)?;
    }
    for (index, commit) in model.commits.iter().enumerate() {
        context.budget.charge(1)?;
        context.budget.text(commit.name.len())?;
        let point = geometry.point(commit.position)?;
        let mut element = Element::new(format!("git{index}"), "git-commit");
        element.name = commit.name.clone();
        context.budget.item()?;
        element.shapes.push(Shape::Ellipse(
            Rect {
                x: point.x - 8.,
                y: point.y - 8.,
                width: 16.,
                height: 16.,
            },
            Paint::Node,
        ));
        context.budget.item()?;
        element.shapes.push(Shape::Text(
            Point::new(point.x + 16., point.y - labels[index].height / 2.),
            labels[index].clone(),
            Decoration::Plain,
        ));
        scene.push(element, context.budget)?;
    }
    Ok(scene)
}
struct Geometry {
    origin: usize,
    pitch: f64,
    row_pitch: f64,
}
impl Geometry {
    fn point(&self, position: Position) -> Result<Point, RenderError> {
        let point = Point::new(
            24. + (position.column as f64 - self.origin as f64) * self.pitch,
            24. + position.row as f64 * self.row_pitch,
        );
        if point.x.is_finite()
            && point.y.is_finite()
            && point.x.abs() <= 1_000_000.
            && point.y.abs() <= 1_000_000.
        {
            return Ok(point);
        }
        Err(RenderError::Backend(format!(
            "Git source lane {}, row {} exceeds finite coordinate bounds",
            position.column, position.row
        )))
    }
}
fn boundary(center: Point, toward: Point) -> Point {
    let (dx, dy) = (toward.x - center.x, toward.y - center.y);
    let length = dx.hypot(dy);
    Point::new(center.x + 8. * dx / length, center.y + 8. * dy / length)
}
fn caption(name: &str, comment: &Comment, budget: &mut Budget) -> Result<String, RenderError> {
    let bytes = name.len().saturating_add(match comment {
        Comment::Absent => 0,
        Comment::Text(text) => text.len().saturating_add(1),
    });
    bound("label bytes", bytes, 16 * 1024)?;
    budget.text(bytes)?;
    let mut result = String::new();
    result.try_reserve_exact(bytes).map_err(|error| {
        RenderError::Backend(format!(
            "Git caption allocation for {bytes} bytes failed: {error}"
        ))
    })?;
    result.push_str(name);
    if let Comment::Text(text) = comment {
        result.push('\n');
        result.push_str(text);
    }
    Ok(result)
}
fn push<T>(values: &mut Vec<T>, value: T, budget: &mut Budget) -> Result<(), RenderError> {
    budget.item()?;
    if values.len() == values.capacity() {
        let capacity = values.capacity().saturating_mul(2).max(4);
        bound(
            "Git layout retained buffer bytes",
            capacity
                .saturating_add(values.capacity())
                .saturating_mul(std::mem::size_of::<T>()),
            budget.limits.source_bytes.saturating_mul(256),
        )?;
        budget.charge(values.len())?;
        values
            .try_reserve_exact(capacity - values.len())
            .map_err(|error| {
                RenderError::Backend(format!(
                    "Git layout allocation for capacity {capacity} failed: {error}"
                ))
            })?;
    }
    values.push(value);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RenderOptions, plantuml::parser::test_support};
    use merman_render::environment::RenderEnvironment;

    #[test]
    fn should_place_newer_commits_above_older_parents_with_measured_nonoverlapping_labels() {
        // given
        let source = "@startgit\n* merge4 (release checkout)\n|\\\n| * side3 (a long measured branch caption)\n* | main2\n|/\n* base1\n@endgit";
        let mut budget = Budget::new(&RenderOptions::default());
        let model =
            test_support::parse(source, &mut budget, crate::plantuml::parser::git::parse).unwrap();
        let session = RenderEnvironment::deterministic()
            .without_math_renderer()
            .begin_session()
            .unwrap();
        // when
        let scene = layout(&model, &mut Context::new(&session, &mut budget)).unwrap();
        // then
        let nodes: Vec<_> = scene
            .elements
            .iter()
            .filter(|element| element.kind == "git-commit")
            .collect();
        assert_eq!(nodes.len(), model.commits.len());
        assert_eq!(
            scene
                .elements
                .iter()
                .filter(|element| element.kind == "git-parent")
                .count(),
            model.parents.len()
        );
        let circles: Vec<_> = nodes
            .iter()
            .map(|element| match element.shapes[0] {
                Shape::Ellipse(rect, _) => rect,
                _ => panic!("commit is not visible"),
            })
            .collect();
        for edge in &model.parents {
            assert!(circles[edge.child].y < circles[edge.parent].y);
        }
        let labels: Vec<_> = nodes
            .iter()
            .map(|element| match &element.shapes[1] {
                Shape::Text(point, label, _) => Rect {
                    x: point.x,
                    y: point.y,
                    width: label.width,
                    height: label.height,
                },
                _ => panic!("commit label missing"),
            })
            .collect();
        for (index, label) in labels.iter().enumerate() {
            assert!(label.x >= scene.min_x && label.y >= scene.min_y);
            assert!(label.x + label.width <= scene.width && label.y + label.height <= scene.height);
            for other in labels.iter().skip(index + 1) {
                assert!(
                    label.x + label.width <= other.x
                        || other.x + other.width <= label.x
                        || label.y + label.height <= other.y
                        || other.y + other.height <= label.y
                );
            }
        }
    }

    #[test]
    fn should_cancel_inside_git_layout_measurement_without_serializing_a_partial_scene() {
        // given
        let mut budget = Budget::new(&RenderOptions::default());
        let model = test_support::body(
            ("* top\n|\n* base", "git"),
            &mut budget,
            crate::plantuml::parser::git::parse,
        )
        .unwrap();
        let session = RenderEnvironment::deterministic()
            .without_math_renderer()
            .begin_session()
            .unwrap();
        budget.control.cancel_after_checkpoints(4);
        // when
        let result = layout(&model, &mut Context::new(&session, &mut budget));
        // then
        assert!(matches!(result, Err(RenderError::Cancelled(_))));
    }
}
