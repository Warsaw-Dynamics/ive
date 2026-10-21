//! A static server's Rune hook: `pub fn request(req)` sees each request before the files do.

use std::path::Path;
use std::sync::Arc;

use rune::runtime::budget;
use rune::{Context, Diagnostics, Source, Sources, Vm};
use serde::Serialize;

use crate::script::engine::{compile_errors, is_out_of_budget};

/// Instructions one call of `request` may run, as `init.rn`'s functions.
const HOOK_BUDGET: usize = 50_000_000;

/// The compiled hook, with the executor its async functions (`fs`, `time`) run on.
pub struct Hook {
    name: String,
    vm: Vm,
    executor: tokio::runtime::Runtime,
}

/// What the hook gets: `#{ method, path, query, headers }`.
#[derive(Debug, Clone, Serialize)]
pub struct HookRequest {
    pub method: String,
    pub path: String,
    pub query: String,
    #[serde(serialize_with = "as_table")]
    pub headers: Vec<(String, String)>,
}

/// What the hook said: go on, go on with another path, or this answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HookAnswer {
    Continue,
    Rewrite(String),
    Respond { status: u16, headers: Vec<(String, String)>, body: String },
}

impl Hook {
    /// Compiles `path` against Rune's standard modules and `fs`, `json`, `time`.
    pub fn load(path: &Path) -> Result<Self, String> {
        let name = path.file_name().map_or_else(|| path.display().to_string(), |name| name.to_string_lossy().into_owned());
        let text = std::fs::read_to_string(path).map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        let mut context = Context::with_default_modules().map_err(|error| error.to_string())?;
        for module in [rune_modules::fs::module(false), rune_modules::json::module(false), rune_modules::time::module(false)] {
            context.install(module.map_err(|error| error.to_string())?).map_err(|error| error.to_string())?;
        }
        let runtime = Arc::new(context.runtime().map_err(|error| error.to_string())?);
        let mut sources = Sources::new();
        sources.insert(Source::with_path(&name, text, path).map_err(|error| error.to_string())?).map_err(|error| error.to_string())?;
        let mut diagnostics = Diagnostics::new();
        let unit = rune::prepare(&mut sources).with_context(&context).with_diagnostics(&mut diagnostics).build();
        let unit = match unit {
            Ok(unit) if !diagnostics.has_error() => unit,
            _ => return Err(compile_errors(&diagnostics, &sources, &name)),
        };
        let executor = tokio::runtime::Builder::new_current_thread().enable_all().build().map_err(|error| error.to_string())?;
        Ok(Self { name, vm: Vm::new(runtime, Arc::new(unit)), executor })
    }

    /// Calls `request(req)` within [`HOOK_BUDGET`] instructions.
    pub fn request(&mut self, request: &HookRequest) -> Result<HookAnswer, String> {
        let json = serde_json::to_value(request).map_err(|error| error.to_string())?;
        let value: rune::Value = serde_json::from_value(json).map_err(|error| error.to_string())?;
        let call = budget::with(HOOK_BUDGET, self.vm.async_call(["request"], (value,)));
        let returned = self.executor.block_on(call).map_err(|error| {
            let error = error.to_string();
            if is_out_of_budget(&error) {
                return format!("{}: request ran over {HOOK_BUDGET} instructions and was stopped; does it loop forever?", self.name);
            }
            format!("{}: {error}", self.name)
        })?;
        let answer = serde_json::to_value(&returned).map_err(|error| format!("{}: request returned what IVE cannot read: {error}", self.name))?;
        answer_of(&answer).map_err(|error| format!("{}: {error}", self.name))
    }
}

/// `()` goes on, `#{ path }` rewrites, `#{ status, headers, body }` answers.
fn answer_of(value: &serde_json::Value) -> Result<HookAnswer, String> {
    let Some(object) = value.as_object() else {
        return match value {
            serde_json::Value::Null => Ok(HookAnswer::Continue),
            other => Err(format!("request returned {other}, not () or an object")),
        };
    };
    if let Some(status) = object.get("status") {
        let status = status.as_u64().and_then(|status| u16::try_from(status).ok()).filter(|status| (100..600).contains(status)).ok_or("status must be 100-599")?;
        let headers = object
            .get("headers")
            .and_then(serde_json::Value::as_object)
            .map(|headers| headers.iter().map(|(name, value)| (name.clone(), value.as_str().map_or_else(|| value.to_string(), str::to_owned))).collect())
            .unwrap_or_default();
        let body = object.get("body").and_then(serde_json::Value::as_str).unwrap_or_default().to_owned();
        return Ok(HookAnswer::Respond { status, headers, body });
    }
    match object.get("path").and_then(serde_json::Value::as_str) {
        Some(path) => Ok(HookAnswer::Rewrite(path.to_owned())),
        None => Err("request returned an object with neither status nor path".to_owned()),
    }
}

fn as_table<S: serde::Serializer>(headers: &[(String, String)], serializer: S) -> Result<S::Ok, S::Error> {
    serializer.collect_map(headers.iter().map(|(name, value)| (name, value)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hook(text: &str) -> (tempfile::TempDir, Hook) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("serve.rn");
        std::fs::write(&path, text).unwrap();
        let hook = Hook::load(&path).unwrap();
        (dir, hook)
    }

    fn get(path: &str) -> HookRequest {
        HookRequest { method: "GET".to_owned(), path: path.to_owned(), query: String::new(), headers: vec![("accept".to_owned(), "*/*".to_owned())] }
    }

    #[test]
    fn should_answer_directly_from_hook() {
        // given
        let (_dir, mut hook) = hook("pub fn request(req) { if req.path == \"/health\" { #{ status: 200, headers: #{ \"Content-Type\": \"text/plain\" }, body: \"ok\" } } else { () } }");

        // when
        let answers = [hook.request(&get("/health")), hook.request(&get("/other"))];

        // then
        let health = HookAnswer::Respond { status: 200, headers: vec![("Content-Type".to_owned(), "text/plain".to_owned())], body: "ok".to_owned() };
        assert_eq!(answers, [Ok(health), Ok(HookAnswer::Continue)]);
    }

    #[test]
    fn should_rewrite_from_hook() {
        // given
        let (_dir, mut hook) = hook("pub fn request(req) { if req.path.starts_with(\"/v2/\") { #{ path: req.path.replace(\"/v2/\", \"/\") } } else { () } }");

        // when
        let answer = hook.request(&get("/v2/index.html"));

        // then
        assert_eq!(answer, Ok(HookAnswer::Rewrite("/index.html".to_owned())));
    }

    #[test]
    fn should_run_the_readme_example() {
        // given
        let (_dir, mut hook) = hook(r#"pub fn request(req) {                           // req: #{ method, path, query, headers }
    if req.path == "/health" {
        #{ status: 200, headers: #{ "Content-Type": "text/plain" }, body: "ok" }   // answer
    } else if req.path.starts_with("/v2/") {
        #{ path: req.path.replace("/v2/", "/") }                                    // rewrite, go on
    } else {
        ()                                                                          // go on
    }
}"#);

        // when
        let answers = [hook.request(&get("/v2/a.html")), hook.request(&get("/a.html"))];

        // then
        assert_eq!(answers, [Ok(HookAnswer::Rewrite("/a.html".to_owned())), Ok(HookAnswer::Continue)]);
    }

    #[test]
    fn should_stop_a_runaway_hook() {
        // given
        let (_dir, mut hook) = hook("pub fn request(req) { loop {} }");

        // when
        let answer = hook.request(&get("/"));

        // then
        assert!(answer.as_ref().is_err_and(|error| error.contains("50000000 instructions")), "{answer:?}");
    }

    #[test]
    fn should_report_a_compile_error_with_its_place() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("serve.rn");
        std::fs::write(&path, "pub fn request(req) { let }").unwrap();

        // when
        let loaded = Hook::load(&path);

        // then
        assert!(loaded.as_ref().is_err_and(|error| error.starts_with("serve.rn:1:")), "{:?}", loaded.err());
    }
}
