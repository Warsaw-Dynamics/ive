use super::protocol;
use diagram_engine::{
    HostMeasurementResult, HostTextMeasurement, HostTextMeasurementRequest, HostTextMeasurer,
    MeasurementProfileId, TextMeasurementOperation, TextMeasurementPhase, TextMeasurementPolicy,
    TextMeasurementProfileIdentity, TextMetrics, TextStyle,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MeasureQuery {
    pub operation: i32,
    pub text: String,
    pub style: TextStyle,
}
impl MeasureQuery {
    pub fn validate(&self) -> Result<(), String> {
        if self.text.len() > 16 * 1024
            || self
                .style
                .font_family
                .as_ref()
                .is_some_and(|family| family.len() > 1024)
            || !self.style.font_size.is_finite()
            || !(1.0..=256.0).contains(&self.style.font_size)
        {
            return Err("native text measurement request exceeds text/font limits".into());
        }
        Ok(())
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub enum MeasureReply {
    Fallback,
    Length(f64),
    Metrics(TextMetrics),
}
pub trait Measurer: Send + Sync {
    fn measure(&self, query: &MeasureQuery) -> MeasureReply;
}
struct Bridge(Mutex<()>);
impl HostTextMeasurer for Bridge {
    fn measure(&self, request: HostTextMeasurementRequest<'_>) -> HostMeasurementResult {
        if !matches!(
            request.operation,
            TextMeasurementOperation::Measure
                | TextMeasurementOperation::ComputedLength
                | TextMeasurementOperation::CanvasMeasureTextWidth
        ) {
            return Ok(None);
        }
        let query = MeasureQuery {
            operation: request.operation.external_code(),
            text: request.text.into(),
            style: request.style.clone(),
        };
        if query.validate().is_err() {
            return Ok(None);
        }
        let _lock = self.0.lock().map_err(|_| {
            diagram_engine::HostTextMeasurementError::new(
                "native measurement transport lock poisoned",
            )
        })?;
        exchange(&query).map_err(diagram_engine::HostTextMeasurementError::new)
    }
}
fn exchange(query: &MeasureQuery) -> Result<Option<HostTextMeasurement>, String> {
    protocol::write_json(&mut std::io::stdout().lock(), protocol::MEASURE, query)?;
    let reply: MeasureReply = protocol::read_json(
        &mut std::io::stdin().lock(),
        protocol::ANSWER,
        &mut protocol::Budget::new(protocol::HEADER_LIMIT),
    )?;
    Ok(match reply {
        MeasureReply::Fallback => None,
        MeasureReply::Length(length) => Some(HostTextMeasurement::Length(length)),
        MeasureReply::Metrics(metrics) => Some(HostTextMeasurement::Metrics(metrics)),
    })
}
pub fn policy() -> Result<TextMeasurementPolicy, String> {
    let profile =
        MeasurementProfileId::new("pgui-native-unwrapped").map_err(|error| error.to_string())?;
    let identity = TextMeasurementProfileIdentity::new(profile, "f5d5fa2-v1")
        .map_err(|error| error.to_string())?;
    Ok(TextMeasurementPolicy::host_display(
        identity,
        Arc::new(Bridge(Mutex::new(()))),
        TextMeasurementPhase::ALL,
    ))
}
