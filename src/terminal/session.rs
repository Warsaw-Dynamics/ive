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

use super::foreground::ForegroundProbe;
use super::directory::DirectoryReport;
use super::launch::Launch;
use super::attention::CallReport;
use super::progress::Progress;
use super::theme_reports::theme_report;
use super::hosted::{Connection, HostedTerminals};
#[cfg(test)]
use ptyhost::client::Client;
use ptyhost::registry::SessionInfo;
#[cfg(test)]
use ptyhost::registry::Registry;

use crate::theme::Appearance;

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
/// hangs up, or in a host, which dropping the session leaves running; [`Session::hang_up`] ends either.
pub struct Session {
    term: Arc<FairMutex<Term<Listener>>>,
    transport: Transport,
    size: Cell<GridSize>,
    focus_reports: Cell<FocusReports>,
    program_pid: Option<u32>,
    foreground: Option<ForegroundProbe>,
    /// the job holding the program and what it started: IVE's own, or the host's
    #[cfg(windows)]
    job: Option<Arc<crate::job::Job>>,
}

enum Transport {
    /// Windows: the PTY reports nothing once it is shut down, so a hang-up reports the exit through the listener
    InProcess(EventLoopSender, #[cfg(windows)] Listener),
    Hosted(Connection),
}

/// Last focus state told to a program using terminal focus reporting (DECSET 1004).
#[derive(Default, Clone, Copy)]
struct FocusReports(Option<bool>);

impl FocusReports {
    fn next(&mut self, mode: TermMode, visible: bool) -> Option<&'static [u8]> {
        if !mode.contains(TermMode::FOCUS_IN_OUT) {
            self.0 = None;
            return None;
        }
        if self.0 == Some(visible) {
            return None;
        }
        self.0 = Some(visible);
        Some(if visible { b"\x1b[I" } else { b"\x1b[O" })
    }
}

impl Session {
    /// Starts `launch` (the user's login shell when it names no program) in `working_directory`; its events arrive
    /// on the returned receiver.
    pub fn spawn(working_directory: &Path, launch: &Launch) -> io::Result<(Self, UnboundedReceiver<Event>)> {
        let (term, listener, events) = new_term();
        let shell = launch.program.split_first().map(|(program, args)| tty::Shell::new(quoted(program), args.to_vec()));
        let options = tty::Options {
            shell,
            working_directory: Some(working_directory.to_path_buf()),
            env: super::env::variables().into_iter().chain(launch.env.iter().cloned()).collect(),
            ..tty::Options::default()
        };
        let pty = open_pty(&options)?;
        // taken before the event loop owns the PTY
        #[cfg(unix)]
        let foreground = ForegroundProbe::of(&pty);
        #[cfg(unix)]
        let program_pid = Some(pty.child().id());
        #[cfg(windows)]
        let program_pid = pty.child_watcher().pid().map(std::num::NonZeroU32::get);
        #[cfg(windows)]
        let foreground = program_pid.and_then(ForegroundProbe::of_shell);
        #[cfg(windows)]
        let reporter = listener.clone();
        // what the shell starts before this escapes the job; without a job, Kill is a hang-up
        #[cfg(windows)]
        let job = crate::job::Job::holding(pty.child_watcher().raw_handle()).ok().map(Arc::new);
        let event_loop = EventLoop::new(term.clone(), listener, pty, false, false)?;
        let transport = Transport::InProcess(
            event_loop.channel(),
            #[cfg(windows)]
            reporter,
        );
        event_loop.spawn();
        let session = Self {
            term,
            transport,
            size: Cell::new(INITIAL_SIZE),
            focus_reports: Cell::new(FocusReports::default()),
            program_pid,
            foreground,
            #[cfg(windows)]
            job,
        };
        Ok((session, events))
    }

    /// Starts `launch` in a new host, labelled with `labels`.
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
    pub fn attach(terminals: &HostedTerminals, session: &SessionInfo) -> io::Result<(Self, UnboundedReceiver<Event>)> {
        let (term, listener, events) = new_term();
        let connection = terminals.connect(&session.id, term.clone(), listener)?;
        #[cfg_attr(unix, allow(unused_mut))]
        let mut hosted = Self::hosted(term, connection);
        // Windows: Stop and Kill reach what the program started through the host's job
        #[cfg(windows)]
        {
            hosted.job = crate::job::Job::open(&crate::job::Job::host_job_name(session.host_pid)).ok().map(Arc::new);
        }
        Ok((hosted, events))
    }

    /// A hosted session over an already attached client, for tests that play the host.
    #[cfg(test)]
    pub fn over_client(id: String, client: Client, registry: Option<Registry>) -> (Self, UnboundedReceiver<Event>) {
        let (term, listener, events) = new_term();
        let connection = Connection::start(id, client, registry, term.clone(), listener);
        (Self::hosted(term, connection), events)
    }

    fn hosted(term: Arc<FairMutex<Term<Listener>>>, connection: Connection) -> Self {
        let program_pid = Some(connection.program_pid());
        let foreground = Some(ForegroundProbe::of_program(connection.program_pid()));
        // the host replays only after a first resize; sent now, so a tab never shown still gets its output and title
        connection.resize(INITIAL_SIZE);
        Self {
            term,
            transport: Transport::Hosted(connection),
            size: Cell::new(INITIAL_SIZE),
            focus_reports: Cell::new(FocusReports::default()),
            program_pid,
            foreground,
            #[cfg(windows)]
            job: None,
        }
    }

    /// The program the terminal started, its process group's leader: the shell, or a run's command.
    pub fn program_pid(&self) -> Option<u32> {
        self.program_pid
    }

    /// The job holding the program and what it started.
    #[cfg(windows)]
    pub fn job(&self) -> Option<Arc<crate::job::Job>> {
        self.job.clone()
    }

    /// Ends the shell, as closing its terminal does.
    pub fn hang_up(&self) {
        self.send(Msg::Shutdown);
        // alacritty's loop ends at the shutdown, before the closed pseudoconsole has ended the program, so the exit
        // a Unix PTY would report comes from here; a host reports its program's exit itself
        #[cfg(windows)]
        if let Transport::InProcess(_, listener) = &self.transport {
            listener.send_event(Event::ChildExit(std::os::windows::process::ExitStatusExt::from_raw(1)));
            listener.send_event(Event::Exit);
        }
    }

    /// The host session's id, for hosted sessions.
    pub fn hosted_id(&self) -> Option<&str> {
        match &self.transport {
            Transport::InProcess(..) => None,
            Transport::Hosted(connection) => Some(connection.id()),
        }
    }

    /// Why a hosted session's connection broke for good, once it has; its program may still run.
    pub fn lost_reason(&self) -> Option<String> {
        match &self.transport {
            Transport::InProcess(..) => None,
            Transport::Hosted(connection) => connection.lost_reason(),
        }
    }

    /// What the program last reported with `OSC 9;4`; only hosted sessions read it.
    pub fn progress(&self) -> Progress {
        match &self.transport {
            Transport::InProcess(..) => Progress::None,
            Transport::Hosted(connection) => connection.progress(),
        }
    }

    /// The program's last call for the user's attention in live output; only hosted sessions read it.
    pub fn call(&self) -> Option<CallReport> {
        match &self.transport {
            Transport::InProcess(..) => None,
            Transport::Hosted(connection) => connection.call(),
        }
    }

    /// What the program last reported with `OSC 7`; only hosted sessions read it.
    pub fn directory(&self) -> Option<DirectoryReport> {
        match &self.transport {
            Transport::InProcess(..) => None,
            Transport::Hosted(connection) => connection.directory(),
        }
    }

    /// Tells the program the theme turned `appearance`, if it asked to hear of that; only hosted sessions know.
    pub fn report_theme(&self, appearance: Appearance) {
        if self.wants_theme_reports() {
            self.write(theme_report(appearance));
        }
    }

    fn wants_theme_reports(&self) -> bool {
        match &self.transport {
            Transport::InProcess(..) => false,
            Transport::Hosted(connection) => connection.wants_theme_reports(),
        }
    }

    /// Runs `read` with the terminal locked; keep it short, the PTY thread waits meanwhile.
    pub fn with_term<R>(&self, read: impl FnOnce(&mut Term<Listener>) -> R) -> R {
        read(&mut self.term.lock())
    }

    pub fn mode(&self) -> TermMode {
        *self.term.lock().mode()
    }

    /// Tells a program using terminal focus reporting whether the user can see this terminal.
    pub fn report_focus(&self, visible: bool) {
        let mut reports = self.focus_reports.get();
        let reply = reports.next(self.mode(), visible);
        self.focus_reports.set(reports);
        if let Some(bytes) = reply {
            self.write(bytes);
        }
    }

    /// Which program has the terminal; `None` where IVE cannot tell (no spare descriptor).
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
            Transport::InProcess(pty, ..) => drop(pty.send(message)),
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
        if let Transport::InProcess(..) = self.transport {
            self.send(Msg::Shutdown);
        }
    }
}

#[cfg(unix)]
fn quoted(program: &str) -> String {
    program.to_owned()
}

/// On Windows alacritty joins the program and its arguments into one command line as they are, so a program path
/// with a space in it needs quotes.
#[cfg(windows)]
fn quoted(program: &str) -> String {
    if program.contains(' ') {
        return format!("\"{program}\"");
    }
    program.to_owned()
}

#[cfg(unix)]
fn open_pty(options: &tty::Options) -> io::Result<tty::Pty> {
    tty::new(options, INITIAL_SIZE.window_size(), 0)
}

/// alacritty panics, rather than failing, when Windows refuses a pseudoconsole.
#[cfg(windows)]
fn open_pty(options: &tty::Options) -> io::Result<tty::Pty> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| tty::new(options, INITIAL_SIZE.window_size(), 0)))
        .unwrap_or_else(|_| Err(io::Error::other("Windows refused to create a pseudoconsole")))
}

/// A terminal at the initial size, the listener feeding it and the events it raises.
fn new_term() -> (Arc<FairMutex<Term<Listener>>>, Listener, UnboundedReceiver<Event>) {
    let (sender, events) = unbounded();
    let listener = Listener::new(sender);
    let config = Config { scrolling_history: SCROLLBACK_LINES, ..Config::default() };
    let term = Arc::new(FairMutex::new(Term::new(config, &INITIAL_SIZE, listener.clone())));
    (term, listener, events)
}

#[cfg(test)]
mod tests {
    use ptyhost::client::Client;
    use ptyhost::net::UnixStream;
    use ptyhost::protocol::{Frame, PROTOCOL};

    use super::*;

    #[test]
    fn should_report_visibility_changes_when_a_program_requests_focus_events() {
        // given
        let mut reports = FocusReports::default();
        let enabled = TermMode::FOCUS_IN_OUT;

        // when
        let sent = [
            reports.next(TermMode::empty(), false),
            reports.next(enabled, false),
            reports.next(enabled, false),
            reports.next(enabled, true),
            reports.next(TermMode::empty(), true),
            reports.next(enabled, false),
        ];

        // then
        assert_eq!(sent, [None, Some(b"\x1b[O".as_slice()), None, Some(b"\x1b[I".as_slice()), None, Some(b"\x1b[O".as_slice())]);
    }

    #[test]
    fn should_report_hidden_focus_after_a_hosted_session_replays_its_mode() {
        // given
        let (session, mut host) = attached_session();
        host.set_read_timeout(Some(std::time::Duration::from_secs(2))).unwrap();
        Frame::Replay(b"\x1b[?1004h".to_vec()).write_to(&mut host).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        while !session.mode().contains(TermMode::FOCUS_IN_OUT) && std::time::Instant::now() < deadline {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(session.mode().contains(TermMode::FOCUS_IN_OUT));

        // when
        session.report_focus(false);

        // then
        assert_eq!(Frame::read_from(&mut host).unwrap(), Frame::Input(b"\x1b[O".to_vec()));
    }

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
