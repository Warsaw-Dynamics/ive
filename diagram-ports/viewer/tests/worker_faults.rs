//! A test-only native child executable exercises hostile pipe and memory behavior.
use diagram_viewer::worker::{self, Job, Limits};
use std::{
    io::{Read, Write},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

fn main() {
    if std::env::args().any(|arg| arg == "--worker") {
        fixture();
        return;
    }
    for mode in [
        "blocked-stdin",
        "blocked-stdout",
        "stderr-flood",
        "oversize-frame",
        "early-exit",
        "memory",
        "cancel",
    ] {
        should_reap_a_faulting_worker(mode);
        println!("passed worker fault: {mode}");
    }
}

fn should_reap_a_faulting_worker(mode: &'static str) {
    // given
    let directory = tempfile::tempdir().unwrap();
    let pid_file = directory.path().join("worker.pid");
    let mut source = format!("{}\n", pid_file.display());
    if mode == "blocked-stdin" {
        source.push_str(&"x".repeat(250_000));
    }
    let cancelled = Arc::new(AtomicBool::new(false));
    let limits = Limits {
        timeout: if mode == "cancel" {
            Duration::from_secs(2)
        } else {
            Duration::from_millis(200)
        },
        stderr: 32,
        memory: 128 * 1024 * 1024,
        ..Default::default()
    };
    let job = Job {
        source,
        language: mode.into(),
        limits,
        cancelled: cancelled.clone(),
        ..Job::headless("")
    };
    let marker = pid_file.clone();
    let cancel = std::thread::spawn(move || {
        if mode == "cancel" {
            let started = Instant::now();
            while !marker.exists() && started.elapsed() < Duration::from_secs(1) {
                std::thread::sleep(Duration::from_millis(2));
            }
            cancelled.store(true, Ordering::Release);
        }
    });
    let started = Instant::now();
    // when
    let error = worker::run(&std::env::current_exe().unwrap(), job).unwrap_err();
    cancel.join().unwrap();
    // then
    let expected = match mode {
        "blocked-stdin" | "blocked-stdout" => "deadline",
        "stderr-flood" => "stderr",
        "oversize-frame" => "limit",
        "memory" => "allocation refused",
        "cancel" => "cancelled",
        _ => "worker",
    };
    assert!(error.contains(expected), "{mode}: {error}");
    assert!(
        started.elapsed() < Duration::from_secs(2),
        "{mode}: deadline monitor stalled"
    );
    #[cfg(unix)]
    {
        let pid: i32 = std::fs::read_to_string(pid_file).unwrap().parse().unwrap();
        assert_eq!(
            unsafe { libc::kill(pid, 0) },
            -1,
            "{mode}: child {pid} survived"
        );
        assert_eq!(
            std::io::Error::last_os_error().raw_os_error(),
            Some(libc::ESRCH)
        );
    }
}

fn frame(reader: &mut impl Read) -> (u8, Vec<u8>) {
    let mut header = [0_u8; 5];
    reader.read_exact(&mut header).unwrap();
    let length = u32::from_be_bytes(header[1..].try_into().unwrap()) as usize;
    assert!(length <= 262_144);
    let mut data = vec![0; length];
    reader.read_exact(&mut data).unwrap();
    (header[0], data)
}

fn fixture() {
    let mut input = std::io::stdin().lock();
    let (_, config) = frame(&mut input);
    let config: serde_json::Value = serde_json::from_slice(&config).unwrap();
    let mode = config["language"].as_str().unwrap();
    let path = if mode == "blocked-stdin" {
        let mut header = [0; 5];
        input.read_exact(&mut header).unwrap();
        let mut path = Vec::new();
        loop {
            let mut byte = [0];
            input.read_exact(&mut byte).unwrap();
            if byte[0] == b'\n' {
                break;
            }
            path.push(byte[0]);
        }
        String::from_utf8(path).unwrap()
    } else {
        String::from_utf8(frame(&mut input).1)
            .unwrap()
            .trim()
            .to_owned()
    };
    std::fs::write(path, std::process::id().to_string()).unwrap();
    match mode {
        "stderr-flood" => {
            std::io::stderr().write_all(&[b'x'; 4096]).unwrap();
        }
        "oversize-frame" => {
            std::io::stdout()
                .write_all(&[6, 255, 255, 255, 255])
                .unwrap();
            std::io::stdout().flush().unwrap();
        }
        "early-exit" => std::process::exit(23),
        "memory" => {
            #[cfg(unix)]
            {
                let mut limit = libc::rlimit {
                    rlim_cur: 0,
                    rlim_max: 0,
                };
                assert_eq!(unsafe { libc::getrlimit(libc::RLIMIT_AS, &mut limit) }, 0);
                assert_eq!(limit.rlim_cur, 128 * 1024 * 1024);
                let mut bytes = Vec::<u8>::new();
                assert!(bytes.try_reserve_exact(256 * 1024 * 1024).is_err());
            }
            let message = b"allocation refused by worker memory limit";
            let mut output = std::io::stdout().lock();
            output.write_all(&[7]).unwrap();
            output
                .write_all(&(message.len() as u32).to_be_bytes())
                .unwrap();
            output.write_all(message).unwrap();
            output.flush().unwrap();
            return;
        }
        _ => {}
    }
    std::thread::sleep(Duration::from_secs(30));
}
