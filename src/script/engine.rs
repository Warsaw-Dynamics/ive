//! Compiles IVE's `ive.rn` and the user's `init.rn` and runs their `init`, turning what they declared into one
//! [`Keymap`] (the user's entries over the defaults), the fonts, and the bound functions.

use std::path::Path;
use std::sync::Arc;

use rune::diagnostics::{Diagnostic, FatalDiagnosticKind};
use rune::modules::capture_io::{self, CaptureIo};
use rune::runtime::{Function, RuntimeContext, budget};
use rune::{Context, Diagnostics, Source, Sources, Vm};

use futures::channel::mpsc::UnboundedSender;

use super::{api_module, http_module, toml_module};
use super::config::{self, Carried, Config};
use super::declare::{self, FontChoice, Functions};
use super::keymap::Keymap;
use super::tab_menus::{self, TabMenus};
use super::excluded::{self, Excluded};
use super::open_folders::{self, OpenFolders};
use super::quit_countdown::{self, QuitCountdown};
use super::temporary_tabs::{self, TemporaryTabs};
use crate::api::Call;
use crate::theme::Fonts;
use crate::theme_config::{self, ThemeSpec};

pub const INIT: &str = "init.rn";
pub const IVE_RN_NAME: &str = "ive.rn";
/// IVE's default bindings; always loaded from here, never from the copy written next to `init.rn`.
pub const IVE_RN: &str = include_str!("../../assets/scripts/ive.rn");
/// Rune instructions `init` may run: plenty for configuration, and a loop in it ends the load instead of
/// holding the script thread forever.
const INIT_BUDGET: usize = 50_000_000;

/// A load of `ive.rn` and `init.rn`: the keymap, the fonts, the theme, and the functions its `Invoke` actions name by
/// index.
pub struct Loaded {
    pub keymap: Keymap,
    pub fonts: Fonts,
    pub theme: ThemeSpec,
    pub excluded: Excluded,
    pub temporary_tabs: TemporaryTabs,
    pub tab_menus: TabMenus,
    pub quit_countdown: QuitCountdown,
    pub open_folders: OpenFolders,
    pub functions: Vec<Function>,
    /// whether there was an `init.rn`; without it the keymap is `ive.rn`'s alone
    pub has_init: bool,
}

/// A script to run: its file name, text, and path on disk (for `mod` next to it).
struct Script<'a> {
    name: &'a str,
    text: String,
    path: Option<&'a Path>,
}

/// The Rune context scripts compile against: Rune's standard modules, the enabled `rune-modules`, IVE's `http` and `toml`, `ive`, `act`,
/// and `println!` captured for IVE's script log.
pub struct Engine {
    context: Context,
    runtime: Arc<RuntimeContext>,
    output: CaptureIo,
}

impl Engine {
    pub fn new(calls: UnboundedSender<Call>) -> Result<Self, String> {
        let output = CaptureIo::new();
        let mut context = rune_modules::with_config(false).map_err(|error| format!("cannot set up Rune: {error}"))?;
        context.install(http_module::module().map_err(|error| format!("cannot set up the http module: {error}"))?).map_err(|error| error.to_string())?;
        context.install(toml_module::module().map_err(|error| format!("cannot set up the toml module: {error}"))?).map_err(|error| error.to_string())?;
        context.install(capture_io::module(&output).map_err(|error| format!("cannot set up Rune output: {error}"))?).map_err(|error| error.to_string())?;
        context.install(config::module().map_err(|error| format!("cannot set up the ive module: {error}"))?).map_err(|error| error.to_string())?;
        context.install(config::act_module().map_err(|error| format!("cannot set up the act module: {error}"))?).map_err(|error| error.to_string())?;
        context.install(api_module::module(calls).map_err(|error| format!("cannot set up the ive API: {error}"))?).map_err(|error| error.to_string())?;
        let runtime = Arc::new(context.runtime().map_err(|error| format!("cannot set up Rune: {error}"))?);
        Ok(Self { context, runtime, output })
    }

    /// Loads `ive.rn`, then `dir/init.rn` when there is one, as load number `generation`. The error names the file,
    /// and line and column for each compile error.
    pub fn load(&self, dir: Option<&Path>, generation: u64) -> Result<Loaded, String> {
        let mut bound = Vec::new();
        let mut functions = Functions { bound: &mut bound, generation };
        let ive_rn = Script { name: IVE_RN_NAME, text: IVE_RN.to_owned(), path: None };
        let start = Carried {
            theme: theme_config::default_value(),
            excluded: rune::Value::from(()),
            temporary_tabs: rune::Value::from(()),
            tab_menus: rune::Value::from(()),
            quit_countdown: rune::Value::from(()),
            open_folders: rune::Value::from(()),
        };
        let (defaults, default_fonts, default_carried) = self.run(&ive_rn, &mut functions, start)?;
        let default_fonts = default_fonts.over(Fonts::default());
        let carried_defaults = Carried {
            theme: rune::Value::from(()),
            excluded: rune::Value::from(()),
            temporary_tabs: default_carried.temporary_tabs.clone(),
            tab_menus: default_carried.tab_menus.clone(),
            quit_countdown: rune::Value::from(()),
            open_folders: rune::Value::from(()),
        };
        let init = dir.map(|dir| dir.join(INIT)).filter(|path| path.exists());
        let Some(path) = init else {
            let checked = checked(&default_carried, &carried_defaults, &defaults, IVE_RN_NAME)?;
            return Ok(Loaded {
                keymap: defaults,
                fonts: default_fonts,
                theme: checked.theme,
                excluded: checked.excluded,
                temporary_tabs: checked.temporary_tabs,
                tab_menus: checked.tab_menus,
                quit_countdown: checked.quit_countdown,
                open_folders: checked.open_folders,
                functions: bound,
                has_init: false,
            });
        };
        let text = std::fs::read_to_string(&path).map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        let (user, fonts, carried) = self.run(&Script { name: INIT, text, path: Some(&path) }, &mut functions, default_carried)?;
        let keymap = Keymap::merged(defaults, user);
        let checked = checked(&carried, &carried_defaults, &keymap, INIT)?;
        Ok(Loaded {
            keymap,
            fonts: fonts.over(default_fonts),
            theme: checked.theme,
            excluded: checked.excluded,
            temporary_tabs: checked.temporary_tabs,
            tab_menus: checked.tab_menus,
            quit_countdown: checked.quit_countdown,
            open_folders: checked.open_folders,
            functions: bound,
            has_init: true,
        })
    }

    /// What scripts printed since the last call, one entry per line.
    pub fn drain_output(&self) -> Vec<String> {
        let printed = self.output.drain_utf8().map(|text| text.to_string()).unwrap_or_default();
        printed.lines().map(str::to_owned).collect()
    }

    /// Runs `script`'s `init` with `carried` in `config`: its keymap, fonts and what it left for the next script.
    fn run(&self, script: &Script, functions: &mut Functions, carried: Carried) -> Result<(Keymap, FontChoice, Carried), String> {
        let name = script.name;
        let (mut vm, sources) = self.compile(script)?;
        let mut config = Config::new(carried);
        let returned = budget::with(INIT_BUDGET, || vm.call(["init"], (&mut config,)))
            .call()
            .map_err(|error| init_error(&error_place(&error, &sources, name), &error.to_string()))?;
        if let Ok(Err(error)) = rune::from_value::<Result<rune::Value, rune::Value>>(returned) {
            return Err(format!("{name}: init returned an error: {}", describe(error)));
        }
        let declared = declare::declared(config, name, functions)?;
        let keymap = Keymap::new(declared.bindings).map_err(|error| format!("{name}: {error}"))?;
        Ok((keymap, declared.fonts, declared.carried))
    }

    /// The script's VM, and its sources for placing run-time errors.
    fn compile(&self, script: &Script) -> Result<(Vm, Sources), String> {
        let source = match script.path {
            Some(path) => Source::with_path(script.name, script.text.as_str(), path),
            None => Source::new(script.name, script.text.as_str()),
        };
        let mut sources = Sources::new();
        sources.insert(source.map_err(|error| error.to_string())?).map_err(|error| error.to_string())?;
        let mut diagnostics = Diagnostics::new();
        let unit = rune::prepare(&mut sources).with_context(&self.context).with_diagnostics(&mut diagnostics).build();
        match unit {
            Ok(unit) if !diagnostics.has_error() => Ok((Vm::new(self.runtime.clone(), Arc::new(unit)), sources)),
            _ => Err(compile_errors(&diagnostics, &sources, script.name)),
        }
    }
}

/// What one script left in `config` for the next, checked.
struct Checked {
    theme: ThemeSpec,
    excluded: Excluded,
    temporary_tabs: TemporaryTabs,
    tab_menus: TabMenus,
    quit_countdown: QuitCountdown,
    open_folders: OpenFolders,
}

/// The theme, excluded folders, temporary tabs and tab menus `file` left, checked: keys of `temporary_tabs` and menus of `tab_menus`
/// it left out taken from `defaults`, and every menu `keymap` opens declared; the error names `file`.
fn checked(carried: &Carried, defaults: &Carried, keymap: &Keymap, file: &str) -> Result<Checked, String> {
    let named = |problem: String| format!("{file}: {problem}");
    let theme = theme_config::parse(&carried.theme).map_err(named)?;
    let excluded = excluded::parse(&carried.excluded).map_err(named)?;
    let temporary_tabs = temporary_tabs::parse(&defaults.temporary_tabs, &carried.temporary_tabs).map_err(named)?;
    let tab_menus = tab_menus::parse(&defaults.tab_menus, &carried.tab_menus).map_err(named)?;
    tab_menus::check_shown(&tab_menus, keymap.tab_menu_names()).map_err(named)?;
    let quit_countdown = quit_countdown::parse(&carried.quit_countdown).map_err(named)?;
    let open_folders = open_folders::parse(&carried.open_folders).map_err(named)?;
    Ok(Checked { theme, excluded, temporary_tabs, tab_menus, quit_countdown, open_folders })
}

/// Where a run-time error happened, as `file:line:column` like a compile error; `name` when Rune does not know.
fn error_place(error: &rune::runtime::VmError, sources: &Sources, name: &str) -> String {
    let place = error.first_location().and_then(|at| {
        let instruction = at.unit.debug_info()?.instruction_at(at.ip)?;
        let source = sources.get(instruction.source_id)?;
        let (line, column) = source.pos_to_utf8_linecol(instruction.span.start.into_usize());
        Some(format!("{}:{}:{}", source_file_name(source), line + 1, column + 1))
    });
    place.unwrap_or_else(|| name.to_owned())
}

fn init_error(name: &str, error: &str) -> String {
    if is_out_of_budget(error) {
        return format!("{name}: init ran over {INIT_BUDGET} instructions and was stopped; does it loop forever?");
    }
    format!("{name}: {error}")
}

/// Whether a Rune error is a spent instruction budget.
pub fn is_out_of_budget(error: &str) -> bool {
    // Rune's error kinds are private; a spent budget shows only as this halt reason
    error.ends_with("`limited`")
}

/// One line per error: `file:line:column: message`.
pub(crate) fn compile_errors(diagnostics: &Diagnostics, sources: &Sources, name: &str) -> String {
    let lines: Vec<String> = diagnostics
        .diagnostics()
        .iter()
        .filter_map(|diagnostic| match diagnostic {
            Diagnostic::Fatal(fatal) => Some(fatal),
            _ => None,
        })
        .map(|fatal| {
            let source = sources.get(fatal.source_id());
            let file = source.map_or(name.to_owned(), source_file_name);
            match (fatal.kind(), source) {
                (FatalDiagnosticKind::CompileError(error), Some(source)) => {
                    let (line, column) = source.pos_to_utf8_linecol(rune::ast::Spanned::span(error).start.into_usize());
                    format!("{file}:{}:{}: {error}", line + 1, column + 1)
                }
                _ => format!("{file}: {fatal}"),
            }
        })
        .collect();
    lines.join("\n")
}

fn source_file_name(source: &Source) -> String {
    source.path().and_then(Path::file_name).map_or_else(|| source.name().to_owned(), |name| name.to_string_lossy().into_owned())
}

/// A Rune value as a script author reads it: strings as they are, anything else in debug form.
pub fn describe(value: rune::Value) -> String {
    rune::from_value::<String>(value.clone()).unwrap_or_else(|_| format!("{value:?}"))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{Duration, Instant};

    use gpui::{Keystroke, Modifiers};

    use super::*;
    use crate::sidebar::folders::FolderKind;
    use crate::theme::Fonts;
    use crate::theme_config::{Height, Px, Rgb};
    use crate::script::keymap::{Action, Builtin, ClipboardTarget, MouseButton, MouseKind, MouseTrigger, PasteKind, Place, Scope, SelectionMode};
    use crate::script::tab_menus::MenuEntry;
    use crate::script::temporary_tabs::{TabKind, TabMarker, TemporaryTabs};

    const ALL_KINDS: [TabKind; 4] = [TabKind::Project, TabKind::Settings, TabKind::Hub, TabKind::RunConfigs];

    fn load(init: &str) -> (Engine, Result<Loaded, String>) {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("init.rn"), init).unwrap();
        let engine = Engine::new(futures::channel::mpsc::unbounded().0).unwrap();
        let loaded = engine.load(Some(dir.path()), 1);
        (engine, loaded)
    }

    fn defaults() -> Loaded {
        Engine::new(futures::channel::mpsc::unbounded().0).unwrap().load(None, 1).unwrap()
    }

    fn pressed(key: &str) -> Keystroke {
        Keystroke::parse(key).unwrap()
    }

    fn mouse(kind: MouseKind, button: MouseButton, streak: u8) -> MouseTrigger {
        MouseTrigger { kind, button, streak, mods: Modifiers::default(), mouse_reporting: false }
    }

    fn builtin(builtin: Builtin) -> Option<Action> {
        Some(Action::Builtin(builtin))
    }

    #[test]
    fn should_load_default_temporary_tabs() {
        // when
        let temporary_tabs = defaults().temporary_tabs;

        // then
        assert_eq!(temporary_tabs, TemporaryTabs { enabled: true, kinds: ALL_KINDS.to_vec(), marker: TabMarker::Italic });
    }

    #[test]
    fn should_give_scripts_ives_http_and_toml_modules() {
        // given
        let init = r#"
            pub async fn deploy() {
                let deploy = http::request(#{ method: "POST", url: "https://example.test/deploy", body: json::to_string(#{ env: "dev" })?, headers: #{ "Content-Type": "application/json" } }).await?;
                if deploy.status != 200 {
                    println!("deploy: {} {}", deploy.status, deploy.body);
                }
                Ok(())
            }
            pub fn init(config) { config.temporary_tabs.enabled = toml::from_string("on = false")?.on; }
        "#;

        // when
        let temporary_tabs = load(init).1.unwrap().temporary_tabs;

        // then
        assert!(!temporary_tabs.enabled);
    }

    #[test]
    fn should_let_init_switch_temporary_tabs_off() {
        // given
        let init = "pub fn init(config) { config.temporary_tabs.enabled = false; }";

        // when
        let temporary_tabs = load(init).1.unwrap().temporary_tabs;

        // then
        assert_eq!(temporary_tabs, TemporaryTabs { enabled: false, kinds: ALL_KINDS.to_vec(), marker: TabMarker::Italic });
    }

    #[test]
    fn should_fill_missing_temporary_tab_keys_from_defaults() {
        // given
        let init = r#"pub fn init(config) { config.temporary_tabs = #{ marker: #{ Prefix: "*" } }; }"#;

        // when
        let temporary_tabs = load(init).1.unwrap().temporary_tabs;

        // then
        assert_eq!(temporary_tabs, TemporaryTabs { enabled: true, kinds: ALL_KINDS.to_vec(), marker: TabMarker::Prefix("*".into()) });
    }

    #[test]
    fn should_reject_an_unknown_temporary_tab_kind() {
        // given
        let init = r#"pub fn init(config) { config.temporary_tabs.kinds = ["Files"]; }"#;

        // when
        let error = load(init).1.err().unwrap();

        // then
        assert_eq!(error, r#"init.rn: temporary_tabs.kinds: "Files" is not one of "Project", "Settings", "Hub", "RunConfigs""#);
    }

    #[test]
    fn should_parse_run_configs_kind() {
        // given
        let init = r#"pub fn init(config) { config.temporary_tabs.kinds = ["RunConfigs"]; }"#;

        // when
        let kinds = load(init).1.unwrap().temporary_tabs.kinds;

        // then
        assert_eq!(kinds, [TabKind::RunConfigs]);
    }

    #[test]
    fn should_reject_a_duplicate_temporary_tab_kind() {
        // given
        let init = r#"pub fn init(config) { config.temporary_tabs.kinds = ["Project", "Project"]; }"#;

        // when
        let error = load(init).1.err().unwrap();

        // then
        assert!(error.contains(r#""Project" is listed twice"#), "{error}");
    }

    #[test]
    fn should_reject_a_long_temporary_tab_prefix() {
        // given
        let init = r#"pub fn init(config) { config.temporary_tabs.marker = #{ Prefix: "123456789" }; }"#;

        // when
        let error = load(init).1.err().unwrap();

        // then
        assert!(error.contains("1 to 8 characters") && error.contains("123456789"), "{error}");
    }

    #[test]
    fn should_reject_an_unknown_temporary_tabs_key() {
        // given
        let init = "pub fn init(config) { config.temporary_tabs = #{ enable: false }; }";

        // when
        let error = load(init).1.err().unwrap();

        // then
        assert!(error.contains(r#""enable""#) && error.contains("enabled, kinds, marker"), "{error}");
    }

    #[test]
    fn should_accept_no_temporary_tab_kinds() {
        // given
        let init = "pub fn init(config) { config.temporary_tabs.kinds = []; }";

        // when
        let temporary_tabs = load(init).1.unwrap().temporary_tabs;

        // then
        assert!(!temporary_tabs.opens_temporary(TabKind::Project));
    }

    #[test]
    fn should_load_default_theme_without_init() {
        // when
        let theme = defaults().theme;

        // then
        assert_eq!(theme.dark.tree, Rgb(0x191a1c));
    }

    #[test]
    fn should_apply_init_theme_override() {
        // given
        let init = r##"pub fn init(config) { config.theme.dark.tree = "#202124"; }"##;

        // when
        let theme = load(init).1.unwrap().theme;

        // then
        assert_eq!(theme.dark.tree, Rgb(0x202124));
        assert_eq!(theme.dark.background, Rgb(0x1e1f22));
    }

    #[test]
    fn should_let_init_read_default_theme() {
        // given
        let init = "pub fn init(config) { config.theme.panels.gap = config.theme.panels.radius + 5.0; }";

        // when
        let theme = load(init).1.unwrap().theme;

        // then
        assert_eq!(theme.panels.gap, Px(13.));
    }

    #[test]
    fn should_count_the_quit_prompt_down_from_ten_seconds_by_default() {
        // when
        let quit_countdown = defaults().quit_countdown;

        // then
        assert_eq!(quit_countdown.seconds(), 10);
    }

    #[test]
    fn should_take_the_quit_countdown_from_init() {
        // given
        let init = "pub fn init(config) { config.quit_countdown = 3; }";

        // when
        let quit_countdown = load(init).1.unwrap().quit_countdown;

        // then
        assert_eq!(quit_countdown.seconds(), 3);
    }

    #[test]
    fn should_reject_a_quit_countdown_out_of_range_from_init() {
        // given
        let init = "pub fn init(config) { config.quit_countdown = 0; }";

        // when
        let error = load(init).1.err().unwrap();

        // then
        assert_eq!(error, "init.rn: quit_countdown must be whole seconds from 1 to 60, not 0");
    }

    #[test]
    fn should_open_all_folders_but_files_by_default() {
        // when
        let open = defaults().open_folders;

        // then
        let kinds = FolderKind::ALL.map(|kind| open.is_open(kind));
        assert_eq!(kinds, [true, true, true, true, false]);
    }

    #[test]
    fn should_take_the_open_folders_from_init() {
        // given
        let init = "pub fn init(config) { config.open_folders = [\"processes\"]; }";

        // when
        let open = load(init).1.unwrap().open_folders;

        // then
        assert!(open.is_open(FolderKind::Processes));
        assert!(!open.is_open(FolderKind::Uncommitted));
    }

    #[test]
    fn should_exclude_target_and_node_modules_by_default() {
        // when
        let excluded = defaults().excluded;

        // then
        assert_eq!(excluded.names(), ["target", "node_modules"]);
    }

    #[test]
    fn should_let_init_remove_default_exclusion() {
        // given
        let init = r#"pub fn init(config) { config.excluded = config.excluded.iter().filter(|name| name != "node_modules").collect::<Vec>(); }"#;

        // when
        let excluded = load(init).1.unwrap().excluded;

        // then
        assert_eq!(excluded.names(), ["target"]);
    }

    #[test]
    fn should_reject_excluded_path_from_init() {
        // given
        let init = r#"pub fn init(config) { config.excluded.push("target/debug"); }"#;

        // when
        let error = load(init).1.err().unwrap();

        // then
        assert!(error.starts_with(r#"init.rn: excluded: "target/debug" is not a folder name"#), "{error}");
    }

    #[test]
    fn should_reject_excluded_under_theme() {
        // given
        let init = r#"pub fn init(config) { config.theme.excluded = ["dist"]; }"#;

        // when
        let error = load(init).1.err().unwrap();

        // then
        assert!(error.contains("theme.excluded") || error.contains("unknown field `excluded`"), "{error}");
    }

    #[test]
    fn should_load_the_readme_excluded_example() {
        // given
        let readme = include_str!("../../README.md");
        let section = &readme[readme.find("### Excluded folders").unwrap()..];
        let start = section.find("```rune\n").unwrap() + "```rune\n".len();
        let example = &section[start..start + section[start..].find("```").unwrap()];

        // when
        let excluded = load(example).1.unwrap().excluded;

        // then
        assert_eq!(excluded.names(), ["target", "dist"]);
    }

    #[test]
    fn should_override_one_syntax_key_from_init() {
        // given
        let init = r#"pub fn init(config) { config.theme.dark.syntax["CSS.COLOR"] = #{ foreground: "FF0000" }; }"#;

        // when
        let theme = load(init).1.unwrap().theme;

        // then
        assert_eq!(theme.dark.syntax["CSS.COLOR"].foreground.as_deref(), Some("FF0000"));
        assert!(theme.dark.syntax.contains_key("CSS.IMPORTANT"));
    }

    #[test]
    fn should_load_the_readme_theme_example() {
        // given
        let readme = include_str!("../../README.md");
        let section = &readme[readme.find("### Theme").unwrap()..];
        let start = section.find("```rune\n").unwrap() + "```rune\n".len();
        let example = &section[start..start + section[start..].find("```").unwrap()];

        // when
        let theme = load(example).1.unwrap().theme;

        // then
        // "1.75rem" at rem 18: 31.5, rounded
        assert_eq!((theme.dark.tree, theme.sizes.tree.row_height), (Rgb(0x202124), Height(32.)));
        assert_eq!(theme.dark.syntax["CSS.COLOR"].foreground.as_deref(), Some("FF0000"));
    }

    #[test]
    fn should_name_line_of_runtime_error_in_init() {
        // given
        let init = "pub fn init(config) {\n    config.theme.sizes.tre.row_height = 1.0;\n}";

        // when
        let error = load(init).1.err().unwrap();

        // then
        assert!(error.starts_with("init.rn:2:"), "{error}");
    }

    #[test]
    fn should_name_file_and_path_of_bad_theme_value() {
        // given
        let init = "pub fn init(config) { config.theme.dark.tree = 5; }";

        // when
        let error = load(init).1.err().unwrap();

        // then
        assert!(error.starts_with("init.rn: theme.dark.tree:"), "{error}");
    }

    #[test]
    #[cfg(not(target_os = "macos"))]
    fn should_close_with_ctrl_shift_w_and_stop_with_ctrl_shift_q_everywhere() {
        // when
        let keymap = defaults().keymap;

        // then
        // close_back_to_hub and stop_back_to_hub, ive.rn's first two functions
        for place in [Place::Terminal, Place::Editor, Place::Elsewhere] {
            assert_eq!(keymap.find_key(&pressed("ctrl-shift-w"), place).cloned(), Some(Action::Invoke { generation: 1, function: 0 }), "{place:?}");
            assert_eq!(keymap.find_key(&pressed("ctrl-shift-q"), place).cloned(), Some(Action::Invoke { generation: 1, function: 1 }), "{place:?}");
            assert_eq!(keymap.find_key(&pressed("ctrl-w"), place), None, "{place:?}");
        }
    }

    #[test]
    fn should_let_init_rebind_and_remove_the_close_and_stop_keys() {
        // given
        let init = r#"pub fn init(config) {
            config.keys = [
                #{ key: "F4", mods: "CTRL", action: act::CloseTab },
                #{ key: "q", mods: "CTRL|SHIFT", action: act::DisableDefaultAssignment },
            ];
        }"#;

        // when
        let keymap = load(init).1.unwrap().keymap;

        // then
        assert_eq!(keymap.find_key(&pressed("ctrl-f4"), Place::Terminal).cloned(), builtin(Builtin::CloseTab));
        assert_eq!(keymap.find_key(&pressed("ctrl-shift-q"), Place::Terminal), None);
    }

    #[test]
    fn should_flip_changes_layout_with_alt_shift_g_anywhere() {
        // when
        let keymap = defaults().keymap;

        // then
        let flip = builtin(Builtin::ToggleChangesLayout);
        assert_eq!(keymap.find_key(&pressed("alt-shift-g"), Place::Elsewhere).cloned(), flip);
        assert_eq!(keymap.find_key(&pressed("alt-shift-g"), Place::Terminal).cloned(), flip);
    }

    #[test]
    fn should_load_ive_rn_defaults() {
        // when
        let keymap = defaults().keymap;

        // then
        assert_eq!(keymap.find_key(&pressed("alt-f12"), Place::Elsewhere).cloned(), builtin(Builtin::OpenTerminal));
        assert_eq!(keymap.find_key(&pressed("secondary-,"), Place::Elsewhere).cloned(), builtin(Builtin::OpenSettings));
        assert_eq!(keymap.find_mouse(Scope::Tab, &mouse(MouseKind::Up, MouseButton::Middle, 1)).cloned(), builtin(Builtin::CloseTab));
        assert_eq!(keymap.find_mouse(Scope::Tab, &mouse(MouseKind::Up, MouseButton::Left, 1)).cloned(), builtin(Builtin::ActivateTab));
        assert_eq!(keymap.find_mouse(Scope::Tree, &mouse(MouseKind::Up, MouseButton::Left, 1)).cloned(), builtin(Builtin::ActivateRow));
        let word = builtin(Builtin::SelectTextAtMouseCursor(SelectionMode::Word));
        assert_eq!(keymap.find_mouse(Scope::Terminal, &mouse(MouseKind::Down, MouseButton::Left, 2)).cloned(), word);
        let paste = builtin(Builtin::PasteFrom(ClipboardTarget::Primary));
        assert_eq!(keymap.find_mouse(Scope::Terminal, &mouse(MouseKind::Up, MouseButton::Middle, 1)).cloned(), paste);
        assert_eq!(keymap.find_key(&pressed("secondary-z"), Place::Editor).cloned(), builtin(Builtin::Undo));
        assert_eq!(keymap.find_key(&pressed("shift-insert"), Place::Terminal).cloned(), builtin(Builtin::PasteFrom(ClipboardTarget::Clipboard)));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn should_bind_terminal_clipboard_to_ctrl_shift_on_linux() {
        // when
        let keymap = defaults().keymap;

        // then
        let (copy, paste) = (keymap.find_key(&pressed("ctrl-shift-c"), Place::Terminal), keymap.find_key(&pressed("ctrl-shift-v"), Place::Terminal));
        assert_eq!(copy.cloned(), builtin(Builtin::CopyTo(ClipboardTarget::Clipboard)));
        assert_eq!(paste.cloned(), builtin(Builtin::PasteFrom(ClipboardTarget::Clipboard)));
        let both = keymap.find_key(&pressed("ctrl-alt-shift-v"), Place::Terminal);
        assert_eq!(both.cloned(), builtin(Builtin::Paste(PasteKind::TextAndImage, ClipboardTarget::Clipboard)));
    }

    #[test]
    fn should_bind_selection_clicks_in_tree() {
        // given
        let keymap = defaults().keymap;
        let up = mouse(MouseKind::Up, MouseButton::Left, 1);
        let secondary = Modifiers { control: cfg!(not(target_os = "macos")), platform: cfg!(target_os = "macos"), ..Modifiers::default() };

        // when
        let found = [secondary, Modifiers { shift: true, ..Modifiers::default() }]
            .map(|mods| keymap.find_mouse(Scope::Tree, &MouseTrigger { mods, ..up }).cloned());

        // then
        assert_eq!(found, [builtin(Builtin::ToggleRowSelection), builtin(Builtin::ExtendRowSelection)]);
    }

    #[test]
    fn should_reject_row_selection_outside_tree() {
        // given
        let init = r#"pub fn init(config) {
            config.mouse_bindings = [#{ event: #{ Up: #{ streak: 1, button: "Left" } }, mods: "SHIFT", action: act::ExtendRowSelection, scope: "terminal" }];
        }"#;

        // when
        let (_, loaded) = load(init);

        // then
        let error = loaded.err().unwrap();
        assert!(error.contains("ExtendRowSelection") && error.contains("terminal"), "{error}");
    }

    #[test]
    fn should_report_no_init_without_init_rn() {
        // when
        let loaded = defaults();

        // then
        assert!(!loaded.has_init);
        assert!(loaded.keymap.len() > 20);
    }

    #[test]
    fn should_load_send_binding() {
        // given
        let init = r#"pub fn init(config) { config.keys = [#{ key: "Enter", mods: "SHIFT", action: act::SendString("\x1b\r"), scope: "terminal" }]; }"#;

        // when
        let (_, loaded) = load(init);

        // then
        let loaded = loaded.unwrap();
        assert!(loaded.has_init);
        assert_eq!(loaded.keymap.find_key(&pressed("shift-enter"), Place::Terminal), Some(&Action::Send("\u{1b}\r".to_owned())));
    }

    #[test]
    fn should_bind_function_from_init() {
        // given
        let init = "pub fn init(config) { config.keys = [#{ key: \"r\", mods: \"CTRL|ALT\", action: restart }]; }\nasync fn restart(context) {}";

        // when
        let (_, loaded) = load(init);

        // then
        // ive.rn's own functions come first
        let loaded = loaded.unwrap();
        let defaults = defaults().functions.len();
        assert_eq!(loaded.keymap.find_key(&pressed("ctrl-alt-r"), Place::Elsewhere), Some(&Action::Invoke { generation: 1, function: defaults }));
        assert_eq!(loaded.functions.len(), defaults + 1);
    }

    #[test]
    fn should_override_default_from_init() {
        // given
        let init = r#"pub fn init(config) {
            config.mouse_bindings = [#{ event: #{ Up: #{ streak: 1, button: "Middle" } }, action: act::DisableDefaultAssignment, scope: "tab" }];
        }"#;

        // when
        let (_, loaded) = load(init);

        // then
        let keymap = loaded.unwrap().keymap;
        assert_eq!(keymap.find_mouse(Scope::Tab, &mouse(MouseKind::Up, MouseButton::Middle, 1)), None);
        assert_eq!(keymap.find_mouse(Scope::Tab, &mouse(MouseKind::Up, MouseButton::Left, 1)).cloned(), builtin(Builtin::ActivateTab));
    }

    #[test]
    fn should_bind_ctrl_click_to_close_tab() {
        // given
        let init = r#"pub fn init(config) {
            config.mouse_bindings = [#{ event: #{ Up: #{ streak: 1, button: "Left" } }, mods: "CTRL", action: act::CloseTab, scope: "tab" }];
        }"#;

        // when
        let (_, loaded) = load(init);

        // then
        let keymap = loaded.unwrap().keymap;
        let ctrl = MouseTrigger { mods: Modifiers { control: true, ..Modifiers::default() }, ..mouse(MouseKind::Up, MouseButton::Left, 1) };
        assert_eq!(keymap.find_mouse(Scope::Tab, &ctrl).cloned(), builtin(Builtin::CloseTab));
    }

    #[test]
    fn should_name_file_and_entry_in_errors() {
        // given
        let init = r#"pub fn init(config) { config.keys = [#{ key: "a", action: act::Nop }, #{ key: "Banana", action: act::Nop }]; }"#;

        // when
        let (_, loaded) = load(init);

        // then
        let error = loaded.err().unwrap();
        assert!(error.starts_with("init.rn: keys[1]:") && error.contains("Banana"), "{error}");
    }

    #[test]
    fn should_reject_action_in_wrong_scope() {
        // given
        let init = r#"pub fn init(config) {
            config.mouse_bindings = [#{ event: #{ Up: #{ streak: 1, button: "Left" } }, action: act::ActivateRow, scope: "tab" }];
        }"#;

        // when
        let (_, loaded) = load(init);

        // then
        let error = loaded.err().unwrap();
        assert!(error.contains("ActivateRow") && error.contains("tab"), "{error}");
    }

    #[test]
    fn should_bind_double_click_on_a_tab_to_keep_it() {
        // when
        let keymap = defaults().keymap;

        // then
        assert_eq!(keymap.find_mouse(Scope::Tab, &mouse(MouseKind::Up, MouseButton::Left, 2)).cloned(), builtin(Builtin::KeepTab));
    }

    #[test]
    fn should_bind_right_press_on_a_tab_to_the_default_menu() {
        // when
        let keymap = defaults().keymap;

        // then
        let found = keymap.find_mouse(Scope::Tab, &mouse(MouseKind::Down, MouseButton::Right, 1)).cloned();
        assert_eq!(found, Some(Action::ShowTabMenu("main".to_owned())));
    }

    #[test]
    fn should_load_the_default_tab_menu() {
        // when
        let menus = defaults().tab_menus;

        // then
        let item = MenuEntry::Item;
        let expected = [
            item(Builtin::KeepTab),
            item(Builtin::StopProgram),
            MenuEntry::Separator,
            item(Builtin::CloseTab),
            item(Builtin::CloseOtherTabs),
            item(Builtin::CloseTabsToTheLeft),
            item(Builtin::CloseTabsToTheRight),
        ];
        assert_eq!(menus.get("main"), Some(&expected[..]));
    }

    #[test]
    fn should_let_init_add_a_tab_menu_shown_by_shift_right_press() {
        // given
        let init = r#"pub fn init(config) {
            config.tab_menus.close = [act::CloseOtherTabs, act::CloseTabsToTheRight];
            config.mouse_bindings = [
                #{ event: #{ Down: #{ streak: 1, button: "Right" } }, mods: "SHIFT", action: act::ShowTabMenu("close"), scope: "tab" },
            ];
        }"#;

        // when
        let loaded = load(init).1.unwrap();

        // then
        let shift = MouseTrigger { mods: Modifiers { shift: true, ..Modifiers::default() }, ..mouse(MouseKind::Down, MouseButton::Right, 1) };
        assert_eq!(loaded.keymap.find_mouse(Scope::Tab, &shift).cloned(), Some(Action::ShowTabMenu("close".to_owned())));
        assert_eq!(loaded.tab_menus.get("close"), Some(&[MenuEntry::Item(Builtin::CloseOtherTabs), MenuEntry::Item(Builtin::CloseTabsToTheRight)][..]));
        assert!(loaded.tab_menus.get("main").is_some());
    }

    #[test]
    fn should_reject_a_tab_menu_item_that_is_not_a_tab_action() {
        // given
        let init = "pub fn init(config) { config.tab_menus.mine = [act::CloseTab, act::OpenSettings]; }";

        // when
        let error = load(init).1.err().unwrap();

        // then
        assert!(error.contains("mine") && error.contains("OpenSettings"), "{error}");
    }

    #[test]
    fn should_reject_showing_an_undeclared_tab_menu() {
        // given
        let init = r#"pub fn init(config) {
            config.mouse_bindings = [#{ event: #{ Down: #{ streak: 1, button: "Right" } }, action: act::ShowTabMenu("nope"), scope: "tab" }];
        }"#;

        // when
        let error = load(init).1.err().unwrap();

        // then
        assert!(error.contains("\"nope\""), "{error}");
    }

    #[test]
    fn should_reject_show_tab_menu_on_a_key() {
        // given
        let init = r#"pub fn init(config) { config.keys = [#{ key: "m", mods: "ALT", action: act::ShowTabMenu("main") }]; }"#;

        // when
        let error = load(init).1.err().unwrap();

        // then
        assert!(error.contains("ShowTabMenu") && error.contains("window"), "{error}");
    }

    #[test]
    fn should_bind_tab_menu_actions_to_keys() {
        // given
        let init = r#"pub fn init(config) { config.keys = [#{ key: "k", mods: "CTRL|ALT", action: act::KillProgram }]; }"#;

        // when
        let keymap = load(init).1.unwrap().keymap;

        // then
        assert_eq!(keymap.find_key(&pressed("ctrl-alt-k"), Place::Elsewhere).cloned(), builtin(Builtin::KillProgram));
    }

    #[test]
    fn should_bind_middle_click_on_a_tree_row_to_activate_it_to_keep() {
        // when
        let keymap = defaults().keymap;

        // then
        assert_eq!(keymap.find_mouse(Scope::Tree, &mouse(MouseKind::Up, MouseButton::Middle, 1)).cloned(), builtin(Builtin::ActivateRowToKeep));
    }

    #[test]
    fn should_reject_activate_row_to_keep_on_a_tab() {
        // given
        let init = r#"pub fn init(config) {
            config.mouse_bindings = [#{ event: #{ Up: #{ streak: 1, button: "Middle" } }, action: act::ActivateRowToKeep, scope: "tab" }];
        }"#;

        // when
        let (_, loaded) = load(init);

        // then
        let error = loaded.err().unwrap();
        assert!(error.contains("ActivateRowToKeep") && error.contains("tab"), "{error}");
    }

    #[test]
    fn should_reject_image_paste_outside_terminal() {
        // given
        let init = r#"pub fn init(config) {
            config.keys = [#{ key: "v", mods: "CTRL|ALT", action: act::PasteImageFrom("Clipboard"), scope: "editor" }];
        }"#;

        // when
        let (_, loaded) = load(init);

        // then
        let error = loaded.err().unwrap();
        assert!(error.contains("PasteImageFrom") && error.contains("editor"), "{error}");
    }

    #[test]
    fn should_bind_text_and_image_pastes_from_init() {
        // given
        let init = r#"pub fn init(config) {
            config.keys = [
                #{ key: "t", mods: "CTRL|ALT", action: act::PasteTextFrom("Clipboard"), scope: "terminal" },
                #{ key: "i", mods: "CTRL|ALT", action: act::PasteImageFrom("PrimarySelection"), scope: "terminal" },
            ];
        }"#;

        // when
        let (_, loaded) = load(init);

        // then
        let keymap = loaded.unwrap().keymap;
        let found = [pressed("ctrl-alt-t"), pressed("ctrl-alt-i")].map(|keys| keymap.find_key(&keys, Place::Terminal).cloned());
        assert_eq!(found, [
            builtin(Builtin::Paste(PasteKind::Text, ClipboardTarget::Clipboard)),
            builtin(Builtin::Paste(PasteKind::Image, ClipboardTarget::Primary)),
        ]);
    }

    #[test]
    fn should_reject_mouse_action_on_key() {
        // given
        let init = r#"pub fn init(config) { config.keys = [#{ key: "F5", action: act::SelectTextAtMouseCursor("Word"), scope: "terminal" }]; }"#;

        // when
        let (_, loaded) = load(init);

        // then
        assert!(loaded.err().unwrap().contains("SelectTextAtMouseCursor"));
    }

    #[test]
    fn should_require_scope_for_mouse_binding() {
        // given
        let init = r#"pub fn init(config) { config.mouse_bindings = [#{ event: #{ Up: #{ streak: 1, button: "Left" } }, action: act::Nop }]; }"#;

        // when
        let (_, loaded) = load(init);

        // then
        assert!(loaded.err().unwrap().contains("scope"));
    }

    #[test]
    fn should_reject_unknown_field() {
        // given
        let init = r#"pub fn init(config) { config.keys = [#{ key: "a", modz: "CTRL", action: act::Nop }]; }"#;

        // when
        let (_, loaded) = load(init);

        // then
        assert!(loaded.err().unwrap().contains("modz"));
    }

    #[test]
    fn should_reject_same_trigger_twice_in_init() {
        // given
        let init = r#"pub fn init(config) { config.keys = [#{ key: "F5", action: act::Nop }, #{ key: "F5", action: act::OpenSettings }]; }"#;

        // when
        let (_, loaded) = load(init);

        // then
        let error = loaded.err().unwrap();
        assert!(error.starts_with("init.rn:") && error.contains("f5"), "{error}");
    }

    #[test]
    fn should_tell_os_to_scripts() {
        // given
        let init = r#"pub fn init(config) { if ive::os() == "plan9" { config.keys = [#{ key: "Banana", action: act::Nop }]; } }"#;

        // when
        let (_, loaded) = load(init);

        // then
        assert!(loaded.is_ok());
    }

    #[test]
    fn should_tell_profile_to_scripts() {
        // given
        let init = r#"pub fn init(config) { if ive::profile() != "default" { config.keys = [#{ key: "Banana", action: act::Nop }]; } }"#;

        // when
        let (_, loaded) = load(init);

        // then
        assert!(loaded.is_ok(), "{:?}", loaded.err());
    }

    #[test]
    fn should_tell_config_dir_to_scripts() {
        // given
        let init = r#"pub fn init(config) {
            let dir = ive::config_dir().unwrap_or("none");
            if !dir.ends_with("/ive/default") { config.keys = [#{ key: "Banana", action: act::Nop }]; }
        }"#;

        // when
        let (_, loaded) = load(init);

        // then
        assert!(loaded.is_ok(), "{:?}", loaded.err());
    }

    #[test]
    fn should_report_syntax_error_with_line() {
        // given
        let init = "pub fn init(config) {\n    config.keys = [#{ key: \"F5\" action: act::Nop }];\n}";

        // when
        let (_, loaded) = load(init);

        // then
        let error = loaded.err().unwrap();
        assert!(error.starts_with("init.rn:2:"), "{error}");
    }

    #[test]
    fn should_report_error_returned_by_init() {
        // given
        let init = r#"pub fn init(config) { Err("nope") }"#;

        // when
        let (_, loaded) = load(init);

        // then
        assert!(loaded.err().unwrap().contains("nope"));
    }

    #[test]
    fn should_fail_load_when_init_exceeds_budget() {
        // given
        let started = Instant::now();

        // when
        let (_, loaded) = load("pub fn init(config) { loop {} }");

        // then
        assert!(loaded.err().unwrap().contains("instructions"));
        assert!(started.elapsed() < Duration::from_secs(5));
    }

    #[test]
    fn should_load_module_next_to_init() {
        // given
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("keys.rn"), r#"pub fn text() { "x" }"#).unwrap();
        fs::write(dir.path().join("init.rn"), "mod keys;\npub fn init(config) { config.keys = [#{ key: \"F5\", action: act::SendString(keys::text()) }]; }").unwrap();
        let engine = Engine::new(futures::channel::mpsc::unbounded().0).unwrap();

        // when
        let loaded = engine.load(Some(dir.path()), 1);

        // then
        let action = loaded.unwrap().keymap.find_key(&pressed("f5"), Place::Elsewhere).cloned();
        assert_eq!(action, Some(Action::Send("x".to_owned())));
    }

    /// The first `rune` example of the README's Scripting section.
    fn readme_example() -> String {
        let readme = include_str!("../../README.md");
        let section = &readme[readme.find("## Scripting").unwrap()..];
        let start = section.find("```rune\n").unwrap() + "```rune\n".len();
        section[start..start + section[start..].find("```").unwrap()].to_owned()
    }

    #[test]
    fn should_load_readme_example() {
        // when
        let (_, loaded) = load(&readme_example());

        // then
        let keymap = loaded.unwrap().keymap;
        let middle = mouse(MouseKind::Up, MouseButton::Middle, 1);
        assert_eq!(keymap.find_mouse(Scope::Tab, &middle), None);
        assert_eq!(keymap.find_key(&pressed("shift-enter"), Place::Terminal), Some(&Action::Send("\u{1b}\r".to_owned())));
    }

    #[test]
    fn should_capture_println() {
        // given
        let (engine, loaded) = load(r#"pub fn init(config) { println!("hi"); }"#);

        // when
        let output = engine.drain_output();

        // then
        assert!(loaded.is_ok());
        assert_eq!(output, vec!["hi".to_owned()]);
    }

    #[test]
    fn should_declare_bundled_fonts_in_ive_rn() {
        // when
        let fonts = defaults().fonts;

        // then
        assert_eq!(fonts, Fonts { ui: "Inter".into(), mono: "JetBrains Mono".into() });
    }

    #[test]
    fn should_override_one_font_from_init() {
        // given
        let init = r#"pub fn init(config) { config.font = "Fira Code"; }"#;

        // when
        let (_, loaded) = load(init);

        // then
        assert_eq!(loaded.unwrap().fonts, Fonts { ui: "Inter".into(), mono: "Fira Code".into() });
    }

    #[test]
    fn should_reject_font_that_is_not_a_name() {
        // given
        let init = r#"pub fn init(config) { config.ui_font = 12; }"#;

        // when
        let (_, loaded) = load(init);

        // then
        assert_eq!(loaded.err().unwrap(), "init.rn: ui_font must be a font family name such as \"Inter\", not 12");
    }

    #[test]
    fn should_bind_ctrl_shift_e_to_open_the_hub() {
        // given
        let open_hub = if cfg!(target_os = "macos") { "cmd-shift-e" } else { "ctrl-shift-e" };

        // when
        let action = defaults().keymap.find_key(&pressed(open_hub), Place::Elsewhere).cloned();

        // then
        assert_eq!(action, builtin(Builtin::OpenHub));
    }

    #[test]
    fn should_bind_ctrl_shift_t_to_open_terminal_in() {
        // given
        let open_terminal_in = if cfg!(target_os = "macos") { "cmd-shift-t" } else { "ctrl-shift-t" };

        // when
        let action = defaults().keymap.find_key(&pressed(open_terminal_in), Place::Terminal).cloned();

        // then
        assert_eq!(action, builtin(Builtin::OpenTerminalIn));
    }

    #[test]
    fn should_toggle_changed_with_ctrl_shift_e_in_the_hub() {
        // given
        let toggle_changed = if cfg!(target_os = "macos") { "cmd-shift-e" } else { "ctrl-shift-e" };

        // when
        let action = defaults().keymap.find_key(&pressed(toggle_changed), Place::Hub).cloned();

        // then
        assert_eq!(action, builtin(Builtin::HubToggleChanged));
    }

    #[test]
    fn should_open_the_third_row_with_alt_3() {
        // when
        let action = defaults().keymap.find_key(&pressed("alt-3"), Place::Hub).cloned();

        // then
        assert_eq!(action, builtin(Builtin::HubOpenRow(3)));
    }

    #[test]
    fn should_bind_the_hub_keys_of_the_spec() {
        // given
        let keymap = defaults().keymap;
        let expected = [
            ("down", Builtin::HubDown), ("ctrl-j", Builtin::HubDown), ("ctrl-n", Builtin::HubDown),
            ("up", Builtin::HubUp), ("ctrl-k", Builtin::HubUp), ("ctrl-p", Builtin::HubUp),
            ("pagedown", Builtin::HubPageDown), ("pageup", Builtin::HubPageUp),
            ("enter", Builtin::HubOpen), ("shift-enter", Builtin::HubOpenBehind), ("alt-enter", Builtin::HubTypePath),
            ("ctrl-y", Builtin::HubCopyPath), ("tab", Builtin::HubCycleScope), ("backspace", Builtin::HubWiden),
            ("alt-i", Builtin::HubToggleExcluded), ("escape", Builtin::HubBack), ("alt-9", Builtin::HubOpenRow(9)),
        ];

        // when
        let found: Vec<_> = expected.iter().map(|(key, _)| keymap.find_key(&pressed(key), Place::Hub).cloned()).collect();

        // then
        assert_eq!(found, expected.iter().map(|(_, action)| builtin(*action)).collect::<Vec<_>>());
    }

    #[test]
    fn should_fall_back_to_window_keys_in_the_hub() {
        // when
        let action = defaults().keymap.find_key(&pressed("secondary-,"), Place::Hub).cloned();

        // then
        assert_eq!(action, builtin(Builtin::OpenSettings));
    }

    #[test]
    fn should_not_run_hub_keys_outside_the_hub() {
        // when
        let action = defaults().keymap.find_key(&pressed("down"), Place::Elsewhere).cloned();

        // then
        assert_eq!(action, None);
    }

    #[test]
    fn should_reject_a_hub_row_out_of_range() {
        // given
        let init = r#"pub fn init(config) { config.keys = [#{ key: "0", mods: "ALT", action: act::HubOpenRow(10), scope: "hub" }]; }"#;

        // when
        let error = load(init).1.err().unwrap();

        // then
        assert!(error.contains("1 to 9") && error.contains("10"), "{error}");
    }

    #[test]
    fn should_reject_a_hub_action_in_the_window_scope() {
        // given
        let init = r#"pub fn init(config) { config.keys = [#{ key: "j", mods: "CTRL|ALT", action: act::HubDown }]; }"#;

        // when
        let error = load(init).1.err().unwrap();

        // then
        assert!(error.contains("HubDown does nothing in the window scope"), "{error}");
    }

    #[test]
    fn should_bind_copy_full_path_in_the_hub_and_the_tree() {
        // given
        let copy = if cfg!(target_os = "macos") { "cmd-alt-c" } else { "ctrl-shift-c" };
        let keymap = defaults().keymap;

        // when
        let found = [Place::Hub, Place::Tree].map(|place| keymap.find_key(&pressed(copy), place).cloned());

        // then
        assert_eq!(found, [builtin(Builtin::CopyFullPath), builtin(Builtin::CopyFullPath)]);
    }

    #[test]
    fn should_keep_the_terminal_copy_key_in_a_terminal() {
        // when
        let action = defaults().keymap.find_key(&pressed(if cfg!(target_os = "macos") { "cmd-c" } else { "ctrl-shift-c" }), Place::Terminal).cloned();

        // then
        assert_eq!(action, builtin(Builtin::CopyTo(ClipboardTarget::Clipboard)));
    }

    #[test]
    fn should_reject_a_row_click_action_on_a_tree_key() {
        // given
        let init = r#"pub fn init(config) { config.keys = [#{ key: "Enter", action: act::ActivateRow, scope: "tree" }]; }"#;

        // when
        let error = load(init).1.err().unwrap();

        // then
        assert!(error.contains("ActivateRow works only in mouse_bindings"), "{error}");
    }

    #[test]
    fn should_let_init_replace_a_hub_key() {
        // given
        let init = r#"pub fn init(config) { config.keys = [#{ key: "j", mods: "CTRL", action: act::DisableDefaultAssignment, scope: "hub" }]; }"#;

        // when
        let keymap = load(init).1.unwrap().keymap;

        // then
        assert_eq!(keymap.find_key(&pressed("ctrl-j"), Place::Hub), None);
    }

    #[test]
    fn should_load_hub_as_a_temporary_kind() {
        // when
        let kinds = defaults().temporary_tabs.kinds;

        // then
        assert!(kinds.contains(&TabKind::Hub), "{kinds:?}");
    }

    #[test]
    fn should_bind_the_close_keys_to_the_back_to_hub_functions() {
        // given
        let keymap = defaults().keymap;
        let (close, stop) = if cfg!(target_os = "macos") { ("cmd-w", "cmd-alt-w") } else { ("ctrl-shift-w", "ctrl-shift-q") };

        // when
        let keys = [close, stop].map(|key| keymap.find_key(&pressed(key), Place::Elsewhere).cloned());
        let middle_click = keymap.find_mouse(Scope::Tab, &mouse(MouseKind::Up, MouseButton::Middle, 1)).cloned();

        // then
        assert!(keys.iter().all(|action| matches!(action, Some(Action::Invoke { .. }))), "{keys:?}");
        assert_eq!(middle_click, builtin(Builtin::CloseTab));
    }

    #[test]
    fn should_bind_the_stop_key_to_hub_stop_in_the_hub() {
        // given
        let stop = if cfg!(target_os = "macos") { "cmd-alt-w" } else { "ctrl-shift-q" };

        // when
        let action = defaults().keymap.find_key(&pressed(stop), Place::Hub).cloned();

        // then
        assert_eq!(action, builtin(Builtin::HubStop));
    }
}
