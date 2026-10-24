use crate::worker::{MeasureQuery, MeasureReply, Measurer};
use diagram_engine::{
    HostMeasurementResult, HostTextMeasurement, HostTextMeasurementRequest, HostTextMeasurer,
    MeasurementProfileId, TextMeasurementOperation, TextMeasurementPhase, TextMeasurementPolicy,
    TextMeasurementProfileIdentity, TextMetrics,
};
use gpui::{FontStyle, FontWeight, TextRun, TextSystem, WindowTextSystem, font, px};
use std::{
    collections::BTreeSet,
    sync::{Arc, Mutex},
};

pub struct NativeText {
    system: WindowTextSystem,
    names: Arc<[String]>,
    resolved: Mutex<BTreeSet<String>>,
}

pub struct NativeTextResources {
    system: Arc<TextSystem>,
    names: Arc<[String]>,
}

struct DirectHost(Arc<NativeText>);

impl NativeTextResources {
    pub fn new(system: Arc<TextSystem>) -> Self {
        Self {
            names: system.all_font_names().into(),
            system,
        }
    }
    pub fn start_render(&self) -> Arc<NativeText> {
        Arc::new(NativeText::new(self.system.clone(), self.names.clone()))
    }
}

impl NativeText {
    fn new(system: Arc<TextSystem>, names: Arc<[String]>) -> Self {
        Self {
            names,
            system: WindowTextSystem::new(system),
            resolved: Mutex::new(BTreeSet::new()),
        }
    }
    pub fn report(&self) -> Vec<String> {
        self.resolved
            .lock()
            .map(|items| items.iter().take(8).cloned().collect())
            .unwrap_or_default()
    }

    pub fn policy(self: &Arc<Self>) -> Result<TextMeasurementPolicy, String> {
        let profile = MeasurementProfileId::new("pgui-native-unwrapped")
            .map_err(|error| error.to_string())?;
        let identity = TextMeasurementProfileIdentity::new(profile, "f5d5fa2-v1")
            .map_err(|error| error.to_string())?;
        Ok(TextMeasurementPolicy::host_display(
            identity,
            Arc::new(DirectHost(self.clone())),
            TextMeasurementPhase::ALL,
        ))
    }
}

impl HostTextMeasurer for DirectHost {
    fn measure(&self, request: HostTextMeasurementRequest<'_>) -> HostMeasurementResult {
        let query = MeasureQuery {
            operation: request.operation.external_code(),
            text: request.text.into(),
            style: request.style.clone(),
        };
        if query.validate().is_err() {
            return Ok(None);
        }
        Ok(match self.0.measure(&query) {
            MeasureReply::Fallback => None,
            MeasureReply::Length(length) => Some(HostTextMeasurement::Length(length)),
            MeasureReply::Metrics(metrics) => Some(HostTextMeasurement::Metrics(metrics)),
        })
    }
}

fn resolve_family(css: &str, installed: &[String]) -> String {
    for family in family_list(css) {
        if let Some(found) = installed
            .iter()
            .find(|name| name.eq_ignore_ascii_case(&family))
        {
            return found.clone();
        }
        let fallbacks: &[&str] = match family.to_ascii_lowercase().as_str() {
            "sans-serif" | "system-ui" | "ui-sans-serif" => {
                &["Arial", "Liberation Sans", "DejaVu Sans", "Noto Sans"]
            }
            "serif" | "ui-serif" => &[
                "Times New Roman",
                "Liberation Serif",
                "DejaVu Serif",
                "Noto Serif",
            ],
            "monospace" | "ui-monospace" => &[
                "Menlo",
                "DejaVu Sans Mono",
                "Liberation Mono",
                "Noto Sans Mono",
            ],
            _ => &[],
        };
        for fallback in fallbacks {
            if let Some(found) = installed
                .iter()
                .find(|name| name.eq_ignore_ascii_case(fallback))
            {
                return found.clone();
            }
        }
    }
    ".SystemUIFont".into()
}

fn family_list(css: &str) -> Vec<String> {
    let mut families = Vec::new();
    let mut current = String::new();
    let mut quote = None;
    let mut characters = css.chars();
    while let Some(character) = characters.next() {
        match character {
            '\\' => {
                if let Some(escaped) = characters.next() {
                    current.push(escaped);
                }
            }
            '\'' | '"' if quote == Some(character) => quote = None,
            '\'' | '"' if quote.is_none() => quote = Some(character),
            ',' if quote.is_none() => {
                families.push(current.trim().to_owned());
                current.clear();
            }
            _ => current.push(character),
        }
    }
    families.push(current.trim().to_owned());
    families
}

impl Measurer for NativeText {
    fn measure(&self, query: &MeasureQuery) -> MeasureReply {
        if query.validate().is_err() {
            return MeasureReply::Fallback;
        }
        let operation = TextMeasurementOperation::from_external_code(query.operation);
        if !matches!(
            operation,
            Some(
                TextMeasurementOperation::Measure
                    | TextMeasurementOperation::ComputedLength
                    | TextMeasurementOperation::CanvasMeasureTextWidth
            )
        ) {
            return MeasureReply::Fallback;
        }
        let requested = query.style.font_family.as_deref().unwrap_or("sans-serif");
        let family = resolve_family(requested, &self.names);
        if let Ok(mut resolved) = self.resolved.lock()
            && resolved.len() < 64
        {
            resolved.insert(format!("{requested} → {family}"));
        }
        let descriptor = style_font(&family, query);
        let mut width = 0_f64;
        let mut height = 0_f64;
        let mut line_count = 0;
        for line in query.text.split('\n') {
            let run = TextRun {
                len: line.len(),
                font: descriptor.clone(),
                ..Default::default()
            };
            let layout =
                self.system
                    .layout_line(line, px(query.style.font_size as f32), &[run], None);
            width = width.max(f64::from(f32::from(layout.width)));
            height += f64::from(f32::from(layout.ascent + layout.descent));
            line_count += 1;
        }
        if operation == Some(TextMeasurementOperation::Measure) {
            return MeasureReply::Metrics(TextMetrics {
                width,
                height,
                line_count,
            });
        }
        MeasureReply::Length(width)
    }
}

fn style_font(family: &str, query: &MeasureQuery) -> gpui::Font {
    let mut descriptor = font(family.to_owned());
    descriptor.weight = match query.style.font_weight.as_deref().unwrap_or("normal") {
        "bold" | "bolder" => FontWeight::BOLD,
        value => FontWeight(value.parse::<f32>().unwrap_or(400.).clamp(100., 900.)),
    };
    descriptor.style = match query.style.font_style.as_deref() {
        Some("italic" | "oblique") => FontStyle::Italic,
        _ => FontStyle::Normal,
    };
    descriptor
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn should_resolve_a_quoted_css_family_containing_a_comma() {
        // given
        let installed = vec!["Font, With Comma".into(), "Liberation Sans".into()];
        // when
        let family = resolve_family("Missing, 'Font, With Comma', sans-serif", &installed);
        // then
        assert_eq!(family, "Font, With Comma");
    }
    #[test]
    fn should_resolve_a_family_list_to_the_first_installed_family() {
        // given
        let installed = vec!["Liberation Sans".into(), "DejaVu Serif".into()];
        // when
        let family = resolve_family("trebuchet ms, 'liberation sans', serif", &installed);
        // then
        assert_eq!(family, "Liberation Sans");
    }
    #[test]
    fn should_resolve_generic_serif_to_a_native_serif_family() {
        // given
        let installed = vec!["DejaVu Sans".into(), "DejaVu Serif".into()];
        // when
        let family = resolve_family("Missing, serif", &installed);
        // then
        assert_eq!(family, "DejaVu Serif");
    }
}
