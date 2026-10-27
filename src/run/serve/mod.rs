//! The static server: `ive --serve`, run as a run's program, its setup in `IVE_SERVE`.

pub mod files;
pub mod hook;
pub mod live;

use std::convert::Infallible;
use std::path::{Path, PathBuf};
use std::sync::{Arc, mpsc};
use std::time::{Duration, Instant};

use futures::channel::oneshot;
use http_body_util::StreamBody;
use hyper::body::{Bytes, Frame, Incoming};
use hyper::header::{self, HeaderName, HeaderValue};
use hyper::{Request, Response, StatusCode};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

use crate::http_server::{self, Body, HttpResponse};
use crate::run::file::{Rewrite, ServeSpec};
use crate::terminal::Launch;
use files::{Resolution, Site};
use hook::{Hook, HookAnswer, HookRequest};
use live::{EVENTS_PATH, LIVE_JS, SCRIPT_PATH, Served};

/// The environment variable `ive --serve` reads its setup from.
pub const SETUP_VAR: &str = "IVE_SERVE";
/// Changes closer together than this make one reload.
const DEBOUNCE: Duration = Duration::from_millis(100);
/// Often enough for proxies that drop a stream idle for 30 s or more.
const KEEP_ALIVE: Duration = Duration::from_secs(15);

/// What a static server serves and how, its values filled in by the IVE that starts it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServeSetup {
    pub root: PathBuf,
    pub port: u16,
    pub live_reload: bool,
    pub fallback: Option<String>,
    pub rewrites: Vec<Rewrite>,
    /// the Rune hook, absolute
    pub script: Option<PathBuf>,
}

/// The link to a running server's page `open`.
pub fn address(port: u16, open: &str) -> String {
    format!("http://127.0.0.1:{port}{open}")
}

/// The port of a link [`address`] made.
pub fn port_of(address: &str) -> Option<u16> {
    let host = address.split("://").nth(1)?.split('/').next()?;
    host.rsplit_once(':')?.1.parse().ok()
}

/// Whether a server could listen on `port` now.
pub fn is_free(port: u16) -> bool {
    bindable(port).is_ok()
}

/// The port to serve on: `hint`, the `.run.toml`'s port, while it is free, else one the OS picks.
pub fn free_port(hint: u16) -> Result<u16, String> {
    if is_free(hint) {
        return Ok(hint);
    }
    bindable(0).map_err(|error| format!("cannot find a free port (port {hint} is taken): {error}"))
}

/// The port a listener could bind on `127.0.0.1:port`. Address reuse matches the server's own listener, so a
/// port left in TIME_WAIT by a stopped run counts as free. On Windows the listener sets none, and the socket option
/// would let the probe bind a port another program listens on.
fn bindable(port: u16) -> std::io::Result<u16> {
    let socket = tokio::net::TcpSocket::new_v4()?;
    #[cfg(unix)]
    socket.set_reuseaddr(true)?;
    socket.bind(std::net::SocketAddr::from(([127, 0, 0, 1], port)))?;
    Ok(socket.local_addr()?.port())
}

/// The program serving `root` on `port` as `spec` says; `folder` holds the `.run.toml`, which `script` is relative to.
pub fn launch(root: &Path, port: u16, spec: &ServeSpec, folder: &Path) -> Result<Launch, String> {
    let setup = ServeSetup {
        root: root.to_path_buf(),
        port,
        live_reload: spec.live_reload,
        fallback: spec.fallback.clone(),
        rewrites: spec.rewrites.clone(),
        script: spec.script.as_ref().map(|script| folder.join(script)),
    };
    let json = serde_json::to_string(&setup).map_err(|error| error.to_string())?;
    Ok(Launch { program: vec![own_program()?, "--serve".to_owned()], env: vec![(SETUP_VAR.to_owned(), json)] })
}

/// IVE's running image: `/proc/self/exe` still names it after a rebuild replaced its file, which `current_exe`
/// then reports as deleted. It is read by whoever execs it: IVE, or a pty host IVE started from the same image.
#[cfg(target_os = "linux")]
fn own_program() -> Result<String, String> {
    Ok("/proc/self/exe".to_owned())
}

#[cfg(target_os = "macos")]
fn own_program() -> Result<String, String> {
    std::env::current_exe().map(|exe| exe.display().to_string()).map_err(|error| format!("cannot find IVE's own program: {error}"))
}

/// `ive-cli.exe` next to `ive.exe`: the GUI program has no console to print the server's output into.
#[cfg(windows)]
fn own_program() -> Result<String, String> {
    std::env::current_exe()
        .map(|exe| exe.with_file_name("ive-cli.exe").display().to_string())
        .map_err(|error| format!("cannot find IVE's own program: {error}"))
}

/// `ive --serve`: serves until stopped; the exit code says why it could not.
pub fn main() -> i32 {
    let setup = std::env::var(SETUP_VAR)
        .map_err(|_| format!("{SETUP_VAR} is not set; IVE sets it when it starts a static server"))
        .and_then(|json| serde_json::from_str::<ServeSetup>(&json).map_err(|error| format!("{SETUP_VAR}: {error}")));
    match setup.and_then(run) {
        Ok(()) => 0,
        Err(error) => {
            eprintln!("{error}");
            1
        }
    }
}

/// A request for the hook, and where its answer goes.
type HookCall = (HookRequest, oneshot::Sender<Result<HookAnswer, String>>);

/// What every request handler shares.
struct Server {
    site: Site,
    /// with live reload only
    served: Option<Arc<Served>>,
    changes: broadcast::Sender<&'static str>,
    hook: Option<mpsc::Sender<HookCall>>,
}

fn run(setup: ServeSetup) -> Result<(), String> {
    let site = Site::new(&setup)?;
    let hook = match &setup.script {
        Some(script) => Some(start_hook(script)?),
        None => None,
    };
    let (changes, _) = broadcast::channel(16);
    let served = match setup.live_reload {
        true => Some(watch(site.root.clone(), changes.clone())?),
        false => None,
    };
    let root = site.root.display().to_string();
    let server = Arc::new(Server { site, served, changes, hook });
    let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build().map_err(|error| error.to_string())?;
    runtime.block_on(async move {
        let listener = match tokio::net::TcpListener::bind(("127.0.0.1", setup.port)).await {
            Ok(listener) => listener,
            Err(error) if error.kind() == std::io::ErrorKind::AddrInUse => return Err(format!("port {} is in use", setup.port)),
            Err(error) => return Err(format!("cannot listen on port {}: {error}", setup.port)),
        };
        println!("Serving {root} at http://127.0.0.1:{}/", setup.port);
        http_server::serve(listener, "ive: static server", move |request| routed(server.clone(), request)).await;
        Ok(())
    })
}

async fn routed(server: Arc<Server>, request: Request<Incoming>) -> HttpResponse {
    match request.uri().path() {
        SCRIPT_PATH => http_server::respond(StatusCode::OK, &[(header::CONTENT_TYPE, "text/javascript")], LIVE_JS),
        EVENTS_PATH => events(&server),
        _ => handle(&server, request).await,
    }
}

/// Compiles the hook on a thread of its own, which then answers one request at a time.
fn start_hook(script: &Path) -> Result<mpsc::Sender<HookCall>, String> {
    let (requests, incoming) = mpsc::channel::<HookCall>();
    let (loaded, load_result) = mpsc::channel();
    let script = script.to_path_buf();
    std::thread::spawn(move || {
        let mut hook = match Hook::load(&script) {
            Ok(hook) => {
                let _ = loaded.send(Ok(()));
                hook
            }
            Err(error) => {
                let _ = loaded.send(Err(error));
                return;
            }
        };
        for (request, answer) in incoming {
            let _ = answer.send(hook.request(&request));
        }
    });
    load_result.recv().map_err(|_| "the hook's thread ended".to_owned())??;
    Ok(requests)
}

/// Sends one change per burst of changes to served files, [`DEBOUNCE`] apart, logging which files they were.
fn watch(root: PathBuf, changes: broadcast::Sender<&'static str>) -> Result<Arc<Served>, String> {
    let (served, received) = Served::start()?;
    let served = Arc::new(served);
    let watched = served.clone();
    std::thread::spawn(move || {
        while let Some(paths) = live::next_burst(&received, DEBOUNCE, |path| !watched.contains(path)) {
            println!("{} {}", clock(), live::describe(&paths, &root));
            let _ = changes.send(live::change_of(&paths).data());
        }
    });
    Ok(served)
}

/// Server-sent events: `data: <change>` per change, a comment every [`KEEP_ALIVE`] so idle proxies keep the stream.
fn events(server: &Server) -> HttpResponse {
    let received = server.changes.subscribe();
    let ticks = tokio::time::interval_at(tokio::time::Instant::now() + KEEP_ALIVE, KEEP_ALIVE);
    let stream = futures::stream::unfold((received, ticks), |(mut received, mut ticks)| async move {
        let event = next_event(&mut received, &mut ticks).await?;
        Some((Ok::<_, Infallible>(Frame::data(Bytes::from(event))), (received, ticks)))
    });
    let mut response = Response::new(Body::new(StreamBody::new(stream)));
    response.headers_mut().insert(header::CONTENT_TYPE, HeaderValue::from_static("text/event-stream"));
    response.headers_mut().insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    response
}

/// The next change as an event, or a keep-alive comment; `None` once no more changes can come.
async fn next_event(received: &mut broadcast::Receiver<&'static str>, ticks: &mut tokio::time::Interval) -> Option<String> {
    loop {
        tokio::select! {
            change = received.recv() => match change {
                Ok(change) => return Some(format!("data: {change}\n\n")),
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(broadcast::error::RecvError::Closed) => return None,
            },
            _ = ticks.tick() => return Some(":\n\n".to_owned()),
        }
    }
}

/// The hook, then the file; one log line per request.
async fn handle(server: &Server, request: Request<Incoming>) -> HttpResponse {
    let started = Instant::now();
    let headers = request.headers().iter().map(|(name, value)| (name.to_string(), value.to_str().unwrap_or_default().to_owned())).collect();
    let asked = HookRequest {
        method: request.method().to_string(),
        path: request.uri().path().to_owned(),
        query: request.uri().query().unwrap_or_default().to_owned(),
        headers,
    };
    let (method, path) = (asked.method.clone(), asked.path.clone());
    let response = answer(server, asked).await;
    println!("{} {method} {path} {} {} ms", clock(), response.status().as_u16(), started.elapsed().as_millis());
    response
}

async fn answer(server: &Server, asked: HookRequest) -> HttpResponse {
    let (method, mut path) = (asked.method.clone(), asked.path.clone());
    if let Some(hook) = &server.hook {
        let (reply, answer) = oneshot::channel();
        let answer = match hook.send((asked, reply)) {
            Ok(()) => answer.await.unwrap_or_else(|_| Err("the hook's thread ended".to_owned())),
            Err(_) => Err("the hook's thread ended".to_owned()),
        };
        match answer {
            Ok(HookAnswer::Continue) => {}
            Ok(HookAnswer::Rewrite(rewritten)) => path = rewritten,
            Ok(HookAnswer::Respond { status, headers, body }) => return hook_response(status, headers, body),
            Err(error) => {
                println!("{error}");
                return http_server::text(StatusCode::INTERNAL_SERVER_ERROR, "the server's hook failed");
            }
        }
    }
    match server.site.resolve(&method, &path) {
        Resolution::File(file) => file_response(&file, server.served.as_deref()).await,
        Resolution::NotFound => not_found(),
    }
}

fn hook_response(status: u16, headers: Vec<(String, String)>, body: String) -> HttpResponse {
    let mut response = Response::new(http_server::full(body));
    *response.status_mut() = StatusCode::from_u16(status).unwrap_or(StatusCode::OK);
    for (name, value) in headers {
        if let (Ok(name), Ok(value)) = (HeaderName::try_from(name), HeaderValue::try_from(value)) {
            response.headers_mut().insert(name, value);
        }
    }
    response
}

/// `file`, with the live-reload script in a page while `served` watches what it sends.
async fn file_response(file: &Path, served: Option<&Served>) -> HttpResponse {
    let Ok(bytes) = tokio::fs::read(file).await else {
        return not_found();
    };
    if let Some(Err(error)) = served.map(|served| served.add(file)) {
        println!("{error}");
    }
    let kind = content_type(file);
    let body = match (served.is_some() && kind.starts_with("text/html"), String::from_utf8(bytes)) {
        (true, Ok(html)) => live::inject(&html).into_bytes(),
        (_, Ok(text)) => text.into_bytes(),
        (_, Err(error)) => error.into_bytes(),
    };
    http_server::respond(StatusCode::OK, &[(header::CONTENT_TYPE, kind), (header::CACHE_CONTROL, "no-store")], body)
}

fn not_found() -> HttpResponse {
    let headers = [(header::CONTENT_TYPE, "text/plain; charset=utf-8"), (header::CACHE_CONTROL, "no-store")];
    http_server::respond(StatusCode::NOT_FOUND, &headers, "Not found")
}

/// By extension, as browsers need for pages, styles, scripts, images and fonts.
fn content_type(file: &Path) -> &'static str {
    let extension = file.extension().map(|extension| extension.to_string_lossy().to_ascii_lowercase()).unwrap_or_default();
    match extension.as_str() {
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "json" | "map" => "application/json",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "wasm" => "application/wasm",
        "txt" | "md" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

/// The local time as `HH:MM:SS`, for log lines.
fn clock() -> String {
    let now = crate::age::Clock::system().when();
    let seconds = now.seconds.saturating_add(i64::from(now.offset_minutes) * 60).rem_euclid(24 * 60 * 60);
    format!("{:02}:{:02}:{:02}", seconds / 3600, seconds % 3600 / 60, seconds % 60)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "linux")]
    #[test]
    fn should_start_the_server_from_ives_running_image() {
        // given
        let spec = ServeSpec { root: ".".to_owned(), port: "4200".to_owned(), live_reload: true, fallback: None, rewrites: Vec::new(), open: "/".to_owned(), script: Some("serve.rn".to_owned()) };

        // when
        let launched = launch(Path::new("/r/site"), 4200, &spec, Path::new("/r/misc")).unwrap();

        // then
        assert_eq!(launched.program, ["/proc/self/exe", "--serve"]);
        let setup: ServeSetup = serde_json::from_str(&launched.env[0].1).unwrap();
        assert_eq!(setup.script, Some(PathBuf::from("/r/misc/serve.rn")));
    }

    #[test]
    fn should_serve_on_the_hinted_port_while_it_is_free() {
        // given
        let hint = bindable(0).unwrap();

        // when
        let port = free_port(hint).unwrap();

        // then
        assert_eq!(port, hint);
    }

    #[test]
    fn should_serve_on_another_free_port_when_the_hinted_one_is_taken() {
        // given
        let taken = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let hint = taken.local_addr().unwrap().port();

        // when
        let port = free_port(hint).unwrap();

        // then
        assert_ne!(port, hint);
        assert!(std::net::TcpListener::bind(("127.0.0.1", port)).is_ok(), "port {port} is free");
    }

    #[test]
    fn should_read_the_port_back_from_a_link() {
        // given
        let url = address(4304, "/docs/");

        // when
        let port = port_of(&url);

        // then
        assert_eq!(port, Some(4304));
    }

    #[test]
    fn should_link_to_the_page_to_open_on_the_local_port() {
        // given
        let open = "/docs/design/mock/home.html";

        // when
        let url = address(4304, open);

        // then
        assert_eq!(url, "http://127.0.0.1:4304/docs/design/mock/home.html");
    }
}
