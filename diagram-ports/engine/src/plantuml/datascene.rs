//! MIT PlantUML TextBlockJson row/type/child identity; original native compartment geometry.
use super::{
    data::Value,
    scene::{Dash, Decoration, Element, Paint, Point, Rect, Scene, Shape},
    text::{Context, Label},
};
use crate::RenderError;

enum Child {
    Scalar,
    Container(usize),
}
struct Row {
    name: String,
    key: Label,
    value: Label,
    kind: &'static str,
    child: Child,
    height: f64,
}
struct Container {
    rows: Vec<Row>,
    key_width: f64,
    width: f64,
    height: f64,
    kind: &'static str,
    owner: String,
}

pub(super) fn layout(value: &Value, context: &mut Context<'_>) -> Result<Scene, RenderError> {
    let mut containers = Vec::new();
    collect(value, String::new(), &mut containers, context)?;
    let mut heights: Vec<_> = containers.iter().map(|c| c.height).collect();
    for id in (0..containers.len()).rev() {
        let mut children_height = 0.;
        for row in &containers[id].rows {
            context.budget.charge(1)?;
            if let Child::Container(child) = row.child {
                children_height += heights[child] + 24.;
            }
        }
        heights[id] = heights[id].max((children_height - 24.).max(0.));
    }
    let mut bounds = vec![
        Rect {
            x: 0.,
            y: 0.,
            width: 0.,
            height: 0.
        };
        containers.len()
    ];
    bounds[0].x = 24.;
    bounds[0].y = 24. + (heights[0] - containers[0].height) / 2.;
    for id in 0..containers.len() {
        context.budget.charge(containers[id].rows.len() + 1)?;
        bounds[id].width = containers[id].width;
        bounds[id].height = containers[id].height;
        let total = containers[id]
            .rows
            .iter()
            .filter_map(|row| {
                if let Child::Container(child) = row.child {
                    Some(heights[child] + 24.)
                } else {
                    None
                }
            })
            .sum::<f64>();
        let mut y = bounds[id].y + bounds[id].height / 2. - (total - 24.).max(0.) / 2.;
        for row in &containers[id].rows {
            if let Child::Container(child) = row.child {
                bounds[child].x = bounds[id].x + bounds[id].width + 80.;
                bounds[child].y = y + (heights[child] - containers[child].height) / 2.;
                y += heights[child] + 24.;
            }
        }
    }
    let mut scene = Scene::new();
    for (id, container) in containers.iter().enumerate() {
        let rect = bounds[id];
        let mut box_element = Element::new(
            format!("d{id}"),
            if container.kind == "object" {
                "data-object"
            } else if container.kind == "array" {
                "data-array"
            } else {
                "data-scalar"
            },
        );
        box_element.owner = container.owner.clone();
        box_element.shapes.push(Shape::Rect(rect, 0., Paint::Node));
        box_element.shapes.push(Shape::Line(
            vec![
                Point::new(rect.x + container.key_width, rect.y),
                Point::new(rect.x + container.key_width, rect.y + rect.height),
            ],
            Dash::Solid,
        ));
        scene.push(box_element, context.budget)?;
        let mut y = rect.y;
        for (index, row) in container.rows.iter().enumerate() {
            let mut element = Element::new(format!("d{id}r{index}"), row_kind(row.kind));
            element.name = row.name.clone();
            element.owner = format!("d{id}");
            if index > 0 {
                element.shapes.push(Shape::Line(
                    vec![Point::new(rect.x, y), Point::new(rect.x + rect.width, y)],
                    Dash::Solid,
                ));
            }
            element.shapes.push(Shape::Text(
                Point::new(rect.x + 8., y + (row.height - row.key.height) / 2.),
                row.key.clone(),
                Decoration::Plain,
            ));
            element.shapes.push(Shape::Text(
                Point::new(
                    rect.x + container.key_width + 8.,
                    y + (row.height - row.value.height) / 2.,
                ),
                row.value.clone(),
                Decoration::Plain,
            ));
            scene.push(element, context.budget)?;
            if let Child::Container(child) = row.child {
                let from = Point::new(rect.x + rect.width, y + row.height / 2.);
                let to = Point::new(
                    bounds[child].x,
                    bounds[child].y + containers[child].height / 2.,
                );
                let mid = from.x + 40.;
                let mut link = Element::new(format!("de{id}r{index}"), "data-link");
                link.owner = format!("d{id}r{index}");
                link.target = format!("d{child}");
                link.shapes.push(Shape::Line(
                    vec![from, Point::new(mid, from.y), Point::new(mid, to.y), to],
                    Dash::Solid,
                ));
                scene.push(link, context.budget)?;
            }
            y += row.height;
        }
    }
    Ok(scene)
}

fn collect(
    value: &Value,
    owner: String,
    containers: &mut Vec<Container>,
    context: &mut Context<'_>,
) -> Result<usize, RenderError> {
    context.budget.charge(1)?;
    let id = containers.len();
    containers.push(Container {
        rows: Vec::new(),
        key_width: 0.,
        width: 0.,
        height: 0.,
        kind: value.kind(),
        owner,
    });
    let mut rows = Vec::new();
    match value {
        Value::Object(members) => {
            for (key, value) in members {
                rows.push(make_row(
                    (key.clone(), value),
                    (id, rows.len()),
                    containers,
                    context,
                )?);
            }
        }
        Value::Array(values) => {
            for (index, value) in values.iter().enumerate() {
                rows.push(make_row(
                    (index.to_string(), value),
                    (id, rows.len()),
                    containers,
                    context,
                )?);
            }
        }
        _ => rows.push(make_row(("0".into(), value), (id, 0), containers, context)?),
    }
    if rows.is_empty() {
        rows.push(Row {
            name: String::new(),
            key: context.data_label("")?,
            value: context.data_label("")?,
            kind: value.kind(),
            child: Child::Scalar,
            height: 34.,
        });
    }
    let key_width = rows.iter().map(|r| r.key.width + 16.).fold(32., f64::max);
    let value_width = rows.iter().map(|r| r.value.width + 16.).fold(32., f64::max);
    let height = rows.iter().map(|r| r.height).sum();
    containers[id].rows = rows;
    containers[id].key_width = key_width;
    containers[id].width = key_width + value_width;
    containers[id].height = height;
    Ok(id)
}

fn make_row(
    pair: (String, &Value),
    identity: (usize, usize),
    containers: &mut Vec<Container>,
    context: &mut Context<'_>,
) -> Result<Row, RenderError> {
    context.budget.charge(pair.0.len() + 1)?;
    let key = context.data_label(&visible_text(&pair.0))?;
    let value = context.data_label(&visible_text(pair.1.display()))?;
    let child = if pair.1.is_container() {
        Child::Container(collect(
            pair.1,
            format!("d{}r{}", identity.0, identity.1),
            containers,
            context,
        )?)
    } else {
        Child::Scalar
    };
    let height = key.height.max(value.height) + 16.;
    Ok(Row {
        name: pair.0,
        key,
        value,
        kind: pair.1.kind(),
        child,
        height,
    })
}

fn visible_text(text: &str) -> String {
    text.chars()
        .map(|ch| {
            if ch.is_control() && !matches!(ch, '\n' | '\r' | '\t') {
                char::from_u32(0x2400 + ch as u32).unwrap_or('�')
            } else {
                ch
            }
        })
        .collect()
}

fn row_kind(kind: &str) -> &'static str {
    match kind {
        "object" => "data-row-object",
        "array" => "data-row-array",
        "number" => "data-row-number",
        "boolean" => "data-row-boolean",
        "null" => "data-row-null",
        _ => "data-row-string",
    }
}

#[cfg(test)]
mod tests {
    use super::super::{budget::Budget, data, source::Line};
    use super::*;
    use crate::RenderOptions;
    use merman_render::environment::RenderEnvironment;

    #[test]
    fn should_connect_actual_container_rows_without_compressing_scalar_placeholders() {
        // given
        let mut budget = Budget::new(&RenderOptions::default());
        let model = data::parse_json(
            &[Line {
                number: 2,
                text:
                    r#"{"first":{"x":"a long long label"},"scalar":1,"second":[2,3],"end":false}"#
                        .into(),
            }],
            &mut budget,
        )
        .unwrap();
        let session = RenderEnvironment::deterministic().begin_session().unwrap();
        // when
        let scene = layout(&model, &mut Context::new(&session, &mut budget)).unwrap();
        // then
        let container = scene.elements.iter().find(|e| e.id == "d0").unwrap();
        let Shape::Rect(root, _, _) = container.shapes[0] else {
            panic!()
        };
        let links: Vec<_> = scene
            .elements
            .iter()
            .filter(|e| e.kind == "data-link")
            .collect();
        assert_eq!(links.len(), 2);
        assert_eq!(links[0].owner, "d0r0");
        assert_eq!(links[1].owner, "d0r2");
        let Shape::Line(first, _) = &links[0].shapes[0] else {
            panic!()
        };
        let Shape::Line(second, _) = &links[1].shapes[0] else {
            panic!()
        };
        assert_eq!(first[0].x, root.x + root.width);
        assert_eq!(first[0].y, root.y + root.height / 8.);
        assert_eq!(second[0].y, root.y + root.height * 5. / 8.);
        for element in scene
            .elements
            .iter()
            .filter(|e| e.kind.starts_with("data-row-"))
        {
            let owner = scene
                .elements
                .iter()
                .find(|e| e.id == element.owner)
                .unwrap();
            let Shape::Rect(bounds, _, _) = owner.shapes[0] else {
                panic!()
            };
            for shape in &element.shapes {
                if let Shape::Text(p, label, _) = shape {
                    assert!(
                        p.x >= bounds.x
                            && p.y >= bounds.y
                            && p.x + label.width <= bounds.x + bounds.width + 1e-6
                            && p.y + label.height <= bounds.y + bounds.height + 1e-6
                    );
                }
            }
        }
        let boxes: Vec<_> = scene
            .elements
            .iter()
            .filter_map(|e| {
                if let Some(Shape::Rect(r, _, _)) = e.shapes.first() {
                    Some(*r)
                } else {
                    None
                }
            })
            .collect();
        for (index, a) in boxes.iter().enumerate() {
            for b in &boxes[index + 1..] {
                assert!(
                    a.x + a.width <= b.x
                        || b.x + b.width <= a.x
                        || a.y + a.height <= b.y
                        || b.y + b.height <= a.y
                );
            }
        }
    }
}
