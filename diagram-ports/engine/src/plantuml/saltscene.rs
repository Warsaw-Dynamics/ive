//! Measured Salt tables and concrete controls; PlantUML MIT ElementPyramid and widgets.
//! Copyright 2009-2024 Arnaud Roques; exact source traces/notices in provenance manifest.
use super::{
    budget::{Budget, bound, invalid},
    salt::{Border, Kind, Leaf, LeafKind, Salt, Table},
    scene::{Dash, Decoration, Element, Paint, Point, Rect, Scene, Shape},
    text::{Context, Label},
};
use crate::RenderError;

const GAP: f64 = 6.;
struct Size {
    width: f64,
    height: f64,
}
enum Content {
    Leaf(Label),
    Table { x: Vec<f64>, y: Vec<f64> },
}
struct Measured {
    size: Size,
    content: Content,
}

pub(super) fn layout(model: &Salt, context: &mut Context<'_>) -> Result<Scene, RenderError> {
    let measured = measure(model, context)?;
    let mut scene = Scene::new();
    Painter {
        model,
        measured: &measured,
        scene: &mut scene,
        budget: context.budget,
    }
    .widget(model.root, Point::new(24., 24.), "")?;
    Ok(scene)
}

fn measure(model: &Salt, context: &mut Context<'_>) -> Result<Vec<Measured>, RenderError> {
    let mut measured = Vec::new();
    for widget in &model.widgets {
        context.budget.charge(1)?;
        measured.push(match &widget.kind {
            Kind::Leaf(leaf) => measure_leaf(leaf, context)?,
            Kind::Table(table) => measure_table(table, &measured, context.budget)?,
        });
    }
    Ok(measured)
}

fn measure_leaf(leaf: &Leaf, context: &mut Context<'_>) -> Result<Measured, RenderError> {
    let padded = context.label(&leaf.raw)?;
    let label = if leaf.raw.trim() == leaf.raw {
        padded.clone()
    } else {
        context.label(leaf.raw.trim())?
    };
    let managed = matches!(leaf.kind, LeafKind::Field | LeafKind::Button);
    let mut characters = 0;
    if managed {
        for line in padded.lines.iter() {
            let mut count = 0;
            for run in &line.runs {
                context.budget.charge(run.run.text.len() + 1)?;
                count += run.run.text.encode_utf16().count();
            }
            characters = characters.max(count);
        }
    }
    let width = label.width.max(characters as f64 * 8.);
    let size = match leaf.kind {
        LeafKind::Field => Size {
            width: width + 6.,
            height: label.height + 6.,
        },
        LeafKind::Button => Size {
            width: width + 12.,
            height: label.height + 10.,
        },
        LeafKind::CheckboxOn | LeafKind::CheckboxOff | LeafKind::RadioOn | LeafKind::RadioOff => {
            Size {
                width: width + 20.,
                height: label.height.max(12.),
            }
        }
        LeafKind::Text => Size {
            width: width.max(1.),
            height: label.height,
        },
    };
    validate_size(&size)?;
    Ok(Measured {
        size,
        content: Content::Leaf(label),
    })
}

fn measure_table(
    table: &Table,
    measured: &[Measured],
    budget: &mut Budget,
) -> Result<Measured, RenderError> {
    budget.charge(table.rows + table.columns + 2)?;
    bound(
        "Salt row boundaries",
        table.rows + 1,
        budget.limits.model_items,
    )?;
    bound(
        "Salt column boundaries",
        table.columns + 1,
        budget.limits.model_items,
    )?;
    let mut widths = vec![0_f64; table.columns];
    let mut heights = vec![0_f64; table.rows];
    for cell in &table.cells {
        budget.charge(1)?;
        let size = &measured[cell.widget].size;
        heights[cell.row] = heights[cell.row].max(size.height + GAP * 2.);
        if cell.column_span == 1 {
            widths[cell.column] = widths[cell.column].max(size.width + GAP * 2.);
        }
    }
    // Increasing a covered width never invalidates an earlier constraint.
    for cell in &table.cells {
        budget.charge(cell.column_span + 1)?;
        let end = cell.column + cell.column_span;
        let actual = widths[cell.column..end].iter().sum::<f64>();
        let missing = measured[cell.widget].size.width + GAP * 2. - actual;
        if missing > 0. {
            widths[end - 1] += missing;
        }
    }
    let x = boundaries(widths, budget)?;
    let y = boundaries(heights, budget)?;
    let size = Size {
        width: *x.last().unwrap(),
        height: *y.last().unwrap(),
    };
    validate_size(&size)?;
    Ok(Measured {
        size,
        content: Content::Table { x, y },
    })
}

fn boundaries(widths: Vec<f64>, budget: &mut Budget) -> Result<Vec<f64>, RenderError> {
    budget.charge(widths.len() + 1)?;
    let mut points = Vec::with_capacity(widths.len() + 1);
    let mut end = 0.;
    points.push(end);
    for width in widths {
        budget.charge(1)?;
        end += width;
        points.push(end);
    }
    Ok(points)
}

fn validate_size(size: &Size) -> Result<(), RenderError> {
    if size.width.is_finite()
        && size.height.is_finite()
        && size.width > 0.
        && size.height > 0.
        && size.width <= 1_000_000.
        && size.height <= 1_000_000.
    {
        return Ok(());
    }
    Err(invalid(
        1,
        "nonfinite, nonpositive or excessive Salt layout dimensions",
    ))
}

struct Painter<'a> {
    model: &'a Salt,
    measured: &'a [Measured],
    scene: &'a mut Scene,
    budget: &'a mut Budget,
}
impl Painter<'_> {
    fn widget(&mut self, index: usize, at: Point, owner: &str) -> Result<(), RenderError> {
        self.budget.charge(1)?;
        self.budget.text(owner.len() + 32)?;
        let widget = &self.model.widgets[index];
        let measured = &self.measured[index];
        let id = format!("salt-widget-{}", widget.id);
        match (&widget.kind, &measured.content) {
            (Kind::Table(table), Content::Table { x, y }) => {
                let mut element = Element::new(&id, "salt-table");
                element.name = table.border.name().into();
                element.owner = owner.into();
                self.table_border(&mut element, table, at, x, y)?;
                self.scene.push(element, self.budget)?;
                for cell in &table.cells {
                    self.budget.charge(8)?;
                    self.budget.text(id.len() + 96)?;
                    let cell_id = format!("salt-cell-{}", cell.id);
                    let mut element = Element::new(&cell_id, "salt-cell");
                    element.owner = id.clone();
                    element.target = format!("salt-widget-{}", self.model.widgets[cell.widget].id);
                    element.name = format!(
                        "{}:{}:{}:{}",
                        cell.row, cell.column, cell.row_span, cell.column_span
                    );
                    self.scene.push(element, self.budget)?;
                    self.widget(
                        cell.widget,
                        Point::new(at.x + x[cell.column] + GAP, at.y + y[cell.row] + GAP),
                        &cell_id,
                    )?;
                }
            }
            (Kind::Leaf(leaf), Content::Leaf(label)) => {
                self.budget.charge(32)?;
                let mut element = Element::new(id, leaf.kind.name());
                element.owner = owner.into();
                draw_leaf(
                    &mut element,
                    leaf.kind,
                    &Placement {
                        at,
                        size: &measured.size,
                        label,
                    },
                )?;
                self.scene.push(element, self.budget)?;
            }
            _ => unreachable!("measured Salt widget kind matches its model"),
        }
        Ok(())
    }

    fn table_border(
        &mut self,
        element: &mut Element,
        table: &Table,
        at: Point,
        x: &[f64],
        y: &[f64],
    ) -> Result<(), RenderError> {
        if table.border == Border::None {
            return Ok(());
        }
        self.budget.charge(1)?;
        element.shapes.push(Shape::Rect(
            Rect {
                x: at.x,
                y: at.y,
                width: *x.last().unwrap(),
                height: *y.last().unwrap(),
            },
            0.,
            Paint::None,
        ));
        if table.border == Border::Outer {
            return Ok(());
        }
        for cell in &table.cells {
            self.budget.charge(6)?;
            let right = cell.column + cell.column_span;
            let bottom = cell.row + cell.row_span;
            if right < table.columns {
                element.shapes.push(line(
                    Point::new(at.x + x[right], at.y + y[cell.row]),
                    Point::new(at.x + x[right], at.y + y[bottom]),
                ));
            }
            if bottom < table.rows {
                element.shapes.push(line(
                    Point::new(at.x + x[cell.column], at.y + y[bottom]),
                    Point::new(at.x + x[right], at.y + y[bottom]),
                ));
            }
        }
        Ok(())
    }
}

struct Placement<'a> {
    at: Point,
    size: &'a Size,
    label: &'a Label,
}
fn draw_leaf(
    element: &mut Element,
    kind: LeafKind,
    placement: &Placement<'_>,
) -> Result<(), RenderError> {
    let Placement { at, size, label } = placement;
    let mut text = *at;
    match kind {
        LeafKind::Field => {
            text.x += 3.;
            let bottom = at.y + size.height - 2.;
            element.shapes.push(line(
                Point::new(at.x + 1., bottom),
                Point::new(at.x + size.width - 1., bottom),
            ));
            for x in [at.x + 1., at.x + size.width - 1.] {
                element
                    .shapes
                    .push(line(Point::new(x, bottom - 3.), Point::new(x, bottom)));
            }
        }
        LeafKind::Button => {
            element.shapes.push(Shape::Rect(
                Rect {
                    x: at.x,
                    y: at.y,
                    width: size.width,
                    height: size.height,
                },
                5.,
                Paint::Node,
            ));
            text.x += (size.width - label.width) / 2.;
            text.y += 5.;
        }
        LeafKind::CheckboxOn | LeafKind::CheckboxOff | LeafKind::RadioOn | LeafKind::RadioOff => {
            draw_control(element, kind, placement);
            text.x += 20.;
        }
        LeafKind::Text => {}
    }
    element
        .shapes
        .push(Shape::Text(text, (*label).clone(), Decoration::Plain));
    Ok(())
}

fn draw_control(element: &mut Element, kind: LeafKind, placement: &Placement<'_>) {
    let Placement { at, size, .. } = placement;
    let rect = Rect {
        x: at.x + 2.,
        y: at.y + (size.height - 10.) / 2.,
        width: 10.,
        height: 10.,
    };
    if matches!(kind, LeafKind::RadioOn | LeafKind::RadioOff) {
        element.shapes.push(Shape::Ellipse(rect, Paint::None));
        if kind == LeafKind::RadioOn {
            element.shapes.push(Shape::Ellipse(
                Rect {
                    x: rect.x + 3.,
                    y: rect.y + 3.,
                    width: 4.,
                    height: 4.,
                },
                Paint::Ink,
            ));
        }
    } else {
        element.shapes.push(Shape::Rect(rect, 0., Paint::None));
        if kind == LeafKind::CheckboxOn {
            element.shapes.push(Shape::Polygon(
                vec![
                    Point::new(rect.x + 1., rect.y + 5.),
                    Point::new(rect.x + 4., rect.y + 8.),
                    Point::new(rect.x + 10., rect.y + 1.),
                    Point::new(rect.x + 4., rect.y + 6.),
                ],
                Paint::Ink,
            ));
        }
    }
}

fn line(start: Point, end: Point) -> Shape {
    Shape::Line(vec![start, end], Dash::Solid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RenderOptions;
    use merman_render::environment::RenderEnvironment;

    fn inspect(source: &str, verify: impl FnOnce(Salt, Vec<Measured>, Scene)) {
        let mut budget = Budget::new(&RenderOptions::default());
        let (preamble, lines) = super::super::source::prepare_styled(source, &mut budget).unwrap();
        let model = super::super::salt::parse(&lines, &mut budget).unwrap();
        let session = RenderEnvironment::deterministic().begin_session().unwrap();
        let mut context = Context::new(&session, &mut budget).with_fonts(preamble.fonts);
        let measured = measure(&model, &mut context).unwrap();
        let scene = layout(&model, &mut context).unwrap();
        verify(model, measured, scene);
    }

    #[test]
    fn should_contain_nested_measured_controls_and_span_constraints_without_overlap() {
        // given
        for size in [6, 96] {
            let source = format!(
                "@startsalt\nskinparam defaultFontSize {size}\n{{#\nName | \"Wide field        \"\n{{+\n[X] **Checked**\\nsecond row\n( ) Other\n}} | {{\n[Cancel] | [Continue]\nWide status label across both columns | *\n}}\n}}\n@endsalt"
            );
            // when
            inspect(&source, |model, measured, scene| {
                // then
                for (index, widget) in model.widgets.iter().enumerate() {
                    if let (Kind::Table(table), Content::Table { x, y }) =
                        (&widget.kind, &measured[index].content)
                    {
                        assert_eq!((x.len(), y.len()), (table.columns + 1, table.rows + 1));
                        for cell in &table.cells {
                            let child = &measured[cell.widget].size;
                            assert!(
                                child.width + 2. * GAP
                                    <= x[cell.column + cell.column_span] - x[cell.column]
                                        + 0.000001
                            );
                            assert!(
                                child.height + 2. * GAP
                                    <= y[cell.row + cell.row_span] - y[cell.row] + 0.000001
                            );
                        }
                        for cells in table.cells.windows(2) {
                            if cells[0].row == cells[1].row {
                                assert!(cells[0].column + cells[0].column_span <= cells[1].column);
                            } else {
                                assert!(cells[0].row + cells[0].row_span <= cells[1].row);
                            }
                        }
                    }
                }
                for element in &scene.elements {
                    for shape in &element.shapes {
                        if let Shape::Text(p, label, _) = shape {
                            assert!(
                                p.x >= scene.min_x
                                    && p.y >= scene.min_y
                                    && p.x + label.width <= scene.width
                                    && p.y + label.height <= scene.height
                            );
                        }
                    }
                }
            });
        }
    }

    #[test]
    fn should_suppress_internal_vertical_grid_segments_through_spans() {
        // given
        let source = "@startsalt\n{#\n[Left] | [Right]\nSpanning | *\n}\n@endsalt";
        // when
        inspect(source, |_, _, scene| {
            let table = scene
                .elements
                .iter()
                .find(|e| e.kind == "salt-table")
                .unwrap();
            let vertical = table
                .shapes
                .iter()
                .filter_map(|s| match s {
                    Shape::Line(points, _) if points[0].x == points[1].x => Some(points),
                    _ => None,
                })
                .collect::<Vec<_>>();
            // then
            assert_eq!(vertical.len(), 1);
            let Shape::Rect(rect, _, _) = table.shapes[0] else {
                panic!()
            };
            assert!(vertical[0][1].y < rect.y + rect.height);
        });
    }

    #[test]
    fn should_keep_ragged_tables_sparse_and_charge_layout_propagation() {
        // given
        let source = format!(
            "@startsalt\n{{\n{}\n{}\n}}\n@endsalt",
            (0..100).map(|_| "a").collect::<Vec<_>>().join(" | "),
            "b\n".repeat(100)
        );
        // when
        inspect(&source, |model, measured, _| {
            let Kind::Table(table) = &model.widgets[model.root].kind else {
                panic!()
            };
            let Content::Table { x, y } = &measured[model.root].content else {
                panic!()
            };
            // then
            assert_eq!(table.cells.len(), 200);
            assert_eq!((x.len(), y.len()), (101, 102));
            assert_eq!(model.widgets.len(), 201);
        });
        let mut options = RenderOptions::default();
        options.limits.layout_work = 10;
        let mut budget = Budget::new(&options);
        let table = Table {
            border: Border::None,
            cells: Vec::new(),
            rows: 100,
            columns: 100,
        };
        assert!(measure_table(&table, &[], &mut budget).is_err());
        assert_eq!(budget.work, 0);
    }
}
