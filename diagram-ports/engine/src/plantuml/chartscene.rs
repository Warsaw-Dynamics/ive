//! Native measured categorical projection; MIT ChartRenderer/BarRenderer/LineRenderer semantics.
//! Copyright 2009–2024 Arnaud Roques; Original Author: David Fyfe. Native geometry differs.
use super::{
    budget::{Budget, bound},
    chart::{self, Chart, LabelsMode, LegendPosition, SeriesKind},
    scene::{Dash, Decoration, Element, Paint, Point, Rect, Scene, Shape},
    style::Role,
    text::{Context, Label},
};
use crate::RenderError;

struct Measures {
    categories: Vec<Label>,
    ticks: Vec<Label>,
    values: Vec<Vec<Label>>,
    names: Vec<Label>,
    x_title: Label,
    y_title: Label,
}
struct Geometry {
    plot: Rect,
    legend: Rect,
    bottom: f64,
}

pub(super) fn layout(chart: &Chart, context: &mut Context<'_>) -> Result<Scene, RenderError> {
    let count = chart.category_axis.categories.len();
    let samples = count
        .checked_mul(chart.series.len())
        .ok_or_else(|| RenderError::Backend("chart sample product overflow".into()))?;
    context.budget.charge(samples)?;
    bound(
        "chart layout retained bytes",
        samples
            .saturating_add(count + chart.series.len() + 6)
            .saturating_mul(1024),
        context.budget.limits.source_bytes.saturating_mul(256),
    )?;
    let measured = measure(chart, context)?;
    let geometry = reserve(chart, &measured, context)?;
    let mut scene = Scene::new();
    draw_axes((&mut scene, &geometry), chart, &measured, context)?;
    draw_series((&mut scene, &geometry), chart, &measured, context)?;
    draw_legend((&mut scene, &geometry), chart, &measured, context)?;
    scene.height = scene.height.max(geometry.bottom + 24.);
    Ok(scene)
}
fn store<T>(values: &mut Vec<T>, value: T, budget: &mut Budget) -> Result<(), RenderError> {
    super::parser::context::push(values, value, budget)
        .map_err(|failure| failure.into_document(&super::origin::EMPTY, budget))
}
fn measure(chart: &Chart, context: &mut Context<'_>) -> Result<Measures, RenderError> {
    let mut categories = Vec::new();
    let mut ticks = Vec::new();
    let mut names = Vec::new();
    let mut values = Vec::new();
    for category in &chart.category_axis.categories {
        let label = context.label(category)?;
        store(&mut categories, label, context.budget)?;
    }
    let (min, max) = chart.value_axis.range.bounds();
    for i in 0..=5 {
        context.budget.item()?;
        let label = context.data_label(&chart::format_number(min + (max - min) * i as f64 / 5.))?;
        store(&mut ticks, label, context.budget)?;
    }
    for series in &chart.series {
        let name = if chart.legend == LegendPosition::None {
            context.region_label("", Role::Legend)?
        } else {
            context.region_label(&series.name, Role::Legend)?
        };
        store(&mut names, name, context.budget)?;
        let mut labels = Vec::new();
        if series.labels == LabelsMode::Values {
            for value in &series.samples {
                let label = context.data_label(&chart::format_number(*value))?;
                store(&mut labels, label, context.budget)?;
            }
        }
        store(&mut values, labels, context.budget)?;
    }
    Ok(Measures {
        categories,
        ticks,
        values,
        names,
        x_title: context.label(&chart.category_axis.title)?,
        y_title: context.label(&chart.value_axis.title)?,
    })
}
fn reserve(
    chart: &Chart,
    m: &Measures,
    context: &mut Context<'_>,
) -> Result<Geometry, RenderError> {
    let mut slot = 60_f64;
    let mut value_height = 0_f64;
    for label in m.categories.iter().chain(m.values.iter().flatten()) {
        context.budget.charge(1)?;
        slot = slot.max(label.width + 20.);
    }
    for label in m.values.iter().flatten() {
        context.budget.charge(1)?;
        value_height = value_height.max(label.height + 10.);
    }
    let ticks_width = m.ticks.iter().map(|l| l.width).fold(0., f64::max);
    let ticks_height = m.ticks.iter().map(|l| l.height).fold(0., f64::max);
    let categories_height = m.categories.iter().map(|l| l.height).fold(0., f64::max);
    let mut width = (slot * m.categories.len() as f64)
        .max(400.)
        .max(m.x_title.width + 20.);
    let height = 300_f64.max((ticks_height + 12.) * 5.);
    let (legend_width, legend_height) = legend_size(chart.legend, &m.names, context.budget)?;
    if matches!(chart.legend, LegendPosition::Top | LegendPosition::Bottom) {
        width = width.max(legend_width);
    }
    let left = 32. + ticks_width + 10. + m.y_title.width / 2.;
    let top = 32. + ticks_height / 2. + value_height + m.y_title.height;
    let x = left
        + if chart.legend == LegendPosition::Left {
            legend_width + 24.
        } else {
            0.
        };
    let y = top
        + if chart.legend == LegendPosition::Top {
            legend_height + 24.
        } else {
            0.
        };
    let bottom = y + height + value_height + categories_height + 20. + m.x_title.height;
    let legend = match chart.legend {
        LegendPosition::Left => Rect {
            x: 24.,
            y,
            width: legend_width,
            height: legend_height,
        },
        LegendPosition::Right => Rect {
            x: x + width + 24.,
            y,
            width: legend_width,
            height: legend_height,
        },
        LegendPosition::Top => Rect {
            x,
            y: 24.,
            width: legend_width,
            height: legend_height,
        },
        _ => Rect {
            x,
            y: bottom + 24.,
            width: legend_width,
            height: legend_height,
        },
    };
    for dimension in [
        x + width,
        y + height,
        legend.x + legend.width,
        legend.y + legend.height,
        bottom,
    ] {
        if !dimension.is_finite() || dimension > 999_900. {
            return Err(RenderError::Backend(
                "chart measured geometry exceeds coordinate limit".into(),
            ));
        }
    }
    Ok(Geometry {
        plot: Rect {
            x,
            y,
            width,
            height,
        },
        legend,
        bottom: bottom.max(if chart.legend == LegendPosition::None {
            0.
        } else {
            legend.y + legend.height
        }),
    })
}
fn legend_size(
    position: LegendPosition,
    names: &[Label],
    budget: &mut Budget,
) -> Result<(f64, f64), RenderError> {
    if position == LegendPosition::None {
        return Ok((0., 0.));
    }
    let mut width = 0_f64;
    let mut height = 0_f64;
    for label in names {
        budget.charge(1)?;
        if matches!(position, LegendPosition::Left | LegendPosition::Right) {
            width = width.max(label.width + 32.);
            height += label.height.max(12.) + 12.;
        } else {
            width += label.width + 44.;
            height = height.max(label.height.max(12.));
        }
    }
    Ok((width, height))
}
fn put_text(
    scene: &mut Scene,
    value: (&str, Point, Label),
    budget: &mut Budget,
) -> Result<(), RenderError> {
    let mut element = Element::new(value.0, "chart-label");
    element
        .shapes
        .push(Shape::Text(value.1, value.2, Decoration::Plain));
    scene.push(element, budget)
}
fn draw_axes(
    target: (&mut Scene, &Geometry),
    chart: &Chart,
    m: &Measures,
    context: &mut Context<'_>,
) -> Result<(), RenderError> {
    let (scene, g) = target;
    let p = g.plot;
    let (min, max) = chart.value_axis.range.bounds();
    let baseline = project(0., (min, max), p).clamp(p.y, p.y + p.height);
    let mut axes = Element::new("chart-axis", "chart-axis");
    axes.shapes.push(Shape::Line(
        vec![Point::new(p.x, p.y), Point::new(p.x, p.y + p.height)],
        Dash::Solid,
    ));
    axes.shapes.push(Shape::Line(
        vec![
            Point::new(p.x, baseline),
            Point::new(p.x + p.width, baseline),
        ],
        Dash::Solid,
    ));
    scene.push(axes, context.budget)?;
    for (i, label) in m.ticks.iter().enumerate() {
        context.budget.charge(1)?;
        let y = p.y + p.height * (1. - i as f64 / 5.);
        let mut tick = Element::new(format!("ct{i}"), "chart-tick");
        tick.name = chart::format_number(min + (max - min) * i as f64 / 5.);
        tick.shapes.push(Shape::Line(
            vec![Point::new(p.x - 5., y), Point::new(p.x, y)],
            Dash::Solid,
        ));
        tick.shapes.push(Shape::Text(
            Point::new(p.x - label.width - 10., y - label.height / 2.),
            label.clone(),
            Decoration::Plain,
        ));
        scene.push(tick, context.budget)?;
    }
    let value_height = m
        .values
        .iter()
        .flatten()
        .map(|l| l.height + 10.)
        .fold(0., f64::max);
    let category_y = p.y + p.height + value_height + 12.;
    for (i, label) in m.categories.iter().enumerate() {
        let x = p.x + (i as f64 + 0.5) * p.width / m.categories.len() as f64;
        let mut element = Element::new(format!("cc{i}"), "chart-category");
        element.name = chart.category_axis.categories[i].clone();
        element.shapes.push(Shape::Text(
            Point::new(x - label.width / 2., category_y),
            label.clone(),
            Decoration::Plain,
        ));
        scene.push(element, context.budget)?;
    }
    let categories_height = m.categories.iter().map(|l| l.height).fold(0., f64::max);
    put_text(
        scene,
        (
            "cx-title",
            Point::new(
                p.x + (p.width - m.x_title.width) / 2.,
                category_y + categories_height + 10.,
            ),
            m.x_title.clone(),
        ),
        context.budget,
    )?;
    put_text(
        scene,
        (
            "cy-title",
            Point::new(
                p.x - m.y_title.width / 2.,
                p.y - value_height - m.y_title.height - 16.,
            ),
            m.y_title.clone(),
        ),
        context.budget,
    )
}
fn project(value: f64, range: (f64, f64), plot: Rect) -> f64 {
    plot.y + plot.height * (1. - (value - range.0) / (range.1 - range.0))
}
fn draw_series(
    target: (&mut Scene, &Geometry),
    chart: &Chart,
    m: &Measures,
    context: &mut Context<'_>,
) -> Result<(), RenderError> {
    let (scene, g) = target;
    let p = g.plot;
    let range = chart.value_axis.range.bounds();
    let baseline = project(0., range, p).clamp(p.y, p.y + p.height);
    let slot = p.width / m.categories.len() as f64;
    for (id, series) in chart.series.iter().enumerate() {
        let color = chart::color(series, id);
        let mut parent = Element::new(format!("cs{id}"), "chart-series");
        parent.name = series.name.clone();
        scene.push(parent, context.budget)?;
        let mut points = Vec::new();
        for (i, value) in series.samples.iter().enumerate() {
            context.budget.item()?;
            let x = p.x + (i as f64 + 0.5) * slot;
            let y = project(*value, range, p);
            let mut element = Element::new(
                format!("cs{id}v{i}"),
                if series.kind == SeriesKind::Bar {
                    "chart-bar"
                } else {
                    "chart-point"
                },
            );
            element.owner = format!("cs{id}");
            element.name = chart::format_number(*value);
            if series.kind == SeriesKind::Bar {
                element.shapes.push(Shape::Rect(
                    Rect {
                        x: x - slot * 0.3,
                        y: y.min(baseline),
                        width: slot * 0.6,
                        height: (y - baseline).abs(),
                    },
                    0.,
                    Paint::Color(color),
                ));
            } else {
                element.shapes.push(Shape::Ellipse(
                    Rect {
                        x: x - 3.,
                        y: y - 3.,
                        width: 6.,
                        height: 6.,
                    },
                    Paint::Color(color),
                ));
                store(&mut points, Point::new(x, y), context.budget)?;
            }
            if series.labels == LabelsMode::Values {
                let label = &m.values[id][i];
                element.shapes.push(Shape::Text(
                    Point::new(
                        x - label.width / 2.,
                        if *value < 0. {
                            y + 8.
                        } else {
                            y - label.height - 8.
                        },
                    ),
                    label.clone(),
                    Decoration::Plain,
                ));
            }
            scene.push(element, context.budget)?;
        }
        if points.len() > 1 {
            let mut line = Element::new(format!("cs{id}line"), "chart-line");
            line.owner = format!("cs{id}");
            line.shapes
                .push(Shape::ColoredLine(points, Dash::Bold, color));
            scene.push(line, context.budget)?;
        }
    }
    Ok(())
}
fn draw_legend(
    target: (&mut Scene, &Geometry),
    chart: &Chart,
    m: &Measures,
    context: &mut Context<'_>,
) -> Result<(), RenderError> {
    if chart.legend == LegendPosition::None {
        return Ok(());
    }
    let (scene, g) = target;
    let mut x = g.legend.x;
    let mut y = g.legend.y;
    for (id, series) in chart.series.iter().enumerate() {
        context.budget.charge(1)?;
        let label = &m.names[id];
        let color = chart::color(series, id);
        let mut item = Element::new(format!("cl{id}"), "chart-legend");
        item.owner = format!("cs{id}");
        item.name = series.name.clone();
        if series.kind == SeriesKind::Bar {
            item.shapes.push(Shape::Rect(
                Rect {
                    x,
                    y: y + (label.height - 12.) / 2.,
                    width: 12.,
                    height: 12.,
                },
                0.,
                Paint::Color(color),
            ));
        } else {
            item.shapes.push(Shape::ColoredLine(
                vec![
                    Point::new(x, y + label.height / 2.),
                    Point::new(x + 12., y + label.height / 2.),
                ],
                Dash::Bold,
                color,
            ));
        }
        item.shapes.push(Shape::Text(
            Point::new(x + 20., y),
            label.clone(),
            Decoration::Plain,
        ));
        scene.push(item, context.budget)?;
        if matches!(chart.legend, LegendPosition::Left | LegendPosition::Right) {
            y += label.height.max(12.) + 12.;
        } else {
            x += label.width + 44.;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        RenderOptions,
        plantuml::parser::{chart::parse, test_support},
    };
    use merman_render::environment::RenderEnvironment;
    fn inspect(source: &str, verify: impl FnOnce(Chart, Measures, Geometry, Scene)) {
        let mut budget = Budget::new(&RenderOptions::default());
        let (preamble, model) = test_support::styled(source, &mut budget, parse).unwrap();
        let session = RenderEnvironment::deterministic().begin_session().unwrap();
        let mut context = Context::new(&session, &mut budget).with_fonts(preamble.fonts);
        let measured = measure(&model, &mut context).unwrap();
        let geometry = reserve(&model, &measured, &mut context).unwrap();
        let scene = layout(&model, &mut context).unwrap();
        verify(model, measured, geometry, scene);
    }
    #[test]
    fn should_project_signed_bars_zero_and_every_line_point_from_one_numeric_scale() {
        // given
        let source = "@startchart\nx-axis [\"negative\",\"zero\",\"positive\"]\ny-axis -10-->20\nbar [-10,0,20] labels\nline [20,0,-10] #112233\n@endchart";
        // when
        inspect(source, |_, _, g, scene| {
            // then
            let bars = scene
                .elements
                .iter()
                .filter(|e| e.kind == "chart-bar")
                .map(|e| match e.shapes[0] {
                    Shape::Rect(r, _, _) => r,
                    _ => panic!(),
                })
                .collect::<Vec<_>>();
            assert_eq!(bars.len(), 3);
            assert!((bars[0].height - g.plot.height / 3.).abs() < 1e-6);
            assert_eq!(bars[1].height, 0.);
            assert!((bars[2].height - g.plot.height * 2. / 3.).abs() < 1e-6);
            let line = scene
                .elements
                .iter()
                .find(|e| e.kind == "chart-line")
                .unwrap();
            let Shape::ColoredLine(points, _, _) = &line.shapes[0] else {
                panic!()
            };
            assert_eq!(points.len(), 3);
            for (i, point) in points.iter().enumerate() {
                assert!((point.x - (g.plot.x + (i as f64 + 0.5) * g.plot.width / 3.)).abs() < 1e-6);
            }
            assert_eq!(points[0].y, g.plot.y);
            assert_eq!(points[2].y, g.plot.y + g.plot.height);
        });
    }
    #[test]
    fn should_reserve_measured_legend_title_categories_ticks_and_signed_value_gutters() {
        // given
        for size in [6, 14, 96] {
            for position in ["left", "right", "top", "bottom"] {
                let source = format!(
                    "@startchart\nskinparam defaultFontSize {size}\nlegend {position}\nx-axis \"Category title\" [\"One long category\",\"Two long category\"]\ny-axis \"Value title\" -10-->20\nbar \"**Long series**\\nsecond line\" [-10,20] labels\nline \"Line series\" [20,-10]\n@endchart"
                );
                // when
                inspect(&source, |_, m, g, scene| {
                    // then
                    let p = g.plot;
                    let l = g.legend;
                    assert!(
                        l.x + l.width <= p.x
                            || l.x >= p.x + p.width
                            || l.y + l.height <= p.y
                            || l.y >= p.y + p.height
                    );
                    for element in &scene.elements {
                        for shape in &element.shapes {
                            if let Shape::Text(point, label, _) = shape {
                                assert!(point.x >= scene.min_x && point.y >= scene.min_y);
                                assert!(
                                    point.x + label.width <= scene.width
                                        && point.y + label.height <= scene.height
                                );
                            }
                        }
                    }
                    assert!(
                        m.categories
                            .iter()
                            .all(|label| label.width + 20. <= p.width / 2.)
                    );
                    let categories = scene.elements.iter().filter(|e| e.kind == "chart-category");
                    for e in categories {
                        let Shape::Text(point, _, _) = &e.shapes[0] else {
                            panic!()
                        };
                        assert!(point.y > p.y + p.height);
                    }
                    let ytitle = scene.elements.iter().find(|e| e.id == "cy-title").unwrap();
                    let Shape::Text(point, label, _) = &ytitle.shapes[0] else {
                        panic!()
                    };
                    assert!(point.y + label.height < p.y);
                    if position == "top" {
                        assert!(l.y + l.height < point.y);
                    }
                });
            }
        }
    }
    #[test]
    fn should_bound_tick_allocation_and_measurement_before_chart_projection() {
        // given
        let model = test_support::body(
            ("x-axis [\"a\"]\nbar [1]", "chart"),
            &mut Budget::new(&RenderOptions::default()),
            parse,
        )
        .unwrap();
        let session = RenderEnvironment::deterministic().begin_session().unwrap();
        let mut budget = Budget::new(&RenderOptions::default());
        budget.limits.model_items = 5;
        let mut context = Context::new(&session, &mut budget);
        // when
        let error = match measure(&model, &mut context) {
            Err(error) => error,
            Ok(_) => panic!("sixth tick admitted"),
        };
        // then
        assert!(error.to_string().contains("model items"));
    }
}
