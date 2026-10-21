//! Where a profile's MCP server listens and the token it asks for, remembered in `mcp.toml` so a client registered
//! once (`claude mcp add`) keeps working after IVE restarts, and so agents can find a running IVE.

use std::fs;
use std::io::{self, Write as _};
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::Token;
use crate::args::MIN_PORT;
use crate::retry;
use crate::state;

/// Where a profile's server first tries to listen; when that is taken, the OS picks a free port.
pub const PREFERRED_PORT: u16 = 7461;
/// A file this big is not something IVE wrote.
const MAX_FILE_BYTES: u64 = 4096;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Remembered {
    port: u16,
    token: String,
}

/// What [`open`] serves.
pub struct Request<'a> {
    /// `--mcp PORT`; `None` serves the remembered port, or picks one on the first run
    pub given: Option<u16>,
    /// `mcp.toml`; `None` when the OS reports no config folder, so nothing is remembered
    pub path: Option<&'a Path>,
    /// `IVE_MCP_TOKEN`, served instead of the remembered token and never written down
    pub env_token: Option<Token>,
    /// how often a fixed port is tried; more than once while a restarted IVE waits for the one it replaces
    pub attempts: u32,
}

/// What [`open`] did.
pub struct Opened {
    /// to show the user
    pub lines: Vec<String>,
    /// URL and token clients connect with; `None` when IVE runs without MCP
    pub served: Option<(String, Token)>,
}

/// Serves MCP through `bind` (binds a port, 0 for any free one, and answers the port bound) and remembers port and
/// token. `Err` only when the port `--mcp` asked for cannot be served.
pub fn open(request: Request, mut bind: impl FnMut(u16, &Token) -> io::Result<u16>) -> Result<Opened, String> {
    let (remembered, mut lines) = request.path.map_or((None, Vec::new()), load);
    let kept = match &remembered {
        Some(remembered) => Token::remembered(remembered.token.clone()),
        None => Token::random(),
    }
    .map_err(|error| error.to_string())?;
    let served = request.env_token.clone().unwrap_or_else(|| kept.clone());
    let port = match bind_port(&request, remembered.as_ref(), |port| bind(port, &served)) {
        Ok(port) => port,
        Err(Unserved::Given(reason)) => return Err(reason),
        Err(Unserved::Remembered(reason)) => {
            lines.push(reason);
            return Ok(Opened { lines, served: None });
        }
    };
    let now = Remembered { port, token: kept.value().to_owned() };
    if let Some(path) = request.path.filter(|_| remembered.as_ref() != Some(&now)) {
        lines.extend(save(path, &now).err());
    }
    lines.extend(connect_lines(port, &served));
    Ok(Opened { lines, served: Some((super::url(port), served)) })
}

enum Unserved {
    /// the port `--mcp` asked for: IVE stops
    Given(String),
    /// IVE runs without MCP
    Remembered(String),
}

fn bind_port(request: &Request, remembered: Option<&Remembered>, mut bind: impl FnMut(u16) -> io::Result<u16>) -> Result<u16, Unserved> {
    let interval = retry::RESTART_INTERVAL;
    if let Some(port) = request.given {
        let bound = retry::retry(request.attempts, interval, || bind(port));
        return bound.map_err(|error| Unserved::Given(format!("cannot serve MCP on port {port}: {error}")));
    }
    let Some(remembered) = remembered else {
        let bound = bind(PREFERRED_PORT).or_else(|_| bind(0));
        return bound.map_err(|error| Unserved::Remembered(format!("cannot serve MCP on any port: {error}; running without MCP")));
    };
    let port = remembered.port;
    retry::retry(request.attempts, interval, || bind(port)).map_err(|error| {
        let path = request.path.map_or_else(String::new, |path| format!(" from {}", path.display()));
        Unserved::Remembered(format!("cannot serve MCP on port {port}{path}: {error}; running without MCP (--mcp PORT moves it)"))
    })
}

fn connect_lines(port: u16, served: &Token) -> [String; 2] {
    let url = super::url(port);
    let shown_token = if served.is_from_env() { "$IVE_MCP_TOKEN" } else { served.value() };
    [
        format!("MCP server at {url}"),
        format!("connect with: claude mcp add --transport http ive {url} --header \"Authorization: Bearer {shown_token}\""),
    ]
}

/// The remembered port and token; a missing file is none. An unusable one is set aside as `mcp.toml.bad`, so IVE
/// starts afresh, and the second value says so.
fn load(path: &Path) -> (Option<Remembered>, Vec<String>) {
    match read(path) {
        Ok(remembered) => (remembered, Vec::new()),
        Err(reason) => (None, vec![state::set_aside(path, &reason)]),
    }
}

fn read(path: &Path) -> Result<Option<Remembered>, String> {
    let bytes = match fs::metadata(path) {
        Ok(metadata) => metadata.len(),
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.to_string()),
    };
    if bytes > MAX_FILE_BYTES {
        return Err(format!("{bytes} bytes, the limit is {MAX_FILE_BYTES}"));
    }
    let text = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let remembered: Remembered = toml::from_str(&text).map_err(|error| error.message().to_owned())?;
    if remembered.port < MIN_PORT {
        return Err(format!("port must be {MIN_PORT}-65535, got {}", remembered.port));
    }
    Token::remembered(remembered.token.clone()).map_err(|error| error.to_string())?;
    Ok(Some(remembered))
}

/// Writes `remembered` readable by this user only, replacing the file atomically.
fn save(path: &Path, remembered: &Remembered) -> Result<(), String> {
    let failed = |error: &dyn std::fmt::Display| format!("cannot save {}: {error}", path.display());
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|error| failed(&error))?;
    }
    let text = toml::to_string(remembered).map_err(|error| failed(&error))?;
    let staging = path.with_added_extension(format!("{}.tmp", std::process::id()));
    write_private(&staging, &text).and_then(|()| fs::rename(&staging, path)).map_err(|error| {
        // a staging file left behind would never be cleaned up
        let _ = fs::remove_file(&staging);
        failed(&error)
    })
}

fn write_private(path: &Path, text: &str) -> io::Result<()> {
    let mut options = fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    options.open(path)?.write_all(text.as_bytes())
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::path::PathBuf;

    use super::*;

    const TOKEN: &str = "0123456789abcdef0123";

    fn request(path: &Path) -> Request<'_> {
        Request { given: None, path: Some(path), env_token: None, attempts: 1 }
    }

    fn mcp_file(dir: &tempfile::TempDir) -> PathBuf {
        dir.path().join("mcp.toml")
    }

    fn remember(path: &Path, port: u16) {
        fs::write(path, format!("port = {port}\ntoken = \"{TOKEN}\"\n")).unwrap();
    }

    fn taken(_: u16) -> io::Result<u16> {
        Err(io::Error::from(io::ErrorKind::AddrInUse))
    }

    #[test]
    fn should_take_the_preferred_port_then_any_free_one_on_the_first_run() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let path = mcp_file(&dir);
        let tried = RefCell::new(Vec::new());

        // when
        let lines = open(request(&path), |port, _| {
            tried.borrow_mut().push(port);
            if port == PREFERRED_PORT { taken(port) } else { Ok(40123) }
        }).map(|opened| opened.lines);

        // then
        assert_eq!(*tried.borrow(), [PREFERRED_PORT, 0]);
        assert!(lines.as_ref().is_ok_and(|lines| lines[0] == "MCP server at http://127.0.0.1:40123/mcp"), "{lines:?}");
        assert_eq!(read(&path).unwrap().map(|remembered| remembered.port), Some(40123));
    }

    #[test]
    fn should_tell_the_url_and_served_token() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let path = mcp_file(&dir);
        remember(&path, 41000);

        // when
        let opened = open(request(&path), |port, _| Ok(port));

        // then
        let served = opened.ok().and_then(|opened| opened.served).map(|(url, token)| (url, token.value().to_owned()));
        assert_eq!(served, Some(("http://127.0.0.1:41000/mcp".to_owned(), TOKEN.to_owned())));
    }

    #[test]
    fn should_serve_the_remembered_port_and_token_without_rewriting() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let path = mcp_file(&dir);
        remember(&path, 41000);
        let before = fs::metadata(&path).unwrap().modified().unwrap();
        let served = RefCell::new(None);

        // when
        let lines = open(request(&path), |port, token| {
            *served.borrow_mut() = Some((port, token.value().to_owned()));
            Ok(port)
        }).map(|opened| opened.lines);

        // then
        assert_eq!(*served.borrow(), Some((41000, TOKEN.to_owned())));
        assert!(lines.as_ref().is_ok_and(|lines| lines[1].contains(&format!("Bearer {TOKEN}"))), "{lines:?}");
        assert_eq!(fs::metadata(&path).unwrap().modified().unwrap(), before);
    }

    #[test]
    fn should_remember_the_port_given_by_mcp_with_the_same_token() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let path = mcp_file(&dir);
        remember(&path, 41000);

        // when
        let lines = open(Request { given: Some(42000), ..request(&path) }, |port, _| Ok(port)).map(|opened| opened.lines);

        // then
        assert!(lines.is_ok(), "{lines:?}");
        assert_eq!(read(&path), Ok(Some(Remembered { port: 42000, token: TOKEN.to_owned() })));
    }

    #[test]
    fn should_run_without_mcp_when_the_remembered_port_is_taken() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let path = mcp_file(&dir);
        remember(&path, 41000);

        // when
        let lines = open(request(&path), |port, _| taken(port)).map(|opened| opened.lines);

        // then
        let lines = lines.expect("IVE keeps running");
        assert_eq!(lines.len(), 1, "{lines:?}");
        assert!(lines[0].contains("port 41000") && lines[0].contains("running without MCP"), "{}", lines[0]);
        assert_eq!(read(&path).unwrap().map(|remembered| remembered.port), Some(41000));
    }

    #[test]
    fn should_fail_when_the_port_given_by_mcp_is_taken() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let path = mcp_file(&dir);

        // when
        let lines = open(Request { given: Some(42000), ..request(&path) }, |port, _| taken(port)).map(|opened| opened.lines);

        // then
        assert!(lines.as_ref().is_err_and(|error| error.contains("port 42000")), "{lines:?}");
        assert!(!path.exists());
    }

    #[test]
    fn should_set_a_broken_file_aside_and_start_afresh() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let path = mcp_file(&dir);
        fs::write(&path, "port = 80\ntoken = \"short\"\n").unwrap();

        // when
        let lines = open(request(&path), |_, _| Ok(PREFERRED_PORT)).map(|opened| opened.lines);

        // then
        let lines = lines.expect("served");
        assert!(lines[0].contains("port must be") && lines[0].contains("mcp.toml.bad"), "{}", lines[0]);
        assert_eq!(fs::read_to_string(dir.path().join("mcp.toml.bad")).unwrap(), "port = 80\ntoken = \"short\"\n");
        assert!(read(&path).unwrap().is_some_and(|remembered| remembered.token.len() == 48));
    }

    #[test]
    fn should_serve_the_environment_token_without_writing_it() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let path = mcp_file(&dir);
        remember(&path, 41000);
        let env_token = Token::remembered("from-the-environment".to_owned()).unwrap();
        let served = RefCell::new(String::new());

        // when
        let lines = open(Request { env_token: Some(env_token), ..request(&path) }, |port, token| {
            *served.borrow_mut() = token.value().to_owned();
            Ok(port)
        }).map(|opened| opened.lines);

        // then
        assert_eq!(*served.borrow(), "from-the-environment");
        assert!(lines.as_ref().is_ok_and(|lines| lines[1].contains("Bearer from-the-environment")), "{lines:?}");
        assert_eq!(read(&path).unwrap().map(|remembered| remembered.token), Some(TOKEN.to_owned()));
    }

    #[cfg(unix)]
    #[test]
    fn should_keep_the_file_readable_by_this_user_only() {
        // given
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("profile").join("mcp.toml");

        // when
        let lines = open(request(&path), |_, _| Ok(PREFERRED_PORT)).map(|opened| opened.lines);

        // then
        assert!(lines.is_ok(), "{lines:?}");
        assert_eq!(fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o600);
    }
}
