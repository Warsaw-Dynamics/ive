//! The host: owns one PTY and the program in it, outlives whoever started it, serves one client at a time.

use std::collections::VecDeque;
use std::fs::File;
use std::io::{self, Read, Write};
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
use std::os::unix::net::{UnixListener, UnixStream};
use std::ffi::OsString;
use std::os::unix::process::{CommandExt, ExitStatusExt};
use std::path::PathBuf;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::protocol::{Frame, PROTOCOL, Size};
use crate::registry::{DIR_ENV, Registry, SessionInfo, validate_id, validate_label};
use crate::title::TitleTracker;

/// Output kept for replay to the next client to attach.
const REPLAY_BYTES: usize = 1 << 20;
/// A client that stops reading for this long is dropped rather than stalling the program.
const CLIENT_WRITE_TIMEOUT: Duration = Duration::from_secs(10);
/// How long a frame may take to arrive in full once it started.
const CLIENT_READ_TIMEOUT: Duration = Duration::from_secs(1);
/// After a hang-up, how long the program gets to exit before it is killed.
const HANG_UP_GRACE: Duration = Duration::from_secs(2);
const START_TIMEOUT: Duration = Duration::from_secs(3);
/// Input waiting for the program; past this the host stops reading the client, which then waits in turn.
const MAX_PENDING_INPUT: usize = 4 << 20;
/// How often the host checks that its registry entry is still there.
const WATCHDOG_INTERVAL: Duration = Duration::from_secs(2);
/// Names the host's environment variables meant for its program (see `host_env`).
const PROGRAM_ENV_PREFIX: &str = "PTYHOST_ENV_";

#[derive(Debug, Clone)]
pub struct HostConfig {
    pub id: String,
    pub command: Vec<String>,
    pub cwd: PathBuf,
    pub size: Size,
    /// set for the program, after `TERM`
    pub env: Vec<(String, String)>,
    pub labels: Vec<(String, String)>,
}

/// How to run a host: `exe`, then `args`, then `serve …`. The `ptyhost` binary is `ptyhost serve …`; a program
/// embedding the CLI passes its own flag, such as `ive --pty-host serve …`.
#[derive(Debug, Clone)]
pub struct Launcher {
    pub exe: PathBuf,
    pub args: Vec<OsString>,
}

/// Starts a detached host and returns its pid once the session accepts attaches.
///
/// The host runs in its own session, so closing the starter's terminal or signalling its process group misses it.
pub fn start(launcher: &Launcher, registry: &Registry, config: &HostConfig) -> io::Result<u32> {
    validate_id(&config.id)?;
    registry.check_socket_path(&config.id)?;
    for (key, value) in &config.labels {
        validate_label(key, value)?;
    }
    if registry.session(&config.id)?.is_some() {
        return Err(io::Error::new(io::ErrorKind::AlreadyExists, format!("session {} is already running", config.id)));
    }
    registry.create_dir().map_err(|error| about(error, "cannot create the terminals folder", registry.dir()))?;
    let log = registry.log(&config.id);
    let log_file = File::create(&log).map_err(|error| about(error, "cannot create the terminal host's log", &log))?;
    let mut command = Command::new(&launcher.exe);
    command
        .args(&launcher.args)
        .args(serve_args(config))
        .envs(host_env(config))
        .env(DIR_ENV, registry.dir())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(log_file);
    // SAFETY: setsid is async-signal-safe
    unsafe { command.pre_exec(|| check(libc::setsid()).map(drop)) };
    let mut host = command.spawn().map_err(|error| about(error, "cannot start the terminal host", &launcher.exe))?;
    let pid = host.id();
    let started = wait_until_live(registry, &config.id, &mut host);
    // reaps the host if it ends while the starter still runs; after that, init does
    std::thread::spawn(move || host.wait());
    started.map(|()| pid)
}

/// `error` with what failed and on which path, keeping its kind: `cannot start the terminal host /usr/bin/ive: …`.
fn about(error: io::Error, what: &str, path: &std::path::Path) -> io::Error {
    io::Error::new(error.kind(), format!("{what} {}: {error}", path.display()))
}

fn serve_args(config: &HostConfig) -> Vec<OsString> {
    let mut args: Vec<OsString> = ["serve", "--id", &config.id, "--size", &format!("{}x{}", config.size.columns, config.size.lines), "--cwd"]
        .map(OsString::from)
        .to_vec();
    args.push(config.cwd.clone().into());
    for (key, value) in &config.labels {
        args.push("--label".into());
        args.push(format!("{key}={value}").into());
    }
    args.push("--".into());
    args.extend(config.command.iter().map(OsString::from));
    args
}

/// The program's environment as the host's own, prefixed: a command line is visible to every user (`ps`), and
/// `env` may hold secrets.
fn host_env(config: &HostConfig) -> Vec<(String, String)> {
    config.env.iter().map(|(key, value)| (format!("{PROGRAM_ENV_PREFIX}{key}"), value.clone())).collect()
}

/// The variables `host_env` passed, unprefixed, from the host's environment `vars`; any bytes, since an environment
/// need not be UTF-8.
fn program_env(vars: impl Iterator<Item = (OsString, OsString)>) -> Vec<(OsString, OsString)> {
    use std::os::unix::ffi::{OsStrExt, OsStringExt};
    vars.filter_map(|(key, value)| {
        let name = key.as_bytes().strip_prefix(PROGRAM_ENV_PREFIX.as_bytes()).filter(|name| !name.is_empty())?;
        Some((OsString::from_vec(name.to_vec()), value))
    })
    .collect()
}

/// Waits until the session is live, or the host ended first; on failure the host's log becomes the error.
fn wait_until_live(registry: &Registry, id: &str, host: &mut Child) -> io::Result<()> {
    let deadline = Instant::now() + START_TIMEOUT;
    let mut ended = None;
    while ended.is_none() && Instant::now() < deadline {
        if registry.session(id)?.is_some() {
            return Ok(());
        }
        ended = host.try_wait()?;
        std::thread::sleep(Duration::from_millis(10));
    }
    let log = std::fs::read_to_string(registry.log(id)).unwrap_or_default();
    let _ = std::fs::remove_file(registry.log(id));
    let reason = match ended {
        Some(status) => format!("ended ({status})"),
        None => "did not start within 3 s".to_owned(),
    };
    Err(io::Error::other(format!("session {id} {reason}: {}", log.trim())))
}

/// Runs the host in this process until its program exits; returns the program's exit code.
pub fn serve(registry: &Registry, config: &HostConfig) -> io::Result<i32> {
    close_inherited_descriptors();
    ignore_signal(libc::SIGHUP);
    ignore_signal(libc::SIGPIPE);
    // before the program starts, so it and all it starts are in the scope too
    #[cfg(target_os = "linux")]
    if let Err(reason) = crate::scope::move_into_own_scope(&config.id, &config.cwd) {
        eprintln!("ptyhost: {reason}; the terminal stays in its starter's cgroup");
    }
    let (master, slave) = open_pty(config.size)?;
    let mut program = spawn_program(slave, config)?;
    let exited = exit_watch(program.id())?;
    registry.create_dir()?;
    let socket = registry.socket(&config.id);
    // left behind by a host that was killed; `start` checked no live host owns the id
    let _ = std::fs::remove_file(&socket);
    let listener = UnixListener::bind(&socket)?;
    let watchdog = Watchdog { registry: registry.clone(), id: config.id.clone(), socket: socket.clone(), inode: inode_of(&socket)?, next_check: Instant::now() };
    let session = SessionInfo {
        id: config.id.clone(),
        host_pid: std::process::id(),
        program_pid: program.id(),
        cwd: config.cwd.clone(),
        command: config.command.clone(),
        labels: config.labels.clone(),
        started_ms: SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |elapsed| elapsed.as_millis() as u64),
    };
    let _lease = registry.record(&session)?;
    let mut host = Host {
        master: Some(master),
        program_pid: program.id(),
        listener,
        client: None,
        ready: false,
        replay: Replay::default(),
        hung_up: None,
        pending: VecDeque::new(),
        watchdog,
    };
    host.run(&exited)?;
    let code = exit_code(program.wait()?);
    host.drain_output()?;
    // forgotten before the client hears of the exit, so it finds the registry already clean
    registry.forget(&config.id);
    host.send(&Frame::Exited(code));
    Ok(code)
}

struct Host {
    /// `None` once the PTY hung up: the program closed its terminal, or a client asked for a hang-up
    master: Option<File>,
    program_pid: u32,
    listener: UnixListener,
    client: Option<UnixStream>,
    /// the client sent its first `Resize` and got the replay; until then output only goes to the replay
    ready: bool,
    replay: Replay,
    hung_up: Option<HungUp>,
    /// input the program has not taken yet; written as the PTY accepts it, between reads of its output, so a
    /// program that echoes what it reads never blocks on output while the host blocks on its input
    pending: VecDeque<u8>,
    watchdog: Watchdog,
}

/// A host whose socket or meta file is gone can never be reached again (its registry was deleted, as at logout
/// without logind killing user processes); it hangs up rather than run on unseen.
struct Watchdog {
    registry: Registry,
    id: String,
    socket: PathBuf,
    inode: (u64, u64),
    next_check: Instant,
}

impl Watchdog {
    /// True when a check was due and found the entry gone or taken over.
    fn entry_lost(&mut self) -> bool {
        if Instant::now() < self.next_check {
            return false;
        }
        self.next_check = Instant::now() + WATCHDOG_INTERVAL;
        let own_socket = inode_of(&self.socket).is_ok_and(|inode| inode == self.inode);
        let listed = self.registry.session(&self.id).is_ok_and(|session| session.is_some());
        !(own_socket && listed)
    }
}

fn inode_of(path: &std::path::Path) -> io::Result<(u64, u64)> {
    use std::os::unix::fs::MetadataExt;
    let metadata = std::fs::metadata(path)?;
    Ok((metadata.dev(), metadata.ino()))
}

/// A hang-up under way: the process groups to kill once the grace ends.
struct HungUp {
    deadline: Instant,
    groups: Vec<libc::pid_t>,
    killed: bool,
}

#[derive(Default)]
struct Ready {
    exited: bool,
    output: bool,
    writable: bool,
    attach: bool,
    client: bool,
}

impl Host {
    fn run(&mut self, exited: &OwnedFd) -> io::Result<()> {
        loop {
            let ready = self.wait(exited)?;
            if ready.exited {
                return Ok(());
            }
            self.kill_after_grace();
            if self.watchdog.entry_lost() {
                self.hang_up();
            }
            if ready.output {
                self.forward_output()?;
            }
            if ready.writable {
                self.write_pending();
            }
            if ready.attach {
                self.attach()?;
            }
            if ready.client {
                self.handle_client();
            }
        }
    }

    fn wait(&self, exited: &OwnedFd) -> io::Result<Ready> {
        let mut fds = vec![pollfd(exited.as_raw_fd()), pollfd(self.listener.as_raw_fd())];
        if let Some(master) = &self.master {
            let events = if self.pending.is_empty() { libc::POLLIN } else { libc::POLLIN | libc::POLLOUT };
            fds.push(libc::pollfd { fd: master.as_raw_fd(), events, revents: 0 });
        }
        if let Some(client) = &self.client
            && self.pending.len() < MAX_PENDING_INPUT
        {
            fds.push(pollfd(client.as_raw_fd()));
        }
        let mut deadline = self.watchdog.next_check;
        if let Some(hung_up) = self.hung_up.as_ref().filter(|hung_up| !hung_up.killed) {
            deadline = deadline.min(hung_up.deadline);
        }
        poll(&mut fds, deadline.saturating_duration_since(Instant::now()).as_millis() as i32)?;
        let revents = |fd: RawFd| fds.iter().find(|polled| polled.fd == fd).map_or(0, |polled| polled.revents);
        let master = self.master.as_ref().map_or(0, |master| revents(master.as_raw_fd()));
        Ok(Ready {
            exited: revents(exited.as_raw_fd()) != 0,
            attach: revents(self.listener.as_raw_fd()) != 0,
            output: master & !libc::POLLOUT != 0,
            writable: master & libc::POLLOUT != 0,
            client: self.client.as_ref().is_some_and(|client| revents(client.as_raw_fd()) != 0),
        })
    }

    /// Passes on one read of program output; false when there was none to read.
    fn forward_output(&mut self) -> io::Result<bool> {
        let Some(master) = self.master.as_mut() else { return Ok(false) };
        let mut buffer = [0u8; 64 * 1024];
        match master.read(&mut buffer) {
            Ok(0) => self.master = None,
            Ok(read) => {
                self.replay.push(&buffer[..read]);
                if self.ready {
                    self.send(&Frame::Output(buffer[..read].to_vec()));
                }
                return Ok(true);
            }
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
            // EIO: every holder of the PTY's other end closed it
            Err(error) if error.raw_os_error() == Some(libc::EIO) => self.master = None,
            Err(error) => return Err(error),
        }
        Ok(false)
    }

    fn drain_output(&mut self) -> io::Result<()> {
        while self.forward_output()? {}
        Ok(())
    }

    /// A new client replaces the current one, which sees its connection close. It gets the handshake now and the
    /// replay after its first `Resize`, so the replay is drawn at the client's size.
    fn attach(&mut self) -> io::Result<()> {
        let (client, _) = self.listener.accept()?;
        self.drain_displaced_client();
        client.set_write_timeout(Some(CLIENT_WRITE_TIMEOUT))?;
        client.set_read_timeout(Some(CLIENT_READ_TIMEOUT))?;
        self.client = Some(client);
        self.ready = false;
        self.send(&Frame::Attached { protocol: PROTOCOL, program_pid: self.program_pid });
        Ok(())
    }

    /// Applies what the client being replaced already sent: its last frames, a hang-up above all, must not be lost
    /// because a new client connected in the same poll round.
    fn drain_displaced_client(&mut self) {
        let Some(mut client) = self.client.take() else { return };
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
            Err(_) => self.client = None,
        }
    }

    fn write_to_program(&mut self, bytes: &[u8]) {
        self.pending.extend(bytes);
        self.write_pending();
    }

    /// Writes what the PTY takes now without waiting; the rest goes when the poll loop sees it writable.
    fn write_pending(&mut self) {
        while let Some(master) = self.master.as_mut()
            && !self.pending.is_empty()
        {
            match master.write(self.pending.as_slices().0) {
                Ok(written) => drop(self.pending.drain(..written)),
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => return,
                // the terminal is gone; nothing will read this input
                Err(_) => return self.pending.clear(),
            }
        }
    }

    /// Hangs up like closing a terminal: closing the master makes the kernel send SIGHUP to the program and the
    /// foreground job; both groups get SIGHUP from here too, and SIGKILL once the grace ends.
    fn hang_up(&mut self) {
        if self.hung_up.is_some() {
            return;
        }
        let program = self.program_pid as libc::pid_t;
        let mut groups = vec![program];
        if let Some(master) = self.master.take() {
            // SAFETY: tcgetpgrp only reads terminal state of a descriptor we own
            let foreground = unsafe { libc::tcgetpgrp(master.as_raw_fd()) };
            if foreground > 0 && foreground != program {
                groups.push(foreground);
            }
        }
        signal_groups(&groups, libc::SIGHUP);
        self.hung_up = Some(HungUp { deadline: Instant::now() + HANG_UP_GRACE, groups, killed: false });
    }

    fn kill_after_grace(&mut self) {
        if let Some(hung_up) = self.hung_up.as_mut()
            && !hung_up.killed
            && Instant::now() >= hung_up.deadline
        {
            signal_groups(&hung_up.groups, libc::SIGKILL);
            hung_up.killed = true;
        }
    }

    /// Sets the size and signals the foreground program even when the size is unchanged, so a re-attached
    /// full-screen program redraws.
    fn resize(&self, size: Size) {
        let Some(master) = &self.master else { return };
        let window = winsize(size);
        let fd = master.as_raw_fd();
        // SAFETY: TIOCSWINSZ reads a valid winsize; tcgetpgrp and killpg only query and signal
        unsafe {
            libc::ioctl(fd, libc::TIOCSWINSZ, &window);
            let group = libc::tcgetpgrp(fd);
            if group > 0 {
                libc::killpg(group, libc::SIGWINCH);
            }
        }
    }

    fn send(&mut self, frame: &Frame) {
        let failed = self.client.as_mut().is_some_and(|client| frame.write_to(client).is_err());
        if failed {
            self.client = None;
        }
    }
}

/// The tail of the output. Once trimmed it starts after a newline, so replay does not begin inside an escape
/// sequence (one spanning a newline is rare), and with the last title, which programs set only when it changes.
#[derive(Default)]
struct Replay {
    bytes: VecDeque<u8>,
    trimmed: bool,
    title: TitleTracker,
}

impl Replay {
    fn push(&mut self, data: &[u8]) {
        self.title.scan(data);
        self.bytes.extend(data);
        let excess = self.bytes.len().saturating_sub(REPLAY_BYTES);
        if excess > 0 {
            self.bytes.drain(..excess);
            self.trimmed = true;
        }
    }

    fn snapshot(&self) -> Vec<u8> {
        let bytes: Vec<u8> = self.bytes.iter().copied().collect();
        if self.trimmed {
            let start = bytes.iter().position(|&b| b == b'\n').map_or(bytes.len(), |newline| newline + 1);
            return [self.title.sequence().unwrap_or_default().as_slice(), &bytes[start..]].concat();
        }
        bytes
    }
}

fn signal_groups(groups: &[libc::pid_t], signal: libc::c_int) {
    for &group in groups {
        // SAFETY: killpg only sends a signal; a group already gone just fails
        unsafe { libc::killpg(group, signal) };
    }
}

fn open_pty(size: Size) -> io::Result<(File, OwnedFd)> {
    let (mut master, mut slave) = (-1, -1);
    let mut window = winsize(size);
    // SAFETY: valid out-pointers; name and termios may be null. The window size is a raw `*mut`: macOS's libc takes
    // `*mut winsize`, Linux's `*const`, which a `&mut` would trip clippy on
    check(unsafe { libc::openpty(&mut master, &mut slave, std::ptr::null_mut(), std::ptr::null_mut(), &raw mut window) })?;
    // SAFETY: openpty returned two new descriptors nobody else owns
    let (master, slave) = unsafe { (OwnedFd::from_raw_fd(master), OwnedFd::from_raw_fd(slave)) };
    // the program gets the slave as 0-2 only; it must not inherit either descriptor beyond that
    set_flags(&master, libc::F_SETFD, libc::FD_CLOEXEC)?;
    set_flags(&slave, libc::F_SETFD, libc::FD_CLOEXEC)?;
    set_flags(&master, libc::F_SETFL, libc::O_NONBLOCK)?;
    Ok((File::from(master), slave))
}

fn spawn_program(slave: OwnedFd, config: &HostConfig) -> io::Result<Child> {
    let (program, args) = config
        .command
        .split_first()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, format!("session {}: empty command", config.id)))?;
    let mut command = Command::new(program);
    command
        .args(args)
        .current_dir(&config.cwd)
        .env("TERM", "xterm-256color")
        // the host's own; an IVE started from this shell must pick its registry from its profile
        .env_remove(DIR_ENV)
        .envs(config.env.iter().map(|(key, value)| (key, value)))
        .envs(program_env(std::env::vars_os()));
    for (key, _) in std::env::vars_os().filter(|(key, _)| key.as_encoded_bytes().starts_with(PROGRAM_ENV_PREFIX.as_bytes())) {
        command.env_remove(key);
    }
    command
        .stdin(Stdio::from(slave.try_clone()?))
        .stdout(Stdio::from(slave.try_clone()?))
        .stderr(Stdio::from(slave));
    // SAFETY: only async-signal-safe calls between fork and exec
    unsafe {
        command.pre_exec(|| {
            // the host ignores hang-ups; ignored signals survive exec, so restore the default
            libc::signal(libc::SIGHUP, libc::SIG_DFL);
            check(libc::setsid())?;
            check(libc::ioctl(0, libc::TIOCSCTTY as _, 0))?;
            Ok(())
        })
    };
    command.spawn().map_err(|error| io::Error::new(error.kind(), format!("cannot start {program}: {error}")))
}

/// A host lives long; descriptors its starter leaked to it (sockets, watches, GPU handles) would stay open as long.
#[cfg(target_os = "linux")]
fn close_inherited_descriptors() {
    // SAFETY: closes descriptors 3 and up, none of which this process uses yet
    unsafe { libc::syscall(libc::SYS_close_range, 3, libc::c_uint::MAX, 0) };
}

/// A host lives long; descriptors its starter leaked to it (sockets, watches, GPU handles) would stay open as long.
/// macOS has no close_range; `/dev/fd` lists the open ones, its own listing's descriptor among them.
#[cfg(target_os = "macos")]
fn close_inherited_descriptors() {
    let Ok(entries) = std::fs::read_dir("/dev/fd") else { return };
    let inherited: Vec<RawFd> = entries.filter_map(|entry| entry.ok()?.file_name().to_str()?.parse().ok()).filter(|fd| *fd > 2).collect();
    for fd in inherited {
        // SAFETY: closes descriptors 3 and up, none of which this process uses yet; the listing's is closed already
        unsafe { libc::close(fd) };
    }
}

/// A descriptor that becomes readable when the process exits: the exit hook, no polling.
#[cfg(target_os = "linux")]
fn exit_watch(pid: u32) -> io::Result<OwnedFd> {
    // SAFETY: plain syscall; works on the zombie too if the program already exited, as nobody reaped it yet
    let fd = unsafe { libc::syscall(libc::SYS_pidfd_open, pid as libc::pid_t, 0) };
    check(fd as i32)?;
    // SAFETY: a new descriptor nobody else owns
    Ok(unsafe { OwnedFd::from_raw_fd(fd as RawFd) })
}

/// A descriptor that becomes readable when the process exits: a kqueue watching it, which poll takes like any
/// descriptor.
#[cfg(target_os = "macos")]
fn exit_watch(pid: u32) -> io::Result<OwnedFd> {
    // SAFETY: kqueue takes no arguments; a new descriptor nobody else owns
    let queue = unsafe { OwnedFd::from_raw_fd(check(libc::kqueue())?) };
    let watch = libc::kevent {
        ident: pid as libc::uintptr_t,
        filter: libc::EVFILT_PROC,
        flags: libc::EV_ADD,
        fflags: libc::NOTE_EXIT,
        data: 0,
        udata: std::ptr::null_mut(),
    };
    // SAFETY: one valid change, no events asked for
    let added = unsafe { libc::kevent(queue.as_raw_fd(), &watch, 1, std::ptr::null_mut(), 0, std::ptr::null()) };
    match check(added) {
        Ok(_) => Ok(queue),
        // unlike a pidfd, a kqueue refuses a zombie: the program already exited
        Err(error) if error.raw_os_error() == Some(libc::ESRCH) => already_readable(),
        Err(error) => Err(error),
    }
}

/// The read end of a pipe whose write end is closed: poll reports it at once.
#[cfg(target_os = "macos")]
fn already_readable() -> io::Result<OwnedFd> {
    let mut ends = [-1; 2];
    // SAFETY: a valid out-array of two descriptors
    check(unsafe { libc::pipe(ends.as_mut_ptr()) })?;
    // SAFETY: pipe returned two new descriptors nobody else owns; the write end closes as it drops
    let (read, _write) = unsafe { (OwnedFd::from_raw_fd(ends[0]), OwnedFd::from_raw_fd(ends[1])) };
    Ok(read)
}

fn exit_code(status: ExitStatus) -> i32 {
    status.code().unwrap_or_else(|| 128 + status.signal().unwrap_or(0))
}

fn winsize(size: Size) -> libc::winsize {
    libc::winsize { ws_row: size.lines, ws_col: size.columns, ws_xpixel: 0, ws_ypixel: 0 }
}

fn set_flags(fd: &OwnedFd, command: libc::c_int, flag: libc::c_int) -> io::Result<()> {
    let get = if command == libc::F_SETFD { libc::F_GETFD } else { libc::F_GETFL };
    // SAFETY: fcntl on a descriptor we own
    let current = check(unsafe { libc::fcntl(fd.as_raw_fd(), get) })?;
    check(unsafe { libc::fcntl(fd.as_raw_fd(), command, current | flag) }).map(drop)
}

fn ignore_signal(signal: libc::c_int) {
    // SAFETY: installs SIG_IGN, no handler code runs
    unsafe { libc::signal(signal, libc::SIG_IGN) };
}

fn pollfd(fd: RawFd) -> libc::pollfd {
    libc::pollfd { fd, events: libc::POLLIN, revents: 0 }
}

pub(crate) fn poll(fds: &mut [libc::pollfd], timeout_ms: i32) -> io::Result<()> {
    loop {
        // SAFETY: fds is a valid slice of pollfd
        if unsafe { libc::poll(fds.as_mut_ptr(), fds.len() as libc::nfds_t, timeout_ms) } >= 0 {
            return Ok(());
        }
        let error = io::Error::last_os_error();
        if error.kind() != io::ErrorKind::Interrupted {
            return Err(error);
        }
    }
}

fn check(result: libc::c_int) -> io::Result<libc::c_int> {
    if result == -1 {
        return Err(io::Error::last_os_error());
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_name_host_program_that_cannot_start() {
        // given
        let dir = tempfile::tempdir().unwrap();
        let registry = Registry::at(dir.path().join("r"));
        let launcher = Launcher { exe: PathBuf::from("/no/such/ive"), args: Vec::new() };
        let config = HostConfig { id: "t1".to_owned(), command: vec!["sh".to_owned()], cwd: PathBuf::from("/"), size: Size::DEFAULT, env: Vec::new(), labels: Vec::new() };

        // when
        let error = start(&launcher, &registry, &config).unwrap_err();

        // then
        assert!(error.to_string().starts_with("cannot start the terminal host /no/such/ive: "), "{error}");
    }

    #[test]
    fn should_pass_env_to_the_host_outside_its_command_line() {
        // given
        let config = HostConfig {
            id: "t1".to_owned(),
            command: vec!["sh".to_owned()],
            cwd: PathBuf::from("/"),
            size: Size::DEFAULT,
            env: vec![("SECRET".to_owned(), "s3cret".to_owned())],
            labels: Vec::new(),
        };

        // when
        let args = serve_args(&config);

        // then
        assert!(!args.iter().any(|arg| arg.to_string_lossy().contains("s3cret")), "{args:?}");
        assert_eq!(host_env(&config), [("PTYHOST_ENV_SECRET".to_owned(), "s3cret".to_owned())]);
    }

    #[test]
    fn should_take_prefixed_env_for_the_program() {
        // given
        use std::os::unix::ffi::OsStringExt;
        let vars = [("PTYHOST_ENV_SECRET", "s3cret"), ("PATH", "/bin"), ("PTYHOST_ENV_", "empty name")]
            .map(|(key, value)| (OsString::from(key), OsString::from(value)))
            .into_iter()
            .chain([(OsString::from("PTYHOST_ENV_RAW"), OsString::from_vec(b"\xff".to_vec())), (OsString::from_vec(b"OTHER\xff".to_vec()), OsString::from("x"))]);

        // when
        let env = program_env(vars);

        // then
        assert_eq!(env, [(OsString::from("SECRET"), OsString::from("s3cret")), (OsString::from("RAW"), OsString::from_vec(b"\xff".to_vec()))]);
    }

    #[test]
    fn should_replay_from_after_a_newline_once_trimmed() {
        // given
        let mut replay = Replay::default();
        replay.push(&vec![b'a'; REPLAY_BYTES]);

        // when
        replay.push(b"cut\x1b[1mline\nwhole line\n");

        // then
        assert_eq!(replay.snapshot(), b"whole line\n");
    }

    #[test]
    fn should_replay_the_last_title_once_trimmed_past_it() {
        // given
        let mut replay = Replay::default();
        replay.push(b"\x1b]0;\xe2\x9c\xb3 Summary\x07");

        // when
        replay.push(&vec![b'a'; REPLAY_BYTES]);
        replay.push(b"\nprompt");

        // then
        assert_eq!(replay.snapshot(), b"\x1b]2;\xe2\x9c\xb3 Summary\x07prompt");
    }
}
