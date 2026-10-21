//! Waiting for what an IVE that is restarting still holds: its terminal registry and its MCP port.

use std::time::Duration;

/// How long a restarted IVE waits for the one it replaces to exit: 50 attempts, 200 ms apart, 10 s in all.
pub const RESTART_ATTEMPTS: u32 = 50;
pub const RESTART_INTERVAL: Duration = Duration::from_millis(200);

/// Calls `attempt` until it succeeds, at most `attempts` times, waiting `interval` in between; the last error
/// otherwise.
pub fn retry<T, E>(attempts: u32, interval: Duration, mut attempt: impl FnMut() -> Result<T, E>) -> Result<T, E> {
    let mut result = attempt();
    for _ in 1..attempts {
        if result.is_ok() {
            break;
        }
        std::thread::sleep(interval);
        result = attempt();
    }
    result
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;

    #[test]
    fn should_retry_until_the_attempt_succeeds() {
        // given
        let calls = Cell::new(0);

        // when
        let result: Result<u32, &str> = retry(5, Duration::ZERO, || {
            calls.set(calls.get() + 1);
            if calls.get() < 3 { Err("busy") } else { Ok(calls.get()) }
        });

        // then
        assert_eq!(result, Ok(3));
    }

    #[test]
    fn should_give_the_last_error_after_the_last_attempt() {
        // given
        let calls = Cell::new(0);

        // when
        let result: Result<(), u32> = retry(4, Duration::ZERO, || {
            calls.set(calls.get() + 1);
            Err(calls.get())
        });

        // then
        assert_eq!(result, Err(4));
    }
}
