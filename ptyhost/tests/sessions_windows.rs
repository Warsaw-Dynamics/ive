//! Real hosts started through the `ptyhost` binary on Windows, each test in its own registry directory. ConPTY
//! echoes what is typed, so answers are computed (`set /a`) to tell them from the command line.
#![cfg(windows)]

use std::path::Path;
use std::time::{Duration, Instant};

use ptyhost::client::Client;
use ptyhost::host::{self, HostConfig, Launcher};
use ptyhost::job::{Job, TERMINATED};
use ptyhost::protocol::{Frame, Size};
use ptyhost::registry::Registry;

const TIMEOUT: Duration = Duration::from_secs(10);

fn launcher() -> Launcher {
    Launcher { exe: Path::new(env!("CARGO_BIN_EXE_ptyhost")).to_path_buf(), args: Vec::new() }
}

fn config(id: &str, command: &[&str]) -> HostConfig {
    HostConfig {
        id: id.into(),
        command: command.iter().map(|arg| arg.to_string()).collect(),
        cwd: std::env::temp_dir(),
        size: Size::DEFAULT,
        env: Vec::new(),
        labels: Vec::new(),
    }
}

fn start(registry: &Registry, id: &str, command: &[&str]) -> u32 {
    host::start(&launcher(), registry, &config(id, command)).unwrap()
}

/// An interactive cmd that announces itself.
fn start_shell(registry: &Registry, id: &str) {
    start(registry, id, &["cmd", "/D", "/Q", "/K", "set /a 40+2"]);
}

/// Attaches and sends the first resize, which the host waits for before replaying.
fn attach(registry: &Registry, id: &str) -> Client {
    let mut client = Client::connect(registry, id).unwrap();
    client.set_read_timeout(Some(TIMEOUT)).unwrap();
    client.send(&Frame::Resize(Size::DEFAULT)).unwrap();
    client
}

/// Output received, replay included, until it contains `expected`; panics after the timeout.
fn read_until(client: &mut Client, expected: &str) -> String {
    let mut seen: Vec<u8> = Vec::new();
    while !String::from_utf8_lossy(&seen).contains(expected) {
        match client.receive().unwrap() {
            Frame::Output(bytes) | Frame::Replay(bytes) => seen.extend(bytes),
            other => panic!("expected {expected:?}, got {other:?} after {:?}", String::from_utf8_lossy(&seen)),
        }
    }
    String::from_utf8_lossy(&seen).into_owned()
}

fn wait_for(condition: impl Fn() -> bool) {
    let deadline = Instant::now() + TIMEOUT;
    while !condition() {
        assert!(Instant::now() < deadline, "condition not met within {TIMEOUT:?}");
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn exit_code(client: &mut Client) -> Option<i32> {
    std::iter::from_fn(|| client.receive().ok()).find_map(|frame| match frame {
        Frame::Exited(code) => Some(code),
        _ => None,
    })
}

fn is_running(pid: u32) -> bool {
    let output = std::process::Command::new("tasklist").args(["/FI", &format!("PID eq {pid}"), "/NH"]).output().unwrap();
    String::from_utf8_lossy(&output.stdout).contains(&pid.to_string())
}

#[test]
fn should_keep_program_running_after_detach_and_replay_its_output() {
    // given
    let dir = tempfile::tempdir().unwrap();
    let registry = Registry::at(dir.path());
    start_shell(&registry, "shell");
    let mut first = attach(&registry, "shell");
    read_until(&mut first, "42");
    first.send(&Frame::Input(b"set /a 6*111\r".to_vec())).unwrap();
    read_until(&mut first, "666");
    drop(first);

    // when
    let mut second = attach(&registry, "shell");
    second.send(&Frame::Input(b"set /a 7*111\r".to_vec())).unwrap();
    let output = read_until(&mut second, "777");

    // then
    assert!(output.contains("42") && output.contains("666"), "replay missing from {output:?}");
}

#[test]
fn should_send_replay_only_after_first_resize() {
    // given
    let dir = tempfile::tempdir().unwrap();
    let registry = Registry::at(dir.path());
    start_shell(&registry, "waits");
    std::thread::sleep(Duration::from_millis(500));
    let mut client = Client::connect(&registry, "waits").unwrap();
    client.set_read_timeout(Some(Duration::from_millis(300))).unwrap();
    let before_resize = client.receive();

    // when
    client.send(&Frame::Resize(Size { columns: 100, lines: 30 })).unwrap();
    client.set_read_timeout(Some(TIMEOUT)).unwrap();
    let after_resize = client.receive().unwrap();

    // then
    let error = before_resize.unwrap_err();
    assert!(matches!(error.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut), "got {error}");
    match after_resize {
        Frame::Replay(bytes) => assert!(String::from_utf8_lossy(&bytes).contains("42")),
        other => panic!("expected the replay, got {other:?}"),
    }
}

#[test]
fn should_report_program_pid_in_handshake() {
    // given
    let dir = tempfile::tempdir().unwrap();
    let registry = Registry::at(dir.path());
    start_shell(&registry, "pid");

    // when
    let client = attach(&registry, "pid");

    // then
    let session = registry.session("pid").unwrap().unwrap();
    assert_eq!(client.program_pid(), session.program_pid);
    assert!(is_running(session.program_pid));
}

#[test]
fn should_forget_session_when_its_program_exits() {
    // given
    let dir = tempfile::tempdir().unwrap();
    let registry = Registry::at(dir.path());
    start(&registry, "exits", &["cmd", "/D", "/C", "ping -n 2 127.0.0.1 >NUL & exit 3"]);
    let mut client = attach(&registry, "exits");

    // when
    let code = exit_code(&mut client);

    // then
    assert_eq!(code, Some(3));
    assert!(registry.sessions().unwrap().is_empty());
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0, "files left behind");
}

#[test]
fn should_end_program_and_what_it_started_on_hang_up() {
    // given
    let dir = tempfile::tempdir().unwrap();
    let registry = Registry::at(dir.path());
    start(&registry, "hangs", &["cmd", "/D", "/C", "ping -n 60 127.0.0.1"]);
    let mut client = attach(&registry, "hangs");
    let program = client.program_pid();

    // when
    client.hang_up().unwrap();

    // then
    assert!(exit_code(&mut client).is_some());
    wait_for(|| !is_running(program));
    assert!(registry.sessions().unwrap().is_empty());
}

#[test]
fn should_drop_session_and_end_program_whose_host_was_killed() {
    // given
    let dir = tempfile::tempdir().unwrap();
    let registry = Registry::at(dir.path());
    let host_pid = start(&registry, "orphan", &["cmd", "/D", "/C", "ping -n 60 127.0.0.1"]);
    let program = registry.session("orphan").unwrap().unwrap().program_pid;

    // when
    std::process::Command::new("taskkill").args(["/F", "/PID", &host_pid.to_string()]).output().unwrap();

    // then
    wait_for(|| registry.sessions().unwrap().is_empty());
    wait_for(|| !is_running(program));
}

#[test]
fn should_hand_session_to_the_latest_client() {
    // given
    let dir = tempfile::tempdir().unwrap();
    let registry = Registry::at(dir.path());
    start_shell(&registry, "twice");
    let mut first = attach(&registry, "twice");
    read_until(&mut first, "42");

    // when
    let mut second = attach(&registry, "twice");
    second.send(&Frame::Input(b"set /a 8*111\r".to_vec())).unwrap();
    read_until(&mut second, "888");

    // then
    let first_frames: Vec<Frame> = std::iter::from_fn(|| first.receive().ok()).collect();
    assert!(!first_frames.iter().any(|frame| matches!(frame, Frame::Output(bytes) if String::from_utf8_lossy(bytes).contains("888"))));
}

#[test]
fn should_pass_env_and_labels_to_session() {
    // given
    let dir = tempfile::tempdir().unwrap();
    let registry = Registry::at(dir.path());
    let mut config = config("env", &["cmd", "/D", "/Q", "/K", "echo [%IVE_TEST_VALUE%] [%IVE_PTY_DIR%]"]);
    config.env = vec![("IVE_TEST_VALUE".into(), "from-ive".into())];
    config.labels = vec![("ive.project".into(), "p1".into())];

    // when
    host::start(&launcher(), &registry, &config).unwrap();
    let mut client = attach(&registry, "env");

    // then
    let output = read_until(&mut client, "[from-ive]");
    assert!(output.contains("[%IVE_PTY_DIR%]"), "the registry dir reached the program: {output:?}");
    assert_eq!(registry.session("env").unwrap().unwrap().labels, config.labels);
}

#[test]
fn should_report_failed_start_and_clean_up_its_log() {
    // given
    let dir = tempfile::tempdir().unwrap();
    let registry = Registry::at(dir.path());

    // when
    let error = host::start(&launcher(), &registry, &config("broken", &["no-such-program-ive"])).unwrap_err();

    // then
    assert!(error.to_string().contains("broken"), "{error}");
    assert!(!registry.log("broken").exists());
}

#[test]
fn should_hang_up_when_its_registry_entry_disappears() {
    // given
    let dir = tempfile::tempdir().unwrap();
    let registry = Registry::at(dir.path());
    start(&registry, "lost", &["cmd", "/D", "/C", "ping -n 60 127.0.0.1"]);
    let mut client = attach(&registry, "lost");
    let program = client.program_pid();

    // when
    std::fs::remove_file(dir.path().join("lost.meta")).unwrap();

    // then
    assert!(exit_code(&mut client).is_some());
    wait_for(|| !is_running(program));
}

#[test]
fn should_let_the_starter_end_the_program_through_the_host_job() {
    // given
    let dir = tempfile::tempdir().unwrap();
    let registry = Registry::at(dir.path());
    start(&registry, "job", &["cmd", "/D", "/C", "ping -n 60 127.0.0.1"]);
    let mut client = attach(&registry, "job");
    let host_pid = registry.session("job").unwrap().unwrap().host_pid;

    // when
    Job::open(&Job::host_job_name(host_pid)).unwrap().terminate();

    // then
    assert_eq!(exit_code(&mut client), Some(TERMINATED as i32));
}
