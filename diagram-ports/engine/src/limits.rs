use crate::{RenderError, RenderLimits};
use merman::svg::{RenderResourcePolicy, ResourceLimitId};

pub(crate) fn validate_source(source: &str, limits: &RenderLimits) -> Result<(), RenderError> {
    if source.len() > limits.source_bytes {
        return Err(RenderError::SourceLimit {
            actual: source.len(),
            limit: limits.source_bytes,
        });
    }
    if limits.timeout.is_zero() || limits.timeout.as_secs_f64() > 120.0 {
        return Err(RenderError::InvalidOptions(format!(
            "timeout {:?} must be greater than zero and at most 120 seconds",
            limits.timeout
        )));
    }
    Ok(())
}

pub(crate) fn resource_policy(limits: &RenderLimits) -> Result<RenderResourcePolicy, RenderError> {
    let settings = [
        (
            ResourceLimitId::MaxSourceBytes,
            limits.source_bytes,
            16 * 1024 * 1024,
        ),
        (
            ResourceLimitId::MaxSvgBytes,
            limits.svg_bytes,
            64 * 1024 * 1024,
        ),
        (ResourceLimitId::MaxModelItems, limits.model_items, 100_000),
        (
            ResourceLimitId::MaxModelNestingDepth,
            limits.nesting_depth,
            128,
        ),
        (
            ResourceLimitId::MaxLayoutWorkUnits,
            limits.layout_work,
            100_000_000,
        ),
    ];
    settings.into_iter().try_fold(
        RenderResourcePolicy::interactive(),
        |policy, (id, value, maximum)| {
            if value == 0 || value > maximum {
                return Err(RenderError::InvalidOptions(format!(
                    "{id:?} value {value} must be in 1..={maximum}"
                )));
            }
            policy
                .with_limit(id, value)
                .map_err(|error| RenderError::InvalidOptions(error.to_string()))
        },
    )
}
