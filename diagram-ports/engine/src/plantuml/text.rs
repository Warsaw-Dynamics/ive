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
    pub font_family: &'static str,
    pub lines: Vec<String>,
    pub width: f64,
    pub height: f64,
    pub line_height: f64,
}

pub(super) struct Context<'a> {
    pub session: &'a RenderSession,
    pub budget: &'a mut Budget,
    cache: HashMap<(String, Decoration, LabelMode), Label>,
    calls: usize,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
enum LabelMode {
    Display,
    Tree,
    Data,
}

impl LabelMode {
    fn font_family(self) -> &'static str {
        match self {
            Self::Display => "sans-serif",
            Self::Tree | Self::Data => "Arial, Liberation Sans, DejaVu Sans, Noto Sans, sans-serif",
        }
    }
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
        self.measure_label(text, decoration, LabelMode::Display)
    }

    pub fn data_label(&mut self, text: &str) -> Result<Label, RenderError> {
        self.measure_label(text, Decoration::Plain, LabelMode::Data)
    }

    pub fn tree_label(&mut self, text: &str) -> Result<Label, RenderError> {
        self.measure_label(text, Decoration::Plain, LabelMode::Tree)
    }

    fn measure_label(
        &mut self,
        text: &str,
        decoration: Decoration,
        mode: LabelMode,
    ) -> Result<Label, RenderError> {
        self.budget.charge(text.len() + 1)?;
        bound("label bytes", text.len(), 16 * 1024)?;
        let key = (text.to_owned(), decoration, mode);
        if let Some(label) = self.cache.get(&key) {
            return Ok(label.clone());
        }
        let normalized = match mode {
            LabelMode::Display | LabelMode::Tree => literal_label(text, 1)?.replace("\\n", "\n"),
            LabelMode::Data => text.to_owned(),
        };
        let lines: Vec<String> = normalized.split('\n').map(str::to_owned).collect();
        bound("measured lines", self.calls + lines.len(), 4096)?;
        let style = TextStyle {
            font_family: Some(mode.font_family().into()),
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
            font_family: mode.font_family(),
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
    literal_label(text, line).map(|_| ())
}

fn literal_label(text: &str, line: usize) -> Result<String, RenderError> {
    let mut literal = String::with_capacity(text.len());
    let mut rest = text;
    let mut delimiters = [false; 6];
    let mut url = false;
    while let Some(ch) = rest.chars().next() {
        if ch.is_whitespace() {
            url = false;
        }
        if let Some(scheme) = ["https://", "http://", "ftp://", "file://"]
            .iter()
            .find(|scheme| rest.starts_with(**scheme))
        {
            literal.push_str(scheme);
            rest = &rest[scheme.len()..];
            url = true;
            continue;
        }
        if rest.starts_with("\\~") {
            literal.push('~');
            rest = &rest[2..];
            continue;
        }
        if rest.starts_with('~') && !rest.starts_with("~~") {
            let after = &rest[1..];
            if let Some(escaped) = after
                .chars()
                .next()
                .filter(|c| matches!(c, '<' | '*' | '_' | '-' | '/' | '[' | ']' | '"' | '#' | '.'))
            {
                literal.push(escaped);
                rest = &after[escaped.len_utf8()..];
                continue;
            }
        }
        let url_slashes = rest.starts_with("//") && (url || literal.ends_with(':'));
        if url_slashes {
            literal.push_str("//");
            rest = &rest[2..];
            continue;
        }
        if rest.starts_with("[[") || (ch == '<' && rich_tag(&rest[1..])) {
            return Err(unsupported(
                line,
                "rich text, links, images, sprites or math",
            ));
        }
        if let Some(index) = ["**", "__", "~~", "--", "//", "\"\""]
            .iter()
            .position(|marker| rest.starts_with(marker))
        {
            if delimiters[index] {
                return Err(unsupported(line, "paired rich-text delimiter"));
            }
            delimiters[index] = true;
            literal.push_str(&rest[..2]);
            rest = &rest[2..];
            continue;
        }
        literal.push(ch);
        rest = &rest[ch.len_utf8()..];
    }
    Ok(literal)
}

fn rich_tag(rest: &str) -> bool {
    let rest = rest.strip_prefix('/').unwrap_or(rest);
    if rest.starts_with(['$', '&', ':', '#']) {
        return true;
    }
    let end = rest
        .find(|ch: char| !ch.is_ascii_alphabetic())
        .unwrap_or(rest.len());
    let name = &rest[..end];
    matches!(
        rest.as_bytes().get(end),
        Some(b'>' | b':' | b'/' | b' ' | b'\t')
    ) && [
        "b", "i", "u", "s", "strike", "del", "w", "plain", "back", "color", "size", "font", "text",
        "sup", "sub", "img", "image", "sprite", "math", "latex", "qrcode", "space",
    ]
    .iter()
    .any(|reserved| name.eq_ignore_ascii_case(reserved))
}
