//! Keeps terminal programs running while IVE is closed, tmux-style: a detached host process owns each PTY and
//! serves it over a Unix socket; clients attach, detach and attach again, seeing recent output replayed.
//!
//! Linux and macOS: the host learns that its program exited through a pidfd, or a kqueue on macOS. Windows: a
//! ConPTY in place of the PTY, a job object in place of the process groups, and AF_UNIX sockets as elsewhere.

pub mod cli;
pub mod client;
pub mod host;
#[cfg(windows)]
pub mod job;
mod modes;
pub mod net;
pub mod protocol;
pub mod registry;
#[cfg(target_os = "linux")]
mod scope;
mod title;
