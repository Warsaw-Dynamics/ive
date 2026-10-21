//! Opt-in diagnostics: `IVE_LOG=debug ive` prints IVE's spans and events, and the `log` records of gpui and other
//! libraries, to stderr; a closing span prints how long it ran. The value is a tracing-subscriber filter, e.g.
//! `ive=debug,warn`. Without `IVE_LOG` nothing is installed and every span costs a branch.

use std::sync::OnceLock;
use std::time::{Duration, Instant};

use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::format::FmtSpan;

pub const ENV: &str = "IVE_LOG";

static START: OnceLock<Instant> = OnceLock::new();

/// Call first in `main`: startup times count from here.
pub fn init() {
    START.get_or_init(Instant::now);
    let Some(filter) = std::env::var_os(ENV) else {
        return;
    };
    let filter = EnvFilter::try_new(filter.to_string_lossy()).unwrap_or_else(|error| {
        eprintln!("ive: {ENV}={}: {error}; logging warnings only", filter.to_string_lossy());
        EnvFilter::new("warn")
    });
    let installed = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .with_span_events(FmtSpan::CLOSE)
        .with_thread_names(true)
        .try_init();
    if let Err(error) = installed {
        eprintln!("ive: {ENV} ignored: {error}");
    }
}

pub fn since_start() -> Duration {
    START.get().map_or(Duration::ZERO, Instant::elapsed)
}

/// Logs how long IVE took to reach `milestone`, with its resident memory where the OS reports it.
pub fn milestone(milestone: &str) {
    let elapsed_ms = since_start().as_millis();
    match resident_mib() {
        Some(rss_mib) => tracing::info!(elapsed_ms, rss_mib, "{milestone}"),
        None => tracing::info!(elapsed_ms, "{milestone}"),
    }
}

#[cfg(target_os = "linux")]
fn resident_mib() -> Option<u64> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    let kib = status.lines().find_map(|line| line.strip_prefix("VmRSS:"))?;
    Some(kib.trim().trim_end_matches("kB").trim().parse::<u64>().ok()? / 1024)
}

#[cfg(not(target_os = "linux"))]
fn resident_mib() -> Option<u64> {
    None
}
