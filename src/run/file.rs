//! `.run.toml`: a folder's run configurations, parsed and checked; nothing in it runs here.

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::marker::PhantomData;
use std::ops::Range;
use std::path::Path;

use serde::Deserialize;
use serde::de::{Deserializer, MapAccess, Visitor};
use toml::Spanned;

pub const RUN_FILE: &str = ".run.toml";

/// A folder's `.run.toml`, configurations in file order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunFile {
    pub vars: Vec<VarDecl>,
    pub configs: Vec<Config>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VarDecl {
    pub name: String,
    pub description: String,
    pub secret: bool,
    pub default: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub key: String,
    pub name: String,
    pub kind: ConfigKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigKind {
    Run(RunSpec),
    Serve(ServeSpec),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunSpec {
    pub command: String,
    pub cwd: Option<String>,
    pub env: Vec<(String, String)>,
    pub before: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServeSpec {
    pub root: String,
    /// a number, or text with variables checked once substituted
    pub port: String,
    pub live_reload: bool,
    pub fallback: Option<String>,
    pub rewrites: Vec<Rewrite>,
    pub script: Option<String>,
    /// the page the link to the running server goes to, `/` by default
    pub open: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rewrite {
    pub from: String,
    pub to: String,
}

/// Why a `.run.toml` does not load, at a 1-based line and column.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunFileError {
    pub line: usize,
    pub column: usize,
    pub message: String,
}

impl fmt::Display for RunFileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{RUN_FILE}:{}:{}: {}", self.line, self.column, self.message)
    }
}

/// A `.run.toml` larger than this is refused rather than read into the tree.
const MAX_BYTES: u64 = 64 * 1024;
/// Variables every file has: `USER`, `HOME`, `REPO` (the checkout), `DIR` (the folder of `.run.toml`).
pub const BUILTINS: [&str; 4] = ["USER", "HOME", "REPO", "DIR"];

/// Reads `folder`'s `.run.toml`; `None` when it has none.
pub fn load(folder: &Path) -> Option<Result<RunFile, RunFileError>> {
    let path = folder.join(RUN_FILE);
    let metadata = std::fs::metadata(&path).ok().filter(|metadata| metadata.is_file())?;
    if metadata.len() > MAX_BYTES {
        return Some(Err(RunFileError { line: 1, column: 1, message: "the file is over 64 KiB".to_owned() }));
    }
    Some(match std::fs::read_to_string(&path) {
        Ok(text) => parse(&text),
        Err(error) => Err(RunFileError { line: 1, column: 1, message: format!("cannot read it: {error}") }),
    })
}

/// Parses and checks `text`: keys, `before`, variables, ports. The first problem found is the error.
pub fn parse(text: &str) -> Result<RunFile, RunFileError> {
    let raw: RawFile = toml::from_str(text).map_err(|error| {
        let start = error.span().map_or(0, |span| span.start);
        at(text, start, error.message().trim_end().to_owned())
    })?;
    Checker { text, raw: &raw }.check()?;
    Ok(build(raw))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawFile {
    #[serde(default)]
    vars: Entries<RawVar>,
    #[serde(default)]
    run: Entries<RawRun>,
    #[serde(default)]
    serve: Entries<RawServe>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawVar {
    #[serde(default)]
    description: String,
    #[serde(default)]
    secret: bool,
    default: Option<Spanned<String>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRun {
    name: Option<String>,
    command: Spanned<String>,
    cwd: Option<Spanned<String>>,
    #[serde(default)]
    env: Entries<Spanned<String>>,
    before: Option<Spanned<Vec<String>>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawServe {
    name: Option<String>,
    root: Spanned<String>,
    port: Spanned<Port>,
    live_reload: Option<bool>,
    fallback: Option<String>,
    #[serde(default)]
    rewrites: Vec<Rewrite>,
    script: Option<String>,
    open: Option<Spanned<String>>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Port {
    Number(i64),
    Text(String),
}

/// A table's entries with where their keys are, so errors and the file order can use it.
struct Entries<T>(Vec<(Spanned<String>, T)>);

impl<T> Entries<T> {
    /// The parser gives tables sorted by key; the file's order is where the keys are.
    fn in_file_order(mut self) -> impl Iterator<Item = (Spanned<String>, T)> {
        self.0.sort_by_key(|(key, _)| key.span().start);
        self.0.into_iter()
    }
}

impl<T> Default for Entries<T> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Entries<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct EntriesVisitor<T>(PhantomData<T>);

        impl<'de, T: Deserialize<'de>> Visitor<'de> for EntriesVisitor<T> {
            type Value = Entries<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a table")
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut entries = Vec::new();
                while let Some(entry) = map.next_entry::<Spanned<String>, T>()? {
                    entries.push(entry);
                }
                Ok(Entries(entries))
            }
        }

        deserializer.deserialize_map(EntriesVisitor(PhantomData))
    }
}

/// The checks serde cannot make, each failing at the place in `text` it is about.
struct Checker<'a> {
    text: &'a str,
    raw: &'a RawFile,
}

impl Checker<'_> {
    fn check(&self) -> Result<(), RunFileError> {
        self.check_keys()?;
        self.check_vars()?;
        self.check_configs()?;
        self.check_before()
    }

    fn error(&self, span: Range<usize>, message: String) -> RunFileError {
        at(self.text, span.start, message)
    }

    fn check_keys(&self) -> Result<(), RunFileError> {
        let keys = self.raw.vars.0.iter().map(|(key, _)| key).chain(self.config_keys());
        if let Some(bad) = keys.clone().find(|key| !is_valid_key(key.get_ref())) {
            return Err(self.error(bad.span(), format!("key \"{}\" must be 1-64 of a-z, 0-9, _ and -", bad.get_ref())));
        }
        let mut configs: Vec<&Spanned<String>> = self.config_keys().collect();
        configs.sort_by_key(|key| key.span().start);
        let mut seen = HashSet::new();
        match configs.into_iter().find(|key| !seen.insert(key.get_ref().as_str())) {
            Some(twice) => Err(self.error(twice.span(), format!("key \"{}\" is used twice", twice.get_ref()))),
            None => Ok(()),
        }
    }

    fn config_keys(&self) -> impl Iterator<Item = &Spanned<String>> + Clone {
        self.raw.run.0.iter().map(|(key, _)| key).chain(self.raw.serve.0.iter().map(|(key, _)| key))
    }

    fn check_vars(&self) -> Result<(), RunFileError> {
        for (name, var) in &self.raw.vars.0 {
            match &var.default {
                Some(default) if var.secret => {
                    return Err(self.error(default.span(), format!("secret variable {} cannot have a default", name.get_ref())));
                }
                Some(default) => self.check_text(default, Use::Plain)?,
                None => {}
            }
        }
        Ok(())
    }

    fn check_configs(&self) -> Result<(), RunFileError> {
        for (_, run) in &self.raw.run.0 {
            self.check_text(&run.command, Use::Plain)?;
            if let Some(cwd) = &run.cwd {
                self.check_text(cwd, Use::Plain)?;
            }
            for (_, value) in &run.env.0 {
                self.check_text(value, Use::Env)?;
            }
        }
        for (_, serve) in &self.raw.serve.0 {
            self.check_text(&serve.root, Use::Plain)?;
            self.check_port(&serve.port)?;
            if let Some(open) = serve.open.as_ref().filter(|open| !open.get_ref().starts_with('/')) {
                return Err(self.error(open.span(), format!("open must be a path starting with /, got {:?}", open.get_ref())));
            }
        }
        Ok(())
    }

    fn check_port(&self, port: &Spanned<Port>) -> Result<(), RunFileError> {
        let number = match port.get_ref() {
            Port::Number(number) => *number,
            Port::Text(text) if text.contains("${") => return self.check_text(&Spanned::new(port.span(), text.clone()), Use::Plain),
            Port::Text(text) => text.trim().parse().unwrap_or(0),
        };
        if (1024..=65535).contains(&number) {
            return Ok(());
        }
        let shown = match port.get_ref() {
            Port::Number(number) => number.to_string(),
            Port::Text(text) => format!("{text:?}"),
        };
        Err(self.error(port.span(), format!("port must be 1024-65535, got {shown}")))
    }

    /// Every `${name}` in `value` names a variable of the file or a built-in; a secret one only where `used` allows.
    fn check_text(&self, value: &Spanned<String>, used: Use) -> Result<(), RunFileError> {
        for name in references(value.get_ref()) {
            let declared = self.raw.vars.0.iter().find(|(key, _)| key.get_ref() == name);
            match declared {
                None if BUILTINS.contains(&name) => {}
                None => return Err(self.error(value.span(), format!("unknown variable {name}"))),
                Some((_, var)) if var.secret && used == Use::Plain => {
                    return Err(self.error(value.span(), format!("secret variable {name} may only be used in env")));
                }
                Some(_) => {}
            }
        }
        Ok(())
    }

    fn check_before(&self) -> Result<(), RunFileError> {
        let runs: HashMap<&str, &RawRun> = self.raw.run.0.iter().map(|(key, run)| (key.get_ref().as_str(), run)).collect();
        for (_, run) in &self.raw.run.0 {
            let Some(before) = &run.before else { continue };
            for step in before.get_ref() {
                if runs.contains_key(step.as_str()) {
                    continue;
                }
                let serves = self.raw.serve.0.iter().any(|(key, _)| key.get_ref() == step);
                let message = if serves {
                    format!("before names \"{step}\", a static server, which never ends")
                } else {
                    format!("before names \"{step}\", which this file does not define")
                };
                return Err(self.error(before.span(), message));
            }
        }
        for (key, run) in &self.raw.run.0 {
            if let (Some(cycle), Some(before)) = (cycle_from(key.get_ref(), &runs), &run.before) {
                return Err(self.error(before.span(), format!("before forms a cycle: {}", cycle.join(" → "))));
            }
        }
        Ok(())
    }
}

/// Where a variable is used: secrets may only reach a program through its environment.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Use {
    Plain,
    Env,
}

/// The keys from `start` back to `start` through `before`, when there is such a loop.
fn cycle_from(start: &str, runs: &HashMap<&str, &RawRun>) -> Option<Vec<String>> {
    fn visit(key: &str, start: &str, runs: &HashMap<&str, &RawRun>, path: &mut Vec<String>, seen: &mut HashSet<String>) -> bool {
        let steps = runs.get(key).and_then(|run| run.before.as_ref()).map(|before| before.get_ref().clone()).unwrap_or_default();
        for step in steps {
            path.push(step.clone());
            if step == start || (seen.insert(step.clone()) && visit(&step, start, runs, path, seen)) {
                return true;
            }
            path.pop();
        }
        false
    }
    let mut path = vec![start.to_owned()];
    visit(start, start, runs, &mut path, &mut HashSet::new()).then_some(path)
}

/// The variable names `${name}` in `text` uses; `$${` is a literal `${`.
pub fn references(text: &str) -> Vec<&str> {
    let mut names = Vec::new();
    let mut rest = text;
    while let Some(found) = rest.find("${") {
        let escaped = rest[..found].ends_with('$');
        let after = &rest[found + 2..];
        let Some(end) = after.find('}') else { break };
        if !escaped {
            names.push(&after[..end]);
        }
        rest = &after[end + 1..];
    }
    names
}

fn is_valid_key(key: &str) -> bool {
    (1..=64).contains(&key.len()) && key.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_' || byte == b'-')
}

/// An error at byte `offset` of `text`, with its 1-based line and column (in characters).
fn at(text: &str, offset: usize, message: String) -> RunFileError {
    let before = &text[..offset.min(text.len())];
    let line = before.matches('\n').count() + 1;
    let column = before.rsplit('\n').next().map_or(0, |last| last.chars().count()) + 1;
    RunFileError { line, column, message }
}

fn build(raw: RawFile) -> RunFile {
    let vars = raw
        .vars
        .in_file_order()
        .map(|(name, var)| VarDecl {
            name: name.into_inner(),
            description: var.description,
            secret: var.secret,
            default: var.default.map(Spanned::into_inner),
        })
        .collect();
    let runs = raw.run.0.into_iter().map(|(key, run)| (key.span().start, config(key.into_inner(), run.name.clone(), ConfigKind::Run(run_spec(run)))));
    let serves = raw.serve.0.into_iter().map(|(key, serve)| (key.span().start, config(key.into_inner(), serve.name.clone(), ConfigKind::Serve(serve_spec(serve)))));
    let mut configs: Vec<(usize, Config)> = runs.chain(serves).collect();
    configs.sort_by_key(|(start, _)| *start);
    RunFile { vars, configs: configs.into_iter().map(|(_, config)| config).collect() }
}

fn config(key: String, name: Option<String>, kind: ConfigKind) -> Config {
    Config { name: name.unwrap_or_else(|| key.clone()), key, kind }
}

fn run_spec(run: RawRun) -> RunSpec {
    RunSpec {
        command: run.command.into_inner(),
        cwd: run.cwd.map(Spanned::into_inner),
        env: run.env.in_file_order().map(|(key, value)| (key.into_inner(), value.into_inner())).collect(),
        before: run.before.map(Spanned::into_inner).unwrap_or_default(),
    }
}

fn serve_spec(serve: RawServe) -> ServeSpec {
    ServeSpec {
        root: serve.root.into_inner(),
        port: match serve.port.into_inner() {
            Port::Number(number) => number.to_string(),
            Port::Text(text) => text,
        },
        live_reload: serve.live_reload.unwrap_or(true),
        fallback: serve.fallback,
        rewrites: serve.rewrites,
        script: serve.script,
        open: serve.open.map_or_else(|| "/".to_owned(), Spanned::into_inner),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MISC: &str = r#"[vars.zoho_secret]
description = "Zoho OAuth client secret"
secret = true

[vars.ngrok_subdomain]
description = "ngrok subdomain for misc"
default = "misc-${USER}"

[run.build]
name = "Build misc"
command = "mvn -q package -DskipTests"

[run.app]
name = "MiscApplication"
command = "mvn spring-boot:run -Dspring-boot.run.profiles=dev"
before = ["build"]
env = { ZOHO_SECRET = "${zoho_secret}" }

[run.ngrok]
name = "Misc - Ngrok"
command = "ngrok http 8080 --subdomain ${ngrok_subdomain}"

[serve.docs]
name = "Docs preview"
root = "docs/site"
port = 4200
live_reload = true
fallback = "index.html"
rewrites = [{ from = "^/blog/([^/]+)$", to = "/blog/$1.html" }]
script = "serve.rn"
"#;

    fn error(text: &str) -> String {
        parse(text).unwrap_err().to_string()
    }

    #[test]
    fn should_parse_the_spec_example() {
        // given
        let text = MISC;

        // when
        let file = parse(text).unwrap();

        // then
        let keys: Vec<&str> = file.configs.iter().map(|config| config.key.as_str()).collect();
        assert_eq!(keys, ["build", "app", "ngrok", "docs"]);
        assert_eq!(file.vars.iter().map(|var| var.name.as_str()).collect::<Vec<_>>(), ["zoho_secret", "ngrok_subdomain"]);
        assert!(file.vars[0].secret);
        assert_eq!(file.vars[1].default.as_deref(), Some("misc-${USER}"));
        let ConfigKind::Run(app) = &file.configs[1].kind else { panic!("app is a run") };
        assert_eq!((app.before.clone(), app.env.clone()), (vec!["build".to_owned()], vec![("ZOHO_SECRET".to_owned(), "${zoho_secret}".to_owned())]));
        let ConfigKind::Serve(docs) = &file.configs[3].kind else { panic!("docs is a serve") };
        assert_eq!((docs.port.as_str(), docs.live_reload, docs.rewrites.len()), ("4200", true, 1));
    }

    #[test]
    fn should_name_config_after_its_key_without_a_name() {
        // given
        let text = "[run.build]\ncommand = \"make\"\n";

        // when
        let file = parse(text).unwrap();

        // then
        assert_eq!(file.configs[0].name, "build");
    }

    #[test]
    fn should_keep_file_order_across_run_and_serve() {
        // given
        let text = "[serve.web]\nroot = \".\"\nport = 4200\n\n[run.build]\ncommand = \"make\"\n";

        // when
        let file = parse(text).unwrap();

        // then
        assert_eq!(file.configs.iter().map(|config| config.key.as_str()).collect::<Vec<_>>(), ["web", "build"]);
    }

    #[test]
    fn should_reject_unknown_field() {
        // given
        let text = "[run.a]\ncommand = \"x\"\ncolour = 1\n";

        // when
        let message = error(text);

        // then
        assert!(message.starts_with(".run.toml:3:1: unknown field `colour`"), "{message}");
    }

    #[test]
    fn should_reject_bad_key() {
        // given
        let text = "[run.Build]\ncommand = \"x\"\n";

        // when
        let message = error(text);

        // then
        assert_eq!(message, ".run.toml:1:6: key \"Build\" must be 1-64 of a-z, 0-9, _ and -");
    }

    #[test]
    fn should_reject_before_naming_a_missing_key() {
        // given
        let text = "[run.test]\nname = \"Test lib\"\ncommand = \"npm test\"\nbefore = [\"compile\"]\n";

        // when
        let message = error(text);

        // then
        assert_eq!(message, ".run.toml:4:10: before names \"compile\", which this file does not define");
    }

    #[test]
    fn should_reject_before_naming_a_serve() {
        // given
        let text = "[serve.web]\nroot = \".\"\nport = 4200\n\n[run.build]\ncommand = \"make\"\nbefore = [\"web\"]\n";

        // when
        let message = error(text);

        // then
        assert_eq!(message, ".run.toml:7:10: before names \"web\", a static server, which never ends");
    }

    #[test]
    fn should_reject_before_cycle() {
        // given
        let text = "[run.a]\ncommand = \"x\"\nbefore = [\"b\"]\n\n[run.b]\ncommand = \"y\"\nbefore = [\"a\"]\n";

        // when
        let message = error(text);

        // then
        assert_eq!(message, ".run.toml:3:10: before forms a cycle: a → b → a");
    }

    #[test]
    fn should_reject_secret_in_command() {
        // given
        let text = "[vars.token]\nsecret = true\n\n[run.a]\ncommand = \"curl -H ${token}\"\n";

        // when
        let message = error(text);

        // then
        assert_eq!(message, ".run.toml:5:11: secret variable token may only be used in env");
    }

    #[test]
    fn should_reject_unknown_variable() {
        // given
        let text = "[run.a]\ncommand = \"echo ${nope}\"\n";

        // when
        let message = error(text);

        // then
        assert_eq!(message, ".run.toml:2:11: unknown variable nope");
    }

    #[test]
    fn should_allow_builtins_and_escaped_dollar() {
        // given
        let text = "[run.a]\ncommand = \"echo ${USER} ${HOME} ${REPO} ${DIR} $${PWD}\"\n";

        // when
        let parsed = parse(text);

        // then
        assert!(parsed.is_ok(), "{parsed:?}");
    }

    #[test]
    fn should_reject_default_on_secret() {
        // given
        let text = "[vars.token]\nsecret = true\ndefault = \"x\"\n";

        // when
        let message = error(text);

        // then
        assert_eq!(message, ".run.toml:3:11: secret variable token cannot have a default");
    }

    #[test]
    fn should_reject_port_out_of_range() {
        // given
        let text = "[serve.web]\nroot = \".\"\nport = 80\n";

        // when
        let message = error(text);

        // then
        assert_eq!(message, ".run.toml:3:8: port must be 1024-65535, got 80");
    }

    #[test]
    fn should_read_the_page_a_server_opens_at() {
        // given
        let text = "[serve.web]\nroot = \".\"\nport = 4301\nopen = \"/mock/\"\n\n[serve.plain]\nroot = \".\"\nport = 4302\n";

        // when
        let file = parse(text).expect("valid file");

        // then
        let opens: Vec<_> = file.configs.iter().map(|config| match &config.kind {
            ConfigKind::Serve(spec) => spec.open.clone(),
            ConfigKind::Run(_) => panic!("only servers here"),
        }).collect();
        assert_eq!(opens, ["/mock/", "/"]);
    }

    #[test]
    fn should_reject_a_page_to_open_outside_the_site() {
        // given
        let text = "[serve.web]\nroot = \".\"\nport = 4301\nopen = \"mock/\"\n";

        // when
        let message = error(text);

        // then
        assert_eq!(message, ".run.toml:4:8: open must be a path starting with /, got \"mock/\"");
    }

    #[test]
    fn should_reject_duplicate_key_across_run_and_serve() {
        // given
        let text = "[run.web]\ncommand = \"x\"\n\n[serve.web]\nroot = \".\"\nport = 4200\n";

        // when
        let message = error(text);

        // then
        assert_eq!(message, ".run.toml:4:8: key \"web\" is used twice");
    }

    #[test]
    fn should_reject_file_over_64_kib() {
        // given
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(RUN_FILE), format!("# {}\n", "x".repeat(70_000))).unwrap();

        // when
        let loaded = load(dir.path());

        // then
        assert_eq!(loaded.unwrap().unwrap_err().to_string(), ".run.toml:1:1: the file is over 64 KiB");
    }

    #[test]
    fn should_load_nothing_without_a_file() {
        // given
        let dir = tempfile::tempdir().unwrap();

        // when
        let loaded = load(dir.path());

        // then
        assert!(loaded.is_none());
    }
}
