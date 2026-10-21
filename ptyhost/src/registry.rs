//! Which sessions are running: one directory, per session `<id>.sock` to attach to, `<id>.meta` describing it and
//! `<id>.log` with the host's errors.
//!
//! A host holds an exclusive lock on its meta file while alive, so a session whose host died, even by SIGKILL,
//! shows as stale and is cleaned up by the next listing. No PID files, so no PID reuse.

use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::{Path, PathBuf};

/// Overrides where sessions are registered; tests use it to stay apart.
pub const DIR_ENV: &str = "IVE_PTY_DIR";
/// The profile IVE runs as without `--profile`.
pub const DEFAULT_PROFILE: &str = "default";

const MAX_ID_LENGTH: usize = 64;
const MAX_LABEL_KEY_LENGTH: usize = 64;
/// `sun_path` holds 108 bytes on Linux, its terminating NUL included.
const MAX_SOCKET_PATH: usize = 108;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionInfo {
    pub id: String,
    pub host_pid: u32,
    pub program_pid: u32,
    pub cwd: PathBuf,
    pub command: Vec<String>,
    /// what the starter wants to know about the session later, such as where it belongs
    pub labels: Vec<(String, String)>,
    /// when the host started, in milliseconds since the Unix epoch
    pub started_ms: u64,
}

#[derive(Debug, Clone)]
pub struct Registry {
    dir: PathBuf,
}

/// Marks a session live until dropped.
pub struct Lease {
    _locked: File,
}

/// The one process that adopts this registry's sessions, until dropped.
pub struct Owner {
    _locked: File,
}

impl Registry {
    pub fn at(dir: impl Into<PathBuf>) -> Registry {
        Registry { dir: dir.into() }
    }

    /// The registry of the default profile, as the `--pty-host` commands use it.
    pub fn from_env() -> Registry {
        Registry::for_profile(DEFAULT_PROFILE)
    }

    /// `$IVE_PTY_DIR`, else `$XDG_RUNTIME_DIR/ive-pty/<profile>`, which the system empties at logout, as it ends
    /// the sessions.
    pub fn for_profile(profile: &str) -> Registry {
        Registry::for_profile_in(profile, std::env::var_os(DIR_ENV), std::env::var_os("XDG_RUNTIME_DIR"))
    }

    fn for_profile_in(profile: &str, env_dir: Option<OsString>, runtime: Option<OsString>) -> Registry {
        if let Some(dir) = env_dir {
            return Registry::at(dir);
        }
        if let Some(runtime) = runtime {
            return Registry::at(Path::new(&runtime).join("ive-pty").join(profile));
        }
        // SAFETY: getuid cannot fail
        let uid = unsafe { libc::getuid() };
        Registry::at(std::env::temp_dir().join(format!("ive-pty-{uid}")).join(profile))
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn socket(&self, id: &str) -> PathBuf {
        self.dir.join(format!("{id}.sock"))
    }

    pub fn log(&self, id: &str) -> PathBuf {
        self.dir.join(format!("{id}.log"))
    }

    fn meta(&self, id: &str) -> PathBuf {
        self.dir.join(format!("{id}.meta"))
    }

    pub fn create_dir(&self) -> io::Result<()> {
        fs::DirBuilder::new().recursive(true).mode(0o700).create(&self.dir)
    }

    /// Becomes the registry's owner unless another process is; `None` then.
    pub fn try_own(&self) -> io::Result<Option<Owner>> {
        self.create_dir()?;
        let file = OpenOptions::new().write(true).create(true).truncate(false).mode(0o600).open(self.dir.join("owner.lock"))?;
        match file.try_lock() {
            Ok(()) => Ok(Some(Owner { _locked: file })),
            Err(fs::TryLockError::WouldBlock) => Ok(None),
            Err(fs::TryLockError::Error(error)) => Err(error),
        }
    }

    /// Fails when the session's socket path would not fit a Unix socket address.
    pub fn check_socket_path(&self, id: &str) -> io::Result<()> {
        let socket = self.socket(id);
        let length = socket.as_os_str().len();
        if length < MAX_SOCKET_PATH {
            return Ok(());
        }
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("socket path {} is {length} bytes; Unix sockets need under {MAX_SOCKET_PATH}", socket.display()),
        ))
    }

    /// Publishes the session, live until the lease drops.
    pub fn record(&self, session: &SessionInfo) -> io::Result<Lease> {
        for (key, value) in &session.labels {
            validate_label(key, value)?;
        }
        self.create_dir()?;
        // locked before it appears under its final name, so a listing never sees it unlocked
        let draft = self.dir.join(format!("{}.meta.draft", session.id));
        let mut file = OpenOptions::new().write(true).create(true).truncate(true).mode(0o600).open(&draft)?;
        file.try_lock().map_err(|error| io::Error::other(format!("cannot lock {}: {error}", draft.display())))?;
        file.write_all(encode(session).as_bytes())?;
        fs::rename(&draft, self.meta(&session.id))?;
        Ok(Lease { _locked: file })
    }

    pub fn forget(&self, id: &str) {
        for path in [self.socket(id), self.meta(id), self.log(id)] {
            // already gone is fine: a listing may have cleaned up first
            let _ = fs::remove_file(path);
        }
    }

    /// Live sessions in start order; removes what stale ones left behind.
    pub fn sessions(&self) -> io::Result<Vec<SessionInfo>> {
        let entries = match fs::read_dir(&self.dir) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            entries => entries?,
        };
        let mut live = Vec::new();
        for entry in entries {
            let path = entry?.path();
            let id = path.file_stem().and_then(|stem| stem.to_str()).filter(|_| path.extension().is_some_and(|ext| ext == "meta"));
            if let Some(id) = id {
                match self.probe(id) {
                    Ok(Some(session)) => live.push(session),
                    Ok(None) => self.forget(id),
                    // its host lives on; one unreadable entry must not hide the others
                    Err(error) => eprintln!("ptyhost: skipping session {id}: {error}"),
                }
            }
        }
        live.sort_by(|a, b| (a.started_ms, &a.id).cmp(&(b.started_ms, &b.id)));
        Ok(live)
    }

    /// The live session with this id, if any.
    pub fn session(&self, id: &str) -> io::Result<Option<SessionInfo>> {
        validate_id(id)?;
        self.probe(id)
    }

    fn probe(&self, id: &str) -> io::Result<Option<SessionInfo>> {
        let file = match File::open(self.meta(id)) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            file => file?,
        };
        // getting the lock means no host holds it; it is released again when `file` drops
        if file.try_lock().is_ok() {
            return Ok(None);
        }
        decode(id, &fs::read_to_string(self.meta(id))?).map(Some)
    }
}

/// Ids name files, so they are kept to `[A-Za-z0-9_-]{1,64}`.
pub fn validate_id(id: &str) -> io::Result<()> {
    let valid = (1..=MAX_ID_LENGTH).contains(&id.len()) && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_');
    if valid {
        return Ok(());
    }
    Err(io::Error::new(io::ErrorKind::InvalidInput, format!("session id must be 1-{MAX_ID_LENGTH} of [A-Za-z0-9_-], got {id:?}")))
}

/// Label keys are `[a-z0-9._-]{1,64}`; values are free text without line breaks or NULs, as each is one meta line.
pub fn validate_label(key: &str, value: &str) -> io::Result<()> {
    let key_valid = (1..=MAX_LABEL_KEY_LENGTH).contains(&key.len())
        && key.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"._-".contains(&b));
    if !key_valid {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("label key must be 1-{MAX_LABEL_KEY_LENGTH} of [a-z0-9._-], got {key:?}")));
    }
    if value.contains(['\n', '\0']) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("label {key} must not contain a line break or NUL, got {value:?}")));
    }
    Ok(())
}

fn encode(session: &SessionInfo) -> String {
    let mut text = format!(
        "host_pid={}\nprogram_pid={}\nstarted_ms={}\ncwd={}\n",
        session.host_pid,
        session.program_pid,
        session.started_ms,
        session.cwd.display()
    );
    for arg in &session.command {
        text.push_str(&format!("arg={arg}\n"));
    }
    for (key, value) in &session.labels {
        text.push_str(&format!("label={key}={value}\n"));
    }
    text
}

fn decode(id: &str, text: &str) -> io::Result<SessionInfo> {
    let mut session = SessionInfo {
        id: id.to_string(),
        host_pid: 0,
        program_pid: 0,
        cwd: PathBuf::new(),
        command: Vec::new(),
        labels: Vec::new(),
        started_ms: 0,
    };
    for line in text.lines() {
        match line.split_once('=') {
            Some(("host_pid", pid)) => session.host_pid = parse_number(id, pid)?,
            Some(("program_pid", pid)) => session.program_pid = parse_number(id, pid)?,
            Some(("cwd", cwd)) => session.cwd = PathBuf::from(cwd),
            Some(("arg", arg)) => session.command.push(arg.to_string()),
            Some(("started_ms", millis)) => session.started_ms = parse_number(id, millis)?,
            Some(("label", label)) => {
                if let Some((key, value)) = label.split_once('=') {
                    session.labels.push((key.to_string(), value.to_string()));
                }
            }
            // hosts outlive the build that started them: lines a newer build added are skipped, not fatal
            _ => {}
        }
    }
    Ok(session)
}

fn parse_number<T: std::str::FromStr>(id: &str, text: &str) -> io::Result<T> {
    text.parse().map_err(|_| io::Error::new(io::ErrorKind::InvalidData, format!("session {id}: {text:?} is not a number")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session(id: &str) -> SessionInfo {
        SessionInfo {
            id: id.into(),
            host_pid: 10,
            program_pid: 11,
            cwd: "/work".into(),
            command: vec!["sh".into(), "-l".into()],
            labels: Vec::new(),
            started_ms: 1,
        }
    }

    #[test]
    fn should_round_trip_labels_and_start_time() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let registry = Registry::at(dir.path());
        let labelled = SessionInfo {
            labels: vec![("ive.project".into(), "p1".into()), ("ive.checkout".into(), "/w/a b=c".into())],
            started_ms: 1_700_000_000_000,
            ..session("a")
        };

        // when
        let _lease = registry.record(&labelled).unwrap();

        // then
        assert_eq!(registry.sessions().unwrap(), vec![labelled]);
    }

    #[test]
    fn should_reject_label_with_newline_or_bad_key() {
        // given
        let labels = [("ive.checkout", "a\nb"), ("Bad Key", "x"), ("ive.x", "a\0b")];

        // when
        let errors: Vec<_> = labels.iter().map(|(key, value)| validate_label(key, value).unwrap_err().kind()).collect();

        // then
        assert_eq!(errors, vec![io::ErrorKind::InvalidInput; 3]);
    }

    #[test]
    fn should_let_only_one_owner_hold_the_registry() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let registry = Registry::at(dir.path());
        let owner = registry.try_own().unwrap();

        // when
        let second = registry.try_own().unwrap();
        drop(owner);
        let after_release = registry.try_own().unwrap();

        // then
        assert!(second.is_none());
        assert!(after_release.is_some());
    }

    #[test]
    fn should_reject_socket_path_too_long() {
        // given
        let registry = Registry::at(format!("/tmp/{}", "d".repeat(100)));

        // when
        let error = registry.check_socket_path("abc").unwrap_err();

        // then
        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
        assert!(error.to_string().contains("108"), "{error}");
    }

    #[test]
    fn should_ignore_unknown_meta_lines() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let registry = Registry::at(dir.path());
        let _lease = registry.record(&session("a")).unwrap();
        let meta = dir.path().join("a.meta");
        let text = fs::read_to_string(&meta).unwrap();
        fs::write(&meta, format!("format=2\nnew_key=value\n{text}")).unwrap();

        // when
        let sessions = registry.sessions().unwrap();

        // then
        assert_eq!(sessions, vec![session("a")]);
    }

    #[test]
    fn should_list_other_sessions_when_one_meta_is_unreadable() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let registry = Registry::at(dir.path());
        let _good = registry.record(&session("good")).unwrap();
        let _bad = registry.record(&session("bad")).unwrap();
        fs::write(dir.path().join("bad.meta"), "host_pid=not-a-number\n").unwrap();

        // when
        let sessions = registry.sessions().unwrap();

        // then
        assert_eq!(sessions, vec![session("good")]);
    }

    #[test]
    fn should_list_sessions_in_start_order() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let registry = Registry::at(dir.path());
        let _b = registry.record(&SessionInfo { started_ms: 1, ..session("b") }).unwrap();
        let _a = registry.record(&SessionInfo { started_ms: 2, ..session("a") }).unwrap();

        // when
        let ids: Vec<String> = registry.sessions().unwrap().into_iter().map(|session| session.id).collect();

        // then
        assert_eq!(ids, vec!["b", "a"]);
    }

    #[test]
    fn should_list_session_while_its_lease_is_held() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let registry = Registry::at(dir.path());

        // when
        let _lease = registry.record(&session("a")).unwrap();

        // then
        assert_eq!(registry.sessions().unwrap(), vec![session("a")]);
    }

    #[test]
    fn should_clean_up_session_once_its_lease_drops() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let registry = Registry::at(dir.path());
        drop(registry.record(&session("a")).unwrap());

        // when
        let sessions = registry.sessions().unwrap();

        // then
        assert!(sessions.is_empty());
        assert!(!dir.path().join("a.meta").exists());
    }

    #[test]
    fn should_reject_id_that_would_escape_the_directory() {
        // given
        let registry = Registry::at("/nonexistent");

        // when
        let error = registry.session("../x").unwrap_err();

        // then
        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
    }

    #[test]
    fn should_put_each_profile_in_its_own_folder_under_the_runtime_dir() {
        // given
        let runtime = Some(std::ffi::OsString::from("/run/user/1000"));

        // when
        let registry = Registry::for_profile_in("work", None, runtime);

        // then
        assert_eq!(registry.dir(), Path::new("/run/user/1000/ive-pty/work"));
    }

    #[test]
    fn should_let_the_env_dir_override_the_profile_folder() {
        // given
        let env_dir = Some(std::ffi::OsString::from("/tmp/mine"));

        // when
        let registry = Registry::for_profile_in("work", env_dir, Some("/run/user/1000".into()));

        // then
        assert_eq!(registry.dir(), Path::new("/tmp/mine"));
    }
}
