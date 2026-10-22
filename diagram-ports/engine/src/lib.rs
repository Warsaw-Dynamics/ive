use std::{fmt, time::Duration};

mod limits;
mod mermaid;
mod plantuml;
mod resources;
mod svg;

pub use merman::OperationControl as RenderControl;
pub use merman::svg::{
    HostFallbackReason, HostMeasurementResult, HostTextMeasurement, HostTextMeasurementError,
    HostTextMeasurementRequest, HostTextMeasurer, MeasurementProfileId, TextMeasurementOperation,
    TextMeasurementPhase, TextMeasurementPolicy, TextMeasurementProfileIdentity,
    TextMeasurementReport, TextMeasurementSource, TextMetrics, TextStyle, WrapMode,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Language {
    #[default]
    Mermaid,
    PlantUml,
}

#[derive(Debug, Clone)]
pub struct RenderLimits {
    pub source_bytes: usize,
    pub svg_bytes: usize,
    pub model_items: usize,
    pub nesting_depth: usize,
    pub layout_work: usize,
    pub timeout: Duration,
}

impl Default for RenderLimits {
    fn default() -> Self {
        Self {
            source_bytes: 256 * 1024,
            svg_bytes: 16 * 1024 * 1024,
            model_items: 10_000,
            nesting_depth: 64,
            layout_work: 5_000_000,
            timeout: Duration::from_secs(10),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RenderOptions {
    pub language: Language,
    pub limits: RenderLimits,
    pub control: RenderControl,
    pub text_measurement: TextMeasurementPolicy,
}

#[derive(Debug, Clone)]
pub struct RenderedDiagram {
    pub svg: String,
    pub language: Language,
    pub backend: &'static str,
    pub upstream: &'static str,
    pub family: String,
    pub diagnostics: Vec<String>,
    pub measurement: TextMeasurementReport,
    pub layout_work_units: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderError {
    UnsupportedLanguage(Language),
    InvalidOptions(String),
    SourceLimit { actual: usize, limit: usize },
    ExternalResource(String),
    UnsupportedLayout(String),
    UnsupportedFeature(String),
    InvalidSource(String),
    Cancelled(String),
    Backend(String),
    UnsafeSvg(String),
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedLanguage(language) => {
                write!(f, "diagram language {language:?} is unavailable")
            }
            Self::InvalidOptions(message) => write!(f, "invalid diagram options: {message}"),
            Self::SourceLimit { actual, limit } => write!(
                f,
                "diagram source has {actual} bytes, exceeding the {limit}-byte limit"
            ),
            Self::ExternalResource(value) => {
                write!(f, "diagram external resource is forbidden: {value}")
            }
            Self::UnsupportedLayout(value) => write!(
                f,
                "diagram layout {value:?} is unavailable in the native backend"
            ),
            Self::UnsupportedFeature(message) => {
                write!(f, "unsupported diagram feature: {message}")
            }
            Self::InvalidSource(message) => write!(f, "invalid diagram source: {message}"),
            Self::Cancelled(message) => write!(f, "diagram rendering cancelled: {message}"),
            Self::Backend(message) => write!(f, "native diagram rendering failed: {message}"),
            Self::UnsafeSvg(message) => write!(f, "diagram SVG rejected: {message}"),
        }
    }
}

impl std::error::Error for RenderError {}

/// Renders caller-owned source synchronously without reading files or resolving external resources.
/// Run outside the UI thread; deadlines and cancellation are cooperative.
pub fn render(source: &str, options: &RenderOptions) -> Result<RenderedDiagram, RenderError> {
    match options.language {
        Language::Mermaid => mermaid::render(source, options),
        Language::PlantUml => plantuml::render(source, options),
    }
}
