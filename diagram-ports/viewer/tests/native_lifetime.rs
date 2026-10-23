#![cfg(target_os = "linux")]

use diagram_viewer::{
    native_text::{NativeText, NativeTextResources},
    worker::{self, Job, MeasureQuery, MeasureReply, Measurement, Measurer},
};
use std::{
    path::Path,
    sync::{
        Arc, Condvar, Mutex, Weak,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};

fn resources() -> (Arc<gpui::TextSystem>, NativeTextResources) {
    let platform = gpui_platform::current_platform(true);
    let system = Arc::new(gpui::TextSystem::new(platform.text_system()));
    let resources = NativeTextResources::new(system.clone());
    (system, resources)
}

fn run(job: Job) -> Result<worker::Output, String> {
    worker::run(Path::new(env!("CARGO_BIN_EXE_diagram-viewer")), job)
}

#[test]
fn should_drop_each_completed_native_render_cache_while_reusing_fonts() {
    // given
    let (fonts, resources) = resources();
    let shared_fonts = Arc::downgrade(&fonts);
    drop(fonts);
    // when
    for source in [
        "flowchart LR\n A[First unique label]-->B",
        "flowchart LR\n A[Second unrelated label]-->B",
    ] {
        let native = resources.start_render();
        let cache = Arc::downgrade(&native);
        let output = run(Job {
            measurement: Measurement::Native(native),
            ..Job::headless(source)
        })
        .unwrap();
        // then
        assert!(output.metadata.host_calls > 0);
        assert!(
            cache.upgrade().is_none(),
            "completed native layout cache was retained by shared resources"
        );
        assert!(
            shared_fonts.upgrade().is_some(),
            "shared native fonts were dropped with the job"
        );
    }
    drop(resources);
    assert!(shared_fonts.upgrade().is_none());
}

#[test]
fn should_measure_plantuml_in_native_worker_and_release_each_cache() {
    // given
    let (_fonts, resources) = resources();
    for source in [
        include_str!("../../fixtures/plantuml/nwdiag.puml"),
        include_str!("../../fixtures/plantuml/salt.puml"),
        include_str!("../../fixtures/plantuml/gantt.puml"),
        include_str!("../../fixtures/plantuml/timing.puml"),
        include_str!("../../fixtures/plantuml/constantsequence.puml"),
        include_str!("../../fixtures/plantuml/constantgraph.puml"),
        include_str!("../../fixtures/plantuml/styledsequence.puml"),
        include_str!("../../fixtures/plantuml/styledgraph.puml"),
        include_str!("../../fixtures/plantuml/sequence.puml"),
        include_str!("../../fixtures/plantuml/deployment.puml"),
        include_str!("../../fixtures/plantuml/state.puml"),
        include_str!("../../fixtures/plantuml/activity.puml"),
        include_str!("../../fixtures/plantuml/mindmap.puml"),
        include_str!("../../fixtures/plantuml/wbs.puml"),
        include_str!("../../fixtures/plantuml/json.puml"),
        include_str!("../../fixtures/plantuml/yaml.puml"),
    ] {
        let native = resources.start_render();
        let cache = Arc::downgrade(&native);
        let job = Job {
            language: "plantuml".into(),
            measurement: Measurement::Native(native),
            ..Job::headless(source)
        };
        // when
        let output = run(job).unwrap();
        // then
        eprintln!(
            "PlantUML {}: native={}, fallback={}",
            output.metadata.family, output.metadata.host_calls, output.metadata.fallback_calls
        );
        assert!(output.metadata.host_calls > 0);
        assert_eq!(output.metadata.fallback_calls, 0);
        assert!(cache.upgrade().is_none());
    }
}

struct PausedNative {
    native: Arc<NativeText>,
    entered: mpsc::Sender<()>,
    release: Arc<(Mutex<bool>, Condvar)>,
}

impl Measurer for PausedNative {
    fn measure(&self, query: &MeasureQuery) -> MeasureReply {
        let reply = self.native.measure(query);
        let _ = self.entered.send(());
        let (lock, signal) = &*self.release;
        let mut released = lock.lock().unwrap();
        while !*released {
            released = signal.wait(released).unwrap();
        }
        reply
    }
}

fn await_drop(cache: &Weak<NativeText>) {
    let started = Instant::now();
    while cache.upgrade().is_some() && started.elapsed() < Duration::from_secs(1) {
        std::thread::sleep(Duration::from_millis(2));
    }
    assert!(
        cache.upgrade().is_none(),
        "cancelled callback's native cache survived callback completion"
    );
}

#[test]
fn should_keep_a_cancelled_callback_cache_alive_only_until_that_callback_finishes() {
    // given
    let (_fonts, resources) = resources();
    let native = resources.start_render();
    let old_cache = Arc::downgrade(&native);
    let release = Arc::new((Mutex::new(false), Condvar::new()));
    let (entered, started) = mpsc::channel();
    let cancelled = Arc::new(AtomicBool::new(false));
    let job = Job {
        measurement: Measurement::Native(Arc::new(PausedNative {
            native,
            entered,
            release: release.clone(),
        })),
        cancelled: cancelled.clone(),
        ..Job::headless("flowchart LR\n A[Cancelled label]-->B")
    };
    let worker = std::thread::spawn(move || run(job));
    started.recv_timeout(Duration::from_secs(2)).unwrap();
    // when
    cancelled.store(true, Ordering::Release);
    let error = worker.join().unwrap().unwrap_err();
    assert!(error.contains("cancelled"));
    assert!(
        old_cache.upgrade().is_some(),
        "in-flight callback lost its native cache"
    );
    let current = resources.start_render();
    let current_cache = Arc::downgrade(&current);
    let output = run(Job {
        measurement: Measurement::Native(current),
        ..Job::headless("flowchart LR\n A[Current label]-->B")
    })
    .unwrap();
    let (lock, signal) = &*release;
    *lock.lock().unwrap() = true;
    signal.notify_all();
    // then
    assert!(output.svg.contains("Current"));
    assert!(
        current_cache.upgrade().is_none(),
        "current cache was retained after completion"
    );
    await_drop(&old_cache);
}
