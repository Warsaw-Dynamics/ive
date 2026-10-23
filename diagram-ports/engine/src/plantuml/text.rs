//! PlantUML MIT StringBounder descent; shared bounded measured runs via Merman.
use super::{
    budget::{Budget, bound, invalid},
    creole::{self, Run, RunStyle},
    scene::Decoration,
    style::{Font, Preamble, Role},
};
use crate::{RenderError, TextMeasurementPhase, TextStyle};
use merman_render::{environment::RenderSession, text::TextMeasurer};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub(super) struct MeasuredRun {
    pub run: Run,
    pub advance: f64,
}
#[derive(Clone, Debug)]
pub(super) struct MeasuredLine {
    pub runs: Vec<MeasuredRun>,
    pub height: f64,
    pub baseline: f64,
}
#[derive(Clone, Debug)]
pub(super) struct Label {
    pub lines: Arc<[MeasuredLine]>,
    pub width: f64,
    pub height: f64,
}

pub(super) struct Context<'a> {
    pub session: &'a RenderSession,
    pub budget: &'a mut Budget,
    fonts: [Font; 5],
    cache: HashMap<(String, RunStyle, LabelMode), Label>,
    calls: usize,
    origins: &'a super::origin::Document,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
enum LabelMode {
    Display,
    Tree,
    Data,
    Document,
}
impl LabelMode {
    fn family(self) -> &'static str {
        match self {
            Self::Display => "sans-serif",
            Self::Document => "serif",
            Self::Tree | Self::Data => "Arial, Liberation Sans, DejaVu Sans, Noto Sans, sans-serif",
        }
    }
}

impl<'a> Context<'a> {
    pub fn new(session: &'a RenderSession, budget: &'a mut Budget) -> Self {
        Self {
            session,
            budget,
            fonts: Preamble::default().fonts,
            cache: HashMap::new(),
            calls: 0,
            origins: &super::origin::EMPTY,
        }
    }
    pub fn with_fonts(mut self, fonts: [Font; 5]) -> Self {
        self.fonts = fonts;
        self
    }
    pub fn with_origins(mut self, origins: &'a super::origin::Document) -> Self {
        self.origins = origins;
        self
    }
    pub fn label(&mut self, text: &str) -> Result<Label, RenderError> {
        self.styled_label(text, Decoration::Plain)
    }
    pub fn styled_label(
        &mut self,
        text: &str,
        decoration: Decoration,
    ) -> Result<Label, RenderError> {
        let mut base = self.base(Role::Element, LabelMode::Display);
        base.underline = decoration == Decoration::Underlined;
        base.font.italic |= decoration == Decoration::Italic;
        self.measure_label(text, base, LabelMode::Display)
    }
    pub fn data_label(&mut self, text: &str) -> Result<Label, RenderError> {
        self.measure_label(
            text,
            self.base(Role::Element, LabelMode::Data),
            LabelMode::Data,
        )
    }
    pub fn document_label(&mut self, text: &str) -> Result<Label, RenderError> {
        let mut base = self.base(Role::Element, LabelMode::Document);
        base.font.color = super::style::Color(0);
        self.measure_label(text, base, LabelMode::Document)
    }
    pub fn tree_label(&mut self, text: &str) -> Result<Label, RenderError> {
        self.measure_label(
            text,
            self.base(Role::Element, LabelMode::Tree),
            LabelMode::Tree,
        )
    }
    pub fn region_label(&mut self, text: &str, role: Role) -> Result<Label, RenderError> {
        self.measure_label(
            text,
            self.base(role, LabelMode::Display),
            LabelMode::Display,
        )
    }

    fn base(&self, role: Role, mode: LabelMode) -> RunStyle {
        let mut font = self.fonts[role.index()].clone();
        if font.family.is_empty() {
            font.family = mode.family().into();
        }
        RunStyle {
            font,
            underline: false,
            strike: false,
        }
    }
    fn measure_label(
        &mut self,
        text: &str,
        base: RunStyle,
        mode: LabelMode,
    ) -> Result<Label, RenderError> {
        bound("label bytes", text.len(), 16 * 1024)?;
        self.budget.charge(text.len() + 1)?;
        self.budget.text(text.len())?;
        let key = (text.to_owned(), base.clone(), mode);
        if let Some(label) = self.cache.get(&key) {
            return Ok(label.clone());
        }
        let parsed = if mode == LabelMode::Data {
            self.literal_lines(text, &base)?
        } else {
            creole::parse(text, base.clone(), self.budget)
                .map_err(|error| self.origins.label_error(text, error, self.budget))?
        };
        let mut lines = Vec::new();
        let mut width: f64 = 0.;
        let mut height = 0.;
        for line in parsed {
            self.budget.item()?;
            let (measured, line_width) = self.measure_line(line, &base)?;
            width = width.max(line_width);
            height += measured.height;
            lines.push(measured);
        }
        let label = Label {
            lines: lines.into(),
            width,
            height,
        };
        self.cache.insert(key, label.clone());
        Ok(label)
    }
    fn literal_lines(&mut self, text: &str, base: &RunStyle) -> Result<Vec<Vec<Run>>, RenderError> {
        let mut lines = Vec::new();
        for line in text.split('\n') {
            self.budget.item()?;
            self.budget.text(line.len())?;
            lines.push(if line.is_empty() {
                Vec::new()
            } else {
                vec![Run {
                    text: line.into(),
                    style: base.clone(),
                }]
            });
        }
        Ok(lines)
    }
    fn measure_line(
        &mut self,
        runs: Vec<Run>,
        base: &RunStyle,
    ) -> Result<(MeasuredLine, f64), RenderError> {
        let size = base.font.size;
        let mut line = MeasuredLine {
            runs: Vec::new(),
            height: f64::from(size) + 4.,
            baseline: 0.,
        };
        let mut width = 0.;
        for run in runs {
            bound("measurement requests", self.calls + 1, 4096)?;
            self.budget.checkpoint()?;
            let metrics = self
                .session
                .text_measurer(TextMeasurementPhase::Layout)
                .measure(&run.text, &font_style(&run.style.font));
            self.calls += 1;
            self.budget.checkpoint()?;
            if !metrics.width.is_finite()
                || !metrics.height.is_finite()
                || metrics.width < 0.
                || metrics.height < 0.
            {
                return Err(invalid(1, "invalid text measurement"));
            }
            width += metrics.width;
            line.height = line.height.max(metrics.height + 4.);
            line.runs.push(MeasuredRun {
                run,
                advance: metrics.width,
            });
        }
        if line.runs.is_empty() {
            self.budget.item()?;
            line.runs.push(MeasuredRun {
                run: Run {
                    text: String::new(),
                    style: base.clone(),
                },
                advance: 0.,
            });
        }
        // The bridge provides line boxes/advances, not native baselines or ink bounds.
        line.baseline = line.height - f64::from(size) / 4.5;
        Ok((line, width))
    }
}

fn font_style(font: &Font) -> TextStyle {
    TextStyle {
        font_family: Some(font.family.to_string()),
        font_size: f64::from(font.size),
        font_weight: Some(font.weight.to_string()),
        font_style: Some(if font.italic { "italic" } else { "normal" }.into()),
    }
}

// Lexical/style validation happens once under the render Context's shared budget.
pub(super) fn validate_label(text: &str, _line: usize) -> Result<(), RenderError> {
    bound("label bytes", text.len(), 16 * 1024)
}

pub(super) fn append_label(
    out: &mut String,
    part: &str,
    budget: &mut Budget,
) -> Result<(), RenderError> {
    bound(
        "label bytes",
        out.len().saturating_add(part.len()),
        16 * 1024,
    )?;
    budget.text(part.len())?;
    out.push_str(part);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        HostMeasurementResult, HostTextMeasurementRequest, HostTextMeasurer, MeasurementProfileId,
        RenderOptions, TextMeasurementPolicy, TextMeasurementProfileIdentity,
    };
    use merman_render::environment::RenderEnvironment;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    struct Count(Arc<AtomicUsize>);
    impl HostTextMeasurer for Count {
        fn measure(&self, _: HostTextMeasurementRequest<'_>) -> HostMeasurementResult {
            self.0.fetch_add(1, Ordering::Relaxed);
            Ok(None)
        }
    }

    #[test]
    fn should_bound_aggregate_body_and_chrome_measurement_requests_before_host_call() {
        // given
        let calls = Arc::new(AtomicUsize::new(0));
        let policy = TextMeasurementPolicy::host_display(
            TextMeasurementProfileIdentity::new(
                MeasurementProfileId::new("aggregate-run-test").unwrap(),
                "1",
            )
            .unwrap(),
            Arc::new(Count(calls.clone())),
            [TextMeasurementPhase::Layout],
        );
        let session = RenderEnvironment::deterministic()
            .with_text_measurement_policy(policy)
            .begin_session()
            .unwrap();
        let mut options = RenderOptions::default();
        options.limits.model_items = 100_000;
        let mut budget = Budget::new(&options);
        let mut context = Context::new(&session, &mut budget);
        for index in 0..2047 {
            context.label(&format!("**a{index}**//b{index}//")).unwrap();
        }
        context.label("last-body").unwrap();
        context.region_label("last-header", Role::Header).unwrap();
        // when
        let result = context.region_label("overflow-title", Role::Title);
        // then
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("measurement requests value 4097")
        );
        assert_eq!(calls.load(Ordering::Relaxed), 4096);
    }

    #[test]
    fn should_reject_oversized_label_before_scanning_or_retaining_it() {
        // given
        let session = RenderEnvironment::deterministic().begin_session().unwrap();
        let mut budget = Budget::new(&RenderOptions::default());
        let mut context = Context::new(&session, &mut budget);
        let text = "*".repeat(16 * 1024 + 1);
        // when
        let error = context.label(&text).unwrap_err();
        // then
        assert!(error.to_string().contains("label bytes"));
        assert_eq!(context.budget.work, 0);
        assert_eq!(context.calls, 0);
        assert!(context.cache.is_empty());
    }

    #[test]
    fn should_coalesce_equal_styles_and_keep_color_only_cache_changes() {
        // given
        let session = RenderEnvironment::deterministic().begin_session().unwrap();
        let mut budget = Budget::new(&RenderOptions::default());
        let mut context = Context::new(&session, &mut budget);
        let mut second = context.base(Role::Element, LabelMode::Display);
        second.font.color = super::super::style::Color(0xff0000);
        let first = context.label("**same**<b>same</b>").unwrap();
        // when
        let red = context
            .measure_label("**same**<b>same</b>", second, LabelMode::Display)
            .unwrap();
        // then
        assert_eq!(first.lines[0].runs.len(), 1);
        assert_eq!(first.lines[0].runs[0].run.text, "samesame");
        assert_eq!(context.calls, 2);
        assert_eq!(red.lines[0].runs[0].run.style.font.color.0, 0xff0000);
        assert_eq!(first.lines[0].runs[0].run.style.font.color.0, 0x172033);
    }

    #[test]
    fn should_bound_tokens_before_measuring_many_short_runs() {
        // given
        let session = RenderEnvironment::deterministic().begin_session().unwrap();
        let mut options = RenderOptions::default();
        options.limits.model_items = 100_000;
        let mut budget = Budget::new(&options);
        let mut context = Context::new(&session, &mut budget);
        let text = "**x**a".repeat(1025);
        // when
        let error = context.label(&text).unwrap_err();
        // then
        assert!(error.to_string().contains("label tokens"));
        assert_eq!(context.calls, 0);
    }

    #[test]
    fn should_bound_marker_activation_and_escaped_punctuation_with_linear_work() {
        // given
        for count in [1000, 2000, 4000] {
            let session = RenderEnvironment::deterministic().begin_session().unwrap();
            let mut options = RenderOptions::default();
            options.limits.layout_work = count * 30;
            let mut budget = Budget::new(&options);
            let mut context = Context::new(&session, &mut budget);
            let text = format!("{} **bold**", "~*".repeat(count));
            // when
            let label = context.label(&text).unwrap();
            // then
            assert_eq!(label.lines[0].runs.len(), 2);
            assert_eq!(context.calls, 2);
            assert!(context.budget.work < count * 30);
        }
    }
}
