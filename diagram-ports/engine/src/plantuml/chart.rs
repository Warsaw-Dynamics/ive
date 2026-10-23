//! Categorical chart semantics from PlantUML MIT ChartAxis/ChartDiagram/ChartSeries.
//! Copyright 2009–2024 Arnaud Roques; Original Author: David Fyfe. MIT grant in provenance/.
use super::{
    budget::Budget,
    parser::{context, error::ParseFailure},
    style::Color,
};
use crate::RenderError;
use std::ops::Range;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SeriesKind {
    Bar,
    Line,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum LabelsMode {
    Hidden,
    Values,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum LegendPosition {
    None,
    Left,
    Right,
    Top,
    Bottom,
}
#[derive(Clone, Copy, Debug)]
pub(super) enum SeriesColor {
    Palette,
    Rgb(Color),
}
#[derive(Clone, Copy, Debug)]
pub(super) enum ScaleRange {
    Auto(f64, f64),
    Fixed(f64, f64),
}
impl ScaleRange {
    pub fn bounds(self) -> (f64, f64) {
        match self {
            Self::Auto(a, b) | Self::Fixed(a, b) => (a, b),
        }
    }
}
pub(super) struct CategoryAxis {
    pub title: String,
    pub categories: Vec<String>,
}
pub(super) struct NumericAxis {
    pub title: String,
    pub range: ScaleRange,
}
pub(super) struct Series {
    pub span: Range<usize>,
    pub name: String,
    pub kind: SeriesKind,
    pub samples: Vec<f64>,
    pub labels: LabelsMode,
    pub color: SeriesColor,
}
pub(super) struct Chart {
    pub category_axis: CategoryAxis,
    pub value_axis: NumericAxis,
    pub series: Vec<Series>,
    pub legend: LegendPosition,
}
pub(in crate::plantuml) enum Command {
    Categories(String, Vec<String>),
    AxisTitle(String),
    AxisRange(String, f64, f64),
    Series(Series),
    Legend(LegendPosition),
}
pub(in crate::plantuml) struct Builder<'b> {
    pub budget: &'b mut Budget,
    chart: Chart,
    bars: usize,
}
impl<'b> Builder<'b> {
    pub fn new(budget: &'b mut Budget) -> Self {
        Self {
            budget,
            bars: 0,
            chart: Chart {
                category_axis: CategoryAxis {
                    title: String::new(),
                    categories: Vec::new(),
                },
                value_axis: NumericAxis {
                    title: String::new(),
                    range: ScaleRange::Auto(0., 100.),
                },
                series: Vec::new(),
                legend: LegendPosition::None,
            },
        }
    }
    pub fn push<T>(&mut self, mut values: Vec<T>, value: T) -> Result<Vec<T>, ParseFailure> {
        self.budget.item()?;
        context::push(&mut values, value, self.budget)?;
        Ok(values)
    }
    pub fn command(&mut self, command: Command, span: Range<usize>) -> Result<(), ParseFailure> {
        self.budget.charge(1)?;
        match command {
            Command::Categories(title, categories) => {
                for category in &categories {
                    self.budget.charge(category.len())?;
                    if category.contains(',') {
                        return Err(ParseFailure::unsupported(
                            span,
                            "chart category labels containing commas are unsupported",
                        ));
                    }
                }
                self.chart.category_axis = CategoryAxis { title, categories };
            }
            Command::AxisTitle(title) => {
                if !title.is_empty() {
                    self.chart.value_axis.title = title;
                }
            }
            Command::AxisRange(title, min, max) => {
                if min >= max || !(max - min).is_finite() {
                    return Err(ParseFailure::unsupported(
                        span,
                        "chart range requires finite min < max",
                    ));
                }
                if !title.is_empty() {
                    self.chart.value_axis.title = title;
                }
                self.chart.value_axis.range = ScaleRange::Fixed(min, max);
            }
            Command::Legend(position) => self.chart.legend = position,
            Command::Series(mut series) => {
                if series.kind == SeriesKind::Line && series.labels == LabelsMode::Values {
                    return Err(ParseFailure::unsupported(
                        span,
                        "chart line labels are unsupported",
                    ));
                }
                if series.kind == SeriesKind::Bar && self.bars > 0 {
                    return Err(ParseFailure::unsupported(
                        span,
                        "multiple chart bar series require unsupported grouped or stacked layout",
                    ));
                }
                self.bars += usize::from(series.kind == SeriesKind::Bar);
                if series.name.is_empty() {
                    series.name = format!(
                        "{}{}",
                        if series.kind == SeriesKind::Bar {
                            "bar"
                        } else {
                            "line"
                        },
                        self.chart.series.len()
                    );
                }
                for value in &series.samples {
                    self.budget.charge(1)?;
                    if let ScaleRange::Auto(min, max) = self.chart.value_axis.range {
                        self.chart.value_axis.range =
                            ScaleRange::Auto(min.min(*value), max.max(*value));
                    }
                }
                self.budget.item()?;
                context::push(&mut self.chart.series, series, self.budget)?;
            }
        }
        Ok(())
    }
    pub fn finish(self, span: Range<usize>) -> Result<Chart, ParseFailure> {
        let count = self.chart.category_axis.categories.len();
        if count == 0 || self.chart.series.is_empty() {
            return Err(ParseFailure::unsupported(
                span,
                "chart requires nonempty categories and series",
            ));
        }
        let product = count.checked_mul(self.chart.series.len()).ok_or_else(|| {
            ParseFailure::terminal(RenderError::Backend("chart sample product overflow".into()))
        })?;
        super::budget::bound(
            "chart sample product",
            product,
            self.budget.limits.model_items,
        )?;
        self.budget.charge(product)?;
        let (min, max) = self.chart.value_axis.range.bounds();
        for series in &self.chart.series {
            if series.samples.len() != count {
                self.budget
                    .text(series.name.len().saturating_mul(4).saturating_add(128))?;
                return Err(ParseFailure {
                    span: series.span.clone(),
                    kind: super::parser::error::FailureKind::Unsupported(
                        format!(
                            "chart series {:?} has {} samples for {count} categories",
                            series.name,
                            series.samples.len()
                        )
                        .into(),
                    ),
                });
            }
            for value in &series.samples {
                self.budget.charge(1)?;
                if *value < min || *value > max {
                    self.budget
                        .text(series.name.len().saturating_mul(4).saturating_add(256))?;
                    return Err(ParseFailure { span:series.span.clone(), kind:super::parser::error::FailureKind::Unsupported(format!("chart series {:?} sample {value} lies outside fixed Y range {min}..{max}",series.name).into()) });
                }
            }
        }
        Ok(self.chart)
    }
}
pub(super) fn color(series: &Series, index: usize) -> Color {
    match series.color {
        SeriesColor::Rgb(color) => color,
        SeriesColor::Palette => Color(
            [
                0x1f77b4, 0xff7f0e, 0x2ca02c, 0xd62728, 0x9467bd, 0x8c564b, 0xe377c2, 0x7f7f7f,
                0xbcbd22, 0x17becf,
            ][index % 10],
        ),
    }
}
pub(super) fn format_number(value: f64) -> String {
    if value == 0. {
        "0".into()
    } else {
        value.to_string()
    }
}
