//! Waiting for what another program still holds: a restarting IVE its terminal registry and MCP port, a reader on
//! Windows the file IVE replaces.

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

/// Moves `staging` over `path`.
#[cfg(unix)]
pub fn replace_file(staging: &std::path::Path, path: &std::path::Path) -> std::io::Result<()> {
    std::fs::rename(staging, path)
}

/// Moves `staging` over `path`. Windows refuses while another program has `path` open without delete sharing, as an
/// agent reading `mcp.toml` may, so a refused move is tried again for up to half a second.
#[cfg(windows)]
pub fn replace_file(staging: &std::path::Path, path: &std::path::Path) -> std::io::Result<()> {
    const ERROR_ACCESS_DENIED: i32 = 5;
    const ERROR_SHARING_VIOLATION: i32 = 32;
    let mut result = std::fs::rename(staging, path);
    for _ in 0..10 {
        match &result {
            Err(error) if matches!(error.raw_os_error(), Some(ERROR_ACCESS_DENIED | ERROR_SHARING_VIOLATION)) => {
                std::thread::sleep(Duration::from_millis(50));
                result = std::fs::rename(staging, path);
            }
            _ => break,
        }
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

    #[cfg(windows)]
    #[test]
    fn should_replace_a_file_once_its_reader_closes_it() {
        // given
        use std::os::windows::fs::OpenOptionsExt;
        let dir = tempfile::tempdir().unwrap();
        let (staging, path) = (dir.path().join("a.tmp"), dir.path().join("a.toml"));
        std::fs::write(&staging, "new").unwrap();
        std::fs::write(&path, "old").unwrap();
        // read sharing only, as a program that opens a file without delete sharing holds it
        let reader = std::fs::OpenOptions::new().read(true).share_mode(1).open(&path).unwrap();
        let closer = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(150));
            drop(reader);
        });

        // when
        let result = replace_file(&staging, &path);

        // then
        closer.join().unwrap();
        assert!(result.is_ok(), "{result:?}");
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "new");
    }
}
