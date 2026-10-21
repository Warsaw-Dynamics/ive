//! Keeps terminal programs running while IVE is closed, tmux-style: a detached host process owns each PTY and
//! serves it over a Unix socket; clients attach, detach and attach again, seeing recent output replayed.
//!
//! Linux only: the host learns that its program exited through a pidfd.

pub mod cli;
pub mod client;
pub mod host;
pub mod protocol;
pub mod registry;
mod scope;
mod title;
