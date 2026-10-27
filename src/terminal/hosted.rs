//! Terminals whose shell runs in a detached `ptyhost` host, so they outlive IVE: owning the registry of sessions,
//! starting hosts, and a connection that feeds a host's output into IVE's terminal state.

use std::io;
use std::path::Path;
use std::process::ExitStatus;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock, mpsc};
use std::time::{Duration, Instant};

use alacritty_terminal::event::{Event, EventListener};
use alacritty_terminal::sync::FairMutex;
use alacritty_terminal::term::Term;
use alacritty_terminal::vte::ansi::{Processor, StdSyncHandler};
use ptyhost::client::Client;
use ptyhost::host::{self, HostConfig, Launcher};
use ptyhost::protocol::{Frame, Size};
use ptyhost::registry::{Owner, Registry, SessionInfo};

use super::attention::{Call, CallReport, CallScanner, finished};
use super::directory::{DirectoryReport, DirectoryScanner, host_name};
use super::foreground::ForegroundProbe;
use super::launch::Launch;
use super::progress::{Progress, ProgressScanner};
use super::session::{GridSize, Listener};
use super::theme_reports::ThemeReportScanner;
use crate::profile::Profile;
use crate::retry::{RESTART_ATTEMPTS, RESTART_INTERVAL, retry};

/// How long `hang_up` waits for its frame to be written: long enough for a host that reads, short enough that one
/// that stopped reading cannot freeze IVE.
const HANG_UP_WAIT: Duration = Duration::from_secs(1);

/// Input goes to the host in frames of at most this, so a paste of any size stays under the frame limit.
const INPUT_CHUNK: usize = 64 * 1024;

/// The registry of hosted sessions, owned by this IVE: it starts hosts and adopts what earlier runs left.
pub struct HostedTerminals {
    registry: Registry,
    launcher: Launcher,
    _owner: Owner,
}

impl HostedTerminals {
    /// The profile's registry (or `$IVE_PTY_DIR`), unless another IVE owns it; the reason goes to stderr then, and
    /// terminals end with IVE as before.
    pub fn claim(profile: &Profile) -> Option<Self> {
        let registry = Registry::for_profile(profile.name());
        let dir = registry.dir().to_path_buf();
        match Self::with(registry, crate::pty_host_launcher()) {
            Ok(Some(terminals)) => Some(terminals),
            Ok(None) => {
                crate::notices::log_only(&format!("another IVE keeps the terminals in {}; terminals here end when IVE closes", dir.display()));
                None
            }
            Err(error) => {
                crate::notices::log_only(&format!("cannot keep terminals in {}: {error}; terminals end when IVE closes", dir.display()));
                None
            }
        }
    }

    /// As [`claim`](Self::claim), waiting up to 10 s while another IVE still owns the registry: the one this IVE
    /// replaces on a restart, which lets go of it as it quits.
    pub fn claim_after_restart(profile: &Profile) -> Option<Self> {
        let registry = || Registry::for_profile(profile.name());
        let owned = retry(RESTART_ATTEMPTS, RESTART_INTERVAL, || match Self::with(registry(), crate::pty_host_launcher()) {
            Ok(None) => Err(()),
            other => Ok(other),
        });
        match owned {
            Ok(Ok(Some(terminals))) => Some(terminals),
            _ => Self::claim(profile),
        }
    }

    pub fn with(registry: Registry, launcher: Launcher) -> io::Result<Option<Self>> {
        Ok(registry.try_own()?.map(|owner| Self { registry, launcher, _owner: owner }))
    }

    /// Starts `launch` (the user's shell when it names no program) in a new host.
    pub fn start(&self, cwd: &Path, launch: &Launch, labels: Vec<(String, String)>) -> io::Result<SessionInfo> {
        let command = if launch.program.is_empty() { shell_command() } else { launch.program.clone() };
        let config = HostConfig {
            id: new_id()?,
            command,
            cwd: cwd.to_path_buf(),
            size: Size::DEFAULT,
            env: [super::env::variables(), launch.env.clone()].concat(),
            labels,
        };
        host::start(&self.launcher, &self.registry, &config)?;
        self.registry
            .session(&config.id)?
            .ok_or_else(|| io::Error::other(format!("terminal {} ended right after it started", config.id)))
    }

    /// Sessions running now, oldest first; an unreadable registry counts as none, reported to stderr.
    pub fn live(&self) -> Vec<SessionInfo> {
        self.registry.sessions().unwrap_or_else(|error| {
            crate::notices::log_only(&format!("cannot list running terminals in {}: {error}", self.registry.dir().display()));
            Vec::new()
        })
    }

    /// Attaches to session `id`, feeding its output into `term`.
    pub fn connect(&self, id: &str, term: Arc<FairMutex<Term<Listener>>>, listener: Listener) -> io::Result<Connection> {
        let client = Client::connect(&self.registry, id)?;
        Ok(Connection::start(id.to_owned(), client, Some(self.registry.clone()), term, listener))
    }
}

/// An attached session. Output arrives on a reader thread; input and resizes go out from the caller's thread.
/// Dropping it detaches: the program keeps running.
pub struct Connection {
    id: String,
    program_pid: u32,
    shared: Arc<Shared>,
    /// frames for the writer thread: the caller (IVE's UI thread) never waits on the socket
    outbox: mpsc::Sender<Outgoing>,
}

/// A frame for the writer thread, and for a hang-up, whom to tell once it is written.
struct Outgoing {
    frame: Frame,
    written: Option<mpsc::Sender<()>>,
}

/// What the caller's side and the reader thread both use.
struct Shared {
    writer: Mutex<Writer>,
    /// the last size sent, sent again after a re-attach
    size: Mutex<Option<Size>>,
    /// why the connection broke for good, once it has
    lost: OnceLock<String>,
    /// set by a hang-up: input still queued is dropped, so the hang-up goes out next
    hanging_up: AtomicBool,
    /// the program's last progress report
    progress: Mutex<Progress>,
    /// the program's last call for the user's attention, in live output
    call: Mutex<Option<CallReport>>,
    /// the program's last working-directory report
    directory: Mutex<Option<DirectoryReport>>,
    /// whether the program asked to hear when the theme turns dark or light
    theme_reports: AtomicBool,
}

/// The client input goes to, and whether the tab let go of it. One lock covers both, so a re-attach either lands
/// before the detach (which then closes it) or sees the detach and closes its new client itself.
struct Writer {
    client: Client,
    detached: bool,
    /// counts re-attaches, so the writer thread knows when its copy of the client is stale
    generation: u64,
}

impl Connection {
    /// Starts the reader; with `registry`, a broken connection is re-attached once before it counts as lost.
    pub fn start(
        id: String,
        client: Client,
        registry: Option<Registry>,
        term: Arc<FairMutex<Term<Listener>>>,
        listener: Listener,
    ) -> Self {
        let program_pid = client.program_pid();
        let reader = client.try_clone();
        let shared =
            Arc::new(Shared {
                writer: Mutex::new(Writer { client, detached: false, generation: 0 }),
                size: Mutex::new(None),
                lost: OnceLock::new(),
                hanging_up: AtomicBool::new(false),
                progress: Mutex::new(Progress::None),
                call: Mutex::new(None),
                directory: Mutex::new(None),
                theme_reports: AtomicBool::new(false),
            });
        let (outbox, frames) = mpsc::channel();
        let writer = shared.clone();
        std::thread::Builder::new()
            .name(format!("ive-terminal-input-{id}"))
            .spawn(move || write_frames(&writer, &frames))
            .expect("cannot start a terminal writer thread");
        let foreground = ForegroundProbe::of_program(program_pid);
        let output = Output { id: id.clone(), registry, shared: shared.clone(), term, listener, foreground };
        match reader {
            Ok(reader) => {
                std::thread::Builder::new()
                    .name(format!("ive-terminal-{id}"))
                    .spawn(move || output.run(reader))
                    .expect("cannot start a terminal reader thread");
            }
            Err(error) => output.end_lost(format!("cannot read from the terminal host: {error}")),
        }
        Self { id, program_pid, shared, outbox }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn program_pid(&self) -> u32 {
        self.program_pid
    }

    /// Why the connection broke and could not be re-attached, once it has; the program may still run.
    pub fn lost_reason(&self) -> Option<String> {
        self.shared.lost.get().cloned()
    }

    pub fn progress(&self) -> Progress {
        *self.shared.progress.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn call(&self) -> Option<CallReport> {
        self.shared.call.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).clone()
    }

    pub fn directory(&self) -> Option<DirectoryReport> {
        self.shared.directory.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).clone()
    }

    pub fn wants_theme_reports(&self) -> bool {
        self.shared.theme_reports.load(Ordering::Relaxed)
    }

    pub fn input(&self, bytes: &[u8]) {
        if self.shared.hanging_up.load(Ordering::Relaxed) {
            return;
        }
        for chunk in bytes.chunks(INPUT_CHUNK) {
            self.queue(Frame::Input(chunk.to_vec()));
        }
    }

    /// Always sent, even unchanged: the host replays only after the first one, and each makes programs redraw.
    pub fn resize(&self, size: GridSize) {
        let size = Size { columns: size.columns, lines: size.lines };
        *self.shared.size.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(size);
        self.queue(Frame::Resize(size));
    }

    /// Ends the program as closing its terminal would, ahead of input still queued; the exit arrives as events.
    /// Returns once the hang-up is written, or after [`HANG_UP_WAIT`], since IVE may quit right after.
    pub fn hang_up(&self) {
        self.shared.hanging_up.store(true, Ordering::Relaxed);
        let (written, wait) = mpsc::channel();
        if self.outbox.send(Outgoing { frame: Frame::HangUp, written: Some(written) }).is_ok() {
            let _ = wait.recv_timeout(HANG_UP_WAIT);
        }
    }

    fn queue(&self, frame: Frame) {
        // fails only once the writer thread ended with the connection
        let _ = self.outbox.send(Outgoing { frame, written: None });
    }
}

/// The writer thread: sends queued frames to the current client, skipping input once a hang-up was asked for and
/// everything but a hang-up once detached. It writes on its own copy of the client, so a blocked write never holds
/// the lock the caller's side needs; closing the connection unblocks it. It ends when the connection is dropped,
/// closing the client a pending hang-up kept open.
fn write_frames(shared: &Shared, frames: &mpsc::Receiver<Outgoing>) {
    let mut current: Option<(u64, Client)> = None;
    for Outgoing { frame, written } in frames {
        if matches!(frame, Frame::Input(_)) && shared.hanging_up.load(Ordering::Relaxed) {
            continue;
        }
        let writer = shared.writer();
        if writer.detached && frame != Frame::HangUp {
            continue;
        }
        if current.as_ref().is_none_or(|(generation, _)| *generation != writer.generation) {
            current = writer.client.try_clone().ok().map(|client| (writer.generation, client));
        }
        drop(writer);
        if let Some((_, client)) = current.as_mut() {
            // fails only when the host is gone; the reader reports that
            let _ = client.send(&frame);
        }
        if let Some(written) = written {
            let _ = written.send(());
        }
    }
    if let Some((_, client)) = current {
        client.shutdown();
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        let mut writer = self.shared.writer();
        writer.detached = true;
        // a pending hang-up must still go out: the writer thread closes the connection once it has
        if !self.shared.hanging_up.load(Ordering::Relaxed) {
            writer.client.shutdown();
        }
    }
}

impl Shared {
    fn writer(&self) -> std::sync::MutexGuard<'_, Writer> {
        self.writer.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn detached(&self) -> bool {
        self.writer().detached
    }
}

/// The terminal's parser, and those for what it drops that IVE shows.
struct Parsers {
    terminal: Processor<StdSyncHandler>,
    progress: ProgressScanner,
    call: CallScanner,
    directory: DirectoryScanner,
    theme_reports: ThemeReportScanner,
}

impl Default for Parsers {
    fn default() -> Self {
        Self {
            terminal: Processor::default(),
            progress: ProgressScanner::default(),
            call: CallScanner::default(),
            directory: DirectoryScanner::new(host_name()),
            theme_reports: ThemeReportScanner::default(),
        }
    }
}

/// The reader thread's side of a connection.
struct Output {
    id: String,
    registry: Option<Registry>,
    shared: Arc<Shared>,
    term: Arc<FairMutex<Term<Listener>>>,
    listener: Listener,
    /// who leads the session's foreground, for tying a directory report to the program that made it
    foreground: ForegroundProbe,
}

/// Whether output is new, or replayed from before this connection.
#[derive(Clone, Copy)]
enum Origin {
    Live,
    Replayed,
}

impl Output {
    fn run(self, mut reader: Client) {
        let mut parser = Parsers::default();
        loop {
            match self.next_frame(&mut reader, &parser) {
                Ok(Some(frame)) => {
                    if self.apply(frame, &mut parser) {
                        return;
                    }
                }
                Ok(None) => self.flush_synchronized(&mut parser),
                Err(_) if self.shared.detached() => return,
                Err(error) => match self.reattach() {
                    Some(client) => reader = client,
                    None => return self.end_lost(format!("the connection to the terminal host broke ({error}) and attaching again failed")),
                },
            }
        }
    }

    /// The next frame, or `None` when a synchronized update timed out first.
    fn next_frame(&self, reader: &mut Client, parser: &Parsers) -> io::Result<Option<Frame>> {
        let timeout = parser.terminal.sync_timeout().sync_timeout().map(|deadline| deadline.saturating_duration_since(Instant::now()));
        if reader.wait_readable(timeout)? {
            return reader.receive().map(Some);
        }
        Ok(None)
    }

    /// Applies one frame; true when the program exited and the reader is done.
    fn apply(&self, frame: Frame, parser: &mut Parsers) -> bool {
        match frame {
            Frame::Output(bytes) => self.parse(parser, &bytes, Origin::Live),
            Frame::Replay(bytes) => {
                self.listener.mute_replies(true);
                self.parse(parser, &bytes, Origin::Replayed);
                self.listener.mute_replies(false);
            }
            Frame::Exited(code) => {
                self.listener.send_event(Event::ChildExit(exit_status(code)));
                self.listener.send_event(Event::Exit);
                return true;
            }
            Frame::Input(_) | Frame::Resize(_) | Frame::Attached { .. } | Frame::HangUp => {}
        }
        false
    }

    fn parse(&self, parser: &mut Parsers, bytes: &[u8], origin: Origin) {
        parser.terminal.advance(&mut *self.term.lock(), bytes);
        let call = parser.call.scan(bytes);
        // each report in turn: an end followed by a new start in the same read still ends
        let ends = parser.progress.scan_all(bytes).into_iter().filter(|progress| self.take_progress(*progress)).count();
        let ended = (ends > 0).then_some(Call::Finished);
        // a replay holds calls the user already saw
        if let (Some(call), Origin::Live) = (call.or(ended), origin) {
            self.keep_call(call);
        }
        // replayed output counts too: it restores what the program asked for before IVE re-attached
        if let Some(wanted) = parser.theme_reports.scan(bytes) {
            self.shared.theme_reports.store(wanted, Ordering::Relaxed);
        }
        if let Some(path) = parser.directory.scan(bytes) {
            let mut directory = self.shared.directory.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            let number = directory.as_ref().map_or(1, |last| last.number + 1);
            let leader = match origin {
                Origin::Live => Some(self.foreground.leader()),
                Origin::Replayed => None,
            };
            *directory = Some(DirectoryReport { number, path, leader });
        }
        self.listener.send_event(Event::Wakeup);
    }

    /// Keeps `progress` as the program's last report; whether it ended busy progress.
    fn take_progress(&self, progress: Progress) -> bool {
        let mut last = self.shared.progress.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let before = std::mem::replace(&mut *last, progress);
        finished(before, progress)
    }

    fn keep_call(&self, call: Call) {
        let mut last = self.shared.call.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let number = last.as_ref().map_or(1, |report| report.number + 1);
        *last = Some(CallReport { number, call });
    }

    /// Draws a synchronized update whose end never came in time, as alacritty does.
    fn flush_synchronized(&self, parser: &mut Parsers) {
        parser.terminal.stop_sync(&mut *self.term.lock());
        self.listener.send_event(Event::Wakeup);
    }

    /// One new connection to the same session, with the last size sent so the host replays; `None` when the host
    /// is gone or another client keeps taking it.
    fn reattach(&self) -> Option<Client> {
        let registry = self.registry.as_ref()?;
        let client = Client::connect(registry, &self.id).ok()?;
        let reader = client.try_clone().ok()?;
        let size = *self.shared.size.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut writer = self.shared.writer();
        if writer.detached {
            // the tab closed while this connected
            client.shutdown();
            return None;
        }
        writer.client = client;
        writer.generation += 1;
        if let Some(size) = size {
            let _ = writer.client.send(&Frame::Resize(size));
        }
        Some(reader)
    }

    fn end_lost(&self, reason: String) {
        let _ = self.shared.lost.set(reason);
        self.listener.send_event(Event::Exit);
    }
}

/// The user's shell as IVE's own terminals start it: on macOS a login shell, as alacritty's `login -flp` makes it.
#[cfg(unix)]
fn shell_command() -> Vec<String> {
    let shell = super::launch::login_shell();
    if cfg!(target_os = "macos") {
        return vec![shell, "-l".to_owned()];
    }
    vec![shell]
}

#[cfg(windows)]
fn shell_command() -> Vec<String> {
    Launch::shell().program
}

/// The host reports 128 + signal for programs killed by a signal, as shells do.
#[cfg(unix)]
fn exit_status(code: i32) -> ExitStatus {
    use std::os::unix::process::ExitStatusExt;
    match code {
        129..=192 => ExitStatus::from_raw(code - 128),
        code => ExitStatus::from_raw((code & 0xff) << 8),
    }
}

/// The host reports the program's exit code as it is, an NTSTATUS such as Ctrl+C's included.
#[cfg(windows)]
fn exit_status(code: i32) -> ExitStatus {
    std::os::windows::process::ExitStatusExt::from_raw(code as u32)
}

fn new_id() -> io::Result<String> {
    let mut bytes = [0u8; 6];
    getrandom::fill(&mut bytes).map_err(|error| io::Error::other(format!("no randomness for a terminal id: {error}")))?;
    Ok(format!("t{}", bytes.iter().map(|byte| format!("{byte:02x}")).collect::<String>()))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    use alacritty_terminal::event::Event;
    use alacritty_terminal::sync::FairMutex;
    use alacritty_terminal::term::{Config, Term};
    use futures::channel::mpsc::{UnboundedReceiver, unbounded};
    use ptyhost::client::Client;
    use ptyhost::net::{UnixListener, UnixStream};
    use ptyhost::protocol::{Frame, PROTOCOL, Size};
    use ptyhost::registry::Registry;

    use super::*;
    use crate::terminal::session::{GridSize, Listener};
    use crate::terminal::text::screen_text;

    const TIMEOUT: Duration = Duration::from_secs(3);

    struct Fixture {
        connection: Connection,
        host: UnixStream,
        term: Arc<FairMutex<Term<Listener>>>,
        events: UnboundedReceiver<Event>,
    }

    fn fixture() -> Fixture {
        let (client_end, mut host) = UnixStream::pair().unwrap();
        Frame::Attached { protocol: PROTOCOL, program_pid: 7 }.write_to(&mut host).unwrap();
        let client = Client::from_stream(client_end).unwrap();
        let (sender, events) = unbounded();
        let listener = Listener::new(sender);
        let size = GridSize { columns: 40, lines: 5, cell_width: 8, cell_height: 16 };
        let term = Arc::new(FairMutex::new(Term::new(Config::default(), &size, listener.clone())));
        let connection = Connection::start("t1".to_owned(), client, None, term.clone(), listener);
        Fixture { connection, host, term, events }
    }

    impl Fixture {
        fn send(&mut self, frame: Frame) {
            frame.write_to(&mut self.host).unwrap();
        }

        fn first_line(&self) -> String {
            screen_text(&*self.term.lock()).lines[0].clone()
        }

        /// Waits until the first screen line reads `expected`.
        fn wait_for_line(&self, expected: &str) {
            let deadline = Instant::now() + TIMEOUT;
            while self.first_line() != expected {
                assert!(Instant::now() < deadline, "first line {:?}, expected {expected:?}", self.first_line());
                std::thread::sleep(Duration::from_millis(5));
            }
        }

        /// Events received until one matches `last`, that one included; panics after the timeout.
        fn events_until(&mut self, last: impl Fn(&Event) -> bool) -> Vec<Event> {
            let deadline = Instant::now() + TIMEOUT;
            let mut seen = Vec::new();
            loop {
                match self.events.try_recv() {
                    Ok(event) => {
                        let done = last(&event);
                        seen.push(event);
                        if done {
                            return seen;
                        }
                    }
                    Err(_) => {
                        assert!(Instant::now() < deadline, "no matching event among {seen:?}");
                        std::thread::sleep(Duration::from_millis(5));
                    }
                }
            }
        }

        fn drain_events(&mut self) -> Vec<Event> {
            std::iter::from_fn(|| self.events.try_recv().ok()).collect()
        }
    }

    #[test]
    fn should_feed_output_into_term() {
        // given
        let mut fixture = fixture();

        // when
        fixture.send(Frame::Replay(b"hello".to_vec()));
        fixture.send(Frame::Output(b" world".to_vec()));

        // then
        fixture.wait_for_line("hello world");
        fixture.events_until(|event| matches!(event, Event::Wakeup));
    }

    #[test]
    fn should_not_answer_queries_in_the_replay() {
        // given
        let mut fixture = fixture();

        // when
        fixture.send(Frame::Replay(b"\x1b[c\x1b[6nreplayed".to_vec()));
        fixture.wait_for_line("replayed");
        let after_replay = fixture.drain_events();
        fixture.send(Frame::Output(b"\x1b[c".to_vec()));

        // then
        assert!(!after_replay.iter().any(|event| matches!(event, Event::PtyWrite(_))), "{after_replay:?}");
        fixture.events_until(|event| matches!(event, Event::PtyWrite(_)));
    }

    /// An `OSC 7` report of folder `name` and the path it names on this OS.
    fn report_of(name: &str) -> (Vec<u8>, std::path::PathBuf) {
        let (url_path, path) = if cfg!(windows) { (format!("/C:/tmp/{name}"), format!("C:\\tmp\\{name}")) } else { (format!("/tmp/{name}"), format!("/tmp/{name}")) };
        (format!("\x1b]7;file://{url_path}\x07").into_bytes(), path.into())
    }

    #[test]
    fn should_keep_last_directory_report() {
        // given
        let mut fixture = fixture();
        let ((one, _), (two, two_path)) = (report_of("one"), report_of("two"));

        // when
        fixture.send(Frame::Output(one));
        fixture.send(Frame::Output([two, b"shown".to_vec()].concat()));
        fixture.wait_for_line("shown");

        // then
        let report = fixture.connection.directory().unwrap();
        assert_eq!((report.number, report.path, report.leader.is_some()), (2, two_path, true));
    }

    #[test]
    fn should_number_live_calls() {
        // given
        let mut fixture = fixture();

        // when
        fixture.send(Frame::Output(b"\x1b]9;4;3\x07\x1b]9;Waiting\x07".to_vec()));
        fixture.send(Frame::Output(b"\x1b]9;4;0\x07done".to_vec()));
        fixture.wait_for_line("done");

        // then
        assert_eq!(fixture.connection.call(), Some(CallReport { number: 2, call: Call::Finished }));
    }

    #[test]
    fn should_finish_when_progress_ends_and_starts_again_in_one_read() {
        // given
        let mut fixture = fixture();
        fixture.send(Frame::Output(b"\x1b]9;4;3\x07".to_vec()));

        // when
        fixture.send(Frame::Output(b"\x1b]9;4;0\x07\x1b]9;4;1;5\x07next".to_vec()));
        fixture.wait_for_line("next");

        // then
        assert_eq!((fixture.connection.call(), fixture.connection.progress()), (Some(CallReport { number: 1, call: Call::Finished }), Progress::Normal(5)));
    }

    #[test]
    fn should_take_no_call_from_the_replay() {
        // given
        let mut fixture = fixture();

        // when
        fixture.send(Frame::Replay(b"\x1b]9;Old\x07\x07\x1b]9;4;1;50\x07\x1b]9;4;0\x07\x1b]9;4;3\x07replayed".to_vec()));
        fixture.wait_for_line("replayed");
        let replayed = fixture.connection.call();
        fixture.send(Frame::Output(b"\x1b]9;4;0\x07\r\x1b[2Klive".to_vec()));
        fixture.wait_for_line("live");

        // then
        assert_eq!((replayed, fixture.connection.progress()), (None, Progress::None));
        assert_eq!(fixture.connection.call(), Some(CallReport { number: 1, call: Call::Finished }));
    }

    #[test]
    fn should_keep_replayed_directory_report_without_its_leader() {
        // given
        let mut fixture = fixture();
        let (old, old_path) = report_of("old");

        // when
        fixture.send(Frame::Replay([old, b"replayed".to_vec()].concat()));
        fixture.wait_for_line("replayed");

        // then
        let report = fixture.connection.directory().unwrap();
        assert_eq!((report.path, report.leader), (old_path, None));
    }

    #[test]
    fn should_report_exit_status() {
        // given
        let mut fixture = fixture();

        // when
        fixture.send(Frame::Exited(3));

        // then
        let events = fixture.events_until(|event| matches!(event, Event::Exit));
        let status = events.iter().find_map(|event| match event {
            Event::ChildExit(status) => Some(*status),
            _ => None,
        });
        assert_eq!(status.and_then(|status| status.code()), Some(3));
        assert_eq!(fixture.connection.lost_reason(), None);
    }

    #[test]
    fn should_say_why_when_host_disappears() {
        // given
        let mut fixture = fixture();

        // when
        fixture.host.shutdown(std::net::Shutdown::Both).unwrap();

        // then
        fixture.events_until(|event| matches!(event, Event::Exit));
        let reason = fixture.connection.lost_reason().expect("lost");
        assert!(!reason.is_empty(), "{reason}");
    }

    #[test]
    fn should_hang_up_when_asked() {
        // given
        let mut fixture = fixture();

        // when
        fixture.connection.hang_up();

        // then
        assert_eq!(Frame::read_from(&mut fixture.host).unwrap(), Frame::HangUp);
    }

    #[test]
    fn should_deliver_hang_up_asked_right_before_drop() {
        // given
        let Fixture { connection, mut host, .. } = fixture();

        // when
        connection.hang_up();
        drop(connection);

        // then
        assert_eq!(Frame::read_from(&mut host).unwrap(), Frame::HangUp);
    }

    #[cfg(unix)]
    #[test]
    fn should_have_sent_hang_up_when_hang_up_returns() {
        for attempt in 0..200 {
            // given
            let fixture = fixture();

            // when
            fixture.connection.hang_up();

            // then
            // IVE may quit right after, ending the writer thread: the frame must be in the socket already
            let mut fds = [libc::pollfd { fd: std::os::fd::AsRawFd::as_raw_fd(&fixture.host), events: libc::POLLIN, revents: 0 }];
            // SAFETY: polls one descriptor the fixture owns, without waiting
            unsafe { libc::poll(fds.as_mut_ptr(), 1, 0) };
            assert_ne!(fds[0].revents, 0, "attempt {attempt}: hang_up() returned before the hang-up was sent");
        }
    }

    #[test]
    fn should_detach_without_hanging_up_on_drop() {
        // given
        let Fixture { connection, mut host, .. } = fixture();

        // when
        drop(connection);

        // then
        assert_eq!(Frame::read_from(&mut host).unwrap_err().kind(), std::io::ErrorKind::UnexpectedEof);
    }

    #[test]
    fn should_send_resize_even_when_unchanged() {
        // given
        let mut fixture = fixture();
        let size = GridSize { columns: 40, lines: 5, cell_width: 8, cell_height: 16 };

        // when
        fixture.connection.resize(size);
        fixture.connection.resize(size);

        // then
        let expected = Frame::Resize(Size { columns: 40, lines: 5 });
        assert_eq!(Frame::read_from(&mut fixture.host).unwrap(), expected);
        assert_eq!(Frame::read_from(&mut fixture.host).unwrap(), expected);
    }

    #[test]
    fn should_send_large_input_in_bounded_frames() {
        // given
        let mut fixture = fixture();
        let paste = vec![b'x'; 200_000];

        // when
        std::thread::scope(|scope| {
            scope.spawn(|| fixture.connection.input(&paste));
            let mut received = Vec::new();
            while received.len() < paste.len() {
                match Frame::read_from(&mut fixture.host).unwrap() {
                    // then
                    Frame::Input(bytes) => {
                        assert!(bytes.len() <= INPUT_CHUNK, "a frame of {} bytes", bytes.len());
                        received.extend(bytes);
                    }
                    other => panic!("expected input, got {other:?}"),
                }
            }
            assert_eq!(received, paste);
        });
    }

    #[test]
    fn should_not_keep_a_connection_reattached_after_its_tab_closed() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let registry = Registry::at(dir.path());
        let host = UnixListener::bind(registry.socket("t1")).unwrap();
        let client_end = UnixStream::connect(registry.socket("t1")).unwrap();
        let (mut first, _) = host.accept().unwrap();
        Frame::Attached { protocol: PROTOCOL, program_pid: 7 }.write_to(&mut first).unwrap();
        let (sender, _events) = unbounded();
        let listener = Listener::new(sender);
        let size = GridSize { columns: 40, lines: 5, cell_width: 8, cell_height: 16 };
        let term = Arc::new(FairMutex::new(Term::new(Config::default(), &size, listener.clone())));
        let connection = Connection::start("t1".to_owned(), Client::from_stream(client_end).unwrap(), Some(registry), term, listener);
        // the connection breaks; the reader re-attaches and waits for the new handshake
        drop(first);
        let (mut second, _) = host.accept().unwrap();

        // when
        drop(connection);
        Frame::Attached { protocol: PROTOCOL, program_pid: 7 }.write_to(&mut second).unwrap();

        // then
        second.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
        let mut byte = [0u8; 1];
        let read = std::io::Read::read(&mut second, &mut byte);
        assert!(matches!(read, Ok(0)), "the re-attached connection is still open: {read:?}");
    }

    #[test]
    fn should_not_block_the_caller_while_the_host_does_not_read() {
        // given
        let Fixture { connection, host, .. } = fixture();
        let (returned, wait) = std::sync::mpsc::channel();
        let paste = vec![b'x'; 1_000_000];

        // when
        let typing = std::thread::spawn(move || {
            connection.input(&paste);
            let _ = returned.send(());
            connection
        });

        // then
        let in_time = wait.recv_timeout(Duration::from_millis(500)).is_ok();
        drop(host);
        drop(typing.join());
        assert!(in_time, "input() blocked while the host was not reading");
    }

    #[test]
    fn should_send_hang_up_ahead_of_queued_input() {
        // given
        let mut fixture = fixture();
        fixture.connection.input(&vec![b'x'; 1_000_000]);

        // when
        fixture.connection.hang_up();
        let mut input = 0;
        let hang_up = loop {
            match Frame::read_from(&mut fixture.host).unwrap() {
                Frame::Input(bytes) => input += bytes.len(),
                other => break other,
            }
        };

        // then
        assert_eq!(hang_up, Frame::HangUp);
        assert!(input < 1_000_000, "all {input} bytes of input went before the hang-up");
    }

    #[test]
    fn should_flush_synchronized_update_after_timeout() {
        // given
        let mut fixture = fixture();

        // when
        fixture.send(Frame::Output(b"\x1b[?2026hpartial".to_vec()));

        // then
        fixture.wait_for_line("partial");
    }
}
