//! The sockets a host listens on and a client connects through: std's on Unix, `uds_windows`'s (the same API) on
//! Windows.

#[cfg(unix)]
pub use std::os::unix::net::{UnixListener, UnixStream};
#[cfg(windows)]
pub use uds_windows::{UnixListener, UnixStream};
