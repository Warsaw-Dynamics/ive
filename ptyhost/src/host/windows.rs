//! The Windows host: alacritty's ConPTY (with `conpty.dll` next to the exe when there is one), the program in a job
//! object, and one `polling` loop over the PTY, the listener and the client. A thread waiting on the program's
//! process wakes the loop when it exits, since the PTY reports no end of its own once it is closed.

use std::collections::{HashMap, VecDeque};
use std::io::{self, Read, Write};
use std::os::windows::io::{AsRawSocket, BorrowedSocket};
use std::os::windows::process::CommandExt;
use std::process::{Child, Command};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use alacritty_terminal::event::{OnResize, WindowSize};
use alacritty_terminal::tty::{self, EventedReadWrite, Pty, Shell};
use polling::{Event, Events, PollMode, Poller};
use uds_windows::{UnixListener, UnixStream};
use windows_sys::Win32::Foundation::{CloseHandle, ERROR_ACCESS_DENIED, HANDLE, WAIT_OBJECT_0};
use windows_sys::Win32::System::Threading::{
    CREATE_BREAKAWAY_FROM_JOB, CREATE_NEW_PROCESS_GROUP, DETACHED_PROCESS, GetExitCodeProcess, INFINITE, OpenProcess,
    PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SYNCHRONIZE, WaitForSingleObject,
};

use super::{CLIENT_READ_TIMEOUT, CLIENT_WRITE_TIMEOUT, HANG_UP_GRACE, HostConfig, MAX_PENDING_INPUT, Replay, Watchdog, hidden_env, program_env};
use crate::job::Job;
use crate::protocol::{Frame, PROTOCOL, Size};
use crate::registry::{Registry, SessionInfo};

/// The PTY's own registration key is alacritty's business; these are the sockets'.
const LISTENER_KEY: usize = 3;
const CLIENT_KEY: usize = 4;
/// Output reads once the program exited, for what ConPTY still had on its way.
const DRAIN_WAIT: Duration = Duration::from_millis(100);

/// Runs `command` without a console and outside the starter's job, so neither closing the starter's terminal nor
/// ending its job ends the host.
pub fn spawn_detached(command: &mut Command) -> io::Result<Child> {
    let flags = DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP;
    match command.creation_flags(flags | CREATE_BREAKAWAY_FROM_JOB).spawn() {
        // the starter's job forbids leaving it; the host then ends with that job
        Err(error) if error.raw_os_error() == Some(ERROR_ACCESS_DENIED as i32) => command.creation_flags(flags).spawn(),
        spawned => spawned,
    }
}

/// Runs the host in this process until its program exits; returns the program's exit code.
pub fn serve(registry: &Registry, config: &HostConfig) -> io::Result<i32> {
    let pty = open_pty(config)?;
    let program = Program::of(&pty)?;
    let job = program_job(&pty);
    let poller = Arc::new(Poller::new()?);
    let exited = program.watch_exit(poller.clone());
    registry.create_dir()?;
    let socket = registry.socket(&config.id);
    // left behind by a host that was killed; `start` checked no live host owns the id
    let _ = std::fs::remove_file(&socket);
    let listener = UnixListener::bind(&socket)?;
    let watchdog = Watchdog::new(registry, &config.id, &socket)?;
    let session = SessionInfo {
        id: config.id.clone(),
        host_pid: std::process::id(),
        program_pid: program.pid,
        cwd: config.cwd.clone(),
        command: config.command.clone(),
        labels: config.labels.clone(),
        started_ms: SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |elapsed| elapsed.as_millis() as u64),
    };
    let _lease = registry.record(&session)?;
    let mut host = Host {
        pty: Some(pty),
        size: config.size,
        program_pid: program.pid,
        job,
        poller,
        listener,
        client: None,
        client_heard: false,
        ready: false,
        replay: Replay::default(),
        hang_up_deadline: None,
        pending: VecDeque::new(),
        watchdog,
    };
    host.register()?;
    host.run(&exited)?;
    host.drain_output()?;
    let code = program.exit_code();
    // forgotten before the client hears of the exit, so it finds the registry already clean
    registry.forget(&config.id);
    host.send(&Frame::Exited(code));
    Ok(code)
}

fn open_pty(config: &HostConfig) -> io::Result<Pty> {
    let (program, args) = config
        .command
        .split_first()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, format!("session {}: empty command", config.id)))?;
    let mut env: HashMap<String, String> = config.env.iter().cloned().collect();
    env.extend(program_env(std::env::vars_os()).into_iter().filter_map(|(key, value)| Some((key.into_string().ok()?, value.into_string().ok()?))));
    // the program inherits this process's environment, so what it must not see goes; no thread runs yet
    for key in hidden_env(std::env::vars_os()) {
        // SAFETY: the host is still single-threaded
        unsafe { std::env::remove_var(key) };
    }
    let options = tty::Options {
        shell: Some(Shell::new(quoted(program), args.to_vec())),
        working_directory: Some(config.cwd.clone()),
        env,
        ..tty::Options::default()
    };
    // alacritty panics, rather than failing, when Windows refuses a pseudoconsole
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| tty::new(&options, window_size(config.size), 0)))
        .unwrap_or_else(|_| Err(io::Error::other("Windows refused to create a pseudoconsole")))
        .map_err(|error| io::Error::new(error.kind(), format!("cannot start {program}: {error}")))
}

/// The program's job, which also ends it should the host crash, named so IVE can Stop and Kill through it; `None`
/// when Windows refuses one, and then a hang-up reaches the program through its console only. What the program starts
/// before it is in the job escapes it.
fn program_job(pty: &Pty) -> Option<Job> {
    // alacritty's handle has the rights a job needs; the host's own (`Program`) only waits and reads the exit code
    let job = Job::named_holding(Some(&Job::host_job_name(std::process::id())), pty.child_watcher().raw_handle()).ok()?;
    if let Err(error) = job.kill_on_close() {
        eprintln!("ptyhost: the program outlives a crash of its host: {error}");
    }
    Some(job)
}

/// alacritty joins the program and its arguments into one command line as they are, so a program path with a space
/// in it needs quotes.
fn quoted(program: &str) -> String {
    if program.contains(' ') {
        return format!("\"{program}\"");
    }
    program.to_owned()
}

fn window_size(size: Size) -> WindowSize {
    WindowSize { num_lines: size.lines, num_cols: size.columns, cell_width: 1, cell_height: 1 }
}

/// The program's process, held open so its pid is not reused and its exit code stays readable.
struct Program {
    pid: u32,
    handle: Arc<OwnedHandle>,
}

impl Program {
    fn of(pty: &Pty) -> io::Result<Program> {
        let pid = pty.child_watcher().pid().map(std::num::NonZeroU32::get).ok_or_else(|| io::Error::other("the program has no pid"))?;
        // SAFETY: plain call; a null handle is the failure
        let handle = unsafe { OpenProcess(PROCESS_SYNCHRONIZE | PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
        if handle.is_null() {
            return Err(io::Error::last_os_error());
        }
        Ok(Program { pid, handle: Arc::new(OwnedHandle(handle)) })
    }

    /// A flag set, and the loop woken, once the program exits.
    fn watch_exit(&self, poller: Arc<Poller>) -> Arc<AtomicBool> {
        let exited = Arc::new(AtomicBool::new(false));
        let (flag, handle) = (exited.clone(), self.handle.clone());
        std::thread::spawn(move || {
            // SAFETY: the handle stays open while this thread holds its Arc
            if unsafe { WaitForSingleObject(handle.0, INFINITE) } == WAIT_OBJECT_0 {
                flag.store(true, Ordering::SeqCst);
                let _ = poller.notify();
            }
        });
        exited
    }

    fn exit_code(&self) -> i32 {
        let mut code = 0u32;
        // SAFETY: the handle is open and `code` a valid out-pointer
        unsafe { GetExitCodeProcess(self.handle.0, &mut code) };
        code as i32
    }
}

struct OwnedHandle(HANDLE);

// SAFETY: a process handle may be used from any thread
unsafe impl Send for OwnedHandle {}
// SAFETY: as above; waiting on it and reading its exit code are thread-safe
unsafe impl Sync for OwnedHandle {}

impl Drop for OwnedHandle {
    fn drop(&mut self) {
        // SAFETY: the handle is open and owned
        unsafe { CloseHandle(self.0) };
    }
}

struct Host {
    /// `None` once hung up: closing the pseudoconsole is the hang-up
    pty: Option<Pty>,
    /// the size of the pseudoconsole, which a resize to the same size would not repaint
    size: Size,
    program_pid: u32,
    job: Option<Job>,
    poller: Arc<Poller>,
    listener: UnixListener,
    client: Option<UnixStream>,
    /// whether the poller wakes for the client's frames; not while the program has a backlog of input
    client_heard: bool,
    /// the client sent its first `Resize` and got the replay; until then output only goes to the replay
    ready: bool,
    replay: Replay,
    /// when the job is ended after a hang-up, until it is
    hang_up_deadline: Option<Instant>,
    /// input the program has not taken yet, written as the PTY accepts it
    pending: VecDeque<u8>,
    watchdog: Watchdog,
}

impl Host {
    fn register(&mut self) -> io::Result<()> {
        if let Some(pty) = self.pty.as_mut() {
            // SAFETY: the PTY is deregistered by being dropped, which ends its registrations with it
            unsafe { pty.register(&self.poller, Event::all(0), PollMode::Level)? };
        }
        // SAFETY: the listener lives as long as the poller, in this struct
        unsafe { self.poller.add_with_mode(&self.listener, Event::readable(LISTENER_KEY), PollMode::Level) }
    }

    fn run(&mut self, exited: &AtomicBool) -> io::Result<()> {
        let mut events = Events::new();
        loop {
            events.clear();
            self.follow_backlog();
            self.poller.wait(&mut events, Some(self.timeout()))?;
            if exited.load(Ordering::SeqCst) {
                return Ok(());
            }
            self.kill_after_grace();
            if self.watchdog.entry_lost() {
                self.hang_up();
            }
            self.forward_output();
            self.write_pending();
            if events.iter().any(|event| event.key == LISTENER_KEY) {
                self.attach()?;
            }
            if events.iter().any(|event| event.key == CLIENT_KEY) {
                self.handle_client();
            }
        }
    }

    fn timeout(&self) -> Duration {
        let deadline = self.hang_up_deadline.map_or(self.watchdog.next_check, |deadline| deadline.min(self.watchdog.next_check));
        deadline.saturating_duration_since(Instant::now())
    }

    /// Passes on what the PTY has; false when it had nothing.
    fn forward_output(&mut self) -> bool {
        let mut buffer = [0u8; 64 * 1024];
        let mut forwarded = false;
        // alacritty's reader reads 0 when nothing is waiting, and wakes the poller when something arrives
        while let Some(pty) = self.pty.as_mut()
            && let Ok(read @ 1..) = pty.reader().read(&mut buffer)
        {
            forwarded = true;
            self.replay.push(&buffer[..read]);
            if self.ready {
                self.send(&Frame::Output(buffer[..read].to_vec()));
            }
        }
        forwarded
    }

    /// Passes on what ConPTY still had on its way once the program exited, until it has been quiet for
    /// [`DRAIN_WAIT`]; the sockets are left out, so a client's unread frames cannot end it early.
    fn drain_output(&mut self) -> io::Result<()> {
        // SAFETY: the listener outlives this borrow; uds_windows gives it no AsSocket of its own
        let listener = unsafe { BorrowedSocket::borrow_raw(self.listener.as_raw_socket()) };
        let _ = self.poller.delete(listener);
        if let Some(client) = &self.client {
            let _ = self.poller.delete(client);
        }
        let mut events = Events::new();
        let mut quiet_since = Instant::now();
        while let Some(left) = DRAIN_WAIT.checked_sub(quiet_since.elapsed()) {
            events.clear();
            self.poller.wait(&mut events, Some(left))?;
            if self.forward_output() {
                quiet_since = Instant::now();
            }
        }
        Ok(())
    }

    /// Hears the client only while the program has room for more input, as the Unix host leaves it out of `poll`:
    /// a level-triggered socket left unread would wake the loop again at once.
    fn follow_backlog(&mut self) {
        let Some(client) = &self.client else { return };
        let hear = self.pending.len() < MAX_PENDING_INPUT;
        if hear == self.client_heard {
            return;
        }
        let event = if hear { Event::readable(CLIENT_KEY) } else { Event::none(CLIENT_KEY) };
        if self.poller.modify_with_mode(client, event, PollMode::Level).is_ok() {
            self.client_heard = hear;
        }
    }

    /// A new client replaces the current one, which sees its connection close. It gets the handshake now and the
    /// replay after its first `Resize`, so the replay is drawn at the client's size.
    fn attach(&mut self) -> io::Result<()> {
        let (client, _) = self.listener.accept()?;
        self.drain_displaced_client();
        client.set_write_timeout(Some(CLIENT_WRITE_TIMEOUT))?;
        client.set_read_timeout(Some(CLIENT_READ_TIMEOUT))?;
        // SAFETY: the client is deleted from the poller before it is dropped (`drop_client`)
        unsafe { self.poller.add_with_mode(&client, Event::readable(CLIENT_KEY), PollMode::Level)? };
        self.client = Some(client);
        self.client_heard = true;
        self.ready = false;
        self.send(&Frame::Attached { protocol: PROTOCOL, program_pid: self.program_pid });
        Ok(())
    }

    /// Applies what the client being replaced already sent: its last frames, a hang-up above all, must not be lost
    /// because a new client connected in the same round.
    fn drain_displaced_client(&mut self) {
        let Some(mut client) = self.take_client() else { return };
        if client.set_nonblocking(true).is_err() {
            return;
        }
        // a frame cut short by the nonblocking read belonged to a client that is going away anyway
        while let Ok(frame) = Frame::read_from(&mut client) {
            match frame {
                Frame::Input(bytes) => self.write_to_program(&bytes),
                Frame::Resize(size) => self.resize(size),
                Frame::HangUp => self.hang_up(),
                Frame::Output(_) | Frame::Replay(_) | Frame::Exited(_) | Frame::Attached { .. } => {}
            }
        }
    }

    fn handle_client(&mut self) {
        if self.pending.len() >= MAX_PENDING_INPUT {
            return;
        }
        let Some(client) = self.client.as_mut() else { return };
        match Frame::read_from(client) {
            Ok(Frame::Input(bytes)) => self.write_to_program(&bytes),
            Ok(Frame::Resize(size)) => {
                self.resize(size);
                if !self.ready {
                    self.ready = true;
                    self.send(&Frame::Replay(self.replay.snapshot()));
                }
            }
            Ok(Frame::HangUp) => self.hang_up(),
            Ok(_) => {}
            Err(_) => drop(self.take_client()),
        }
    }

    fn take_client(&mut self) -> Option<UnixStream> {
        let client = self.client.take()?;
        let _ = self.poller.delete(&client);
        Some(client)
    }

    fn write_to_program(&mut self, bytes: &[u8]) {
        self.pending.extend(bytes);
        self.write_pending();
    }

    /// Writes what the PTY takes now; alacritty's writer takes 0 when full and wakes the poller once it has room.
    fn write_pending(&mut self) {
        let Some(pty) = self.pty.as_mut() else { return self.pending.clear() };
        while !self.pending.is_empty() {
            match pty.writer().write(self.pending.as_slices().0) {
                Ok(0) => return,
                Ok(written) => drop(self.pending.drain(..written)),
                // the terminal is gone; nothing will read this input
                Err(_) => return self.pending.clear(),
            }
        }
    }

    /// Hangs up like closing a console window: closing the pseudoconsole sends its programs `CTRL_CLOSE_EVENT`; the
    /// job ends what is left once the grace ends.
    fn hang_up(&mut self) {
        if self.hang_up_deadline.is_some() || self.pty.is_none() {
            return;
        }
        self.forward_output();
        self.pty = None;
        self.hang_up_deadline = Some(Instant::now() + HANG_UP_GRACE);
    }

    fn kill_after_grace(&mut self) {
        if self.hang_up_deadline.is_some_and(|deadline| Instant::now() >= deadline) {
            self.hang_up_deadline = None;
            if let Some(job) = &self.job {
                job.terminate();
            }
        }
    }

    /// Sets the size; a resize to the size it has goes through one column less first, as ConPTY repaints only on a
    /// change, and a re-attached client must get the screen drawn.
    fn resize(&mut self, size: Size) {
        let Some(pty) = self.pty.as_mut() else { return };
        if size == self.size && size.columns > 1 {
            pty.on_resize(window_size(Size { columns: size.columns - 1, ..size }));
        }
        pty.on_resize(window_size(size));
        self.size = size;
    }

    fn send(&mut self, frame: &Frame) {
        let failed = self.client.as_mut().is_some_and(|client| frame.write_to(client).is_err());
        if failed {
            drop(self.take_client());
        }
    }
}
