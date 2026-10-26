use crate::{
    cli::Input,
    examples,
    native_text::NativeTextResources,
    svg::DisplaySvg,
    viewport::{self, Viewport, Zoom},
    worker::{self, Job, Measurement, Metadata},
};
use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender, unbounded};
use gpui::{RenderImage, TextSystem};
use std::{
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::JoinHandle,
};

pub struct Request {
    pub generation: u64,
    pub input: Input,
    pub language: String,
    pub zoom: Zoom,
    pub viewport: (f32, f32),
    pub scale: f32,
}
pub struct Ready {
    pub image: Arc<RenderImage>,
    pub logical_size: (f32, f32),
    pub metadata: Metadata,
    pub display_note: String,
}
pub struct Response {
    pub generation: u64,
    pub source: String,
    pub result: Result<Ready, String>,
}
enum Pending {
    Idle,
    Render(Request),
    Closed,
}
struct Queue {
    pending: Pending,
    active: Arc<AtomicBool>,
}
enum Thread {
    Running(JoinHandle<()>),
    Joined,
}
pub struct Service {
    queue: Arc<(Mutex<Queue>, Condvar)>,
    thread: Thread,
}

impl Service {
    pub fn start(text: Arc<TextSystem>) -> (Self, UnboundedReceiver<Response>) {
        let queue = Arc::new((
            Mutex::new(Queue {
                pending: Pending::Idle,
                active: Arc::new(AtomicBool::new(false)),
            }),
            Condvar::new(),
        ));
        let (sender, receiver) = unbounded();
        let worker_queue = queue.clone();
        let thread = std::thread::spawn(move || work(worker_queue, sender, text));
        (
            Self {
                queue,
                thread: Thread::Running(thread),
            },
            receiver,
        )
    }
    pub fn submit(&self, request: Request) {
        let (lock, signal) = &*self.queue;
        if let Ok(mut queue) = lock.lock() {
            queue.active.store(true, Ordering::Release);
            queue.pending = Pending::Render(request);
            signal.notify_one();
        }
    }
    pub fn stop(&mut self) -> impl FnOnce() + Send + 'static {
        let (lock, signal) = &*self.queue;
        if let Ok(mut queue) = lock.lock() {
            queue.active.store(true, Ordering::Release);
            queue.pending = Pending::Closed;
            signal.notify_one();
        }
        let thread = std::mem::replace(&mut self.thread, Thread::Joined);
        move || {
            if let Thread::Running(thread) = thread {
                let _ = thread.join();
            }
        }
    }
}
impl Drop for Service {
    fn drop(&mut self) {
        let cleanup = self.stop();
        std::thread::spawn(cleanup);
    }
}

enum Cache {
    Empty,
    Parsed {
        source: String,
        language: String,
        svg: Box<DisplaySvg>,
        metadata: Metadata,
    },
}

fn work(
    queue: Arc<(Mutex<Queue>, Condvar)>,
    sender: UnboundedSender<Response>,
    text: Arc<TextSystem>,
) {
    let text = NativeTextResources::new(text);
    let mut cache = Cache::Empty;
    while let Some((request, cancelled)) = next(&queue) {
        let source = match &request.input {
            Input::File(path) => worker::read_source(path),
            Input::Example(name) => examples::source(name).map(str::to_owned),
        };
        let result = source.as_ref().map_err(Clone::clone).and_then(|source| {
            render(
                &request,
                source,
                RenderContext {
                    text: &text,
                    cancelled,
                    cache: &mut cache,
                },
            )
        });
        let response = Response {
            generation: request.generation,
            source: source.unwrap_or_default(),
            result,
        };
        if sender.unbounded_send(response).is_err() {
            break;
        }
    }
}

fn next(queue: &Arc<(Mutex<Queue>, Condvar)>) -> Option<(Request, Arc<AtomicBool>)> {
    let (lock, signal) = &**queue;
    let mut queue = lock.lock().ok()?;
    loop {
        match std::mem::replace(&mut queue.pending, Pending::Idle) {
            Pending::Render(request) => {
                let cancelled = Arc::new(AtomicBool::new(false));
                queue.active = cancelled.clone();
                return Some((request, cancelled));
            }
            Pending::Closed => {
                queue.pending = Pending::Closed;
                return None;
            }
            Pending::Idle => queue = signal.wait(queue).ok()?,
        }
    }
}

struct RenderContext<'a> {
    text: &'a NativeTextResources,
    cancelled: Arc<AtomicBool>,
    cache: &'a mut Cache,
}

fn render(request: &Request, source: &str, context: RenderContext<'_>) -> Result<Ready, String> {
    let same = matches!(context.cache, Cache::Parsed { source: cached, language, .. } if cached == source && language == &request.language);
    if !same {
        let native = context.text.start_render();
        let executable = std::env::current_exe()
            .map_err(|error| format!("cannot locate diagram worker executable: {error}"))?;
        let job = Job {
            source: source.into(),
            language: request.language.clone(),
            cancelled: context.cancelled.clone(),
            measurement: Measurement::Native(native.clone()),
            ..Job::headless("")
        };
        let output = worker::run(&executable, job)?;
        let svg = DisplaySvg::parse(&output.svg, &context.cancelled)?;
        let mut metadata = output.metadata;
        metadata.diagnostics.extend(native.report());
        metadata.diagnostics.extend(svg.diagnostic());
        *context.cache = Cache::Parsed {
            source: source.into(),
            language: request.language.clone(),
            svg: Box::new(svg),
            metadata,
        };
    }
    if context.cancelled.load(Ordering::Acquire) {
        return Err("diagram rendering cancelled".into());
    }
    let Cache::Parsed { svg, metadata, .. } = context.cache else {
        return Err("diagram cache is empty".into());
    };
    let layout = viewport::layout(
        svg.dimensions(),
        Viewport {
            size: (
                request.viewport.0 / request.scale,
                request.viewport.1 / request.scale,
            ),
            device_scale: request.scale,
        },
        request.zoom,
    )?;
    let image = svg.render(layout.raster_size, &context.cancelled)?;
    Ok(Ready {
        image,
        logical_size: layout.logical_size,
        metadata: metadata.clone(),
        display_note: svg.diagnostic().unwrap_or_default(),
    })
}
