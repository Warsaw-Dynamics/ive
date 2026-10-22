//! What `ive.rn` and `init.rn` see: the `config` object `init` fills in, the `act` module of actions in WezTerm's
//! shape, and `ive::os`.

use std::collections::HashMap;

use rune::{Any, ContextError, Module, Value};

/// Actions without an argument: in WezTerm's Lua these are plain strings, and so they are here.
const UNIT_ACTIONS: [&str; 40] = [
    "Nop", "DisableDefaultAssignment", "OpenSettings", "OpenTerminal", "ActivateTab", "CloseTab", "KeepTab", "StopProgram", "KillProgram",
    "CloseOtherTabs", "CloseTabsToTheLeft", "CloseTabsToTheRight", "ToggleChangesLayout", "OpenTerminalIn", "ActivateRow", "ActivateRowToKeep",
    "ToggleRow", "ToggleRowSelection", "ExtendRowSelection", "SelectAll", "Cut", "Undo", "Redo", "ToggleDiffLayout",
    "OpenHub", "HubDown", "HubUp", "HubPageDown", "HubPageUp", "HubOpen", "HubOpenBehind", "HubTypePath", "HubCopyPath",
    "HubStop", "CopyFullPath", "HubCycleScope", "HubWiden", "HubToggleExcluded", "HubToggleChanged", "HubBack",
];

/// `config` in `pub fn init(config)`: the lists of bindings, as WezTerm's `config.keys` and `config.mouse_bindings`,
/// the font families: `config.font` for code and terminals, as WezTerm's, and `config.ui_font` for the rest, and
/// `config.theme`, `config.excluded`, `config.temporary_tabs`, `config.tab_menus` and `config.quit_countdown`, which
/// start as the script before left them.
#[derive(Any)]
#[rune(item = ::ive)]
pub struct Config {
    #[rune(get, set)]
    keys: Value,
    #[rune(get, set)]
    mouse_bindings: Value,
    #[rune(get, set)]
    font: Value,
    #[rune(get, set)]
    ui_font: Value,
    #[rune(get, set)]
    theme: Value,
    #[rune(get, set)]
    excluded: Value,
    #[rune(get, set)]
    temporary_tabs: Value,
    #[rune(get, set)]
    tab_menus: Value,
    #[rune(get, set)]
    quit_countdown: Value,
}

/// What `init` left in `config`; anything it did not set is `()`.
pub struct Parts {
    pub keys: Value,
    pub mouse_bindings: Value,
    pub font: Value,
    pub ui_font: Value,
    pub carried: Carried,
}

/// What one script hands the next in `config`, checked only once the last script ran.
pub struct Carried {
    pub theme: Value,
    pub excluded: Value,
    pub temporary_tabs: Value,
    pub tab_menus: Value,
    pub quit_countdown: Value,
}

impl Config {
    /// A `config` with nothing declared yet but what the script before left in `carried`.
    pub fn new(carried: Carried) -> Self {
        let Carried { theme, excluded, temporary_tabs, tab_menus, quit_countdown } = carried;
        Self { keys: Value::from(()), mouse_bindings: Value::from(()), font: Value::from(()), ui_font: Value::from(()), theme, excluded, temporary_tabs, tab_menus, quit_countdown }
    }

    pub fn into_parts(self) -> Parts {
        let carried = Carried { theme: self.theme, excluded: self.excluded, temporary_tabs: self.temporary_tabs, tab_menus: self.tab_menus, quit_countdown: self.quit_countdown };
        Parts { keys: self.keys, mouse_bindings: self.mouse_bindings, font: self.font, ui_font: self.ui_font, carried }
    }
}

/// An action with an argument, `#{ <name>: argument }`, as WezTerm's `act.CopyTo 'Clipboard'` table.
fn tagged(name: &str, argument: &str) -> HashMap<String, String> {
    HashMap::from([(name.to_owned(), argument.to_owned())])
}

/// The `ive` module: the `config` type, `os`, `profile` and `config_dir` (the folder of `init.rn`; none when the OS
/// reports no config folder).
pub fn module() -> Result<Module, ContextError> {
    let mut module = Module::with_crate("ive")?;
    module.ty::<Config>()?;
    module.function("os", || std::env::consts::OS.to_owned()).build()?;
    module.function("profile", || crate::profile::current().name().to_owned()).build()?;
    module.function("config_dir", || super::default_dir().map(|dir| dir.to_string_lossy().into_owned())).build()?;
    Ok(module)
}

/// The `act` module: WezTerm's action names, plus IVE's own.
pub fn act_module() -> Result<Module, ContextError> {
    let mut module = Module::with_crate("act")?;
    for name in UNIT_ACTIONS {
        module.constant(name, name).build()?;
    }
    module.function("SendString", |text: &str| tagged("SendString", text)).build()?;
    module.function("ShowTabMenu", |name: &str| tagged("ShowTabMenu", name)).build()?;
    module.function("CopyTo", |target: &str| tagged("CopyTo", target)).build()?;
    module.function("PasteFrom", |source: &str| tagged("PasteFrom", source)).build()?;
    module.function("PasteTextFrom", |source: &str| tagged("PasteTextFrom", source)).build()?;
    module.function("PasteImageFrom", |source: &str| tagged("PasteImageFrom", source)).build()?;
    module.function("PasteTextAndImageFrom", |source: &str| tagged("PasteTextAndImageFrom", source)).build()?;
    module.function("SelectTextAtMouseCursor", |mode: &str| tagged("SelectTextAtMouseCursor", mode)).build()?;
    module.function("ExtendSelectionToMouseCursor", |mode: &str| tagged("ExtendSelectionToMouseCursor", mode)).build()?;
    module.function("CompleteSelection", |target: &str| tagged("CompleteSelection", target)).build()?;
    module.function("HubOpenRow", |row: i64| HashMap::from([("HubOpenRow".to_owned(), row)])).build()?;
    Ok(module)
}
