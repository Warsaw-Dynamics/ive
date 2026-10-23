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

    pub fn into_line(self, line: usize) -> RenderError {
        match self.kind {
            FailureKind::Syntax(message) => super::super::budget::invalid(line, message),
            FailureKind::Unsupported(message) => super::super::budget::unsupported(line, message),
            FailureKind::Terminal(error) => error,
        }
    }

    pub fn into_source(
        self,
        source: &str,
        budget: &mut super::super::budget::Budget,
    ) -> RenderError {
        if matches!(self.kind, FailureKind::Terminal(_)) {
            return self.into_line(1);
        }
        if let Err(error) = budget.charge(source.len()) {
            return error;
        }
        let start = self.span.start.min(source.len());
        let prefix = &source[..start];
        let line = prefix.bytes().filter(|&byte| byte == b'\n').count() + 1;
        let column = prefix.rfind('\n').map_or(start + 1, |last| start - last);
        let message = |detail| {
            format!(
                "PlantUML at line {line}, column {column}: {detail} (authored bytes {}..{})",
                self.span.start, self.span.end
            )
        };
        match self.kind {
            FailureKind::Syntax(detail) => RenderError::InvalidSource(message(detail)),
            FailureKind::Unsupported(detail) => RenderError::UnsupportedFeature(message(detail)),
            FailureKind::Terminal(error) => error,
        }
    }

    pub fn into_document(
        self,
        document: &super::super::origin::Document,
        budget: &mut super::super::budget::Budget,
    ) -> RenderError {
        document.map_failure(self, budget)
    }
}

pub(super) fn generated_failure<T>(
    error: lalrpop_util::ParseError<usize, T, ParseFailure>,
    eof: usize,
    detail: &'static str,
) -> ParseFailure {
    use lalrpop_util::ParseError;
    match error {
        ParseError::User { error } => error,
        ParseError::InvalidToken { location } => ParseFailure::syntax(location..location, detail),
        ParseError::UnrecognizedEof { .. } => ParseFailure::syntax(eof..eof, detail),
        ParseError::UnrecognizedToken {
            token: (start, _, end),
            ..
        }
        | ParseError::ExtraToken {
            token: (start, _, end),
        } => ParseFailure::syntax(start..end, detail),
    }
}
