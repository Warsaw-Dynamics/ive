//! Downloads through the system's `curl`, like scripts' `http`: TLS, proxies and the OS's certificates come with it.

use std::path::Path;
use std::process::{Command, Output};

/// The release description is small; a minute covers a slow network.
const TEXT_SECONDS: &str = "60";
/// A release archive over a slow network.
const FILE_SECONDS: &str = "1800";

/// The body at `url` as text.
pub fn fetch_text(url: &str) -> Result<String, String> {
    let output = curl(url, &["--max-time", TEXT_SECONDS])?;
    String::from_utf8(output.stdout).map_err(|_| format!("cannot download {url}: the answer is not UTF-8"))
}

/// Saves the body at `url` to `to`, refusing one over `max_bytes`.
pub fn fetch_file(url: &str, to: &Path, max_bytes: u64) -> Result<(), String> {
    let max = max_bytes.to_string();
    let target = to.to_string_lossy();
    curl(url, &["--max-time", FILE_SECONDS, "--max-filesize", &max, "--output", &target]).map(drop)
}

fn curl(url: &str, args: &[&str]) -> Result<Output, String> {
    // --disable first: no ~/.curlrc changes the request; --url keeps a URL from reading as an option
    let output = Command::new("curl")
        .args(["--disable", "--fail", "--silent", "--show-error", "--location", "--user-agent", "ive"])
        .args(args)
        .args(["--url", url])
        .output()
        .map_err(|error| format!("cannot download {url}: curl does not start: {error}"))?;
    if output.status.success() {
        return Ok(output);
    }
    let reason = String::from_utf8_lossy(&output.stderr);
    Err(format!("cannot download {url}: {} ({})", reason.trim(), output.status))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_fetch_a_file_url() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("release.json");
        std::fs::write(&source, "{\"tag_name\": \"x\"}").unwrap();
        let url = format!("file://{}", source.display());
        let target = dir.path().join("copy");

        // when
        let text = fetch_text(&url);
        let file = fetch_file(&url, &target, 1024);

        // then
        assert_eq!(text, Ok("{\"tag_name\": \"x\"}".to_owned()));
        assert_eq!(file, Ok(()));
        assert_eq!(std::fs::read_to_string(&target).unwrap(), "{\"tag_name\": \"x\"}");
    }

    #[test]
    fn should_report_a_failed_fetch_with_its_url() {
        // given
        let url = "file:///nonexistent/ive-update-test";

        // when
        let text = fetch_text(url);

        // then
        assert!(text.unwrap_err().contains(url));
    }
}
