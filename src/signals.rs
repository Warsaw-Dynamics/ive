//! SIGTERM, SIGINT and SIGUSR1 as messages to the workspace, so IVE can ask before quitting instead of dying at once,
//! and quit without asking when a restart script replaces it.

use std::io;

use futures::channel::mpsc::{UnboundedReceiver, unbounded};
use signal_hook::consts::{SIGINT, SIGTERM, SIGUSR1};
use signal_hook::iterator::Signals;

/// Catches SIGTERM, SIGINT and SIGUSR1 from now on and forwards each, by number, on the returned receiver. Caught signals
/// return to their default in programs IVE starts, so terminals are unaffected.
pub fn forward() -> io::Result<UnboundedReceiver<i32>> {
    let mut signals = Signals::new([SIGTERM, SIGINT, SIGUSR1])?;
    let (sender, receiver) = unbounded();
    std::thread::Builder::new().name("ive-signals".to_owned()).spawn(move || {
        for signal in signals.forever() {
            if sender.unbounded_send(signal).is_err() {
                return;
            }
        }
    })?;
    Ok(receiver)
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::*;

    #[test]
    fn should_forward_sigterm_to_receiver() {
        // given
        let mut signals = forward().unwrap();

        // when
        // SAFETY: the handler `forward` installed catches it
        unsafe { libc::raise(SIGTERM) };

        // then
        let deadline = Instant::now() + Duration::from_secs(2);
        let received = loop {
            if let Ok(signal) = signals.try_recv() {
                break Some(signal);
            }
            if Instant::now() > deadline {
                break None;
            }
            std::thread::sleep(Duration::from_millis(10));
        };
        assert_eq!(received, Some(SIGTERM));
    }
}
