//! The values a run needs: `vars.toml`, the wallet, defaults and built-ins, filled into its command.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::file::{ConfigKind, RunFile, ServeSpec, references};
use super::wallet::Wallet;

/// `~/.config/ive/<profile>/vars.toml`: the user's values, a string or `{ wallet = "<id>" }` each, at the top level
/// or under `[repo."<main checkout>"]`.
pub struct VarsFile {
    path: PathBuf,
}

/// A value as `vars.toml` holds it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stored {
    Plain(String),
    Wallet(String),
}

/// `vars.toml` as read; a missing file reads as empty.
#[derive(Debug, Default)]
pub struct VarsDoc(toml::Table);

impl VarsDoc {
    pub fn parse(text: &str) -> Result<Self, String> {
        text.parse::<toml::Table>().map(Self).map_err(|error| format!("vars.toml:{}: {}", crate::toml_error::line_of(error.span(), text), error.message().trim_end()))
    }

    /// `name`'s value for the project whose main checkout is `project`: its `[repo]` table first, then the top level.
    pub fn lookup(&self, project: &Path, name: &str) -> Option<Stored> {
        let repos = self.0.get("repo").and_then(toml::Value::as_table);
        let repo = repos.and_then(|repos| repos.iter().find(|(path, _)| crate::paths::same(Path::new(path), project))).map(|(_, table)| table);
        let value = repo.and_then(toml::Value::as_table).and_then(|table| table.get(name)).or_else(|| self.0.get(name))?;
        match value {
            toml::Value::String(text) => Some(Stored::Plain(text.clone())),
            toml::Value::Table(table) => table.get("wallet").and_then(toml::Value::as_str).map(|id| Stored::Wallet(id.to_owned())),
            toml::Value::Integer(number) => Some(Stored::Plain(number.to_string())),
            _ => None,
        }
    }
}

impl VarsFile {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn load(&self) -> Result<VarsDoc, String> {
        match std::fs::read_to_string(&self.path) {
            Ok(text) => VarsDoc::parse(&text),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(VarsDoc::default()),
            Err(error) => Err(format!("cannot read {}: {error}", self.path.display())),
        }
    }

    /// Writes `name = "value"` at the top level, keeping the rest of the file as it is.
    pub fn save_plain(&self, name: &str, value: &str) -> Result<(), String> {
        self.edit(|doc| doc[name] = toml_edit::value(value))
    }

    /// Puts `secret` in `wallet` under `name` and `name = { wallet = "name" }` in the file; the file never holds it.
    pub fn save_secret(&self, name: &str, secret: &str, wallet: &dyn Wallet) -> Result<(), String> {
        wallet.set(name, secret)?;
        let mut reference = toml_edit::InlineTable::new();
        reference.insert("wallet", name.into());
        self.edit(|doc| doc[name] = toml_edit::value(reference))
    }

    fn edit(&self, change: impl FnOnce(&mut toml_edit::DocumentMut)) -> Result<(), String> {
        let text = match std::fs::read_to_string(&self.path) {
            Ok(text) => text,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
            Err(error) => return Err(format!("cannot read {}: {error}", self.path.display())),
        };
        let mut doc: toml_edit::DocumentMut = text.parse().map_err(|error: toml_edit::TomlError| format!("{}:{}: {}", self.path.display(), crate::toml_error::line_of(error.span(), &text), error.message().trim_end()))?;
        change(&mut doc);
        if let Some(dir) = self.path.parent() {
            std::fs::create_dir_all(dir).map_err(|error| format!("cannot create {}: {error}", dir.display()))?;
        }
        std::fs::write(&self.path, doc.to_string()).map_err(|error| format!("cannot write {}: {error}", self.path.display()))
    }
}

/// The variables every `.run.toml` has.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Builtins {
    pub user: String,
    pub home: PathBuf,
    /// the checkout's root, so a worktree uses its own
    pub repo: PathBuf,
    /// the folder of `.run.toml`
    pub dir: PathBuf,
}

impl Builtins {
    fn get(&self, name: &str) -> Option<String> {
        match name {
            "USER" => Some(self.user.clone()),
            "HOME" => Some(self.home.display().to_string()),
            "REPO" => Some(self.repo.display().to_string()),
            "DIR" => Some(self.dir.display().to_string()),
            _ => None,
        }
    }
}

/// Where values come from: `vars.toml`, the wallet it points into, and the project it is looked up for.
pub struct Sources<'a> {
    pub doc: &'a VarsDoc,
    pub wallet: &'a dyn Wallet,
    /// the project's main checkout, naming its `[repo]` table
    pub project: &'a Path,
}

/// The values of the variables a run needs, and the ones that have none.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolved {
    pub values: HashMap<String, String>,
    pub missing: Vec<String>,
}

/// What starting a configuration takes, its values filled in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prepared {
    pub cwd: PathBuf,
    /// the run's `env`, secrets included: only ever given to the program's environment
    pub launch_env: Vec<(String, String)>,
    /// the command as rows show it: secrets stay `${name}`
    pub shown: String,
    pub kind: PreparedKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreparedKind {
    Command(String),
    Serve { root: PathBuf, port: u16, spec: ServeSpec },
}

/// The variables `key` and its `before` steps use, also through other variables' defaults, in first-use order.
pub fn needed(file: &RunFile, key: &str) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    let add = |text: &str, names: &mut Vec<String>| {
        for name in references(text) {
            if file.vars.iter().any(|var| var.name == name) && !names.iter().any(|known| known == name) {
                names.push(name.to_owned());
            }
        }
    };
    for config in steps_then(file, key).into_iter().filter_map(|key| file.configs.iter().find(|config| config.key == key)) {
        for text in texts(&config.kind) {
            add(text, &mut names);
        }
    }
    // defaults may name further variables
    let mut index = 0;
    while index < names.len() {
        if let Some(default) = file.vars.iter().find(|var| var.name == names[index]).and_then(|var| var.default.clone()) {
            add(&default, &mut names);
        }
        index += 1;
    }
    names
}

/// The values of what `key` needs: stored ones (a wallet entry must exist), else defaults filled in.
pub fn resolve(file: &RunFile, key: &str, sources: &Sources, builtins: &Builtins) -> Result<Resolved, String> {
    let names = needed(file, key);
    let mut values = HashMap::new();
    for name in &names {
        let value = match sources.doc.lookup(sources.project, name) {
            Some(Stored::Plain(value)) => Some(value),
            Some(Stored::Wallet(id)) => sources.wallet.get(&id)?,
            None => None,
        };
        if let Some(value) = value {
            values.insert(name.clone(), value);
        }
    }
    // defaults last, in reverse first-use order, so one naming a later variable sees its value
    for name in names.iter().rev() {
        if values.contains_key(name) {
            continue;
        }
        if let Some(default) = file.vars.iter().find(|var| &var.name == name).and_then(|var| var.default.as_ref()) {
            let filled = fill(default, &values, builtins);
            values.insert(name.clone(), filled);
        }
    }
    let missing = names.into_iter().filter(|name| !values.contains_key(name)).collect();
    Ok(Resolved { values, missing })
}

/// `text` with `${name}` replaced by a built-in or one of `values`; others stay as written, `$${` becomes `${`.
pub fn fill(text: &str, values: &HashMap<String, String>, builtins: &Builtins) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(found) = rest.find("${") {
        let Some(end) = rest[found..].find('}').map(|end| found + end) else { break };
        let name = &rest[found + 2..end];
        if rest[..found].ends_with('$') {
            out.push_str(&rest[..found - 1]);
            out.push_str(&rest[found..=end]);
        } else {
            out.push_str(&rest[..found]);
            match builtins.get(name).or_else(|| values.get(name).cloned()) {
                Some(value) => out.push_str(&value),
                None => out.push_str(&rest[found..=end]),
            }
        }
        rest = &rest[end + 1..];
    }
    out.push_str(rest);
    out
}

/// What starting `key` takes with `resolved`'s values; `Err` when the file has no such configuration or a server's
/// port is not a port.
pub fn prepare(file: &RunFile, key: &str, resolved: &Resolved, builtins: &Builtins) -> Result<Prepared, String> {
    let config = file.configs.iter().find(|config| config.key == key).ok_or_else(|| format!("no configuration {key} in .run.toml"))?;
    let secrets: Vec<&str> = file.vars.iter().filter(|var| var.secret).map(|var| var.name.as_str()).collect();
    let plain: HashMap<String, String> = resolved.values.iter().filter(|(name, _)| !secrets.contains(&name.as_str())).map(|(k, v)| (k.clone(), v.clone())).collect();
    let in_dir = |path: &str| {
        let path = PathBuf::from(fill(path, &plain, builtins));
        if path.is_absolute() { normalized(&path) } else { normalized(&builtins.dir.join(path)) }
    };
    match &config.kind {
        ConfigKind::Run(run) => Ok(Prepared {
            cwd: run.cwd.as_deref().map_or_else(|| builtins.dir.clone(), in_dir),
            launch_env: run.env.iter().map(|(name, value)| (name.clone(), fill(value, &resolved.values, builtins))).collect(),
            shown: fill(&run.command, &plain, builtins),
            kind: PreparedKind::Command(fill(&run.command, &plain, builtins)),
        }),
        ConfigKind::Serve(serve) => {
            let port_text = fill(&serve.port, &plain, builtins);
            let port: u16 = port_text.trim().parse().ok().filter(|port| *port >= 1024).ok_or_else(|| format!("port must be 1024-65535, got {port_text:?}"))?;
            Ok(Prepared {
                cwd: builtins.dir.clone(),
                launch_env: Vec::new(),
                shown: format!("serve {} → :{port}", fill(&serve.root, &plain, builtins)),
                kind: PreparedKind::Serve { root: in_dir(&serve.root), port, spec: serve.clone() },
            })
        }
    }
}

/// `path` without `.` and with each `..` taking off the folder before it, as rows show it; the disk is not read.
fn normalized(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for part in path.components() {
        match part {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                out.pop();
            }
            other => out.push(other),
        }
    }
    out
}

/// The steps that run before `key`, nested ones first, each once: a step's own `before` runs before it.
pub fn steps_of(file: &RunFile, key: &str) -> Vec<String> {
    fn visit(file: &RunFile, key: &str, steps: &mut Vec<String>) {
        let before = file.configs.iter().find(|config| config.key == key).map_or_else(Vec::new, |config| match &config.kind {
            ConfigKind::Run(run) => run.before.clone(),
            ConfigKind::Serve(_) => Vec::new(),
        });
        for step in before {
            // a cycle cannot reach here: parsing rejects it
            if !steps.contains(&step) {
                visit(file, &step, steps);
                steps.push(step);
            }
        }
    }
    let mut steps = Vec::new();
    visit(file, key, &mut steps);
    steps
}

/// `key`'s `before` steps, nested ones first, then `key`.
fn steps_then(file: &RunFile, key: &str) -> Vec<String> {
    steps_of(file, key).into_iter().chain([key.to_owned()]).collect()
}

/// The texts of a configuration that may name variables.
fn texts(kind: &ConfigKind) -> Vec<&str> {
    match kind {
        ConfigKind::Run(run) => [run.command.as_str()].into_iter().chain(run.cwd.as_deref()).chain(run.env.iter().map(|(_, value)| value.as_str())).collect(),
        ConfigKind::Serve(serve) => vec![serve.root.as_str(), serve.port.as_str()],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::run::file::parse;
    use crate::run::wallet::MemoryWallet;

    #[test]
    fn should_keep_line_of_vars_error() {
        // given
        let text = "a = \"1\"\n\nb = \n";

        // when
        let error = VarsDoc::parse(text).unwrap_err();

        // then
        assert!(error.starts_with("vars.toml:3: "), "{error}");
    }

    const MISC: &str = r#"[vars.zoho_secret]
description = "Zoho OAuth client secret"
secret = true

[vars.ngrok_subdomain]
description = "ngrok subdomain for misc"
default = "misc-${USER}"

[run.build]
command = "mvn -q package"

[run.app]
command = "mvn spring-boot:run -Dzoho=$ZOHO_SECRET"
cwd = ".."
before = ["build"]
env = { ZOHO_SECRET = "${zoho_secret}" }

[run.ngrok]
command = "ngrok http 8080 --subdomain ${ngrok_subdomain}"
cwd = "/srv/data"
"#;

    fn builtins() -> Builtins {
        Builtins { user: "kb".to_owned(), home: PathBuf::from("/home/kb"), repo: PathBuf::from("/r/wt"), dir: PathBuf::from("/r/wt/misc") }
    }

    fn doc(text: &str) -> VarsDoc {
        VarsDoc::parse(text).unwrap()
    }

    fn resolved(file: &RunFile, key: &str, vars: &str, wallet: &MemoryWallet) -> Resolved {
        let doc = doc(vars);
        let sources = Sources { doc: &doc, wallet, project: Path::new("/r") };
        resolve(file, key, &sources, &builtins()).unwrap()
    }

    #[test]
    fn should_list_variables_of_config_and_its_before_steps() {
        // given
        let file = parse(MISC).unwrap();

        // when
        let needs = [needed(&file, "app"), needed(&file, "ngrok"), needed(&file, "build")];

        // then
        assert_eq!(needs, [vec!["zoho_secret".to_owned()], vec!["ngrok_subdomain".to_owned()], vec![]]);
    }

    #[test]
    fn should_run_nested_before_steps_first_and_once() {
        // given
        let text = "[vars.token]\nsecret = true\n\n[run.codegen]\ncommand = \"gen\"\nenv = { T = \"${token}\" }\n\n[run.build]\ncommand = \"make\"\nbefore = [\"codegen\"]\n\n[run.lint]\ncommand = \"lint\"\nbefore = [\"codegen\"]\n\n[run.app]\ncommand = \"run\"\nbefore = [\"build\", \"lint\"]\n";
        let file = parse(text).unwrap();

        // when
        let steps = steps_of(&file, "app");

        // then
        assert_eq!(steps, ["codegen", "build", "lint"]);
        assert_eq!(needed(&file, "app"), ["token"]);
    }

    #[test]
    fn should_use_repo_table_before_top_level() {
        // given
        let file = parse(MISC).unwrap();
        let vars = "ngrok_subdomain = \"kb-misc\"\n\n[repo.\"/r\"]\nngrok_subdomain = \"kb-r\"\n";

        // when
        let found = resolved(&file, "ngrok", vars, &MemoryWallet::default());

        // then
        assert_eq!(found.values["ngrok_subdomain"], "kb-r");
    }

    #[test]
    fn should_fall_back_to_default_with_builtins() {
        // given
        let file = parse(MISC).unwrap();

        // when
        let found = resolved(&file, "ngrok", "", &MemoryWallet::default());

        // then
        assert_eq!((found.values["ngrok_subdomain"].as_str(), found.missing.len()), ("misc-kb", 0));
    }

    #[test]
    fn should_report_missing_wallet_entry_as_missing() {
        // given
        let file = parse(MISC).unwrap();

        // when
        let found = resolved(&file, "app", "zoho_secret = { wallet = \"zoho\" }\n", &MemoryWallet::default());

        // then
        assert_eq!(found.missing, ["zoho_secret"]);
    }

    #[test]
    fn should_read_a_secret_from_the_wallet() {
        // given
        let file = parse(MISC).unwrap();
        let wallet = MemoryWallet::default();
        wallet.set("zoho", "s3cret").unwrap();

        // when
        let found = resolved(&file, "app", "zoho_secret = { wallet = \"zoho\" }\n", &wallet);

        // then
        assert_eq!(found.values["zoho_secret"], "s3cret");
    }

    #[test]
    fn should_keep_literal_after_double_dollar() {
        // given
        let values = HashMap::new();

        // when
        let filled = fill("echo $${PWD} in ${REPO}", &values, &builtins());

        // then
        assert_eq!(filled, "echo ${PWD} in /r/wt");
    }

    #[test]
    fn should_store_a_secret_in_the_wallet_and_a_reference_in_vars() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let vars = VarsFile::new(dir.path().join("vars.toml"));
        let wallet = MemoryWallet::default();

        // when
        vars.save_secret("zoho_secret", "s3cret", &wallet).unwrap();

        // then
        let text = std::fs::read_to_string(dir.path().join("vars.toml")).unwrap();
        assert_eq!(text.trim(), "zoho_secret = { wallet = \"zoho_secret\" }");
        assert_eq!(wallet.get("zoho_secret").unwrap().as_deref(), Some("s3cret"));
    }

    #[test]
    fn should_keep_comments_when_saving() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vars.toml");
        std::fs::write(&path, "# mine\nport = \"8080\" # local\n").unwrap();

        // when
        VarsFile::new(path.clone()).save_plain("ngrok_subdomain", "kb-misc").unwrap();

        // then
        let text = std::fs::read_to_string(&path).unwrap();
        assert_eq!(text, "# mine\nport = \"8080\" # local\nngrok_subdomain = \"kb-misc\"\n");
    }

    #[test]
    fn should_show_secret_as_its_name_and_put_it_only_in_env() {
        // given
        let file = parse(MISC).unwrap();
        let wallet = MemoryWallet::default();
        wallet.set("zoho", "s3cret").unwrap();
        let found = resolved(&file, "app", "zoho_secret = { wallet = \"zoho\" }\n", &wallet);

        // when
        let prepared = prepare(&file, "app", &found, &builtins()).unwrap();

        // then
        assert_eq!(prepared.launch_env, [("ZOHO_SECRET".to_owned(), "s3cret".to_owned())]);
        assert_eq!(prepared.kind, PreparedKind::Command("mvn spring-boot:run -Dzoho=$ZOHO_SECRET".to_owned()));
        assert!(!prepared.shown.contains("s3cret"), "{}", prepared.shown);
    }

    #[test]
    fn should_resolve_cwd_relative_and_absolute() {
        // given
        let file = parse(MISC).unwrap();
        let found = resolved(&file, "ngrok", "", &MemoryWallet::default());

        // when
        let cwds = [
            prepare(&file, "build", &found, &builtins()).unwrap().cwd,
            prepare(&file, "app", &found, &builtins()).unwrap().cwd,
            prepare(&file, "ngrok", &found, &builtins()).unwrap().cwd,
        ];

        // then
        assert_eq!(cwds, [PathBuf::from("/r/wt/misc"), PathBuf::from("/r/wt"), PathBuf::from("/srv/data")]);
    }

    #[test]
    fn should_show_a_server_as_root_and_port() {
        // given
        let file = parse("[vars.port]\ndefault = \"4300\"\n\n[serve.web]\nroot = \"site\"\nport = \"${port}\"\n").unwrap();
        let found = resolved(&file, "web", "", &MemoryWallet::default());

        // when
        let prepared = prepare(&file, "web", &found, &builtins()).unwrap();

        // then
        assert_eq!(prepared.shown, "serve site → :4300");
        let PreparedKind::Serve { root, port, .. } = prepared.kind else { panic!("a server") };
        assert_eq!((root, port), (PathBuf::from("/r/wt/misc/site"), 4300));
    }
}
