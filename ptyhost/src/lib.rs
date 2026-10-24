//! Keeps terminal programs running while IVE is closed, tmux-style: a detached host process owns each PTY and
//! serves it over a Unix socket; clients attach, detach and attach again, seeing recent output replayed.
//!
//! Linux and macOS: the host learns that its program exited through a pidfd, or a kqueue on macOS.

pub mod cli;
pub mod client;
pub mod host;
mod modes;
pub mod protocol;
pub mod registry;
#[cfg(target_os = "linux")]
mod scope;
mod title;
