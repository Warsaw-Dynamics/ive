use std::borrow::Cow;
use std::cell::Cell;
use std::io;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use alacritty_terminal::event::{Event, EventListener, WindowSize};
use alacritty_terminal::event_loop::{EventLoop, EventLoopSender, Msg};
use alacritty_terminal::grid::{Dimensions, Scroll};
use alacritty_terminal::index::{Point, Side};
use alacritty_terminal::selection::{Selection, SelectionType};
use alacritty_terminal::sync::FairMutex;
use alacritty_terminal::term::{Config, Term, TermMode};
use alacritty_terminal::tty;
use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender, unbounded};

#[cfg(unix)]
use super::foreground::ForegroundProbe;
use super::directory::DirectoryReport;
use super::launch::Launch;
use super::progress::Progress;
#[cfg(target_os = "linux")]
use super::hosted::{Connection, HostedTerminals};
#[cfg(all(test, target_os = "linux"))]
use ptyhost::client::Client;
#[cfg(target_os = "linux")]
use ptyhost::registry::SessionInfo;
#[cfg(all(test, target_os = "linux"))]
use ptyhost::registry::Registry;

const SCROLLBACK_LINES: usize = 10_000;
/// Grid used until the first layout reports the real one.
const INITIAL_SIZE: GridSize = GridSize { columns: 80, lines: 24, cell_width: 8, cell_height: 16 };

/// Forwards terminal events from the PTY thread to the UI.
#[derive(Clone)]
pub struct Listener {
    sender: UnboundedSender<Event>,
    muted: Arc<AtomicBool>,
}

impl Listener {
    pub fn new(sender: UnboundedSender<Event>) -> Self {
        Self { sender, muted: Arc::new(AtomicBool::new(false)) }
    }

    /// While muted, the terminal's answers to queries are dropped: output replayed from a host was answered when
    /// it first ran, and answering again would type the answers into the program.
    pub fn mute_replies(&self, muted: bool) {
        self.muted.store(muted, Ordering::Relaxed);
    }
}

impl EventListener for Listener {
    fn send_event(&self, event: Event) {
        let reply = matches!(event, Event::PtyWrite(_) | Event::ClipboardLoad(..) | Event::ColorRequest(..) | Event::TextAreaSizeRequest(_));
        if reply && self.muted.load(Ordering::Relaxed) {
            return;
        }
        // fails only once the terminal tab is gone, when nobody is left to tell
        let _ = self.sender.unbounded_send(event);
    }
}

/// Grid size in cells, plus the cell size in pixels, which programs can query.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridSize {
    pub columns: u16,
    pub lines: u16,
    pub cell_width: u16,
    pub cell_height: u16,
}

impl GridSize {
    pub fn window_size(self) -> WindowSize {
        WindowSize {
            num_lines: self.lines,
            num_cols: self.columns,
            cell_width: self.cell_width,
            cell_height: self.cell_height,
        }
    }
}

impl Dimensions for GridSize {
    fn total_lines(&self) -> usize {
        self.screen_lines()
    }

    fn screen_lines(&self) -> usize {
        usize::from(self.lines)
    }

    fn columns(&self) -> usize {
        usize::from(self.columns)
    }
}

/// A shell and the terminal state it draws into. The shell runs in a PTY of IVE's own, which dropping the session
/// hangs up, or in a host (Linux), which dropping the session leaves running; [`Session::hang_up`] ends either.
pub struct Session {
    term: Arc<FairMutex<Term<Listener>>>,
    transport: Transport,
    size: Cell<GridSize>,
    program_pid: Option<u32>,
    #[cfg(unix)]
    foreground: Option<ForegroundProbe>,
}

enum Transport {
    InProcess(EventLoopSender),
    #[cfg(target_os = "linux")]
    Hosted(Connection),
}

impl Session {
    /// Starts `launch` (the user's login shell when it names no program) in `working_directory`; its events arrive
    /// on the returned receiver.
    pub fn spawn(working_directory: &Path, launch: &Launch) -> io::Result<(Self, UnboundedReceiver<Event>)> {
        let (term, listener, events) = new_term();
        let shell = launch.program.split_first().map(|(program, args)| tty::Shell::new(program.clone(), args.to_vec()));
        let options = tty::Options {
            shell,
            working_directory: Some(working_directory.to_path_buf()),
            env: super::env::variables().into_iter().chain(launch.env.iter().cloned()).collect(),
            ..tty::Options::default()
        };
        let pty = tty::new(&options, INITIAL_SIZE.window_size(), 0)?;
        // taken before the event loop owns the PTY
        #[cfg(unix)]
        let foreground = ForegroundProbe::of(&pty);
        #[cfg(unix)]
        let program_pid = Some(pty.child().id());
        #[cfg(not(unix))]
        let program_pid = None;
        let event_loop = EventLoop::new(term.clone(), listener, pty, false, false)?;
        let transport = Transport::InProcess(event_loop.channel());
        event_loop.spawn();
        let session = Self {
            term,
            transport,
            size: Cell::new(INITIAL_SIZE),
            program_pid,
            #[cfg(unix)]
            foreground,
        };
        Ok((session, events))
    }

    /// Starts `launch` in a new host, labelled with `labels`.
    #[cfg(target_os = "linux")]
    pub fn spawn_hosted(
        terminals: &HostedTerminals,
        working_directory: &Path,
        launch: &Launch,
        labels: Vec<(String, String)>,
    ) -> io::Result<(Self, UnboundedReceiver<Event>)> {
        let session = terminals.start(working_directory, launch, labels)?;
        Self::attach(terminals, &session)
    }

    /// Attaches to a session already running in a host.
    #[cfg(target_os = "linux")]
    pub fn attach(terminals: &HostedTerminals, session: &SessionInfo) -> io::Result<(Self, UnboundedReceiver<Event>)> {
        let (term, listener, events) = new_term();
        let connection = terminals.connect(&session.id, term.clone(), listener)?;
        Ok((Self::hosted(term, connection), events))
    }

    /// A hosted session over an already attached client, for tests that play the host.
    #[cfg(all(test, target_os = "linux"))]
    pub fn over_client(id: String, client: Client, registry: Option<Registry>) -> (Self, UnboundedReceiver<Event>) {
        let (term, listener, events) = new_term();
        let connection = Connection::start(id, client, registry, term.clone(), listener);
        (Self::hosted(term, connection), events)
    }

    #[cfg(target_os = "linux")]
    fn hosted(term: Arc<FairMutex<Term<Listener>>>, connection: Connection) -> Self {
        let program_pid = Some(connection.program_pid());
        let foreground = Some(ForegroundProbe::of_program(connection.program_pid()));
        // the host replays only after a first resize; sent now, so a tab never shown still gets its output and title
        connection.resize(INITIAL_SIZE);
        Self { term, transport: Transport::Hosted(connection), size: Cell::new(INITIAL_SIZE), program_pid, foreground }
    }

    /// The program the terminal started, its process group's leader: the shell, or a run's command.
    pub fn program_pid(&self) -> Option<u32> {
        self.program_pid
    }

    /// Ends the shell, as closing its terminal does.
    pub fn hang_up(&self) {
        self.send(Msg::Shutdown);
    }

    /// The host session's id, for hosted sessions.
    pub fn hosted_id(&self) -> Option<&str> {
        match &self.transport {
            Transport::InProcess(_) => None,
            #[cfg(target_os = "linux")]
            Transport::Hosted(connection) => Some(connection.id()),
        }
    }

    /// True once a hosted session's connection broke for good; its program may still run.
    pub fn lost(&self) -> bool {
        match &self.transport {
            Transport::InProcess(_) => false,
            #[cfg(target_os = "linux")]
            Transport::Hosted(connection) => connection.lost(),
        }
    }

    /// What the program last reported with `OSC 9;4`; only hosted sessions read it.
    pub fn progress(&self) -> Progress {
        match &self.transport {
            Transport::InProcess(_) => Progress::None,
            #[cfg(target_os = "linux")]
            Transport::Hosted(connection) => connection.progress(),
        }
    }

    /// What the program last reported with `OSC 7`; only hosted sessions read it.
    pub fn directory(&self) -> Option<DirectoryReport> {
        match &self.transport {
            Transport::InProcess(_) => None,
            #[cfg(target_os = "linux")]
            Transport::Hosted(connection) => connection.directory(),
        }
    }

    /// Runs `read` with the terminal locked; keep it short, the PTY thread waits meanwhile.
    pub fn with_term<R>(&self, read: impl FnOnce(&mut Term<Listener>) -> R) -> R {
        read(&mut self.term.lock())
    }

    pub fn mode(&self) -> TermMode {
        *self.term.lock().mode()
    }

    /// Which program has the terminal; `None` where IVE cannot tell (not Unix, or no spare descriptor).
    #[cfg(unix)]
    pub fn foreground(&self) -> Option<&ForegroundProbe> {
        self.foreground.as_ref()
    }

    pub fn size(&self) -> GridSize {
        self.size.get()
    }

    pub fn resize(&self, size: GridSize) {
        if self.size.replace(size) == size {
            return;
        }
        self.term.lock().resize(size);
        self.send(Msg::Resize(size.window_size()));
    }

    /// Sends keyboard input: like other terminals, typing jumps back to the bottom and drops the selection.
    pub fn type_bytes(&self, bytes: Vec<u8>) {
        self.with_term(|term| {
            term.scroll_display(Scroll::Bottom);
            term.selection = None;
        });
        self.write(bytes);
    }

    /// Pastes `text`, bracketed when the program asked for it so it can tell a paste from typing.
    pub fn paste(&self, text: &str) {
        let bytes = if self.mode().contains(TermMode::BRACKETED_PASTE) {
            // a pasted end marker would let the paste break out of the bracket and run as typed
            format!("\x1b[200~{}\x1b[201~", text.replace('\x1b', ""))
        } else {
            text.replace("\r\n", "\r").replace('\n', "\r")
        };
        self.type_bytes(bytes.into_bytes());
    }

    /// Writes to the shell as is, for replies to its queries.
    pub fn write(&self, bytes: impl Into<Cow<'static, [u8]>>) {
        self.send(Msg::Input(bytes.into()));
    }

    pub fn scroll(&self, lines: i32) {
        self.with_term(|term| term.scroll_display(Scroll::Delta(lines)));
    }

    pub fn start_selection(&self, kind: SelectionType, point: Point, side: Side) {
        self.with_term(|term| term.selection = Some(Selection::new(kind, point, side)));
    }

    pub fn extend_selection(&self, point: Point, side: Side) {
        self.with_term(|term| {
            if let Some(selection) = term.selection.as_mut() {
                selection.update(point, side);
            }
        });
    }

    pub fn selection_text(&self) -> Option<String> {
        self.with_term(|term| term.selection_to_string()).filter(|text| !text.is_empty())
    }

    fn send(&self, message: Msg) {
        match &self.transport {
            // fails only after the shell exited and the PTY thread ended; the tab is closing then anyway
            Transport::InProcess(pty) => drop(pty.send(message)),
            #[cfg(target_os = "linux")]
            Transport::Hosted(connection) => match message {
                Msg::Input(bytes) => connection.input(&bytes),
                Msg::Resize(_) => connection.resize(self.size.get()),
                Msg::Shutdown => connection.hang_up(),
            },
        }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        // the PTY thread then drops the PTY, which sends SIGHUP to the shell; a hosted shell keeps running
        if let Transport::InProcess(_) = self.transport {
            self.send(Msg::Shutdown);
        }
    }
}

/// A terminal at the initial size, the listener feeding it and the events it raises.
fn new_term() -> (Arc<FairMutex<Term<Listener>>>, Listener, UnboundedReceiver<Event>) {
    let (sender, events) = unbounded();
    let listener = Listener::new(sender);
    let config = Config { scrolling_history: SCROLLBACK_LINES, ..Config::default() };
    let term = Arc::new(FairMutex::new(Term::new(config, &INITIAL_SIZE, listener.clone())));
    (term, listener, events)
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use std::os::unix::net::UnixStream;

    use ptyhost::client::Client;
    use ptyhost::protocol::{Frame, PROTOCOL};

    use super::*;

    fn hosted_session() -> (Session, UnixStream) {
        let (client_end, mut host) = UnixStream::pair().unwrap();
        Frame::Attached { protocol: PROTOCOL, program_pid: std::process::id() }.write_to(&mut host).unwrap();
        let (session, _events) = Session::over_client("t1".to_owned(), Client::from_stream(client_end).unwrap(), None);
        (session, host)
    }

    /// A hosted session past the resize it sends on attach.
    fn attached_session() -> (Session, UnixStream) {
        let (session, mut host) = hosted_session();
        assert!(matches!(Frame::read_from(&mut host).unwrap(), Frame::Resize(_)));
        (session, host)
    }

    #[test]
    fn should_detach_hosted_session_on_drop() {
        // given
        let (session, mut host) = attached_session();

        // when
        drop(session);

        // then
        assert_eq!(Frame::read_from(&mut host).unwrap_err().kind(), io::ErrorKind::UnexpectedEof);
    }

    #[test]
    fn should_send_hang_up_from_hang_up() {
        // given
        let (session, mut host) = attached_session();

        // when
        session.hang_up();

        // then
        assert_eq!(Frame::read_from(&mut host).unwrap(), Frame::HangUp);
    }

    #[test]
    fn should_ask_for_the_replay_before_the_first_layout() {
        // given
        let (_session, mut host) = hosted_session();
        host.set_read_timeout(Some(std::time::Duration::from_secs(1))).unwrap();

        // when
        let first = Frame::read_from(&mut host);

        // then
        assert_eq!(first.unwrap(), Frame::Resize(ptyhost::protocol::Size { columns: 80, lines: 24 }));
    }

    #[test]
    fn should_send_the_layout_size_once_known() {
        // given
        let (session, mut host) = hosted_session();
        host.set_read_timeout(Some(std::time::Duration::from_secs(1))).unwrap();
        let laid_out = GridSize { columns: 120, lines: 40, cell_width: 8, cell_height: 16 };

        // when
        session.resize(laid_out);

        // then
        let frames: Vec<Frame> = (0..2).map(|_| Frame::read_from(&mut host).unwrap()).collect();
        assert_eq!(frames.last(), Some(&Frame::Resize(ptyhost::protocol::Size { columns: 120, lines: 40 })));
    }
}
