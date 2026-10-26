use diagram_engine::{GraphLayout, Language, RenderLimits, RenderOptions};
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    path::Path,
    process::{Child, Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};
mod protocol;
mod text;
pub use text::{MeasureQuery, MeasureReply, Measurer};

#[derive(Clone)]
pub struct Limits {
    pub source: usize,
    pub svg: usize,
    pub stderr: usize,
    pub timeout: Duration,
    pub memory: u64,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            source: 256 * 1024,
            svg: 16 * 1024 * 1024,
            stderr: 64 * 1024,
            timeout: Duration::from_secs(10),
            memory: 2 * 1024 * 1024 * 1024,
        }
    }
}

pub enum Measurement {
    Deterministic,
    Native(Arc<dyn Measurer>),
}

pub struct Job {
    pub source: String,
    pub language: String,
    pub limits: Limits,
    pub cancelled: Arc<AtomicBool>,
    pub measurement: Measurement,
}

impl Job {
    pub fn headless(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            language: "mermaid".into(),
            limits: Limits::default(),
            cancelled: Arc::new(AtomicBool::new(false)),
            measurement: Measurement::Deterministic,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Metadata {
    pub family: String,
    pub backend: String,
    pub upstream: String,
    pub diagnostics: Vec<String>,
    pub measurement: Vec<String>,
    pub host_calls: u64,
    pub fallback_calls: u64,
}

#[derive(Debug)]
pub struct Output {
    pub svg: String,
    pub metadata: Metadata,
}

#[derive(Serialize, Deserialize)]
struct Config {
    language: String,
    source_limit: usize,
    svg_limit: usize,
    timeout_ms: u64,
    native: bool,
}

enum Event {
    Protocol(Result<Output, String>),
    Stderr(Result<String, String>),
}

struct Worker(Child);
impl Drop for Worker {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// Runs the same executable as a bounded child; cancellation, timeout and pipe limits reap it.
pub fn run(executable: &Path, job: Job) -> Result<Output, String> {
    validate(&job)?;
    let mut command = Command::new(executable);
    command.arg("--worker");
    supervise(command, job)
}

fn validate(job: &Job) -> Result<(), String> {
    if job.source.len() > job.limits.source {
        return Err(format!(
            "source has {} bytes, exceeding the {}-byte limit",
            job.source.len(),
            job.limits.source
        ));
    }
    if job.limits.source == 0
        || job.limits.source > 256 * 1024
        || job.limits.svg == 0
        || job.limits.svg > 16 * 1024 * 1024
        || job.limits.stderr == 0
        || job.limits.stderr > 64 * 1024
    {
        return Err("worker limits exceed audited source/SVG/stderr bounds".into());
    }
    if job.limits.timeout.is_zero() || job.limits.timeout > Duration::from_secs(10) {
        return Err(format!(
            "worker deadline {:?} must be greater than zero and at most 10 seconds",
            job.limits.timeout
        ));
    }
    if job.limits.memory < 64 * 1024 * 1024 || job.limits.memory > 2 * 1024 * 1024 * 1024 {
        return Err(format!(
            "worker memory limit {} must be between 64 MiB and 2 GiB",
            job.limits.memory
        ));
    }
    Ok(())
}

fn supervise(mut command: Command, job: Job) -> Result<Output, String> {
    let state = Monitor {
        started: Instant::now(),
        limits: job.limits.clone(),
        cancelled: job.cancelled.clone(),
    };
    configure(&mut command, &state.limits);
    let mut child = Worker(
        command
            .spawn()
            .map_err(|error| format!("cannot start diagram worker: {error}"))?,
    );
    let input = child.0.stdin.take().ok_or("worker stdin unavailable")?;
    let output = child.0.stdout.take().ok_or("worker stdout unavailable")?;
    let stderr = child.0.stderr.take().ok_or("worker stderr unavailable")?;
    let (sender, receiver) = mpsc::channel();
    let protocol_sender = sender.clone();
    let protocol_thread = std::thread::spawn(move || {
        let _ = protocol_sender.send(Event::Protocol(exchange(input, output, job)));
    });
    let stderr_limit = state.limits.stderr;
    let stderr_thread = std::thread::spawn(move || {
        let _ = sender.send(Event::Stderr(read_stderr(stderr, stderr_limit)));
    });
    let result = monitor(&mut child, &receiver, state);
    drop(child);
    // Child termination releases blocked pipes. Native shaping has bounded query input;
    // platform latency cannot delay the independent deadline monitor.
    if protocol_thread.is_finished() {
        let _ = protocol_thread.join();
    }
    if stderr_thread.is_finished() {
        let _ = stderr_thread.join();
    }
    result
}

struct Monitor {
    started: Instant,
    limits: Limits,
    cancelled: Arc<AtomicBool>,
}

fn monitor(
    child: &mut Worker,
    events: &mpsc::Receiver<Event>,
    state: Monitor,
) -> Result<Output, String> {
    let mut output = None;
    let mut stderr = None;
    loop {
        if state.cancelled.load(Ordering::Acquire) {
            return Err("diagram rendering cancelled".into());
        }
        if state.started.elapsed() >= state.limits.timeout {
            return Err(format!(
                "diagram worker exceeded its {:?} hard deadline",
                state.limits.timeout
            ));
        }
        match events.recv_timeout(Duration::from_millis(2)) {
            Ok(Event::Protocol(result)) => output = Some(result?),
            Ok(Event::Stderr(result)) => stderr = Some(result?),
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                if output.is_none() || stderr.is_none() {
                    return Err("diagram worker closed its response channels".into());
                }
                std::thread::sleep(Duration::from_millis(2));
            }
        }
        if let Some(status) = child
            .0
            .try_wait()
            .map_err(|error| format!("cannot inspect diagram worker: {error}"))?
        {
            if !status.success() {
                return Err(format!(
                    "diagram worker exited with {status}: {}",
                    stderr.as_deref().unwrap_or("no diagnostic received")
                ));
            }
            if stderr.is_some()
                && let Some(output) = output
            {
                return Ok(output);
            }
        }
    }
}

fn configure(command: &mut Command, limits: &Limits) {
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let memory = limits.memory as libc::rlim_t;
        // Only async-signal-safe libc calls run between fork and exec.
        unsafe {
            command.pre_exec(move || {
                let ceiling = libc::rlimit {
                    rlim_cur: memory,
                    rlim_max: memory,
                };
                if libc::setrlimit(libc::RLIMIT_AS, &ceiling) == -1 {
                    return Err(std::io::Error::last_os_error());
                }
                #[cfg(target_os = "linux")]
                if libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL) == -1 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
    }
}

fn read_stderr(stderr: impl Read, limit: usize) -> Result<String, String> {
    let mut bytes = Vec::new();
    stderr
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("cannot read worker stderr: {error}"))?;
    if bytes.len() > limit {
        return Err(format!(
            "diagram worker stderr exceeded its {limit}-byte limit"
        ));
    }
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

fn exchange(mut input: impl Write, mut output: impl Read, job: Job) -> Result<Output, String> {
    let config = Config {
        language: job.language,
        source_limit: job.limits.source,
        svg_limit: job.limits.svg,
        timeout_ms: job.limits.timeout.as_millis().max(1) as u64,
        native: matches!(job.measurement, Measurement::Native(_)),
    };
    protocol::write_json(&mut input, protocol::CONFIG, &config)?;
    protocol::write_frame(&mut input, protocol::SOURCE, job.source.as_bytes())?;
    let mut budget = protocol::Budget::new(job.limits.svg + 4 * 1024 * 1024);
    let mut queries = 0;
    loop {
        let (tag, bytes) = protocol::read_frame(&mut output, &mut budget)?;
        match tag {
            protocol::MEASURE => {
                queries += 1;
                if queries > 4096 {
                    return Err("worker text measurement query limit exceeded (4096)".into());
                }
                let query: MeasureQuery = protocol::decode(&bytes)?;
                query.validate()?;
                let reply = match &job.measurement {
                    Measurement::Native(host) => host.measure(&query),
                    Measurement::Deterministic => MeasureReply::Fallback,
                };
                protocol::write_json(&mut input, protocol::ANSWER, &reply)?;
            }
            protocol::RESULT => {
                let metadata = protocol::decode(&bytes)?;
                let (tag, svg) = protocol::read_frame(&mut output, &mut budget)?;
                if tag != protocol::SVG || svg.len() > job.limits.svg {
                    return Err(format!(
                        "worker SVG response exceeds its {}-byte limit or has the wrong frame type",
                        job.limits.svg
                    ));
                }
                let mut trailing = [0_u8];
                if output
                    .read(&mut trailing)
                    .map_err(|error| format!("cannot read worker protocol end: {error}"))?
                    != 0
                {
                    return Err(
                        "worker stdout has trailing data after SVG output limit boundary".into(),
                    );
                }
                return Ok(Output {
                    svg: String::from_utf8(svg)
                        .map_err(|error| format!("worker SVG is not UTF-8: {error}"))?,
                    metadata,
                });
            }
            protocol::ERROR => {
                return Err(format!(
                    "diagram worker: {}",
                    String::from_utf8_lossy(&bytes)
                ));
            }
            _ => return Err(format!("unexpected worker protocol frame {tag}")),
        }
    }
}

/// Internal child entrypoint; no GUI platform, file lookup or external tool is initialized.
pub fn main() -> Result<(), String> {
    let mut budget = protocol::Budget::new(256 * 1024 + protocol::HEADER_LIMIT);
    let config: Config =
        protocol::read_json(&mut std::io::stdin().lock(), protocol::CONFIG, &mut budget)?;
    let (_, source) =
        protocol::read_expected(&mut std::io::stdin().lock(), protocol::SOURCE, &mut budget)?;
    let source = String::from_utf8(source)
        .map_err(|error| format!("worker source is not UTF-8: {error}"))?;
    let options = options(config)?;
    match diagram_engine::render(&source, &options) {
        Ok(rendered) => {
            protocol::write_json(
                &mut std::io::stdout().lock(),
                protocol::RESULT,
                &metadata(&rendered),
            )?;
            protocol::write_frame(
                &mut std::io::stdout().lock(),
                protocol::SVG,
                rendered.svg.as_bytes(),
            )
        }
        Err(error) => protocol::write_frame(
            &mut std::io::stdout().lock(),
            protocol::ERROR,
            error.to_string().as_bytes(),
        ),
    }
}

fn options(config: Config) -> Result<RenderOptions, String> {
    let language = match config.language.as_str() {
        "mermaid" => Language::Mermaid,
        "plantuml" => Language::PlantUml,
        value => return Err(format!("unknown worker language {value:?}")),
    };
    if config.source_limit == 0
        || config.source_limit > 256 * 1024
        || config.svg_limit == 0
        || config.svg_limit > 16 * 1024 * 1024
        || config.timeout_ms == 0
        || config.timeout_ms > 10_000
    {
        return Err("worker configuration exceeds audited limits".into());
    }
    Ok(RenderOptions {
        language,
        limits: RenderLimits {
            source_bytes: config.source_limit,
            svg_bytes: config.svg_limit,
            timeout: Duration::from_millis(config.timeout_ms),
            ..Default::default()
        },
        text_measurement: if config.native {
            text::policy()?
        } else {
            Default::default()
        },
        graph_layout: if language == Language::PlantUml {
            GraphLayout::Dot(Arc::new(crate::dot::Process))
        } else {
            GraphLayout::Dagre
        },
        ..Default::default()
    })
}

fn metadata(rendered: &diagram_engine::RenderedDiagram) -> Metadata {
    use diagram_engine::TextMeasurementSource;
    let host_calls = rendered
        .measurement
        .entries()
        .iter()
        .filter(|entry| entry.provenance().source == TextMeasurementSource::Host)
        .map(|entry| entry.count())
        .sum();
    let fallback_calls = rendered
        .measurement
        .entries()
        .iter()
        .filter(|entry| entry.provenance().fallback_reason.is_some())
        .map(|entry| entry.count())
        .sum();
    Metadata {
        family: rendered.family.clone(),
        backend: rendered.backend.into(),
        upstream: rendered.upstream.into(),
        diagnostics: rendered.diagnostics.clone(),
        measurement: rendered
            .measurement
            .entries()
            .iter()
            .map(|entry| format!("{:?}: {}", entry.provenance(), entry.count()))
            .collect(),
        host_calls,
        fallback_calls,
    }
}

pub fn read_source(path: &Path) -> Result<String, String> {
    let mut open = std::fs::OpenOptions::new();
    open.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        open.custom_flags(libc::O_NONBLOCK);
    }
    let file = open
        .open(path)
        .map_err(|error| format!("cannot open diagram source {}: {error}", path.display()))?;
    let metadata = file
        .metadata()
        .map_err(|error| format!("cannot inspect diagram source {}: {error}", path.display()))?;
    if !metadata.is_file() {
        return Err(format!(
            "diagram source {} is not a regular file",
            path.display()
        ));
    }
    let mut source = Vec::new();
    file.take(256 * 1024 + 1)
        .read_to_end(&mut source)
        .map_err(|error| format!("cannot read diagram source {}: {error}", path.display()))?;
    if source.len() > 256 * 1024 {
        return Err(format!(
            "diagram source {} exceeds the 262144-byte limit",
            path.display()
        ));
    }
    String::from_utf8(source)
        .map_err(|error| format!("diagram source {} is not UTF-8: {error}", path.display()))
}
