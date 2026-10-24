use crate::{GraphLayout, RenderControl, RenderError, RenderLimits, RenderOptions};

pub(super) struct Budget {
    pub limits: RenderLimits,
    pub control: RenderControl,
    pub graph_layout: GraphLayout,
    pub work: usize,
    items: usize,
    text_bytes: usize,
}

impl Budget {
    pub fn new(options: &RenderOptions) -> Self {
        Self {
            limits: options.limits.clone(),
            control: crate::child_control(options),
            graph_layout: options.graph_layout,
            work: 0,
            items: 0,
            text_bytes: 0,
        }
    }

    pub fn checkpoint(&self) -> Result<(), RenderError> {
        self.control
            .checkpoint()
            .map_err(|e| RenderError::Cancelled(e.to_string()))
    }

    pub fn charge(&mut self, units: usize) -> Result<(), RenderError> {
        self.checkpoint()?;
        let next = self
            .work
            .checked_add(units)
            .ok_or_else(|| exceeded("work arithmetic", usize::MAX, self.limits.layout_work))?;
        bound("work units", next, self.limits.layout_work)?;
        self.work = next;
        Ok(())
    }

    pub fn item(&mut self) -> Result<(), RenderError> {
        bound("model items", self.items + 1, self.limits.model_items)?;
        self.items += 1;
        self.charge(1)
    }

    pub fn depth(&self, depth: usize) -> Result<(), RenderError> {
        bound("nesting depth", depth, self.limits.nesting_depth)
    }

    pub fn text(&mut self, bytes: usize) -> Result<(), RenderError> {
        let limit = self
            .limits
            .source_bytes
            .saturating_mul(8)
            .min(self.limits.svg_bytes);
        let next = self.text_bytes.saturating_add(bytes);
        bound("expanded model text bytes", next, limit)?;
        self.text_bytes = next;
        self.charge(bytes)
    }
}

pub(super) fn bound(resource: &str, actual: usize, limit: usize) -> Result<(), RenderError> {
    if actual > limit {
        return Err(exceeded(resource, actual, limit));
    }
    Ok(())
}

fn exceeded(resource: &str, actual: usize, limit: usize) -> RenderError {
    RenderError::Backend(format!(
        "PlantUML {resource} value {actual} exceeds limit {limit}"
    ))
}

pub(super) fn invalid(line: usize, message: impl std::fmt::Display) -> RenderError {
    RenderError::InvalidSource(format!("PlantUML at line {line}, column 1: {message}"))
}

pub(super) fn unsupported(line: usize, feature: &str) -> RenderError {
    RenderError::UnsupportedFeature(format!("PlantUML at line {line}, column 1: {feature:?}"))
}
