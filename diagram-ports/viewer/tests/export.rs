use std::{
    process::Command,
    time::{Duration, Instant},
};

#[test]
fn should_export_svg_without_a_display_server() {
    // given
    let directory = tempfile::tempdir().unwrap();
    let source = directory.path().join("source ü.mmd");
    let output = directory.path().join("result.svg");
    std::fs::write(&source, "flowchart LR\n A[Hello] --> B[World]").unwrap();
    // when
    let result = Command::new(env!("CARGO_BIN_EXE_diagram-viewer"))
        .env_remove("DISPLAY")
        .env_remove("WAYLAND_DISPLAY")
        .arg("--render")
        .arg(&source)
        .arg("--output")
        .arg(&output)
        .output()
        .unwrap();
    // then
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let svg = std::fs::read_to_string(output).unwrap();
    let parsed = roxmltree::Document::parse(&svg).unwrap();
    assert_eq!(parsed.root_element().tag_name().name(), "svg");
    assert!(svg.contains("Hello") && svg.contains("World"));
}

#[test]
fn should_leave_the_output_untouched_when_rendering_fails() {
    // given
    let directory = tempfile::tempdir().unwrap();
    let source = directory.path().join("invalid.mmd");
    let output = directory.path().join("result.svg");
    std::fs::write(&source, "not a diagram").unwrap();
    std::fs::write(&output, "previous output").unwrap();
    // when
    let result = Command::new(env!("CARGO_BIN_EXE_diagram-viewer"))
        .arg("--render")
        .arg(&source)
        .arg("--output")
        .arg(&output)
        .output()
        .unwrap();
    // then
    assert!(!result.status.success());
    assert_eq!(std::fs::read_to_string(output).unwrap(), "previous output");
    assert!(String::from_utf8_lossy(&result.stderr).contains("invalid"));
}

#[test]
fn should_reject_an_oversize_selected_file() {
    // given
    let directory = tempfile::tempdir().unwrap();
    let source = directory.path().join("large.mmd");
    std::fs::write(&source, vec![b'x'; 262_145]).unwrap();
    // when
    let error = diagram_viewer::worker::read_source(&source).unwrap_err();
    // then
    assert!(error.contains("262144") && error.contains("large.mmd"));
}

#[cfg(unix)]
#[test]
fn should_reject_a_named_pipe_without_blocking_file_open() {
    // given
    use std::os::unix::ffi::OsStrExt;
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("source.fifo");
    let name = std::ffi::CString::new(path.as_os_str().as_bytes()).unwrap();
    assert_eq!(unsafe { libc::mkfifo(name.as_ptr(), 0o600) }, 0);
    let started = Instant::now();
    // when
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = sender.send(diagram_viewer::worker::read_source(&path));
    });
    let error = receiver
        .recv_timeout(Duration::from_millis(100))
        .expect("opening a named pipe blocked the source reader")
        .unwrap_err();
    // then
    assert!(error.contains("regular file"), "{error}");
    assert!(started.elapsed() < Duration::from_secs(1));
}
