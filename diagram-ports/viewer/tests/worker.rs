use diagram_viewer::worker::{self, Job, Limits};
use std::{
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
fn job(source: &str, limits: Limits, cancelled: Arc<AtomicBool>) -> Job {
    Job {
        limits,
        cancelled,
        ..Job::headless(source)
    }
}

fn binary() -> &'static Path {
    Path::new(env!("CARGO_BIN_EXE_diagram-viewer"))
}

#[test]
fn should_render_real_svg_in_an_isolated_worker() {
    // given
    let source = "flowchart LR\n A[Start] --> B[Finish]";
    // when
    let output = worker::run(
        binary(),
        job(source, Limits::default(), Arc::new(AtomicBool::new(false))),
    )
    .unwrap();
    // then
    assert!(output.svg.contains("Start") && output.svg.contains("Finish"));
    assert_eq!(output.metadata.family, "flowchart-v2");
}

#[test]
fn should_report_invalid_source_from_worker() {
    // given
    let source = "this is not a diagram";
    // when
    let result = worker::run(
        binary(),
        job(source, Limits::default(), Arc::new(AtomicBool::new(false))),
    );
    // then
    assert!(result.unwrap_err().contains("invalid"));
}

#[test]
fn should_enforce_hard_deadline_independently_of_worker_io() {
    // given
    let limits = Limits {
        timeout: Duration::from_nanos(1),
        ..Limits::default()
    };
    let started = Instant::now();
    // when
    let error = worker::run(
        binary(),
        job(
            "flowchart LR\n A-->B",
            limits,
            Arc::new(AtomicBool::new(false)),
        ),
    )
    .unwrap_err();
    // then
    assert!(error.contains("deadline"), "{error}");
    assert!(started.elapsed() < Duration::from_secs(2));
}

#[test]
fn should_reject_worker_svg_over_output_limit() {
    // given
    let limits = Limits {
        svg: 32,
        ..Limits::default()
    };
    // when
    let error = worker::run(
        binary(),
        job(
            "flowchart LR\n A-->B",
            limits,
            Arc::new(AtomicBool::new(false)),
        ),
    )
    .unwrap_err();
    // then
    assert!(error.contains("limit"), "{error}");
}

#[test]
fn should_cancel_and_reap_an_active_worker() {
    // given
    let cancelled = Arc::new(AtomicBool::new(false));
    let signal = cancelled.clone();
    let cancel = std::thread::spawn(move || signal.store(true, Ordering::Release));
    // when
    let error = worker::run(
        binary(),
        job("flowchart LR\n A-->B", Limits::default(), cancelled),
    )
    .unwrap_err();
    cancel.join().unwrap();
    // then
    assert!(error.contains("cancelled"), "{error}");
}

#[test]
fn should_report_early_worker_exit_without_waiting_for_deadline() {
    // given
    let started = Instant::now();
    // when
    let error = worker::run(
        Path::new("/does-not-exist/diagram-viewer"),
        job(
            "flowchart LR\n A-->B",
            Limits::default(),
            Arc::new(AtomicBool::new(false)),
        ),
    )
    .unwrap_err();
    // then
    assert!(error.contains("start"), "{error}");
    assert!(started.elapsed() < Duration::from_secs(2));
}
