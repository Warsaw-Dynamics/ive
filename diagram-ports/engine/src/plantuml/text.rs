//! PlantUML MIT: klimt/font/StringBounder default descent; native metrics via Merman.
use super::{
    budget::{Budget, bound, invalid, unsupported},
    scene::Decoration,
};
use crate::{RenderError, TextMeasurementPhase, TextStyle};
use merman_render::{environment::RenderSession, text::TextMeasurer};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub(super) struct Label {
    pub lines: Vec<String>,
    pub width: f64,
    pub height: f64,
    pub line_height: f64,
}

pub(super) struct Context<'a> {
    pub session: &'a RenderSession,
    pub budget: &'a mut Budget,
    cache: HashMap<(String, Decoration), Label>,
    calls: usize,
}

impl<'a> Context<'a> {
    pub fn new(session: &'a RenderSession, budget: &'a mut Budget) -> Self {
        Self {
            session,
            budget,
            cache: HashMap::new(),
            calls: 0,
        }
    }

    pub fn label(&mut self, text: &str) -> Result<Label, RenderError> {
        self.styled_label(text, Decoration::Plain)
    }

    pub fn styled_label(
        &mut self,
        text: &str,
        decoration: Decoration,
    ) -> Result<Label, RenderError> {
        self.budget.charge(text.len() + 1)?;
        bound("label bytes", text.len(), 16 * 1024)?;
        let key = (text.to_owned(), decoration);
        if let Some(label) = self.cache.get(&key) {
            return Ok(label.clone());
        }
        let lines: Vec<String> = text
            .replace("\\n", "\n")
            .split('\n')
            .map(str::to_owned)
            .collect();
        bound("measured lines", self.calls + lines.len(), 4096)?;
        let style = TextStyle {
            font_family: Some("sans-serif".into()),
            font_size: 14.,
            font_style: Some(
                if matches!(decoration, Decoration::Italic) {
                    "italic"
                } else {
                    "normal"
                }
                .into(),
            ),
            ..Default::default()
        };
        let mut width: f64 = 0.;
        let mut line_height: f64 = 18.;
        for line in &lines {
            self.budget.checkpoint()?;
            let metrics = self
                .session
                .text_measurer(TextMeasurementPhase::Layout)
                .measure(line, &style);
            self.calls += 1;
            self.budget.checkpoint()?;
            if !metrics.width.is_finite()
                || !metrics.height.is_finite()
                || metrics.width < 0.
                || metrics.height < 0.
            {
                return Err(invalid(1, "invalid text measurement"));
            }
            width = width.max(metrics.width);
            line_height = line_height.max(metrics.height + 4.);
        }
        let label = Label {
            width,
            height: line_height * lines.len() as f64,
            lines,
            line_height,
        };
        self.cache.insert(key, label.clone());
        Ok(label)
    }
}

pub(super) fn validate_label(text: &str, line: usize) -> Result<(), RenderError> {
    let italic = text
        .match_indices("//")
        .filter(|(i, _)| *i == 0 || !text[..*i].ends_with(':'))
        .count()
        >= 2;
    if italic
        || [
            "[[", "<img", "<image", "<$", "<sprite", "<math", "<latex", "<b>", "<i>", "<u>",
            "<back", "<color", "<size", "<font", "**", "__", "~~",
        ]
        .iter()
        .any(|marker| text.contains(marker))
    {
        return Err(unsupported(
            line,
            "rich text, links, images, sprites or math",
        ));
    }
    Ok(())
}
