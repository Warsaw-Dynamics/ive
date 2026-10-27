//! The host: owns one PTY and the program in it, outlives whoever started it, serves one client at a time.

use std::collections::VecDeque;
use std::ffi::OsString;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use crate::modes::ModeTracker;
use crate::protocol::Size;
use crate::registry::{DIR_ENV, Registry, validate_id, validate_label};
use crate::title::TitleTracker;

#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

#[cfg(unix)]
use unix as os;
#[cfg(windows)]
use windows as os;

pub use os::{serve, spawn_detached};
#[cfg(unix)]
pub(crate) use unix::poll;

/// Output kept for replay to the next client to attach.
const REPLAY_BYTES: usize = 1 << 20;
/// A client that stops reading for this long is dropped rather than stalling the program.
const CLIENT_WRITE_TIMEOUT: Duration = Duration::from_secs(10);
/// How long a frame may take to arrive in full once it started.
const CLIENT_READ_TIMEOUT: Duration = Duration::from_secs(1);
/// After a hang-up, how long the program gets to exit before it is killed.
const HANG_UP_GRACE: Duration = Duration::from_secs(2);
const START_TIMEOUT: Duration = Duration::from_secs(3);
/// Input waiting for the program; past this the host stops reading the client, which then waits in turn.
const MAX_PENDING_INPUT: usize = 4 << 20;
/// How often the host checks that its registry entry is still there.
const WATCHDOG_INTERVAL: Duration = Duration::from_secs(2);
/// Names the host's environment variables meant for its program (see `host_env`).
const PROGRAM_ENV_PREFIX: &str = "PTYHOST_ENV_";

#[derive(Debug, Clone)]
pub struct HostConfig {
    pub id: String,
    pub command: Vec<String>,
    pub cwd: PathBuf,
    pub size: Size,
    /// set for the program, after `TERM`
    pub env: Vec<(String, String)>,
    pub labels: Vec<(String, String)>,
}

/// How to run a host: `exe`, then `args`, then `serve …`. The `ptyhost` binary is `ptyhost serve …`; a program
/// embedding the CLI passes its own flag, such as `ive --pty-host serve …`.
#[derive(Debug, Clone)]
pub struct Launcher {
    pub exe: PathBuf,
    pub args: Vec<OsString>,
}

/// Starts a detached host and returns its pid once the session accepts attaches.
///
/// The host runs apart from its starter (its own session on Unix; no console and out of the starter's job on
/// Windows), so closing the starter's terminal or ending its process group misses it.
pub fn start(launcher: &Launcher, registry: &Registry, config: &HostConfig) -> io::Result<u32> {
    validate_id(&config.id)?;
    registry.check_socket_path(&config.id)?;
    for (key, value) in &config.labels {
        validate_label(key, value)?;
    }
    if registry.session(&config.id)?.is_some() {
        return Err(io::Error::new(io::ErrorKind::AlreadyExists, format!("session {} is already running", config.id)));
    }
    registry.create_dir().map_err(|error| about(error, "cannot create the terminals folder", registry.dir()))?;
    let log = registry.log(&config.id);
    let log_file = File::create(&log).map_err(|error| about(error, "cannot create the terminal host's log", &log))?;
    let mut command = Command::new(&launcher.exe);
    command
        .args(&launcher.args)
        .args(serve_args(config))
        .envs(host_env(config))
        .env(DIR_ENV, registry.dir())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(log_file);
    let mut host = os::spawn_detached(&mut command).map_err(|error| about(error, "cannot start the terminal host", &launcher.exe))?;
    let pid = host.id();
    let started = wait_until_live(registry, &config.id, &mut host);
    // reaps the host if it ends while the starter still runs; after that, init does
    std::thread::spawn(move || host.wait());
    started.map(|()| pid)
}

/// `error` with what failed and on which path, keeping its kind: `cannot start the terminal host /usr/bin/ive: …`.
fn about(error: io::Error, what: &str, path: &std::path::Path) -> io::Error {
    io::Error::new(error.kind(), format!("{what} {}: {error}", path.display()))
}

fn serve_args(config: &HostConfig) -> Vec<OsString> {
    let mut args: Vec<OsString> = ["serve", "--id", &config.id, "--size", &format!("{}x{}", config.size.columns, config.size.lines), "--cwd"]
        .map(OsString::from)
        .to_vec();
    args.push(config.cwd.clone().into());
    for (key, value) in &config.labels {
        args.push("--label".into());
        args.push(format!("{key}={value}").into());
    }
    args.push("--".into());
    args.extend(config.command.iter().map(OsString::from));
    args
}

/// The program's environment as the host's own, prefixed: a command line is visible to every user (`ps`), and
/// `env` may hold secrets.
fn host_env(config: &HostConfig) -> Vec<(String, String)> {
    config.env.iter().map(|(key, value)| (format!("{PROGRAM_ENV_PREFIX}{key}"), value.clone())).collect()
}

/// The variables `host_env` passed, unprefixed, from the host's environment `vars`; values in any bytes, since an
/// environment need not be UTF-8.
fn program_env(vars: impl Iterator<Item = (OsString, OsString)>) -> Vec<(OsString, OsString)> {
    vars.filter_map(|(key, value)| {
        let name = key.to_str()?.strip_prefix(PROGRAM_ENV_PREFIX).filter(|name| !name.is_empty())?;
        Some((OsString::from(name), value))
    })
    .collect()
}

/// The host's own variables its program must not see: the registry folder (an IVE started from the program's shell
/// must pick its registry from its profile) and the prefixed copies `program_env` unpacks.
fn hidden_env(vars: impl Iterator<Item = (OsString, OsString)>) -> Vec<OsString> {
    vars.map(|(key, _)| key)
        .filter(|key| key == DIR_ENV || key.as_encoded_bytes().starts_with(PROGRAM_ENV_PREFIX.as_bytes()))
        .collect()
}

/// Waits until the session is live, or the host ended first; on failure the host's log becomes the error.
fn wait_until_live(registry: &Registry, id: &str, host: &mut Child) -> io::Result<()> {
    let deadline = Instant::now() + START_TIMEOUT;
    let mut ended = None;
    while ended.is_none() && Instant::now() < deadline {
        if registry.session(id)?.is_some() {
            return Ok(());
        }
        ended = host.try_wait()?;
        std::thread::sleep(Duration::from_millis(10));
    }
    let log = std::fs::read_to_string(registry.log(id)).unwrap_or_default();
    let _ = std::fs::remove_file(registry.log(id));
    let reason = match ended {
        Some(status) => format!("ended ({status})"),
        None => "did not start within 3 s".to_owned(),
    };
    Err(io::Error::other(format!("session {id} {reason}: {}", log.trim())))
}

/// A host whose socket or meta file is gone can never be reached again (its registry was deleted, as at logout
/// without logind killing user processes); it hangs up rather than run on unseen.
struct Watchdog {
    registry: Registry,
    id: String,
    socket: PathBuf,
    identity: FileIdentity,
    next_check: Instant,
}

impl Watchdog {
    fn new(registry: &Registry, id: &str, socket: &Path) -> io::Result<Watchdog> {
        Ok(Watchdog {
            registry: registry.clone(),
            id: id.to_owned(),
            socket: socket.to_path_buf(),
            identity: identity_of(socket)?,
            next_check: Instant::now(),
        })
    }

    /// True when a check was due and found the entry gone or taken over.
    fn entry_lost(&mut self) -> bool {
        if Instant::now() < self.next_check {
            return false;
        }
        self.next_check = Instant::now() + WATCHDOG_INTERVAL;
        let own_socket = identity_of(&self.socket).is_ok_and(|identity| identity == self.identity);
        let listed = self.registry.session(&self.id).is_ok_and(|session| session.is_some());
        !(own_socket && listed)
    }
}

/// What tells this file from another created at the same path later.
type FileIdentity = (u64, u64);

#[cfg(unix)]
fn identity_of(path: &Path) -> io::Result<FileIdentity> {
    use std::os::unix::fs::MetadataExt;
    let metadata = std::fs::metadata(path)?;
    Ok((metadata.dev(), metadata.ino()))
}

/// Std's volume serial and file index are unstable; a socket bound again at the path has a new creation time.
#[cfg(windows)]
fn identity_of(path: &Path) -> io::Result<FileIdentity> {
    use std::os::windows::fs::MetadataExt;
    let metadata = std::fs::symlink_metadata(path)?;
    Ok((metadata.creation_time(), 0))
}

/// The tail of the output. Once trimmed it starts after a newline, so replay does not begin inside an escape
/// sequence (one spanning a newline is rare), with the last title, which programs set only when it changes, and
/// the modes set before the tail, which full-screen programs set only at their start.
#[derive(Default)]
struct Replay {
    bytes: VecDeque<u8>,
    trimmed: bool,
    title: TitleTracker,
    /// the modes as of the first kept byte: it reads the output as it is trimmed
    modes: ModeTracker,
}

impl Replay {
    fn push(&mut self, data: &[u8]) {
        self.title.scan(data);
        self.bytes.extend(data);
        let excess = self.bytes.len().saturating_sub(REPLAY_BYTES);
        if excess > 0 {
            let (front, back) = self.bytes.as_slices();
            let front_part = excess.min(front.len());
            self.modes.scan(&front[..front_part]);
            self.modes.scan(&back[..excess - front_part]);
            self.bytes.drain(..excess);
            self.trimmed = true;
        }
    }

    fn snapshot(&self) -> Vec<u8> {
        let bytes: Vec<u8> = self.bytes.iter().copied().collect();
        if self.trimmed {
            let start = bytes.iter().position(|&b| b == b'\n').map_or(bytes.len(), |newline| newline + 1);
            let mut modes = self.modes.clone();
            modes.scan(&bytes[..start]);
            return [self.title.sequence().unwrap_or_default().as_slice(), &modes.sequence(), &bytes[start..]].concat();
        }
        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_name_host_program_that_cannot_start() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let registry = Registry::at(dir.path().join("r"));
        let launcher = Launcher { exe: PathBuf::from("/no/such/ive"), args: Vec::new() };
        let config = HostConfig { id: "t1".to_owned(), command: vec!["sh".to_owned()], cwd: PathBuf::from("/"), size: Size::DEFAULT, env: Vec::new(), labels: Vec::new() };

        // when
        let error = start(&launcher, &registry, &config).unwrap_err();

        // then
        assert!(error.to_string().starts_with("cannot start the terminal host /no/such/ive: "), "{error}");
    }

    #[test]
    fn should_pass_env_to_the_host_outside_its_command_line() {
        // given
        let config = HostConfig {
            id: "t1".to_owned(),
            command: vec!["sh".to_owned()],
            cwd: PathBuf::from("/"),
            size: Size::DEFAULT,
            env: vec![("SECRET".to_owned(), "s3cret".to_owned())],
            labels: Vec::new(),
        };

        // when
        let args = serve_args(&config);

        // then
        assert!(!args.iter().any(|arg| arg.to_string_lossy().contains("s3cret")), "{args:?}");
        assert_eq!(host_env(&config), [("PTYHOST_ENV_SECRET".to_owned(), "s3cret".to_owned())]);
    }

    #[test]
    fn should_take_prefixed_env_for_the_program() {
        // given
        let vars = [("PTYHOST_ENV_SECRET", "s3cret"), ("PATH", "/bin"), ("PTYHOST_ENV_", "empty name")]
            .map(|(key, value)| (OsString::from(key), OsString::from(value)))
            .into_iter()
            .chain([(OsString::from("PTYHOST_ENV_RAW"), not_utf8())]);

        // when
        let env = program_env(vars);

        // then
        assert_eq!(env, [(OsString::from("SECRET"), OsString::from("s3cret")), (OsString::from("RAW"), not_utf8())]);
    }

    #[test]
    fn should_hide_registry_dir_and_prefixed_env_from_the_program() {
        // given
        let vars = [(DIR_ENV, "/run/ive-pty"), ("PTYHOST_ENV_SECRET", "s3cret"), ("PATH", "/bin")]
            .map(|(key, value)| (OsString::from(key), OsString::from(value)))
            .into_iter();

        // when
        let hidden = hidden_env(vars);

        // then
        assert_eq!(hidden, [OsString::from(DIR_ENV), OsString::from("PTYHOST_ENV_SECRET")]);
    }

    #[cfg(unix)]
    fn not_utf8() -> OsString {
        use std::os::unix::ffi::OsStringExt;
        OsString::from_vec(b"\xff".to_vec())
    }

    #[cfg(windows)]
    fn not_utf8() -> OsString {
        use std::os::windows::ffi::OsStringExt;
        OsString::from_wide(&[0xD800])
    }

    #[test]
    fn should_replay_from_after_a_newline_once_trimmed() {
        // given
        let mut replay = Replay::default();
        replay.push(&vec![b'a'; REPLAY_BYTES]);

        // when
        replay.push(b"cut\x1b[1mline\nwhole line\n");

        // then
        assert_eq!(replay.snapshot(), b"whole line\n");
    }

    #[test]
    fn should_replay_the_last_title_once_trimmed_past_it() {
        // given
        let mut replay = Replay::default();
        replay.push(b"\x1b]0;\xe2\x9c\xb3 Summary\x07");

        // when
        replay.push(&vec![b'a'; REPLAY_BYTES]);
        replay.push(b"\nprompt");

        // then
        assert_eq!(replay.snapshot(), b"\x1b]2;\xe2\x9c\xb3 Summary\x07prompt");
    }

    #[test]
    fn should_replay_the_modes_set_before_the_kept_output_once_trimmed_past_them() {
        // given
        let mut replay = Replay::default();
        replay.push(b"\x1b]0;codex\x07\x1b[?1049h\x1b[?1000h\x1b[?1006h");

        // when
        replay.push(&vec![b'a'; REPLAY_BYTES]);
        replay.push(b"\x1b[?2004h\nscreen\x1b[?25l");

        // then
        assert_eq!(replay.snapshot(), b"\x1b]2;codex\x07\x1b[?1049h\x1b[?1000h\x1b[?1006h\x1b[?2004hscreen\x1b[?25l");
    }

    #[test]
    fn should_replay_the_screen_left_before_the_kept_output() {
        // given
        let mut replay = Replay::default();
        replay.push(b"\x1b[?1049h\x1b[?1000hfull screen\x1b[?1000l\x1b[?1049l");

        // when
        replay.push(&vec![b'a'; REPLAY_BYTES]);
        replay.push(b"\nshell\x1b[?1049h");

        // then
        assert_eq!(replay.snapshot(), b"\x1b[?1049l\x1b[?1000lshell\x1b[?1049h");
    }
}
