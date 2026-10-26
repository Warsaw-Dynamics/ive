//! Original native measured board geometry; one paint per card and no inferred arrows.
use super::{
    board::{Board, Parent},
    scene::{Dash, Element, Paint, Point, Rect, Scene, Shape, text_at},
    text::Context,
};
use crate::RenderError;

pub(super) fn layout(model: &Board, context: &mut Context<'_>) -> Result<Scene, RenderError> {
    let mut labels = Vec::new();
    let mut width: f64 = 150.;
    let mut height: f64 = 70.;
    for card in &model.cards {
        let label = context.tree_label(&card.name)?;
        width = width.max(label.width + 24.);
        height = height.max(label.height + 20.);
        labels.push(label);
    }
    context.budget.charge(model.cards.len())?;
    let mut columns = vec![0; model.cards.len()];
    let mut column = 0;
    let mut stage = 0;
    for &root in &model.activities {
        let mut stack = vec![(root, 0)];
        while let Some((id, sibling)) = stack.pop() {
            context.budget.charge(1)?;
            if sibling > 0 {
                column += 1;
            }
            columns[id] = column;
            stage = stage.max(model.cards[id].stage);
            context.budget.charge(model.cards[id].children.len())?;
            stack.extend(
                model.cards[id]
                    .children
                    .iter()
                    .enumerate()
                    .rev()
                    .map(|(i, &child)| (child, i)),
            );
        }
        column += 1;
    }
    let mut scene = Scene::new();
    for (id, card) in model.cards.iter().enumerate() {
        context.budget.text(card.name.len())?;
        let mut element = Element::new(format!("b{id}"), "board-card");
        element.name = card.name.clone();
        if let Parent::Node(parent) = card.parent {
            element.owner = format!("b{parent}");
        }
        let rect = Rect {
            x: 24. + columns[id] as f64 * (width + 20.),
            y: 24. + card.stage as f64 * (height + 20.),
            width,
            height,
        };
        element.shapes.push(Shape::Rect(rect, 4., Paint::Node));
        element.shapes.push(text_at(rect, labels[id].clone()));
        scene.push(element, context.budget)?;
    }
    for row in 1..=stage {
        context.budget.item()?;
        let mut element = Element::new(format!("bs{row}"), "board-stage");
        element.name = row.to_string();
        let y = 24. + row as f64 * (height + 20.) - 10.;
        element.shapes.push(Shape::Line(
            vec![
                Point::new(24., y),
                Point::new(24. + column as f64 * (width + 20.) - 20., y),
            ],
            Dash::Dashed,
        ));
        scene.push(element, context.budget)?;
    }
    Ok(scene)
}

#[cfg(test)]
mod tests {
    use super::super::budget::Budget;
    use super::*;
    use crate::RenderOptions;
    use merman_render::environment::RenderEnvironment;

    #[test]
    fn should_project_nearest_lower_ancestors_to_dfs_columns_and_empty_stages() {
        // given
        let lines =
            "Checkout\n+ Draft\n++ Validate\n+ Submit\n+++ Ship\nAudit\n+ Record".to_owned();
        let mut budget = Budget::new(&RenderOptions::default());
        let model = crate::plantuml::parser::test_support::body(
            (&lines, "board"),
            &mut budget,
            crate::plantuml::parser::board::parse,
        )
        .unwrap();
        let session = RenderEnvironment::deterministic().begin_session().unwrap();
        let mut context = Context::new(&session, &mut budget);
        // when
        let scene = layout(&model, &mut context).unwrap();
        let boxes: Vec<_> = scene
            .elements
            .iter()
            .filter_map(|e| e.shapes.first())
            .filter_map(|s| {
                if let Shape::Rect(r, _, _) = s {
                    Some(*r)
                } else {
                    None
                }
            })
            .collect();
        // then
        assert_eq!(
            model.cards.iter().map(|c| c.stage).collect::<Vec<_>>(),
            [0, 1, 2, 1, 3, 0, 1]
        );
        assert_eq!(model.cards[3].parent, Parent::Node(0));
        assert_eq!(model.cards[4].parent, Parent::Node(3));
        assert_eq!(boxes.len(), 7);
        assert_eq!(
            boxes.iter().map(|r| r.x).collect::<Vec<_>>(),
            [24., 24., 24., 194., 194., 364., 364.]
        );
        assert_eq!(
            boxes.iter().map(|r| r.y).collect::<Vec<_>>(),
            [24., 114., 204., 114., 294., 24., 114.]
        );
        assert_eq!(
            scene
                .elements
                .iter()
                .filter(|e| e.kind == "board-stage")
                .count(),
            3
        );
    }

    #[test]
    fn should_contain_long_board_labels_with_measured_card_pitches() {
        // given
        let lines = [
            "A long card title with significant measured width".to_owned(),
            "+ Child".to_owned(),
        ]
        .join("\n");
        let mut budget = Budget::new(&RenderOptions::default());
        let model = crate::plantuml::parser::test_support::body(
            (&lines, "board"),
            &mut budget,
            crate::plantuml::parser::board::parse,
        )
        .unwrap();
        let session = RenderEnvironment::deterministic().begin_session().unwrap();
        let mut context = Context::new(&session, &mut budget);
        // when
        let scene = layout(&model, &mut context).unwrap();
        // then
        for element in scene.elements.iter().filter(|e| e.kind == "board-card") {
            let Shape::Rect(r, _, _) = element.shapes[0] else {
                panic!()
            };
            let Shape::Text(p, ref l, _) = element.shapes[1] else {
                panic!()
            };
            assert!(
                p.x > r.x
                    && p.y > r.y
                    && p.x + l.width < r.x + r.width
                    && p.y + l.height < r.y + r.height
            );
        }
    }
}
