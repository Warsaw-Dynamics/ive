//! Rune's `toml` module over IVE's own toml crate: `toml::from_string(text)` and `toml::to_string(value)`, each
//! `Ok(..)` or `Err(reason)`.

use rune::{ContextError, Module, Value};

pub fn module() -> Result<Module, ContextError> {
    let mut module = Module::with_crate("toml")?;
    module.function("from_string", |text: String| from_string(&text)).build()?;
    module.function("to_string", |value: Value| to_string(&value)).build()?;
    Ok(module)
}

fn from_string(text: &str) -> Result<Value, String> {
    toml::from_str(text).map_err(|error| format!("toml::from_string: {}", error.message()))
}

/// Only an object makes a TOML document.
fn to_string(value: &Value) -> Result<String, String> {
    toml::to_string(value).map_err(|error| format!("toml::to_string: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_read_toml_into_rune_values_and_write_them_back() {
        // given
        let text = "name = \"ive\"\n\n[window]\nwidth = 1200\n";

        // when
        let value = from_string(text).unwrap();
        let written = to_string(&value).unwrap();

        // then
        let json = serde_json::to_value(&value).unwrap();
        assert_eq!(json, serde_json::json!({ "name": "ive", "window": { "width": 1200 } }));
        assert_eq!(written, text);
    }

    #[test]
    fn should_name_the_function_and_the_reason_when_toml_is_broken() {
        // given
        let text = "name = ";

        // when
        let read = from_string(text);

        // then
        assert!(read.as_ref().is_err_and(|error| error.starts_with("toml::from_string: ")), "{read:?}");
    }
}
