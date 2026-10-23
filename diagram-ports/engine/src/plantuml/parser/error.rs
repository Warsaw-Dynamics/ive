use super::super::origin::SourceSpan;
use crate::RenderError;
use std::ops::Range;

#[derive(Debug)]
pub(in crate::plantuml) enum FailureKind {
    Syntax(&'static str),
    Unsupported(&'static str),
    Terminal(RenderError),
}

#[derive(Debug)]
pub(in crate::plantuml) struct ParseFailure {
    pub span: Range<usize>,
    pub kind: FailureKind,
}

impl ParseFailure {
    pub fn syntax(span: Range<usize>, detail: &'static str) -> Self {
        Self {
            span,
            kind: FailureKind::Syntax(detail),
        }
    }

    pub fn unsupported(span: Range<usize>, detail: &'static str) -> Self {
        Self {
            span,
            kind: FailureKind::Unsupported(detail),
        }
    }

    pub fn terminal(error: RenderError) -> Self {
        Self {
            span: 0..0,
            kind: FailureKind::Terminal(error),
        }
    }

    pub fn into_authored(self, authored: SourceSpan) -> RenderError {
        let relative = self.span.start.saturating_sub(authored.start_byte);
        let detail = |message| {
            format!(
                "PlantUML at line {}, column {}: {message} (definition bytes {}..{})",
                authored.line,
                authored.column + relative,
                self.span.start,
                self.span.end
            )
        };
        match self.kind {
            FailureKind::Syntax(message) => RenderError::InvalidSource(detail(message)),
            FailureKind::Unsupported(message) => RenderError::UnsupportedFeature(detail(message)),
            FailureKind::Terminal(error) => error,
        }
    }
}
