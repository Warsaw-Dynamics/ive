//! Original native proportional packet rows/ticks; full labels set one measured common bit scale.
use super::{
    packet::{Direction, Packet},
    scene::{Dash, Decoration, Element, Paint, Point, Rect, Scene, Shape, text_at},
    text::Context,
};
use crate::RenderError;

pub(super) fn layout(model: &Packet, context: &mut Context<'_>) -> Result<Scene, RenderError> {
    let mut labels = Vec::new();
    for field in &model.fields {
        labels.push(context.tree_label(&field.label)?);
    }
    let mut bit_width: f64 = 24.;
    for fragment in &model.fragments {
        context.budget.charge(1)?;
        bit_width = bit_width.max((labels[fragment.field].width + 20.) / fragment.width as f64);
    }
    let mut ticks = Vec::new();
    let mut tick_height: f64 = 0.;
    for index in 0..=model.config.columns {
        context.budget.item()?;
        let value = if model.config.direction == Direction::Rtl {
            model.config.columns - index
        } else {
            index
        };
        let label = if value % model.config.interval == 0 {
            context.data_label(&value.to_string())?
        } else {
            context.data_label("")?
        };
        bit_width = bit_width.max(label.width + 4.);
        tick_height = tick_height.max(label.height);
        ticks.push((value, label));
    }
    context.budget.charge(model.rows)?;
    let mut heights = vec![0_f64; model.rows];
    for f in &model.fragments {
        context.budget.charge(1)?;
        heights[f.row] = heights[f.row].max(
            (model.fields[f.field].height as f64 * model.config.node_height as f64)
                .max(labels[f.field].height + 20.),
        );
    }
    let mut ys = Vec::new();
    let mut y = 24. + tick_height + 16.;
    for &height in &heights {
        context.budget.charge(1)?;
        ys.push(y);
        y += height + 16.;
    }
    let x = 24. + ticks.iter().map(|(_, l)| l.width / 2.).fold(0., f64::max);
    let mut scene = Scene::new();
    for (id, field) in model.fields.iter().enumerate() {
        context.budget.text(field.label.len() + 32)?;
        let mut element = Element::new(format!("p{id}"), "packet-field");
        element.name = format!("{}-{}:{}", field.start, field.end, field.label);
        scene.push(element, context.budget)?;
    }
    for (id, f) in model.fragments.iter().enumerate() {
        let offset = if model.config.direction == Direction::Rtl {
            model.config.columns - f.offset - f.width
        } else {
            f.offset
        };
        let field = &model.fields[f.field];
        let height = if model.config.same_height {
            heights[f.row]
        } else {
            (field.height as f64 * model.config.node_height as f64)
                .max(labels[f.field].height + 20.)
        };
        let rect = Rect {
            x: x + offset as f64 * bit_width,
            y: ys[f.row],
            width: f.width as f64 * bit_width,
            height,
        };
        let mut element = Element::new(format!("pf{id}"), "packet-fragment");
        element.owner = format!("p{}", f.field);
        element.name = format!("{}:{}:{}", f.row, f.offset, f.width);
        element.shapes.push(Shape::Rect(rect, 0., Paint::Node));
        element.shapes.push(text_at(rect, labels[f.field].clone()));
        scene.push(element, context.budget)?;
    }
    for (index, (value, label)) in ticks.into_iter().enumerate() {
        let mut element = Element::new(format!("pt{index}"), "packet-tick");
        element.name = value.to_string();
        let tx = x + index as f64 * bit_width;
        element.shapes.push(Shape::Text(
            Point::new(tx - label.width / 2., 24.),
            label,
            Decoration::Plain,
        ));
        element.shapes.push(Shape::Line(
            vec![
                Point::new(
                    tx,
                    24. + tick_height
                        + if value % model.config.interval == 0 {
                            0.
                        } else {
                            8.
                        },
                ),
                Point::new(tx, ys[0]),
            ],
            Dash::Solid,
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
    fn should_conserve_fragment_widths_and_retain_inclusive_field_identity() {
        // given
        let lines = "colwidth=8\n0-2: P\n3-22: Data".to_owned();
        let mut budget = Budget::new(&RenderOptions::default());
        // when
        let model = crate::plantuml::parser::test_support::body(
            (&lines, "packetdiag"),
            &mut budget,
            crate::plantuml::parser::packet::parse,
        )
        .unwrap();
        // then
        assert_eq!(model.fields[1].start, 3);
        assert_eq!(model.fields[1].end, 22);
        assert_eq!(
            model
                .fragments
                .iter()
                .map(|f| (f.field, f.row, f.offset, f.width))
                .collect::<Vec<_>>(),
            [(0, 0, 0, 3), (1, 0, 3, 5), (1, 1, 0, 8), (1, 2, 0, 7)]
        );
    }

    #[test]
    fn should_draw_proportional_cells_ticks_rtl_and_height_clearance() {
        // given
        for direction in ["ltr", "rtl"] {
            let input = format!(
                "colwidth=12\nscale_interval=4\nscale_direction={direction}\nnode_height=50\nsame_height=true\n0-3: Four\n4-11: Eight [height=2]\n12-15: Next"
            );
            let lines = input.to_owned();
            let mut budget = Budget::new(&RenderOptions::default());
            let model = crate::plantuml::parser::test_support::body(
                (&lines, "packetdiag"),
                &mut budget,
                crate::plantuml::parser::packet::parse,
            )
            .unwrap();
            let session = RenderEnvironment::deterministic().begin_session().unwrap();
            // when
            let scene = layout(&model, &mut Context::new(&session, &mut budget)).unwrap();
            let boxes: Vec<_> = scene
                .elements
                .iter()
                .filter(|e| e.kind == "packet-fragment")
                .map(|e| {
                    if let Shape::Rect(r, _, _) = e.shapes[0] {
                        r
                    } else {
                        panic!()
                    }
                })
                .collect();
            let ticks: Vec<_> = scene
                .elements
                .iter()
                .filter(|e| e.kind == "packet-tick")
                .collect();
            // then
            assert_eq!(boxes[1].width, boxes[0].width * 2.);
            assert_eq!(boxes[0].height, 100.);
            assert_eq!(boxes[1].height, 100.);
            assert!(boxes[2].y > boxes[0].y + boxes[0].height);
            if direction == "rtl" {
                assert!(boxes[0].x > boxes[1].x);
                assert_eq!(ticks[0].name, "12");
            } else {
                assert_eq!(boxes[1].x, boxes[0].x + boxes[0].width);
                assert_eq!(ticks[0].name, "0");
            }
            let Shape::Line(ref line, _) = ticks[4].shapes[1] else {
                panic!()
            };
            let left = boxes[0].x.min(boxes[1].x);
            assert_eq!(line[0].x, left + boxes[0].width);
            assert_eq!(
                ticks
                    .iter()
                    .filter(|e| if let Shape::Text(_, ref l, _) = e.shapes[0] {
                        l.width > 0.
                    } else {
                        false
                    })
                    .count(),
                4
            );
        }
    }

    #[test]
    fn should_reserve_variable_row_maximum_when_the_last_cell_is_shorter() {
        // given
        let lines="colwidth=8\nnode_height=50\nsame_height=false\n0-3: Tall [height=3]\n4-7: Short\n8-11: Next".to_owned();
        let mut budget = Budget::new(&RenderOptions::default());
        let model = crate::plantuml::parser::test_support::body(
            (&lines, "packetdiag"),
            &mut budget,
            crate::plantuml::parser::packet::parse,
        )
        .unwrap();
        let session = RenderEnvironment::deterministic().begin_session().unwrap();
        // when
        let scene = layout(&model, &mut Context::new(&session, &mut budget)).unwrap();
        let boxes: Vec<_> = scene
            .elements
            .iter()
            .filter(|e| e.kind == "packet-fragment")
            .map(|e| {
                if let Shape::Rect(r, _, _) = e.shapes[0] {
                    r
                } else {
                    panic!()
                }
            })
            .collect();
        // then
        assert_eq!(boxes[0].height, 150.);
        assert_eq!(boxes[1].height, 50.);
        assert!(boxes[2].y > boxes[0].y + 150.);
    }
}
