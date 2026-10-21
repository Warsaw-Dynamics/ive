//! Run configurations: `.run.toml` files declaring what a folder can run (commands and static servers), the
//! values they need (`vars.toml`, the OS wallet), and the static server itself.

pub mod chain;
pub mod configs_list;
pub mod configs_tab;
pub mod file;
pub mod link_bar;
pub mod serve;
#[cfg(unix)]
pub mod signal;
pub mod values;
pub mod values_tab;
pub mod wallet;
