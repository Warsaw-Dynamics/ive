use diagram_engine::DotRunner;
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

pub struct Process;

const HELPER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/ive-dot-helper"));

struct ChildGuard(Child);

impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

impl DotRunner for Process {
    fn run(&self, source: &str) -> Result<String, String> {
        let (_directory, executable) = materialize()?;
        run_executable(&executable, source)
    }
}

fn materialize() -> Result<(tempfile::TempDir, PathBuf), String> {
    let directory = tempfile::Builder::new()
        .prefix("ive-dot-")
        .tempdir()
        .map_err(|error| format!("cannot create bundled DOT directory: {error}"))?;
    let executable = directory.path().join("dot");
    fs::write(&executable, HELPER).map_err(|error| {
        format!(
            "cannot write bundled DOT helper {}: {error}",
            executable.display()
        )
    })?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&executable, fs::Permissions::from_mode(0o700)).map_err(|error| {
            format!(
                "cannot make bundled DOT helper executable {}: {error}",
                executable.display()
            )
        })?;
    }
    Ok((directory, executable))
}

fn run_executable(executable: &Path, source: &str) -> Result<String, String> {
    if source.len() > 2 * 1024 * 1024 {
        return Err(format!(
            "DOT input has {} bytes, exceeding 2 MiB",
            source.len()
        ));
    }
    let mut command = Command::new(executable);
    command
        .arg("-Tjson0")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::process::CommandExt;
        unsafe {
            command.pre_exec(|| {
                if libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL) == -1 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
    }
    let mut child = ChildGuard(
        command
            .spawn()
            .map_err(|error| format!("cannot start dot: {error}"))?,
    );
    let mut input = child.0.stdin.take().ok_or("dot stdin unavailable")?;
    let output = child.0.stdout.take().ok_or("dot stdout unavailable")?;
    let stderr = child.0.stderr.take().ok_or("dot stderr unavailable")?;
    let source = source.as_bytes().to_vec();
    let writer = std::thread::spawn(move || input.write_all(&source));
    let stdout = std::thread::spawn(move || read_bounded(output, 8 * 1024 * 1024));
    let errors = std::thread::spawn(move || read_bounded(stderr, 64 * 1024));
    let started = Instant::now();
    let status = loop {
        if let Some(status) = child
            .0
            .try_wait()
            .map_err(|error| format!("cannot inspect dot: {error}"))?
        {
            break status;
        }
        if started.elapsed() >= Duration::from_secs(8) {
            return Err("dot exceeded its 8-second deadline".into());
        }
        std::thread::sleep(Duration::from_millis(2));
    };
    let written = writer.join().map_err(|_| "dot input thread panicked")?;
    let output = stdout.join().map_err(|_| "dot output thread panicked")??;
    let errors = errors.join().map_err(|_| "dot error thread panicked")??;
    if !status.success() {
        return Err(format!(
            "dot exited with {status}: {}",
            String::from_utf8_lossy(&errors)
        ));
    }
    written.map_err(|error| format!("cannot write dot input: {error}"))?;
    String::from_utf8(output).map_err(|error| format!("dot output is not UTF-8: {error}"))
}

fn read_bounded(pipe: impl Read, limit: usize) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    pipe.take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("cannot read dot output: {error}"))?;
    if bytes.len() > limit {
        return Err(format!("dot output exceeds {limit} bytes"));
    }
    Ok(bytes)
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn should_report_a_crashed_dot_child_without_crashing_its_caller() {
        // given
        let directory = tempfile::tempdir().unwrap();
        let executable = directory.path().join("dot");
        fs::write(&executable, "#!/bin/sh\nulimit -c 0\nkill -SEGV $$\n").unwrap();
        fs::set_permissions(&executable, fs::Permissions::from_mode(0o700)).unwrap();

        // when
        let result = run_executable(&executable, "digraph G { a -> b }");

        // then
        assert!(result.unwrap_err().contains("dot exited with"));
    }
}
