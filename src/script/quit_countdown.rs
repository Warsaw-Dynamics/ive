use gpui::Global;
use serde_json::Value;

const MAX_SECONDS: u64 = 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuitCountdown(u8);

impl Global for QuitCountdown {}

impl QuitCountdown {
    pub fn seconds(self) -> u8 {
        self.0
    }
}

pub fn parse(value: &rune::Value) -> Result<QuitCountdown, String> {
    let json = serde_json::to_value(value).map_err(|error| format!("quit_countdown: {error}"))?;
    match json.as_u64().and_then(|seconds| u8::try_from(seconds).ok()) {
        Some(seconds) if (1..=MAX_SECONDS).contains(&u64::from(seconds)) => Ok(QuitCountdown(seconds)),
        _ => Err(format!("quit_countdown must be whole seconds from 1 to {MAX_SECONDS}, not {}", describe(&json))),
    }
}

fn describe(json: &Value) -> String {
    match json {
        Value::Null => "unset".to_owned(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn value(json: serde_json::Value) -> rune::Value {
        serde_json::from_value(json).unwrap()
    }

    #[test]
    fn should_take_whole_seconds_from_one_to_sixty() {
        // given
        let values = [value(serde_json::json!(1)), value(serde_json::json!(60))];

        // when
        let seconds = values.map(|value| parse(&value).map(QuitCountdown::seconds));

        // then
        assert_eq!(seconds, [Ok(1), Ok(60)]);
    }

    #[test]
    fn should_reject_seconds_outside_one_to_sixty_and_other_values() {
        // given
        let values = [serde_json::json!(0), serde_json::json!(61), serde_json::json!(-5), serde_json::json!(2.5), serde_json::json!("10")];

        // when
        let errors: Vec<String> = values.iter().map(|json| parse(&value(json.clone())).unwrap_err()).collect();

        // then
        assert_eq!(errors[0], "quit_countdown must be whole seconds from 1 to 60, not 0");
        assert!(errors.iter().all(|error| error.starts_with("quit_countdown must be whole seconds from 1 to 60, not ")), "{errors:?}");
    }
}
