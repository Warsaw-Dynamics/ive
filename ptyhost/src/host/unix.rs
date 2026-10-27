//! The Unix host: one poll loop over the PTY, the program's exit, the listener and the client.

use std::collections::VecDeque;
use std::fs::File;
use std::io::{self, Read, Write};
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
use std::os::unix::net::{UnixListener, UnixStream};
use std::os::unix::process::{CommandExt, ExitStatusExt};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use super::{CLIENT_READ_TIMEOUT, CLIENT_WRITE_TIMEOUT, HANG_UP_GRACE, HostConfig, MAX_PENDING_INPUT, Replay, Watchdog, hidden_env, program_env};
use crate::protocol::{Frame, PROTOCOL, Size};
use crate::registry::{Registry, SessionInfo};

/// Runs `command` in its own session.
pub fn spawn_detached(command: &mut Command) -> io::Result<Child> {
    // SAFETY: setsid is async-signal-safe
    unsafe { command.pre_exec(|| check(libc::setsid()).map(drop)) };
    command.spawn()
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
    let watchdog = Watchdog::new(registry, &config.id, &socket)?;
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
    for key in hidden_env(std::env::vars_os()) {
        command.env_remove(key);
    }
    command
        .args(args)
        .current_dir(&config.cwd)
        .env("TERM", "xterm-256color")
        .envs(config.env.iter().map(|(key, value)| (key, value)))
        .envs(program_env(std::env::vars_os()))
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
