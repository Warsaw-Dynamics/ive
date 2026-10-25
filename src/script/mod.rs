//! User scripts in Rune, from IVE's config folder: key bindings and functions that call [`crate::api`].
//! They run on their own thread; the gpui thread sees only the plain-data [`keymap::Keymap`].

mod api_module;
mod config;
mod declare;
pub mod defaults_file;
pub mod engine;
pub mod excluded;
pub mod open_folders;
mod fingerprint;
mod http_module;
pub mod keymap;
pub mod delete_mode;
pub mod quit_countdown;
pub mod drop_modifiers;
pub mod log;
pub mod state;
pub mod menus;
pub mod temporary_tabs;
mod thread;
mod toml_module;

pub use thread::{ScriptCommand, ScriptEvent, Scripts, Setup, start};

use std::path::PathBuf;

/// The profile's folder, e.g. `~/.config/ive/default/`, where `init.rn` lives.
pub fn default_dir() -> Option<PathBuf> {
    crate::profile::current().config_dir()
}
