//! Real hosts started through the `ptyhost` binary, each test in its own registry directory.

use std::path::Path;
use std::time::{Duration, Instant};

use ptyhost::client::Client;
use ptyhost::host::{self, HostConfig, Launcher};
use ptyhost::protocol::{Frame, Size};
use ptyhost::registry::Registry;

const TIMEOUT: Duration = Duration::from_secs(5);

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

fn start(registry: &Registry, id: &str, script: &str) -> u32 {
    host::start(&launcher(), registry, &config(id, &["sh", "-c", script])).unwrap()
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
    // only the tail can complete the match, so large outputs are not searched again and again
    let found = |seen: &[u8], from: usize| seen[from..].windows(expected.len()).any(|window| window == expected.as_bytes());
    let mut searched = 0;
    while !found(&seen, searched) {
        searched = seen.len().saturating_sub(expected.len());
        match client.receive().unwrap() {
            Frame::Output(bytes) | Frame::Replay(bytes) => seen.extend(bytes),
            other => panic!("expected {expected:?}, got {other:?} after {} bytes", seen.len()),
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

#[test]
fn should_keep_program_running_after_detach_and_replay_its_output() {
    // given
    let dir = tempfile::tempdir().unwrap();
    let registry = Registry::at(dir.path());
    start(&registry, "cat", "echo ready; cat");
    let mut first = attach(&registry, "cat");
    first.send(&Frame::Input(b"before detach\n".to_vec())).unwrap();
    read_until(&mut first, "before detach\r\nbefore detach");
    drop(first);

    // when
    let mut second = attach(&registry, "cat");
    second.send(&Frame::Input(b"after attach\n".to_vec())).unwrap();
    let output = read_until(&mut second, "after attach\r\nafter attach");

    // then
    assert!(output.starts_with("ready\r\nbefore detach"), "replay missing from {output:?}");
}

#[test]
fn should_send_replay_only_after_first_resize() {
    // given
    let dir = tempfile::tempdir().unwrap();
    let registry = Registry::at(dir.path());
    start(&registry, "waits", "echo ready; cat");
    std::thread::sleep(Duration::from_millis(200));
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
        Frame::Replay(bytes) => assert!(String::from_utf8_lossy(&bytes).contains("ready")),
        other => panic!("expected the replay, got {other:?}"),
    }
}

#[test]
fn should_report_program_pid_in_handshake() {
    // given
    let dir = tempfile::tempdir().unwrap();
    let registry = Registry::at(dir.path());
    start(&registry, "pid", "cat");

    // when
    let client = Client::connect(&registry, "pid").unwrap();

    // then
    assert_eq!(client.program_pid(), registry.session("pid").unwrap().unwrap().program_pid);
}

#[test]
fn should_forget_session_when_its_program_exits() {
    // given
    let dir = tempfile::tempdir().unwrap();
    let registry = Registry::at(dir.path());
    start(&registry, "exits", "read line; exit 3");
    let mut client = attach(&registry, "exits");

    // when
    client.send(&Frame::Input(b"\n".to_vec())).unwrap();
    let code = exit_code(&mut client);

    // then
    assert_eq!(code, Some(3));
    assert!(registry.sessions().unwrap().is_empty());
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0);
}

#[test]
fn should_kill_program_that_ignores_hang_up() {
    // given
    let dir = tempfile::tempdir().unwrap();
    let registry = Registry::at(dir.path());
    start(&registry, "stubborn", "trap '' HUP; echo trapped; sleep 30");
    let mut client = attach(&registry, "stubborn");
    read_until(&mut client, "trapped");
    let started = Instant::now();

    // when
    client.hang_up().unwrap();
    let code = exit_code(&mut client);

    // then
    assert!(code.is_some(), "the program should have exited");
    assert!(started.elapsed() < Duration::from_secs(4), "took {:?}", started.elapsed());
    assert!(registry.sessions().unwrap().is_empty());
}

#[test]
fn should_keep_client_that_stalls_briefly() {
    // given
    let dir = tempfile::tempdir().unwrap();
    let registry = Registry::at(dir.path());
    start(&registry, "flood", "yes | head -c 4000000; echo; echo flooded; cat");
    let mut client = attach(&registry, "flood");

    // when
    std::thread::sleep(Duration::from_secs(2));
    read_until(&mut client, "flooded");
    client.send(&Frame::Input(b"ok\n".to_vec())).unwrap();

    // then
    read_until(&mut client, "ok\r\nok");
}

#[test]
fn should_drop_session_whose_host_was_killed() {
    // given
    let dir = tempfile::tempdir().unwrap();
    let registry = Registry::at(dir.path());
    let host = start(&registry, "orphan", "sleep 30");
    let program = registry.session("orphan").unwrap().unwrap().program_pid;

    // when
    // SAFETY: signals only processes this test started
    unsafe {
        libc::kill(host as libc::pid_t, libc::SIGKILL);
        libc::killpg(program as libc::pid_t, libc::SIGKILL);
    }

    // then
    wait_for(|| registry.sessions().unwrap().is_empty());
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0);
}

#[test]
fn should_hand_session_to_the_latest_client() {
    // given
    let dir = tempfile::tempdir().unwrap();
    let registry = Registry::at(dir.path());
    start(&registry, "shared", "cat");
    let mut first = attach(&registry, "shared");

    // when
    let mut second = attach(&registry, "shared");
    second.send(&Frame::Input(b"hi\n".to_vec())).unwrap();
    read_until(&mut second, "hi\r\nhi");

    // then
    let ended = std::iter::from_fn(|| first.receive().ok()).count();
    assert!(ended < 10, "the first client should have been disconnected");
}

#[test]
fn should_keep_the_registry_dir_from_the_session_program() {
    // given
    let dir = tempfile::tempdir().unwrap();
    let registry = Registry::at(dir.path());

    // when
    start(&registry, "dir", "echo \"dir=${IVE_PTY_DIR:-unset}\"; cat");
    let mut client = attach(&registry, "dir");

    // then
    read_until(&mut client, "dir=unset");
}

#[test]
fn should_pass_env_and_labels_to_session() {
    // given
    let dir = tempfile::tempdir().unwrap();
    let registry = Registry::at(dir.path());
    let config = HostConfig {
        env: vec![("COLORTERM".into(), "truecolor".into())],
        labels: vec![("ive.project".into(), "p1".into())],
        ..config("env", &["sh", "-c", "echo \"c=$COLORTERM\"; cat"])
    };

    // when
    host::start(&launcher(), &registry, &config).unwrap();
    let mut client = attach(&registry, "env");

    // then
    read_until(&mut client, "c=truecolor");
    assert_eq!(registry.session("env").unwrap().unwrap().labels, config.labels);
}

#[test]
fn should_not_leak_starter_descriptors_into_host() {
    // given
    let dir = tempfile::tempdir().unwrap();
    let registry = Registry::at(dir.path());
    let marker = dir.path().join("marker");
    let path = std::ffi::CString::new(marker.to_str().unwrap()).unwrap();
    // SAFETY: plain open without O_CLOEXEC, so a spawned child inherits it
    let fd = unsafe { libc::open(path.as_ptr(), libc::O_CREAT | libc::O_RDWR, 0o600) };
    assert!(fd >= 0);

    // when
    let host = start(&registry, "fds", "cat");
    // SAFETY: closes the descriptor opened above
    unsafe { libc::close(fd) };

    // then
    let links: Vec<_> = std::fs::read_dir(format!("/proc/{host}/fd")).unwrap().filter_map(|entry| std::fs::read_link(entry.ok()?.path()).ok()).collect();
    assert!(!links.iter().any(|link| link.ends_with("marker")), "host holds {links:?}");
}

#[test]
fn should_report_failed_start_and_clean_up_its_log() {
    // given
    let dir = tempfile::tempdir().unwrap();
    let registry = Registry::at(dir.path());
    let started = Instant::now();

    // when
    let error = host::start(&launcher(), &registry, &config("broken", &["/nonexistent/program"])).unwrap_err();

    // then
    assert!(error.to_string().contains("/nonexistent/program"), "{error}");
    assert!(started.elapsed() < Duration::from_secs(2), "took {:?}", started.elapsed());
    assert!(!registry.log("broken").exists());
}

#[test]
fn should_deliver_large_paste_to_program_that_echoes() {
    // given
    let dir = tempfile::tempdir().unwrap();
    let registry = Registry::at(dir.path());
    start(&registry, "paste", "stty raw -echo; echo ready; cat");
    let mut client = attach(&registry, "paste");
    read_until(&mut client, "ready");
    let paste: Vec<u8> = (0..1_000_000).map(|index| b'a' + (index % 26) as u8).collect();

    // when
    client.send(&Frame::Input(paste.clone())).unwrap();
    client.send(&Frame::Input(b"END".to_vec())).unwrap();
    let echoed = read_until(&mut client, "END");

    // then
    let received = echoed.trim_start_matches(['\r', '\n']);
    assert_eq!(received.len(), paste.len() + 3, "lost {} bytes", (paste.len() + 3).saturating_sub(received.len()));
    assert!(received.as_bytes().starts_with(&paste));
    client.hang_up().unwrap();
}

#[test]
fn should_hang_up_when_its_registry_entry_disappears() {
    // given
    let dir = tempfile::tempdir().unwrap();
    let registry = Registry::at(dir.path());
    let host = start(&registry, "orphaned", "sleep 300");

    // when
    for entry in std::fs::read_dir(dir.path()).unwrap() {
        std::fs::remove_file(entry.unwrap().path()).unwrap();
    }

    // then
    // SAFETY: signal 0 only checks that the process exists
    wait_for(|| unsafe { libc::kill(host as libc::pid_t, 0) } != 0 || is_zombie(host));
}

fn process_state(pid: u32) -> Option<char> {
    let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    stat.rsplit(')').next()?.trim_start().chars().next()
}

fn is_zombie(pid: u32) -> bool {
    std::fs::read_to_string(format!("/proc/{pid}/stat")).map_or(true, |stat| stat.rsplit(')').next().is_some_and(|rest| rest.trim_start().starts_with('Z')))
}

#[test]
fn should_hang_up_even_when_another_client_connects_at_the_same_moment() {
    // given
    let dir = tempfile::tempdir().unwrap();
    let registry = Registry::at(dir.path());
    let host = start(&registry, "race", "echo ready; sleep 300");
    let mut first = attach(&registry, "race");
    read_until(&mut first, "ready");
    let program = first.program_pid();
    // frozen, the host sees the hang-up and the new connection in the same poll round
    // SAFETY: stops and resumes only the host this test started
    unsafe { libc::kill(host as libc::pid_t, libc::SIGSTOP) };
    wait_for(|| process_state(host) == Some('T'));
    first.hang_up().unwrap();
    let second = std::os::unix::net::UnixStream::connect(registry.socket("race")).unwrap();

    // when
    // SAFETY: as above
    unsafe { libc::kill(host as libc::pid_t, libc::SIGCONT) };

    // then
    // SAFETY: signal 0 only checks that the process exists
    wait_for(|| unsafe { libc::kill(program as libc::pid_t, 0) } != 0 || is_zombie(program));
    drop(second);
}
